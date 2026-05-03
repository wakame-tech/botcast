pub(crate) mod agent_service;
pub(crate) mod episode_service;
pub(crate) mod mcp_client;
pub(crate) mod provider;
pub(crate) mod task_service;

use openapi_client::apis::configuration::{ApiKey, Configuration};
pub use provider::Provider;
use std::fmt::Debug;

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

