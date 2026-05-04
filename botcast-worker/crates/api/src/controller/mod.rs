mod auth;

use anyhow::Result;
use async_trait::async_trait;
use axum::http::{HeaderMap, Method};
use axum_extra::extract::{CookieJar, Host};
use openapi::apis::{ApiAuthBasic, BasicAuthKind, ErrorHandler};
use supabase_auth::models::{AuthClient, User};

#[derive(Clone)]
pub(crate) struct ApiImpl {
    pub(crate) auth_client: AuthClient,
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
        let bearer = authorization.to_str().ok()?;
        let token = bearer.strip_prefix("Bearer ")?;
        self.auth_client.get_user(token).await.ok()
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
