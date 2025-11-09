use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthUser;
use crate::handlers::users::ErrorResponse;

/// Request to create a new application
#[derive(Debug, Deserialize, Validate)]
pub struct CreateApplicationRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,

    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
}

/// Request to update an application
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateApplicationRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,

    pub is_active: Option<bool>,
}

/// Application response
#[derive(Debug, Serialize)]
pub struct ApplicationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub api_key: String,
    pub webhook_secret: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// List of applications
#[derive(Debug, Serialize)]
pub struct ApplicationListResponse {
    pub applications: Vec<ApplicationResponse>,
    pub total: i64,
}

/// Create a new application
pub async fn create_application(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Json(payload): Json<CreateApplicationRequest>,
) -> Result<(StatusCode, Json<ApplicationResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {e}"),
            }),
        )
    })?;

    // Generate API key and webhook secret
    let api_key = generate_api_key();
    let webhook_secret = generate_hmac_secret();

    // Create application
    let app = sqlx::query!(
        r#"
        INSERT INTO applications (user_id, name, description, api_key, webhook_secret)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at
        "#,
        auth_user.user_id,
        payload.name,
        payload.description,
        api_key,
        webhook_secret
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create application: {e}"),
            }),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(ApplicationResponse {
            id: app.id,
            user_id: app.user_id,
            name: app.name,
            description: app.description,
            api_key: app.api_key.unwrap_or_default(),
            webhook_secret: app.webhook_secret,
            is_active: app.is_active.unwrap_or(true),
            created_at: app.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: app.updated_at.unwrap_or_else(chrono::Utc::now),
        }),
    ))
}

/// List all applications for the authenticated user
pub async fn list_applications(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
) -> Result<Json<ApplicationListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get applications
    let apps = sqlx::query!(
        r#"
        SELECT id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at
        FROM applications
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        auth_user.user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch applications: {e}"),
            }),
        )
    })?;

    let total = apps.len() as i64;
    let applications = apps
        .into_iter()
        .map(|app| ApplicationResponse {
            id: app.id,
            user_id: app.user_id,
            name: app.name,
            description: app.description,
            api_key: app.api_key.unwrap_or_default(),
            webhook_secret: app.webhook_secret,
            is_active: app.is_active.unwrap_or(true),
            created_at: app.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: app.updated_at.unwrap_or_else(chrono::Utc::now),
        })
        .collect();

    Ok(Json(ApplicationListResponse {
        applications,
        total,
    }))
}

/// Get a specific application
pub async fn get_application(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApplicationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let app = sqlx::query!(
        r#"
        SELECT id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at
        FROM applications
        WHERE id = $1 AND user_id = $2
        "#,
        app_id,
        auth_user.user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {e}"),
            }),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Application not found".to_string(),
            }),
        )
    })?;

    Ok(Json(ApplicationResponse {
        id: app.id,
        user_id: app.user_id,
        name: app.name,
        description: app.description,
        api_key: app.api_key.unwrap_or_default(),
        webhook_secret: app.webhook_secret,
        is_active: app.is_active.unwrap_or(true),
        created_at: app.created_at.unwrap_or_else(chrono::Utc::now),
        updated_at: app.updated_at.unwrap_or_else(chrono::Utc::now),
    }))
}

/// Update an application
pub async fn update_application(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
    Json(payload): Json<UpdateApplicationRequest>,
) -> Result<Json<ApplicationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {e}"),
            }),
        )
    })?;

    // Check if application exists and belongs to user
    let _existing = sqlx::query!(
        "SELECT id FROM applications WHERE id = $1 AND user_id = $2",
        app_id,
        auth_user.user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {e}"),
            }),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Application not found".to_string(),
            }),
        )
    })?;

    // Build update query dynamically
    let mut query = String::from("UPDATE applications SET updated_at = NOW()");
    let mut params: Vec<String> = vec![];

    if let Some(name) = &payload.name {
        params.push(format!("name = '{}'", name.replace('\'', "''")));
    }
    if let Some(description) = &payload.description {
        params.push(format!(
            "description = '{}'",
            description.replace('\'', "''")
        ));
    }
    if let Some(is_active) = payload.is_active {
        params.push(format!("is_active = {is_active}"));
    }

    if !params.is_empty() {
        query.push_str(", ");
        query.push_str(&params.join(", "));
    }

    query.push_str(&format!(
        " WHERE id = '{app_id}' RETURNING id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at"
    ));

    let app = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            Option<String>,
            String,
            String,
            bool,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(&query)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to update application: {e}"),
            }),
        )
    })?;

    Ok(Json(ApplicationResponse {
        id: app.0,
        user_id: app.1,
        name: app.2,
        description: app.3,
        api_key: app.4,
        webhook_secret: app.5,
        is_active: app.6,
        created_at: app.7,
        updated_at: app.8,
    }))
}

/// Delete an application
pub async fn delete_application(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query!(
        "DELETE FROM applications WHERE id = $1 AND user_id = $2",
        app_id,
        auth_user.user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to delete application: {e}"),
            }),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Application not found".to_string(),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Regenerate API key for an application
pub async fn regenerate_api_key(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApplicationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Generate new API key
    let new_api_key = generate_api_key();

    let app = sqlx::query!(
        r#"
        UPDATE applications
        SET api_key = $1, updated_at = NOW()
        WHERE id = $2 AND user_id = $3
        RETURNING id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at
        "#,
        new_api_key,
        app_id,
        auth_user.user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to regenerate API key: {e}"),
            }),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Application not found".to_string(),
            }),
        )
    })?;

    Ok(Json(ApplicationResponse {
        id: app.id,
        user_id: app.user_id,
        name: app.name,
        description: app.description,
        api_key: app.api_key.unwrap_or_default(),
        webhook_secret: app.webhook_secret,
        is_active: app.is_active.unwrap_or(true),
        created_at: app.created_at.unwrap_or_else(chrono::Utc::now),
        updated_at: app.updated_at.unwrap_or_else(chrono::Utc::now),
    }))
}

/// Generate a secure API key
fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const KEY_LEN: usize = 32;

    let mut rng = rand::thread_rng();
    let key: String = (0..KEY_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("ethk_{key}")
}

/// Generate a secure HMAC secret (64 characters)
fn generate_hmac_secret() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const SECRET_LEN: usize = 64;

    let mut rng = rand::thread_rng();
    (0..SECRET_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_application_validation() {
        // Valid request
        let valid = CreateApplicationRequest {
            name: "My App".to_string(),
            description: Some("Test application".to_string()),
        };
        assert!(valid.validate().is_ok());

        // Empty name
        let empty_name = CreateApplicationRequest {
            name: "".to_string(),
            description: None,
        };
        assert!(empty_name.validate().is_err());
    }

    #[test]
    fn test_api_key_generation() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();

        // Keys should be different
        assert_ne!(key1, key2);

        // Keys should start with prefix
        assert!(key1.starts_with("ethk_"));
        assert!(key2.starts_with("ethk_"));

        // Keys should have correct length (prefix + 32 chars)
        assert_eq!(key1.len(), 37); // "ethk_" + 32
        assert_eq!(key2.len(), 37);
    }
}
