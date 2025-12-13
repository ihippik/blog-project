use crate::domain::error::DomainError;
use crate::domain::post::Post;
use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use tracing::{error, info};
use uuid::Uuid;

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: Post) -> Result<Post, DomainError>;
    async fn update(&self, post: Post) -> Result<Post, DomainError>;
    async fn get(&self, id: Uuid) -> Result<Option<Post>, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn list(&self, author_id: Uuid) -> Result<Vec<Post>, DomainError>;
}

#[derive(Clone)]
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, post: Post) -> Result<Post, DomainError> {
        sqlx::query(
            r#"
            INSERT INTO posts (id, author_id, title, content, created_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(post.id)
        .bind(post.author_id)
        .bind(&post.title)
        .bind(&post.content)
        .bind(&post.created_at)
        .bind(&post.deleted_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to create user: {}", e);
            DomainError::Internal(format!("database error: {}", e))
        })?;

        info!(post_id = %post.id, title = %post.title, "post created");
        Ok(post)
    }

    async fn update(&self, post: Post) -> Result<Post, DomainError> {
        sqlx::query(
            r#"
            UPDATE posts
            SET title = $2, content = $3
            WHERE id = $1
            "#,
        )
        .bind(post.id)
        .bind(&post.title)
        .bind(&post.content)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to update post: {}", e);
            DomainError::Internal(format!("database error: {}", e))
        })?;

        info!(post_id = %post.id, title = %post.title, "post updated");

        Ok(post)
    }
    async fn get(&self, id: Uuid) -> Result<Option<Post>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, author_id , title, content, created_at, deleted_at
            FROM posts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to find post by id {}: {}", id, e);
            DomainError::Internal(format!("database error: {}", e))
        })?;

        Ok(row.map(|row| Post {
            id: row.get("id"),
            author_id: row.get("author_id"),
            title: row.get("title"),
            content: row.get("content"),
            created_at: row.get("created_at"),
            deleted_at: row.get("deleted_at"),
        }))
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query!(
            r#"
        DELETE FROM posts WHERE id = $1
        "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::PostNotFound(id.to_string()));
        }

        Ok(())
    }

    async fn list(&self, author_id: Uuid) -> Result<Vec<Post>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, author_id, title, content,created_at, deleted_at
            FROM posts
            WHERE author_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(author_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to list posts for author {}: {}", author_id, e);
            DomainError::Internal(format!("database error: {}", e))
        })?;

        rows.into_iter().map(map_row).collect::<Result<Vec<_>, _>>()
    }
}

fn map_row(row: PgRow) -> Result<Post, DomainError> {
    let decode_err = |e: sqlx::Error| DomainError::Internal(format!("row decode error: {}", e));

    Ok(Post {
        id: row.try_get("id").map_err(decode_err)?,
        author_id: row.try_get("author_id").map_err(decode_err)?,
        title: row.try_get("title").map_err(decode_err)?,
        content: row.try_get("content").map_err(decode_err)?,
        created_at: row.try_get("created_at").map_err(decode_err)?,
        deleted_at: row.try_get("deleted_at").map_err(decode_err)?,
    })
}
