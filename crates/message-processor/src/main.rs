/*!
 * Message Processor Service
 *
 * Reads blockchain events from Redis Streams, queries database for matching
 * webhook endpoints, and publishes delivery jobs to Redis Queue.
 *
 * ## Service Flow
 *
 * ```text
 * 1. XREADGROUP from Redis Streams
 *    ├─> events:1 (Ethereum)
 *    ├─> events:42161 (Arbitrum)
 *    ├─> events:10 (Optimism)
 *    └─> events:8453 (Base)
 *
 * 2. For each event:
 *    ├─> Query PostgreSQL for matching endpoints
 *    ├─> Create delivery job for each endpoint
 *    └─> LPUSH to delivery_queue
 *
 * 3. XACK processed messages
 * ```
 *
 * ## Horizontal Scaling
 *
 * Multiple instances can run in parallel using consumer groups:
 * - Each instance gets different messages automatically
 * - Crash recovery via pending entry list (PEL)
 * - Load balancing built-in
 *
 * ## Configuration
 *
 * Environment variables:
 * - DATABASE_URL: PostgreSQL connection URL
 * - REDIS_HOST: Redis hostname
 * - REDIS_PORT: Redis port
 * - CONSUMER_GROUP: Consumer group name (default: "message_processors")
 * - CONSUMER_NAME: Consumer name (default: hostname)
 * - BATCH_SIZE: Events per XREAD (default: 100)
 * - BLOCK_TIME_MS: XREAD block time (default: 5000)
 */

use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

mod config;
mod consumer;
mod matcher;
mod metrics;
mod publisher;

use config::ProcessorConfig;
use consumer::StreamConsumer;
use matcher::EndpointMatcher;
use publisher::DeliveryPublisher;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    ethhook_common::init_tracing();

    info!("🚀 Starting Message Processor Service");

    // Load configuration
    let config = ProcessorConfig::from_env().context("Failed to load configuration")?;

    info!("📋 Configuration loaded:");
    info!(
        "   - Database: {}",
        config.database_url.split('@').next_back().unwrap_or("***")
    );
    info!("   - Redis: {}:{}", config.redis_host, config.redis_port);
    info!("   - Consumer Group: {}", config.consumer_group);
    info!("   - Consumer Name: {}", config.consumer_name);
    info!("   - Chains: {}", config.chains.len());
    info!("   - Batch Size: {}", config.batch_size);
    info!("   - Block Time: {}ms", config.block_time_ms);

    // Create PostgreSQL connection pool
    info!("📦 Connecting to PostgreSQL...");
    let db_pool = ethhook_common::create_pool(&config.database_url, 20)
        .await
        .context("Failed to create database pool")?;
    info!("✅ PostgreSQL connected");

    // Create endpoint matcher
    let matcher = Arc::new(EndpointMatcher::new(db_pool.clone()));

    // Share db_pool for event storage
    let db_pool = Arc::new(db_pool);

    // Create Redis Stream consumer
    info!("📡 Connecting to Redis Streams...");
    let consumer = Arc::new(Mutex::new(
        StreamConsumer::new(
            &config.redis_url(),
            &config.consumer_group,
            &config.consumer_name,
        )
        .await
        .context("Failed to create stream consumer")?,
    ));
    info!("✅ Redis Streams connected");

    // Create Redis Queue publisher
    info!("📤 Connecting to Redis Queue...");
    let publisher = Arc::new(Mutex::new(
        DeliveryPublisher::new(&config.redis_url(), "delivery_queue")
            .await
            .context("Failed to create delivery publisher")?,
    ));
    info!("✅ Redis Queue connected");

    // Ensure consumer groups exist for all streams
    info!("🔧 Ensuring consumer groups exist...");
    for chain in &config.chains {
        consumer
            .lock()
            .await
            .ensure_consumer_group(&chain.stream_name)
            .await
            .context("Failed to ensure consumer group")?;
    }
    info!("✅ Consumer groups ready");

    // Start metrics server on port 9090
    info!("📊 Starting metrics server on :9090...");
    let _metrics_handle = tokio::spawn(async move {
        let app = axum::Router::new().route("/metrics", axum::routing::get(metrics_handler));

        let addr = "0.0.0.0:9090";
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        info!("✅ Metrics server listening on {}", addr);

        axum::serve(listener, app).await.unwrap();
    });

    // Create shutdown channel
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // Spawn processing tasks for each chain
    let mut handles = vec![];
    for chain in &config.chains {
        let chain_config = chain.clone();

        // Create separate consumer for each stream to avoid mutex contention
        // Each stream needs its own consumer so XREADGROUP BLOCK doesn't block other streams
        let stream_consumer = StreamConsumer::new(
            &config.redis_url(),
            &config.consumer_group,
            &config.consumer_name,
        )
        .await
        .context(format!(
            "Failed to create consumer for stream {}",
            chain_config.stream_name
        ))?;
        let consumer = Arc::new(Mutex::new(stream_consumer));

        let matcher = Arc::clone(&matcher);
        let publisher = Arc::clone(&publisher);
        let pool = Arc::clone(&db_pool);
        let mut shutdown_rx = shutdown_tx.subscribe();
        let batch_size = config.batch_size;
        let block_time_ms = config.block_time_ms;

        let handle = tokio::spawn(async move {
            info!("[{}] Starting processing task", chain_config.stream_name);

            let result = process_stream_loop(
                &chain_config.stream_name,
                consumer,
                matcher,
                publisher,
                pool,
                batch_size,
                block_time_ms,
                &mut shutdown_rx,
            )
            .await;

            match result {
                Ok(_) => {
                    info!("[{}] Processing task stopped", chain_config.stream_name);
                }
                Err(e) => {
                    error!(
                        "[{}] Processing task failed: {}",
                        chain_config.stream_name, e
                    );
                }
            }
        });

        handles.push(handle);
    }

    info!("✅ Message Processor is running");
    info!("   - Press Ctrl+C to shutdown gracefully");

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
            "All processing tasks stopped"
        }
    };

    info!("📡 {}", shutdown_reason);
    info!("🛑 Shutting down Message Processor...");

    // Broadcast shutdown signal
    let _ = shutdown_tx.send(());

    // Wait for tasks to finish
    let _ = tokio::time::timeout(Duration::from_secs(10), async {
        for handle in handles {
            let _ = handle.await;
        }
    })
    .await;

    info!("👋 Message Processor stopped");
    Ok(())
}

/// Process events from a single stream
#[allow(clippy::too_many_arguments)]
async fn process_stream_loop(
    stream_name: &str,
    consumer: Arc<Mutex<StreamConsumer>>,
    matcher: Arc<EndpointMatcher>,
    publisher: Arc<Mutex<DeliveryPublisher>>,
    db_pool: Arc<sqlx::PgPool>,
    batch_size: usize,
    block_time_ms: usize,
    shutdown_rx: &mut tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    let mut events_processed = 0u64;
    let mut jobs_created = 0u64;

    loop {
        // Check for shutdown signal
        if shutdown_rx.try_recv().is_ok() {
            info!("[{}] Shutdown signal received", stream_name);
            break;
        }

        // Read events from stream with error recovery
        let entries = {
            let mut consumer = consumer.lock().await;
            match consumer
                .read_events(stream_name, batch_size, block_time_ms)
                .await
            {
                Ok(entries) => entries,
                Err(e) => {
                    error!(
                        "[{}] Failed to read events: {:?}. Retrying in 1s...",
                        stream_name, e
                    );
                    drop(consumer); // Release lock before sleeping
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue; // Retry instead of crashing
                }
            }
        };

        if entries.is_empty() {
            // No new events, continue waiting
            continue;
        }

        info!(
            "[{}] Processing {} events (total: {})",
            stream_name,
            entries.len(),
            events_processed + entries.len() as u64
        );

        // Record metrics: events consumed from stream
        metrics::EVENTS_CONSUMED_TOTAL
            .with_label_values(&[stream_name])
            .inc_by(entries.len() as u64);

        // Process each event
        let mut message_ids = Vec::new();
        for entry in &entries {
            message_ids.push(entry.id.clone());

            // Find matching endpoints with error recovery
            let endpoints = match matcher.find_matching_endpoints(&entry.event).await {
                Ok(eps) => eps,
                Err(e) => {
                    error!(
                        "[{}] Failed to find matching endpoints for event {}: {:?}",
                        stream_name, entry.event.transaction_hash, e
                    );
                    // Skip this event but continue processing others
                    continue;
                }
            };

            if endpoints.is_empty() {
                info!(
                    "[{}] No endpoints for event {} (contract: {})",
                    stream_name,
                    entry.event.transaction_hash,
                    &entry.event.contract_address[..10]
                );
                continue;
            }

            // Store event in database for dashboard and API queries
            match store_event_in_database(&db_pool, &entry.event).await {
                Ok(Some(event_id)) => {
                    debug!(
                        "[{}] Stored event {} in database with ID {}",
                        stream_name, entry.event.transaction_hash, event_id
                    );
                }
                Ok(None) => {
                    // Event already exists (duplicate), this is fine
                    debug!(
                        "[{}] Event {} already exists in database",
                        stream_name, entry.event.transaction_hash
                    );
                }
                Err(e) => {
                    // Log error but continue - event is already in Redis Stream
                    error!(
                        "[{}] Failed to store event {} in database: {:?}",
                        stream_name, entry.event.transaction_hash, e
                    );
                }
            }

            // Create delivery jobs with error recovery
            let mut pub_client = publisher.lock().await;
            for endpoint in &endpoints {
                match pub_client.publish(endpoint, &entry.event).await {
                    Ok(_) => {
                        jobs_created += 1;
                        // Record metrics: webhook published
                        metrics::WEBHOOKS_PUBLISHED_TOTAL
                            .with_label_values(&[&endpoint.endpoint_id.to_string()])
                            .inc();
                    }
                    Err(e) => {
                        error!(
                            "[{}] Failed to publish delivery job for endpoint {}: {:?}",
                            stream_name, endpoint.endpoint_id, e
                        );
                        // Record error metric
                        metrics::PROCESSING_ERRORS_TOTAL
                            .with_label_values(&["publish_failed"])
                            .inc();
                        // Continue with other endpoints
                    }
                }
            }
            drop(pub_client);

            // Record metrics: event processed successfully
            metrics::EVENTS_PROCESSED_TOTAL
                .with_label_values(&[stream_name])
                .inc();

            info!(
                "[{}] Created {} delivery jobs for event {}",
                stream_name,
                endpoints.len(),
                &entry.event.transaction_hash[..10]
            );
        }

        events_processed += entries.len() as u64;

        info!(
            "[{}] Preparing to ACK {} message IDs",
            stream_name,
            message_ids.len()
        );

        // Acknowledge processed messages with error recovery
        {
            let mut consumer = consumer.lock().await;
            if let Err(e) = consumer.ack_messages(stream_name, &message_ids).await {
                error!(
                    "[{}] Failed to acknowledge {} messages: {:?}. Messages may be reprocessed.",
                    stream_name,
                    message_ids.len(),
                    e
                );
                // Continue - messages will remain in pending list and be retried
            }
        }

        info!(
            "[{}] Stats: {} events processed, {} jobs created",
            stream_name, events_processed, jobs_created
        );
    }

    Ok(())
}

/// Store event in database for API queries and dashboard statistics
///
/// This inserts the event into the `events` table so it can be:
/// 1. Queried via GET /api/v1/events
/// 2. Counted in dashboard statistics
/// 3. Linked to delivery_attempts for tracking
///
/// Uses ON CONFLICT DO NOTHING to handle duplicates gracefully.
async fn store_event_in_database(
    pool: &sqlx::PgPool,
    event: &consumer::StreamEvent,
) -> Result<Option<uuid::Uuid>> {
    let event_id = sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO events (
            block_number,
            block_hash,
            transaction_hash,
            log_index,
            contract_address,
            topics,
            data
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (transaction_hash, log_index) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(event.block_number as i64)
    .bind(&event.block_hash)
    .bind(&event.transaction_hash)
    .bind(event.log_index as i32)
    .bind(&event.contract_address)
    .bind(&event.topics)
    .bind(&event.data)
    .fetch_optional(pool)
    .await
    .context("Failed to insert event into database")?;

    Ok(event_id)
}

/// Metrics endpoint handler
async fn metrics_handler() -> Result<String, (axum::http::StatusCode, String)> {
    metrics::render_metrics()
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
