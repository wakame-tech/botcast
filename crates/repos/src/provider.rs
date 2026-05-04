use crate::{r2_storage::R2Storage, storage::Storage};
use std::{fmt::Debug, sync::Arc};

pub trait ProvideStorage: Debug + Send + Sync {
    fn storage(&self) -> Arc<dyn Storage>;
}

#[derive(Debug, Clone)]
pub struct DefaultProvider;

impl DefaultProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvideStorage for DefaultProvider {
    fn storage(&self) -> Arc<dyn Storage> {
        Arc::new(R2Storage::new().expect("Failed to create storage"))
    }
}
