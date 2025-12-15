use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

/// Domain-level application errors.
///
/// Used across the domain and automatically mapped
/// to HTTP responses in the web layer.
#[derive(Debug, Error)]
pub enum DomainError {
    /// Internal application error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Validation error caused by invalid input.
    #[error("validation error: {0}")]
    Validation(String),

    /// User was not found.
    #[error("user not found: {0}")]
    UserNotFound(String),

    /// Post was not found.
    #[error("post not found: {0}")]
    PostNotFound(String),

    /// Authentication or authorization failure.
    #[error("forbidden: {0}")]
    InvalidCredentials(String),
}

/// HTTP error response body.
#[derive(Serialize)]
struct ErrorBody<'a> {
    /// Human-readable error message.
    error: &'a str,

    /// Optional structured error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ResponseError for DomainError {
    /// Maps domain errors to HTTP status codes.
    fn status_code(&self) -> StatusCode {
        match self {
            DomainError::Validation(_) => StatusCode::BAD_REQUEST,
            DomainError::UserNotFound(_) => StatusCode::NOT_FOUND,
            DomainError::PostNotFound(_) => StatusCode::NOT_FOUND,
            DomainError::InvalidCredentials(_) => StatusCode::UNAUTHORIZED,
            DomainError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Builds an HTTP JSON error response.
    fn error_response(&self) -> HttpResponse {
        let message = self.to_string();
        let details = match self {
            DomainError::Validation(msg)
            | DomainError::UserNotFound(msg)
            | DomainError::PostNotFound(msg)
            | DomainError::InvalidCredentials(msg) => {
                Some(json!({ "message": msg }))
            }
            DomainError::Internal(_) => None,
        };

        let body = ErrorBody {
            error: &message,
            details,
        };

        HttpResponse::build(self.status_code()).json(body)
    }
}
