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
    State(pool): State<PgPool>,
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

    // Verify application belongs to user
    let _app = sqlx::query!(
        "SELECT id FROM applications WHERE id = $1 AND user_id = $2",
        payload.application_id,
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

    // Generate HMAC secret
    let hmac_secret = generate_hmac_secret();

    // Create endpoint
    let endpoint = sqlx::query!(
        r#"
        INSERT INTO endpoints (
            application_id, name, webhook_url, description, hmac_secret,
            chain_ids, contract_addresses, event_signatures
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, application_id, name, webhook_url, description, hmac_secret,
                  chain_ids, contract_addresses, event_signatures,
                  is_active, created_at, updated_at
        "#,
        payload.application_id,
        payload.name,
        payload.webhook_url,
        payload.description,
        hmac_secret,
        &payload.chain_ids,
        &payload.contract_addresses,
        &payload.event_signatures
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

    Ok((
        StatusCode::CREATED,
        Json(EndpointResponse {
            id: endpoint.id,
            application_id: endpoint.application_id,
            name: endpoint.name,
            webhook_url: endpoint.webhook_url,
            description: endpoint.description,
            hmac_secret: endpoint.hmac_secret,
            chain_ids: endpoint.chain_ids.unwrap_or_default(),
            contract_addresses: endpoint.contract_addresses.unwrap_or_default(),
            event_signatures: endpoint.event_signatures.unwrap_or_default(),
            is_active: endpoint.is_active.unwrap_or(true),
            created_at: endpoint.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: endpoint.updated_at.unwrap_or_else(chrono::Utc::now),
        }),
    ))
}

/// List endpoints for an application
pub async fn list_endpoints(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<Json<EndpointListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify application belongs to user
    let _app = sqlx::query!(
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

    // Get endpoints
    let endpoints = sqlx::query!(
        r#"
        SELECT id, application_id, name, webhook_url, description, hmac_secret,
               chain_ids, contract_addresses, event_signatures,
               is_active, created_at, updated_at
        FROM endpoints
        WHERE application_id = $1
        ORDER BY created_at DESC
        "#,
        app_id
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

    let total = endpoints.len() as i64;
    let endpoints = endpoints
        .into_iter()
        .map(|ep| EndpointResponse {
            id: ep.id,
            application_id: ep.application_id,
            name: ep.name,
            webhook_url: ep.webhook_url,
            description: ep.description,
            hmac_secret: ep.hmac_secret,
            chain_ids: ep.chain_ids.unwrap_or_default(),
            contract_addresses: ep.contract_addresses.unwrap_or_default(),
            event_signatures: ep.event_signatures.unwrap_or_default(),
            is_active: ep.is_active.unwrap_or(true),
            created_at: ep.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: ep.updated_at.unwrap_or_else(chrono::Utc::now),
        })
        .collect();

    Ok(Json(EndpointListResponse { endpoints, total }))
}

/// List all endpoints for the authenticated user across all their applications
pub async fn list_all_user_endpoints(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
) -> Result<Json<EndpointListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get all endpoints for user's applications
    let endpoints = sqlx::query!(
        r#"
        SELECT e.id, e.application_id, e.name, e.webhook_url, e.description, e.hmac_secret,
               e.chain_ids, e.contract_addresses, e.event_signatures,
               e.is_active, e.created_at, e.updated_at
        FROM endpoints e
        JOIN applications a ON e.application_id = a.id
        WHERE a.user_id = $1
        ORDER BY e.created_at DESC
        "#,
        auth_user.user_id
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

    let total = endpoints.len() as i64;
    let endpoints = endpoints
        .into_iter()
        .map(|ep| EndpointResponse {
            id: ep.id,
            application_id: ep.application_id,
            name: ep.name,
            webhook_url: ep.webhook_url,
            description: ep.description,
            hmac_secret: ep.hmac_secret,
            chain_ids: ep.chain_ids.unwrap_or_default(),
            contract_addresses: ep.contract_addresses.unwrap_or_default(),
            event_signatures: ep.event_signatures.unwrap_or_default(),
            is_active: ep.is_active.unwrap_or(true),
            created_at: ep.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: ep.updated_at.unwrap_or_else(chrono::Utc::now),
        })
        .collect();

    Ok(Json(EndpointListResponse { endpoints, total }))
}

/// Get a specific endpoint
pub async fn get_endpoint(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
) -> Result<Json<EndpointResponse>, (StatusCode, Json<ErrorResponse>)> {
    let endpoint = sqlx::query!(
        r#"
        SELECT e.id, e.application_id, e.name, e.webhook_url, e.description, e.hmac_secret,
               e.chain_ids, e.contract_addresses, e.event_signatures,
               e.is_active, e.created_at, e.updated_at
        FROM endpoints e
        JOIN applications a ON e.application_id = a.id
        WHERE e.id = $1 AND a.user_id = $2
        "#,
        endpoint_id,
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
                error: "Endpoint not found".to_string(),
            }),
        )
    })?;

    Ok(Json(EndpointResponse {
        id: endpoint.id,
        application_id: endpoint.application_id,
        name: endpoint.name,
        webhook_url: endpoint.webhook_url,
        description: endpoint.description,
        hmac_secret: endpoint.hmac_secret,
        chain_ids: endpoint.chain_ids.unwrap_or_default(),
        contract_addresses: endpoint.contract_addresses.unwrap_or_default(),
        event_signatures: endpoint.event_signatures.unwrap_or_default(),
        is_active: endpoint.is_active.unwrap_or(true),
        created_at: endpoint.created_at.unwrap_or_else(chrono::Utc::now),
        updated_at: endpoint.updated_at.unwrap_or_else(chrono::Utc::now),
    }))
}

/// Update an endpoint
pub async fn update_endpoint(
    State(pool): State<PgPool>,
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

    // Verify endpoint exists and belongs to user
    let _existing = sqlx::query!(
        r#"
        SELECT e.id
        FROM endpoints e
        JOIN applications a ON e.application_id = a.id
        WHERE e.id = $1 AND a.user_id = $2
        "#,
        endpoint_id,
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
                error: "Endpoint not found".to_string(),
            }),
        )
    })?;

    // Build update query
    let _updates = ["updated_at = NOW()"];
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
            "UPDATE endpoints SET webhook_url = $1 WHERE id = $2",
            url,
            endpoint_id
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
            "UPDATE endpoints SET description = $1 WHERE id = $2",
            desc,
            endpoint_id
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
        sqlx::query!(
            "UPDATE endpoints SET chain_ids = $1 WHERE id = $2",
            chain_ids,
            endpoint_id
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
        sqlx::query!(
            "UPDATE endpoints SET contract_addresses = $1 WHERE id = $2",
            addrs,
            endpoint_id
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
        sqlx::query!(
            "UPDATE endpoints SET event_signatures = $1 WHERE id = $2",
            sigs,
            endpoint_id
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
            "UPDATE endpoints SET is_active = $1 WHERE id = $2",
            active,
            endpoint_id
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
        "UPDATE endpoints SET updated_at = NOW() WHERE id = $1",
        endpoint_id
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
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query!(
        r#"
        DELETE FROM endpoints
        WHERE id = $1
        AND application_id IN (
            SELECT id FROM applications WHERE user_id = $2
        )
        "#,
        endpoint_id,
        auth_user.user_id
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
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
) -> Result<Json<EndpointResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Generate new HMAC secret
    let new_secret = generate_hmac_secret();

    let endpoint = sqlx::query!(
        r#"
        UPDATE endpoints
        SET hmac_secret = $1, updated_at = NOW()
        WHERE id = $2
        AND application_id IN (
            SELECT id FROM applications WHERE user_id = $3
        )
        RETURNING id, application_id, name, webhook_url, description, hmac_secret,
                  chain_ids, contract_addresses, event_signatures,
                  is_active, created_at, updated_at
        "#,
        new_secret,
        endpoint_id,
        auth_user.user_id
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

    Ok(Json(EndpointResponse {
        id: endpoint.id,
        application_id: endpoint.application_id,
        name: endpoint.name,
        webhook_url: endpoint.webhook_url,
        description: endpoint.description,
        hmac_secret: endpoint.hmac_secret,
        chain_ids: endpoint.chain_ids.unwrap_or_default(),
        contract_addresses: endpoint.contract_addresses.unwrap_or_default(),
        event_signatures: endpoint.event_signatures.unwrap_or_default(),
        is_active: endpoint.is_active.unwrap_or(true),
        created_at: endpoint.created_at.unwrap_or_else(chrono::Utc::now),
        updated_at: endpoint.updated_at.unwrap_or_else(chrono::Utc::now),
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
