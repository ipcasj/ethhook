// Temporary stub for events handlers until ClickHouse integration
// Events and delivery_attempts tables moved to ClickHouse

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::AppState;

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
    pub delivery_count: Option<i64>,
    pub successful_deliveries: Option<i64>,
    pub event_type: String,
    pub chain_id: Option<i32>,
    pub endpoint_name: Option<String>,
    pub status: String,
    pub attempts: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct EventListResponse {
    pub events: Vec<EventResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct DeliveryAttemptResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub endpoint_id: Uuid,
    pub endpoint_name: String,
    pub attempt_number: i32,
    pub status: String,
    pub http_status: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub attempted_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: i32,
}

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub endpoint_id: Option<Uuid>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListDeliveriesQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// List events for authenticated user
pub async fn list_events(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    Query(_query): Query<ListEventsQuery>,
) -> Result<Json<EventListResponse>, StatusCode> {
    // TODO: Implement ClickHouse query
    Ok(Json(EventListResponse {
        events: vec![],
        total: 0,
    }))
}

/// List events for specific endpoint
pub async fn list_endpoint_events(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    Path(_endpoint_id): Path<Uuid>,
    Query(_query): Query<ListEventsQuery>,
) -> Result<Json<EventListResponse>, StatusCode> {
    // TODO: Implement ClickHouse query
    Ok(Json(EventListResponse {
        events: vec![],
        total: 0,
    }))
}

/// Get total event count for user
pub async fn count_events(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement ClickHouse query
    Ok(Json(serde_json::json!({ "count": 0 })))
}

/// Get total event count for endpoint
pub async fn count_endpoint_events(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    Path(_endpoint_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement ClickHouse query
    Ok(Json(serde_json::json!({ "count": 0 })))
}

/// Get single event by ID
pub async fn get_event(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    Path(_event_id): Path<Uuid>,
) -> Result<Json<EventResponse>, StatusCode> {
    // TODO: Implement ClickHouse query
    Err(StatusCode::NOT_FOUND)
}

/// List delivery attempts for an event
pub async fn list_event_deliveries(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    Path(_event_id): Path<Uuid>,
    Query(_query): Query<ListDeliveriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement ClickHouse query
    Ok(Json(serde_json::json!({
        "deliveries": [],
        "total": 0
    })))
}

/// List all delivery attempts (needed by main.rs route)
pub async fn list_delivery_attempts(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
    Query(_query): Query<ListDeliveriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement ClickHouse query
    Ok(Json(serde_json::json!({
        "deliveries": [],
        "total": 0
    })))
}
