use super::episode_service::EpisodeService;
use super::task_service::TaskService;
use kafru::queue::Queue;
use crate::storage::provider::ProvideStorage;
use std::sync::Arc;

/// ユースケース層のサービスを組み立てるプロバイダー。
///
/// # クレート依存関係
///
/// ```mermaid
/// graph TD
///   worker --> audio_generator
///   worker --> storage["storage (R2)"]\n///   worker --> openapi_client["openapi_client (Section 型)"]
///   worker --> readable_text
///   api["api (別バイナリ)"]
/// ```
#[cfg_attr(doc, aquamarine::aquamarine)]
#[derive(Debug, Clone)]
pub struct Provider {
    pub(crate) provide_storage: Arc<dyn ProvideStorage>,
    pub(crate) kafru_queue: Arc<Queue<'static>>,
}

impl Provider {
    pub fn new(kafru_queue: Arc<Queue<'static>>) -> Self {
        let provider = crate::storage::provider::DefaultProvider::new();
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
