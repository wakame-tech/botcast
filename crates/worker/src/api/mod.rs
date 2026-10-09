use crate::usecase::provider::Provider;
use auth::CmsTokenVerifier;
use router::routers;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

mod auth;
mod error;
mod router;

#[derive(Debug)]
struct AppState {
    provider: Arc<Provider>,
    verifier: CmsTokenVerifier,
}

pub async fn start_api(provider: Arc<Provider>) -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        provider: provider.clone(),
        verifier: CmsTokenVerifier::from_env(),
    });
    // 認証は Authorization ヘッダー（Cookie 不使用）なので、任意のオリジンからの呼び出しを許可する
    let router = routers(provider).with_state(state).layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive()),
    );
    let port = std::env::var("PORT").unwrap_or("9001".to_string());
    tracing::info!("Listen port: {}", port);
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
