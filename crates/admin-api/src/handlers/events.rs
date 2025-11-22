use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::auth::AuthUser;

/// Event response
#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: Uuid,
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub log_index: i32,
    pub contract_address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub ingested_at: chrono::DateTime<chrono::Utc>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    // Include delivery info if available
    pub delivery_count: Option<i64>,
    pub successful_deliveries: Option<i64>,
    // Derived fields for UI
    pub event_type: String,            // First topic (event signature)
    pub chain_id: Option<i32>,         // From endpoint
    pub endpoint_name: Option<String>, // From endpoint
    pub status: String,                // 'delivered', 'failed', 'pending'
    pub attempts: i64,                 // Total attempts
    pub created_at: chrono::DateTime<chrono::Utc>, // Alias for ingested_at
}

/// List of events response
#[derive(Debug, Serialize)]
pub struct EventListResponse {
    pub events: Vec<EventResponse>,
    pub total: i64,
}

/// Delivery attempt response
#[derive(Debug, Serialize)]
pub struct DeliveryAttemptResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub endpoint_id: Uuid,
    pub endpoint_name: String,
    pub attempt_number: i32,
    pub http_status_code: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub attempted_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<i32>,
    pub success: Option<bool>,
    pub should_retry: Option<bool>,
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// List of delivery attempts response
#[derive(Debug, Serialize)]
pub struct DeliveryAttemptListResponse {
    pub delivery_attempts: Vec<DeliveryAttemptResponse>,
    pub total: i64,
}

/// Query parameters for listing events
#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub endpoint_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for listing delivery attempts
#[derive(Debug, Deserialize)]
pub struct ListDeliveryAttemptsQuery {
    pub event_id: Option<Uuid>,
    pub endpoint_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// List all events for the authenticated user's endpoints
pub async fn list_events(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Query(params): Query<ListEventsQuery>,
) -> Result<Json<EventListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Get events that have been delivered to user's endpoints
    let (events, total) = match params.endpoint_id {
        Some(endpoint_id) => {
            // Verify endpoint belongs to user first
            let endpoint_check = sqlx::query!(
                r#"
                SELECT e.id
                FROM endpoints e
                JOIN applications a ON e.application_id = a.id
                WHERE e.id = ? AND a.user_id = ?
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
            })?;

            if endpoint_check.is_none() {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Endpoint not found".to_string(),
                    }),
                ));
            }

            // Get events for this specific endpoint
            let events = list_events_for_endpoint(&pool, endpoint_id, limit, offset).await?;
            let total = count_events_for_endpoint(&pool, endpoint_id).await?;
            (events, total)
        }
        None => {
            // Get all events for all user's endpoints
            let events = list_events_for_user(&pool, auth_user.user_id, limit, offset).await?;
            let total = count_events_for_user(&pool, auth_user.user_id).await?;
            (events, total)
        }
    };

    Ok(Json(EventListResponse { events, total }))
}

async fn list_events_for_endpoint(
    pool: &SqlitePool,
    endpoint_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<EventResponse>, (StatusCode, Json<ErrorResponse>)> {
    let records = sqlx::query!(
        r#"
        SELECT DISTINCT e.id, e.block_number, e.block_hash, e.transaction_hash,
               e.log_index, e.contract_address, e.topics, e.data,
               e.ingested_at, e.processed_at,
               COUNT(da.id) as "delivery_count!",
               COUNT(da.id) FILTER (WHERE da.success = true) as "successful_deliveries!",
               ep.name as endpoint_name,
               (CASE WHEN CARDINALITY(ep.chain_ids) > 0 THEN ep.chain_ids[1] ELSE NULL END) as chain_id,
               (CASE 
                   WHEN COUNT(da.id) FILTER (WHERE da.success = true) > 0 THEN 'delivered'
                   WHEN COUNT(da.id) > 0 THEN 'failed'
                   ELSE 'pending'
               END) as "status!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        WHERE da.endpoint_id = ?
        GROUP BY e.id, ep.name, ep.chain_ids
        ORDER BY e.block_number DESC, e.log_index DESC
        LIMIT ? OFFSET ?
        "#,
        endpoint_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch events: {e}"),
            }),
        )
    })?;

    Ok(records
        .into_iter()
        .map(|ev| {
            let event_type = ev
                .topics
                .first()
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());

            EventResponse {
                id: ev.id,
                block_number: ev.block_number,
                block_hash: ev.block_hash.clone(),
                transaction_hash: ev.transaction_hash.clone(),
                log_index: ev.log_index,
                contract_address: ev.contract_address.clone(),
                topics: ev.topics.clone(),
                data: ev.data.clone(),
                ingested_at: ev.ingested_at.unwrap_or_else(chrono::Utc::now),
                processed_at: ev.processed_at,
                delivery_count: Some(ev.delivery_count),
                successful_deliveries: Some(ev.successful_deliveries),
                event_type,
                chain_id: ev.chain_id,
                endpoint_name: Some(ev.endpoint_name),
                status: ev.status,
                attempts: ev.delivery_count,
                created_at: ev.ingested_at.unwrap_or_else(chrono::Utc::now),
            }
        })
        .collect())
}

async fn list_events_for_user(
    pool: &SqlitePool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<EventResponse>, (StatusCode, Json<ErrorResponse>)> {
    let records = sqlx::query!(
        r#"
        SELECT DISTINCT ON (e.id) 
               e.id, e.block_number, e.block_hash, e.transaction_hash,
               e.log_index, e.contract_address, e.topics, e.data,
               e.ingested_at, e.processed_at,
               COUNT(da.id) OVER (PARTITION BY e.id) as "delivery_count!",
               COUNT(da.id) FILTER (WHERE da.success = true) OVER (PARTITION BY e.id) as "successful_deliveries!",
               ep.name as endpoint_name,
               (CASE WHEN CARDINALITY(ep.chain_ids) > 0 THEN ep.chain_ids[1] ELSE NULL END) as chain_id,
               (CASE 
                   WHEN COUNT(da.id) FILTER (WHERE da.success = true) OVER (PARTITION BY e.id) > 0 THEN 'delivered'
                   WHEN COUNT(da.id) OVER (PARTITION BY e.id) > 0 THEN 'failed'
                   ELSE 'pending'
               END) as "status!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id
        WHERE a.user_id = ?
        ORDER BY e.id, e.block_number DESC, e.log_index DESC
        LIMIT ? OFFSET ?
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch events: {e}"),
            }),
        )
    })?;

    Ok(records
        .into_iter()
        .map(|ev| {
            let event_type = ev
                .topics
                .first()
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());

            EventResponse {
                id: ev.id,
                block_number: ev.block_number,
                block_hash: ev.block_hash.clone(),
                transaction_hash: ev.transaction_hash.clone(),
                log_index: ev.log_index,
                contract_address: ev.contract_address.clone(),
                topics: ev.topics.clone(),
                data: ev.data.clone(),
                ingested_at: ev.ingested_at.unwrap_or_else(chrono::Utc::now),
                processed_at: ev.processed_at,
                delivery_count: Some(ev.delivery_count),
                successful_deliveries: Some(ev.successful_deliveries),
                event_type,
                chain_id: ev.chain_id,
                endpoint_name: Some(ev.endpoint_name),
                status: ev.status,
                attempts: ev.delivery_count,
                created_at: ev.ingested_at.unwrap_or_else(chrono::Utc::now),
            }
        })
        .collect())
}

async fn count_events_for_endpoint(
    pool: &SqlitePool,
    endpoint_id: Uuid,
) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(DISTINCT e.id) as "count!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        WHERE da.endpoint_id = ?
        "#,
        endpoint_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to count events: {e}"),
            }),
        )
    })?;

    Ok(result.count)
}

async fn count_events_for_user(
    pool: &SqlitePool,
    user_id: Uuid,
) -> Result<i64, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(DISTINCT e.id) as "count!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id
        WHERE a.user_id = ?
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to count events: {e}"),
            }),
        )
    })?;

    Ok(result.count)
}

/// Get a specific event by ID
pub async fn get_event(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventResponse>, (StatusCode, Json<ErrorResponse>)> {
    let event = sqlx::query!(
        r#"
        SELECT DISTINCT ON (e.id)
               e.id, e.block_number, e.block_hash, e.transaction_hash,
               e.log_index, e.contract_address, e.topics, e.data,
               e.ingested_at, e.processed_at,
               COUNT(da.id) OVER (PARTITION BY e.id) as "delivery_count!",
               COUNT(da.id) FILTER (WHERE da.success = true) OVER (PARTITION BY e.id) as "successful_deliveries!",
               ep.name as endpoint_name,
               (CASE WHEN CARDINALITY(ep.chain_ids) > 0 THEN ep.chain_ids[1] ELSE NULL END) as chain_id,
               (CASE 
                   WHEN COUNT(da.id) FILTER (WHERE da.success = true) OVER (PARTITION BY e.id) > 0 THEN 'delivered'
                   WHEN COUNT(da.id) OVER (PARTITION BY e.id) > 0 THEN 'failed'
                   ELSE 'pending'
               END) as "status!"
        FROM events e
        LEFT JOIN delivery_attempts da ON e.id = da.event_id
        LEFT JOIN endpoints ep ON da.endpoint_id = ep.id
        LEFT JOIN applications a ON ep.application_id = a.id
        WHERE e.id = ? AND (a.user_id = ? OR a.user_id IS NULL)
        ORDER BY e.id
        "#,
        event_id,
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
                error: "Event not found".to_string(),
            }),
        )
    })?;

    let event_type = event
        .topics
        .first()
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(Json(EventResponse {
        id: event.id,
        block_number: event.block_number,
        block_hash: event.block_hash.clone(),
        transaction_hash: event.transaction_hash.clone(),
        log_index: event.log_index,
        contract_address: event.contract_address.clone(),
        topics: event.topics.clone(),
        data: event.data.clone(),
        ingested_at: event.ingested_at.unwrap_or_else(chrono::Utc::now),
        processed_at: event.processed_at,
        delivery_count: Some(event.delivery_count),
        successful_deliveries: Some(event.successful_deliveries),
        event_type,
        chain_id: event.chain_id,
        endpoint_name: Some(event.endpoint_name),
        status: event.status,
        attempts: event.delivery_count,
        created_at: event.ingested_at.unwrap_or_else(chrono::Utc::now),
    }))
}

/// List delivery attempts for events
pub async fn list_delivery_attempts(
    State(pool): State<SqlitePool>,
    auth_user: AuthUser,
    Query(params): Query<ListDeliveryAttemptsQuery>,
) -> Result<Json<DeliveryAttemptListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Call appropriate helper based on filter
    let delivery_attempts = if let Some(event_id) = params.event_id {
        list_delivery_attempts_for_event(&pool, auth_user.user_id, event_id, limit, offset).await?
    } else if let Some(endpoint_id) = params.endpoint_id {
        list_delivery_attempts_for_endpoint(&pool, auth_user.user_id, endpoint_id, limit, offset)
            .await?
    } else {
        list_delivery_attempts_for_user(&pool, auth_user.user_id, limit, offset).await?
    };

    let total = delivery_attempts.len() as i64;
    Ok(Json(DeliveryAttemptListResponse {
        delivery_attempts,
        total,
    }))
}

async fn list_delivery_attempts_for_event(
    pool: &SqlitePool,
    user_id: Uuid,
    event_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<DeliveryAttemptResponse>, (StatusCode, Json<ErrorResponse>)> {
    let records = sqlx::query!(
        r#"
        SELECT da.id, da.event_id, da.endpoint_id, ep.name as endpoint_name,
               da.attempt_number, da.http_status_code, da.response_body,
               da.error_message, da.attempted_at, da.completed_at,
               da.duration_ms, da.success, da.should_retry, da.next_retry_at
        FROM delivery_attempts da
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id
        WHERE da.event_id = ? AND a.user_id = ?
        ORDER BY da.attempted_at DESC
        LIMIT ? OFFSET ?
        "#,
        event_id,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch delivery attempts: {e}"),
            }),
        )
    })?;

    Ok(records
        .into_iter()
        .map(|da| DeliveryAttemptResponse {
            id: da.id,
            event_id: da.event_id,
            endpoint_id: da.endpoint_id,
            endpoint_name: da.endpoint_name,
            attempt_number: da.attempt_number,
            http_status_code: da.http_status_code,
            response_body: da.response_body,
            error_message: da.error_message,
            attempted_at: da.attempted_at.unwrap_or_else(chrono::Utc::now),
            completed_at: da.completed_at,
            duration_ms: da.duration_ms,
            success: da.success,
            should_retry: da.should_retry,
            next_retry_at: da.next_retry_at,
        })
        .collect())
}

async fn list_delivery_attempts_for_endpoint(
    pool: &SqlitePool,
    user_id: Uuid,
    endpoint_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<DeliveryAttemptResponse>, (StatusCode, Json<ErrorResponse>)> {
    let records = sqlx::query!(
        r#"
        SELECT da.id, da.event_id, da.endpoint_id, ep.name as endpoint_name,
               da.attempt_number, da.http_status_code, da.response_body,
               da.error_message, da.attempted_at, da.completed_at,
               da.duration_ms, da.success, da.should_retry, da.next_retry_at
        FROM delivery_attempts da
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id
        WHERE da.endpoint_id = ? AND a.user_id = ?
        ORDER BY da.attempted_at DESC
        LIMIT ? OFFSET ?
        "#,
        endpoint_id,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch delivery attempts: {e}"),
            }),
        )
    })?;

    Ok(records
        .into_iter()
        .map(|da| DeliveryAttemptResponse {
            id: da.id,
            event_id: da.event_id,
            endpoint_id: da.endpoint_id,
            endpoint_name: da.endpoint_name,
            attempt_number: da.attempt_number,
            http_status_code: da.http_status_code,
            response_body: da.response_body,
            error_message: da.error_message,
            attempted_at: da.attempted_at.unwrap_or_else(chrono::Utc::now),
            completed_at: da.completed_at,
            duration_ms: da.duration_ms,
            success: da.success,
            should_retry: da.should_retry,
            next_retry_at: da.next_retry_at,
        })
        .collect())
}

async fn list_delivery_attempts_for_user(
    pool: &SqlitePool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<DeliveryAttemptResponse>, (StatusCode, Json<ErrorResponse>)> {
    let records = sqlx::query!(
        r#"
        SELECT da.id, da.event_id, da.endpoint_id, ep.name as endpoint_name,
               da.attempt_number, da.http_status_code, da.response_body,
               da.error_message, da.attempted_at, da.completed_at,
               da.duration_ms, da.success, da.should_retry, da.next_retry_at
        FROM delivery_attempts da
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id
        WHERE a.user_id = ?
        ORDER BY da.attempted_at DESC
        LIMIT ? OFFSET ?
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch delivery attempts: {e}"),
            }),
        )
    })?;

    Ok(records
        .into_iter()
        .map(|da| DeliveryAttemptResponse {
            id: da.id,
            event_id: da.event_id,
            endpoint_id: da.endpoint_id,
            endpoint_name: da.endpoint_name,
            attempt_number: da.attempt_number,
            http_status_code: da.http_status_code,
            response_body: da.response_body,
            error_message: da.error_message,
            attempted_at: da.attempted_at.unwrap_or_else(chrono::Utc::now),
            completed_at: da.completed_at,
            duration_ms: da.duration_ms,
            success: da.success,
            should_retry: da.should_retry,
            next_retry_at: da.next_retry_at,
        })
        .collect())
}
