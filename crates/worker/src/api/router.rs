use super::AppState;
use crate::{
    error::Error,
    usecase::task_service::Args,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip(state))]
async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(args): Json<Args>,
) -> Result<impl IntoResponse, Error> {
    state.0.task_service().create_task(args).await?;
    Ok(StatusCode::CREATED)
}

async fn version() -> Result<impl IntoResponse, Error> {
    let worker_version = env!("CARGO_PKG_VERSION");
    Ok(Json(json!({
        "worker": worker_version,
    })))
}

pub(crate) fn routers() -> Router<Arc<AppState>> {
    Router::new()
        .route("/version", get(version))
        .route("/createTask", post(create_task))
}
