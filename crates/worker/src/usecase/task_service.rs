use super::agent_service::AgentService;
use super::episode_service::EpisodeService;
use super::mcp_client::McpClient;
use crate::error::Error;
use crate::worker::use_work_dir;
use anyhow::Context;
use kafru::queue::{Queue, QueueData, QueueListConditions, QueueStatus};
use kafru::task::RecordId;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum Args {
    GenerateAudio {
        episode_id: Uuid,
    },
    GenerateScript {
        episode_id: Uuid,
        prompt: String,
    },
}

/// タスクのエンキューと実行を担当するサービス。
///
/// kafru キューに `Args` を直接 JSON として積み、ジョブハンドラー側で取り出して実行する。
///
/// # タスクフロー
///
/// ## 台本生成 (`GenerateScript`)
///
/// ```mermaid
/// sequenceDiagram
///   participant web as フロントエンド
///   participant wapi as worker API
///   participant kafru as kafru
///   participant worker as worker
///   participant llm as Anthropic API
///   participant cms as botcast-cms MCP
///
///   web->>wapi: POST /createTask (generateScript)
///   wapi->>kafru: push job {args}
///   kafru->>worker: execute job
///   loop エージェントループ
///     worker->>llm: messages + tools
///     llm-->>worker: tool_use
///     worker->>cms: tools/call
///     cms-->>worker: result
///   end
///   llm-->>worker: end_turn
/// ```
#[cfg_attr(doc, aquamarine::aquamarine)]
#[derive(Clone)]
pub(crate) struct TaskService {
    episode_service: EpisodeService,
    kafru_queue: Arc<Queue<'static>>,
}

impl TaskService {
    pub(crate) fn new(
        episode_service: EpisodeService,
        kafru_queue: Arc<Queue<'static>>,
    ) -> Self {
        Self {
            episode_service,
            kafru_queue,
        }
    }

    pub(crate) async fn create_task(&self, args: Args) -> anyhow::Result<(), Error> {
        let mut params = HashMap::new();
        params.insert(
            "args".to_string(),
            serde_json::to_value(&args).map_err(|e| Error::Other(anyhow::anyhow!(e)))?,
        );
        let queue_data = QueueData {
            queue: Some("botcast-worker-default".to_string()),
            name: Some("execute_task".to_string()),
            handler: Some("execute_task".to_string()),
            parameters: Some(params),
            ..Default::default()
        };
        self.kafru_queue
            .push(queue_data)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to enqueue task: {}", e)))?;
        Ok(())
    }

    pub(crate) async fn list_jobs(&self) -> anyhow::Result<Vec<JobInfo>, Error> {
        self.kafru_queue
            .list(QueueListConditions {
                status: Some(vec![
                    QueueStatus::Waiting.to_string(),
                    QueueStatus::InProgress.to_string(),
                    QueueStatus::Error.to_string(),
                    QueueStatus::Completed.to_string(),
                ]),
                queue: Some(vec!["botcast-worker-default".to_string()]),
                limit: Some(100),
            })
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!(e)))?
            .into_iter()
            .map(JobInfo::try_from)
            .collect()
    }

    pub(crate) async fn get_job_status(&self, job_id: &str) -> anyhow::Result<JobInfo, Error> {
        let id: RecordId = job_id
            .parse()
            .map_err(|e| Error::Other(anyhow::anyhow!("invalid job_id: {}", e)))?;
        let data = self
            .kafru_queue
            .get(id)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("job not found: {}", e)))?;
        JobInfo::try_from(data)
    }

    #[instrument(skip(self))]
    pub(crate) async fn execute_args(&self, args: Args) -> anyhow::Result<(), Error> {
        match args {
            Args::GenerateAudio { episode_id } => {
                let work_dir = use_work_dir(&Uuid::new_v4())
                    .context("Failed to create work dir")
                    .map_err(Error::Other)?;
                self.episode_service
                    .generate_audio(&work_dir, &episode_id)
                    .await
            }
            Args::GenerateScript { episode_id, prompt } => {
                self.run_generate_script(&episode_id, &prompt).await?;
                Ok(())
            }
        }
    }

    async fn run_generate_script(
        &self,
        episode_id: &Uuid,
        prompt: &str,
    ) -> anyhow::Result<String, Error> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY is not set")
            .map_err(Error::Other)?;
        let mcp_cmd = std::env::var("MCP_SERVER_CMD").unwrap_or_else(|_| "node".to_string());
        let mcp_args_str = std::env::var("MCP_SERVER_ARGS")
            .context("MCP_SERVER_ARGS is not set")
            .map_err(Error::Other)?;
        let mcp_args: Vec<&str> = mcp_args_str.split_whitespace().collect();

        let mcp_client = McpClient::new(&mcp_cmd, &mcp_args)
            .await
            .context("Failed to create MCP client")
            .map_err(Error::Other)?;

        let agent = AgentService::new(api_key, mcp_client);

        let system = "あなたはポッドキャストの台本を生成するアシスタントです。\
MCPツールを使用してエピソードの台本を生成し、CMSに保存してください。\
台本はsectionsフィールドに格納され、各セクションはSerifSection(speaker, text)またはAudioSection(url, from, to)の形式です。";

        let full_prompt = format!(
            "エピソードID: {}\n\n{}",
            episode_id.hyphenated(),
            prompt
        );

        let result = agent
            .run(system, &full_prompt)
            .await
            .context("Agent failed")
            .map_err(Error::Other)?;

        agent.close().await.map_err(Error::Other)?;

        Ok(result)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct JobInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

impl TryFrom<QueueData> for JobInfo {
    type Error = Error;

    fn try_from(q: QueueData) -> Result<Self, Self::Error> {
        Ok(JobInfo {
            id: q
                .id
                .map(|r| r.to_string())
                .ok_or_else(|| Error::Other(anyhow::anyhow!("missing job id")))?,
            name: q.name.unwrap_or_default(),
            status: q.status.map(|s| s.to_string()).unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn job_info_try_from_missing_id_returns_error() {
        let data = QueueData {
            id: None,
            name: Some("test".to_string()),
            ..Default::default()
        };
        let result = JobInfo::try_from(data);
        assert!(result.is_err());
    }

    #[test]
    fn args_generate_audio_serializes_correctly() {
        let args = Args::GenerateAudio { episode_id: Uuid::nil() };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["type"], "generateAudio");
        assert!(json["episodeId"].is_string());
    }

    #[test]
    fn args_generate_script_serializes_correctly() {
        let args = Args::GenerateScript {
            episode_id: Uuid::nil(),
            prompt: "テスト台本を生成してください".to_string(),
        };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["type"], "generateScript");
        assert_eq!(json["prompt"], "テスト台本を生成してください");
        assert!(json["episodeId"].is_string());
    }

    #[test]
    fn args_deserializes_generate_script() {
        let json = serde_json::json!({
            "type": "generateScript",
            "episodeId": Uuid::nil().to_string(),
            "prompt": "プロンプト"
        });
        let args: Args = serde_json::from_value(json).unwrap();
        assert!(matches!(args, Args::GenerateScript { .. }));
    }
}
