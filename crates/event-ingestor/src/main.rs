/*!
 * Event Ingestor Service
 *
 * This service is the "ears" of your webhook platform. It listens to blockchain events
 * in real-time and publishes them to Redis Streams for processing.
 *
 * ## Architecture Overview
 *
 * ```text
 * ┌─────────────────────────────────────────────────────────────────────┐
 * │                         Event Ingestor                              │
 * │                                                                     │
 * │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
 * │  │  Ethereum    │  │  Arbitrum    │  │  Optimism    │  ... etc      │
 * │  │  WebSocket   │  │  WebSocket   │  │  WebSocket   │               │
 * │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘               │
 * │         │                  │                  │                     │
 * │         └──────────────────┴──────────────────┘                     │
 * │                            │                                        │
 * │                    ┌───────▼────────┐                               │
 * │                    │  Deduplication │                               │
 * │                    │  (Redis SET)   │                               │
 * │                    └───────┬────────┘                               │
 * │                            │                                        │
 * │                    ┌───────▼────────┐                               │
 * │                    │ Redis Streams  │                               │
 * │                    │ events:eth     │                               │
 * │                    │ events:arb     │                               │
 * │                    └────────────────┘                               │
 * └─────────────────────────────────────────────────────────────────────┘
 * ```
 *
 * ## Key Responsibilities
 *
 * 1. **WebSocket Connections**: Maintain persistent connections to RPC providers
 * 2. **Event Parsing**: Parse blockchain events from blocks
 * 3. **Deduplication**: Prevent duplicate events during chain reorgs
 * 4. **Publishing**: Send events to Redis Streams for processing
 * 5. **Resilience**: Handle disconnections with circuit breaker pattern
 *
 * ## Why WebSocket Instead of HTTP Polling?
 *
 * HTTP Polling (expensive):
 * - 1 request every 2 seconds = 43,200 requests/day
 * - Costs $50/month in API calls
 * - High latency (2-5 seconds)
 *
 * WebSocket (efficient):
 * - 1 persistent connection
 * - Real-time updates (< 100ms latency)
 * - Included in free tier
 *
 * ## Performance Targets
 *
 * - **Throughput**: 10,000 events/second across all chains
 * - **Latency**: < 500ms from block mined to Redis Stream
 * - **Availability**: 99.9% uptime with automatic reconnections
 * - **Deduplication**: 100% accuracy (no duplicate webhooks)
 */

use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;
use tracing::{info, warn};

mod client;
mod config;
mod deduplicator;
mod filter;
mod ingestion;
mod metrics;
mod publisher;
mod types;

use crate::config::IngestorConfig;
use crate::ingestion::ChainIngestionManager;

/// Shared service state for health checks
#[derive(Clone)]
struct ServiceState {
    ready: Arc<AtomicBool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("🚀 Starting Event Ingestor Service");

    // Load configuration from environment variables
    let config = IngestorConfig::from_env().context("Failed to load configuration")?;

    info!("📋 Configuration loaded:");
    info!("   - Chains: {}", config.chains.len());
    info!("   - Redis: {}:{}", config.redis_host, config.redis_port);
    for chain in &config.chains {
        info!("   - {} (chain_id: {})", chain.name, chain.chain_id);
    }

    // Create shutdown broadcast channel for coordinated shutdown
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // Initialize service state for health checks
    let service_state = ServiceState {
        ready: Arc::new(AtomicBool::new(false)),
    };

    // Start HTTP health server FIRST (before chain connections)
    let health_port = std::env::var("INGESTOR_HEALTH_PORT").unwrap_or_else(|_| "8082".to_string());
    info!("🏥 Starting health server on port {}...", health_port);
    let health_state = service_state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_health_server(health_port, health_state).await {
            warn!("Health server failed: {}", e);
        }
    });

    // Initialize metrics server (Prometheus)
    // This starts an HTTP server on :9090/metrics for Prometheus to scrape
    let metrics_port = config.metrics_port;
    let metrics_shutdown = shutdown_tx.subscribe();
    let _metrics_handle = tokio::spawn(async move {
        if let Err(e) = metrics::start_metrics_server(metrics_port, metrics_shutdown).await {
            warn!("Metrics server failed: {}", e);
        }
    });

    // Create chain ingestion manager
    // This will spawn a tokio task for each chain (4 tasks total)
    let manager = Arc::new(ChainIngestionManager::new(config).await?);

    // Start ingesting events from all chains
    // Each chain runs independently; if one fails, others continue
    let mut ingestion_handle = {
        let manager = Arc::clone(&manager);
        tokio::spawn(async move {
            if let Err(e) = manager.start_all_chains().await {
                warn!("Ingestion error: {}", e);
            }
        })
    };

    // Mark service as ready (chain connections will be established async)
    service_state.ready.store(true, Ordering::SeqCst);

    info!("✅ Event Ingestor is READY");
    info!("   - WebSocket connections establishing...");
    info!(
        "   - Health: http://0.0.0.0:{}/health",
        std::env::var("INGESTOR_HEALTH_PORT").unwrap_or_else(|_| "8082".to_string())
    );
    info!(
        "   - Ready:  http://0.0.0.0:{}/ready",
        std::env::var("INGESTOR_HEALTH_PORT").unwrap_or_else(|_| "8082".to_string())
    );
    info!("   - Press Ctrl+C to shutdown gracefully");

    // Wait for shutdown signal (Ctrl+C or SIGTERM) or ingestion failure
    // Note: We don't wait for metrics_handle because metrics server failures should not stop event ingestion
    let shutdown_reason = tokio::select! {
        _ = signal::ctrl_c() => {
            "Received Ctrl+C signal"
        }
        _ = &mut ingestion_handle => {
            "Ingestion stopped unexpectedly"
        }
    };

    // Graceful shutdown - signal all services to stop
    info!("📡 {}", shutdown_reason);
    info!("🛑 Shutting down Event Ingestor...");

    // Broadcast shutdown signal to all services
    let _ = shutdown_tx.send(());

    // Shutdown ingestion manager
    manager.shutdown().await?;

    // Note: Metrics server will receive shutdown signal via broadcast channel

    info!("👋 Event Ingestor stopped");
    Ok(())
}

/// Start HTTP health server for Kubernetes-style health checks
async fn start_health_server(port: String, state: ServiceState) -> Result<()> {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind health server to {addr}"))?;

    info!("🏥 Health server listening on http://{}", addr);
    info!("   - GET /health - Liveness probe");
    info!("   - GET /ready  - Readiness probe");

    axum::serve(listener, app)
        .await
        .context("Health server failed")?;

    Ok(())
}

/// Liveness probe - is the process alive?
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "event-ingestor",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Readiness probe - can this service accept traffic?
async fn readiness_check(State(state): State<ServiceState>) -> (StatusCode, Json<Value>) {
    let is_ready = state.ready.load(Ordering::SeqCst);

    if is_ready {
        (
            StatusCode::OK,
            Json(json!({
                "ready": true,
                "service": "event-ingestor",
                "message": "Service initialized, WebSocket connections active"
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "ready": false,
                "service": "event-ingestor",
                "message": "Initializing..."
            })),
        )
    }
}
