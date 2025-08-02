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
pub enum CornersCornerIdMailsGetResponse {
    /// OK
    Status200_OK
    (Vec<models::Mail>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CornersCornerIdMailsMailIdDeleteResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CornersCornerIdMailsPostResponse {
    /// OK
    Status200_OK
}


/// Mails
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Mails<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// CornersCornerIdMailsGet - GET /corners/{cornerId}/mails
    async fn corners_corner_id_mails_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::CornersCornerIdMailsGetPathParams,
    ) -> Result<CornersCornerIdMailsGetResponse, E>;

    /// CornersCornerIdMailsMailIdDelete - DELETE /corners/{cornerId}/mails/{mailId}
    async fn corners_corner_id_mails_mail_id_delete(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::CornersCornerIdMailsMailIdDeletePathParams,
    ) -> Result<CornersCornerIdMailsMailIdDeleteResponse, E>;

    /// CornersCornerIdMailsPost - POST /corners/{cornerId}/mails
    async fn corners_corner_id_mails_post(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::CornersCornerIdMailsPostPathParams,
            body: &models::CornersCornerIdMailsPostRequest,
    ) -> Result<CornersCornerIdMailsPostResponse, E>;
}
