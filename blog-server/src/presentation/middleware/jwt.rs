use crate::application::auth_service::AuthService;
use crate::data::user_repository::PostgresUserRepository;
use crate::infrastructure::security::JwtKeys;
use crate::presentation::auth::extract_user_from_token;
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, Error, HttpMessage};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::cell::RefCell;
use std::rc::Rc;
use std::task::{Context, Poll};

/// JWT authentication middleware.
///
/// Validates the `Authorization` header, extracts the user from the JWT,
/// and attaches it to the request extensions.
pub struct JwtAuthMiddleware {
    keys: JwtKeys,
}

impl JwtAuthMiddleware {
    /// Creates a new JWT authentication middleware.
    pub fn new(keys: JwtKeys) -> Self {
        Self { keys }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates a new middleware service.
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthService {
            service: Rc::new(RefCell::new(service)),
            keys: self.keys.clone(),
        }))
    }
}

/// JWT authentication service.
///
/// Wraps an Actix service and performs JWT validation before request handling.
pub struct JwtAuthService<S> {
    service: Rc<RefCell<S>>,
    keys: JwtKeys,
}

impl<S, B> Service<ServiceRequest> for JwtAuthService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    /// Checks whether the underlying service is ready.
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(cx)
    }

    /// Processes an incoming request.
    ///
    /// Extracts and validates the JWT token, resolves the authenticated user,
    /// and stores it in the request extensions.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let keys = self.keys.clone();
        let service = Rc::clone(&self.service);

        let auth_service = req
            .app_data::<web::Data<AuthService<PostgresUserRepository>>>()
            .cloned();

        let auth_header = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        Box::pin(async move {
            let auth_service = auth_service
                .ok_or_else(|| actix_web::error::ErrorInternalServerError("AuthService missing"))?;

            let header = auth_header.ok_or_else(|| {
                actix_web::error::ErrorUnauthorized("missing authorization header")
            })?;

            let token = header.strip_prefix("Bearer ").ok_or_else(|| {
                actix_web::error::ErrorUnauthorized("invalid authorization header")
            })?;

            let user =
                extract_user_from_token(token, &keys, auth_service.get_ref()).await?;

            req.extensions_mut().insert(user);

            let fut = {
                let svc = service.borrow_mut();
                svc.call(req)
            };

            let res = fut.await?;
            Ok(res)
        })
    }
}
