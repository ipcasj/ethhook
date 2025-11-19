use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Datelike, Utc};
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
    /// Optional: Filter by chain IDs (comma-separated)
    pub chain_ids: Option<String>,
    /// Optional: Filter by success status (true/false)
    pub success: Option<bool>,
    /// Optional: Custom start date (ISO 8601)
    pub start_date: Option<String>,
    /// Optional: Custom end date (ISO 8601)
    pub end_date: Option<String>,
}

fn default_time_range() -> String {
    "24h".to_string()
}

fn default_granularity() -> String {
    "hour".to_string()
}

/// Advanced filter query parameters for statistics
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct StatisticsFilterQuery {
    /// Filter by chain IDs (comma-separated, e.g., "1,137,42161")
    pub chain_ids: Option<String>,
    /// Filter by success status
    pub success: Option<bool>,
    /// Custom start date (ISO 8601)
    pub start_date: Option<String>,
    /// Custom end date (ISO 8601)
    pub end_date: Option<String>,
    /// Filter by minimum latency (ms)
    pub min_latency: Option<i32>,
    /// Filter by maximum latency (ms)
    pub max_latency: Option<i32>,
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

/// Helper function to parse chain IDs from comma-separated string
fn parse_chain_ids(chain_ids_str: Option<String>) -> Option<Vec<i32>> {
    chain_ids_str.and_then(|s| {
        let ids: Vec<i32> = s
            .split(',')
            .filter_map(|id| id.trim().parse::<i32>().ok())
            .collect();
        if ids.is_empty() { None } else { Some(ids) }
    })
}

/// Helper function to parse ISO 8601 date string
fn parse_date(date_str: Option<String>) -> Option<DateTime<Utc>> {
    date_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    })
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
    // Parse filter parameters
    let chain_ids = parse_chain_ids(params.chain_ids);
    let start_date = parse_date(params.start_date);
    let end_date = parse_date(params.end_date);

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

    // Build dynamic WHERE clause based on filters
    let mut where_conditions = vec![
        "a.user_id = $2".to_string(),
        "e.ingested_at >= NOW() - ($3 || ' hours')::interval".to_string(),
    ];

    if chain_ids.is_some() {
        where_conditions.push("ep.chain_id = ANY($4)".to_string());
    }

    if let Some(success_filter) = params.success {
        where_conditions.push(format!("da.success = {success_filter}"));
    }

    if start_date.is_some() && end_date.is_some() {
        where_conditions.push("e.ingested_at BETWEEN $5 AND $6".to_string());
    }

    let where_clause = where_conditions.join(" AND ");

    // Build dynamic query
    let query_str = format!(
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
            WHERE {where_clause}
            GROUP BY time_bucket
            ORDER BY time_bucket ASC
        )
        SELECT 
            time_bucket,
            event_count,
            delivery_count,
            successful_deliveries,
            failed_deliveries,
            avg_latency_ms
        FROM time_buckets
        "#
    );

    // Execute query with dynamic parameters
    let mut query = sqlx::query_as::<
        _,
        (
            chrono::DateTime<chrono::Utc>,
            i64,
            i64,
            i64,
            i64,
            Option<f64>,
        ),
    >(&query_str)
    .bind(trunc_format)
    .bind(auth_user.user_id)
    .bind(hours_back.to_string());

    if let Some(chain_ids_vec) = chain_ids {
        query = query.bind(chain_ids_vec);
    }

    if let (Some(start), Some(end)) = (start_date, end_date) {
        query = query.bind(start).bind(end);
    }

    let data_points = query.fetch_all(&pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch timeseries data: {e}"),
            }),
        )
    })?;

    let data_points: Vec<TimeseriesDataPoint> = data_points
        .into_iter()
        .map(
            |(
                timestamp,
                event_count,
                delivery_count,
                successful_deliveries,
                failed_deliveries,
                avg_latency_ms,
            )| {
                let success_rate = if delivery_count > 0 {
                    (successful_deliveries as f64 / delivery_count as f64) * 100.0
                } else {
                    0.0
                };

                TimeseriesDataPoint {
                    timestamp,
                    event_count,
                    delivery_count,
                    successful_deliveries,
                    failed_deliveries,
                    success_rate,
                    avg_latency_ms,
                }
            },
        )
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

// ============================================================================
// Application-Specific Statistics (Phase 2 Feature 1)
// ============================================================================

use axum::extract::Path;
use uuid::Uuid;

/// Application statistics response
#[derive(Debug, Serialize)]
pub struct ApplicationStatistics {
    pub application_id: Uuid,
    pub events_total: i64,
    pub events_24h: i64,
    pub endpoints_count: i64,
    pub active_endpoints: i64,
    pub total_deliveries: i64,
    pub successful_deliveries: i64,
    pub failed_deliveries: i64,
    pub success_rate: f64,
    pub avg_delivery_time_ms: Option<f64>,
    pub min_delivery_time_ms: Option<f64>,
    pub max_delivery_time_ms: Option<f64>,
    pub first_event_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
}

/// Get statistics for a specific application
pub async fn get_application_statistics(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApplicationStatistics>, (StatusCode, Json<ErrorResponse>)> {
    // Verify application belongs to user
    let app = sqlx::query!(
        r#"
        SELECT id FROM applications
        WHERE id = $1 AND user_id = $2
        "#,
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
    })?;

    if app.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Application not found".to_string(),
            }),
        ));
    }

    // Get events count (total and 24h)
    let events = sqlx::query!(
        r#"
        SELECT 
            COUNT(DISTINCT e.id) as "total!",
            COUNT(DISTINCT e.id) FILTER (WHERE e.ingested_at >= NOW() - INTERVAL '24 hours') as "events_24h!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        WHERE ep.application_id = $1
        "#,
        app_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch event counts: {e}"),
            }),
        )
    })?;

    // Get endpoints count
    let endpoints = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as "total!",
            COUNT(*) FILTER (WHERE is_active = true) as "active!"
        FROM endpoints
        WHERE application_id = $1
        "#,
        app_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch endpoint counts: {e}"),
            }),
        )
    })?;

    // Get delivery statistics
    let delivery_stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(*)::bigint as "total!",
            COUNT(*) FILTER (WHERE success = true)::bigint as "successful!",
            COUNT(*) FILTER (WHERE success = false)::bigint as "failed!",
            AVG(duration_ms)::float8 as "avg_duration",
            MIN(duration_ms)::float8 as "min_duration",
            MAX(duration_ms)::float8 as "max_duration"
        FROM delivery_attempts da
        JOIN endpoints ep ON da.endpoint_id = ep.id
        WHERE ep.application_id = $1
        "#,
        app_id
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

    // Get first and last event timestamps
    let event_times = sqlx::query!(
        r#"
        SELECT 
            MIN(e.ingested_at) as "first_event",
            MAX(e.ingested_at) as "last_event"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        WHERE ep.application_id = $1
        "#,
        app_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch event times: {e}"),
            }),
        )
    })?;

    // Calculate success rate
    let success_rate = if delivery_stats.total > 0 {
        (delivery_stats.successful as f64 / delivery_stats.total as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(ApplicationStatistics {
        application_id: app_id,
        events_total: events.total,
        events_24h: events.events_24h,
        endpoints_count: endpoints.total,
        active_endpoints: endpoints.active,
        total_deliveries: delivery_stats.total,
        successful_deliveries: delivery_stats.successful,
        failed_deliveries: delivery_stats.failed,
        success_rate,
        avg_delivery_time_ms: delivery_stats.avg_duration,
        min_delivery_time_ms: delivery_stats.min_duration,
        max_delivery_time_ms: delivery_stats.max_duration,
        first_event_at: event_times.first_event,
        last_event_at: event_times.last_event,
    }))
}

/// Get timeseries statistics for a specific application
pub async fn get_application_timeseries(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
    Query(params): Query<TimeseriesQuery>,
) -> Result<Json<TimeseriesResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify application belongs to user
    let app = sqlx::query!(
        r#"
        SELECT id FROM applications
        WHERE id = $1 AND user_id = $2
        "#,
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
    })?;

    if app.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Application not found".to_string(),
            }),
        ));
    }

    // Parse filter parameters
    let chain_ids = parse_chain_ids(params.chain_ids);
    let start_date = parse_date(params.start_date);
    let end_date = parse_date(params.end_date);

    // Parse time range
    let hours: i64 = match params.time_range.as_str() {
        "24h" => 24,
        "7d" => 168,
        "30d" => 720,
        _ => 24, // default to 24h
    };

    // Determine granularity
    let granularity = match params.granularity.as_str() {
        "hour" => "hour",
        "day" => "day",
        _ if hours <= 24 => "hour",
        _ => "day",
    };

    // Build dynamic WHERE clause
    let mut where_conditions = vec![
        "ep.application_id = $2".to_string(),
        "e.ingested_at >= NOW() - ($3 || ' hours')::interval".to_string(),
    ];

    if chain_ids.is_some() {
        where_conditions.push("ep.chain_id = ANY($4)".to_string());
    }

    if let Some(success_filter) = params.success {
        where_conditions.push(format!("da.success = {success_filter}"));
    }

    if start_date.is_some() && end_date.is_some() {
        where_conditions.push("e.ingested_at BETWEEN $5 AND $6".to_string());
    }

    let where_clause = where_conditions.join(" AND ");

    // Build dynamic query
    let query_str = format!(
        r#"
        SELECT 
            date_trunc($1, e.ingested_at) as bucket,
            COUNT(DISTINCT e.id)::bigint as event_count,
            COUNT(da.id)::bigint as delivery_count,
            COUNT(da.id) FILTER (WHERE da.success = true)::bigint as successful_deliveries,
            COUNT(da.id) FILTER (WHERE da.success = false)::bigint as failed_deliveries,
            AVG(da.duration_ms)::float8 as avg_latency_ms
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        JOIN endpoints ep ON da.endpoint_id = ep.id
        WHERE {where_clause}
        GROUP BY date_trunc($1, e.ingested_at)
        ORDER BY date_trunc($1, e.ingested_at) ASC
        "#
    );

    // Execute query with dynamic parameters
    let mut query = sqlx::query_as::<
        _,
        (
            chrono::DateTime<chrono::Utc>,
            i64,
            i64,
            i64,
            i64,
            Option<f64>,
        ),
    >(&query_str)
    .bind(granularity)
    .bind(app_id)
    .bind(hours.to_string());

    if let Some(chain_ids_vec) = chain_ids {
        query = query.bind(chain_ids_vec);
    }

    if let (Some(start), Some(end)) = (start_date, end_date) {
        query = query.bind(start).bind(end);
    }

    let data_points = query
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to fetch timeseries data: {e}"),
                }),
            )
        })?
        .into_iter()
        .map(
            |(
                bucket,
                event_count,
                delivery_count,
                successful_deliveries,
                failed_deliveries,
                avg_latency_ms,
            )| {
                let success_rate = if delivery_count > 0 {
                    (successful_deliveries as f64 / delivery_count as f64) * 100.0
                } else {
                    0.0
                };

                TimeseriesDataPoint {
                    timestamp: bucket,
                    event_count,
                    delivery_count,
                    successful_deliveries,
                    failed_deliveries,
                    success_rate,
                    avg_latency_ms,
                }
            },
        )
        .collect();

    Ok(Json(TimeseriesResponse {
        data_points,
        time_range: params.time_range.clone(),
        granularity: granularity.to_string(),
    }))
}

/// Endpoint performance summary
#[derive(Debug, Serialize)]
pub struct EndpointPerformance {
    pub endpoint_id: Uuid,
    pub name: String,
    pub url: String,
    pub events_count: i64,
    pub success_rate: f64,
    pub avg_latency_ms: Option<f64>,
    pub last_event_at: Option<DateTime<Utc>>,
}

/// Endpoint performance response
#[derive(Debug, Serialize)]
pub struct EndpointPerformanceResponse {
    pub endpoints: Vec<EndpointPerformance>,
}

/// Get endpoint performance for a specific application
pub async fn get_application_endpoints_performance(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    Path(app_id): Path<Uuid>,
) -> Result<Json<EndpointPerformanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify application belongs to user
    let app = sqlx::query!(
        r#"
        SELECT id FROM applications
        WHERE id = $1 AND user_id = $2
        "#,
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
    })?;

    if app.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Application not found".to_string(),
            }),
        ));
    }

    // Get endpoint performance data
    let endpoints = sqlx::query!(
        r#"
        SELECT 
            ep.id,
            ep.name,
            ep.webhook_url,
            COUNT(DISTINCT e.id)::bigint as "events_count!",
            COUNT(da.id)::bigint as "total_deliveries!",
            COUNT(da.id) FILTER (WHERE da.success = true)::bigint as "successful!",
            AVG(da.duration_ms)::float8 as "avg_latency",
            MAX(e.ingested_at) as "last_event"
        FROM endpoints ep
        LEFT JOIN delivery_attempts da ON ep.id = da.endpoint_id
        LEFT JOIN events e ON da.event_id = e.id
        WHERE ep.application_id = $1
        GROUP BY ep.id, ep.name, ep.webhook_url
        ORDER BY COUNT(DISTINCT e.id) DESC
        "#,
        app_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch endpoint performance: {e}"),
            }),
        )
    })?
    .into_iter()
    .map(|row| {
        let success_rate = if row.total_deliveries > 0 {
            (row.successful as f64 / row.total_deliveries as f64) * 100.0
        } else {
            0.0
        };

        EndpointPerformance {
            endpoint_id: row.id,
            name: row.name,
            url: row.webhook_url,
            events_count: row.events_count,
            success_rate,
            avg_latency_ms: row.avg_latency,
            last_event_at: row.last_event,
        }
    })
    .collect();

    Ok(Json(EndpointPerformanceResponse { endpoints }))
}

// ============================================================================
// Endpoints Analytics
// ============================================================================

/// Simple timeseries data point for endpoints
#[derive(Debug, Serialize)]
pub struct SimpleTimeseriesDataPoint {
    pub timestamp: DateTime<Utc>,
    pub count: i64,
}

/// Simple timeseries response for endpoints
#[derive(Debug, Serialize)]
pub struct SimpleTimeseriesResponse {
    pub data_points: Vec<SimpleTimeseriesDataPoint>,
    pub time_range: String,
    pub granularity: String,
}

/// Endpoint statistics response
#[derive(Debug, Serialize)]
pub struct EndpointStatistics {
    pub endpoint_id: uuid::Uuid,
    pub name: String,
    pub webhook_url: String,
    pub status: String,
    pub events_total: i64,
    pub events_24h: i64,
    pub deliveries_total: i64,
    pub successful_deliveries: i64,
    pub failed_deliveries: i64,
    pub success_rate: f64,
    pub avg_delivery_time_ms: Option<f64>,
    pub p50_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub p99_latency_ms: Option<f64>,
    pub health_score: f64,
    pub first_event_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
}

/// Delivery attempt response
#[derive(Debug, Serialize)]
pub struct DeliveryAttempt {
    pub id: uuid::Uuid,
    pub event_id: uuid::Uuid,
    pub attempt_number: i32,
    pub http_status_code: Option<i32>,
    pub success: bool,
    pub duration_ms: Option<i32>,
    pub attempted_at: DateTime<Utc>,
    pub error_message: Option<String>,
    pub response_body: Option<String>,
}

/// Paginated deliveries response
#[derive(Debug, Serialize)]
pub struct DeliveriesResponse {
    pub deliveries: Vec<DeliveryAttempt>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Query parameters for deliveries endpoint
#[derive(Debug, Deserialize)]
pub struct DeliveriesQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    #[serde(default = "default_status")]
    pub status: String, // "all", "success", "failed"
}

fn default_limit() -> i64 {
    50
}

fn default_status() -> String {
    "all".to_string()
}

/// Calculate health score for an endpoint
fn calculate_health_score(
    success_rate: f64,
    avg_latency_ms: Option<f64>,
    last_event_at: Option<DateTime<Utc>>,
) -> f64 {
    let mut score = 0.0;

    // Success rate component (60% of total)
    score += (success_rate / 100.0) * 60.0;

    // Latency component (30% of total)
    if let Some(latency) = avg_latency_ms {
        let latency_score = if latency < 100.0 {
            30.0
        } else if latency < 500.0 {
            20.0
        } else if latency < 1000.0 {
            10.0
        } else {
            0.0
        };
        score += latency_score;
    }

    // Uptime/activity component (10% of total)
    if let Some(last_event) = last_event_at {
        let hours_since = (Utc::now() - last_event).num_hours();
        let uptime_score = if hours_since < 24 {
            10.0
        } else if hours_since < 168 {
            5.0
        } else {
            0.0
        };
        score += uptime_score;
    }

    score.min(100.0)
}

/// Get endpoint statistics
pub async fn get_endpoint_statistics(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    axum::extract::Path(endpoint_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<EndpointStatistics>, (StatusCode, Json<ErrorResponse>)> {
    // First, verify the user owns the application this endpoint belongs to
    let endpoint = sqlx::query!(
        r#"
        SELECT ep.id, ep.name, ep.webhook_url, ep.health_status, a.user_id
        FROM endpoints ep
        JOIN applications a ON ep.application_id = a.id
        WHERE ep.id = $1
        "#,
        endpoint_id
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

    let endpoint = endpoint.ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Endpoint not found".to_string(),
        }),
    ))?;

    if endpoint.user_id != auth_user.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
            }),
        ));
    }

    // Get events count (total and 24h)
    let events_total = sqlx::query!(
        r#"
        SELECT COUNT(DISTINCT e.id) as "count!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        WHERE da.endpoint_id = $1
        "#,
        endpoint_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {e}"),
            }),
        )
    })?
    .count;

    let events_24h = sqlx::query!(
        r#"
        SELECT COUNT(DISTINCT e.id) as "count!"
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        WHERE da.endpoint_id = $1
          AND e.ingested_at >= NOW() - INTERVAL '24 hours'
        "#,
        endpoint_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {e}"),
            }),
        )
    })?
    .count;

    // Get delivery statistics with percentiles
    let delivery_stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as "total!",
            SUM(CASE WHEN success THEN 1 ELSE 0 END) as "successful!",
            SUM(CASE WHEN NOT success THEN 1 ELSE 0 END) as "failed!",
            AVG(duration_ms)::float8 as avg_duration,
            PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY duration_ms)::float8 as p50,
            PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms)::float8 as p95,
            PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY duration_ms)::float8 as p99
        FROM delivery_attempts
        WHERE endpoint_id = $1
        "#,
        endpoint_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {e}"),
            }),
        )
    })?;

    // Get first and last event timestamps
    let event_times = sqlx::query!(
        r#"
        SELECT 
            MIN(e.ingested_at) as first_event,
            MAX(e.ingested_at) as last_event
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        WHERE da.endpoint_id = $1
        "#,
        endpoint_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {e}"),
            }),
        )
    })?;

    let success_rate = if delivery_stats.total > 0 {
        (delivery_stats.successful as f64 / delivery_stats.total as f64) * 100.0
    } else {
        0.0
    };

    let health_score = calculate_health_score(
        success_rate,
        delivery_stats.avg_duration,
        event_times.last_event,
    );

    Ok(Json(EndpointStatistics {
        endpoint_id: endpoint.id,
        name: endpoint.name,
        webhook_url: endpoint.webhook_url,
        status: endpoint
            .health_status
            .unwrap_or_else(|| "unknown".to_string()),
        events_total,
        events_24h,
        deliveries_total: delivery_stats.total,
        successful_deliveries: delivery_stats.successful,
        failed_deliveries: delivery_stats.failed,
        success_rate,
        avg_delivery_time_ms: delivery_stats.avg_duration,
        p50_latency_ms: delivery_stats.p50,
        p95_latency_ms: delivery_stats.p95,
        p99_latency_ms: delivery_stats.p99,
        health_score,
        first_event_at: event_times.first_event,
        last_event_at: event_times.last_event,
    }))
}

/// Get endpoint timeseries data
pub async fn get_endpoint_timeseries(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    axum::extract::Path(endpoint_id): axum::extract::Path<uuid::Uuid>,
    Query(query): Query<TimeseriesQuery>,
) -> Result<Json<SimpleTimeseriesResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify the user owns the application this endpoint belongs to
    let endpoint = sqlx::query!(
        r#"
        SELECT ep.id, a.user_id
        FROM endpoints ep
        JOIN applications a ON ep.application_id = a.id
        WHERE ep.id = $1
        "#,
        endpoint_id
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

    let endpoint = endpoint.ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Endpoint not found".to_string(),
        }),
    ))?;

    if endpoint.user_id != auth_user.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
            }),
        ));
    }

    // Parse filter parameters
    let start_date = parse_date(query.start_date);
    let end_date = parse_date(query.end_date);

    // Parse time range
    let hours = match query.time_range.as_str() {
        "24h" => 24,
        "7d" => 168,
        "30d" => 720,
        _ => 24,
    };

    // Determine granularity
    let granularity = match query.granularity.as_str() {
        "hour" => "hour",
        "day" => "day",
        _ => {
            if hours <= 48 {
                "hour"
            } else {
                "day"
            }
        }
    };

    // Build dynamic WHERE clause
    let mut where_conditions = vec![
        "da.endpoint_id = $2".to_string(),
        "e.ingested_at >= NOW() - ($3 || ' hours')::INTERVAL".to_string(),
    ];

    if let Some(success_filter) = query.success {
        where_conditions.push(format!("da.success = {success_filter}"));
    }

    if start_date.is_some() && end_date.is_some() {
        where_conditions.push("e.ingested_at BETWEEN $4 AND $5".to_string());
    }

    let where_clause = where_conditions.join(" AND ");

    // Build dynamic query
    let query_str = format!(
        r#"
        SELECT 
            date_trunc($1, e.ingested_at) as bucket,
            COUNT(DISTINCT e.id) as count
        FROM events e
        JOIN delivery_attempts da ON e.id = da.event_id
        WHERE {where_clause}
        GROUP BY date_trunc($1, e.ingested_at)
        ORDER BY date_trunc($1, e.ingested_at) ASC
        "#
    );

    // Execute query with dynamic parameters
    let mut query_exec = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>, i64)>(&query_str)
        .bind(granularity)
        .bind(endpoint_id)
        .bind(hours.to_string());

    if let (Some(start), Some(end)) = (start_date, end_date) {
        query_exec = query_exec.bind(start).bind(end);
    }

    let rows = query_exec.fetch_all(&pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {e}"),
            }),
        )
    })?;

    let data_points: Vec<SimpleTimeseriesDataPoint> = rows
        .into_iter()
        .map(|(bucket, count)| SimpleTimeseriesDataPoint {
            timestamp: bucket,
            count,
        })
        .collect();

    Ok(Json(SimpleTimeseriesResponse {
        data_points,
        time_range: query.time_range,
        granularity: granularity.to_string(),
    }))
}

/// Get endpoint deliveries with pagination
pub async fn get_endpoint_deliveries(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
    axum::extract::Path(endpoint_id): axum::extract::Path<uuid::Uuid>,
    Query(query): Query<DeliveriesQuery>,
) -> Result<Json<DeliveriesResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify the user owns the application this endpoint belongs to
    let endpoint = sqlx::query!(
        r#"
        SELECT ep.id, a.user_id
        FROM endpoints ep
        JOIN applications a ON ep.application_id = a.id
        WHERE ep.id = $1
        "#,
        endpoint_id
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

    let endpoint = endpoint.ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Endpoint not found".to_string(),
        }),
    ))?;

    if endpoint.user_id != auth_user.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
            }),
        ));
    }

    // Limit max to 100
    let limit = query.limit.min(100);

    // Get total count
    let total = if query.status == "all" {
        sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM delivery_attempts
            WHERE endpoint_id = $1
            "#,
            endpoint_id
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {e}"),
                }),
            )
        })?
        .count
    } else {
        let success_filter = query.status == "success";
        sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM delivery_attempts
            WHERE endpoint_id = $1 AND success = $2
            "#,
            endpoint_id,
            success_filter
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {e}"),
                }),
            )
        })?
        .count
    };

    // Get deliveries with optional status filter
    let deliveries = if query.status == "all" {
        sqlx::query!(
            r#"
            SELECT 
                da.id,
                da.event_id,
                da.attempt_number,
                da.http_status_code,
                da.success,
                da.duration_ms,
                da.attempted_at,
                da.error_message,
                LEFT(da.response_body, 1000) as response_body
            FROM delivery_attempts da
            WHERE da.endpoint_id = $1
            ORDER BY da.attempted_at DESC
            LIMIT $2 OFFSET $3
            "#,
            endpoint_id,
            limit,
            query.offset
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {e}"),
                }),
            )
        })?
        .into_iter()
        .map(|row| DeliveryAttempt {
            id: row.id,
            event_id: row.event_id,
            attempt_number: row.attempt_number,
            http_status_code: row.http_status_code,
            success: row.success.unwrap_or(false),
            duration_ms: row.duration_ms,
            attempted_at: row.attempted_at.unwrap_or_else(Utc::now),
            error_message: row.error_message,
            response_body: row.response_body,
        })
        .collect()
    } else {
        let success_filter = query.status == "success";
        sqlx::query!(
            r#"
            SELECT 
                da.id,
                da.event_id,
                da.attempt_number,
                da.http_status_code,
                da.success,
                da.duration_ms,
                da.attempted_at,
                da.error_message,
                LEFT(da.response_body, 1000) as response_body
            FROM delivery_attempts da
            WHERE da.endpoint_id = $1 AND da.success = $2
            ORDER BY da.attempted_at DESC
            LIMIT $3 OFFSET $4
            "#,
            endpoint_id,
            success_filter,
            limit,
            query.offset
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {e}"),
                }),
            )
        })?
        .into_iter()
        .map(|row| DeliveryAttempt {
            id: row.id,
            event_id: row.event_id,
            attempt_number: row.attempt_number,
            http_status_code: row.http_status_code,
            success: row.success.unwrap_or(false),
            duration_ms: row.duration_ms,
            attempted_at: row.attempted_at.unwrap_or_else(Utc::now),
            error_message: row.error_message,
            response_body: row.response_body,
        })
        .collect()
    };

    Ok(Json(DeliveriesResponse {
        deliveries,
        total,
        limit,
        offset: query.offset,
    }))
}

/// Alchemy API usage statistics
#[derive(Debug, Serialize)]
pub struct AlchemyCUStats {
    /// Total CU consumed today
    pub cu_consumed_today: i64,

    /// Total CU consumed this month
    pub cu_consumed_month: i64,

    /// Current monthly limit (based on plan)
    pub monthly_limit: i64,

    /// Percentage used (0-100)
    pub usage_percent: f64,

    /// Estimated monthly burn rate (projected from current usage)
    pub estimated_monthly_burn: i64,

    /// Days until limit reached (at current burn rate)
    pub days_until_limit: Option<f64>,

    /// CU breakdown by operation type
    pub breakdown_by_operation: Vec<CUOperationBreakdown>,

    /// CU breakdown by chain
    pub breakdown_by_chain: Vec<CUChainBreakdown>,

    /// Alert level: "ok" | "warning" | "critical"
    pub alert_level: String,
}

#[derive(Debug, Serialize)]
pub struct CUOperationBreakdown {
    pub operation: String,
    pub cu_consumed: i64,
    pub api_calls: i64,
    pub avg_cu_per_call: f64,
}

#[derive(Debug, Serialize)]
pub struct CUChainBreakdown {
    pub chain_name: String,
    pub cu_consumed: i64,
    pub percentage: f64,
}

/// Get Alchemy API CU usage statistics
///
/// Queries Prometheus metrics from event-ingestor to track Alchemy costs.
/// Provides alerts when approaching usage limits.
///
/// **ADMIN ONLY** - Returns 403 Forbidden for non-admin users.
pub async fn get_alchemy_cu_stats(
    State(pool): State<PgPool>,
    auth_user: AuthUser,
) -> Result<Json<AlchemyCUStats>, (StatusCode, Json<ErrorResponse>)> {
    // Check admin access
    if !auth_user.is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Admin access required".to_string(),
            }),
        ));
    }

    // For MVP: Calculate estimated CU usage based on events processed
    // In production: Query actual Prometheus metrics from event-ingestor

    // Get events processed today
    let events_today = sqlx::query!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM events
        WHERE ingested_at >= NOW() - INTERVAL '24 hours'
        "#
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch events: {e}"),
            }),
        )
    })?
    .count;

    // Get events this month
    let events_month = sqlx::query!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM events
        WHERE ingested_at >= DATE_TRUNC('month', NOW())
        "#
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch monthly events: {e}"),
            }),
        )
    })?
    .count;

    // Get chain distribution (placeholder - events table doesn't have chain_name column)
    let _chain_stats: Vec<(String, i64)> = vec![];
    // TODO: Add chain_id column to events table or use a different approach for chain stats

    // Estimated CU consumption calculations
    // Based on architecture analysis:
    // - WebSocket subscription (newHeads): ~2,880 CUs/day (10 CU/block, 12s block time)
    // - eth_getLogs per event: ~75 CUs (with filtering, down from 750 CUs without)
    // - Overhead: ~20% buffer

    const CU_PER_SUBSCRIPTION_DAY: i64 = 2_880;
    const CU_PER_EVENT_FILTERED: i64 = 75;
    const OVERHEAD_MULTIPLIER: f64 = 1.2;

    let cu_subscription_today = CU_PER_SUBSCRIPTION_DAY;
    let cu_events_today = events_today * CU_PER_EVENT_FILTERED;
    let cu_consumed_today =
        ((cu_subscription_today + cu_events_today) as f64 * OVERHEAD_MULTIPLIER) as i64;

    let cu_subscription_month = CU_PER_SUBSCRIPTION_DAY * 30; // Approximate
    let cu_events_month = events_month * CU_PER_EVENT_FILTERED;
    let cu_consumed_month =
        ((cu_subscription_month + cu_events_month) as f64 * OVERHEAD_MULTIPLIER) as i64;

    // Calculate monthly limit based on plan (default: Growth plan = 300M CUs)
    let monthly_limit: i64 = 300_000_000;

    let usage_percent = (cu_consumed_month as f64 / monthly_limit as f64) * 100.0;

    // Estimate monthly burn rate
    let days_elapsed = chrono::Utc::now().day() as f64;
    let estimated_monthly_burn = if days_elapsed > 0.0 {
        (cu_consumed_month as f64 / days_elapsed * 30.0) as i64
    } else {
        cu_consumed_month
    };

    // Calculate days until limit
    let days_until_limit = if cu_consumed_today > 0 {
        let remaining = monthly_limit - cu_consumed_month;
        Some(remaining as f64 / cu_consumed_today as f64)
    } else {
        None
    };

    // Build breakdown by operation
    let total_ops = cu_subscription_today + cu_events_today;
    let breakdown_by_operation = vec![
        CUOperationBreakdown {
            operation: "newHeads_subscription".to_string(),
            cu_consumed: cu_subscription_today,
            api_calls: 7200, // ~12s blocks = 7200 blocks/day
            avg_cu_per_call: 10.0,
        },
        CUOperationBreakdown {
            operation: "eth_getLogs_filtered".to_string(),
            cu_consumed: cu_events_today,
            api_calls: events_today,
            avg_cu_per_call: CU_PER_EVENT_FILTERED as f64,
        },
        CUOperationBreakdown {
            operation: "overhead".to_string(),
            cu_consumed: ((total_ops as f64 * 0.2) as i64),
            api_calls: 0,
            avg_cu_per_call: 0.0,
        },
    ];

    // Build breakdown by chain (currently empty - needs chain_id in events table)
    let breakdown_by_chain: Vec<CUChainBreakdown> = vec![];

    // Determine alert level
    let alert_level = if usage_percent >= 85.0 {
        "critical".to_string()
    } else if usage_percent >= 70.0 {
        "warning".to_string()
    } else {
        "ok".to_string()
    };

    Ok(Json(AlchemyCUStats {
        cu_consumed_today,
        cu_consumed_month,
        monthly_limit,
        usage_percent,
        estimated_monthly_burn,
        days_until_limit,
        breakdown_by_operation,
        breakdown_by_chain,
        alert_level,
    }))
}
