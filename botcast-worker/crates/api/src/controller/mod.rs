mod auth;
mod corners;
mod episodes;
mod mails;
mod podcasts;
mod scripts;
mod secrets;
mod tasks;

use anyhow::Result;
use async_trait::async_trait;
use axum::http::{HeaderMap, Method};
use axum_extra::extract::{CookieJar, Host};
use openapi::{
    apis::{ApiAuthBasic, BasicAuthKind, ErrorHandler},
    types::Object,
};
use repos::{
    entities::users::Model as User,
    postgres::{
        PostgresCornerRepo, PostgresEpisodeRepo, PostgresMailRepo, PostgresPodcastRepo,
        PostgresScriptRepo, PostgresSecretRepo, PostgresTaskRepo, PostgresUserRepo,
    },
    r2_storage::R2Storage,
    repo::UserRepo,
};
use std::{collections::HashMap, sync::Arc};
use supabase_auth::models::AuthClient;

#[derive(Clone)]
pub(crate) struct ApiImpl {
    pub(crate) auth_client: AuthClient,
    pub(crate) storage: Arc<R2Storage>,
    pub(crate) user_repo: Arc<PostgresUserRepo>,
    pub(crate) secret_repo: Arc<PostgresSecretRepo>,
    pub(crate) podcast_repo: Arc<PostgresPodcastRepo>,
    pub(crate) episode_repo: Arc<PostgresEpisodeRepo>,
    pub(crate) script_repo: Arc<PostgresScriptRepo>,
    pub(crate) corner_repo: Arc<PostgresCornerRepo>,
    pub(crate) mail_repo: Arc<PostgresMailRepo>,
    pub(crate) task_repo: Arc<PostgresTaskRepo>,
}

impl AsRef<ApiImpl> for ApiImpl {
    fn as_ref(&self) -> &ApiImpl {
        self
    }
}

#[async_trait]
impl ApiAuthBasic for ApiImpl {
    type Claims = User;

    async fn extract_claims_from_auth_header(
        &self,
        _kind: BasicAuthKind,
        headers: &HeaderMap,
        key: &str,
    ) -> Option<Self::Claims> {
        let authorization = headers.get(key)?;
        let bearer = authorization.to_str().unwrap();
        let token = if bearer.starts_with("Bearer ") {
            let token = bearer.trim_start_matches("Bearer ");
            token.to_string()
        } else {
            return None;
        };
        let supabase_user = self.auth_client.get_user(&token).await.ok()?;
        let user = self
            .user_repo
            .find_by_auth_id(&supabase_user.id)
            .await
            .ok()?;
        Some(user)
    }
}

#[async_trait]
impl ErrorHandler<anyhow::Error> for ApiImpl {
    async fn handle_error(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        err: anyhow::Error,
    ) -> Result<axum::response::Response, axum::http::StatusCode> {
        eprintln!("{:?}", err);
        Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub(crate) fn into_openapi_object(value: serde_json::Value) -> Result<HashMap<String, Object>> {
    let kv = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Expected a JSON object, but got: {}", value))?
        .into_iter()
        .map(|(k, v)| {
            (
                k.clone(),
                serde_json::to_string(v).unwrap().parse().unwrap(),
            )
        });
    Ok(HashMap::from_iter(kv))
}
