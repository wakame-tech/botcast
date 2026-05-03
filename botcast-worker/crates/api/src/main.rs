mod controller;

use controller::ApiImpl;
use openapi::server::new;
use repos::{Database, postgres::PostgresUserRepo};
use std::sync::Arc;
use supabase_auth::models::AuthClient;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let database_url = std::env::var("DATABASE_URL")?;
    let db = Database::connect(&database_url).await?;

    let api = ApiImpl {
        auth_client: AuthClient::new_from_env()?,
        user_repo: Arc::new(PostgresUserRepo::new(db.clone())),
    };
    let router = new(api).layer(CorsLayer::permissive());
    let listener = TcpListener::bind("0.0.0.0:1234").await.unwrap();
    axum::serve(listener, router).await.unwrap();
    Ok(())
}
