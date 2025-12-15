use tonic::{Request, Response, Status};
use tracing_log::log::info;
use uuid::Uuid;
use crate::application::auth_service::AuthService;
use crate::data::post_repository::{PostgresPostRepository};
use crate::application::post_service::PostService;
use crate::data::user_repository::PostgresUserRepository;
use crate::domain::error::DomainError;
use crate::presentation::blog::{EmptyResponse, GetPostRequest, ListPostRequest, ListPostsResponse, LoginRequest, LoginResponse, Post, PostResponse, RegisterRequest, RegisterResponse, UpdatePostRequest};

pub struct GrpcService {
    post: PostService<PostgresPostRepository>,
    auth: AuthService<PostgresUserRepository>,
}

impl GrpcService {
    pub fn new(post: PostService<PostgresPostRepository>,auth: AuthService<PostgresUserRepository>) -> Self {
        Self { post,auth }
    }
}

#[tonic::async_trait]
impl BlogService for GrpcService {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        let user = self.auth.register(req.username,req.email,req.password).await.map_err(to_status)?;

        Ok(Response::new(RegisterResponse{
            user: Some(user.into()),
        }))
    }

    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        let token = self.auth.login(req.email.as_ref(),req.password.as_ref())
            .await.map_err(to_status)?;

        Ok(Response::new(LoginResponse{
            token: token.into(),
        }))
    }

    async fn get_post(&self, request: Request<GetPostRequest>) -> Result<Response<PostResponse>, Status> {
        let token = extract_token(&request)?;
        self.auth.keys()
            .verify_token(&token)
            .map_err(|_| Status::unauthenticated("invalid token"))?;

        let req = request.into_inner();
        let id =Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("invalid id"))?;
        let post = self.post.get_post(id).await.map_err(to_status)?;

        Ok(Response::new(PostResponse{
            post: Some(post.into()),
        }))
    }

    async fn list_posts(&self, request: Request<ListPostRequest>) -> Result<Response<ListPostsResponse>, Status> {
        let token = extract_token(&request)?;
        let claims = self.auth.keys()
            .verify_token(&token)
            .map_err(|_| Status::unauthenticated("invalid token claims"))?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| Status::unauthenticated("invalid token"))?;
        let posts = self.post.list_posts(user_id).await.map_err(to_status)?;
        let response: Vec<Post> = posts
            .into_iter()
            .map(Into::into)
            .collect();

        info!("grpc got {} posts", response.len());

        Ok(Response::new(ListPostsResponse{
            posts: response,
        }))
    }

    async fn update_post(&self, request: Request<UpdatePostRequest>) -> Result<Response<PostResponse>, Status> {
        let token = extract_token(&request)?;
        self.auth.keys()
            .verify_token(&token)
            .map_err(|_| Status::unauthenticated("invalid token"))?;

        let req = request.into_inner();
        let id =Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("invalid id"))?;
        let post= self.post.update_post(id,req.title, req.content).await.map_err(to_status)?;

        Ok(Response::new(PostResponse{
            post: Some(post.into()),
        }))
    }

    async fn delete_post(&self, request: Request<GetPostRequest>) -> Result<Response<EmptyResponse>, Status> {
        let token = extract_token(&request)?;
        self.auth.keys()
            .verify_token(&token)
            .map_err(|_| Status::unauthenticated("invalid token"))?;

        let req = request.into_inner();
        let id =Uuid::parse_str(&req.id).map_err(|_| Status::invalid_argument("invalid id"))?;

        self.post.delete_post(id).await.map_err(to_status)?;

        Ok(Response::new(EmptyResponse{}))
    }

    async fn create_post(&self, request: Request<Post>) -> Result<Response<PostResponse>, Status> {
        let token = extract_token(&request)?;
        self.auth.keys()
            .verify_token(&token)
            .map_err(|_| Status::unauthenticated("invalid token"))?;

        let req = request.into_inner();
        let post = self.post.create_post(req.title,req.content,Uuid::parse_str(&req.author_id)
            .map_err(|_| Status::invalid_argument("invalid author id"))?)
            .await.map_err(to_status)?;

        Ok(Response::new(PostResponse{
            post: Some(post.into()),
        }))
    }
}

fn to_status(err: DomainError) -> Status {
    match err {
        DomainError::Validation(msg) =>
            Status::invalid_argument(msg),

        DomainError::UserNotFound(id) =>
            Status::not_found(format!("user not found: {id}")),

        DomainError::PostNotFound(id) =>
            Status::not_found(format!("post not found: {id}")),

        DomainError::InvalidCredentials(msg) =>
            Status::unauthenticated(msg),

        DomainError::Internal(msg) =>
            Status::internal(msg),
    }
}

use crate::presentation::blog::Post as ProtoPost;
use crate::domain::post::Post as DomainPost;

impl From<DomainPost> for ProtoPost {
    fn from(p: DomainPost) -> Self {
        Self {
            id: p.id.to_string(),
            title: p.title,
            content: p.content,
            author_id: p.author_id.to_string(),
        }
    }
}

use crate::presentation::blog::User as ProtoUser;
use crate::domain::user::User as DomainUser;
use crate::presentation::blog::blog_service_server::BlogService;

impl From<DomainUser> for ProtoUser {
    fn from(p: DomainUser) -> Self {
        Self {
            id: p.id.to_string(),
            username: p.username,
            email: p.email,
        }
    }
}

fn extract_token<T>(request: &Request<T>) -> Result<String, Status> {
    let value = request.metadata()
        .get("authorization")
        .ok_or_else(|| Status::unauthenticated("authorization header missing"))?;

    let auth_str = value
        .to_str()
        .map_err(|_| Status::unauthenticated("invalid authorization header"))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(Status::unauthenticated("invalid authorization scheme"));
    }

    let token = auth_str.trim_start_matches("Bearer ").to_string();
    Ok(token)
}
