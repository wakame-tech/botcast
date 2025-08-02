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
pub enum EpisodesEpisodeIdDeleteResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum EpisodesEpisodeIdGetResponse {
    /// OK
    Status200_OK
    (models::Episode)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum EpisodesEpisodeIdPutResponse {
    /// OK
    Status200_OK
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum EpisodesPostResponse {
    /// OK
    Status200_OK
}


/// Episodes
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Episodes<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    type Claims;

    /// EpisodesEpisodeIdDelete - DELETE /episodes/{episodeId}
    async fn episodes_episode_id_delete(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::EpisodesEpisodeIdDeletePathParams,
    ) -> Result<EpisodesEpisodeIdDeleteResponse, E>;

    /// EpisodesEpisodeIdGet - GET /episodes/{episodeId}
    async fn episodes_episode_id_get(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::EpisodesEpisodeIdGetPathParams,
    ) -> Result<EpisodesEpisodeIdGetResponse, E>;

    /// EpisodesEpisodeIdPut - PUT /episodes/{episodeId}
    async fn episodes_episode_id_put(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::EpisodesEpisodeIdPutPathParams,
            body: &models::EpisodesEpisodeIdPutRequest,
    ) -> Result<EpisodesEpisodeIdPutResponse, E>;

    /// EpisodesPost - POST /episodes
    async fn episodes_post(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
        claims: &Self::Claims,
            body: &models::EpisodesPostRequest,
    ) -> Result<EpisodesPostResponse, E>;
}
