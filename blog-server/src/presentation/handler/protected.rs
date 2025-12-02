use actix_web::{HttpResponse, Responder, Scope, get, web, HttpRequest, HttpMessage};
use chrono::Utc;
use tracing::info;
use crate::application::post_service::PostService;
use crate::data::post_repository::PostgresPostRepository;
use crate::domain::error::DomainError;
use crate::presentation::auth::AuthenticatedUser;
use crate::presentation::dto::{HealthResponse, PostResponse};
use crate::presentation::middleware::JwtAuthMiddleware;

pub fn scope() -> Scope {
    web::scope("").service(list_posts)
}

#[get("/posts")]
async fn list_posts(
    req: HttpRequest,
    user: AuthenticatedUser,
    post: web::Data<PostService<PostgresPostRepository>>,
) -> Result<HttpResponse, DomainError> {
    let posts = post.list_posts(user.id).await?;
    let response: Vec<_> = posts.into_iter().map(PostResponse::from).collect();

    info!(
        request_id = %request_id(&req),
        author_id = %user.id,
        count = response.len(),
        "posts listed"
    );

    Ok(HttpResponse::Ok().json(response))
}

fn request_id(req: &HttpRequest) -> String {
    req.extensions()
        .get::<crate::presentation::middleware::RequestId>()
        .map(|rid| rid.0.clone())
        .unwrap_or_else(|| "unknown".into())
}