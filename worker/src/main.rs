use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use scrape::ScrapeEpisode;
use std::{sync::LazyLock, time::Duration};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};
use task::{Task, TaskRepo};

mod scrape;
mod synthesis;
mod task;
mod voicevox_client;

#[derive(Debug, Clone)]
struct Ctx {
    db: Surreal<Db>,
}

impl Ctx {
    fn task_repo(&self) -> TaskRepo {
        TaskRepo {
            db: self.db.clone(),
        }
    }
}

static DB: LazyLock<Surreal<Db>> = LazyLock::new(|| Surreal::init());

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", self.0)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Debug, serde::Deserialize)]
struct CreateTaskBody {
    url: String,
}

async fn list_task(State(ctx): State<Ctx>) -> Result<Json<Vec<Task>>, AppError> {
    let repo = ctx.task_repo();
    let tasks = repo.list().await?;
    Ok(Json(tasks))
}

async fn create_task(
    State(db): State<Ctx>,
    Json(body): Json<CreateTaskBody>,
) -> Result<StatusCode, AppError> {
    let repo = db.task_repo();
    let task_id = TaskRepo::new_id();
    let scrape = ScrapeEpisode::new(task_id, body.url);
    repo.create(Task::Scrape(scrape)).await?;
    Ok(StatusCode::CREATED)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    DB.connect::<Mem>(()).await?;
    DB.use_ns("default").use_db("database").await?;

    tokio::spawn(async move {
        let interval = Duration::from_secs(5);
        loop {
            log::info!("Watching tasks...");
            let repo = TaskRepo { db: DB.clone() };
            if let Err(e) = repo.watch().await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });

    let app = Router::new()
        .route("/", get(list_task))
        .route("/scripts", post(create_task))
        .with_state(Ctx { db: DB.clone() });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9999").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
