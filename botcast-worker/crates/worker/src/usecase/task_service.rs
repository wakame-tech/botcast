use super::agent_service::AgentService;
use super::episode_service::EpisodeService;
use super::mcp_client::McpClient;
use crate::error::Error;
use crate::worker::use_work_dir;
use anyhow::Context;
use chrono::{DateTime, FixedOffset, Utc};
use kafru::queue::{Queue, QueueData};
use openapi_client::apis::auth_api::me_get;
use openapi_client::apis::configuration::Configuration;
use repos::entities::sea_orm_active_enums::TaskStatus;
use repos::entities::tasks::Model as Task;
use repos::id::TaskId;
use repos::repo::TaskRepo;
use std::collections::HashMap;
use std::str::FromStr;
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

pub(crate) fn new_task(
    user_id: Option<Uuid>,
    cron: Option<String>,
    args: Args,
    execute_after: DateTime<FixedOffset>,
) -> Task {
    Task {
        id: Uuid::new_v4(),
        user_id,
        status: TaskStatus::Pending,
        cron,
        args: serde_json::to_value(args).unwrap(),
        result: None,
        execute_after,
        executed_at: None,
        executed_finished_at: None,
    }
}

#[derive(Clone)]
pub(crate) struct TaskService {
    task_repo: Arc<dyn TaskRepo>,
    configuration: Configuration,
    episode_service: EpisodeService,
    kafru_queue: Arc<Queue<'static>>,
}

impl TaskService {
    pub(crate) fn new(
        task_repo: Arc<dyn TaskRepo>,
        configuration: Configuration,
        episode_service: EpisodeService,
        kafru_queue: Arc<Queue<'static>>,
    ) -> Self {
        Self {
            task_repo,
            configuration,
            episode_service,
            kafru_queue,
        }
    }

    #[instrument(skip(self))]
    async fn execute(&self, task: &Task) -> anyhow::Result<serde_json::Value, Error> {
        let args: Args = serde_json::from_value(task.args.clone())
            .map_err(|e| Error::InvalidInput(anyhow::anyhow!("Args {}", e)))?;

        if let Some(cron) = &task.cron {
            let next = cron::Schedule::from_str(cron)
                .context("Invalid cron")
                .map_err(Error::Other)?
                .upcoming(Utc)
                .next()
                .context("Failed to get next cron")
                .map_err(Error::Other)?;
            let task = new_task(
                task.user_id,
                Some(cron.to_string()),
                args.clone(),
                next.into(),
            );
            self.task_repo.create(task).await?;
        }

        match args {
            Args::GenerateAudio { episode_id } => {
                let work_dir = use_work_dir(&task.id)
                    .context("Failed to create work dir")
                    .map_err(Error::Other)?;
                self.episode_service
                    .generate_audio(&work_dir, &episode_id)
                    .await?;
                Ok(serde_json::Value::String("OK".to_string()))
            }
            Args::GenerateScript { episode_id, prompt } => {
                let result = self.run_generate_script(&episode_id, &prompt).await?;
                Ok(serde_json::Value::String(result))
            }
        }
    }

    async fn run_task(&self, mut task: Task) -> anyhow::Result<(), Error> {
        task.status = TaskStatus::Running;
        task.executed_at = Some(Utc::now().into());
        self.task_repo.update(task.clone()).await?;
        (task.status, task.result) = match self.execute(&task).await {
            Ok(result) => (TaskStatus::Completed, Some(result)),
            Err(e) => (
                TaskStatus::Failed,
                Some(serde_json::Value::String(e.to_string())),
            ),
        };
        task.executed_finished_at = Some(Utc::now().into());
        self.task_repo.update(task.clone()).await?;
        tracing::info!("task: {} completed", task.id);
        Ok(())
    }

    pub(crate) async fn create_task(&self, args: Args) -> anyhow::Result<(), Error> {
        let user = me_get(&self.configuration)
            .await
            .context("Failed to get user")
            .map_err(Error::Other)?;
        let task = new_task(Some(user.id), None, args, Utc::now().into());
        let task_id = task.id;
        self.task_repo.create(task).await?;

        let mut params = HashMap::new();
        params.insert(
            "task_id".to_string(),
            serde_json::Value::String(task_id.to_string()),
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
            .unwrap_or_else(|_| "/Users/kmt/dev/botcast-cms/mcp/build/index.js".to_string());
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

    pub(crate) async fn execute_by_id(&self, task_id: &TaskId) -> anyhow::Result<(), Error> {
        let task = self.task_repo.find_by_id(task_id).await?;
        tracing::info!("Executing task: {} args={}", task.id, task.args);
        self.run_task(task).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

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
