use super::episode_service::EpisodeService;
use super::task_service::TaskService;
use kafru::queue::Queue;
use repos::provider::ProvideStorage;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Provider {
    pub(crate) provide_storage: Arc<dyn ProvideStorage>,
    pub(crate) kafru_queue: Arc<Queue<'static>>,
}

impl Provider {
    pub fn new(kafru_queue: Arc<Queue<'static>>) -> Self {
        let provider = repos::provider::DefaultProvider::new();
        Self {
            provide_storage: Arc::new(provider),
            kafru_queue,
        }
    }

    pub(crate) fn task_service(&self) -> TaskService {
        TaskService::new(self.episode_service(), self.kafru_queue.clone())
    }

    pub(crate) fn episode_service(&self) -> EpisodeService {
        EpisodeService::new(self.provide_storage.storage())
    }
}
