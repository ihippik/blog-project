use thiserror::Error;

/// Blog client errors.
#[derive(Debug, Error)]
pub enum BlogClientError {
    /// HTTP transport error.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// gRPC status error returned by the server.
    #[error("grpc status: {0}")]
    GrpcStatus(#[from] tonic::Status),

    /// gRPC transport-level error.
    #[error("grpc transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    /// Serialization or deserialization error.
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Unauthorized request or missing authentication.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// Requested resource was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Invalid client request.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Invalid client state.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// Other client error.
    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}
