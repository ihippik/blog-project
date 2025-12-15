use std::sync::Arc;
use uuid::Uuid;

use crate::data::post_repository::PostRepository;
use crate::domain::error::DomainError;
use crate::domain::post::Post;

/// Post management service.
///
/// Provides CRUD operations for blog posts.
#[derive(Clone)]
pub struct PostService<R: PostRepository + 'static> {
    repo: Arc<R>,
}

impl<R> PostService<R>
where
    R: PostRepository + 'static,
{
    /// Creates a new post service.
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    /// Creates a new post authored by the given user.
    pub async fn create_post(
        &self,
        title: String,
        content: String,
        author_id: Uuid,
    ) -> Result<Post, DomainError> {
        let model = Post::new(title, content, author_id);
        let post = self.repo.create(model).await.map_err(DomainError::from)?;

        Ok(post)
    }

    /// Updates an existing post.
    pub async fn update_post(
        &self,
        id: Uuid,
        title: String,
        content: String,
    ) -> Result<Post, DomainError> {
        let mut post = self.repo
            .get(id)
            .await
            .map_err(DomainError::from)?
            .ok_or_else(|| DomainError::PostNotFound(format!("post id: {}", id)))?;

        post.title = title;
        post.content = content;

        let updated = self.repo.update(post).await.map_err(DomainError::from)?;

        Ok(updated)
    }

    /// Returns a post by its ID.
    pub async fn get_post(&self, id: Uuid) -> Result<Post, DomainError> {
        self.repo
            .get(id)
            .await
            .map_err(DomainError::from)?
            .ok_or_else(|| DomainError::PostNotFound(format!("post id: {}", id)))
    }

    /// Deletes a post by its ID.
    pub async fn delete_post(&self, id: Uuid) -> Result<(), DomainError> {
        self.repo.delete(id).await.map_err(DomainError::from)?;
        Ok(())
    }

    /// Returns a list of posts for the given author.
    pub async fn list_posts(&self, author_id: Uuid) -> Result<Vec<Post>, DomainError> {
        self.repo.list(author_id).await.map_err(DomainError::from)
    }
}
