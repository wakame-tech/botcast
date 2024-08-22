use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::{sync::LazyLock, time::Duration};
use surrealdb::{
    engine::local::{Db, Mem},
    opt::RecordId,
    Surreal,
};

#[derive(Debug, Clone)]
struct Ctx(Surreal<Db>);

static DB: LazyLock<Surreal<Db>> = LazyLock::new(|| Surreal::init());

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Task {
    id: RecordId,
}

impl Task {
    fn new(id: usize) -> Self {
        Self {
            id: RecordId::from(("task".to_string(), format!("{}", id))),
        }
    }
}

async fn list_task(State(db): State<Ctx>) -> Json<Vec<Task>> {
    let tasks: Vec<Task> = db.0.select("tasks").await.unwrap();
    return Json(tasks);
}

async fn create_task(State(db): State<Ctx>) -> impl IntoResponse {
    let len = db.0.select::<Vec<Task>>("tasks").await.unwrap().len();
    let task = Task::new(len);
    db.0.create::<Vec<Task>>("tasks")
        .content(task)
        .await
        .unwrap();
    StatusCode::CREATED
}

async fn watch_tasks() -> anyhow::Result<()> {
    let db = DB.clone();
    loop {
        println!("Watching tasks...");
        let tasks: Vec<Task> = db.select("tasks").await?;
        if let Some(task) = tasks.first() {
            let res: Option<Task> = db.delete(&task.id).await?;
            println!("Deleted: {:?}", res);
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    DB.connect::<Mem>(()).await?;
    DB.use_ns("default").use_db("database").await?;

    tokio::spawn(watch_tasks());

    let app = Router::new()
        .route("/", get(list_task))
        .route("/", post(create_task))
        .with_state(Ctx(DB.clone()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9999").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
