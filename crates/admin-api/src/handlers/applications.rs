use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthUser;
use crate::handlers::users::ErrorResponse;

/// Helper to parse SQLite datetime string to chrono::DateTime<Utc>
fn parse_sqlite_datetime(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .and_then(|dt| dt.and_local_timezone(chrono::Utc).single())
        .unwrap_or_else(chrono::Utc::now)
}

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
    State(pool): State<SqlitePool>,
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
        VALUES (?, ?, ?, ?, ?)
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

    let id = app.id.ok_or_else(|| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: "Database returned null ID".to_string() }),
    )).and_then(|s| Uuid::parse_str(&s).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: "Invalid UUID format".to_string() }),
    )))?;
    
    let user_id = Uuid::parse_str(&app.user_id).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: "Invalid user ID format".to_string() }),
    ))?;

    Ok((
        StatusCode::CREATED,
        Json(ApplicationResponse {
            id,
            user_id,
            name: app.name,
            description: app.description,
            api_key: app.api_key.unwrap_or_default(),
            webhook_secret: app.webhook_secret.unwrap_or_default(),
            is_active: app.is_active != 0,
            created_at: parse_sqlite_datetime(&app.created_at),
            updated_at: parse_sqlite_datetime(&app.updated_at),
        }),
    ))
}

/// List all applications for the authenticated user
pub async fn list_applications(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
) -> Result<Json<ApplicationListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get applications
    let apps = sqlx::query!(
        r#"
        SELECT id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at
        FROM applications
        WHERE user_id = ?
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

    let applications: Vec<ApplicationResponse> = apps
        .into_iter()
        .filter_map(|app| {
            let id = app.id.and_then(|s| Uuid::parse_str(s.as_str()).ok())?;
            let user_id = Uuid::parse_str(app.user_id.as_str()).ok()?;
            Some(ApplicationResponse {
                id,
                user_id,
                name: app.name,
                description: app.description,
                api_key: app.api_key.unwrap_or_default(),
                webhook_secret: app.webhook_secret.unwrap_or_default(),
                is_active: app.is_active != 0,
                created_at: parse_sqlite_datetime(&app.created_at),
                updated_at: parse_sqlite_datetime(&app.updated_at),
            })
        })
        .collect();
    
    let total = applications.len() as i64;

    Ok(Json(ApplicationListResponse {
        applications,
        total,
    }))
}

/// Get a specific application
pub async fn get_application(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApplicationResponse>, (StatusCode, Json<ErrorResponse>)> {
    let app = sqlx::query!(
        r#"
        SELECT id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at
        FROM applications
        WHERE id = ? AND user_id = ?
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

    let id = app.id.ok_or_else(|| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: "Database returned null ID".to_string() }),
    )).and_then(|s| Uuid::parse_str(s.as_str()).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: "Invalid UUID format".to_string() }),
    )))?;
    
    let user_id = Uuid::parse_str(app.user_id.as_str()).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: "Invalid user ID format".to_string() }),
    ))?;

    Ok(Json(ApplicationResponse {
        id,
        user_id,
        name: app.name,
        description: app.description,
        api_key: app.api_key.unwrap_or_default(),
        webhook_secret: app.webhook_secret.unwrap_or_default(),
        is_active: app.is_active != 0,
        created_at: parse_sqlite_datetime(&app.created_at),
        updated_at: parse_sqlite_datetime(&app.updated_at),
    }))
}

/// Update an application
pub async fn update_application(
    State(pool): State<SqlitePool>,
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
        "SELECT id FROM applications WHERE id = ? AND user_id = ?",
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
    let mut query = String::from("UPDATE applications SET updated_at = datetime('now')");
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
            Option<String>,
            String,
            i64,
            String,
            String,
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
        api_key: app.4.unwrap_or_default(),
        webhook_secret: app.5,
        is_active: app.6 != 0,
        created_at: parse_sqlite_datetime(&app.7),
        updated_at: parse_sqlite_datetime(&app.8),
    }))
}

/// Delete an application
pub async fn delete_application(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query!(
        "DELETE FROM applications WHERE id = ? AND user_id = ?",
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
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApplicationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Generate new API key
    let new_api_key = generate_api_key();

    let app = sqlx::query!(
        r#"
        UPDATE applications
        SET api_key = ?, updated_at = datetime('now')
        WHERE id = ? AND user_id = ?
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

    let id = app.id.ok_or_else(|| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: "Database returned null ID".to_string() }),
    )).and_then(|s| Uuid::parse_str(s.as_str()).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: "Invalid UUID format".to_string() }),
    )))?;
    
    let user_id = Uuid::parse_str(app.user_id.as_str()).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: "Invalid user ID format".to_string() }),
    ))?;

    Ok(Json(ApplicationResponse {
        id,
        user_id,
        name: app.name,
        description: app.description,
        api_key: app.api_key.unwrap_or_default(),
        webhook_secret: app.webhook_secret.unwrap_or_default(),
        is_active: app.is_active != 0,
        created_at: parse_sqlite_datetime(&app.created_at),
        updated_at: parse_sqlite_datetime(&app.updated_at),
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
