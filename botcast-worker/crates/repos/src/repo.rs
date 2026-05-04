use crate::{
    entities::tasks::Model as Task,
    entities::users::Model as User,
    error::Error,
    id::TaskId,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<User, Error>;
    async fn find_by_auth_id(&self, auth_id: &Uuid) -> anyhow::Result<User, Error>;
}

#[async_trait]
pub trait TaskRepo: Send + Sync {
    async fn list(&self, user_id: &Uuid) -> anyhow::Result<Vec<Task>, Error>;
    async fn find_by_id(&self, id: &TaskId) -> anyhow::Result<Task, Error>;
    async fn create(&self, task: Task) -> anyhow::Result<(), Error>;
    async fn update(&self, task: Task) -> anyhow::Result<(), Error>;
    #[allow(dead_code)]
    async fn delete(&self, id: &TaskId) -> anyhow::Result<(), Error>;
}
