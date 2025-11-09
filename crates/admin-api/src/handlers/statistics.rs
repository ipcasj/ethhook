use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;

/// Dashboard statistics response
#[derive(Debug, Serialize)]
pub struct DashboardStatistics {
    pub events_today: i64,
    pub events_total: i64,
    pub success_rate: f64,
    pub avg_delivery_time_ms: Option<f64>,
    pub active_endpoints: i64,
    pub total_deliveries: i64,
    pub successful_deliveries: i64,
    pub failed_deliveries: i64,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Get dashboard statistics for the authenticated user
pub async fn get_dashboard_statistics(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
) -> Result<Json<DashboardStatistics>, (StatusCode, Json<ErrorResponse>)> {
    // Get events today count (last 24 hours)
    let events_today = sqlx::query!(
        r#"
        SELECT COUNT(DISTINCT e.id) as "count!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id
        WHERE a.user_id = $1
          AND e.ingested_at >= NOW() - INTERVAL '24 hours'
        "#,
        auth_user.user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch events today: {e}"),
            }),
        )
    })?
    .count;

    // Get total events count
    let events_total = sqlx::query!(
        r#"
        SELECT COUNT(DISTINCT e.id) as "count!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id
        WHERE a.user_id = $1
        "#,
        auth_user.user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch total events: {e}"),
            }),
        )
    })?
    .count;

    // Get delivery statistics
    let delivery_stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(*)::bigint as "total_deliveries!",
            COUNT(*) FILTER (WHERE success = true)::bigint as "successful_deliveries!",
            COUNT(*) FILTER (WHERE success = false)::bigint as "failed_deliveries!",
            AVG(duration_ms)::float8 as "avg_duration_ms"
        FROM delivery_attempts da
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id
        WHERE a.user_id = $1
        "#,
        auth_user.user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch delivery stats: {e}"),
            }),
        )
    })?;

    // Calculate success rate
    let success_rate = if delivery_stats.total_deliveries > 0 {
        (delivery_stats.successful_deliveries as f64 / delivery_stats.total_deliveries as f64)
            * 100.0
    } else {
        0.0
    };

    // Get active endpoints count
    let active_endpoints = sqlx::query!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM endpoints ep
        JOIN applications a ON ep.application_id = a.id
        WHERE a.user_id = $1 AND ep.is_active = true
        "#,
        auth_user.user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch active endpoints: {e}"),
            }),
        )
    })?
    .count;

    Ok(Json(DashboardStatistics {
        events_today,
        events_total,
        success_rate,
        avg_delivery_time_ms: delivery_stats.avg_duration_ms,
        active_endpoints,
        total_deliveries: delivery_stats.total_deliveries,
        successful_deliveries: delivery_stats.successful_deliveries,
        failed_deliveries: delivery_stats.failed_deliveries,
    }))
}

/// Query parameters for timeseries endpoint
#[derive(Debug, Deserialize)]
pub struct TimeseriesQuery {
    /// Time range: 24h, 7d, 30d
    #[serde(default = "default_time_range")]
    pub time_range: String,
    /// Granularity: hour, day
    #[serde(default = "default_granularity")]
    pub granularity: String,
}

fn default_time_range() -> String {
    "24h".to_string()
}

fn default_granularity() -> String {
    "hour".to_string()
}

/// Single data point in timeseries
#[derive(Debug, Serialize)]
pub struct TimeseriesDataPoint {
    pub timestamp: DateTime<Utc>,
    pub event_count: i64,
    pub delivery_count: i64,
    pub successful_deliveries: i64,
    pub failed_deliveries: i64,
    pub success_rate: f64,
    pub avg_latency_ms: Option<f64>,
}

/// Timeseries response
#[derive(Debug, Serialize)]
pub struct TimeseriesResponse {
    pub data_points: Vec<TimeseriesDataPoint>,
    pub time_range: String,
    pub granularity: String,
}

/// Chain distribution data point
#[derive(Debug, Serialize)]
pub struct ChainDistribution {
    pub chain_id: i32,
    pub chain_name: String,
    pub event_count: i64,
    pub percentage: f64,
}

/// Chain distribution response
#[derive(Debug, Serialize)]
pub struct ChainDistributionResponse {
    pub distributions: Vec<ChainDistribution>,
    pub total_events: i64,
}

/// Get timeseries statistics for charts
pub async fn get_timeseries_statistics(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Query(params): Query<TimeseriesQuery>,
) -> Result<Json<TimeseriesResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Parse time range
    let hours_back = match params.time_range.as_str() {
        "24h" => 24,
        "7d" => 24 * 7,
        "30d" => 24 * 30,
        _ => 24, // default to 24h
    };

    // Determine SQL interval format based on granularity
    let (trunc_format, _interval_minutes) = match params.granularity.as_str() {
        "hour" => ("hour", 60),
        "day" => ("day", 60 * 24),
        _ => ("hour", 60), // default to hour
    };

    // Query events and deliveries grouped by time bucket
    let data_points = sqlx::query!(
        r#"
        WITH time_buckets AS (
            SELECT 
                date_trunc($1, e.ingested_at) as time_bucket,
                COUNT(DISTINCT e.id)::bigint as event_count,
                COUNT(da.id)::bigint as delivery_count,
                COUNT(da.id) FILTER (WHERE da.success = true)::bigint as successful_deliveries,
                COUNT(da.id) FILTER (WHERE da.success = false)::bigint as failed_deliveries,
                CAST(AVG(da.duration_ms) FILTER (WHERE da.duration_ms IS NOT NULL) AS DOUBLE PRECISION) as avg_latency_ms
            FROM events e
            LEFT JOIN delivery_attempts da ON e.id = da.event_id
            JOIN endpoints ep ON da.endpoint_id = ep.id
            JOIN applications a ON ep.application_id = a.id
            WHERE a.user_id = $2
              AND e.ingested_at >= NOW() - ($3 || ' hours')::interval
            GROUP BY time_bucket
            ORDER BY time_bucket ASC
        )
        SELECT 
            time_bucket as "timestamp!",
            event_count as "event_count!",
            delivery_count as "delivery_count!",
            successful_deliveries as "successful_deliveries!",
            failed_deliveries as "failed_deliveries!",
            avg_latency_ms as "avg_latency_ms?"
        FROM time_buckets
        "#,
        trunc_format,
        auth_user.user_id,
        hours_back.to_string()
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch timeseries data: {e}"),
            }),
        )
    })?;

    let data_points: Vec<TimeseriesDataPoint> = data_points
        .into_iter()
        .map(|row| {
            let success_rate = if row.delivery_count > 0 {
                (row.successful_deliveries as f64 / row.delivery_count as f64) * 100.0
            } else {
                0.0
            };

            TimeseriesDataPoint {
                timestamp: row.timestamp,
                event_count: row.event_count,
                delivery_count: row.delivery_count,
                successful_deliveries: row.successful_deliveries,
                failed_deliveries: row.failed_deliveries,
                success_rate,
                avg_latency_ms: row.avg_latency_ms,
            }
        })
        .collect();

    Ok(Json(TimeseriesResponse {
        data_points,
        time_range: params.time_range,
        granularity: params.granularity,
    }))
}

/// Get chain distribution statistics for pie chart
pub async fn get_chain_distribution(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
) -> Result<Json<ChainDistributionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Map chain IDs to names
    let chain_names = |chain_id: i32| -> String {
        match chain_id {
            1 => "Ethereum".to_string(),
            11155111 => "Sepolia".to_string(),
            137 => "Polygon".to_string(),
            42161 => "Arbitrum".to_string(),
            10 => "Optimism".to_string(),
            8453 => "Base".to_string(),
            _ => format!("Chain {chain_id}"),
        }
    };

    // Since chain_id is stored in endpoints as an array, we need to unnest and count
    let distributions = sqlx::query!(
        r#"
        SELECT 
            chain_id as "chain_id!",
            COUNT(DISTINCT e.id)::bigint as "event_count!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        JOIN applications a ON ep.application_id = a.id,
        LATERAL unnest(ep.chain_ids) as chain_id
        WHERE a.user_id = $1
        GROUP BY chain_id
        ORDER BY COUNT(DISTINCT e.id) DESC
        "#,
        auth_user.user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch chain distribution: {e}"),
            }),
        )
    })?;

    let total_events: i64 = distributions.iter().map(|d| d.event_count).sum();

    let distributions: Vec<ChainDistribution> = distributions
        .into_iter()
        .map(|row| {
            let percentage = if total_events > 0 {
                (row.event_count as f64 / total_events as f64) * 100.0
            } else {
                0.0
            };

            ChainDistribution {
                chain_id: row.chain_id,
                chain_name: chain_names(row.chain_id),
                event_count: row.event_count,
                percentage,
            }
        })
        .collect();

    Ok(Json(ChainDistributionResponse {
        distributions,
        total_events,
    }))
}
