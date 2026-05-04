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
pub enum CanvasApiCreateEdgeResponse {
    /// The request has succeeded and a new resource has been created as a result.
    Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
    (models::Edge)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CanvasApiDeleteEdgeResponse {
    /// There is no content to send for this request, but the headers may be useful. 
    Status204_ThereIsNoContentToSendForThisRequest
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CanvasApiListEdgesResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (Vec<models::Edge>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum CanvasApiQueryResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (models::CanvasQueryResponse)
}


/// Canvas
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Canvas<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// CanvasApiCreateEdge - POST /canvas/edges
    async fn canvas_api_create_edge(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &models::CreateEdgeRequest,
    ) -> Result<CanvasApiCreateEdgeResponse, E>;

    /// CanvasApiDeleteEdge - DELETE /canvas/edges/{edgeId}
    async fn canvas_api_delete_edge(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::CanvasApiDeleteEdgePathParams,
    ) -> Result<CanvasApiDeleteEdgeResponse, E>;

    /// CanvasApiListEdges - GET /canvas/edges
    async fn canvas_api_list_edges(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      query_params: &models::CanvasApiListEdgesQueryParams,
    ) -> Result<CanvasApiListEdgesResponse, E>;

    /// CanvasApiQuery - POST /canvas/query
    async fn canvas_api_query(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
            body: &models::CanvasQueryRequest,
    ) -> Result<CanvasApiQueryResponse, E>;
}
