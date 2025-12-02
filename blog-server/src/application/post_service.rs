use std::sync::Arc;

use tracing::instrument;

use crate::data::post_repository::PostRepository;
use crate::domain::error::DomainError;
use crate::domain::post::Post;

#[derive(Clone)]
pub struct PostService<R: PostRepository + 'static> {
    repo: Arc<R>,
}

impl<R> PostService<R>
where
    R: PostRepository + 'static,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo}
    }

    pub async fn get_post(&self, id: uuid::Uuid) -> Result<Post, DomainError> {
        self.repo
            .get(id)
            .await
            .map_err(DomainError::from)?
            .ok_or_else(|| DomainError::UserNotFound(format!("user {}", id)))
    }

    pub async fn list_posts(&self, author_id: uuid::Uuid) -> Result<Vec<Post>, DomainError> {
        self.repo.list(author_id).await.map_err(DomainError::from)
    }
}
