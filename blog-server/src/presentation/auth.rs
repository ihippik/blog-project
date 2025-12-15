use actix_web::dev::Payload;
use actix_web::{error::ErrorUnauthorized, Error, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{ready, Ready};
use uuid::Uuid;

use crate::application::auth_service::AuthService;
use crate::data::user_repository::PostgresUserRepository;
use crate::infrastructure::security::JwtKeys;

/// Authenticated user extracted from the request context.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// Authenticated user ID.
    pub id: Uuid,

    /// Authenticated user email.
    #[allow(dead_code)]
    pub email: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    /// Extracts the authenticated user from request extensions.
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<AuthenticatedUser>() {
            Some(user) => ready(Ok(user.clone())),
            None => ready(Err(ErrorUnauthorized("missing authenticated user"))),
        }
    }
}

/// Extracts an authenticated user from a JWT token.
///
/// Verifies the token, resolves the user from storage,
/// and returns the authenticated user context.
pub async fn extract_user_from_token(
    token: &str,
    keys: &JwtKeys,
    auth_service: &AuthService<PostgresUserRepository>,
) -> Result<AuthenticatedUser, Error> {
    let claims = keys
        .verify_token(token)
        .map_err(|_| ErrorUnauthorized("invalid token"))?;

    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| ErrorUnauthorized("invalid token"))?;

    let user = auth_service
        .get_user(user_id)
        .await
        .map_err(|_| ErrorUnauthorized("user not found"))?;

    Ok(AuthenticatedUser {
        id: user.id,
        email: user.email,
    })
}
