use crate::domain::error::DomainError;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use tracing::{error, info};
use uuid::Uuid;

use crate::domain::user::User;

/// User persistence abstraction.
///
/// Defines operations for storing and retrieving users.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Persists a new user.
    async fn create(&self, user: User) -> Result<User, DomainError>;

    /// Returns a user by email, if it exists.
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;

    /// Returns a user by ID, if it exists.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
}

/// PostgreSQL-backed user repository implementation.
#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    /// Creates a new PostgreSQL user repository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    /// Inserts a new user into the database.
    async fn create(&self, user: User) -> Result<User, DomainError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
            .bind(user.id)
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(&user.created_at)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                error!("failed to create user: {}", e);
                if e.as_database_error()
                    .and_then(|db| db.constraint())
                    .map(|c| c.contains("users_email"))
                    == Some(true)
                {
                    DomainError::Validation("email already registered".into())
                } else {
                    DomainError::Internal(format!("database error: {}", e))
                }
            })?;

        info!(user_id = %user.id, email = %user.email, "user created");
        Ok(user)
    }

    /// Returns a user by email, if present.
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at, deleted_at
            FROM users
            WHERE email = $1
            "#,
        )
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                error!("failed to find user by email {}: {}", email, e);
                DomainError::Internal(format!("database error: {}", e))
            })?;

        Ok(row.map(|row| User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            deleted_at: row.get("deleted_at"),
        }))
    }

    /// Returns a user by ID, if present.
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at, deleted_at
            FROM users
            WHERE id = $1
            "#,
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                error!("failed to find user by id {}: {}", id, e);
                DomainError::Internal(format!("database error: {}", e))
            })?;

        Ok(row.map(|row| User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            created_at: row.get("created_at"),
            deleted_at: row.get("deleted_at"),
        }))
    }
}
