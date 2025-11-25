// Statistics handlers with ClickHouse integration
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct DashboardStatistics {
    pub events_today: u64,
    pub deliveries_today: u64,
    pub success_rate: f64,
    pub active_endpoints: u64,
    pub total_events: u64,
    pub total_deliveries: u64,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct TimeseriesPoint {
    pub timestamp: DateTime<Utc>,
    pub events: u64,
    pub deliveries: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
}

#[derive(Debug, Deserialize)]
pub struct TimeseriesQuery {
    #[allow(dead_code)]
    pub start_time: Option<String>,
    #[allow(dead_code)]
    pub end_time: Option<String>,
    pub interval: Option<String>, // 'hour', 'day', 'week'
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ChainDistribution {
    pub chain_id: u32,
    pub chain_name: String,
    pub event_count: u64,
    pub delivery_count: u64,
    pub success_rate: f64,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct EndpointPerformance {
    pub endpoint_id: String,
    pub endpoint_name: String,
    pub total_events: u64,
    pub total_deliveries: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
    pub success_rate: f64,
    pub avg_delivery_time_ms: f64,
}

/// Get dashboard statistics for authenticated user
pub async fn get_dashboard_statistics(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.clickhouse.client();
    let user_id = auth_user.user_id;

    // Get today's timestamp
    let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_start_ts = today_start.and_utc().timestamp_millis();

    // Get events today
    let events_today_query = format!(
        "SELECT count() as total FROM events 
         WHERE user_id = '{user_id}' AND toUnixTimestamp64Milli(ingested_at) >= {today_start_ts}"
    );

    // Get deliveries today
    let deliveries_today_query = format!(
        "SELECT count() as total FROM delivery_attempts 
         WHERE user_id = '{user_id}' AND toUnixTimestamp64Milli(attempted_at) >= {today_start_ts}"
    );

    // Get success rate (all time)
    let success_rate_query = format!(
        "SELECT 
         countIf(status = 'success') as successful,
         count() as total
         FROM delivery_attempts 
         WHERE user_id = '{user_id}'"
    );

    // Get active endpoints count from SQLite
    let active_endpoints: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT e.id) FROM endpoints e
         JOIN applications a ON e.application_id = a.id
         WHERE a.user_id = ? AND e.is_active = 1",
    )
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    // Total events
    let total_events_query =
        format!("SELECT count() as total FROM events WHERE user_id = '{user_id}'");

    // Total deliveries
    let total_deliveries_query =
        format!("SELECT count() as total FROM delivery_attempts WHERE user_id = '{user_id}'");

    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct CountRow {
        total: u64,
    }

    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct SuccessRateRow {
        successful: u64,
        total: u64,
    }

    let events_today: Vec<CountRow> = client
        .query(&events_today_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    let deliveries_today: Vec<CountRow> = client
        .query(&deliveries_today_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    let success_rate_data: Vec<SuccessRateRow> = client
        .query(&success_rate_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    let total_events: Vec<CountRow> = client
        .query(&total_events_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    let total_deliveries: Vec<CountRow> = client
        .query(&total_deliveries_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    let success_rate = if let Some(sr) = success_rate_data.first() {
        if sr.total > 0 {
            (sr.successful as f64 / sr.total as f64) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    Ok(Json(serde_json::json!({
        "events_today": events_today.first().map(|c| c.total).unwrap_or(0),
        "deliveries_today": deliveries_today.first().map(|c| c.total).unwrap_or(0),
        "success_rate": success_rate,
        "active_endpoints": active_endpoints,
        "total_events": total_events.first().map(|c| c.total).unwrap_or(0),
        "total_deliveries": total_deliveries.first().map(|c| c.total).unwrap_or(0),
    })))
}

/// Get timeseries statistics
pub async fn get_timeseries_statistics(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<TimeseriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.clickhouse.client();
    let user_id = auth_user.user_id;

    let interval = query.interval.unwrap_or_else(|| "day".to_string());
    let interval_func = match interval.as_str() {
        "hour" => "toStartOfHour",
        "week" => "toStartOfWeek",
        _ => "toStartOfDay",
    };

    // Default to last 30 days
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(30);

    let events_query = format!(
        "SELECT 
         {}(ingested_at) as time_bucket,
         count() as event_count
         FROM events
         WHERE user_id = '{}' 
         AND ingested_at >= toDateTime64({}, 3)
         AND ingested_at <= toDateTime64({}, 3)
         GROUP BY time_bucket
         ORDER BY time_bucket",
        interval_func,
        user_id,
        start_time.timestamp_millis(),
        end_time.timestamp_millis()
    );

    let deliveries_query = format!(
        "SELECT 
         {}(attempted_at) as time_bucket,
         count() as total_deliveries,
         countIf(status = 'success') as successful_deliveries,
         countIf(status = 'failed') as failed_deliveries
         FROM delivery_attempts
         WHERE user_id = '{}'
         AND attempted_at >= toDateTime64({}, 3)
         AND attempted_at <= toDateTime64({}, 3)
         GROUP BY time_bucket
         ORDER BY time_bucket",
        interval_func,
        user_id,
        start_time.timestamp_millis(),
        end_time.timestamp_millis()
    );

    #[derive(Debug, Serialize, clickhouse::Row, Deserialize)]
    struct EventTimeseriesRow {
        time_bucket: i64,
        event_count: u64,
    }

    #[derive(Debug, Serialize, clickhouse::Row, Deserialize)]
    struct DeliveryTimeseriesRow {
        time_bucket: i64,
        total_deliveries: u64,
        successful_deliveries: u64,
        failed_deliveries: u64,
    }

    let events: Vec<EventTimeseriesRow> = client
        .query(&events_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    let deliveries: Vec<DeliveryTimeseriesRow> = client
        .query(&deliveries_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "events": events,
        "deliveries": deliveries,
        "interval": interval,
    })))
}

/// Get chain distribution statistics
pub async fn get_chain_distribution(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = state.clickhouse.client();
    let user_id = auth_user.user_id;

    let query = format!(
        "SELECT 
         chain_id,
         count() as event_count
         FROM events
         WHERE user_id = '{user_id}'
         GROUP BY chain_id
         ORDER BY event_count DESC"
    );

    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct ChainRow {
        chain_id: u32,
        event_count: u64,
    }

    let chains: Vec<ChainRow> = client.query(&query).fetch_all().await.unwrap_or_default();

    // Map chain IDs to names
    let chain_names = std::collections::HashMap::from([
        (1_u32, "Ethereum Mainnet"),
        (11155111_u32, "Sepolia"),
        (42161_u32, "Arbitrum One"),
        (10_u32, "Optimism"),
        (8453_u32, "Base"),
        (137_u32, "Polygon"),
    ]);

    let chains_with_names: Vec<_> = chains
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "chain_id": c.chain_id,
                "chain_name": chain_names.get(&c.chain_id).unwrap_or(&"Unknown"),
                "event_count": c.event_count,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "chains": chains_with_names,
        "total": chains_with_names.len(),
    })))
}

/// Get Alchemy CU statistics (placeholder - needs external data)
pub async fn get_alchemy_cu_stats(
    State(_state): State<AppState>,
    _auth_user: AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // This would require integration with Alchemy API
    Ok(Json(serde_json::json!({
        "total_cu": 0,
        "by_chain": [],
        "note": "Alchemy CU tracking requires external API integration"
    })))
}

/// Get statistics for specific application
pub async fn get_application_statistics(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(application_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify application belongs to user
    let app_check = sqlx::query!(
        "SELECT id FROM applications WHERE id = ? AND user_id = ?",
        application_id,
        auth_user.user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if app_check.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let client = state.clickhouse.client();
    let app_id_str = application_id.to_string();

    let stats_query = format!(
        "SELECT 
         count() as total_events,
         (SELECT count() FROM delivery_attempts WHERE application_id = '{app_id_str}') as total_deliveries,
         (SELECT countIf(status = 'success') FROM delivery_attempts WHERE application_id = '{app_id_str}') as successful_deliveries
         FROM events
         WHERE application_id = '{app_id_str}'"
    );

    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct AppStatsRow {
        total_events: u64,
        total_deliveries: u64,
        successful_deliveries: u64,
    }

    let stats: Vec<AppStatsRow> = client
        .query(&stats_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    let stats = stats.first();
    let success_rate = if let Some(s) = stats {
        if s.total_deliveries > 0 {
            (s.successful_deliveries as f64 / s.total_deliveries as f64) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    Ok(Json(serde_json::json!({
        "application_id": application_id,
        "total_events": stats.map(|s| s.total_events).unwrap_or(0),
        "total_deliveries": stats.map(|s| s.total_deliveries).unwrap_or(0),
        "successful_deliveries": stats.map(|s| s.successful_deliveries).unwrap_or(0),
        "success_rate": success_rate,
    })))
}

/// Get timeseries for specific application
pub async fn get_application_timeseries(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(application_id): Path<Uuid>,
    Query(query): Query<TimeseriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify application belongs to user
    let app_check = sqlx::query!(
        "SELECT id FROM applications WHERE id = ? AND user_id = ?",
        application_id,
        auth_user.user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if app_check.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let client = state.clickhouse.client();
    let app_id_str = application_id.to_string();
    let interval = query.interval.unwrap_or_else(|| "day".to_string());
    let interval_func = match interval.as_str() {
        "hour" => "toStartOfHour",
        "week" => "toStartOfWeek",
        _ => "toStartOfDay",
    };

    let end_time = Utc::now();
    let start_time = end_time - Duration::days(30);

    let timeseries_query = format!(
        "SELECT 
         {}(ingested_at) as time_bucket,
         count() as event_count
         FROM events
         WHERE application_id = '{}'
         AND ingested_at >= toDateTime64({}, 3)
         AND ingested_at <= toDateTime64({}, 3)
         GROUP BY time_bucket
         ORDER BY time_bucket",
        interval_func,
        app_id_str,
        start_time.timestamp_millis(),
        end_time.timestamp_millis()
    );

    #[derive(Debug, Serialize, clickhouse::Row, Deserialize)]
    struct TimeseriesRow {
        time_bucket: i64,
        event_count: u64,
    }

    let timeseries: Vec<TimeseriesRow> = client
        .query(&timeseries_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "timeseries": timeseries,
        "total_points": timeseries.len(),
    })))
}

/// Get endpoint statistics
pub async fn get_endpoint_statistics(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify endpoint belongs to user
    let endpoint_check = sqlx::query!(
        "SELECT e.id, e.name FROM endpoints e
         JOIN applications a ON e.application_id = a.id
         WHERE e.id = ? AND a.user_id = ?",
        endpoint_id,
        auth_user.user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let endpoint_name = match endpoint_check {
        Some(ep) => ep.name,
        None => return Err(StatusCode::NOT_FOUND),
    };

    let client = state.clickhouse.client();
    let endpoint_id_str = endpoint_id.to_string();

    let stats_query = format!(
        "SELECT 
         (SELECT count() FROM events WHERE endpoint_id = '{endpoint_id_str}') as total_events,
         count() as total_deliveries,
         countIf(status = 'success') as successful_deliveries,
         countIf(status = 'failed') as failed_deliveries,
         avg(duration_ms) as avg_delivery_time_ms
         FROM delivery_attempts
         WHERE endpoint_id = '{endpoint_id_str}'"
    );

    #[derive(Debug, clickhouse::Row, Deserialize)]
    struct EndpointStatsRow {
        total_events: u64,
        total_deliveries: u64,
        successful_deliveries: u64,
        failed_deliveries: u64,
        avg_delivery_time_ms: f64,
    }

    let stats: Vec<EndpointStatsRow> = client
        .query(&stats_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    let stats = stats.first();
    let success_rate = if let Some(s) = stats {
        if s.total_deliveries > 0 {
            (s.successful_deliveries as f64 / s.total_deliveries as f64) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    Ok(Json(serde_json::json!({
        "endpoint_id": endpoint_id,
        "endpoint_name": endpoint_name,
        "total_events": stats.map(|s| s.total_events).unwrap_or(0),
        "total_deliveries": stats.map(|s| s.total_deliveries).unwrap_or(0),
        "successful_deliveries": stats.map(|s| s.successful_deliveries).unwrap_or(0),
        "failed_deliveries": stats.map(|s| s.failed_deliveries).unwrap_or(0),
        "success_rate": success_rate,
        "avg_delivery_time_ms": stats.map(|s| s.avg_delivery_time_ms).unwrap_or(0.0),
    })))
}

/// Get timeseries for specific endpoint
pub async fn get_endpoint_timeseries(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
    Query(query): Query<TimeseriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify endpoint belongs to user
    let endpoint_check = sqlx::query!(
        "SELECT e.id FROM endpoints e
         JOIN applications a ON e.application_id = a.id
         WHERE e.id = ? AND a.user_id = ?",
        endpoint_id,
        auth_user.user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if endpoint_check.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let client = state.clickhouse.client();
    let endpoint_id_str = endpoint_id.to_string();
    let interval = query.interval.unwrap_or_else(|| "day".to_string());
    let interval_func = match interval.as_str() {
        "hour" => "toStartOfHour",
        "week" => "toStartOfWeek",
        _ => "toStartOfDay",
    };

    let end_time = Utc::now();
    let start_time = end_time - Duration::days(30);

    let timeseries_query = format!(
        "SELECT 
         {}(attempted_at) as time_bucket,
         count() as total_deliveries,
         countIf(status = 'success') as successful_deliveries,
         avg(duration_ms) as avg_duration_ms
         FROM delivery_attempts
         WHERE endpoint_id = '{}'
         AND attempted_at >= toDateTime64({}, 3)
         AND attempted_at <= toDateTime64({}, 3)
         GROUP BY time_bucket
         ORDER BY time_bucket",
        interval_func,
        endpoint_id_str,
        start_time.timestamp_millis(),
        end_time.timestamp_millis()
    );

    #[derive(Debug, Serialize, clickhouse::Row, Deserialize)]
    struct TimeseriesRow {
        time_bucket: i64,
        total_deliveries: u64,
        successful_deliveries: u64,
        avg_duration_ms: f64,
    }

    let timeseries: Vec<TimeseriesRow> = client
        .query(&timeseries_query)
        .fetch_all()
        .await
        .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "timeseries": timeseries,
        "total_points": timeseries.len(),
    })))
}

/// Get delivery attempts for endpoint
pub async fn get_endpoint_deliveries(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(endpoint_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify endpoint belongs to user
    let endpoint_check = sqlx::query!(
        "SELECT e.id FROM endpoints e
         JOIN applications a ON e.application_id = a.id
         WHERE e.id = ? AND a.user_id = ?",
        endpoint_id,
        auth_user.user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if endpoint_check.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let client = state.clickhouse.client();
    let endpoint_id_str = endpoint_id.to_string();

    let query = format!(
        "SELECT 
         id, event_id, attempt_number, status, http_status,
         toUnixTimestamp64Milli(attempted_at) as attempted_at_ts, duration_ms
         FROM delivery_attempts
         WHERE endpoint_id = '{endpoint_id_str}'
         ORDER BY attempted_at DESC
         LIMIT 100"
    );

    #[derive(Debug, Serialize, clickhouse::Row, Deserialize)]
    struct DeliveryRow {
        id: String,
        event_id: String,
        attempt_number: u8,
        status: String,
        http_status: u16,
        attempted_at_ts: i64,
        duration_ms: u32,
    }

    let deliveries: Vec<DeliveryRow> = client.query(&query).fetch_all().await.unwrap_or_default();

    Ok(Json(serde_json::json!({
        "deliveries": deliveries,
        "total": deliveries.len(),
    })))
}

/// Get endpoints performance for application
pub async fn get_application_endpoints_performance(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(application_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify application belongs to user
    let app_check = sqlx::query!(
        "SELECT id FROM applications WHERE id = ? AND user_id = ?",
        application_id,
        auth_user.user_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if app_check.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get all endpoints for this application from SQLite
    let endpoints = sqlx::query!(
        "SELECT id, name FROM endpoints WHERE application_id = ?",
        application_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client = state.clickhouse.client();
    let mut performances = Vec::new();

    for endpoint in endpoints {
        let endpoint_id_str = endpoint.id.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        let endpoint_id_str = endpoint_id_str.as_str();

        let stats_query = format!(
            "SELECT 
             (SELECT count() FROM events WHERE endpoint_id = '{endpoint_id_str}') as total_events,
             count() as total_deliveries,
             countIf(status = 'success') as successful_deliveries,
             avg(duration_ms) as avg_delivery_time_ms
             FROM delivery_attempts
             WHERE endpoint_id = '{endpoint_id_str}'"
        );

        #[derive(Debug, clickhouse::Row, Deserialize)]
        struct StatsRow {
            total_events: u64,
            total_deliveries: u64,
            successful_deliveries: u64,
            avg_delivery_time_ms: f64,
        }

        if let Ok(stats) = client.query(&stats_query).fetch_all::<StatsRow>().await {
            if let Some(s) = stats.first() {
                let success_rate = if s.total_deliveries > 0 {
                    (s.successful_deliveries as f64 / s.total_deliveries as f64) * 100.0
                } else {
                    0.0
                };

                performances.push(serde_json::json!({
                    "endpoint_id": endpoint_id_str,
                    "endpoint_name": endpoint.name,
                    "total_events": s.total_events,
                    "total_deliveries": s.total_deliveries,
                    "successful_deliveries": s.successful_deliveries,
                    "success_rate": success_rate,
                    "avg_delivery_time_ms": s.avg_delivery_time_ms,
                }));
            }
        }
    }

    Ok(Json(serde_json::json!({
        "endpoints": performances,
        "total": performances.len(),
    })))
}
