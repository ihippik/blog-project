use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Post {
    id: Uuid,
    author_id: Uuid,
    title: String,
    content: String,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}
