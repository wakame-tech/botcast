use crate::{error::Error, usecase::into_other_error};
use futures::future::try_join_all;
use openapi_client::apis::{auth_api::me_get, configuration::Configuration};
use repos::{
    id::ScriptId,
    repo::{ScriptRepo, SecretRepo},
};
use std::{collections::BTreeMap, sync::Arc};
use tracing::instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct CmsScriptResponseData {
    stdout: Option<String>,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
struct CmsScriptResponse {
    code: i32,
    data: CmsScriptResponseData,
}

#[derive(Clone)]
pub(crate) struct ScriptService {
    script_repo: Arc<dyn ScriptRepo>,
    secret_repo: Arc<dyn SecretRepo>,
    configuration: Configuration,
    cms_url: String,
}

impl ScriptService {
    pub(crate) fn new(
        script_repo: Arc<dyn ScriptRepo>,
        secret_repo: Arc<dyn SecretRepo>,
        configuration: Configuration,
    ) -> Self {
        let cms_url =
            std::env::var("CMS_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());
        Self {
            script_repo,
            secret_repo,
            configuration,
            cms_url,
        }
    }

    async fn replace_context_to_secrets(
        &self,
        user_id: Uuid,
        context: BTreeMap<String, serde_json::Value>,
    ) -> anyhow::Result<BTreeMap<String, serde_json::Value>, Error> {
        let context: Result<_, Error> =
            try_join_all(context.into_iter().map(|(key, value)| async {
                match (key, value) {
                    (key, serde_json::Value::String(value)) if value.starts_with("$") => {
                        let name = value.trim_start_matches("$");
                        let secret = self.secret_repo.find_by_name(&user_id, name).await?;
                        let secret = secret.decrypted_secret;
                        Ok((key, serde_json::Value::String(secret)))
                    }
                    e => Ok(e),
                }
            }))
            .await;
        Ok(context?.into_iter().collect())
    }

    #[instrument(skip(self), ret)]
    pub(crate) async fn run_template(
        &self,
        template: &serde_json::Value,
        parameters: BTreeMap<String, serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value, Error> {
        let me = me_get(&self.configuration)
            .await
            .map_err(into_other_error)?;

        let context = self.replace_context_to_secrets(me.id, parameters).await?;

        let code = match template {
            serde_json::Value::String(s) => s.clone(),
            _ => {
                return Err(Error::InvalidInput(anyhow::anyhow!(
                    "Template must be a string containing JavaScript code"
                )))
            }
        };

        let preload = format!(
            "const context = {};",
            serde_json::to_string(&context)
                .map_err(|e| Error::Other(anyhow::anyhow!("Failed to serialize context: {}", e)))?
        );

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/scripts", self.cms_url))
            .json(&serde_json::json!({
                "language": "nodejs",
                "code": code,
                "preload": preload,
                "enable_network": true,
            }))
            .send()
            .await
            .map_err(|e| Error::Script(anyhow::anyhow!("CMS request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Script(anyhow::anyhow!(
                "CMS returned {}: {}",
                status,
                body
            )));
        }

        let script_res: CmsScriptResponse = response
            .json()
            .await
            .map_err(|e| Error::Script(anyhow::anyhow!("Failed to parse CMS response: {}", e)))?;

        if script_res.code != 0 {
            return Err(Error::Script(anyhow::anyhow!(
                "Script failed: {}",
                script_res.data.error.unwrap_or_default()
            )));
        }

        let stdout = script_res.data.stdout.unwrap_or_default();
        serde_json::from_str(&stdout).map_err(|e| {
            Error::Script(anyhow::anyhow!(
                "Failed to parse script output as JSON: {}",
                e
            ))
        })
    }

    pub(crate) async fn update_template(
        &self,
        script_id: &ScriptId,
        template: serde_json::Value,
    ) -> anyhow::Result<(), Error> {
        let mut script = self.script_repo.find_by_id(script_id).await?;

        script.template = template;
        self.script_repo.update(script).await?;
        Ok(())
    }
}
