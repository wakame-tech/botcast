use super::{
    episode_service::EpisodeService, task_service::TaskService,
    ProvideApiClient, UserApiClientProvider,
};
use kafru::queue::Queue;
use repos::provider::*;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Provider {
    pub(crate) provide_task_repo: Arc<dyn ProvideTaskRepo>,
    pub(crate) provide_storage: Arc<dyn ProvideStorage>,
    pub(crate) provide_api_client: Arc<dyn ProvideApiClient>,
    pub(crate) kafru_queue: Arc<Queue<'static>>,
}

impl Provider {
    pub fn new(db: DatabaseConnection, kafru_queue: Arc<Queue<'static>>) -> Self {
        let provider = DefaultProvider::new(db);
        Self {
            provide_task_repo: Arc::new(provider.clone()),
            provide_storage: Arc::new(provider.clone()),
            provide_api_client: Arc::new(UserApiClientProvider::default()),
            kafru_queue,
        }
    }

    pub(crate) fn task_service(&self) -> TaskService {
        TaskService::new(
            self.provide_task_repo.task_repo(),
            self.provide_api_client.api_client(),
            self.episode_service(),
            self.kafru_queue.clone(),
        )
    }

    pub(crate) fn episode_service(&self) -> EpisodeService {
        EpisodeService::new(self.provide_storage.storage())
    }
}
