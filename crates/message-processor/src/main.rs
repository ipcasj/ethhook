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
use tracing::{error, info};

mod config;
mod consumer;
mod matcher;
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
    let matcher = Arc::new(EndpointMatcher::new(db_pool));

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

    // Create shutdown channel
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

    // Spawn processing tasks for each chain
    let mut handles = vec![];
    for chain in &config.chains {
        let chain_config = chain.clone();
        let consumer = Arc::clone(&consumer);
        let matcher = Arc::clone(&matcher);
        let publisher = Arc::clone(&publisher);
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
async fn process_stream_loop(
    stream_name: &str,
    consumer: Arc<Mutex<StreamConsumer>>,
    matcher: Arc<EndpointMatcher>,
    publisher: Arc<Mutex<DeliveryPublisher>>,
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

        // Read events from stream
        let entries = {
            let mut consumer = consumer.lock().await;
            consumer
                .read_events(stream_name, batch_size, block_time_ms)
                .await
                .context("Failed to read events")?
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

        // Process each event
        let mut message_ids = Vec::new();
        for entry in &entries {
            message_ids.push(entry.id.clone());

            // Find matching endpoints
            let endpoints = matcher
                .find_matching_endpoints(&entry.event)
                .await
                .context("Failed to find matching endpoints")?;

            if endpoints.is_empty() {
                info!(
                    "[{}] No endpoints for event {} (contract: {})",
                    stream_name,
                    entry.event.transaction_hash,
                    &entry.event.contract_address[..10]
                );
                continue;
            }

            // Create delivery jobs
            let mut pub_client = publisher.lock().await;
            for endpoint in &endpoints {
                pub_client
                    .publish(endpoint, &entry.event)
                    .await
                    .context("Failed to publish delivery job")?;

                jobs_created += 1;
            }
            drop(pub_client);

            info!(
                "[{}] Created {} delivery jobs for event {}",
                stream_name,
                endpoints.len(),
                &entry.event.transaction_hash[..10]
            );
        }

        events_processed += entries.len() as u64;

        // Acknowledge processed messages
        {
            let mut consumer = consumer.lock().await;
            consumer
                .ack_messages(stream_name, &message_ids)
                .await
                .context("Failed to acknowledge messages")?;
        }

        info!(
            "[{}] Stats: {} events processed, {} jobs created",
            stream_name, events_processed, jobs_created
        );
    }

    Ok(())
}
