mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

use crate::application::auth_service::AuthService;
use crate::application::post_service::PostService;
use crate::data::post_repository::PostgresPostRepository;
use crate::data::user_repository::PostgresUserRepository;
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::database::{create_pool, run_migrations};
use crate::infrastructure::logging::init_logging;
use crate::infrastructure::security::JwtKeys;
use crate::presentation::handler;
use crate::presentation::middleware::{JwtAuthMiddleware, RequestIdMiddleware};
use actix_cors::Cors;
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{App, HttpServer, web};
use std::sync::Arc;
use crate::presentation::grpc_service::GrpcService;
use tonic::transport::Server;
use tracing::info;
use crate::presentation::blog::blog_service_server::BlogServiceServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env().expect("invalid configuration");

    init_logging(config.log_format.clone());

    let pool = create_pool(&config.database_url)
        .await
        .expect("failed to connect to database");

    run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let config_data = config.clone();

    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let post_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
    let auth_service = AuthService::new(
        Arc::clone(&user_repo),
        JwtKeys::new(config.jwt_secret.clone()),
    );
    let post_service = PostService::new(Arc::clone(&post_repo));

    let http_auth_service = auth_service.clone();
    let http_post_service = post_service.clone();

    // ---------- HTTP server ----------
    let http_server = HttpServer::new(move || {
        let cors = build_cors(&config_data);
        App::new()
            .wrap(Logger::default())
            .wrap(RequestIdMiddleware)
            .wrap(
                DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("Referrer-Policy", "no-referrer"))
                    .add(("Permissions-Policy", "geolocation=()"))
                    .add(("Cross-Origin-Opener-Policy", "same-origin")),
            )
            .wrap(cors)
            .app_data(web::Data::new(http_auth_service.clone()))
            .app_data(web::Data::new(http_post_service.clone()))
            .service(
                web::scope("/api")
                    .service(web::scope("/public").service(handler::public::scope()))
                    .service(
                        web::scope("/protected")
                            .wrap(JwtAuthMiddleware::new(http_auth_service.keys().clone()))
                            .service(handler::protected::scope()),
                    ),
            )
    })
    .bind((config.host.as_str(), config.http_port))?
    .run();

    // ---------- gRPC server ----------
    let grpc_addr = format!("{}:{}", config.host, config.grpc_port)
        .parse()
        .expect("invalid grpc addr");

    let grpc_service = GrpcService::new(post_service.clone(), auth_service.clone());

    let grpc_server = Server::builder()
        .add_service(BlogServiceServer::new(grpc_service))
        .serve(grpc_addr);

    info!(host=config.host ,port=config.grpc_port, "staring gRPC server");

    tokio::select!{
        http_res = http_server => {
            if let Err(e) = http_res {
                eprintln!("HTTP server error: {e}");
                return Err(e);
            }
        }
        grpc_res = grpc_server => {
            if let Err(e) = grpc_res {
                eprintln!("gRPC server error: {e}");
                return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
            }
        }
    }

    Ok(())
}

fn build_cors(config: &AppConfig) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::AUTHORIZATION,
        ])
        .supports_credentials()
        .max_age(3600);

    for origin in &config.cors_origins {
        cors = cors.allowed_origin(origin);
    }

    cors
}
