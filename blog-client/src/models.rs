use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Authentication response returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// JWT access token, if authentication was successful.
    pub access_token: Option<String>,

    /// Authenticated user information, if available.
    pub user: Option<User>,
}

/// User model returned by the client API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier.
    pub id: Uuid,

    /// User display name.
    pub username: String,

    /// User email address.
    pub email: String,
}

/// Blog post model returned by the client API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    /// Unique post identifier.
    pub id: Uuid,

    /// Post title.
    pub title: String,

    /// Post content.
    pub content: String,

    /// Post author identifier.
    pub author_id: Uuid,

    /// Post creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Post update timestamp, if updated.
    pub updated_at: Option<DateTime<Utc>>,
}
