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
pub enum CollectionApiCreateResponse {
    /// The request has succeeded and a new resource has been created as a result.
    Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
    (models::Collection)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CollectionApiDeleteResponse {
    /// There is no content to send for this request, but the headers may be useful. 
    Status204_ThereIsNoContentToSendForThisRequest
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CollectionApiListResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (Vec<models::Collection>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CollectionApiReadResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (models::Collection)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CollectionApiUpdateResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (models::Collection)
}


/// Collections
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Collections<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// CollectionApiCreate - POST /collections
    async fn collection_api_create(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &models::CreateOrUpdateCollection,
    ) -> Result<CollectionApiCreateResponse, E>;

    /// CollectionApiDelete - DELETE /collections/{collectionId}
    async fn collection_api_delete(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::CollectionApiDeletePathParams,
    ) -> Result<CollectionApiDeleteResponse, E>;

    /// CollectionApiList - GET /collections
    async fn collection_api_list(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
    ) -> Result<CollectionApiListResponse, E>;

    /// CollectionApiRead - GET /collections/{collectionId}
    async fn collection_api_read(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::CollectionApiReadPathParams,
    ) -> Result<CollectionApiReadResponse, E>;

    /// CollectionApiUpdate - PUT /collections/{collectionId}
    async fn collection_api_update(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::CollectionApiUpdatePathParams,
            body: &models::UpdateCollection,
    ) -> Result<CollectionApiUpdateResponse, E>;
}
