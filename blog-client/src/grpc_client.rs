use chrono::Utc;
use crate::blog::{
    blog_service_client::BlogServiceClient,
    CreatePostRequest, GetPostRequest, UpdatePostRequest, ListPostRequest,
    ListPostsResponse as ProtoListPostsResponse, Post as ProtoPost,
    RegisterRequest, LoginRequest, LoginResponse, RegisterResponse,
};
use crate::error::BlogClientError;
use crate::models::{AuthResponse, Post, User};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::Request;
use uuid::Uuid;

/// gRPC client implementation for the Blog service.
#[derive(Clone)]
pub struct GrpcClient {
    inner: BlogServiceClient<Channel>,
}

impl GrpcClient {
    /// Connects to a gRPC server and creates a new client.
    pub async fn connect(addr: String) -> Result<Self, BlogClientError> {
        let inner = BlogServiceClient::connect(addr).await?;
        Ok(Self { inner })
    }

    /// Registers a new user.
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let req = RegisterRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };

        let resp = self.inner.clone().register(Request::new(req)).await?;
        Ok(resp.into_inner().into())
    }

    /// Authenticates a user.
    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, BlogClientError> {
        let req = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let resp = self.inner.clone().login(Request::new(req)).await?;
        Ok(resp.into_inner().into())
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
        let payload = CreatePostRequest {
            title: title.to_string(),
            content: content.to_string(),
        };

        let req = with_auth(Request::new(payload), token)?;
        let resp = self.inner.clone().create_post(req).await?;

        Ok(Self::map_post(resp.into_inner().post.unwrap()))
    }

    /// Returns a post by its ID.
    ///
    /// Requires a valid JWT token.
    pub async fn get_post(&self, token: &str, id: Uuid) -> Result<Post, BlogClientError> {
        let payload = GetPostRequest { id: id.to_string() };

        let req = with_auth(Request::new(payload), token)?;
        let resp = self.inner.clone().get_post(req).await?;
        let post = resp.into_inner().post.unwrap();

        Ok(Self::map_post(post))
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
        let payload = UpdatePostRequest {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
        };

        let req = with_auth(Request::new(payload), token)?;
        let resp = self.inner.clone().update_post(req).await?;

        Ok(Self::map_post(resp.into_inner().post.unwrap()))
    }

    /// Deletes a post by its ID.
    ///
    /// Requires a valid JWT token.
    pub async fn delete_post(&self, token: &str, id: Uuid) -> Result<(), BlogClientError> {
        let payload = GetPostRequest { id: id.to_string() };

        let req = with_auth(Request::new(payload), token)?;
        let _ = self.inner.clone().delete_post(req).await?;

        Ok(())
    }

    /// Lists posts of the authenticated user.
    ///
    /// Requires a valid JWT token.
    pub async fn list_posts(&self, token: &str) -> Result<Vec<Post>, BlogClientError> {
        let payload = ListPostRequest {};

        let req = with_auth(Request::new(payload), token)?;
        let resp = self.inner.clone().list_posts(req).await?;

        let ProtoListPostsResponse { posts } = resp.into_inner();
        Ok(posts.into_iter().map(Self::map_post).collect())
    }

    /// Maps a protobuf post into a client post model.
    fn map_post(proto: ProtoPost) -> Post {
        Post {
            id: Uuid::parse_str(&proto.id).expect("invalid post id"),
            title: proto.title,
            content: proto.content,
            author_id: Uuid::parse_str(&proto.author_id).unwrap(),
            created_at: Utc::now(), // FIXME: add created_at to proto
            updated_at: None,
        }
    }
}

/// Attaches the `authorization` metadata header to a gRPC request.
fn with_auth<T>(mut req: Request<T>, token: &str) -> Result<Request<T>, tonic::Status> {
    let value = format!("Bearer {}", token);

    let meta = MetadataValue::try_from(value)
        .map_err(|_| tonic::Status::invalid_argument("invalid authorization metadata"))?;

    req.metadata_mut().insert("authorization", meta);
    Ok(req)
}

/// Converts a protobuf login response into a client auth response.
impl From<LoginResponse> for AuthResponse {
    fn from(proto: LoginResponse) -> Self {
        Self {
            access_token: if proto.token.is_empty() {
                None
            } else {
                Some(proto.token)
            },
            user: None,
        }
    }
}

/// Converts a protobuf registration response into a client auth response.
impl From<RegisterResponse> for AuthResponse {
    fn from(proto: RegisterResponse) -> Self {
        let user = proto.user.expect("server must return user");

        Self {
            access_token: None,
            user: Some(User {
                id: Uuid::parse_str(user.id.as_str()).expect("invalid user id"),
                username: user.username,
                email: user.email,
            }),
        }
    }
}
