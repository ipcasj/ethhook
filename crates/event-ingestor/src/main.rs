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
use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};

mod client;
mod config;
mod deduplicator;
mod ingestion;
mod metrics;
mod publisher;
mod types;

use crate::config::IngestorConfig;
use crate::ingestion::ChainIngestionManager;

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

    info!("✅ Event Ingestor is running");
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
