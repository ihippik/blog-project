use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User domain model.
#[derive(Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier.
    pub id: Uuid,

    /// User display name.
    pub username: String,

    /// User email address.
    pub email: String,

    /// Hashed user password.
    pub password_hash: String,

    /// User creation timestamp.
    pub created_at: DateTime<Utc>,

    /// User deletion timestamp, if deleted.
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    /// Creates a new user instance.
    ///
    /// Generates a new UUID and sets the creation timestamp.
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            created_at: Utc::now(),
            deleted_at: None,
        }
    }
}
