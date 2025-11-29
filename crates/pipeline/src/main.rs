use anyhow::Result;
use ethhook_domain::{delivery::DeliveryJob, event::BlockchainEvent};
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod batch;
mod config_db;
mod health;
mod metrics;
mod websocket;

/// Channel buffer sizes (tuned for throughput vs memory)
const EVENT_CHANNEL_SIZE: usize = 10_000;
const DELIVERY_CHANNEL_SIZE: usize = 50_000;
const SHUTDOWN_CHANNEL_SIZE: usize = 16;

/// Unified pipeline service - replaces event-ingestor, message-processor, webhook-delivery
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pipeline=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("Starting unified pipeline service");
    info!("Tokio version: {}", env!("CARGO_PKG_VERSION"));

    // Safety Rule #1: Graceful shutdown handling
    // Set up Ctrl-C and SIGTERM handling
    let (shutdown_tx, _) = broadcast::channel::<()>(SHUTDOWN_CHANNEL_SIZE);
    let shutdown_tx_clone = shutdown_tx.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");

        warn!("Received Ctrl-C, initiating graceful shutdown");

        // Broadcast shutdown to all workers
        if let Err(e) = shutdown_tx_clone.send(()) {
            error!("Failed to send shutdown signal: {}", e);
        }
    });

    // Safety Rule #2: Runtime health monitoring
    // Spawn health check task that monitors for deadlocks
    let health_monitor = tokio::spawn(health::monitor_runtime_health(shutdown_tx.subscribe()));

    // Safety Rule #3: Message passing over shared state
    // Create channels for event flow (no shared mutexes across await)
    let (event_tx, event_rx) = mpsc::channel::<BlockchainEvent>(EVENT_CHANNEL_SIZE);
    let (_delivery_tx, _delivery_rx) = mpsc::channel::<DeliveryJob>(DELIVERY_CHANNEL_SIZE);

    info!(
        "Initialized channels: events={}, deliveries={}",
        EVENT_CHANNEL_SIZE, DELIVERY_CHANNEL_SIZE
    );

    // Initialize config database and endpoint cache
    let db_path =
        std::env::var("CONFIG_DB_PATH").unwrap_or_else(|_| "sqlite:config.db".to_string());

    config_db::init_config_db(&db_path, shutdown_tx.subscribe())
        .await
        .expect("Failed to initialize config database");

    // Start WebSocket ingestor (Days 3-4)
    let ws_handle = tokio::spawn(websocket::start_ingestor(
        event_tx.clone(),
        shutdown_tx.subscribe(),
    ));

    // Start batch processor (Days 5-7) - CRITICAL for performance
    let batch_handle = tokio::spawn(batch::start_processor(
        event_rx,
        shutdown_tx.subscribe(),
    ));

    // TODO: Start HTTP delivery workers (Days 8-10)
    // let delivery_handle = tokio::spawn(delivery::start_workers(delivery_rx, shutdown_tx.subscribe()));

    // TODO: Start metrics/admin HTTP server (Day 12)
    // let admin_handle = tokio::spawn(admin::start_server(shutdown_tx.subscribe()));

    // Wait for shutdown signal
    info!("Pipeline service running, waiting for shutdown signal");
    let mut shutdown_rx = shutdown_tx.subscribe();

    // Safety Rule #4: Timeout protection
    // Even shutdown should have timeout to prevent hanging forever
    match tokio::time::timeout(std::time::Duration::from_secs(5), shutdown_rx.recv()).await {
        Ok(Ok(_)) => {
            info!("Shutdown signal received, cleaning up");
        }
        Ok(Err(e)) => {
            error!("Shutdown channel error: {}", e);
        }
        Err(_) => {
            warn!("Initial wait completed, service still running");
        }
    }

    // Safety Rule #5: Graceful degradation
    // Give workers time to finish in-flight work
    info!("Waiting for workers to finish (max 30s)");
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    // Wait for tasks to complete
    if let Err(e) = health_monitor.await {
        error!("Health monitor task failed: {}", e);
    }

    if let Err(e) = ws_handle.await {
        error!("WebSocket ingestor task failed: {}", e);
    }

    if let Err(e) = batch_handle.await {
        error!("Batch processor task failed: {}", e);
    }

    info!("Pipeline service shutdown complete");
    Ok(())
}
