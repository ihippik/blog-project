use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Blog post domain model.
#[derive(Serialize, Deserialize)]
pub struct Post {
    /// Unique post identifier.
    pub id: Uuid,

    /// Identifier of the post author.
    pub author_id: Uuid,

    /// Post title.
    pub title: String,

    /// Post content.
    pub content: String,

    /// Post creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Post deletion timestamp, if deleted.
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Post {
    /// Creates a new post instance.
    ///
    /// Generates a new UUID and sets the creation timestamp.
    pub(crate) fn new(title: String, content: String, author_id: Uuid) -> Self {
        Post {
            id: Uuid::new_v4(),
            author_id,
            title,
            content,
            created_at: Utc::now(),
            deleted_at: None,
        }
    }
}
