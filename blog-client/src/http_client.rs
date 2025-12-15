use crate::error::BlogClientError;
use crate::models::{AuthResponse, Post};
use reqwest::Client;
use uuid::Uuid;

/// HTTP transport implementation for the blog client.
#[derive(Clone)]
pub struct HttpClient {
    base_url: String,
    client: Client,
}

impl HttpClient {
    /// Creates a new HTTP client with the given base URL.
    pub fn new(base_url: String) -> Result<Self, BlogClientError> {
        Ok(Self {
            base_url,
            client: Client::new(),
        })
    }

    /// Builds a full URL from a relative path.
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Registers a new user.
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let body = serde_json::json!({
            "username": username,
            "email": email,
            "password": password,
        });

        let resp = self
            .client
            .post(self.url("/api/public/auth/register"))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(resp.json().await?)
    }

    /// Authenticates a user and returns an auth response.
    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, BlogClientError> {
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        let resp = self
            .client
            .post(self.url("/api/public/auth/login"))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(resp.json().await?)
    }

    /// Creates a new post.
    ///
    /// Requires a valid JWT token.
    pub async fn create_post(
        &self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let body = serde_json::json!({
            "title": title,
            "content": content,
        });

        let resp = self
            .client
            .post(self.url("/api/protected/posts"))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(resp.json().await?)
    }

    /// Returns a post by its ID.
    ///
    /// Requires a valid JWT token.
    pub async fn get_post(&self, token: &str, id: Uuid) -> Result<Post, BlogClientError> {
        let resp = self
            .client
            .get(self.url(&format!("/api/protected/posts/{id}")))
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?;

        Ok(resp.json().await?)
    }

    /// Updates an existing post.
    ///
    /// Requires a valid JWT token.
    pub async fn update_post(
        &self,
        token: &str,
        id: Uuid,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let body = serde_json::json!({
            "title": title,
            "content": content,
        });

        let resp = self
            .client
            .put(self.url(&format!("/api/protected/posts/{id}")))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(resp.json().await?)
    }

    /// Deletes a post by its ID.
    ///
    /// Requires a valid JWT token.
    pub async fn delete_post(&self, token: &str, id: Uuid) -> Result<(), BlogClientError> {
        self.client
            .delete(self.url(&format!("/api/protected/posts/{id}")))
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Lists posts of the authenticated user.
    ///
    /// Requires a valid JWT token.
    pub async fn list_posts(
        &self,
        token: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Post>, BlogClientError> {
        let resp = self
            .client
            .get(self.url("/api/protected/posts"))
            .bearer_auth(token)
            .query(&[("limit", limit), ("offset", offset)])
            .send()
            .await?
            .error_for_status()?;

        Ok(resp.json().await?)
    }
}
