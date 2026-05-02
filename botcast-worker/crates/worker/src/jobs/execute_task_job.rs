use crate::usecase::provider::Provider;
use async_trait::async_trait;
use kafru::task::{RecordId, TaskHandler};
use repos::id::TaskId;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use uuid::Uuid;

static PROVIDER: OnceLock<Arc<Provider>> = OnceLock::new();

pub(crate) fn init_provider(provider: Arc<Provider>) {
    let _ = PROVIDER.set(provider);
}

pub(crate) fn create() -> Box<dyn TaskHandler> {
    Box::new(ExecuteTaskJob)
}

pub(crate) struct ExecuteTaskJob;

#[async_trait]
impl TaskHandler for ExecuteTaskJob {
    async fn run(
        &self,
        params: HashMap<String, Value>,
        _queue_id: Option<RecordId>,
        _agent_id: Option<RecordId>,
    ) -> Result<(), String> {
        let provider = PROVIDER.get().ok_or("Provider not initialized")?;
        let task_id = params
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or("task_id not found in params")?;
        let task_id = Uuid::parse_str(task_id).map_err(|e| e.to_string())?;
        provider
            .task_service()
            .execute_by_id(&TaskId(task_id))
            .await
            .map_err(|e| e.to_string())
    }
}
