use std::fmt::Display;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::identifier;

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

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad request: {msg}"),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized request: {msg}"),
            AppError::NotFound => write!(f, "Not found"),
            AppError::Timeout => write!(f, "Timeout request"),
            AppError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl From<identifier::error::IdentifierError> for AppError {
    fn from(err: identifier::error::IdentifierError) -> Self {
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
