/*!
 * Metrics Module
 *
 * Prometheus metrics for monitoring Event Ingestor performance.
 *
 * ## Available Metrics
 *
 * - `events_received_total{chain}` - Total events received from blockchain
 * - `events_published_total{chain}` - Total events published to Redis Stream
 * - `events_deduplicated_total{chain}` - Total duplicate events skipped
 * - `websocket_reconnects_total{chain}` - Total WebSocket reconnections
 * - `circuit_breaker_state{chain,state}` - Current circuit breaker state (0/1)
 * - `event_processing_errors_total{chain,error_type}` - Errors during processing
 *
 * ## Endpoints
 *
 * - `GET /metrics` - Prometheus exposition format
 * - `GET /health` - Health check (returns 200 OK)
 */

use anyhow::{Context, Result};
use prometheus::{
    register_int_counter_vec, register_int_gauge_vec, Encoder, IntCounterVec, IntGaugeVec,
    TextEncoder,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};

lazy_static::lazy_static! {
    /// Events received from blockchain by chain
    pub static ref EVENTS_RECEIVED: IntCounterVec = register_int_counter_vec!(
        "events_received_total",
        "Total number of events received from blockchain",
        &["chain"]
    )
    .expect("Failed to register events_received_total metric");

    /// Events successfully published to Redis Stream by chain
    pub static ref EVENTS_PUBLISHED: IntCounterVec = register_int_counter_vec!(
        "events_published_total",
        "Total number of events published to Redis Stream",
        &["chain"]
    )
    .expect("Failed to register events_published_total metric");

    /// Duplicate events filtered out by chain
    pub static ref EVENTS_DEDUPLICATED: IntCounterVec = register_int_counter_vec!(
        "events_deduplicated_total",
        "Total number of duplicate events skipped",
        &["chain"]
    )
    .expect("Failed to register events_deduplicated_total metric");

    /// WebSocket reconnections by chain
    pub static ref WEBSOCKET_RECONNECTS: IntCounterVec = register_int_counter_vec!(
        "websocket_reconnects_total",
        "Total number of WebSocket reconnections",
        &["chain"]
    )
    .expect("Failed to register websocket_reconnects_total metric");

    /// Circuit breaker state (0 = Closed, 1 = Open, 2 = Half-Open)
    pub static ref CIRCUIT_BREAKER_STATE: IntGaugeVec = register_int_gauge_vec!(
        "circuit_breaker_state",
        "Current circuit breaker state (0=Closed, 1=Open, 2=Half-Open)",
        &["chain"]
    )
    .expect("Failed to register circuit_breaker_state metric");

    /// Event processing errors by chain and error type
    pub static ref PROCESSING_ERRORS: IntCounterVec = register_int_counter_vec!(
        "event_processing_errors_total",
        "Total number of errors during event processing",
        &["chain", "error_type"]
    )
    .expect("Failed to register event_processing_errors_total metric");

    /// Consecutive failures count by chain
    pub static ref CONSECUTIVE_FAILURES: IntGaugeVec = register_int_gauge_vec!(
        "consecutive_failures",
        "Number of consecutive failures for a chain",
        &["chain"]
    )
    .expect("Failed to register consecutive_failures metric");
}

/// Start Prometheus metrics HTTP server with production-grade patterns
///
/// Modern implementation with:
/// - Bounded concurrency (max 100 concurrent connections)
/// - Graceful shutdown with tokio::select! (event-driven, not polling)
/// - Connection timeouts (30 seconds)
/// - Structured concurrency with JoinSet
/// - Proper resource cleanup
///
/// ## Why `loop {}` instead of threads?
///
/// This uses an async event loop, NOT a thread:
/// - Runs on tokio's thread pool (shared with other tasks)
/// - Yields to other tasks when waiting (tokio::select! is cooperative)
/// - Zero CPU usage when idle (unlike threads)
/// - Single task can handle thousands of connections
///
/// ## Endpoints
///
/// - `GET /metrics` - Prometheus metrics
/// - `GET /health` - Health check
pub async fn start_metrics_server(
    port: u16,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind metrics server to {addr}"))?;

    info!("Metrics server listening on http://{}", addr);
    info!("Available endpoints:");
    info!("  - GET http://{}/metrics - Prometheus metrics", addr);
    info!("  - GET http://{}/health  - Health check", addr);

    const MAX_CONCURRENT_CONNECTIONS: usize = 100;
    const CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

    let mut tasks = tokio::task::JoinSet::new();
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_CONNECTIONS));

    loop {
        // Clean up completed tasks to prevent unbounded growth
        while tasks.try_join_next().is_some() {}

        tokio::select! {
            // Accept new connections (event-driven, yields when no connections)
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, peer_addr)) => {
                        // Check if we have capacity
                        if let Ok(permit) = semaphore.clone().try_acquire_owned() {
                            tasks.spawn(async move {
                                // Permit is automatically released when dropped
                                let _permit = permit;

                                // Apply connection timeout
                                match tokio::time::timeout(
                                    CONNECTION_TIMEOUT,
                                    handle_connection(stream)
                                ).await {
                                    Ok(Ok(())) => {
                                        // Connection handled successfully
                                    }
                                    Ok(Err(e)) => {
                                        error!("Error handling metrics request from {}: {}", peer_addr, e);
                                    }
                                    Err(_) => {
                                        error!("Metrics request from {} timed out after {:?}", peer_addr, CONNECTION_TIMEOUT);
                                    }
                                }
                            });
                        } else {
                            // Too many concurrent connections - reject
                            info!("Rejecting connection from {} - at capacity ({} concurrent connections)",
                                  peer_addr, MAX_CONCURRENT_CONNECTIONS);
                            // Connection is dropped, client will see connection closed
                        }
                    }
                    Err(e) => {
                        error!("Failed to accept metrics connection: {}", e);
                        // Brief pause to avoid tight loop on persistent errors
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }

            // Graceful shutdown (event-driven, not polling!)
            _ = shutdown.recv() => {
                info!("Metrics server received shutdown signal");
                break; // Exit loop gracefully
            }
        }
    }

    // Wait for all in-flight requests to complete (with timeout)
    info!(
        "Waiting for {} in-flight metrics requests to complete...",
        tasks.len()
    );

    let shutdown_timeout = std::time::Duration::from_secs(5);
    let shutdown_deadline = tokio::time::Instant::now() + shutdown_timeout;

    while !tasks.is_empty() {
        tokio::select! {
            _ = tokio::time::sleep_until(shutdown_deadline) => {
                let remaining = tasks.len();
                if remaining > 0 {
                    info!("Forcefully terminating {} remaining metrics requests after {:?}",
                          remaining, shutdown_timeout);
                }
                break;
            }
            _ = tasks.join_next() => {
                // Task completed, continue waiting for others
            }
        }
    }

    info!("Metrics server shut down gracefully");
    Ok(())
}

/// Handle HTTP request for metrics or health endpoint
async fn handle_connection(mut stream: tokio::net::TcpStream) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    // Parse request path
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    match path {
        "/metrics" => {
            // Gather metrics
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            encoder
                .encode(&metric_families, &mut buffer)
                .context("Failed to encode metrics")?;

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\n\r\n{}",
                buffer.len(),
                String::from_utf8_lossy(&buffer)
            );

            stream.write_all(response.as_bytes()).await?;
        }
        "/health" => {
            let response =
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK";
            stream.write_all(response.as_bytes()).await?;
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
            stream.write_all(response.as_bytes()).await?;
        }
    }

    stream.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registered() {
        // Verify metrics are registered and can be accessed without panicking
        // Counters are always >= 0 by definition (unsigned), just verify they exist
        let _ = EVENTS_RECEIVED.with_label_values(&["ethereum"]).get();
        let _ = EVENTS_PUBLISHED.with_label_values(&["ethereum"]).get();
        let _ = EVENTS_DEDUPLICATED.with_label_values(&["ethereum"]).get();
        let _ = WEBSOCKET_RECONNECTS.with_label_values(&["ethereum"]).get();

        // If we get here without panicking, metrics are properly registered
        // Test passes if no panic occurs
    }

    #[test]
    fn test_metrics_increment() {
        // Test incrementing counters
        EVENTS_RECEIVED.with_label_values(&["test_chain"]).inc();
        EVENTS_PUBLISHED.with_label_values(&["test_chain"]).inc();

        assert!(EVENTS_RECEIVED.with_label_values(&["test_chain"]).get() > 0);
        assert!(EVENTS_PUBLISHED.with_label_values(&["test_chain"]).get() > 0);
    }
}
