// @specre 01KNM2BBT5YGP4741F2QK6JNKE
mod jobs;

use crate::jobs::test_job::TestJob;
use kafru::{
    database::Db,
    manager::Manager,
    queue::{Queue, QueueData, QueueListConditions, QueueStatus},
    task::TaskRegistry,
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::task::JoinHandle;

pub struct BotcastJob {
    pub name: String,
    pub params: HashMap<String, Value>,
    pub status: String,
}

pub struct BotcastWorker {
    server: String,
    task_registry: Arc<TaskRegistry>,
    queue: Arc<Queue<'static>>,
    db: Arc<Db>,
}

impl TryInto<BotcastJob> for QueueData {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<BotcastJob, Self::Error> {
        Ok(BotcastJob {
            name: self.name.unwrap(),
            params: self
                .parameters
                .ok_or(anyhow::anyhow!("Job parameters missing"))?,
            status: self
                .status
                .map(|s| s.to_string())
                .ok_or(anyhow::anyhow!("Job status missing"))?,
        })
    }
}

impl BotcastWorker {
    pub async fn new() -> anyhow::Result<Self> {
        let server = "botcast-worker".to_string();
        let db = Db::new(None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;
        let db = Arc::new(db);

        let mut task_registry = TaskRegistry::new().await;
        task_registry
            .register("testjob".to_string(), || Box::new(TestJob))
            .await;

        Ok(Self {
            server,
            task_registry: Arc::new(task_registry),
            queue: Arc::new(Queue::new(Some(db.clone())).await),
            db,
        })
    }

    pub async fn launch(&self) -> JoinHandle<()> {
        let mut manager = Manager::new(self.server.clone(), "admin".to_string()).await;
        let _ = manager
            .worker(
                "default".to_string(),
                5,
                self.task_registry.clone(),
                1,
                Some(self.db.clone()),
            )
            .await;

        let _ = manager
            .scheduler("default".to_string(), 1, Some(self.db.clone()))
            .await;

        tokio::spawn(async move {
            println!("Worker watching...");
            let _ = manager.wait().await;
        })
    }

    pub async fn list_jobs(&self) -> anyhow::Result<Vec<BotcastJob>> {
        let jobs = self
            .queue
            .list(QueueListConditions {
                status: Some(vec![
                    QueueStatus::Waiting.to_string(),
                    QueueStatus::InProgress.to_string(),
                    QueueStatus::Error.to_string(),
                    QueueStatus::Completed.to_string(),
                ]),
                queue: Some(vec![format!("{}-default", self.server)]),
                limit: Some(100),
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list jobs: {}", e))?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<BotcastJob>>>()?;
        Ok(jobs)
    }

    pub async fn enqueue_job(
        &self,
        name: String,
        params: HashMap<String, Value>,
    ) -> anyhow::Result<()> {
        let queue_data = QueueData {
            queue: Some(format!("{}-default", self.server)),
            name: Some(name),
            handler: Some("testjob".to_string()),
            parameters: Some(params),
            ..Default::default()
        };
        self.queue
            .push(queue_data)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to enqueue job: {}", e))?;
        Ok(())
    }
}
