use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
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
