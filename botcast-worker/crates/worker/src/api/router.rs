use super::AppState;
use crate::{
    error::Error,
    usecase::{task_service::Args, Provider, UserApiClientProvider},
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tracing::instrument;

fn with_user_api_client(provider: &Provider, token: Option<String>) -> Provider {
    Provider {
        provide_api_client: Arc::new(UserApiClientProvider::new(token)),
        ..provider.clone()
    }
}

fn get_authorization(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}

#[instrument(skip(state))]
async fn create_task(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(args): Json<Args>,
) -> Result<impl IntoResponse, Error> {
    let provider = with_user_api_client(&state.0, get_authorization(&headers));

    provider.task_service().create_task(args).await?;
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
