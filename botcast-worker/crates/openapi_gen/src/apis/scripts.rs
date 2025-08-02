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
pub enum ScriptsGetResponse {
    /// OK
    Status200_OK
    (Vec<models::Script>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ScriptsPostResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ScriptsScriptIdDeleteResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ScriptsScriptIdGetResponse {
    /// OK
    Status200_OK
    (models::Script)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum ScriptsScriptIdPutResponse {
    /// OK
    Status200_OK
}


/// Scripts
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Scripts<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// ScriptsGet - GET /scripts
    async fn scripts_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<ScriptsGetResponse, E>;

    /// ScriptsPost - POST /scripts
    async fn scripts_post(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::ScriptsPostRequest,
    ) -> Result<ScriptsPostResponse, E>;

    /// ScriptsScriptIdDelete - DELETE /scripts/{scriptId}
    async fn scripts_script_id_delete(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::ScriptsScriptIdDeletePathParams,
    ) -> Result<ScriptsScriptIdDeleteResponse, E>;

    /// ScriptsScriptIdGet - GET /scripts/{scriptId}
    async fn scripts_script_id_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::ScriptsScriptIdGetPathParams,
    ) -> Result<ScriptsScriptIdGetResponse, E>;

    /// ScriptsScriptIdPut - PUT /scripts/{scriptId}
    async fn scripts_script_id_put(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::ScriptsScriptIdPutPathParams,
            body: &models::ScriptsScriptIdPutRequest,
    ) -> Result<ScriptsScriptIdPutResponse, E>;
}
