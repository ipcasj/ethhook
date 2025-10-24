use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Query(params): Query<ListEventsQuery>,
) -> Result<Json<EventListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Get events that have been delivered to user's endpoints
    let events = match params.endpoint_id {
        Some(endpoint_id) => {
            // Verify endpoint belongs to user first
            let endpoint_check = sqlx::query!(
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
            list_events_for_endpoint(&pool, endpoint_id, limit, offset).await?
        }
        None => {
            // Get all events for all user's endpoints
            list_events_for_user(&pool, auth_user.user_id, limit, offset).await?
        }
    };

    let total = events.len() as i64;
    Ok(Json(EventListResponse { events, total }))
}

async fn list_events_for_endpoint(
    pool: &PgPool,
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
               COUNT(da.id) FILTER (WHERE da.success = true) as "successful_deliveries!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        WHERE da.endpoint_id = $1
        GROUP BY e.id
        ORDER BY e.block_number DESC, e.log_index DESC
        LIMIT $2 OFFSET $3
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
        .map(|ev| EventResponse {
            id: ev.id,
            block_number: ev.block_number,
            block_hash: ev.block_hash,
            transaction_hash: ev.transaction_hash,
            log_index: ev.log_index,
            contract_address: ev.contract_address,
            topics: ev.topics,
            data: ev.data,
            ingested_at: ev.ingested_at.unwrap_or_else(chrono::Utc::now),
            processed_at: ev.processed_at,
            delivery_count: Some(ev.delivery_count),
            successful_deliveries: Some(ev.successful_deliveries),
        })
        .collect())
}

async fn list_events_for_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<EventResponse>, (StatusCode, Json<ErrorResponse>)> {
    let records = sqlx::query!(
        r#"
        SELECT DISTINCT e.id, e.block_number, e.block_hash, e.transaction_hash,
               e.log_index, e.contract_address, e.topics, e.data,
               e.ingested_at, e.processed_at,
               COUNT(da.id) as "delivery_count!",
               COUNT(da.id) FILTER (WHERE da.success = true) as "successful_deliveries!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id
        WHERE a.user_id = $1
        GROUP BY e.id
        ORDER BY e.block_number DESC, e.log_index DESC
        LIMIT $2 OFFSET $3
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
        .map(|ev| EventResponse {
            id: ev.id,
            block_number: ev.block_number,
            block_hash: ev.block_hash,
            transaction_hash: ev.transaction_hash,
            log_index: ev.log_index,
            contract_address: ev.contract_address,
            topics: ev.topics,
            data: ev.data,
            ingested_at: ev.ingested_at.unwrap_or_else(chrono::Utc::now),
            processed_at: ev.processed_at,
            delivery_count: Some(ev.delivery_count),
            successful_deliveries: Some(ev.successful_deliveries),
        })
        .collect())
}

/// Get a specific event by ID
pub async fn get_event(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventResponse>, (StatusCode, Json<ErrorResponse>)> {
    let event = sqlx::query!(
        r#"
        SELECT DISTINCT e.id, e.block_number, e.block_hash, e.transaction_hash,
               e.log_index, e.contract_address, e.topics, e.data,
               e.ingested_at, e.processed_at,
               COUNT(da.id) as "delivery_count!",
               COUNT(da.id) FILTER (WHERE da.success = true) as "successful_deliveries!"
        FROM events e
        LEFT JOIN delivery_attempts da ON e.id = da.event_id
        LEFT JOIN endpoints ep ON da.endpoint_id = ep.id
        LEFT JOIN applications a ON ep.application_id = a.id
        WHERE e.id = $1 AND (a.user_id = $2 OR a.user_id IS NULL)
        GROUP BY e.id
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

    Ok(Json(EventResponse {
        id: event.id,
        block_number: event.block_number,
        block_hash: event.block_hash,
        transaction_hash: event.transaction_hash,
        log_index: event.log_index,
        contract_address: event.contract_address,
        topics: event.topics,
        data: event.data,
        ingested_at: event.ingested_at.unwrap_or_else(chrono::Utc::now),
        processed_at: event.processed_at,
        delivery_count: Some(event.delivery_count),
        successful_deliveries: Some(event.successful_deliveries),
    }))
}

/// List delivery attempts for events
pub async fn list_delivery_attempts(
    State(pool): State<PgPool>,
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
    pool: &PgPool,
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
        WHERE da.event_id = $1 AND a.user_id = $2
        ORDER BY da.attempted_at DESC
        LIMIT $3 OFFSET $4
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
    pool: &PgPool,
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
        WHERE da.endpoint_id = $1 AND a.user_id = $2
        ORDER BY da.attempted_at DESC
        LIMIT $3 OFFSET $4
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
    pool: &PgPool,
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
        WHERE a.user_id = $1
        ORDER BY da.attempted_at DESC
        LIMIT $2 OFFSET $3
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
