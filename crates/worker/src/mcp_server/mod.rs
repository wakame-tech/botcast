use crate::usecase::provider::Provider;
use crate::usecase::task_service::Args;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Extensions, Implementation, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::Instrument;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateAudioParams {
    /// botcast-cms の episodes レコード ID
    pub episode_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateScriptParams {
    /// botcast-cms の episodes レコード ID
    pub episode_id: String,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateEpisodeParams {
    /// botcast-cms の podcasts レコード ID (schedule.script_id が必要)
    pub podcast_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetJobStatusParams {
    pub job_id: String,
}

/// `/mcp` の HTTP リクエストの span。API のルーターがリクエストの extensions に入れる
#[derive(Clone)]
pub(crate) struct RequestSpan(pub(crate) tracing::Span);

/// ツールの呼び出しの span。HTTP リクエストの span の子にする
/// （rmcp はハンドラーを別のタスクで動かすので、HTTP の span がそのままでは引き継がれない）
fn tool_span(name: &'static str, extensions: &Extensions) -> tracing::Span {
    let parent = extensions
        .get::<axum::http::request::Parts>()
        .and_then(|parts| parts.extensions.get::<RequestSpan>())
        .map(|span| span.0.clone())
        .unwrap_or_else(tracing::Span::none);
    tracing::info_span!(
        parent: &parent,
        "mcp.tool_call",
        otel.name = format!("mcp.tool_call {name}"),
        mcp.tool.name = name,
    )
}

#[derive(Clone)]
pub struct BotcastMcpServer {
    provider: Arc<Provider>,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for BotcastMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default().with_server_info(Implementation::new(
            "botcast",
            env!("CARGO_PKG_VERSION"),
        ))
    }
}

impl BotcastMcpServer {
    /// ジョブを積み、結果をツールの応答（JSON の文字列）にする
    async fn enqueue(&self, args: Args, span: tracing::Span) -> String {
        match self
            .provider
            .task_service()
            .create_task(args)
            .instrument(span)
            .await
        {
            Ok(()) => serde_json::json!({ "status": "enqueued" }).to_string(),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }
}

#[tool_router(router = tool_router)]
impl BotcastMcpServer {
    pub fn new(provider: Arc<Provider>) -> Self {
        Self {
            provider,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "エピソードの音声を生成するジョブをエンキューします")]
    pub async fn generate_audio(
        &self,
        params: Parameters<GenerateAudioParams>,
        extensions: Extensions,
    ) -> String {
        let args = Args::GenerateAudio {
            episode_id: params.0.episode_id,
        };
        self.enqueue(args, tool_span("generate_audio", &extensions))
            .await
    }

    #[tool(description = "エピソードの台本を生成するジョブをエンキューします")]
    pub async fn generate_script(
        &self,
        params: Parameters<GenerateScriptParams>,
        extensions: Extensions,
    ) -> String {
        let args = Args::GenerateScript {
            episode_id: params.0.episode_id,
            prompt: params.0.prompt,
            generate_audio: false,
        };
        self.enqueue(args, tool_span("generate_script", &extensions))
            .await
    }

    #[tool(description = "番組の定期実行の設定（スクリプトまたは LLM）で新しいエピソードを作り、音声まで生成するジョブをエンキューします")]
    pub async fn generate_episode(
        &self,
        params: Parameters<GenerateEpisodeParams>,
        extensions: Extensions,
    ) -> String {
        let args = Args::GenerateEpisode {
            podcast_id: params.0.podcast_id,
        };
        self.enqueue(args, tool_span("generate_episode", &extensions))
            .await
    }

    #[tool(description = "ジョブ一覧を取得します")]
    pub async fn list_jobs(&self) -> String {
        match self.provider.task_service().list_jobs().await {
            Ok(jobs) => serde_json::to_string(&jobs).unwrap_or_else(|e| e.to_string()),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    #[tool(description = "ジョブの状態を取得します")]
    pub async fn get_job_status(&self, params: Parameters<GetJobStatusParams>) -> String {
        match self
            .provider
            .task_service()
            .get_job_status(&params.0.job_id)
            .await
        {
            Ok(info) => serde_json::to_string(&info).unwrap_or_else(|e| e.to_string()),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }
}
