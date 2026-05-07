use crate::{jobs::execute_task_job, usecase::provider::Provider};
use audio_generator::workdir::WorkDir;
use kafru::{database::Db, manager::Manager, task::TaskRegistry};
use std::sync::Arc;
use uuid::Uuid;

pub(crate) fn use_work_dir(task_id: &Uuid) -> anyhow::Result<WorkDir> {
    let keep = std::env::var("KEEP_WORKDIR")
        .unwrap_or("false".to_string())
        .parse()?;
    WorkDir::new(task_id, keep)
}

pub fn start_worker(provider: Arc<Provider>, kafru_db: Arc<Db>) {
    execute_task_job::init_provider(provider);

    tokio::spawn(async move {
        let mut registry = TaskRegistry::new().await;
        registry
            .register("execute_task".to_string(), execute_task_job::create)
            .await;
        let registry = Arc::new(registry);

        let mut manager = Manager::new("botcast-worker".to_string(), "admin".to_string()).await;
        let _ = manager
            .worker(
                "default".to_string(),
                5,
                registry,
                1,
                Some(kafru_db.clone()),
            )
            .await;
        let _ = manager
            .scheduler("default".to_string(), 1, Some(kafru_db))
            .await;

        tracing::info!("Worker watching with kafru...");
        let _ = manager.wait().await;
    });
}
