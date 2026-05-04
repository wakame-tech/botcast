use crate::usecase::provider::Provider;
use crate::usecase::task_service::Args;
use async_trait::async_trait;
use kafru::task::{RecordId, TaskHandler};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

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
        let args_value = params
            .get("args")
            .ok_or("args not found in params")?
            .clone();
        let args: Args = serde_json::from_value(args_value).map_err(|e| e.to_string())?;
        provider
            .task_service()
            .execute_args(args)
            .await
            .map_err(|e| e.to_string())
    }
}
