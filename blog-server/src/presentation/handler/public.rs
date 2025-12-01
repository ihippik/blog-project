use crate::application::auth_service::AuthService;
use crate::data::user_repository::PostgresUserRepository;
use crate::domain::error::DomainError;
use crate::presentation::dto::{HealthResponse, LoginRequest, RegisterRequest, TokenResponse};
use actix_web::{HttpResponse, Responder, Scope, post, web};
use chrono::Utc;
use tracing::info;

pub fn scope() -> Scope {
    web::scope("")
        .route("/health", web::get().to(health))
        .service(register)
        .service(login)
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        timestamp: Utc::now(),
    })
}

#[post("/auth/register")]
async fn register(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<RegisterRequest>,
) -> Result<impl Responder, DomainError> {
    let user = service
        .register(
            payload.username.clone(),
            payload.email.clone(),
            payload.password.clone(),
        )
        .await?;

    info!(user_id = %user.id, email = %user.email, "user registered");

    Ok(HttpResponse::Created().json(serde_json::json!({
        "user_id": user.id,
        "username": user.username,
        "email": user.email
    })))
}

#[post("/auth/login")]
async fn login(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, DomainError> {
    let jwt = service.login(&payload.email, &payload.password).await?;
    info!(email = %payload.email, "user logged in");
    Ok(HttpResponse::Ok().json(TokenResponse { access_token: jwt }))
}
