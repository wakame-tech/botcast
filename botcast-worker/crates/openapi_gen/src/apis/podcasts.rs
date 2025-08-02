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
pub enum PodcastPodcastIdDeleteResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum PodcastPodcastIdGetResponse {
    /// OK
    Status200_OK
    (models::PodcastPodcastIdGet200Response)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum PodcastPodcastIdPutResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum PodcastsGetResponse {
    /// OK
    Status200_OK
    (Vec<models::Podcast>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum PodcastsPostResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum TopPodcastsGetResponse {
    /// OK
    Status200_OK
    (Vec<models::Podcast>)
}


/// Podcasts
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Podcasts<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// PodcastPodcastIdDelete - DELETE /podcast/{podcastId}
    async fn podcast_podcast_id_delete(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::PodcastPodcastIdDeletePathParams,
    ) -> Result<PodcastPodcastIdDeleteResponse, E>;

    /// PodcastPodcastIdGet - GET /podcast/{podcastId}
    async fn podcast_podcast_id_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::PodcastPodcastIdGetPathParams,
    ) -> Result<PodcastPodcastIdGetResponse, E>;

    /// PodcastPodcastIdPut - PUT /podcast/{podcastId}
    async fn podcast_podcast_id_put(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::PodcastPodcastIdPutPathParams,
            body: &models::PodcastsPostRequest,
    ) -> Result<PodcastPodcastIdPutResponse, E>;

    /// PodcastsGet - GET /podcasts
    async fn podcasts_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<PodcastsGetResponse, E>;

    /// PodcastsPost - POST /podcasts
    async fn podcasts_post(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::PodcastsPostRequest,
    ) -> Result<PodcastsPostResponse, E>;

    /// TopPodcastsGet - GET /topPodcasts
    async fn top_podcasts_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
    ) -> Result<TopPodcastsGetResponse, E>;
}
