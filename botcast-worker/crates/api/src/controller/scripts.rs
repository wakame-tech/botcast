use super::{ApiImpl, into_openapi_object};
use anyhow::Result;
use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use openapi::{
    apis::scripts::*,
    models::{self, *},
};
use repos::{
    entities::{scripts::Model as Script, users::Model as User},
    id::ScriptId,
    repo::ScriptRepo,
};
use uuid::Uuid;

fn into_model(script: Script) -> Result<models::Script> {
    Ok(models::Script {
        id: script.id,
        title: script.title,
        description: script.description.unwrap(),
        arguments: into_openapi_object(script.arguments)?,
        template: into_openapi_object(script.template)?,
    })
}

#[async_trait]
impl Scripts<anyhow::Error> for ApiImpl {
    type Claims = User;

    async fn scripts_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
    ) -> Result<ScriptsGetResponse> {
        let scripts = self
            .script_repo
            .list(&user.id)
            .await?
            .into_iter()
            .map(into_model)
            .collect::<Result<Vec<_>>>()?;
        Ok(ScriptsGetResponse::Status200_OK(scripts))
    }

    async fn scripts_post(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
        body: &ScriptsPostRequest,
    ) -> Result<ScriptsPostResponse> {
        self.script_repo
            .create(Script {
                id: Uuid::new_v4(),
                user_id: user.id,
                title: body.title.clone(),
                description: Some(body.description.clone()),
                template: serde_json::to_value(body.template.clone())
                    .map_err(|_| anyhow::anyhow!("Failed to serialize template"))?,
                arguments: serde_json::to_value(body.arguments.clone())
                    .map_err(|_| anyhow::anyhow!("Failed to serialize arguments"))?,
            })
            .await?;
        Ok(ScriptsPostResponse::Status200_OK)
    }

    async fn scripts_script_id_delete(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        _claims: &Self::Claims,
        path_params: &ScriptsScriptIdDeletePathParams,
    ) -> Result<ScriptsScriptIdDeleteResponse> {
        self.script_repo
            .delete(&ScriptId(path_params.script_id.parse().unwrap()))
            .await?;
        Ok(ScriptsScriptIdDeleteResponse::Status200_OK)
    }

    async fn scripts_script_id_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        _claims: &Self::Claims,
        _path_params: &ScriptsScriptIdGetPathParams,
    ) -> Result<ScriptsScriptIdGetResponse> {
        let script = self
            .script_repo
            .find_by_id(&ScriptId(_path_params.script_id.parse().unwrap()))
            .await?;
        Ok(ScriptsScriptIdGetResponse::Status200_OK(into_model(
            script,
        )?))
    }

    async fn scripts_script_id_put(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        _claims: &Self::Claims,
        path_params: &ScriptsScriptIdPutPathParams,
        body: &ScriptsScriptIdPutRequest,
    ) -> Result<ScriptsScriptIdPutResponse> {
        let script = self
            .script_repo
            .find_by_id(&ScriptId(path_params.script_id.parse().unwrap()))
            .await?;
        let script = Script {
            id: script.id,
            user_id: script.user_id,
            title: body.title.clone(),
            description: Some(body.description.clone()),
            template: serde_json::to_value(body.template.clone())?,
            arguments: script.arguments,
        };
        self.script_repo.update(script).await?;
        Ok(ScriptsScriptIdPutResponse::Status200_OK)
    }
}
