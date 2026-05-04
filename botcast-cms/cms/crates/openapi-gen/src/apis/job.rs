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
pub enum JobApiCreateResponse {
    /// There is no content to send for this request, but the headers may be useful. 
    Status204_ThereIsNoContentToSendForThisRequest
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum JobApiListResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (Vec<models::Job>)
}


/// Job
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Job<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// JobApiCreate - POST /jobs
    async fn job_api_create(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &models::CreateOrUpdateJob,
    ) -> Result<JobApiCreateResponse, E>;

    /// JobApiList - GET /jobs
    async fn job_api_list(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
    ) -> Result<JobApiListResponse, E>;
}
