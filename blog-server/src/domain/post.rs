use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}


impl Post {
    pub(crate) fn new(title: String, content: String, author_id: Uuid) -> Self {
        Post{
            id: Uuid::new_v4(),
            author_id,
            title,
            content,
            created_at: Utc::now(),
            deleted_at: None,
        }
    }
}
