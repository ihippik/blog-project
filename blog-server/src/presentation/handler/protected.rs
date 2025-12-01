use actix_web::{HttpResponse, Responder, Scope, post, web};
use chrono::Utc;

use crate::presentation::dto::HealthResponse;

pub fn scope() -> Scope {
    web::scope("").route("/health", web::get().to(health))
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok!",
        timestamp: Utc::now(),
    })
}
