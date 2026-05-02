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
pub enum MeGetResponse {
    /// OK
    Status200_OK
    (models::User)
    ,
    /// Not Found
    Status404_NotFound
    (models::SignUpPost400Response)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SignInPostResponse {
    /// OK
    Status200_OK
    (models::SignUpPost200Response)
    ,
    /// Not Found
    Status404_NotFound
    (models::SignUpPost400Response)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum SignUpPostResponse {
    /// OK
    Status200_OK
    (models::SignUpPost200Response)
    ,
    /// Bad Request
    Status400_BadRequest
    (models::SignUpPost400Response)
    ,
    /// Conflict
    Status409_Conflict
    (models::SignUpPost400Response)
}


/// Auth
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Auth<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// MeGet - GET /me
    async fn me_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<MeGetResponse, E>;

    /// Sign in.
    ///
    /// SignInPost - POST /signIn
    async fn sign_in_post(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &models::SignUpPostRequest,
    ) -> Result<SignInPostResponse, E>;

    /// Sign up.
    ///
    /// SignUpPost - POST /signUp
    async fn sign_up_post(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &models::SignUpPostRequest,
    ) -> Result<SignUpPostResponse, E>;
}
