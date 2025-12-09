use chrono::Utc;
use crate::blog::{
    blog_service_client::BlogServiceClient,
    CreatePostRequest, GetPostRequest, UpdatePostRequest, ListPostRequest,
    ListPostsResponse as ProtoListPostsResponse, Post as ProtoPost,
    RegisterRequest, LoginRequest, LoginResponse, RegisterResponse
};
use crate::error::BlogClientError;
use crate::models::{AuthResponse, Post, User};
use tonic::transport::Channel;
use tonic::Request;
use uuid::Uuid;

#[derive(Clone)]
pub struct GrpcClient {
    inner: BlogServiceClient<Channel>,
}

impl GrpcClient {
    pub async fn connect(addr: String) -> Result<Self, BlogClientError> {
        let inner = BlogServiceClient::connect(addr).await?;
        Ok(Self { inner })
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let req = RegisterRequest{
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };
        let resp = self.inner.clone().register(Request::new(req)).await?;
        Ok(resp.into_inner().into())
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let req = LoginRequest{
            email: email.to_string(),
            password: password.to_string(),
        };
        let resp = self.inner.clone().login(Request::new(req)).await?;
        Ok(resp.into_inner().into())
    }

    pub async fn create_post(
        &self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let req = CreatePostRequest {
            title: title.to_string(),
            content: content.to_string(),
        };
        let resp = self.inner.clone().create_post(Request::new(req)).await?;

        Ok(Self::map_post(resp.into_inner().post.unwrap()))
    }

    pub async fn get_post(&self, id: Uuid) -> Result<Post, BlogClientError> {
        let req = GetPostRequest {
            id: id.to_string(),
        };

        let resp = self.inner.clone().get_post(Request::new(req)).await?;
        let post = resp.into_inner().post.unwrap();
        Ok(Self::map_post(post))
    }

    pub async fn update_post(
        &self,
        token: &str,
        id: Uuid,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let req = UpdatePostRequest {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
        };

        let resp = self.inner.clone().update_post(Request::new(req)).await?;
        Ok(Self::map_post(resp.into_inner().post.unwrap()))
    }

    pub async fn delete_post(
        &self,
        id: Uuid,
    ) -> Result<(), BlogClientError> {
        let req = GetPostRequest {
            id: id.to_string(),
        };

        let _ = self.inner.clone().delete_post(Request::new(req)).await?;
        Ok(())
    }

    pub async fn list_posts(
        &self,
    ) -> Result<Vec<Post>, BlogClientError> {
        let req = ListPostRequest {};

        let resp = self
            .inner
            .clone()
            .list_posts(Request::new(req))
            .await?;

        let ProtoListPostsResponse { posts } = resp.into_inner();

        Ok(posts.into_iter().map(Self::map_post).collect())
    }

    fn map_post(proto: ProtoPost) -> Post {
        Post {
            id: Uuid::parse_str(&proto.id).expect("invalid post id"),
            title: proto.title,
            content: proto.content,
            author_id: Uuid::parse_str(&proto.author_id).unwrap(),
            created_at: Utc::now(), // FIXME (makarov): add to the proto
            updated_at: None,
        }
    }
}

impl From<LoginResponse> for AuthResponse {
    fn from(proto: LoginResponse) -> Self {
        Self {
            token: if proto.token.is_empty() {
                None
            } else {
                Some(proto.token)
            },
            user: None,
        }
    }
}

impl From<RegisterResponse> for AuthResponse {
    fn from(proto: RegisterResponse) -> Self {
        let user = proto.user.expect("server must return user");

        Self {
            token: None,
            user: Some(
                User{
                    id: Uuid::parse_str(user.id.as_str()).expect("invalid user id"),
                    username: user.username,
                    email: user.email,
                }
            ),
        }
    }
}
