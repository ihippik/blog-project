pub mod auth;
pub mod dto;
pub mod handler;
pub mod middleware;
pub mod grpc_service;

pub mod blog {
    tonic::include_proto!("blog");
}

