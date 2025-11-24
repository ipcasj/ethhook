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

/// Helper to serialize Vec to JSON string for SQLite
fn serialize_json<T: serde::Serialize>(data: &[T]) -> String {
    serde_json::to_string(data).unwrap_or_else(|_| "[]".to_string())
}

/// Helper to deserialize JSON string from SQLite to Vec
fn deserialize_json<T: serde::de::DeserializeOwned>(s: Option<&str>) -> Vec<T> {
    s.and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

/// Helper to parse SQLite datetime string
fn parse_sqlite_datetime(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .and_then(|dt| dt.and_local_timezone(chrono::Utc).single())
        .unwrap_or_else(chrono::Utc::now)
}

/// Request to create a webhook endpoint
#[derive(Debug, Deserialize, Validate)]
pub struct CreateEndpointRequest {
    pub application_id: Uuid,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Name must be between 1 and 100 characters"
    ))]
    pub name: String,

    #[validate(url(message = "Invalid webhook URL"))]
    pub webhook_url: String,

    #[validate(length(max = 200, message = "Description must be at most 200 characters"))]
    pub description: Option<String>,

    /// Chain IDs to filter events (e.g., [1, 137] for Ethereum and Polygon)
    pub chain_ids: Vec<i32>,

    /// Contract addresses to filter (optional, empty = all contracts)
    pub contract_addresses: Vec<String>,

    /// Event signatures to filter (optional, empty = all events)
    pub event_signatures: Vec<String>,
}

/// Request to update an endpoint
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEndpointRequest {
    #[validate(url(message = "Invalid webhook URL"))]
    pub webhook_url: Option<String>,

    #[validate(length(max = 200, message = "Description must be at most 200 characters"))]
    pub description: Option<String>,

    pub chain_ids: Option<Vec<i32>>,
    pub contract_addresses: Option<Vec<String>>,
    pub event_signatures: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

/// Endpoint response
#[derive(Debug, Serialize)]
pub struct EndpointResponse {
    pub id: Uuid,
    pub application_id: Uuid,
    pub name: String,
    pub webhook_url: String,
    pub description: Option<String>,
    pub hmac_secret: String,
    pub chain_ids: Vec<i32>,
    pub contract_addresses: Vec<String>,
    pub event_signatures: Vec<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// List of endpoints
#[derive(Debug, Serialize)]
pub struct EndpointListResponse {
    pub endpoints: Vec<EndpointResponse>,
    pub total: i64,
}

/// Create a new webhook endpoint
pub async fn create_endpoint(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Json(payload): Json<CreateEndpointRequest>,
) -> Result<(StatusCode, Json<EndpointResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {e}"),
            }),
        )
    })?;

    // Convert UUIDs to strings for SQLite
    let user_id_str = auth_user.user_id.to_string();
    let app_id_str = payload.application_id.to_string();

    // Verify application belongs to user
    let _app = sqlx::query!(
        "SELECT id FROM applications WHERE id = ? AND user_id = ?",
        app_id_str,
        user_id_str
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

    // Generate HMAC secret
    let hmac_secret = generate_hmac_secret();

    // Serialize JSON arrays to strings for SQLite storage
    let chain_ids_json = serialize_json(&payload.chain_ids);
    let contract_addresses_json = serialize_json(&payload.contract_addresses);
    let event_signatures_json = serialize_json(&payload.event_signatures);

    // Create endpoint
    let endpoint_id = Uuid::new_v4().to_string();
    let endpoint = sqlx::query!(
        r#"
        INSERT INTO endpoints (
            id, application_id, name, webhook_url, description, hmac_secret,
            chain_ids, contract_addresses, event_signatures
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id, application_id, name, webhook_url, description, hmac_secret,
                  chain_ids, contract_addresses, event_signatures,
                  is_active, created_at, updated_at
        "#,
        endpoint_id,
        app_id_str,
        payload.name,
        payload.webhook_url,
        payload.description,
        hmac_secret,
        chain_ids_json,
        contract_addresses_json,
        event_signatures_json
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create endpoint: {e}"),
            }),
        )
    })?;

    let id = endpoint
        .id
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database returned null ID".to_string(),
                }),
            )
        })
        .and_then(|s| {
            Uuid::parse_str(s.as_str()).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Invalid UUID format".to_string(),
                    }),
                )
            })
        })?;

    let application_id = Uuid::parse_str(endpoint.application_id.as_str()).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Invalid application_id UUID format".to_string(),
            }),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(EndpointResponse {
            id,
            application_id,
            name: endpoint.name,
            webhook_url: endpoint.webhook_url,
            description: endpoint.description,
            hmac_secret: endpoint.hmac_secret,
            chain_ids: deserialize_json(endpoint.chain_ids.as_deref()),
            contract_addresses: deserialize_json(endpoint.contract_addresses.as_deref()),
            event_signatures: deserialize_json(endpoint.event_signatures.as_deref()),
            is_active: endpoint.is_active != 0,
            created_at: parse_sqlite_datetime(&endpoint.created_at),
            updated_at: parse_sqlite_datetime(&endpoint.updated_at),
        }),
    ))
}

/// List endpoints for an application
pub async fn list_endpoints(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<Json<EndpointListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Convert UUIDs to strings for SQLite
    let user_id_str = auth_user.user_id.to_string();
    let app_id_str = app_id.to_string();

    // Verify application belongs to user
    let _app = sqlx::query!(
        "SELECT id FROM applications WHERE id = ? AND user_id = ?",
        app_id_str,
        user_id_str
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

    // Get endpoints
    let endpoints = sqlx::query!(
        r#"
        SELECT id, application_id, name, webhook_url, description, hmac_secret,
               chain_ids, contract_addresses, event_signatures,
               is_active, created_at, updated_at
        FROM endpoints
        WHERE application_id = ?
        ORDER BY created_at DESC
        "#,
        app_id_str
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch endpoints: {e}"),
            }),
        )
    })?;

    let endpoints: Vec<EndpointResponse> = endpoints
        .into_iter()
        .filter_map(|ep| {
            let id = ep.id.and_then(|s| Uuid::parse_str(s.as_str()).ok())?;
            let application_id = Uuid::parse_str(ep.application_id.as_str()).ok()?;
            Some(EndpointResponse {
                id,
                application_id,
                name: ep.name,
                webhook_url: ep.webhook_url,
                description: ep.description,
                hmac_secret: ep.hmac_secret,
                chain_ids: deserialize_json(ep.chain_ids.as_deref()),
                contract_addresses: deserialize_json(ep.contract_addresses.as_deref()),
                event_signatures: deserialize_json(ep.event_signatures.as_deref()),
                is_active: ep.is_active != 0,
                created_at: parse_sqlite_datetime(&ep.created_at),
                updated_at: parse_sqlite_datetime(&ep.updated_at),
            })
        })
        .collect();

    let total = endpoints.len() as i64;

    Ok(Json(EndpointListResponse { endpoints, total }))
}

/// List all endpoints for the authenticated user across all their applications
pub async fn list_all_user_endpoints(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
) -> Result<Json<EndpointListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Convert UUID to string for SQLite
    let user_id_str = auth_user.user_id.to_string();

    // Get all endpoints for user's applications
    let endpoints = sqlx::query!(
        r#"
        SELECT e.id, e.application_id, e.name, e.webhook_url, e.description, e.hmac_secret,
               e.chain_ids, e.contract_addresses, e.event_signatures,
               e.is_active, e.created_at, e.updated_at
        FROM endpoints e
        JOIN applications a ON e.application_id = a.id
        WHERE a.user_id = ?
        ORDER BY e.created_at DESC
        "#,
        user_id_str
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch endpoints: {e}"),
            }),
        )
    })?;

    let endpoints: Vec<EndpointResponse> = endpoints
        .into_iter()
        .filter_map(|ep| {
            let id = ep.id.and_then(|s| Uuid::parse_str(s.as_str()).ok())?;
            let application_id = Uuid::parse_str(ep.application_id.as_str()).ok()?;
            Some(EndpointResponse {
                id,
                application_id,
                name: ep.name,
                webhook_url: ep.webhook_url,
                description: ep.description,
                hmac_secret: ep.hmac_secret,
                chain_ids: deserialize_json(ep.chain_ids.as_deref()),
                contract_addresses: deserialize_json(ep.contract_addresses.as_deref()),
                event_signatures: deserialize_json(ep.event_signatures.as_deref()),
                is_active: ep.is_active != 0,
                created_at: parse_sqlite_datetime(&ep.created_at),
                updated_at: parse_sqlite_datetime(&ep.updated_at),
            })
        })
        .collect();

    let total = endpoints.len() as i64;

    Ok(Json(EndpointListResponse { endpoints, total }))
}

/// Get a specific endpoint
pub async fn get_endpoint(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
) -> Result<Json<EndpointResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Convert UUIDs to strings for SQLite
    let user_id_str = auth_user.user_id.to_string();
    let endpoint_id_str = endpoint_id.to_string();

    let endpoint = sqlx::query!(
        r#"
        SELECT e.id, e.application_id, e.name, e.webhook_url, e.description, e.hmac_secret,
               e.chain_ids, e.contract_addresses, e.event_signatures,
               e.is_active, e.created_at, e.updated_at
        FROM endpoints e
        JOIN applications a ON e.application_id = a.id
        WHERE e.id = ? AND a.user_id = ?
        "#,
        endpoint_id_str,
        user_id_str
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
                error: "Endpoint not found".to_string(),
            }),
        )
    })?;

    let id = endpoint
        .id
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database returned null ID".to_string(),
                }),
            )
        })
        .and_then(|s| {
            Uuid::parse_str(s.as_str()).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Invalid UUID format".to_string(),
                    }),
                )
            })
        })?;

    let application_id = Uuid::parse_str(endpoint.application_id.as_str()).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Invalid application_id UUID format".to_string(),
            }),
        )
    })?;

    Ok(Json(EndpointResponse {
        id,
        application_id,
        name: endpoint.name,
        webhook_url: endpoint.webhook_url,
        description: endpoint.description,
        hmac_secret: endpoint.hmac_secret,
        chain_ids: deserialize_json(endpoint.chain_ids.as_deref()),
        contract_addresses: deserialize_json(endpoint.contract_addresses.as_deref()),
        event_signatures: deserialize_json(endpoint.event_signatures.as_deref()),
        is_active: endpoint.is_active != 0,
        created_at: parse_sqlite_datetime(&endpoint.created_at),
        updated_at: parse_sqlite_datetime(&endpoint.updated_at),
    }))
}

/// Update an endpoint
pub async fn update_endpoint(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
    Json(payload): Json<UpdateEndpointRequest>,
) -> Result<Json<EndpointResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {e}"),
            }),
        )
    })?;

    // Convert UUIDs to strings for SQLite
    let user_id_str = auth_user.user_id.to_string();
    let endpoint_id_str = endpoint_id.to_string();

    // Verify endpoint exists and belongs to user
    let _existing = sqlx::query!(
        r#"
        SELECT e.id
        FROM endpoints e
        JOIN applications a ON e.application_id = a.id
        WHERE e.id = ? AND a.user_id = ?
        "#,
        endpoint_id_str,
        user_id_str
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
                error: "Endpoint not found".to_string(),
            }),
        )
    })?;

    // Build update query
    let _updates = ["updated_at = datetime('now')"];
    let mut params = vec![];

    if payload.webhook_url.is_some() {
        params.push("webhook_url");
    }
    if payload.description.is_some() {
        params.push("description");
    }
    if payload.chain_ids.is_some() {
        params.push("chain_ids");
    }
    if payload.contract_addresses.is_some() {
        params.push("contract_addresses");
    }
    if payload.event_signatures.is_some() {
        params.push("event_signatures");
    }
    if payload.is_active.is_some() {
        params.push("is_active");
    }

    // For simplicity, we'll use individual updates
    // In production, you'd want to build a dynamic query
    let mut tx = pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Transaction error: {e}"),
            }),
        )
    })?;

    if let Some(url) = &payload.webhook_url {
        sqlx::query!(
            "UPDATE endpoints SET webhook_url = ? WHERE id = ?",
            url,
            endpoint_id_str
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Update error: {e}"),
                }),
            )
        })?;
    }

    if let Some(desc) = &payload.description {
        sqlx::query!(
            "UPDATE endpoints SET description = ? WHERE id = ?",
            desc,
            endpoint_id_str
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Update error: {e}"),
                }),
            )
        })?;
    }

    if let Some(chain_ids) = &payload.chain_ids {
        let chain_ids_json = serialize_json(chain_ids);
        sqlx::query!(
            "UPDATE endpoints SET chain_ids = ? WHERE id = ?",
            chain_ids_json,
            endpoint_id_str
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Update error: {e}"),
                }),
            )
        })?;
    }

    if let Some(addrs) = &payload.contract_addresses {
        let addrs_json = serialize_json(addrs);
        sqlx::query!(
            "UPDATE endpoints SET contract_addresses = ? WHERE id = ?",
            addrs_json,
            endpoint_id_str
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Update error: {e}"),
                }),
            )
        })?;
    }

    if let Some(sigs) = &payload.event_signatures {
        let sigs_json = serialize_json(sigs);
        sqlx::query!(
            "UPDATE endpoints SET event_signatures = ? WHERE id = ?",
            sigs_json,
            endpoint_id_str
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Update error: {e}"),
                }),
            )
        })?;
    }

    if let Some(active) = payload.is_active {
        sqlx::query!(
            "UPDATE endpoints SET is_active = ? WHERE id = ?",
            active,
            endpoint_id_str
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Update error: {e}"),
                }),
            )
        })?;
    }

    // Update timestamp
    sqlx::query!(
        "UPDATE endpoints SET updated_at = datetime('now') WHERE id = ?",
        endpoint_id_str
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Update error: {e}"),
            }),
        )
    })?;

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Transaction commit error: {e}"),
            }),
        )
    })?;

    // Fetch updated endpoint
    get_endpoint(State(pool), auth_user, Path(endpoint_id)).await
}

/// Delete an endpoint
pub async fn delete_endpoint(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Convert UUIDs to strings for SQLite
    let user_id_str = auth_user.user_id.to_string();
    let endpoint_id_str = endpoint_id.to_string();

    let result = sqlx::query!(
        r#"
        DELETE FROM endpoints
        WHERE id = ?
        AND application_id IN (
            SELECT id FROM applications WHERE user_id = ?
        )
        "#,
        endpoint_id_str,
        user_id_str
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to delete endpoint: {e}"),
            }),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Endpoint not found".to_string(),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Regenerate HMAC secret for an endpoint
pub async fn regenerate_hmac_secret(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
) -> Result<Json<EndpointResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Convert UUIDs to strings for SQLite
    let user_id_str = auth_user.user_id.to_string();
    let endpoint_id_str = endpoint_id.to_string();

    // Generate new HMAC secret
    let new_secret = generate_hmac_secret();

    let endpoint = sqlx::query!(
        r#"
        UPDATE endpoints
        SET hmac_secret = ?, updated_at = datetime('now')
        WHERE id = ?
        AND application_id IN (
            SELECT id FROM applications WHERE user_id = ?
        )
        RETURNING id, application_id, name, webhook_url, description, hmac_secret,
                  chain_ids, contract_addresses, event_signatures,
                  is_active, created_at, updated_at
        "#,
        new_secret,
        endpoint_id_str,
        user_id_str
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to regenerate secret: {e}"),
            }),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Endpoint not found".to_string(),
            }),
        )
    })?;

    let id = endpoint
        .id
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database returned null ID".to_string(),
                }),
            )
        })
        .and_then(|s| {
            Uuid::parse_str(s.as_str()).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Invalid UUID format".to_string(),
                    }),
                )
            })
        })?;

    let application_id = Uuid::parse_str(endpoint.application_id.as_str()).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Invalid application_id UUID format".to_string(),
            }),
        )
    })?;

    Ok(Json(EndpointResponse {
        id,
        application_id,
        name: endpoint.name,
        webhook_url: endpoint.webhook_url,
        description: endpoint.description,
        hmac_secret: endpoint.hmac_secret,
        chain_ids: deserialize_json(endpoint.chain_ids.as_deref()),
        contract_addresses: deserialize_json(endpoint.contract_addresses.as_deref()),
        event_signatures: deserialize_json(endpoint.event_signatures.as_deref()),
        is_active: endpoint.is_active != 0,
        created_at: parse_sqlite_datetime(&endpoint.created_at),
        updated_at: parse_sqlite_datetime(&endpoint.updated_at),
    }))
}

/// Generate a secure HMAC secret
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
    fn test_create_endpoint_validation() {
        let app_id = Uuid::new_v4();

        // Valid request
        let valid = CreateEndpointRequest {
            application_id: app_id,
            name: "Test Endpoint".to_string(),
            webhook_url: "https://example.com/webhook".to_string(),
            description: Some("Test endpoint".to_string()),
            chain_ids: vec![1, 137],
            contract_addresses: vec![],
            event_signatures: vec![],
        };
        assert!(valid.validate().is_ok());

        // Invalid URL
        let invalid = CreateEndpointRequest {
            application_id: app_id,
            name: "Test Endpoint".to_string(),
            webhook_url: "not-a-url".to_string(),
            description: None,
            chain_ids: vec![1],
            contract_addresses: vec![],
            event_signatures: vec![],
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_hmac_secret_generation() {
        let secret1 = generate_hmac_secret();
        let secret2 = generate_hmac_secret();

        // Secrets should be different
        assert_ne!(secret1, secret2);

        // Secrets should have correct length
        assert_eq!(secret1.len(), 64);
        assert_eq!(secret2.len(), 64);
    }
}
