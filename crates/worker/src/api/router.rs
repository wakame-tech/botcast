use super::AppState;
use crate::{error::Error, mcp_server::BotcastMcpServer};
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use rmcp::transport::{
    streamable_http_server::{session::local::LocalSessionManager, StreamableHttpService},
    StreamableHttpServerConfig,
};
use serde_json::json;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip(state))]
async fn list_jobs(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, Error> {
    let jobs = state.0.task_service().list_jobs().await?;
    Ok(Json(jobs))
}

async fn version() -> Result<impl IntoResponse, Error> {
    let worker_version = env!("CARGO_PKG_VERSION");
    Ok(Json(json!({
        "worker": worker_version,
    })))
}

pub(crate) fn routers(provider: Arc<crate::usecase::provider::Provider>) -> Router<Arc<AppState>> {
    let mcp_server = BotcastMcpServer::new(provider);
    let mcp_service = StreamableHttpService::new(
        move || Ok(mcp_server.clone()),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );

    Router::new()
        .route("/version", get(version))
        .route("/jobs", get(list_jobs))
        .nest_service("/mcp", mcp_service)
}
