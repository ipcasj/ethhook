/**
 * Server-Sent Events (SSE) Handler Module
 *
 * Provides real-time updates to the dashboard via Server-Sent Events.
 * Replaces Redis WebSocket pubsub with in-memory broadcast channels.
 *
 * ## Endpoints
 * - `GET /api/v1/events/stream` - Stream live events as they're ingested
 * - `GET /api/v1/stats/stream` - Stream live statistics updates
 *
 * ## Architecture
 * ```
 * Pipeline → Broadcast Channel → Admin API (this module) → SSE → Dashboard
 * ```
 *
 * ## Why SSE over WebSocket + Redis?
 * - **Zero infrastructure**: No Redis server needed
 * - **Simpler**: HTTP/1.1 compatible, easier to debug
 * - **Automatic reconnection**: Built into EventSource API
 * - **Lower latency**: Direct in-memory broadcast
 * - **Better for one-way updates**: Dashboard only receives, never sends
 *
 * ## Connection Flow
 * 1. Client connects with JWT token in Authorization header
 * 2. Validate JWT and extract user_id
 * 3. Subscribe to broadcast channel
 * 4. Send events as SSE messages
 * 5. Handle disconnection and cleanup
 */

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    TypedHeader,
};
use futures::stream::Stream;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{auth::Claims, AppState};

/// Maximum number of events to buffer in broadcast channel
const EVENT_BUFFER_SIZE: usize = 100;

/// Message types sent via SSE
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SseMessage {
    /// New event ingested
    #[serde(rename = "event")]
    Event {
        endpoint_id: String,
        chain: String,
        block_number: i64,
        transaction_hash: String,
        log_index: i32,
    },

    /// Statistics update
    #[serde(rename = "stats")]
    Stats {
        total_events: i64,
        events_per_second: f64,
        active_endpoints: i32,
    },

    /// Connection confirmation
    #[serde(rename = "connected")]
    Connected { message: String },

    /// Error message
    #[serde(rename = "error")]
    Error { message: String },

    /// Heartbeat to keep connection alive
    #[serde(rename = "ping")]
    Ping { timestamp: i64 },
}

/// Global broadcast channel for events
/// In production, this would be part of AppState
pub static EVENT_BROADCASTER: once_cell::sync::Lazy<broadcast::Sender<SseMessage>> =
    once_cell::sync::Lazy::new(|| {
        let (tx, _) = broadcast::channel(EVENT_BUFFER_SIZE);
        tx
    });

/**
 * SSE handler for live events stream
 *
 * ## Example (JavaScript)
 * ```javascript
 * const eventSource = new EventSource('/api/v1/events/stream', {
 *   headers: { 'Authorization': 'Bearer YOUR_JWT_TOKEN' }
 * });
 *
 * eventSource.onmessage = (event) => {
 *   const data = JSON.parse(event.data);
 *   console.log('Event:', data);
 * };
 *
 * eventSource.onerror = (error) => {
 *   console.error('SSE error:', error);
 * };
 * ```
 */
pub async fn events_stream(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate JWT token
    let claims = decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| {
        warn!("JWT validation failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        warn!("Invalid user_id in JWT: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    info!("SSE events stream: user_id={}", user_id);

    // Subscribe to broadcast channel
    let rx = EVENT_BROADCASTER.subscribe();
    let stream = BroadcastStream::new(rx);

    // Convert broadcast stream to SSE events
    let event_stream = stream.filter_map(|result| match result {
        Ok(msg) => Some(Ok::<Event, Infallible>(
            Event::default().json_data(msg).unwrap(),
        )),
        Err(broadcast::error::RecvError::Lagged(n)) => {
            warn!("SSE client lagged by {} messages", n);
            None
        }
        Err(broadcast::error::RecvError::Closed) => None,
    });

    // Add heartbeat every 30 seconds
    let heartbeat_stream = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
        Duration::from_secs(30),
    ))
    .map(|_| {
        Ok::<Event, Infallible>(
            Event::default()
                .json_data(SseMessage::Ping {
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .unwrap(),
        )
    });

    let merged = tokio_stream::StreamExt::merge(event_stream, heartbeat_stream);

    Ok(Sse::new(merged).keep_alive(KeepAlive::default()))
}

/**
 * SSE handler for live statistics stream
 *
 * ## Example (JavaScript)
 * ```javascript
 * const eventSource = new EventSource('/api/v1/stats/stream', {
 *   headers: { 'Authorization': 'Bearer YOUR_JWT_TOKEN' }
 * });
 * ```
 */
pub async fn stats_stream(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate JWT token
    let claims = decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| {
        warn!("JWT validation failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        warn!("Invalid user_id in JWT: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    info!("SSE stats stream: user_id={}", user_id);

    // Subscribe to broadcast channel (same channel, filter by message type)
    let rx = EVENT_BROADCASTER.subscribe();
    let stream = BroadcastStream::new(rx);

    // Filter only stats messages
    let stats_stream = stream.filter_map(|result| match result {
        Ok(msg) => match msg {
            SseMessage::Stats { .. } => Some(Ok::<Event, Infallible>(
                Event::default().json_data(msg).unwrap(),
            )),
            _ => None,
        },
        Err(broadcast::error::RecvError::Lagged(n)) => {
            warn!("SSE stats client lagged by {} messages", n);
            None
        }
        Err(broadcast::error::RecvError::Closed) => None,
    });

    // Add heartbeat every 30 seconds
    let heartbeat_stream = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
        Duration::from_secs(30),
    ))
    .map(|_| {
        Ok::<Event, Infallible>(
            Event::default()
                .json_data(SseMessage::Ping {
                    timestamp: chrono::Utc::now().timestamp(),
                })
                .unwrap(),
        )
    });

    let merged = tokio_stream::StreamExt::merge(stats_stream, heartbeat_stream);

    Ok(Sse::new(merged).keep_alive(KeepAlive::default()))
}

/**
 * Utility function to broadcast an event to all SSE clients
 * Called by other parts of the application (e.g., pipeline webhook notifications)
 *
 * ## Example
 * ```rust
 * broadcast_event(SseMessage::Event {
 *     endpoint_id: endpoint.id.to_string(),
 *     chain: "ethereum".to_string(),
 *     block_number: 123456,
 *     transaction_hash: "0xabc...".to_string(),
 *     log_index: 0,
 * });
 * ```
 */
pub fn broadcast_event(msg: SseMessage) {
    // send() returns Err if no subscribers - that's OK
    let _ = EVENT_BROADCASTER.send(msg);
}

/**
 * Background task to periodically broadcast statistics
 * Should be spawned when admin-api starts
 */
pub async fn stats_broadcaster_task(pool: sqlx::SqlitePool) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        // Query database for current statistics
        let stats = match get_current_stats(&pool).await {
            Ok(s) => s,
            Err(e) => {
                debug!("Failed to get stats for SSE: {}", e);
                continue;
            }
        };

        broadcast_event(SseMessage::Stats {
            total_events: stats.total_events,
            events_per_second: stats.events_per_second,
            active_endpoints: stats.active_endpoints,
        });
    }
}

#[derive(Debug)]
struct CurrentStats {
    total_events: i64,
    events_per_second: f64,
    active_endpoints: i32,
}

async fn get_current_stats(pool: &sqlx::SqlitePool) -> Result<CurrentStats, sqlx::Error> {
    // Placeholder - implement actual stats queries
    // This would aggregate from events/endpoints tables
    Ok(CurrentStats {
        total_events: 0,
        events_per_second: 0.0,
        active_endpoints: 0,
    })
}
