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
pub enum TasksGetResponse {
    /// OK
    Status200_OK
    (Vec<models::Task>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum TasksPostResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum TasksTaskIdDeleteResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum TasksTaskIdPutResponse {
    /// OK
    Status200_OK
}


/// Tasks
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Tasks<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// TasksGet - GET /tasks
    async fn tasks_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<TasksGetResponse, E>;

    /// TasksPost - POST /tasks
    async fn tasks_post(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::TasksPostRequest,
    ) -> Result<TasksPostResponse, E>;

    /// TasksTaskIdDelete - DELETE /tasks/{taskId}
    async fn tasks_task_id_delete(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::TasksTaskIdDeletePathParams,
    ) -> Result<TasksTaskIdDeleteResponse, E>;

    /// TasksTaskIdPut - PUT /tasks/{taskId}
    async fn tasks_task_id_put(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::TasksTaskIdPutPathParams,
            body: &models::TasksTaskIdPutRequest,
    ) -> Result<TasksTaskIdPutResponse, E>;
}
