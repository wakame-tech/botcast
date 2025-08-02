use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::{CookieJar, Host};
use bytes::Bytes;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SecretsGetResponse {
    /// OK
    Status200_OK
    (Vec<models::Secret>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SecretsPostResponse {
    /// OK
    Status200_OK
}


/// Secrets
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Secrets<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// SecretsGet - GET /secrets
    async fn secrets_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<SecretsGetResponse, E>;

    /// SecretsPost - POST /secrets
    async fn secrets_post(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::SecretsPostRequest,
    ) -> Result<SecretsPostResponse, E>;
}
