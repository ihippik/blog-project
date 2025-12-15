use crate::application::post_service::PostService;
use crate::data::post_repository::PostgresPostRepository;
use crate::domain::error::DomainError;
use crate::presentation::auth::AuthenticatedUser;
use crate::presentation::dto::{CreatePostRequest, PostResponse};
use actix_web::{
    delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Scope,
};
use tracing::info;
use uuid::Uuid;

/// Returns the protected posts API scope.
pub fn scope() -> Scope {
    web::scope("")
        .service(list_posts)
        .service(get_post)
        .service(create_posts)
        .service(update_post)
        .service(delete_post)
}

/// Lists posts of the authenticated user.
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

/// Creates a new post.
#[post("/posts")]
async fn create_posts(
    req: HttpRequest,
    user: AuthenticatedUser,
    post: web::Data<PostService<PostgresPostRepository>>,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, DomainError> {
    let post = post
        .create_post(
            payload.title.clone(),
            payload.content.clone(),
            user.id,
        )
        .await?;

    let response = PostResponse::from(post);

    info!(
        request_id = %request_id(&req),
        author_id = %user.id,
        title = %response.title,
        "post created"
    );

    Ok(HttpResponse::Ok().json(response))
}

/// Returns a post by its ID.
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
        "post retrieved"
    );

    Ok(HttpResponse::Ok().json(response))
}

/// Updates an existing post.
#[put("/posts/{id}")]
async fn update_post(
    req: HttpRequest,
    post: web::Data<PostService<PostgresPostRepository>>,
    path: web::Path<Uuid>,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, DomainError> {
    let id = path.into_inner();
    let payload = payload.into_inner();

    let updated = post
        .update_post(id, payload.title, payload.content)
        .await?;

    let response = PostResponse::from(updated);

    info!(
        request_id = %request_id(&req),
        post_id = %response.id,
        "post updated"
    );

    Ok(HttpResponse::Ok().json(response))
}

/// Deletes a post by its ID.
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

/// Returns the request identifier if present.
fn request_id(req: &HttpRequest) -> String {
    req.extensions()
        .get::<crate::presentation::middleware::RequestId>()
        .map(|rid| rid.0.clone())
        .unwrap_or_else(|| "unknown".into())
}
