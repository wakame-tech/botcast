use super::ApiImpl;
use anyhow::Result;
use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use openapi::{
    apis::auth::{Auth, MeGetResponse, SignInPostResponse},
    models::{self, SignInPost200Response, SignInPost404Response, SignInPostRequest},
};
use repos::entities::users::Model as User;

pub(crate) fn into_user_model(user: User) -> Result<models::User> {
    Ok(models::User {
        id: user.id,
        auth_id: user.auth_id.parse()?,
        email: user.email.clone(),
        name: user.name.unwrap_or_default(),
    })
}

#[async_trait]
impl Auth<anyhow::Error> for ApiImpl {
    type Claims = User;

    async fn me_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        user: &Self::Claims,
    ) -> Result<MeGetResponse> {
        Ok(MeGetResponse::Status200_OK(into_user_model(user.clone())?))
    }

    async fn sign_in_post(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        body: &SignInPostRequest,
    ) -> Result<SignInPostResponse> {
        let session = self
            .auth_client
            .login_with_email(&body.email, &body.password)
            .await;
        match session {
            Ok(session) => Ok(SignInPostResponse::Status200_OK(
                SignInPost200Response::new(session.access_token),
            )),
            Err(e) => {
                tracing::error!("sign_in_post: {}", e);
                Ok(SignInPostResponse::Status404_NotFound(
                    SignInPost404Response {
                        message: Some("User not found".to_string()),
                    },
                ))
            }
        }
    }
}
