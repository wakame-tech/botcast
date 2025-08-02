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
pub fn new<I, A, E, C>(api_impl: I) -> Router
where
    I: AsRef<A> + Clone + Send + Sync + 'static,
    A: apis::auth::Auth<E, Claims = C> + apis::corners::Corners<E, Claims = C> + apis::episodes::Episodes<E, Claims = C> + apis::mails::Mails<E, Claims = C> + apis::podcasts::Podcasts<E, Claims = C> + apis::scripts::Scripts<E, Claims = C> + apis::secrets::Secrets<E, Claims = C> + apis::tasks::Tasks<E, Claims = C> + apis::ApiAuthBasic<Claims = C> + Send + Sync + 'static,
    E: std::fmt::Debug + Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    // build our application with a route
    Router::new()
        .route("/corners/{corner_id}",
            delete(corners_corner_id_delete::<I, A, E, C>).get(corners_corner_id_get::<I, A, E, C>).put(corners_corner_id_put::<I, A, E, C>)
        )
        .route("/corners/{corner_id}/mails",
            get(corners_corner_id_mails_get::<I, A, E, C>).post(corners_corner_id_mails_post::<I, A, E, C>)
        )
        .route("/corners/{corner_id}/mails/{mail_id}",
            delete(corners_corner_id_mails_mail_id_delete::<I, A, E, C>)
        )
        .route("/episodes",
            post(episodes_post::<I, A, E, C>)
        )
        .route("/episodes/{episode_id}",
            delete(episodes_episode_id_delete::<I, A, E, C>).get(episodes_episode_id_get::<I, A, E, C>).put(episodes_episode_id_put::<I, A, E, C>)
        )
        .route("/me",
            get(me_get::<I, A, E, C>)
        )
        .route("/podcast/{podcast_id}",
            delete(podcast_podcast_id_delete::<I, A, E, C>).get(podcast_podcast_id_get::<I, A, E, C>).put(podcast_podcast_id_put::<I, A, E, C>)
        )
        .route("/podcasts",
            get(podcasts_get::<I, A, E, C>).post(podcasts_post::<I, A, E, C>)
        )
        .route("/scripts",
            get(scripts_get::<I, A, E, C>).post(scripts_post::<I, A, E, C>)
        )
        .route("/scripts/{script_id}",
            delete(scripts_script_id_delete::<I, A, E, C>).get(scripts_script_id_get::<I, A, E, C>).put(scripts_script_id_put::<I, A, E, C>)
        )
        .route("/secrets",
            get(secrets_get::<I, A, E, C>).post(secrets_post::<I, A, E, C>)
        )
        .route("/signIn",
            post(sign_in_post::<I, A, E, C>)
        )
        .route("/signUp",
            post(sign_up_post::<I, A, E, C>)
        )
        .route("/tasks",
            get(tasks_get::<I, A, E, C>).post(tasks_post::<I, A, E, C>)
        )
        .route("/tasks/{task_id}",
            delete(tasks_task_id_delete::<I, A, E, C>).put(tasks_task_id_put::<I, A, E, C>)
        )
        .route("/topPodcasts",
            get(top_podcasts_get::<I, A, E, C>)
        )
        .with_state(api_impl)
}


#[tracing::instrument(skip_all)]
fn me_get_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// MeGet - GET /me
#[tracing::instrument(skip_all)]
async fn me_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::auth::Auth<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    me_get_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().me_get(
      &method,
      &host,
      &cookies,
        &claims,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::auth::MeGetResponse::Status200_OK
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
                                                apis::auth::MeGetResponse::Status404_NotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
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
    struct SignInPostBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::SignInRequest,
    }


#[tracing::instrument(skip_all)]
fn sign_in_post_validation(
        body: models::SignInRequest,
) -> std::result::Result<(
        models::SignInRequest,
), ValidationErrors>
{
              let b = SignInPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// SignInPost - POST /signIn
#[tracing::instrument(skip_all)]
async fn sign_in_post<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
          Json(body): Json<models::SignInRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::auth::Auth<E, Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    sign_in_post_validation(
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

  let result = api_impl.as_ref().sign_in_post(
      &method,
      &host,
      &cookies,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::auth::SignInPostResponse::Status200_OK
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
                                                apis::auth::SignInPostResponse::Status404_NotFound
                                                    (body)
                                                => {
                                                  let mut response = response.status(404);
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
    struct SignUpPostBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::SignUpRequest,
    }


#[tracing::instrument(skip_all)]
fn sign_up_post_validation(
        body: models::SignUpRequest,
) -> std::result::Result<(
        models::SignUpRequest,
), ValidationErrors>
{
              let b = SignUpPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// SignUpPost - POST /signUp
#[tracing::instrument(skip_all)]
async fn sign_up_post<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
 State(api_impl): State<I>,
          Json(body): Json<models::SignUpRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::auth::Auth<E, Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    sign_up_post_validation(
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

  let result = api_impl.as_ref().sign_up_post(
      &method,
      &host,
      &cookies,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::auth::SignUpPostResponse::Status200_OK
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
                                                apis::auth::SignUpPostResponse::Status400_BadRequest
                                                    (body)
                                                => {
                                                  let mut response = response.status(400);
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
                                                apis::auth::SignUpPostResponse::Status409_Conflict
                                                    (body)
                                                => {
                                                  let mut response = response.status(409);
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
fn corners_corner_id_delete_validation(
  path_params: models::CornersCornerIdDeletePathParams,
) -> std::result::Result<(
  models::CornersCornerIdDeletePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// CornersCornerIdDelete - DELETE /corners/{cornerId}
#[tracing::instrument(skip_all)]
async fn corners_corner_id_delete<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::CornersCornerIdDeletePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::corners::Corners<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    corners_corner_id_delete_validation(
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

  let result = api_impl.as_ref().corners_corner_id_delete(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::corners::CornersCornerIdDeleteResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn corners_corner_id_get_validation(
  path_params: models::CornersCornerIdGetPathParams,
) -> std::result::Result<(
  models::CornersCornerIdGetPathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// CornersCornerIdGet - GET /corners/{cornerId}
#[tracing::instrument(skip_all)]
async fn corners_corner_id_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::CornersCornerIdGetPathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::corners::Corners<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    corners_corner_id_get_validation(
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

  let result = api_impl.as_ref().corners_corner_id_get(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::corners::CornersCornerIdGetResponse::Status200_OK
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
    struct CornersCornerIdPutBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::CornersCornerIdPutRequest,
    }


#[tracing::instrument(skip_all)]
fn corners_corner_id_put_validation(
  path_params: models::CornersCornerIdPutPathParams,
        body: models::CornersCornerIdPutRequest,
) -> std::result::Result<(
  models::CornersCornerIdPutPathParams,
        models::CornersCornerIdPutRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = CornersCornerIdPutBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// CornersCornerIdPut - PUT /corners/{cornerId}
#[tracing::instrument(skip_all)]
async fn corners_corner_id_put<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::CornersCornerIdPutPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::CornersCornerIdPutRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::corners::Corners<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    corners_corner_id_put_validation(
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

  let result = api_impl.as_ref().corners_corner_id_put(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::corners::CornersCornerIdPutResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn episodes_episode_id_delete_validation(
  path_params: models::EpisodesEpisodeIdDeletePathParams,
) -> std::result::Result<(
  models::EpisodesEpisodeIdDeletePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// EpisodesEpisodeIdDelete - DELETE /episodes/{episodeId}
#[tracing::instrument(skip_all)]
async fn episodes_episode_id_delete<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::EpisodesEpisodeIdDeletePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::episodes::Episodes<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    episodes_episode_id_delete_validation(
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

  let result = api_impl.as_ref().episodes_episode_id_delete(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::episodes::EpisodesEpisodeIdDeleteResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn episodes_episode_id_get_validation(
  path_params: models::EpisodesEpisodeIdGetPathParams,
) -> std::result::Result<(
  models::EpisodesEpisodeIdGetPathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// EpisodesEpisodeIdGet - GET /episodes/{episodeId}
#[tracing::instrument(skip_all)]
async fn episodes_episode_id_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::EpisodesEpisodeIdGetPathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::episodes::Episodes<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    episodes_episode_id_get_validation(
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

  let result = api_impl.as_ref().episodes_episode_id_get(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::episodes::EpisodesEpisodeIdGetResponse::Status200_OK
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
    struct EpisodesEpisodeIdPutBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::EpisodesEpisodeIdPutRequest,
    }


#[tracing::instrument(skip_all)]
fn episodes_episode_id_put_validation(
  path_params: models::EpisodesEpisodeIdPutPathParams,
        body: models::EpisodesEpisodeIdPutRequest,
) -> std::result::Result<(
  models::EpisodesEpisodeIdPutPathParams,
        models::EpisodesEpisodeIdPutRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = EpisodesEpisodeIdPutBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// EpisodesEpisodeIdPut - PUT /episodes/{episodeId}
#[tracing::instrument(skip_all)]
async fn episodes_episode_id_put<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::EpisodesEpisodeIdPutPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::EpisodesEpisodeIdPutRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::episodes::Episodes<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    episodes_episode_id_put_validation(
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

  let result = api_impl.as_ref().episodes_episode_id_put(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::episodes::EpisodesEpisodeIdPutResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
    struct EpisodesPostBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::EpisodesPostRequest,
    }


#[tracing::instrument(skip_all)]
fn episodes_post_validation(
        body: models::EpisodesPostRequest,
) -> std::result::Result<(
        models::EpisodesPostRequest,
), ValidationErrors>
{
              let b = EpisodesPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// EpisodesPost - POST /episodes
#[tracing::instrument(skip_all)]
async fn episodes_post<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::EpisodesPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::episodes::Episodes<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    episodes_post_validation(
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

  let result = api_impl.as_ref().episodes_post(
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::episodes::EpisodesPostResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn corners_corner_id_mails_get_validation(
  path_params: models::CornersCornerIdMailsGetPathParams,
) -> std::result::Result<(
  models::CornersCornerIdMailsGetPathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// CornersCornerIdMailsGet - GET /corners/{cornerId}/mails
#[tracing::instrument(skip_all)]
async fn corners_corner_id_mails_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::CornersCornerIdMailsGetPathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::mails::Mails<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    corners_corner_id_mails_get_validation(
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

  let result = api_impl.as_ref().corners_corner_id_mails_get(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::mails::CornersCornerIdMailsGetResponse::Status200_OK
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
fn corners_corner_id_mails_mail_id_delete_validation(
  path_params: models::CornersCornerIdMailsMailIdDeletePathParams,
) -> std::result::Result<(
  models::CornersCornerIdMailsMailIdDeletePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// CornersCornerIdMailsMailIdDelete - DELETE /corners/{cornerId}/mails/{mailId}
#[tracing::instrument(skip_all)]
async fn corners_corner_id_mails_mail_id_delete<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::CornersCornerIdMailsMailIdDeletePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::mails::Mails<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    corners_corner_id_mails_mail_id_delete_validation(
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

  let result = api_impl.as_ref().corners_corner_id_mails_mail_id_delete(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::mails::CornersCornerIdMailsMailIdDeleteResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
    struct CornersCornerIdMailsPostBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::CornersCornerIdMailsPostRequest,
    }


#[tracing::instrument(skip_all)]
fn corners_corner_id_mails_post_validation(
  path_params: models::CornersCornerIdMailsPostPathParams,
        body: models::CornersCornerIdMailsPostRequest,
) -> std::result::Result<(
  models::CornersCornerIdMailsPostPathParams,
        models::CornersCornerIdMailsPostRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = CornersCornerIdMailsPostBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// CornersCornerIdMailsPost - POST /corners/{cornerId}/mails
#[tracing::instrument(skip_all)]
async fn corners_corner_id_mails_post<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::CornersCornerIdMailsPostPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::CornersCornerIdMailsPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::mails::Mails<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    corners_corner_id_mails_post_validation(
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

  let result = api_impl.as_ref().corners_corner_id_mails_post(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::mails::CornersCornerIdMailsPostResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn podcast_podcast_id_delete_validation(
  path_params: models::PodcastPodcastIdDeletePathParams,
) -> std::result::Result<(
  models::PodcastPodcastIdDeletePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// PodcastPodcastIdDelete - DELETE /podcast/{podcastId}
#[tracing::instrument(skip_all)]
async fn podcast_podcast_id_delete<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::PodcastPodcastIdDeletePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::podcasts::Podcasts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    podcast_podcast_id_delete_validation(
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

  let result = api_impl.as_ref().podcast_podcast_id_delete(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::podcasts::PodcastPodcastIdDeleteResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn podcast_podcast_id_get_validation(
  path_params: models::PodcastPodcastIdGetPathParams,
) -> std::result::Result<(
  models::PodcastPodcastIdGetPathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// PodcastPodcastIdGet - GET /podcast/{podcastId}
#[tracing::instrument(skip_all)]
async fn podcast_podcast_id_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::PodcastPodcastIdGetPathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::podcasts::Podcasts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    podcast_podcast_id_get_validation(
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

  let result = api_impl.as_ref().podcast_podcast_id_get(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::podcasts::PodcastPodcastIdGetResponse::Status200_OK
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
    struct PodcastPodcastIdPutBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::PodcastsPostRequest,
    }


#[tracing::instrument(skip_all)]
fn podcast_podcast_id_put_validation(
  path_params: models::PodcastPodcastIdPutPathParams,
        body: models::PodcastsPostRequest,
) -> std::result::Result<(
  models::PodcastPodcastIdPutPathParams,
        models::PodcastsPostRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = PodcastPodcastIdPutBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// PodcastPodcastIdPut - PUT /podcast/{podcastId}
#[tracing::instrument(skip_all)]
async fn podcast_podcast_id_put<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::PodcastPodcastIdPutPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::PodcastsPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::podcasts::Podcasts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    podcast_podcast_id_put_validation(
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

  let result = api_impl.as_ref().podcast_podcast_id_put(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::podcasts::PodcastPodcastIdPutResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn podcasts_get_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// PodcastsGet - GET /podcasts
#[tracing::instrument(skip_all)]
async fn podcasts_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::podcasts::Podcasts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    podcasts_get_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().podcasts_get(
      &method,
      &host,
      &cookies,
        &claims,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::podcasts::PodcastsGetResponse::Status200_OK
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
    struct PodcastsPostBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::PodcastsPostRequest,
    }


#[tracing::instrument(skip_all)]
fn podcasts_post_validation(
        body: models::PodcastsPostRequest,
) -> std::result::Result<(
        models::PodcastsPostRequest,
), ValidationErrors>
{
              let b = PodcastsPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// PodcastsPost - POST /podcasts
#[tracing::instrument(skip_all)]
async fn podcasts_post<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::PodcastsPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::podcasts::Podcasts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    podcasts_post_validation(
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

  let result = api_impl.as_ref().podcasts_post(
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::podcasts::PodcastsPostResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn top_podcasts_get_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// TopPodcastsGet - GET /topPodcasts
#[tracing::instrument(skip_all)]
async fn top_podcasts_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::podcasts::Podcasts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    top_podcasts_get_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().top_podcasts_get(
      &method,
      &host,
      &cookies,
        &claims,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::podcasts::TopPodcastsGetResponse::Status200_OK
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
fn scripts_get_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// ScriptsGet - GET /scripts
#[tracing::instrument(skip_all)]
async fn scripts_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::scripts::Scripts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    scripts_get_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().scripts_get(
      &method,
      &host,
      &cookies,
        &claims,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::scripts::ScriptsGetResponse::Status200_OK
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
    struct ScriptsPostBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::ScriptsPostRequest,
    }


#[tracing::instrument(skip_all)]
fn scripts_post_validation(
        body: models::ScriptsPostRequest,
) -> std::result::Result<(
        models::ScriptsPostRequest,
), ValidationErrors>
{
              let b = ScriptsPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// ScriptsPost - POST /scripts
#[tracing::instrument(skip_all)]
async fn scripts_post<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::ScriptsPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::scripts::Scripts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    scripts_post_validation(
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

  let result = api_impl.as_ref().scripts_post(
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::scripts::ScriptsPostResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn scripts_script_id_delete_validation(
  path_params: models::ScriptsScriptIdDeletePathParams,
) -> std::result::Result<(
  models::ScriptsScriptIdDeletePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// ScriptsScriptIdDelete - DELETE /scripts/{scriptId}
#[tracing::instrument(skip_all)]
async fn scripts_script_id_delete<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::ScriptsScriptIdDeletePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::scripts::Scripts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    scripts_script_id_delete_validation(
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

  let result = api_impl.as_ref().scripts_script_id_delete(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::scripts::ScriptsScriptIdDeleteResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn scripts_script_id_get_validation(
  path_params: models::ScriptsScriptIdGetPathParams,
) -> std::result::Result<(
  models::ScriptsScriptIdGetPathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// ScriptsScriptIdGet - GET /scripts/{scriptId}
#[tracing::instrument(skip_all)]
async fn scripts_script_id_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::ScriptsScriptIdGetPathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::scripts::Scripts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    scripts_script_id_get_validation(
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

  let result = api_impl.as_ref().scripts_script_id_get(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::scripts::ScriptsScriptIdGetResponse::Status200_OK
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
    struct ScriptsScriptIdPutBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::ScriptsScriptIdPutRequest,
    }


#[tracing::instrument(skip_all)]
fn scripts_script_id_put_validation(
  path_params: models::ScriptsScriptIdPutPathParams,
        body: models::ScriptsScriptIdPutRequest,
) -> std::result::Result<(
  models::ScriptsScriptIdPutPathParams,
        models::ScriptsScriptIdPutRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = ScriptsScriptIdPutBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// ScriptsScriptIdPut - PUT /scripts/{scriptId}
#[tracing::instrument(skip_all)]
async fn scripts_script_id_put<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::ScriptsScriptIdPutPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::ScriptsScriptIdPutRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::scripts::Scripts<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    scripts_script_id_put_validation(
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

  let result = api_impl.as_ref().scripts_script_id_put(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::scripts::ScriptsScriptIdPutResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn secrets_get_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// SecretsGet - GET /secrets
#[tracing::instrument(skip_all)]
async fn secrets_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::secrets::Secrets<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    secrets_get_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().secrets_get(
      &method,
      &host,
      &cookies,
        &claims,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::secrets::SecretsGetResponse::Status200_OK
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
    struct SecretsPostBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::SecretsPostRequest,
    }


#[tracing::instrument(skip_all)]
fn secrets_post_validation(
        body: models::SecretsPostRequest,
) -> std::result::Result<(
        models::SecretsPostRequest,
), ValidationErrors>
{
              let b = SecretsPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// SecretsPost - POST /secrets
#[tracing::instrument(skip_all)]
async fn secrets_post<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::SecretsPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::secrets::Secrets<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    secrets_post_validation(
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

  let result = api_impl.as_ref().secrets_post(
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::secrets::SecretsPostResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn tasks_get_validation(
) -> std::result::Result<(
), ValidationErrors>
{

Ok((
))
}
/// TasksGet - GET /tasks
#[tracing::instrument(skip_all)]
async fn tasks_get<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::tasks::Tasks<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    tasks_get_validation(
    )
  ).await.unwrap();

  let Ok((
  )) = validation else {
    return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(validation.unwrap_err().to_string()))
            .map_err(|_| StatusCode::BAD_REQUEST);
  };

  let result = api_impl.as_ref().tasks_get(
      &method,
      &host,
      &cookies,
        &claims,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::tasks::TasksGetResponse::Status200_OK
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
    struct TasksPostBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::TasksPostRequest,
    }


#[tracing::instrument(skip_all)]
fn tasks_post_validation(
        body: models::TasksPostRequest,
) -> std::result::Result<(
        models::TasksPostRequest,
), ValidationErrors>
{
              let b = TasksPostBodyValidator { body: &body };
              b.validate()?;

Ok((
    body,
))
}
/// TasksPost - POST /tasks
#[tracing::instrument(skip_all)]
async fn tasks_post<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
 State(api_impl): State<I>,
          Json(body): Json<models::TasksPostRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::tasks::Tasks<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    tasks_post_validation(
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

  let result = api_impl.as_ref().tasks_post(
      &method,
      &host,
      &cookies,
        &claims,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::tasks::TasksPostResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
fn tasks_task_id_delete_validation(
  path_params: models::TasksTaskIdDeletePathParams,
) -> std::result::Result<(
  models::TasksTaskIdDeletePathParams,
), ValidationErrors>
{
  path_params.validate()?;

Ok((
  path_params,
))
}
/// TasksTaskIdDelete - DELETE /tasks/{taskId}
#[tracing::instrument(skip_all)]
async fn tasks_task_id_delete<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::TasksTaskIdDeletePathParams>,
 State(api_impl): State<I>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::tasks::Tasks<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    tasks_task_id_delete_validation(
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

  let result = api_impl.as_ref().tasks_task_id_delete(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::tasks::TasksTaskIdDeleteResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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
    struct TasksTaskIdPutBodyValidator<'a> {
            #[validate(nested)]
          body: &'a models::TasksTaskIdPutRequest,
    }


#[tracing::instrument(skip_all)]
fn tasks_task_id_put_validation(
  path_params: models::TasksTaskIdPutPathParams,
        body: models::TasksTaskIdPutRequest,
) -> std::result::Result<(
  models::TasksTaskIdPutPathParams,
        models::TasksTaskIdPutRequest,
), ValidationErrors>
{
  path_params.validate()?;
              let b = TasksTaskIdPutBodyValidator { body: &body };
              b.validate()?;

Ok((
  path_params,
    body,
))
}
/// TasksTaskIdPut - PUT /tasks/{taskId}
#[tracing::instrument(skip_all)]
async fn tasks_task_id_put<I, A, E, C>(
  method: Method,
  host: Host,
  cookies: CookieJar,
  headers: HeaderMap,
  Path(path_params): Path<models::TasksTaskIdPutPathParams>,
 State(api_impl): State<I>,
          Json(body): Json<models::TasksTaskIdPutRequest>,
) -> Result<Response, StatusCode>
where
    I: AsRef<A> + Send + Sync,
    A: apis::tasks::Tasks<E, Claims = C>+ apis::ApiAuthBasic<Claims = C> + Send + Sync,
    E: std::fmt::Debug + Send + Sync + 'static,
        {
    // Authentication
    let claims_in_auth_header = api_impl.as_ref().extract_claims_from_auth_header(apis::BasicAuthKind::Bearer, &headers, "authorization").await;
    let claims = None
             .or(claims_in_auth_header)
          ;
    let Some(claims) = claims else {
        return Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    };


      #[allow(clippy::redundant_closure)]
      let validation = tokio::task::spawn_blocking(move ||
    tasks_task_id_put_validation(
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

  let result = api_impl.as_ref().tasks_task_id_put(
      &method,
      &host,
      &cookies,
        &claims,
        &path_params,
              &body,
  ).await;

  let mut response = Response::builder();

  let resp = match result {
                                            Ok(rsp) => match rsp {
                                                apis::tasks::TasksTaskIdPutResponse::Status200_OK
                                                => {
                                                  let mut response = response.status(200);
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

