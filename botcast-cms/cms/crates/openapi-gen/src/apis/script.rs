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
pub enum ScriptApiExecuteResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (models::ExecuteScriptResponse)
}


/// Script
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Script<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// ScriptApiExecute - POST /scripts
    async fn script_api_execute(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &models::ExecuteScriptRequest,
    ) -> Result<ScriptApiExecuteResponse, E>;
}
