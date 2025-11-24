// Events handlers with ClickHouse integration
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: String,
    pub endpoint_id: String,
    pub application_id: String,
    pub chain_id: u32,
    pub block_number: u64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub log_index: u32,
    pub contract_address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub ingested_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct EventListResponse {
    pub events: Vec<EventResponse>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct DeliveryAttemptResponse {
    pub id: String,
    pub event_id: String,
    pub endpoint_id: String,
    pub attempt_number: u8,
    pub status: String,
    pub http_status: u16,
    pub error_message: String,
    pub attempted_at: DateTime<Utc>,
    pub duration_ms: u32,
}

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub endpoint_id: Option<Uuid>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ListDeliveriesQuery {
    pub status: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// List events for authenticated user
pub async fn list_events(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<EventListResponse>, StatusCode> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).min(100);
    let offset = (page - 1) * page_size;

    let client = state.clickhouse.client();
    let user_id = auth_user.user_id;

    // Build query based on endpoint filter
    let (events_query, count_query) = if let Some(endpoint_id) = query.endpoint_id {
        // Verify endpoint belongs to user via SQLite
        let endpoint_check = sqlx::query!(
            "SELECT e.id FROM endpoints e 
             JOIN applications a ON e.application_id = a.id 
             WHERE e.id = ? AND a.user_id = ?",
            endpoint_id,
            user_id
        )
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if endpoint_check.is_none() {
            return Err(StatusCode::NOT_FOUND);
        }

        let endpoint_id_str = endpoint_id.to_string();
        (
            format!(
                "SELECT id, endpoint_id, application_id, chain_id, block_number, 
                 block_hash, transaction_hash, log_index, contract_address, 
                 topics, data, toUnixTimestamp64Milli(ingested_at) as ingested_at_ts,
                 toUnixTimestamp64Milli(processed_at) as processed_at_ts
                 FROM events 
                 WHERE endpoint_id = '{endpoint_id_str}' AND user_id = '{user_id}'
                 ORDER BY ingested_at DESC, id DESC
                 LIMIT {page_size} OFFSET {offset}"
            ),
            format!(
                "SELECT count() as total FROM events 
                 WHERE endpoint_id = '{endpoint_id_str}' AND user_id = '{user_id}'"
            ),
        )
    } else {
        (
            format!(
                "SELECT id, endpoint_id, application_id, chain_id, block_number, 
                 block_hash, transaction_hash, log_index, contract_address, 
                 topics, data, toUnixTimestamp64Milli(ingested_at) as ingested_at_ts,
                 toUnixTimestamp64Milli(processed_at) as processed_at_ts
                 FROM events 
                 WHERE user_id = '{user_id}'
                 ORDER BY ingested_at DESC, id DESC
                 LIMIT {page_size} OFFSET {offset}"
            ),
            format!("SELECT count() as total FROM events WHERE user_id = '{user_id}'"),
        )
    };

    // Fetch events
    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct EventRow {
        id: String,
        endpoint_id: String,
        application_id: String,
        chain_id: u32,
        block_number: u64,
        block_hash: String,
        transaction_hash: String,
        log_index: u32,
        contract_address: String,
        topics: Vec<String>,
        data: String,
        ingested_at_ts: i64,
        processed_at_ts: i64,
    }

    let events: Vec<EventRow> = client
        .query(&events_query)
        .fetch_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get total count
    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct CountRow {
        total: u64,
    }

    let count: Vec<CountRow> = client
        .query(&count_query)
        .fetch_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = count.first().map(|c| c.total).unwrap_or(0);

    let events: Vec<EventResponse> = events
        .into_iter()
        .map(|e| EventResponse {
            id: e.id,
            endpoint_id: e.endpoint_id,
            application_id: e.application_id,
            chain_id: e.chain_id,
            block_number: e.block_number,
            block_hash: e.block_hash,
            transaction_hash: e.transaction_hash,
            log_index: e.log_index,
            contract_address: e.contract_address,
            topics: e.topics,
            data: e.data,
            ingested_at: DateTime::from_timestamp_millis(e.ingested_at_ts).unwrap_or_else(Utc::now),
            processed_at: if e.processed_at_ts > 0 {
                DateTime::from_timestamp_millis(e.processed_at_ts)
            } else {
                None
            },
        })
        .collect();

    Ok(Json(EventListResponse { events, total }))
}

/// Get single event by ID
pub async fn get_event(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventResponse>, StatusCode> {
    let client = state.clickhouse.client();
    let user_id = auth_user.user_id;
    let event_id_str = event_id.to_string();

    let query = format!(
        "SELECT id, endpoint_id, application_id, chain_id, block_number, 
         block_hash, transaction_hash, log_index, contract_address, 
         topics, data, toUnixTimestamp64Milli(ingested_at) as ingested_at_ts,
         toUnixTimestamp64Milli(processed_at) as processed_at_ts
         FROM events 
         WHERE id = '{event_id_str}' AND user_id = '{user_id}'
         LIMIT 1"
    );

    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct EventRow {
        id: String,
        endpoint_id: String,
        application_id: String,
        chain_id: u32,
        block_number: u64,
        block_hash: String,
        transaction_hash: String,
        log_index: u32,
        contract_address: String,
        topics: Vec<String>,
        data: String,
        ingested_at_ts: i64,
        processed_at_ts: i64,
    }

    let events: Vec<EventRow> = client
        .query(&query)
        .fetch_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event = events.into_iter().next().ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(EventResponse {
        id: event.id,
        endpoint_id: event.endpoint_id,
        application_id: event.application_id,
        chain_id: event.chain_id,
        block_number: event.block_number,
        block_hash: event.block_hash,
        transaction_hash: event.transaction_hash,
        log_index: event.log_index,
        contract_address: event.contract_address,
        topics: event.topics,
        data: event.data,
        ingested_at: DateTime::from_timestamp_millis(event.ingested_at_ts).unwrap_or_else(Utc::now),
        processed_at: if event.processed_at_ts > 0 {
            DateTime::from_timestamp_millis(event.processed_at_ts)
        } else {
            None
        },
    }))
}

/// List all delivery attempts
pub async fn list_delivery_attempts(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListDeliveriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).min(100);
    let offset = (page - 1) * page_size;

    let client = state.clickhouse.client();
    let user_id = auth_user.user_id;

    let status_filter = if let Some(status) = query.status {
        format!("AND status = '{status}'")
    } else {
        String::new()
    };

    let deliveries_query = format!(
        "SELECT id, event_id, endpoint_id, attempt_number, status, http_status,
         error_message, toUnixTimestamp64Milli(attempted_at) as attempted_at_ts, duration_ms
         FROM delivery_attempts 
         WHERE user_id = '{user_id}' {status_filter}
         ORDER BY attempted_at DESC
         LIMIT {page_size} OFFSET {offset}"
    );

    let count_query = format!(
        "SELECT count() as total FROM delivery_attempts WHERE user_id = '{user_id}' {status_filter}"
    );

    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct DeliveryRow {
        id: String,
        event_id: String,
        endpoint_id: String,
        attempt_number: u8,
        status: String,
        http_status: u16,
        error_message: String,
        attempted_at_ts: i64,
        duration_ms: u32,
    }

    let deliveries: Vec<DeliveryRow> = client
        .query(&deliveries_query)
        .fetch_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct CountRow {
        total: u64,
    }

    let count: Vec<CountRow> = client
        .query(&count_query)
        .fetch_all()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = count.first().map(|c| c.total).unwrap_or(0);

    let deliveries: Vec<DeliveryAttemptResponse> = deliveries
        .into_iter()
        .map(|d| DeliveryAttemptResponse {
            id: d.id,
            event_id: d.event_id,
            endpoint_id: d.endpoint_id,
            attempt_number: d.attempt_number,
            status: d.status,
            http_status: d.http_status,
            error_message: d.error_message,
            attempted_at: DateTime::from_timestamp_millis(d.attempted_at_ts)
                .unwrap_or_else(Utc::now),
            duration_ms: d.duration_ms,
        })
        .collect();

    Ok(Json(serde_json::json!({
        "deliveries": deliveries,
        "total": total
    })))
}
