use std::collections::HashMap;

use axum::{body::Body, extract::*, response::Response, routing::*};
use axum_extra::extract::{CookieJar, Host, Query as QueryExtra};
use bytes::Bytes;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use tracing::error;
use validator::{Validate, ValidationErrors};

use crate::{header, types::*};

#[allow(unused_imports)]
use crate::{apis, models};


/// Setup API Server.
pub fn new<I, A, E>(api_impl: I) -> Router
where
    I: AsRef<A> + Clone + Send + Sync + 'static,
    A: apis::canvas::Canvas<E> + apis::collections::Collections<E> + apis::job::Job<E> + apis::record::Record<E> + apis::script::Script<E> + Send + Sync + 'static,
    E: std::fmt::Debug + Send + Sync + 'static,
    
{
    // build our application with a route
    Router::new()
        .route("/canvas/edges",
            get(canvas_api_list_edges::<I, A, E>).post(canvas_api_create_edge::<I, A, E>)
        )
        .route("/canvas/edges/{edge_id}",
            delete(canvas_api_delete_edge::<I, A, E>)
        )
        .route("/canvas/query",
            post(canvas_api_query::<I, A, E>)
        )
        .route("/collections",
            get(collection_api_list::<I, A, E>).post(collection_api_create::<I, A, E>)
        )
        .route("/collections/{collection_id}",
            delete(collection_api_delete::<I, A, E>).get(collection_api_read::<I, A, E>).put(collection_api_update::<I, A, E>)
        )
        .route("/jobs",
            get(job_api_list::<I, A, E>).post(job_api_create::<I, A, E>)
        )
        .route("/records/{collection_id}",
            get(record_api_list::<I, A, E>).post(record_api_create::<I, A, E>)
        )
        .route("/records/{collection_id}/generate",
            post(record_api_generate::<I, A, E>)
        )
        .route("/records/{collection_id}/merge",
            post(record_api_merge_records::<I, A, E>)
        )
        .route("/records/{collection_id}/{record_id}",
            delete(record_api_delete::<I, A, E>).get(record_api_read::<I, A, E>).put(record_api_update::<I, A, E>)
        )
        .route("/records/{collection_id}/{record_id}/copy",
            post(record_api_copy_record::<I, A, E>)
        )
        .route("/records/{collection_id}/{record_id}/images/{field_name}",
            delete(record_api_delete_image::<I, A, E>).get(record_api_get_image::<I, A, E>).post(record_api_upload_image::<I, A, E>)
        )
        .route("/records/{collection_id}/{record_id}/split",
            post(record_api_split_record::<I, A, E>)
        )
        .route("/scripts",
            post(script_api_execute::<I, A, E>)
        )
        .with_state(api_impl)
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct CanvasApiCreateEdgeBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::CreateEdgeRequest,
    }


#[tracing::instrument(skip_all)]
fn canvas_api_create_edge_validation(
        body: models::CreateEdgeRequest,
) -> std::result::Result<(
        models::CreateEdgeRequest,
), ValidationErrors>
{
              let b = CanvasApiCreateEdgeBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// CanvasApiCreateEdge - POST /canvas/edges
#[tracing::instrument(skip_all)]
async fn canvas_api_create_edge<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
          Json(body): Json<models::CreateEdgeRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::canvas::Canvas<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    canvas_api_create_edge_validation(
          body,
    )
  ).await.unwrap();

  let Ok((
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().canvas_api_create_edge(
      &method,
      &host,
      &cookies,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::canvas::CanvasApiCreateEdgeResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
                                                    (body)
                                                => {
                                                  let mut response = response.status(201);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn canvas_api_delete_edge_validation(
  path_params: models::CanvasApiDeleteEdgePathParams,
) -> std::result::Result<(
  models::CanvasApiDeleteEdgePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// CanvasApiDeleteEdge - DELETE /canvas/edges/{edgeId}
#[tracing::instrument(skip_all)]
async fn canvas_api_delete_edge<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::CanvasApiDeleteEdgePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::canvas::Canvas<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    canvas_api_delete_edge_validation(
        path_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().canvas_api_delete_edge(
      &method,
      &host,
      &cookies,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::canvas::CanvasApiDeleteEdgeResponse::Status204_ThereIsNoContentToSendForThisRequest
                                                => {
                                                  let mut response = response.status(204);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn canvas_api_list_edges_validation(
  query_params: models::CanvasApiListEdgesQueryParams,
) -> std::result::Result<(
  models::CanvasApiListEdgesQueryParams,
), ValidationErrors>
{
  query_params.validate()?;

Ok((
  query_params,
))
}
/// CanvasApiListEdges - GET /canvas/edges
#[tracing::instrument(skip_all)]
async fn canvas_api_list_edges<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  QueryExtra(query_params): QueryExtra<models::CanvasApiListEdgesQueryParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::canvas::Canvas<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    canvas_api_list_edges_validation(
        query_params,
    )
  ).await.unwrap();

  let Ok((
    query_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().canvas_api_list_edges(
      &method,
      &host,
      &cookies,
        &query_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::canvas::CanvasApiListEdgesResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct CanvasApiQueryBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::CanvasQueryRequest,
    }


#[tracing::instrument(skip_all)]
fn canvas_api_query_validation(
        body: models::CanvasQueryRequest,
) -> std::result::Result<(
        models::CanvasQueryRequest,
), ValidationErrors>
{
              let b = CanvasApiQueryBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// CanvasApiQuery - POST /canvas/query
#[tracing::instrument(skip_all)]
async fn canvas_api_query<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
          Json(body): Json<models::CanvasQueryRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::canvas::Canvas<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    canvas_api_query_validation(
          body,
    )
  ).await.unwrap();

  let Ok((
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().canvas_api_query(
      &method,
      &host,
      &cookies,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::canvas::CanvasApiQueryResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct CollectionApiCreateBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::CreateOrUpdateCollection,
    }


#[tracing::instrument(skip_all)]
fn collection_api_create_validation(
        body: models::CreateOrUpdateCollection,
) -> std::result::Result<(
        models::CreateOrUpdateCollection,
), ValidationErrors>
{
              let b = CollectionApiCreateBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// CollectionApiCreate - POST /collections
#[tracing::instrument(skip_all)]
async fn collection_api_create<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
          Json(body): Json<models::CreateOrUpdateCollection>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::collections::Collections<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    collection_api_create_validation(
          body,
    )
  ).await.unwrap();

  let Ok((
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().collection_api_create(
      &method,
      &host,
      &cookies,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::collections::CollectionApiCreateResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
                                                    (body)
                                                => {
                                                  let mut response = response.status(201);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn collection_api_delete_validation(
  path_params: models::CollectionApiDeletePathParams,
) -> std::result::Result<(
  models::CollectionApiDeletePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// CollectionApiDelete - DELETE /collections/{collectionId}
#[tracing::instrument(skip_all)]
async fn collection_api_delete<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::CollectionApiDeletePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::collections::Collections<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    collection_api_delete_validation(
        path_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().collection_api_delete(
      &method,
      &host,
      &cookies,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::collections::CollectionApiDeleteResponse::Status204_ThereIsNoContentToSendForThisRequest
                                                => {
                                                  let mut response = response.status(204);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn collection_api_list_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// CollectionApiList - GET /collections
#[tracing::instrument(skip_all)]
async fn collection_api_list<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::collections::Collections<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    collection_api_list_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().collection_api_list(
      &method,
      &host,
      &cookies,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::collections::CollectionApiListResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn collection_api_read_validation(
  path_params: models::CollectionApiReadPathParams,
) -> std::result::Result<(
  models::CollectionApiReadPathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// CollectionApiRead - GET /collections/{collectionId}
#[tracing::instrument(skip_all)]
async fn collection_api_read<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::CollectionApiReadPathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::collections::Collections<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    collection_api_read_validation(
        path_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().collection_api_read(
      &method,
      &host,
      &cookies,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::collections::CollectionApiReadResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct CollectionApiUpdateBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::UpdateCollection,
    }


#[tracing::instrument(skip_all)]
fn collection_api_update_validation(
  path_params: models::CollectionApiUpdatePathParams,
        body: models::UpdateCollection,
) -> std::result::Result<(
  models::CollectionApiUpdatePathParams,
        models::UpdateCollection,
), ValidationErrors>
{
  path_params.validate()?;
              let b = CollectionApiUpdateBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// CollectionApiUpdate - PUT /collections/{collectionId}
#[tracing::instrument(skip_all)]
async fn collection_api_update<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::CollectionApiUpdatePathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::UpdateCollection>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::collections::Collections<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    collection_api_update_validation(
        path_params,
          body,
    )
  ).await.unwrap();

  let Ok((
    path_params,
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().collection_api_update(
      &method,
      &host,
      &cookies,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::collections::CollectionApiUpdateResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct JobApiCreateBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::CreateOrUpdateJob,
    }


#[tracing::instrument(skip_all)]
fn job_api_create_validation(
        body: models::CreateOrUpdateJob,
) -> std::result::Result<(
        models::CreateOrUpdateJob,
), ValidationErrors>
{
              let b = JobApiCreateBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// JobApiCreate - POST /jobs
#[tracing::instrument(skip_all)]
async fn job_api_create<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
          Json(body): Json<models::CreateOrUpdateJob>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::job::Job<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    job_api_create_validation(
          body,
    )
  ).await.unwrap();

  let Ok((
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().job_api_create(
      &method,
      &host,
      &cookies,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::job::JobApiCreateResponse::Status204_ThereIsNoContentToSendForThisRequest
                                                => {
                                                  let mut response = response.status(204);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn job_api_list_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// JobApiList - GET /jobs
#[tracing::instrument(skip_all)]
async fn job_api_list<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::job::Job<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    job_api_list_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().job_api_list(
      &method,
      &host,
      &cookies,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::job::JobApiListResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct RecordApiCopyRecordBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::CopyRecordRequest,
    }


#[tracing::instrument(skip_all)]
fn record_api_copy_record_validation(
  path_params: models::RecordApiCopyRecordPathParams,
        body: models::CopyRecordRequest,
) -> std::result::Result<(
  models::RecordApiCopyRecordPathParams,
        models::CopyRecordRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = RecordApiCopyRecordBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// RecordApiCopyRecord - POST /records/{collectionId}/{recordId}/copy
#[tracing::instrument(skip_all)]
async fn record_api_copy_record<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiCopyRecordPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::CopyRecordRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_copy_record_validation(
        path_params,
          body,
    )
  ).await.unwrap();

  let Ok((
    path_params,
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_copy_record(
      &method,
      &host,
      &cookies,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiCopyRecordResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
                                                    (body)
                                                => {
                                                  let mut response = response.status(201);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct RecordApiCreateBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::CreateOrUpdateRecord,
    }


#[tracing::instrument(skip_all)]
fn record_api_create_validation(
  path_params: models::RecordApiCreatePathParams,
        body: models::CreateOrUpdateRecord,
) -> std::result::Result<(
  models::RecordApiCreatePathParams,
        models::CreateOrUpdateRecord,
), ValidationErrors>
{
  path_params.validate()?;
              let b = RecordApiCreateBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// RecordApiCreate - POST /records/{collectionId}
#[tracing::instrument(skip_all)]
async fn record_api_create<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiCreatePathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::CreateOrUpdateRecord>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_create_validation(
        path_params,
          body,
    )
  ).await.unwrap();

  let Ok((
    path_params,
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_create(
      &method,
      &host,
      &cookies,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiCreateResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
                                                    (body)
                                                => {
                                                  let mut response = response.status(201);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn record_api_delete_validation(
  path_params: models::RecordApiDeletePathParams,
) -> std::result::Result<(
  models::RecordApiDeletePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// RecordApiDelete - DELETE /records/{collectionId}/{recordId}
#[tracing::instrument(skip_all)]
async fn record_api_delete<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiDeletePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_delete_validation(
        path_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_delete(
      &method,
      &host,
      &cookies,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiDeleteResponse::Status204_ThereIsNoContentToSendForThisRequest
                                                => {
                                                  let mut response = response.status(204);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn record_api_delete_image_validation(
  path_params: models::RecordApiDeleteImagePathParams,
) -> std::result::Result<(
  models::RecordApiDeleteImagePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// RecordApiDeleteImage - DELETE /records/{collectionId}/{recordId}/images/{fieldName}
#[tracing::instrument(skip_all)]
async fn record_api_delete_image<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiDeleteImagePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_delete_image_validation(
        path_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_delete_image(
      &method,
      &host,
      &cookies,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiDeleteImageResponse::Status204_ThereIsNoContentToSendForThisRequest
                                                => {
                                                  let mut response = response.status(204);
                                                  response.body(Body::empty())
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct RecordApiGenerateBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::GenerateRecordRequest,
    }


#[tracing::instrument(skip_all)]
fn record_api_generate_validation(
  path_params: models::RecordApiGeneratePathParams,
        body: models::GenerateRecordRequest,
) -> std::result::Result<(
  models::RecordApiGeneratePathParams,
        models::GenerateRecordRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = RecordApiGenerateBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// RecordApiGenerate - POST /records/{collectionId}/generate
#[tracing::instrument(skip_all)]
async fn record_api_generate<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiGeneratePathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::GenerateRecordRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_generate_validation(
        path_params,
          body,
    )
  ).await.unwrap();

  let Ok((
    path_params,
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_generate(
      &method,
      &host,
      &cookies,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiGenerateResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
                                                    (body)
                                                => {
                                                  let mut response = response.status(201);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn record_api_get_image_validation(
  path_params: models::RecordApiGetImagePathParams,
) -> std::result::Result<(
  models::RecordApiGetImagePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// RecordApiGetImage - GET /records/{collectionId}/{recordId}/images/{fieldName}
#[tracing::instrument(skip_all)]
async fn record_api_get_image<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiGetImagePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_get_image_validation(
        path_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_get_image(
      &method,
      &host,
      &cookies,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiGetImageResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("*/*").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn record_api_list_validation(
  path_params: models::RecordApiListPathParams,
) -> std::result::Result<(
  models::RecordApiListPathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// RecordApiList - GET /records/{collectionId}
#[tracing::instrument(skip_all)]
async fn record_api_list<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiListPathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_list_validation(
        path_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_list(
      &method,
      &host,
      &cookies,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiListResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct RecordApiMergeRecordsBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::MergeRecordsRequest,
    }


#[tracing::instrument(skip_all)]
fn record_api_merge_records_validation(
  path_params: models::RecordApiMergeRecordsPathParams,
        body: models::MergeRecordsRequest,
) -> std::result::Result<(
  models::RecordApiMergeRecordsPathParams,
        models::MergeRecordsRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = RecordApiMergeRecordsBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// RecordApiMergeRecords - POST /records/{collectionId}/merge
#[tracing::instrument(skip_all)]
async fn record_api_merge_records<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiMergeRecordsPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::MergeRecordsRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_merge_records_validation(
        path_params,
          body,
    )
  ).await.unwrap();

  let Ok((
    path_params,
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_merge_records(
      &method,
      &host,
      &cookies,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiMergeRecordsResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}


#[tracing::instrument(skip_all)]
fn record_api_read_validation(
  path_params: models::RecordApiReadPathParams,
) -> std::result::Result<(
  models::RecordApiReadPathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// RecordApiRead - GET /records/{collectionId}/{recordId}
#[tracing::instrument(skip_all)]
async fn record_api_read<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiReadPathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_read_validation(
        path_params,
    )
  ).await.unwrap();

  let Ok((
    path_params,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_read(
      &method,
      &host,
      &cookies,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiReadResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct RecordApiSplitRecordBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::SplitRecordRequest,
    }


#[tracing::instrument(skip_all)]
fn record_api_split_record_validation(
  path_params: models::RecordApiSplitRecordPathParams,
        body: models::SplitRecordRequest,
) -> std::result::Result<(
  models::RecordApiSplitRecordPathParams,
        models::SplitRecordRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = RecordApiSplitRecordBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// RecordApiSplitRecord - POST /records/{collectionId}/{recordId}/split
#[tracing::instrument(skip_all)]
async fn record_api_split_record<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiSplitRecordPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::SplitRecordRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_split_record_validation(
        path_params,
          body,
    )
  ).await.unwrap();

  let Ok((
    path_params,
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_split_record(
      &method,
      &host,
      &cookies,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiSplitRecordResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct RecordApiUpdateBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::CreateOrUpdateRecord,
    }


#[tracing::instrument(skip_all)]
fn record_api_update_validation(
  path_params: models::RecordApiUpdatePathParams,
        body: models::CreateOrUpdateRecord,
) -> std::result::Result<(
  models::RecordApiUpdatePathParams,
        models::CreateOrUpdateRecord,
), ValidationErrors>
{
  path_params.validate()?;
              let b = RecordApiUpdateBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// RecordApiUpdate - PUT /records/{collectionId}/{recordId}
#[tracing::instrument(skip_all)]
async fn record_api_update<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiUpdatePathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::CreateOrUpdateRecord>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_update_validation(
        path_params,
          body,
    )
  ).await.unwrap();

  let Ok((
    path_params,
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_update(
      &method,
      &host,
      &cookies,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiUpdateResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct RecordApiUploadImageBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::UploadImageRequest,
    }


#[tracing::instrument(skip_all)]
fn record_api_upload_image_validation(
  path_params: models::RecordApiUploadImagePathParams,
        body: models::UploadImageRequest,
) -> std::result::Result<(
  models::RecordApiUploadImagePathParams,
        models::UploadImageRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = RecordApiUploadImageBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// RecordApiUploadImage - POST /records/{collectionId}/{recordId}/images/{fieldName}
#[tracing::instrument(skip_all)]
async fn record_api_upload_image<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  Path(path_params): Path<models::RecordApiUploadImagePathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::UploadImageRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::record::Record<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    record_api_upload_image_validation(
        path_params,
          body,
    )
  ).await.unwrap();

  let Ok((
    path_params,
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().record_api_upload_image(
      &method,
      &host,
      &cookies,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::record::RecordApiUploadImageResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
                                                    (body)
                                                => {
                                                  let mut response = response.status(201);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

    #[derive(validator::Validate)]
    #[allow(dead_code)]
    struct ScriptApiExecuteBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::ExecuteScriptRequest,
    }


#[tracing::instrument(skip_all)]
fn script_api_execute_validation(
        body: models::ExecuteScriptRequest,
) -> std::result::Result<(
        models::ExecuteScriptRequest,
), ValidationErrors>
{
              let b = ScriptApiExecuteBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// ScriptApiExecute - POST /scripts
#[tracing::instrument(skip_all)]
async fn script_api_execute<I, A, E>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
          Json(body): Json<models::ExecuteScriptRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::script::Script<E> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    script_api_execute_validation(
          body,
    )
  ).await.unwrap();

  let Ok((
      body,
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().script_api_execute(
      &method,
      &host,
      &cookies,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::script::ScriptApiExecuteResponse::Status200_TheRequestHasSucceeded
                                                    (body)
                                                => {
                                                  let mut response = response.status(200);
                                                  {
                                                    let mut response_headers = response.headers_mut().unwrap();
                                                    response_headers.insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json").map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })?);
                                                  }

                                                  let body_content =  tokio::task::spawn_blocking(move ||
                                                      serde_json::to_vec(&body).map_err(|e| {
                                                        error!(error = ?e);
                                                        StatusCode::INTERNAL_SERVER_ERROR
                                                      })).await.unwrap()?;
                                                  response.body(Body::from(body_content))
                                                },
                                            },
                                            Err(why) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                return api_impl.as_ref().handle_error(&method, &host, &cookies, why).await;
                                            },
                                        };

                                        resp.map_err(|e| { error!(error = ?e); StatusCode::INTERNAL_SERVER_ERROR })
}

