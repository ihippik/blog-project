use actix_web::{HttpResponse, Responder, Scope, get, web};
use chrono::Utc;

use crate::presentation::dto::HealthResponse;
use crate::presentation::middleware::JwtAuthMiddleware;

pub fn scope() -> Scope {
    web::scope("").service(check)
}

#[get("/blogs/check")]
async fn check() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok!",
        timestamp: Utc::now(),
    })
}
