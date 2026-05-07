use crate::usecase::provider::Provider;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateAudioParams {
    pub episode_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateScriptParams {
    pub episode_id: Uuid,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetJobStatusParams {
    pub job_id: String,
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

#[tool_router(router = tool_router)]
impl BotcastMcpServer {
    pub fn new(provider: Arc<Provider>) -> Self {
        Self {
            provider,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "エピソードの音声を生成するジョブをエンキューします")]
    pub async fn generate_audio(&self, params: Parameters<GenerateAudioParams>) -> String {
        match self
            .provider
            .task_service()
            .create_task(crate::usecase::task_service::Args::GenerateAudio {
                episode_id: params.0.episode_id,
            })
            .await
        {
            Ok(()) => serde_json::json!({ "status": "enqueued" }).to_string(),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    #[tool(description = "エピソードの台本を生成するジョブをエンキューします")]
    pub async fn generate_script(&self, params: Parameters<GenerateScriptParams>) -> String {
        match self
            .provider
            .task_service()
            .create_task(crate::usecase::task_service::Args::GenerateScript {
                episode_id: params.0.episode_id,
                prompt: params.0.prompt.clone(),
            })
            .await
        {
            Ok(()) => serde_json::json!({ "status": "enqueued" }).to_string(),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
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
