use super::episode_generation::EpisodeGenerationService;
use super::episode_service::EpisodeService;
use super::task_service::TaskService;
use super::cms_client::CmsClient;
use kafru::queue::Queue;
use std::sync::Arc;

/// ユースケース層のサービスを組み立てるプロバイダー。
///
/// # クレート依存関係
///
/// ```mermaid
/// graph TD
///   worker --> audio_generator
///   worker --> cms["botcast-cms (REST / MCP)"]
///   worker --> kafru["kafru (SurrealDB ジョブキュー)"]
///   worker --> readable_text
///   audio_generator --> voicevox["VoiceVox Engine"]
/// ```
#[cfg_attr(doc, aquamarine::aquamarine)]
#[derive(Clone)]
pub struct Provider {
    pub(crate) cms: Arc<CmsClient>,
    pub(crate) kafru_queue: Arc<Queue<'static>>,
}

impl std::fmt::Debug for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Provider").finish()
    }
}

impl Provider {
    pub fn new(kafru_queue: Arc<Queue<'static>>) -> Self {
        Self {
            cms: Arc::new(CmsClient::from_env()),
            kafru_queue,
        }
    }

    pub(crate) fn task_service(&self) -> TaskService {
        TaskService::new(self.episode_service(), self.kafru_queue.clone())
    }

    pub(crate) fn episode_generation_service(&self) -> EpisodeGenerationService {
        EpisodeGenerationService::new(self.cms.clone())
    }

    pub(crate) fn episode_service(&self) -> EpisodeService {
        EpisodeService::new(self.cms.clone())
    }
}
