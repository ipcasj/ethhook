use anyhow::{Context, Result};
use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{header, request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,          // user_id
    pub email: String,
    pub exp: i64,           // expiration timestamp
    pub iat: i64,           // issued at timestamp
}

impl Claims {
    /// Create new claims for a user
    pub fn new(user_id: Uuid, email: String, expiration_hours: i64) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::hours(expiration_hours)).timestamp();

        Claims {
            sub: user_id,
            email,
            exp,
            iat: now.timestamp(),
        }
    }
}

/// Generate a JWT token
pub fn generate_token(user_id: Uuid, email: String, secret: &str, expiration_hours: i64) -> Result<String> {
    let claims = Claims::new(user_id, email, expiration_hours);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("Failed to encode JWT")?;

    Ok(token)
}

/// Validate a JWT token and extract claims
pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .context("Failed to decode JWT")?;

    Ok(token_data.claims)
}

/// Hash a password using bcrypt
pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .context("Failed to hash password")
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(password, hash)
        .context("Failed to verify password")
}

/// Extractor for authenticated user
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        // Check Bearer scheme
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        // Get JWT secret from extensions
        let jwt_secret = parts
            .extensions
            .get::<String>()
            .ok_or(AuthError::InternalError)?;

        // Validate token
        let claims = validate_token(token, jwt_secret)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(AuthUser {
            user_id: claims.sub,
            email: claims.email,
        })
    }
}

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    InternalError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            AuthError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        (status, Json(serde_json::json!({
            "error": message
        }))).into_response()
    }
}

/// Middleware to inject JWT secret into request extensions
pub async fn inject_jwt_secret(
    mut req: Request,
    next: Next,
) -> Response {
    // Get JWT secret from environment variable
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret-key-for-testing-only".to_string());
    req.extensions_mut().insert(secret);
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "my-secure-password";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        let secret = "test-secret-key";
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();

        let token = generate_token(user_id, email.clone(), secret, 24).unwrap();
        let claims = validate_token(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_jwt_invalid_secret() {
        let secret = "test-secret-key";
        let user_id = Uuid::new_v4();
        let email = "test@example.com".to_string();

        let token = generate_token(user_id, email, secret, 24).unwrap();
        let result = validate_token(&token, "wrong-secret");

        assert!(result.is_err());
    }
}
