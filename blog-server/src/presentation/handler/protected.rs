use crate::application::post_service::PostService;
use crate::data::post_repository::PostgresPostRepository;
use crate::domain::error::DomainError;
use crate::presentation::auth::AuthenticatedUser;
use crate::presentation::dto::{PostResponse};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, Scope, get, web, delete};
use tracing::info;
use uuid::Uuid;

pub fn scope() -> Scope {
    web::scope("")
        .service(list_posts)
        .service(get_post)
        .service(delete_post)
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
#[get("/posts/{id}")]
async fn get_post(
    req: HttpRequest,
    post: web::Data<PostService<PostgresPostRepository>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, DomainError> {
    let post = post.get_post(path.into_inner()).await?;
    let response = PostResponse::from(post);

    info!(
        request_id = %request_id(&req),
        post_id = %response.id,
        "post have gotten"
    );

    Ok(HttpResponse::Ok().json(response))
}
#[delete("/posts/{id}")]
async fn delete_post(
    req: HttpRequest,
    post: web::Data<PostService<PostgresPostRepository>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, DomainError> {
    let id = path.into_inner();
    post.delete_post(id).await?;

    info!(
        request_id = %request_id(&req),
        post_id = %id,
        "post deleted"
    );

    Ok(HttpResponse::Ok().json("{}"))
}

fn request_id(req: &HttpRequest) -> String {
    req.extensions()
        .get::<crate::presentation::middleware::RequestId>()
        .map(|rid| rid.0.clone())
        .unwrap_or_else(|| "unknown".into())
}
