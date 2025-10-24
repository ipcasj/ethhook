use axum::Json;
use serde_json::{json, Value};

/// Health check endpoint for event-ingestor
pub async fn health_check() -> Json<Value> {
    // TODO: Add redis connectivity check if needed
    Json(json!({
        "status": "healthy",
        "service": "event-ingestor",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Readiness probe
pub async fn readiness_check() -> Json<Value> {
    Json(json!({
        "ready": true,
        "service": "event-ingestor",
    }))
}

/// Liveness probe
pub async fn liveness_check() -> Json<Value> {
    Json(json!({
        "alive": true,
        "service": "event-ingestor",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
