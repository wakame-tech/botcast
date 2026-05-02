use sea_orm::DatabaseConnection;

use crate::{postgres::*, r2_storage::R2Storage, repo::{EpisodeRepo, PodcastRepo, ScriptRepo, SecretRepo, TaskRepo}, storage::Storage};
use std::{fmt::Debug, sync::Arc};

pub trait ProvidePodcastRepo: Debug + Send + Sync {
    fn podcast_repo(&self) -> Arc<dyn PodcastRepo>;
}

pub trait ProvideEpisodeRepo: Debug + Send + Sync {
    fn episode_repo(&self) -> Arc<dyn EpisodeRepo>;
}

pub trait ProvideScriptRepo: Debug + Send + Sync {
    fn script_repo(&self) -> Arc<dyn ScriptRepo>;
}

pub trait ProvideTaskRepo: Debug + Send + Sync {
    fn task_repo(&self) -> Arc<dyn TaskRepo>;
}

pub trait ProvideSecretRepo: Debug + Send + Sync {
    fn secret_repo(&self) -> Arc<dyn SecretRepo>;
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

impl ProvidePodcastRepo for DefaultProvider {
    fn podcast_repo(&self) -> Arc<dyn PodcastRepo> {
        Arc::new(PostgresPodcastRepo::new(self.db.clone()))
    }
}

impl ProvideEpisodeRepo for DefaultProvider {
    fn episode_repo(&self) -> Arc<dyn EpisodeRepo> {
        Arc::new(PostgresEpisodeRepo::new(self.db.clone()))
    }
}

impl ProvideScriptRepo for DefaultProvider {
    fn script_repo(&self) -> Arc<dyn ScriptRepo> {
        Arc::new(PostgresScriptRepo::new(self.db.clone()))
    }
}

impl ProvideTaskRepo for DefaultProvider {
    fn task_repo(&self) -> Arc<dyn TaskRepo> {
        Arc::new(PostgresTaskRepo::new(self.db.clone()))
    }
}

impl ProvideSecretRepo for DefaultProvider {
    fn secret_repo(&self) -> Arc<dyn SecretRepo> {
        Arc::new(PostgresSecretRepo::new(self.db.clone()))
    }
}

impl ProvideStorage for DefaultProvider {
    fn storage(&self) -> Arc<dyn Storage> {
        Arc::new(R2Storage::new().expect("Failed to create storage"))
    }
}
