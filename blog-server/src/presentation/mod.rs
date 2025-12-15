/// Authentication utilities.
pub mod auth;

/// Data transfer objects.
pub mod dto;

/// HTTP handlers.
pub mod handler;

/// Middleware.
pub mod middleware;

/// gRPC services.
pub mod grpc_service;

/// Generated gRPC code.
pub mod blog {
    tonic::include_proto!("blog");
}
