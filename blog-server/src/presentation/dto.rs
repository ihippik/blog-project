use crate::domain::post::Post;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User registration request payload.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// Desired username.
    pub username: String,

    /// User email address.
    pub email: String,

    /// User plaintext password.
    pub password: String,
}

/// User login request payload.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// User email address.
    pub email: String,

    /// User plaintext password.
    pub password: String,
}

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: &'static str,

    /// Current server timestamp.
    pub timestamp: DateTime<Utc>,
}

/// JWT token response.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// Access token.
    pub access_token: String,
}

/// Post response payload.
#[derive(Debug, Serialize)]
pub struct PostResponse {
    /// Post identifier.
    pub id: Uuid,

    /// Post author identifier.
    pub author_id: Uuid,

    /// Post title.
    pub title: String,

    /// Post content.
    pub content: String,

    /// Post creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Post creation request payload.
#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    /// Post title.
    pub title: String,

    /// Post content.
    pub content: String,
}

impl From<Post> for PostResponse {
    /// Converts a domain post into an HTTP response DTO.
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            author_id: post.author_id,
            title: post.title,
            content: post.content,
            created_at: post.created_at,
        }
    }
}
