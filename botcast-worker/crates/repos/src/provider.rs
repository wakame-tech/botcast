use sea_orm::DatabaseConnection;

use crate::{postgres::*, r2_storage::R2Storage, repo::TaskRepo, storage::Storage};
use std::{fmt::Debug, sync::Arc};

pub trait ProvideTaskRepo: Debug + Send + Sync {
    fn task_repo(&self) -> Arc<dyn TaskRepo>;
}

pub trait ProvideStorage: Debug + Send + Sync {
    fn storage(&self) -> Arc<dyn Storage>;
}

#[derive(Debug, Clone)]
pub struct DefaultProvider {
    db: DatabaseConnection,
}

impl DefaultProvider {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ProvideTaskRepo for DefaultProvider {
    fn task_repo(&self) -> Arc<dyn TaskRepo> {
        Arc::new(PostgresTaskRepo::new(self.db.clone()))
    }
}

impl ProvideStorage for DefaultProvider {
    fn storage(&self) -> Arc<dyn Storage> {
        Arc::new(R2Storage::new().expect("Failed to create storage"))
    }
}
