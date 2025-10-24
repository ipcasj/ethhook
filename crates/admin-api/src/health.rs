use axum::Json;
use serde_json::{json, Value};
use sqlx::PgPool;

/// Health check endpoint with database connectivity test
pub async fn health_check(pool: axum::extract::State<PgPool>) -> Json<Value> {
    // Test database connection
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_optional(&*pool)
        .await
        .is_ok();

    let status = if db_healthy { "healthy" } else { "degraded" };

    Json(json!({
        "status": status,
        "service": "admin-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "database": if db_healthy { "ok" } else { "failed" },
        }
    }))
}

/// Readiness probe - checks if service is ready to accept traffic
pub async fn readiness_check(pool: axum::extract::State<PgPool>) -> Json<Value> {
    let db_ready = sqlx::query("SELECT 1")
        .fetch_optional(&*pool)
        .await
        .is_ok();

    let ready = db_ready;

    Json(json!({
        "ready": ready,
        "service": "admin-api",
        "checks": {
            "database": db_ready,
        }
    }))
}

/// Liveness probe - checks if service is alive
pub async fn liveness_check() -> Json<Value> {
    Json(json!({
        "alive": true,
        "service": "admin-api",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
