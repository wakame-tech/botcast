// @specre 01KNM2BBT5S5Z8EZ77C6CP55GB
mod auth;
mod error;
mod middleware;
mod openapi_adapter;
mod repository;

use crate::{openapi_adapter::DifySandboxClient, repository::CollectionRepository};
use repository::surrealdb::SurrealCollectionRepository;
use serde_json::json;
use std::{env, sync::Arc};
use surrealdb::{Surreal, engine::remote::ws::Client};
use tower_http::cors::CorsLayer;
use worker::BotcastWorker;

#[derive(Clone)]
pub struct AppState {
    pub dify_sandbox_client: Arc<DifySandboxClient>,
    pub worker: Arc<BotcastWorker>,
    pub collection_repo: Arc<dyn CollectionRepository>,
    pub db: Surreal<Client>,
    pub jwt_secret: String,
    pub namespace: String,
    pub database: String,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish()
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let worker = BotcastWorker::new()
        .await
        .expect("Failed to create BotcastWorker");
    let worker = Arc::new(worker);
    worker.launch().await;

    let collection_repo = SurrealCollectionRepository::try_from_env()
        .await
        .expect("Failed to initialize repository");
    let db = collection_repo.db.clone();

    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        eprintln!("Warning: JWT_SECRET not set, using default (unsafe for production)");
        "default-jwt-secret-change-me".to_string()
    });
    let namespace = env::var("DATABASE_NAMESPACE").expect("DATABASE_NAMESPACE must be set");
    let database = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");

    let state = AppState {
        dify_sandbox_client: Arc::new(
            DifySandboxClient::try_new().expect("Failed to create DifySandboxClient"),
        ),
        worker: worker.clone(),
        collection_repo: Arc::new(collection_repo),
        db,
        jwt_secret,
        namespace,
        database,
    };

    seed_collections(&*state.collection_repo).await;
    start_http_server(state).await;
}

async fn seed_collections(repo: &dyn CollectionRepository) {
    let collections = [
        (
            "podcasts",
            json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "icon": { "type": "string" },
                    "description": { "type": "string" },
                    "user_id": { "type": "string" }
                },
                "required": ["title", "icon", "user_id"]
            }),
        ),
        (
            "episodes",
            json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "podcast_id": { "type": "string" },
                    "description": { "type": "string" },
                    "audio_url": { "type": "string" },
                    "srt_url": { "type": "string" },
                    "sections": { "type": "array" },
                    "duration_sec": { "type": "integer" },
                    "user_id": { "type": "string" }
                },
                "required": ["title", "podcast_id", "user_id"]
            }),
        ),
        (
            "scripts",
            json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "description": { "type": "string" },
                    "template": { "type": "string" },
                    "arguments": { "type": "object" },
                    "user_id": { "type": "string" }
                },
                "required": ["title", "user_id"]
            }),
        ),
    ];

    for (name, schema) in &collections {
        match repo.get_by_name(name).await {
            Ok(_) => {
                println!("Collection '{}' already exists, skipping", name);
            }
            Err(_) => match repo.create(name, schema.clone()).await {
                Ok(c) => println!("Created collection '{}' (id: {})", name, c.id),
                Err(e) => eprintln!("Failed to create collection '{}': {}", name, e),
            },
        }
    }
}

async fn start_http_server(state: AppState) {
    let api_impl = openapi_adapter::ApiImpl::new(state.clone());

    let auth_routes = axum::Router::new()
        .route(
            "/auth/signup",
            axum::routing::post(auth::sign_up_handler),
        )
        .route(
            "/auth/signin",
            axum::routing::post(auth::sign_in_handler),
        )
        .with_state(state.clone());

    let custom_routes = axum::Router::new()
        .route(
            "/records/{collectionId}/with-relations",
            axum::routing::get(openapi_adapter::records_with_relations_handler),
        )
        .with_state(api_impl.clone());

    let protected_routes = openapi::server::new(api_impl)
        .merge(custom_routes)
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let app = protected_routes
        .merge(auth_routes)
        .layer(CorsLayer::permissive());

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3002".to_string());
    let bind_address = format!("{}:{}", host, port);

    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    println!("HTTP Server running on http://{}", bind_address);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
