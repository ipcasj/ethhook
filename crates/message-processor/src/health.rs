use axum::Json;
use serde_json::{json, Value};

/// Health check endpoint for message-processor
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "message-processor",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Readiness probe
pub async fn readiness_check() -> Json<Value> {
    Json(json!({
        "ready": true,
        "service": "message-processor",
    }))
}

/// Liveness probe
pub async fn liveness_check() -> Json<Value> {
    Json(json!({
        "alive": true,
        "service": "message-processor",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
