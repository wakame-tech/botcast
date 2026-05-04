//! API クレート — OpenAPI 生成の Axum サーバー。
//!
//! Supabase Auth JWT を検証し、`/me`, `/signIn`, `/signUp` エンドポイントを提供する。
//! ポート `1234` で起動する。

mod controller;

use controller::ApiImpl;
use openapi::server::new;
use supabase_auth::models::AuthClient;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let api = ApiImpl {
        auth_client: AuthClient::new_from_env()?,
    };
    let router = new(api).layer(CorsLayer::permissive());
    let listener = TcpListener::bind("0.0.0.0:1234").await.unwrap();
    axum::serve(listener, router).await.unwrap();
    Ok(())
}
