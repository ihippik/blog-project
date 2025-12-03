use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("grpc status: {0}")]
    GrpcStatus(#[from] tonic::Status),

    #[error("grpc transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}