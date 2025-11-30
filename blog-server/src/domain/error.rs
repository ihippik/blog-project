use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("internal error: {0}")]
    Internal(String),
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
