use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    NotFound,
    Timeout,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => {
                eprintln!("ERROR: unauthorized request: {msg}");
                (StatusCode::UNAUTHORIZED, msg)
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            AppError::Timeout => (StatusCode::REQUEST_TIMEOUT, "request timed out".to_string()),
            AppError::Internal(msg) => {
                eprintln!("ERROR: internal error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message}))).into_response()
    }
}

impl From<crate::identifier::IdentifierError> for AppError {
    fn from(err: crate::identifier::IdentifierError) -> Self {
        AppError::BadRequest(err.to_string())
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::BadRequest(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    // Dont' need the error message
    fn from(_err: tokio::time::error::Elapsed) -> Self {
        AppError::Timeout
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<std::num::TryFromIntError> for AppError {
    fn from(err: std::num::TryFromIntError) -> Self {
        AppError::Internal(err.to_string())
    }
}
