// @specre 01KNM2BBT51Q6B69XYQXPDTC0K
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use surrealdb::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(#[from] Error),

    #[error("Internal server error: {message}")]
    Internal { message: String },
}

// Helper functions for creating common errors
impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::Validation { .. } => StatusCode::BAD_REQUEST,
            AppError::Conflict { .. } => StatusCode::CONFLICT,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Database(_) | AppError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
