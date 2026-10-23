use super::auth::{bearer_token, CallerResolver, TokenStatus};
use super::AppState;
use crate::{
    error::Error,
    mcp_server::{BotcastMcpServer, RequestSpan},
    usecase::{
        task_service::Args,
        voice_preview::{PreviewError, PreviewRequest, VoicePreviewer},
    },
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rmcp::transport::{
    streamable_http_server::{session::local::LocalSessionManager, StreamableHttpService},
    StreamableHttpServerConfig,
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceBuilder;
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

/// 声の試聴を作る（`POST /voices/{voice_id}/previews`）。
///
/// `POST /jobs` と同じく botcast-cms の JWT（`Authorization: Bearer`）を CMS に問い合わせて検証し、
/// 呼べるのは声の持ち主だけ。音声は Base64 で返し、保存しない。
#[instrument(skip(state, headers, body))]
async fn create_voice_previews(
    State(state): State<Arc<AppState>>,
    Path(voice_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let provider = &state.provider;
    let previewer = VoicePreviewer {
        store: provider.cms.as_ref(),
        tts: &provider.tts,
        engines: &provider.engines,
        namespace: &provider.tenant,
        ref_audios: &state.ref_audios,
    };
    handle_voice_previews(&state.verifier, &previewer, &headers, &voice_id, &body).await
}

/// [`create_voice_previews`] の本体（テストでフェイクを差し込めるようにする）
pub(crate) async fn handle_voice_previews(
    callers: &dyn CallerResolver,
    previewer: &VoicePreviewer<'_>,
    headers: &HeaderMap,
    voice_id: &str,
    body: &[u8],
) -> Response {
    let Some(token) = bearer_token(headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let caller = match callers.current_user(token).await {
        Ok(Some(caller)) => caller,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => return (StatusCode::BAD_GATEWAY, format!("{e:#}")).into_response(),
    };
    // body は省略できる（既定の本文・テイク数）
    let req: PreviewRequest = if body.iter().all(u8::is_ascii_whitespace) {
        PreviewRequest::default()
    } else {
        match serde_json::from_slice(body) {
            Ok(req) => req,
            Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        }
    };
    match previewer.generate(&caller, voice_id, &req).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => {
            let status = match &e {
                PreviewError::BadRequest(_) => StatusCode::BAD_REQUEST,
                PreviewError::NotFound => StatusCode::NOT_FOUND,
                PreviewError::Forbidden => StatusCode::FORBIDDEN,
                PreviewError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
                PreviewError::Timeout => StatusCode::GATEWAY_TIMEOUT,
                PreviewError::Other(_) => StatusCode::BAD_GATEWAY,
            };
            let message = match &e {
                PreviewError::Other(e) => format!("{e:#}"),
                e => e.to_string(),
            };
            (status, Json(json!({ "error": message }))).into_response()
        }
    }
}

async fn version() -> Result<impl IntoResponse, Error> {
    let worker_version = env!("CARGO_PKG_VERSION");
    Ok(Json(json!({
        "worker": worker_version,
    })))
}

/// 今の span（TraceLayer の HTTP の span）をリクエストの extensions に入れる
fn insert_request_span<B>(mut req: Request<B>) -> Request<B> {
    req.extensions_mut()
        .insert(RequestSpan(tracing::Span::current()));
    req
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
        .route("/voices/:voice_id/previews", post(create_voice_previews))
        // rmcp はツールのハンドラーを別のタスクで動かし、HTTP の span が引き継がれない。
        // HTTP の span をリクエストに持たせ、ツール側でその子の span を作る
        .nest_service(
            "/mcp",
            ServiceBuilder::new()
                .map_request(insert_request_span::<axum::body::Body>)
                .service(mcp_service),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usecase::voice_preview::{test_support::*, DEFAULT_PREVIEW_TAKES};
    use axum::http::{header::AUTHORIZATION, HeaderValue};

    /// トークン `good` は [`OWNER`]、`other` は別のユーザー、それ以外は無効
    struct FakeCallers;

    #[async_trait::async_trait]
    impl CallerResolver for FakeCallers {
        async fn current_user(&self, token: &str) -> anyhow::Result<Option<String>> {
            Ok(match token {
                "good" => Some(OWNER.to_string()),
                "other" => Some("user:other".to_string()),
                _ => None,
            })
        }
    }

    fn headers(token: Option<&str>) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Some(token) = token {
            h.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            );
        }
        h
    }

    async fn call(
        token: Option<&str>,
        voice_id: &str,
        body: &str,
    ) -> (StatusCode, serde_json::Value) {
        let fixture = Fixture::new();
        let res = handle_voice_previews(
            &FakeCallers,
            &fixture.previewer(),
            &headers(token),
            voice_id,
            body.as_bytes(),
        )
        .await;
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        )
    }

    #[tokio::test]
    async fn previews_require_a_valid_token() {
        assert_eq!(call(None, "v1", "").await.0, StatusCode::UNAUTHORIZED);
        assert_eq!(
            call(Some("bad"), "v1", "").await.0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn previews_are_only_for_the_owner() {
        // 他人の声は CMS が見せない
        assert_eq!(call(Some("other"), "v1", "").await.0, StatusCode::NOT_FOUND);
        let (status, body) = call(Some("good"), "v1", "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            body["takes"].as_array().unwrap().len(),
            DEFAULT_PREVIEW_TAKES as usize
        );
        assert!(body["takes"][0]["seed"].is_i64());
        assert_eq!(body["takes"][0]["contentType"], "audio/wav");
        assert!(body["takes"][0]["audioBase64"].is_string());
    }

    #[tokio::test]
    async fn previews_validate_the_body() {
        assert_eq!(
            call(Some("good"), "v1", r#"{"count": 0}"#).await.0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(Some("good"), "v1", r#"{"count": 5}"#).await.0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(Some("good"), "v1", r#"{"unknown": 1}"#).await.0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(Some("good"), "v1", "not json").await.0,
            StatusCode::BAD_REQUEST
        );
        let (status, body) = call(Some("good"), "v1", r#"{"text": "やあ。", "count": 2}"#).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["text"], "やあ。");
        assert_eq!(body["takes"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn previews_report_unconfigured_engine() {
        let (status, body) = call(Some("good"), "v3", "").await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(body["error"].as_str().unwrap().contains("not configured"));
    }
}
