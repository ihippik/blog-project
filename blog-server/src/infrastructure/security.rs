use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT signing and verification keys.
#[derive(Clone)]
pub struct JwtKeys {
    secret: String,
}

impl JwtKeys {
    /// Creates a new JWT key set.
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    /// Generates a signed JWT for the given user ID.
    pub fn generate_token(
        &self,
        user_id: Uuid,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims {
            sub: user_id.to_string(),
            exp: chrono::Utc::now()
                .checked_add_signed(chrono::Duration::hours(1))
                .unwrap()
                .timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    /// Verifies a JWT and returns its claims.
    pub fn verify_token(
        &self,
        token: &str,
    ) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(data.claims)
    }
}

/// JWT claims payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: String,

    /// Expiration timestamp (seconds since epoch).
    pub exp: usize,

    /// Issued-at timestamp (seconds since epoch).
    pub iat: usize,
}

/// Hashes a plaintext password using Argon2.
pub fn hash_password(
    password: &str,
) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(hash)
}

/// Verifies a plaintext password against a hash.
pub fn verify_password(
    password: &str,
    hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
