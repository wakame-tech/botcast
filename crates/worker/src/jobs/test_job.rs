// @specre 01KNM2BBT5YGP4741F2QK6JNKE
use async_trait::async_trait;
use kafru::task::{RecordId, TaskHandler};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;

pub(crate) struct TestJob;

#[async_trait]
impl TaskHandler for TestJob {
    async fn run(
        &self,
        params: HashMap<String, Value>,
        queue_id: Option<RecordId>,
        agent_id: Option<RecordId>,
    ) -> Result<(), String> {
        sleep(Duration::from_secs(5)).await;
        println!("[{:?}, {:?}] params: {:?}", queue_id, agent_id, params);
        Ok(())
    }
}
