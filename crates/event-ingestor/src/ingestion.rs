/*!
 * Chain Ingestion Manager
 * 
 * Coordinates event ingestion across multiple blockchains.
 * 
 * ## Architecture
 * 
 * ```text
 * ChainIngestionManager
 *         │
 *         ├──> tokio::spawn(ingest_chain(Ethereum))   [Task 1]
 *         ├──> tokio::spawn(ingest_chain(Arbitrum))   [Task 2]
 *         ├──> tokio::spawn(ingest_chain(Optimism))   [Task 3]
 *         └──> tokio::spawn(ingest_chain(Base))       [Task 4]
 * 
 * Each task runs independently:
 * - If Ethereum fails, Arbitrum/Optimism/Base continue
 * - Each task has its own WebSocket connection
 * - Each task auto-reconnects on failure
 * ```
 */

use anyhow::{anyhow, Context, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinSet;
use tracing::{debug, error, info, warn};
use rand::Rng;

use crate::client::WebSocketClient;
use crate::config::{ChainConfig, IngestorConfig};
use crate::deduplicator::Deduplicator;
use crate::publisher::StreamPublisher;

/// Circuit breaker states for connection management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    /// Normal operation - connection healthy
    Closed,
    /// Too many failures - waiting before retry
    Open,
    /// Testing if connection recovered
    HalfOpen,
}

/// Health tracking for a chain connection
#[derive(Debug, Clone)]
struct ChainHealth {
    /// Last time an event was successfully processed
    last_event_time: Instant,
    /// Number of consecutive failures
    consecutive_failures: u32,
    /// Current circuit breaker state
    circuit_state: CircuitState,
    /// Time when circuit opened (for exponential backoff calculation)
    circuit_opened_at: Option<Instant>,
}

impl ChainHealth {
    fn new() -> Self {
        Self {
            last_event_time: Instant::now(),
            consecutive_failures: 0,
            circuit_state: CircuitState::Closed,
            circuit_opened_at: None,
        }
    }

    /// Record successful event processing
    fn record_success(&mut self) {
        self.last_event_time = Instant::now();
        self.consecutive_failures = 0;
        self.circuit_state = CircuitState::Closed;
        self.circuit_opened_at = None;
    }

    /// Record a failure and potentially open circuit
    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        
        // Open circuit after 3 consecutive failures
        if self.consecutive_failures >= 3 && self.circuit_state == CircuitState::Closed {
            self.circuit_state = CircuitState::Open;
            self.circuit_opened_at = Some(Instant::now());
        }
    }

    /// Calculate backoff duration with exponential backoff and jitter
    /// 
    /// Uses the formula: min(base * 2^attempt, max) + jitter
    /// Jitter prevents thundering herd problem
    fn calculate_backoff(&self, base_delay: u64, max_delay: u64) -> Duration {
        let attempt = self.consecutive_failures.min(10); // Cap at 2^10 = 1024x
        
        // Exponential backoff: base * 2^attempt
        let exponential_delay = base_delay.saturating_mul(2u64.saturating_pow(attempt));
        let capped_delay = exponential_delay.min(max_delay);
        
        // Add jitter: ±20% randomness
        let mut rng = rand::thread_rng();
        let jitter_factor = rng.gen_range(0.8..1.2);
        let delay_with_jitter = (capped_delay as f64 * jitter_factor) as u64;
        
        Duration::from_secs(delay_with_jitter)
    }

    /// Check if enough time has passed to attempt reconnection
    fn should_attempt_reconnect(&mut self, base_delay: u64, max_delay: u64) -> bool {
        match self.circuit_state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(opened_at) = self.circuit_opened_at {
                    let backoff = self.calculate_backoff(base_delay, max_delay);
                    if opened_at.elapsed() >= backoff {
                        // Transition to half-open to test connection
                        self.circuit_state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Get time since last successful event
    fn time_since_last_event(&self) -> Duration {
        self.last_event_time.elapsed()
    }
}

/// Manages ingestion across multiple chains
pub struct ChainIngestionManager {
    config: IngestorConfig,
    shutdown_tx: broadcast::Sender<()>,
    deduplicator: Arc<Mutex<Deduplicator>>,
    publisher: Arc<Mutex<StreamPublisher>>,
}

impl ChainIngestionManager {
    /// Create new chain ingestion manager
    pub async fn new(config: IngestorConfig) -> Result<Self> {
        let (shutdown_tx, _) = broadcast::channel(1);

        // Initialize Redis deduplicator
        let deduplicator = Deduplicator::new(
            &config.redis_url(),
            config.dedup_ttl_seconds,
        )
        .await
        .context("Failed to initialize Redis deduplicator")?;

        // Initialize Redis Stream publisher
        let publisher = StreamPublisher::new(&config.redis_url())
            .await
            .context("Failed to initialize Redis Stream publisher")?;

        Ok(Self {
            config,
            shutdown_tx,
            deduplicator: Arc::new(Mutex::new(deduplicator)),
            publisher: Arc::new(Mutex::new(publisher)),
        })
    }

    /// Start ingesting events from all configured chains
    /// 
    /// Uses JoinSet for structured concurrency - better than loose vec of handles.
    /// Each chain runs independently with circuit breaker and health tracking.
    pub async fn start_all_chains(&self) -> Result<()> {
        let mut join_set = JoinSet::new();

        for chain in &self.config.chains {
            let chain_config = chain.clone();
            let shutdown_rx = self.shutdown_tx.subscribe();
            let deduplicator = Arc::clone(&self.deduplicator);
            let publisher = Arc::clone(&self.publisher);

            // Spawn independent task for this chain
            join_set.spawn(async move {
                info!(
                    "[{}] Starting chain ingestion task",
                    chain_config.name
                );

                let mut health = ChainHealth::new();
                let base_delay = chain_config.reconnect_delay_secs;
                let max_delay = 60; // Max 60 seconds between retries

                Self::ingest_chain_with_circuit_breaker(
                    &chain_config,
                    &deduplicator,
                    &publisher,
                    &mut health,
                    shutdown_rx,
                    base_delay,
                    max_delay,
                )
                .await;

                info!("[{}] Chain ingestion task stopped", chain_config.name);
            });
        }

        // Wait for all tasks to complete
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(_) => {
                    debug!("Chain ingestion task completed successfully");
                }
                Err(e) if e.is_panic() => {
                    error!("Chain ingestion task panicked: {}", e);
                }
                Err(e) => {
                    warn!("Chain ingestion task error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Ingest events with circuit breaker and exponential backoff
    /// 
    /// Production-grade implementation with:
    /// - Event-driven shutdown (tokio::select!)
    /// - Exponential backoff with jitter
    /// - Circuit breaker state machine
    /// - Health tracking
    /// - Structured concurrency
    async fn ingest_chain_with_circuit_breaker(
        chain_config: &ChainConfig,
        deduplicator: &Arc<Mutex<Deduplicator>>,
        publisher: &Arc<Mutex<StreamPublisher>>,
        health: &mut ChainHealth,
        mut shutdown_rx: broadcast::Receiver<()>,
        base_delay: u64,
        max_delay: u64,
    ) {
        let health_check_interval = Duration::from_secs(30);
        
        loop {
            // Check circuit breaker before attempting connection
            if !health.should_attempt_reconnect(base_delay, max_delay) {
                let backoff = health.calculate_backoff(base_delay, max_delay);
                info!(
                    "[{}] Circuit breaker OPEN (failures: {}). Waiting {:?} before retry...",
                    chain_config.name, health.consecutive_failures, backoff
                );
                
                // Event-driven wait with shutdown capability
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("[{}] Shutdown during circuit breaker wait", chain_config.name);
                        return;
                    }
                    _ = tokio::time::sleep(backoff) => {
                        // Backoff complete, continue to reconnection attempt
                    }
                }
            }

            // Attempt connection
            info!(
                "[{}] Attempting WebSocket connection (state: {:?})...",
                chain_config.name, health.circuit_state
            );

            let client_result = WebSocketClient::connect(
                &chain_config.ws_url,
                chain_config.chain_id,
                &chain_config.name,
            )
            .await;

            let mut client = match client_result {
                Ok(c) => {
                    info!("[{}] Connected and subscribed to newHeads", chain_config.name);
                    health.record_success();
                    c
                }
                Err(e) => {
                    error!("[{}] Connection failed: {}", chain_config.name, e);
                    health.record_failure();
                    continue;
                }
            };

            // Process events with health monitoring
            let result = Self::process_events_loop(
                chain_config,
                &mut client,
                deduplicator,
                publisher,
                health,
                &mut shutdown_rx,
                health_check_interval,
            )
            .await;

            match result {
                Ok(()) => {
                    info!("[{}] Event processing loop exited normally", chain_config.name);
                    return; // Shutdown requested
                }
                Err(e) => {
                    error!("[{}] Event processing failed: {}", chain_config.name, e);
                    health.record_failure();
                }
            }
        }
    }

    /// Process events from WebSocket with tokio::select! for event-driven patterns
    async fn process_events_loop(
        chain_config: &ChainConfig,
        client: &mut WebSocketClient,
        deduplicator: &Arc<Mutex<Deduplicator>>,
        publisher: &Arc<Mutex<StreamPublisher>>,
        health: &mut ChainHealth,
        shutdown_rx: &mut broadcast::Receiver<()>,
        health_check_interval: Duration,
    ) -> Result<()> {
        let mut events_processed = 0u64;
        let mut blocks_processed = 0u64;
        let mut health_check_timer = tokio::time::interval(health_check_interval);
        health_check_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                // Event-driven shutdown - no polling needed!
                _ = shutdown_rx.recv() => {
                    info!("[{}] Shutdown signal received", chain_config.name);
                    return Ok(());
                }

                // Health check - detect stalled connections
                _ = health_check_timer.tick() => {
                    let elapsed = health.time_since_last_event();
                    if elapsed > Duration::from_secs(120) {
                        warn!(
                            "[{}] No events for {:?}. Connection may be stalled.",
                            chain_config.name, elapsed
                        );
                        // Could trigger reconnect here if needed
                    } else {
                        debug!("[{}] Health check OK. Last event: {:?} ago", chain_config.name, elapsed);
                    }
                }

                // Process next event from WebSocket
                result = client.next_event() => {
                    match result {
                        Ok(Some(event)) => {
                            events_processed += 1;
                            health.record_success();

                            // Log every 100 events to avoid spam
                            if events_processed % 100 == 0 {
                                info!(
                                    "[{}] Processed {} events from {} blocks",
                                    chain_config.name, events_processed, blocks_processed
                                );
                            }

                            debug!(
                                "[{}] Event: block={} tx={} contract={} topics={}",
                                chain_config.name,
                                event.block_number,
                                &event.transaction_hash[..10],
                                &event.contract_address[..10],
                                event.topics.len()
                            );

                            // Phase 4: Check deduplication
                            let event_id = event.event_id();
                            let mut dedup = deduplicator.lock().await;
                            
                            let should_process = match dedup.is_duplicate(&event_id).await {
                                Ok(true) => {
                                    debug!("[{}] Skipping duplicate: {}", chain_config.name, event_id);
                                    false
                                }
                                Ok(false) => {
                                    debug!("[{}] New event: {}", chain_config.name, event_id);
                                    true
                                }
                                Err(e) => {
                                    error!("[{}] Deduplication error: {}. Processing anyway.", chain_config.name, e);
                                    true
                                }
                            };
                            drop(dedup);

                            if !should_process {
                                continue;
                            }

                            // Phase 5: Publish to Redis Stream
                            let mut pub_client = publisher.lock().await;
                            match pub_client.publish(&event).await {
                                Ok(stream_id) => {
                                    debug!(
                                        "[{}] Published to stream: {} (stream_id: {})",
                                        chain_config.name,
                                        event.stream_name(),
                                        stream_id
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        "[{}] Failed to publish: {}",
                                        chain_config.name, e
                                    );
                                }
                            }
                            drop(pub_client);

                            blocks_processed += 1;
                        }
                        Ok(None) => {
                            info!(
                                "[{}] WebSocket closed. Stats: {} events, {} blocks",
                                chain_config.name, events_processed, blocks_processed
                            );
                            return Err(anyhow!("WebSocket connection closed"));
                        }
                        Err(e) => {
                            error!(
                                "[{}] Error processing event: {}. Stats: {} events, {} blocks",
                                chain_config.name, e, events_processed, blocks_processed
                            );
                            return Err(e);
                        }
                    }
                }
            }
        }
    }

    /// Shutdown all chain ingestion tasks gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Sending shutdown signal to all chains");
        let _ = self.shutdown_tx.send(());
        
        // Give tasks time to finish gracefully
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis - run with: cargo test -- --ignored
    async fn test_manager_creation() {
        let config = IngestorConfig {
            chains: vec![],
            redis_host: "localhost".to_string(),
            redis_port: 6379,
            redis_password: None,
            metrics_port: 9090,
            dedup_ttl_seconds: 86400,
        };

        let manager = ChainIngestionManager::new(config).await.unwrap();
        assert!(manager.shutdown().await.is_ok());
    }
}
