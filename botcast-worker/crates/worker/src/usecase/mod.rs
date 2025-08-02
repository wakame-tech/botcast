pub(crate) mod episode_service;
pub(crate) mod provider;
pub(crate) mod script_service;
pub(crate) mod task_service;

use openapi_client::apis::configuration::{ApiKey, Configuration};
pub use provider::Provider;
use serde::Serialize;
use std::fmt::Debug;

use crate::error::Error;

pub(crate) trait ProvideApiClient: Debug + Send + Sync {
    fn api_client(&self) -> Configuration;
}

#[derive(Debug, Default)]
pub(crate) struct UserApiClientProvider {
    user_token: Option<String>,
}

impl UserApiClientProvider {
    pub(crate) fn new(user_token: Option<String>) -> Self {
        Self { user_token }
    }
}

impl ProvideApiClient for UserApiClientProvider {
    fn api_client(&self) -> Configuration {
        let api_endpoint = std::env::var("API_ENDPOINT").expect("API_ENDPOINT is not set");
        let api_key = self.user_token.as_ref().cloned().unwrap_or_else(|| {
            std::env::var("SUPABASE_SERVICE_ROLE_KEY")
                .expect("SUPABASE_SERVICE_ROLE_KEY is not set")
        });
        Configuration {
            base_path: api_endpoint.clone(),
            api_key: Some(ApiKey {
                key: api_key,
                prefix: None,
            }),
            ..Default::default()
        }
    }
}

pub(super) fn into_other_error<T: Serialize>(err: openapi_client::apis::Error<T>) -> Error {
    Error::Other(anyhow::anyhow!("OpenAPI error: {}", err.to_string()))
}
