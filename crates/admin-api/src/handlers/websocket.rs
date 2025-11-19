/**
 * WebSocket Handler Module
 *
 * Provides real-time updates to the dashboard via WebSocket connections.
 *
 * ## Endpoints
 * - `/ws/events` - Stream live events as they're ingested
 * - `/ws/stats` - Stream live statistics updates
 *
 * ## Architecture
 * ```
 * Event Ingestor → Redis PubSub → Admin API (this module) → WebSocket → Dashboard
 * ```
 *
 * ## Connection Flow
 * 1. Client connects with JWT token in query param
 * 2. Validate JWT and extract user_id
 * 3. Subscribe to Redis pubsub channels
 * 4. Forward messages to WebSocket client
 * 5. Handle disconnection and cleanup
 */
use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::{IntoResponse, Response},
};
use futures::{SinkExt, StreamExt};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{AppState, auth::Claims, config::Config};

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// JWT token for authentication
    token: String,
}

/// Message types sent via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// New event ingested
    #[serde(rename = "event")]
    Event {
        id: String,
        chain_id: i32,
        chain_name: String,
        contract_address: String,
        event_name: String,
        block_number: i64,
        transaction_hash: String,
        timestamp: String,
    },
    /// Statistics update
    #[serde(rename = "stats")]
    Stats {
        events_total: i64,
        events_24h: i64,
        success_rate: f64,
        active_endpoints: i64,
    },
    /// Connection established confirmation
    #[serde(rename = "connected")]
    Connected { message: String },
    /// Heartbeat/ping
    #[serde(rename = "ping")]
    Ping { timestamp: i64 },
    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
}

/// WebSocket connection handler for live events stream
pub async fn ws_events_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    // Validate JWT token
    let claims = match validate_token(&query.token, &state.config) {
        Ok(claims) => claims,
        Err(e) => {
            error!("WebSocket authentication failed: {}", e);
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid or expired token",
            )
                .into_response();
        }
    };

    info!(
        "WebSocket /ws/events connection request from user_id={}",
        claims.sub
    );

    // Upgrade to WebSocket connection
    ws.on_upgrade(move |socket| handle_events_socket(socket, claims.sub, state))
}

/// WebSocket connection handler for live stats stream
pub async fn ws_stats_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    // Validate JWT token
    let claims = match validate_token(&query.token, &state.config) {
        Ok(claims) => claims,
        Err(e) => {
            error!("WebSocket authentication failed: {}", e);
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid or expired token",
            )
                .into_response();
        }
    };

    info!(
        "WebSocket /ws/stats connection request from user_id={}",
        claims.sub
    );

    // Upgrade to WebSocket connection
    ws.on_upgrade(move |socket| handle_stats_socket(socket, claims.sub, state))
}

/// Validate JWT token and extract claims
fn validate_token(token: &str, config: &Config) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
    let validation = Validation::default();

    decode::<Claims>(token, &decoding_key, &validation).map(|data| data.claims)
}

/// Handle WebSocket connection for events stream
async fn handle_events_socket(socket: WebSocket, user_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket /ws/events connected: user_id={}", user_id);

    // Send connection confirmation
    let connected_msg = WsMessage::Connected {
        message: "Connected to events stream".to_string(),
    };
    if let Ok(json) = serde_json::to_string(&connected_msg) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            error!("Failed to send connection confirmation");
            return;
        }
    }

    // Create Redis pubsub connection
    let redis_client = match redis::Client::open(state.config.redis_url.as_str()) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            return;
        }
    };

    let mut pubsub_conn = match redis_client.get_async_connection().await {
        Ok(conn) => conn.into_pubsub(),
        Err(e) => {
            error!("Failed to create Redis pubsub connection: {}", e);
            return;
        }
    };

    // Subscribe to events channel
    if let Err(e) = pubsub_conn.subscribe("webhook:events").await {
        error!("Failed to subscribe to Redis channel: {}", e);
        return;
    }

    info!("Subscribed to Redis channel: webhook:events");

    let mut pubsub_stream = pubsub_conn.on_message();

    // Heartbeat interval (every 30 seconds)
    let mut heartbeat_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            // Handle incoming WebSocket messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket /ws/events client closed connection: user_id={}", user_id);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }

            // Handle messages from Redis pubsub
            msg = pubsub_stream.next() => {
                match msg {
                    Some(msg) => {
                        let payload: String = match msg.get_payload() {
                            Ok(p) => p,
                            Err(e) => {
                                error!("Failed to get Redis payload: {}", e);
                                continue;
                            }
                        };

                        debug!("Received event from Redis: {}", payload);

                        // Forward to WebSocket client
                        if sender.send(Message::Text(payload.into())).await.is_err() {
                            error!("Failed to send message to WebSocket client");
                            break;
                        }
                    }
                    None => {
                        warn!("Redis pubsub stream ended");
                        break;
                    }
                }
            }

            // Send periodic heartbeat
            _ = heartbeat_interval.tick() => {
                let ping = WsMessage::Ping {
                    timestamp: chrono::Utc::now().timestamp(),
                };
                if let Ok(json) = serde_json::to_string(&ping) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket /ws/events disconnected: user_id={}", user_id);
}

/// Handle WebSocket connection for stats stream
async fn handle_stats_socket(socket: WebSocket, user_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket /ws/stats connected: user_id={}", user_id);

    // Send connection confirmation
    let connected_msg = WsMessage::Connected {
        message: "Connected to stats stream".to_string(),
    };
    if let Ok(json) = serde_json::to_string(&connected_msg) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            error!("Failed to send connection confirmation");
            return;
        }
    }

    // Create Redis pubsub connection
    let redis_client = match redis::Client::open(state.config.redis_url.as_str()) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            return;
        }
    };

    let mut pubsub_conn = match redis_client.get_async_connection().await {
        Ok(conn) => conn.into_pubsub(),
        Err(e) => {
            error!("Failed to create Redis pubsub connection: {}", e);
            return;
        }
    };

    // Subscribe to stats channel
    if let Err(e) = pubsub_conn.subscribe("webhook:stats").await {
        error!("Failed to subscribe to Redis channel: {}", e);
        return;
    }

    info!("Subscribed to Redis channel: webhook:stats");

    let mut pubsub_stream = pubsub_conn.on_message();

    // Heartbeat and stats refresh interval (every 5 seconds)
    let mut stats_interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        tokio::select! {
            // Handle incoming WebSocket messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket /ws/stats client closed connection: user_id={}", user_id);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }

            // Handle messages from Redis pubsub
            msg = pubsub_stream.next() => {
                match msg {
                    Some(msg) => {
                        let payload: String = match msg.get_payload() {
                            Ok(p) => p,
                            Err(e) => {
                                error!("Failed to get Redis payload: {}", e);
                                continue;
                            }
                        };

                        debug!("Received stats from Redis: {}", payload);

                        // Forward to WebSocket client
                        if sender.send(Message::Text(payload.into())).await.is_err() {
                            error!("Failed to send message to WebSocket client");
                            break;
                        }
                    }
                    None => {
                        warn!("Redis pubsub stream ended");
                        break;
                    }
                }
            }

            // Periodic stats refresh
            _ = stats_interval.tick() => {
                // Query latest stats from database
                match fetch_dashboard_stats(&state.pool).await {
                    Ok(stats) => {
                        let stats_msg = WsMessage::Stats {
                            events_total: stats.0,
                            events_24h: stats.1,
                            success_rate: stats.2,
                            active_endpoints: stats.3,
                        };

                        if let Ok(json) = serde_json::to_string(&stats_msg) {
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch dashboard stats: {}", e);
                    }
                }
            }
        }
    }

    info!("WebSocket /ws/stats disconnected: user_id={}", user_id);
}

/// Fetch latest dashboard statistics
async fn fetch_dashboard_stats(pool: &PgPool) -> Result<(i64, i64, f64, i64), sqlx::Error> {
    let events_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(pool)
        .await?;

    let events_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM events WHERE ingested_at > NOW() - INTERVAL '24 hours'",
    )
    .fetch_one(pool)
    .await?;

    let (successful, total): (i64, i64) = sqlx::query_as(
        "SELECT 
            COUNT(*) FILTER (WHERE success = true) as successful,
            COUNT(*) as total
         FROM delivery_attempts",
    )
    .fetch_one(pool)
    .await?;

    let success_rate = if total > 0 {
        (successful as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let active_endpoints: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM endpoints WHERE is_active = true")
            .fetch_one(pool)
            .await?;

    Ok((events_total, events_24h, success_rate, active_endpoints))
}
