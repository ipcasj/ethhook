use axum::{
    Json,
    extract::{FromRequestParts, Request},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Extractor for API key authentication
/// Reserved for future API key-based authentication
#[allow(dead_code)]
pub struct ApiKeyAuth {
    #[allow(dead_code)]
    pub application_id: Uuid,
    #[allow(dead_code)]
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for ApiKeyAuth
where
    S: Send + Sync,
{
    type Rejection = ApiKeyError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract API key from header
        let api_key = parts
            .headers
            .get("X-API-Key")
            .and_then(|h| h.to_str().ok())
            .ok_or(ApiKeyError::MissingApiKey)?;

        // Get database pool from extensions
        let pool = parts
            .extensions
            .get::<SqlitePool>()
            .ok_or(ApiKeyError::InternalError)?;

        // Validate API key and get application
        let app = sqlx::query!(
            r#"
            SELECT a.id as application_id, a.user_id, a.is_active
            FROM applications a
            WHERE a.api_key = ?
            "#,
            api_key
        )
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiKeyError::InternalError)?
        .ok_or(ApiKeyError::InvalidApiKey)?;

        if app.is_active == 0 {
            return Err(ApiKeyError::InactiveApplication);
        }

        let application_id = app.application_id
            .ok_or(ApiKeyError::InternalError)
            .and_then(|id| Uuid::parse_str(id.as_str()).map_err(|_| ApiKeyError::InternalError))?;
        
        let user_id = Uuid::parse_str(app.user_id.as_str())
            .map_err(|_| ApiKeyError::InternalError)?;

        Ok(ApiKeyAuth {
            application_id,
            user_id,
        })
    }
}

/// API key errors
/// Reserved for future API key-based authentication
#[allow(dead_code)]
#[derive(Debug)]
pub enum ApiKeyError {
    MissingApiKey,
    InvalidApiKey,
    InactiveApplication,
    InternalError,
}

/// API key error response
/// Reserved for future API key-based authentication
#[allow(dead_code)]
#[derive(Serialize)]
struct ApiKeyErrorResponse {
    error: String,
}

impl IntoResponse for ApiKeyError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiKeyError::MissingApiKey => (StatusCode::UNAUTHORIZED, "Missing API key"),
            ApiKeyError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "Invalid API key"),
            ApiKeyError::InactiveApplication => (StatusCode::FORBIDDEN, "Application is inactive"),
            ApiKeyError::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (
            status,
            Json(ApiKeyErrorResponse {
                error: message.to_string(),
            }),
        )
            .into_response()
    }
}

/// Middleware to inject database pool into request extensions
#[allow(dead_code)]
pub async fn inject_db_pool(
    pool: SqlitePool,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
{
    move |mut req: Request, next: Next| {
        let pool = pool.clone();
        Box::pin(async move {
            req.extensions_mut().insert(pool);
            next.run(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_error_responses() {
        let missing = ApiKeyError::MissingApiKey;
        let response = missing.into_response();
        // Response should be unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let inactive = ApiKeyError::InactiveApplication;
        let response = inactive.into_response();
        // Response should be forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
