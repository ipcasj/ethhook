use axum::Json;
use serde_json::{json, Value};

/// Health check endpoint for webhook-delivery
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "webhook-delivery",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Readiness probe
pub async fn readiness_check() -> Json<Value> {
    Json(json!({
        "ready": true,
        "service": "webhook-delivery",
    }))
}

/// Liveness probe
pub async fn liveness_check() -> Json<Value> {
    Json(json!({
        "alive": true,
        "service": "webhook-delivery",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
