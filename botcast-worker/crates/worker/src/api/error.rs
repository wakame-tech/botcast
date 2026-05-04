use crate::error::Error;
use axum::response::IntoResponse;
use reqwest::StatusCode;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}
