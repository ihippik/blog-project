use crate::application::post_service::PostService;
use crate::data::post_repository::PostgresPostRepository;
use crate::domain::error::DomainError;
use crate::domain::post::Post;
use crate::presentation::auth::AuthenticatedUser;
use crate::presentation::dto::{CreatePostRequest, PostResponse};
use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, Responder, Scope, delete, get, post, put, web,
};
use tracing::info;
use uuid::Uuid;

pub fn scope() -> Scope {
    web::scope("")
        .service(list_posts)
        .service(get_post)
        .service(create_posts)
        .service(update_post)
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

#[post("/posts")]
async fn create_posts(
    req: HttpRequest,
    user: AuthenticatedUser,
    post: web::Data<PostService<PostgresPostRepository>>,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, DomainError> {
    let model = Post {
        id: Uuid::new_v4(),
        author_id: user.id,
        title: payload.title.clone(),
        content: payload.content.clone(),
        created_at: Default::default(),
        deleted_at: None,
    };

    let post = post.create_post(model).await?;
    let response = PostResponse::from(post);

    info!(
        request_id = %request_id(&req),
        author_id = %user.id,
        title = %response.title,
        "post created"
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

#[put("/posts/{id}")]
async fn update_post(
    req: HttpRequest,
    post: web::Data<PostService<PostgresPostRepository>>,
    path: web::Path<Uuid>,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, DomainError> {
    let id = path.into_inner();
    let payload = payload.into_inner();
    let mut model = post.get_post(id).await?;

    // обновляем поля из payload
    model.title = payload.title;
    model.content = payload.content;

    let updated = post.update_post(model).await?;

    let response = PostResponse::from(updated);

    info!(
        request_id = %request_id(&req),
        post_id = %response.id,
        "post updated"
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
