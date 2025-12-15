pub mod error;
pub mod http_client;
pub mod grpc_client;
pub mod models;

pub mod blog {
    tonic::include_proto!("blog");
}

use std::sync::Arc;
use error::BlogClientError;
use http_client::HttpClient;
use grpc_client::GrpcClient;

#[derive(Clone, Debug)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

#[derive(Clone)]
pub struct BlogClient {
    transport: Transport,
    http_client: Option<Arc<HttpClient>>,
    grpc_client: Option<Arc<GrpcClient>>,
    token: Option<String>,
}

impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let (http_client, grpc_client) = match &transport {
            Transport::Http(base_url) => {
                let http = HttpClient::new(base_url.clone())?;
                (Some(Arc::new(http)), None)
            }
            Transport::Grpc(addr) => {
                let grpc = GrpcClient::connect(addr.clone()).await?;
                (None, Some(Arc::new(grpc)))
            }
        };

        Ok(Self {
            transport,
            http_client,
            grpc_client,
            token: None,
        })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<models::AuthResponse, BlogClientError> {
        match (&self.transport, &self.http_client, &self.grpc_client) {
            (Transport::Http(_), Some(http), _) => {
                let resp = http.register(&username, &email, &password).await?;
                if let Some(token) = &resp.access_token {
                    self.set_token(token.clone());
                }
                Ok(resp)
            }
            (Transport::Grpc(_), _, Some(grpc)) => {
                let resp = grpc.register(&username, &email, &password).await?;
                if let Some(token) = &resp.access_token {
                    self.set_token(token.clone());
                }
                Ok(resp)
            }
            _ => Err(BlogClientError::InvalidState(
                "transport not properly initialized".into(),
            )),
        }
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<models::AuthResponse, BlogClientError> {
        match (&self.transport, &self.http_client, &self.grpc_client) {
            (Transport::Http(_), Some(http), _) => {
                let resp = http.login(&username, &password).await?;
                if let Some(token) = &resp.access_token {
                    self.set_token(token.clone());
                }
                Ok(resp)
            }
            (Transport::Grpc(_), _, Some(grpc)) => {
                let resp = grpc.login(&username, &password).await?;
                if let Some(token) = &resp.access_token {
                    self.set_token(token.clone());
                }
                Ok(resp)
            }
            _ => Err(BlogClientError::InvalidState(
                "transport not properly initialized".into(),
            )),
        }
    }

    pub async fn create_post(
        &self,
        title: String,
        content: String,
    ) -> Result<models::Post, BlogClientError> {
        let token = self
            .get_token()
            .ok_or(BlogClientError::Unauthorized("token is missing".into()))?;

        match (&self.transport, &self.http_client, &self.grpc_client) {
            (Transport::Http(_), Some(http), _) => {
                http.create_post(token, &title, &content).await
            }
            (Transport::Grpc(_), _, Some(grpc)) => {
                grpc.create_post(token, &title, &content).await
            }
            _ => Err(BlogClientError::InvalidState(
                "transport not properly initialized".into(),
            )),
        }
    }

    pub async fn get_post(
        &self,
        id: uuid::Uuid,
    ) -> Result<models::Post, BlogClientError> {
        let token = self
            .get_token()
            .ok_or(BlogClientError::Unauthorized("token is missing".into()))?;

        match (&self.transport, &self.http_client, &self.grpc_client) {
            (Transport::Http(_), Some(http), _) => http.get_post(token,id).await,
            (Transport::Grpc(_), _, Some(grpc)) => grpc.get_post(token, id).await,
            _ => Err(BlogClientError::InvalidState(
                "transport not properly initialized".into(),
            )),
        }
    }

    pub async fn update_post(
        &self,
        id: uuid::Uuid,
        title: String,
        content: String,
    ) -> Result<models::Post, BlogClientError> {
        let token = self
            .get_token()
            .ok_or(BlogClientError::Unauthorized("token is missing".into()))?;

        match (&self.transport, &self.http_client, &self.grpc_client) {
            (Transport::Http(_), Some(http), _) => {
                http.update_post(token, id, &title, &content).await
            }
            (Transport::Grpc(_), _, Some(grpc)) => {
                grpc.update_post(token, id, &title, &content).await
            }
            _ => Err(BlogClientError::InvalidState(
                "transport not properly initialized".into(),
            )),
        }
    }

    pub async fn delete_post(
        &self,
        id: uuid::Uuid,
    ) -> Result<(), BlogClientError> {
        let token = self
            .get_token()
            .ok_or(BlogClientError::Unauthorized("token is missing".into()))?;

        match (&self.transport, &self.http_client, &self.grpc_client) {
            (Transport::Http(_), Some(http), _) => http.delete_post(token, id).await,
            (Transport::Grpc(_), _, Some(grpc)) => grpc.delete_post(token, id).await,
            _ => Err(BlogClientError::InvalidState(
                "transport not properly initialized".into(),
            )),
        }
    }

    pub async fn list_posts(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<models::Post>, BlogClientError> {
        let token = self
            .get_token()
            .ok_or(BlogClientError::Unauthorized("token is missing".into()))?;

        match (&self.transport, &self.http_client, &self.grpc_client) {
            (Transport::Http(_), Some(http), _) => {
                http.list_posts(token, limit, offset).await
            }
            (Transport::Grpc(_), _, Some(grpc)) => {
                grpc.list_posts(token).await // TODO (makarov): add limit offset
            }
            _ => Err(BlogClientError::InvalidState(
                "transport not properly initialized".into(),
            )),
        }
    }
}
