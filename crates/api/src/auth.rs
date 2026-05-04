// @specre 01KQS7YE9H5SMQB6BBNKNSVW8T
use crate::{error::AppError, AppState};
use axum::{Json, extract::State};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

#[derive(Deserialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Serialize, Debug)]
struct AuthParams {
    email: String,
    pass: String,
}

#[derive(Debug, Deserialize)]
struct UserRecord {
    id: RecordId,
}

pub async fn sign_up_handler(
    State(state): State<AppState>,
    Json(body): Json<SignUpRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let _ = state
        .db
        .signup(surrealdb::opt::auth::Record {
            namespace: &state.namespace,
            database: &state.database,
            access: "user",
            params: AuthParams {
                email: body.email.clone(),
                pass: body.password.clone(),
            },
        })
        .await
        .map_err(|e| {
            let msg = e.to_string().to_lowercase();
            if msg.contains("already") || msg.contains("unique") || msg.contains("duplicate") {
                AppError::Conflict {
                    message: "User already exists".into(),
                }
            } else {
                AppError::Internal {
                    message: format!("Signup failed: {e}"),
                }
            }
        })?;

    let token = issue_jwt_for_email(&state, &body.email).await?;
    Ok(Json(AuthResponse { token }))
}

pub async fn sign_in_handler(
    State(state): State<AppState>,
    Json(body): Json<SignInRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let _ = state
        .db
        .signin(surrealdb::opt::auth::Record {
            namespace: &state.namespace,
            database: &state.database,
            access: "user",
            params: AuthParams {
                email: body.email.clone(),
                pass: body.password.clone(),
            },
        })
        .await
        .map_err(|_| AppError::Unauthorized)?;

    let token = issue_jwt_for_email(&state, &body.email).await?;
    Ok(Json(AuthResponse { token }))
}

async fn issue_jwt_for_email(state: &AppState, email: &str) -> Result<String, AppError> {
    let email_str = email.to_string();
    let user: Option<UserRecord> = state
        .db
        .query("SELECT id FROM user WHERE email = $email")
        .bind(("email", email_str))
        .await
        .map_err(AppError::Database)?
        .take(0)
        .map_err(AppError::Database)?;

    let user_id = user
        .ok_or_else(|| AppError::Internal {
            message: "User not found after auth".into(),
        })?
        .id
        .to_string();

    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = JwtClaims { sub: user_id, exp };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal {
        message: format!("JWT encoding failed: {e}"),
    })
}
