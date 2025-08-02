use super::ApiImpl;
use anyhow::Result;
use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use openapi::{
    apis::auth::{Auth, MeGetResponse, SignInPostResponse, SignUpPostResponse},
    models::{self, SignInRequest, SignInResponse, SignInErrorResponse, SignUpRequest, SignUpResponse, SignUpErrorResponse},
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
        body: &SignInRequest,
    ) -> Result<SignInPostResponse> {
        let session = self
            .auth_client
            .login_with_email(&body.email, &body.password)
            .await;
        match session {
            Ok(session) => Ok(SignInPostResponse::Status200_OK(
                SignInResponse {
                    access_token: session.access_token,
                },
            )),
            Err(e) => {
                tracing::error!("sign_in_post: {}", e);
                Ok(SignInPostResponse::Status404_NotFound(
                    SignInErrorResponse {
                        message: Some("User not found".to_string()),
                    },
                ))
            }
        }
    }

    async fn sign_up_post(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        body: &SignUpRequest,
    ) -> Result<SignUpPostResponse> {
        let session = self
            .auth_client
            .sign_up_with_email_and_password(&body.email, &body.password, None)
            .await;
        match session {
            Ok(_signup_result) => {
                // After successful signup, attempt to login to get access token
                let login_result = self
                    .auth_client
                    .login_with_email(&body.email, &body.password)
                    .await;
                    
                match login_result {
                    Ok(login_session) => Ok(SignUpPostResponse::Status200_OK(
                        SignUpResponse {
                            access_token: login_session.access_token,
                        },
                    )),
                    Err(e) => {
                        tracing::error!("sign_up_post: auto-login failed after signup: {}", e);
                        Ok(SignUpPostResponse::Status400_BadRequest(
                            SignUpErrorResponse {
                                message: Some("Registration succeeded but auto-login failed".to_string()),
                            },
                        ))
                    }
                }
            },
            Err(e) => {
                tracing::error!("sign_up_post: {}", e);
                if e.to_string().contains("User already registered") || e.to_string().contains("already been registered") {
                    Ok(SignUpPostResponse::Status409_Conflict(
                        SignUpErrorResponse {
                            message: Some("User already exists".to_string()),
                        },
                    ))
                } else {
                    Ok(SignUpPostResponse::Status400_BadRequest(
                        SignUpErrorResponse {
                            message: Some("Registration failed".to_string()),
                        },
                    ))
                }
            }
        }
    }
}
