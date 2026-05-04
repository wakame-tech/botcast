use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn get_presign_url(&self, path: &str, expiration: Duration) -> anyhow::Result<String>;
    async fn upload(&self, path: &str, data: &[u8], content_type: &str) -> anyhow::Result<()>;
}
