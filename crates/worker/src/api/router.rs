use super::auth::{bearer_token, TokenStatus};
use super::AppState;
use crate::{error::Error, mcp_server::BotcastMcpServer, usecase::task_service::Args};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use rmcp::transport::{
    streamable_http_server::{session::local::LocalSessionManager, StreamableHttpService},
    StreamableHttpServerConfig,
};
use serde_json::json;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip(state))]
async fn list_jobs(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, Error> {
    let jobs = state.provider.task_service().list_jobs().await?;
    Ok(Json(jobs))
}

/// web から `Args` を受け取りジョブを投入する。
///
/// botcast-cms の JWT (`Authorization: Bearer`) を CMS に問い合わせて検証してから、body を解釈する。
#[instrument(skip(state, headers, body))]
async fn create_job(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, Error> {
    let Some(token) = bearer_token(&headers) else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };
    if state.verifier.verify(token).await.map_err(Error::Other)? == TokenStatus::Invalid {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
    let args: Args = match serde_json::from_slice(&body) {
        Ok(args) => args,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, e.to_string()).into_response()),
    };
    state.provider.task_service().create_task(args).await?;
    Ok(Json(json!({ "status": "enqueued" })).into_response())
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
        .route("/jobs", get(list_jobs).post(create_job))
        .nest_service("/mcp", mcp_service)
}
