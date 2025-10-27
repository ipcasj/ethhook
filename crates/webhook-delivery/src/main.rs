/*!
 * Webhook Delivery Service
 *
 * Consumes delivery jobs from Redis Queue and sends webhooks to customer endpoints.
 *
 * ## Architecture
 *
 * ```text
 * Main Process
 *     │
 *     ├──> Worker Pool (50 tokio tasks)
 *          ├──> Worker 1: BRPOP → HTTP POST → Log Result
 *          ├──> Worker 2: BRPOP → HTTP POST → Log Result
 *          ├──> Worker 3: BRPOP → HTTP POST → Log Result
 *          ...
 *          └──> Worker 50: BRPOP → HTTP POST → Log Result
 * ```
 *
 * Each worker:
 * 1. BRPOP from delivery_queue (blocking, wait 5 seconds)
 * 2. Check circuit breaker (allow request?)
 * 3. Send HTTP POST with HMAC signature
 * 4. Log result to PostgreSQL
 * 5. Update circuit breaker state
 * 6. Retry if failed (up to max_retries with exponential backoff)
 *
 * ## Configuration
 *
 * Environment variables:
 * - DATABASE_URL: PostgreSQL connection URL
 * - REDIS_HOST: Redis hostname
 * - REDIS_PORT: Redis port
 * - WORKER_COUNT: Number of concurrent workers (default: 50)
 * - HTTP_TIMEOUT_SECS: Request timeout (default: 30)
 * - MAX_RETRIES: Maximum retry attempts (default: 5)
 * - CIRCUIT_BREAKER_THRESHOLD: Failures before circuit opens (default: 5)
 */

use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tracing::{debug, error, info, warn};

mod circuit_breaker;
mod config;
mod consumer;
mod delivery;
mod metrics;
mod retry;

use circuit_breaker::CircuitBreakerManager;
use config::DeliveryConfig;
use consumer::JobConsumer;
use delivery::WebhookDelivery;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    ethhook_common::init_tracing();

    info!("🚀 Starting Webhook Delivery Service");

    // Load configuration
    let config = DeliveryConfig::from_env().context("Failed to load configuration")?;

    info!("📋 Configuration loaded:");
    info!(
        "   - Database: {}",
        config.database_url.split('@').next_back().unwrap_or("***")
    );
    info!("   - Redis: {}:{}", config.redis_host, config.redis_port);
    info!("   - Queue: {}", config.queue_name);
    info!("   - Workers: {}", config.worker_count);
    info!("   - HTTP Timeout: {:?}", config.http_timeout);
    info!("   - Max Retries: {}", config.max_retries);
    info!(
        "   - Circuit Breaker: threshold={} timeout={}s",
        config.circuit_breaker_threshold, config.circuit_breaker_timeout_secs
    );

    // Create PostgreSQL connection pool
    info!("📦 Connecting to PostgreSQL...");
    let db_pool = ethhook_common::create_pool(&config.database_url, 20)
        .await
        .context("Failed to create database pool")?;
    info!("✅ PostgreSQL connected");

    // Create webhook delivery service (shared HTTP client)
    let webhook_delivery = Arc::new(
        WebhookDelivery::new(config.http_timeout)
            .context("Failed to create webhook delivery service")?,
    );

    // Create circuit breaker manager (shared across workers)
    let circuit_breaker = Arc::new(CircuitBreakerManager::new(
        config.circuit_breaker_threshold,
        Duration::from_secs(config.circuit_breaker_timeout_secs),
    ));

    // Start metrics server (configurable port via DELIVERY_METRICS_PORT env var)
    let metrics_port = std::env::var("DELIVERY_METRICS_PORT")
        .unwrap_or_else(|_| "9090".to_string());
    let metrics_addr = format!("0.0.0.0:{}", metrics_port);
    
    info!("📊 Starting metrics server on {}...", metrics_addr);
    let _metrics_handle = tokio::spawn(async move {
        let app = axum::Router::new().route("/metrics", axum::routing::get(metrics_handler));

        match tokio::net::TcpListener::bind(&metrics_addr).await {
            Ok(listener) => {
                info!("✅ Metrics server listening on {}", metrics_addr);
                if let Err(e) = axum::serve(listener, app).await {
                    warn!("⚠️  Metrics server error: {}", e);
                }
            }
            Err(e) => {
                warn!("⚠️  Failed to bind metrics server to {}: {}. Metrics will be unavailable.", metrics_addr, e);
            }
        }
    });

    // Create shutdown channel
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // Spawn worker pool
    let mut handles = vec![];
    for worker_id in 0..config.worker_count {
        let config = config.clone();
        let db_pool = db_pool.clone();
        let webhook_delivery = Arc::clone(&webhook_delivery);
        let circuit_breaker = Arc::clone(&circuit_breaker);
        let mut shutdown_rx = shutdown_tx.subscribe();

        let handle = tokio::spawn(async move {
            info!("[Worker {}] Starting", worker_id);

            // Each worker has its own Redis consumer
            let consumer_result = JobConsumer::new(&config.redis_url(), &config.queue_name).await;

            let mut consumer = match consumer_result {
                Ok(c) => c,
                Err(e) => {
                    error!("[Worker {}] Failed to create consumer: {}", worker_id, e);
                    return;
                }
            };

            let result = worker_loop(
                worker_id,
                &mut consumer,
                &db_pool,
                &webhook_delivery,
                &circuit_breaker,
                &config,
                &mut shutdown_rx,
            )
            .await;

            match result {
                Ok(_) => {
                    info!("[Worker {}] Stopped", worker_id);
                }
                Err(e) => {
                    error!("[Worker {}] Error: {}", worker_id, e);
                }
            }
        });

        handles.push(handle);
    }

    info!(
        "✅ Webhook Delivery is running ({} workers)",
        config.worker_count
    );
    info!("   - Press Ctrl+C to shutdown gracefully");

    // Signal readiness: Set a key in Redis to indicate all workers are ready
    // This allows orchestrators/tests to wait for confirmed readiness before sending work
    if let Ok(redis_client) = redis::Client::open(config.redis_url().as_str()) {
        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("SET")
                .arg("webhook_delivery:ready")
                .arg("true")
                .arg("EX")
                .arg(60) // Expire after 60 seconds (will be refreshed by health checks)
                .query_async(&mut conn)
                .await;
            info!("📡 Readiness signal published to Redis");
        }
    }

    // Wait for shutdown signal
    let shutdown_reason = tokio::select! {
        _ = signal::ctrl_c() => {
            "Received Ctrl+C signal"
        }
        _ = async {
            for handle in &mut handles {
                let _ = handle.await;
            }
        } => {
            "All workers stopped"
        }
    };

    info!("📡 {}", shutdown_reason);
    info!("🛑 Shutting down Webhook Delivery...");

    // Broadcast shutdown signal
    let _ = shutdown_tx.send(());

    // Wait for workers to finish
    let _ = tokio::time::timeout(Duration::from_secs(10), async {
        for handle in handles {
            let _ = handle.await;
        }
    })
    .await;

    info!("👋 Webhook Delivery stopped");
    Ok(())
}

/// Worker loop: consume jobs and deliver webhooks
async fn worker_loop(
    worker_id: usize,
    consumer: &mut JobConsumer,
    db_pool: &sqlx::PgPool,
    webhook_delivery: &WebhookDelivery,
    circuit_breaker: &CircuitBreakerManager,
    config: &DeliveryConfig,
    shutdown_rx: &mut tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    info!("[Worker {}] Entered worker_loop", worker_id);
    let mut jobs_processed = 0u64;

    loop {
        // Check for shutdown signal
        if shutdown_rx.try_recv().is_ok() {
            info!("[Worker {}] Shutdown signal received", worker_id);
            break;
        }

        debug!("[Worker {}] Waiting for jobs (BRPOP 5s)...", worker_id);

        // Consume next job (block for 5 seconds) with error recovery
        let job = match consumer.consume(5).await {
            Ok(Some(j)) => j,
            Ok(None) => {
                // Timeout - no jobs available
                continue;
            }
            Err(e) => {
                error!(
                    "[Worker {}] Failed to consume job: {:?}. Retrying in 1s...",
                    worker_id, e
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        jobs_processed += 1;

        // Record metrics: job consumed
        metrics::JOBS_CONSUMED_TOTAL
            .with_label_values(&[&worker_id.to_string()])
            .inc();

        // Check circuit breaker
        if !circuit_breaker.should_allow_request(job.endpoint_id).await {
            warn!(
                "[Worker {}] Circuit breaker OPEN for endpoint {} - skipping job",
                worker_id, job.endpoint_id
            );

            // Don't requeue - endpoint is unhealthy
            // Job will be dropped (in production, might want to save to DLQ)
            continue;
        }

        // Attempt delivery with retries
        let mut attempt = job.attempt;
        let max_attempts = job.max_retries as u32;
        let mut _last_result = None;

        while attempt <= max_attempts {
            // Deliver webhook with error recovery
            let result = match webhook_delivery.deliver(&job).await {
                Ok(r) => r,
                Err(e) => {
                    error!(
                        "[Worker {}] Failed to deliver webhook: {:?}. Continuing to next job...",
                        worker_id, e
                    );
                    // Record metrics: delivery attempt failed
                    metrics::DELIVERY_ATTEMPTS_TOTAL
                        .with_label_values(&[&job.endpoint_id.to_string(), "false"])
                        .inc();
                    // Break retry loop and move to next job
                    break;
                }
            };

            // Record metrics: delivery attempt
            metrics::DELIVERY_ATTEMPTS_TOTAL
                .with_label_values(&[&job.endpoint_id.to_string(), &result.success.to_string()])
                .inc();

            // Record metrics: delivery duration
            metrics::DELIVERY_DURATION
                .with_label_values(&[&job.endpoint_id.to_string()])
                .observe(result.duration_ms as f64 / 1000.0);

            // Record metrics: HTTP status code
            if let Some(status) = result.status_code {
                metrics::HTTP_RESPONSES_TOTAL
                    .with_label_values(&[&status.to_string()])
                    .inc();
            }

            // Log to database
            if let Err(e) = delivery::log_delivery_attempt(
                db_pool,
                job.endpoint_id,
                &job.event.transaction_hash,
                job.event.log_index,
                &result,
            )
            .await
            {
                error!(
                    "[Worker {}] Failed to log delivery attempt: {}",
                    worker_id, e
                );
            }

            // Update circuit breaker
            if result.success {
                circuit_breaker.record_success(job.endpoint_id).await;
                info!(
                    "[Worker {}] ✅ Job completed: endpoint={} attempt={}",
                    worker_id, job.endpoint_id, attempt
                );
                break; // Success - done
            } else {
                circuit_breaker.record_failure(job.endpoint_id).await;

                if result.should_retry && attempt < max_attempts {
                    // Record metrics: retry attempt
                    metrics::RETRY_ATTEMPTS_TOTAL
                        .with_label_values(&[&job.endpoint_id.to_string()])
                        .inc();

                    // Calculate backoff
                    let backoff =
                        retry::calculate_backoff(attempt - 1, config.retry_base_delay_secs, 60);

                    warn!(
                        "[Worker {}] ⏳ Retrying after {:?}: endpoint={} attempt={}/{}",
                        worker_id, backoff, job.endpoint_id, attempt, max_attempts
                    );

                    tokio::time::sleep(backoff).await;
                    attempt += 1;
                    _last_result = Some(result);
                } else {
                    // No more retries or non-retryable error
                    error!(
                        "[Worker {}] ❌ Job failed permanently: endpoint={} attempts={}",
                        worker_id, job.endpoint_id, attempt
                    );
                    break;
                }
            }
        }

        if jobs_processed.is_multiple_of(100) {
            info!("[Worker {}] Processed {} jobs", worker_id, jobs_processed);
        }
    }

    Ok(())
}

/// Metrics endpoint handler
async fn metrics_handler() -> Result<String, (axum::http::StatusCode, String)> {
    metrics::render_metrics()
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
