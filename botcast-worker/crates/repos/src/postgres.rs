use crate::entities::tasks::{self, Entity as TaskEntity, Model as Task};
use crate::entities::users::{self, Entity as UserEntity, Model as User};
use crate::repo::{TaskRepo, UserRepo};
use crate::{error::Error, id::TaskId};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter,
};
use uuid::Uuid;

pub struct PostgresUserRepo {
    db: DatabaseConnection,
}

impl PostgresUserRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepo for PostgresUserRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<User, Error> {
        let Some(user) = UserEntity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("user".to_string(), id.to_string()));
        };
        Ok(user)
    }

    async fn find_by_auth_id(&self, auth_id: &Uuid) -> anyhow::Result<User, Error> {
        let Some(user) = UserEntity::find()
            .filter(users::Column::AuthId.eq(auth_id.to_string()))
            .one(&self.db)
            .await
            .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("user".to_string(), auth_id.to_string()));
        };
        Ok(user)
    }
}

#[derive(Debug, Clone)]
pub struct PostgresTaskRepo {
    db: DatabaseConnection,
}

impl PostgresTaskRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TaskRepo for PostgresTaskRepo {
    async fn list(&self, id: &Uuid) -> anyhow::Result<Vec<Task>, Error> {
        let tasks = TaskEntity::find()
            .filter(tasks::Column::UserId.eq(*id))
            .all(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(tasks)
    }

    async fn find_by_id(&self, id: &TaskId) -> anyhow::Result<Task, Error> {
        let task = TaskEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("task".to_string(), id.0.to_string()))?;
        Ok(task)
    }

    async fn create(&self, task: Task) -> anyhow::Result<(), Error> {
        TaskEntity::insert(task.into_active_model())
            .exec(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(&self, task: Task) -> anyhow::Result<(), Error> {
        task.into_active_model()
            .save(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn delete(&self, id: &TaskId) -> anyhow::Result<(), Error> {
        let task = TaskEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("task".to_string(), id.0.to_string()))?;
        task.delete(&self.db).await.map_err(Error::Other)?;
        Ok(())
    }
}
