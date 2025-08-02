use crate::{error::Error, usecase::into_other_error};
use futures::future::try_join_all;
use openapi_client::apis::{auth_api::me_get, configuration::Configuration};
use repos::{
    id::ScriptId,
    repo::{ScriptRepo, SecretRepo},
};
use script_runtime::{plugins::botcast_api::BotCastApiPlugin, runtime::ScriptRuntime};
use std::{collections::BTreeMap, sync::Arc};
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct ScriptService {
    script_repo: Arc<dyn ScriptRepo>,
    secret_repo: Arc<dyn SecretRepo>,
    configuration: Configuration,
}

impl ScriptService {
    pub(crate) fn new(
        script_repo: Arc<dyn ScriptRepo>,
        secret_repo: Arc<dyn SecretRepo>,
        configuration: Configuration,
    ) -> Self {
        Self {
            script_repo,
            secret_repo,
            configuration,
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

        let mut runtime = ScriptRuntime::default();
        runtime.install_plugin(BotCastApiPlugin::new(self.configuration.clone()));

        let res = runtime
            .run(template, context)
            .await
            .map_err(Error::Script)?;
        Ok(res)
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
