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
pub enum CornersCornerIdDeleteResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CornersCornerIdGetResponse {
    /// OK
    Status200_OK
    (models::Corner)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CornersCornerIdPutResponse {
    /// OK
    Status200_OK
}


/// Corners
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Corners<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// CornersCornerIdDelete - DELETE /corners/{cornerId}
    async fn corners_corner_id_delete(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::CornersCornerIdDeletePathParams,
    ) -> Result<CornersCornerIdDeleteResponse, E>;

    /// CornersCornerIdGet - GET /corners/{cornerId}
    async fn corners_corner_id_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::CornersCornerIdGetPathParams,
    ) -> Result<CornersCornerIdGetResponse, E>;

    /// CornersCornerIdPut - PUT /corners/{cornerId}
    async fn corners_corner_id_put(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::CornersCornerIdPutPathParams,
            body: &models::CornersCornerIdPutRequest,
    ) -> Result<CornersCornerIdPutResponse, E>;
}
