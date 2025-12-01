use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;
use serde_json::json;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("internal error: {0}")]
    Internal(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("user already exists: {0}")]
    UserAlreadyExists(Uuid),
    #[error("user not found: {0}")]
    UserNotFound(String),
    #[error("post not found: {0}")]
    PostNotFound(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("invalid credentials: {0}")]
    InvalidCredentials(String),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ResponseError for DomainError {
    fn status_code(&self) -> StatusCode {
        match self {
            DomainError::Validation(_) => StatusCode::BAD_REQUEST,
            DomainError::UserNotFound(_) => StatusCode::NOT_FOUND,
            DomainError::PostNotFound(_) => StatusCode::NOT_FOUND,
            DomainError::UserAlreadyExists(_) => StatusCode::BAD_REQUEST,
            DomainError::Forbidden(..) => StatusCode::UNAUTHORIZED,
            DomainError::InvalidCredentials(..) => StatusCode::UNAUTHORIZED,
            DomainError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = self.to_string();
        let details = match self {
            DomainError::Validation(msg) => Some(json!({ "message": msg })),
            DomainError::UserNotFound(msg) => Some(json!({ "message": msg })),
            DomainError::Forbidden(msg) => Some(json!({ "message": msg })),
            DomainError::Internal(_) => None,
            DomainError::UserAlreadyExists(msg) => Some(json!( { "message": msg })),
            DomainError::PostNotFound(msg) => Some(json!( { "message": msg })),
            DomainError::InvalidCredentials(msg) => Some(json!( { "message": msg })),
        };
        let body = ErrorBody {
            error: &message,
            details,
        };
        HttpResponse::build(self.status_code()).json(body)
    }
}
