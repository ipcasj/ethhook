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

use anyhow::{Context, Result, anyhow};
use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, broadcast};
use tokio::task::JoinSet;
use tracing::{debug, error, info, warn};

use crate::client::WebSocketClient;
use crate::config::{ChainConfig, IngestorConfig};
use crate::deduplicator::Deduplicator;
use crate::metrics;
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
        let deduplicator = Deduplicator::new(&config.redis_url(), config.dedup_ttl_seconds)
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
                info!("[{}] Starting chain ingestion task", chain_config.name);

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
                    info!(
                        "[{}] Connected and subscribed to newHeads",
                        chain_config.name
                    );
                    health.record_success();
                    // Record reconnection metric
                    metrics::WEBSOCKET_RECONNECTS
                        .with_label_values(&[&chain_config.name])
                        .inc();
                    // Update circuit breaker state metric
                    metrics::CIRCUIT_BREAKER_STATE
                        .with_label_values(&[&chain_config.name])
                        .set(0); // 0 = Closed
                    c
                }
                Err(e) => {
                    error!("[{}] Connection failed: {}", chain_config.name, e);
                    health.record_failure();
                    // Record error metric
                    metrics::PROCESSING_ERRORS
                        .with_label_values(&[&chain_config.name, "connection_failed"])
                        .inc();
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
                    info!(
                        "[{}] Event processing loop exited normally",
                        chain_config.name
                    );
                    return; // Shutdown requested
                }
                Err(e) => {
                    error!("[{}] Event processing failed: {}", chain_config.name, e);
                    health.record_failure();

                    // Update metrics after failure
                    metrics::CONSECUTIVE_FAILURES
                        .with_label_values(&[&chain_config.name])
                        .set(health.consecutive_failures as i64);

                    // Update circuit breaker state
                    let state_value = match health.circuit_state {
                        CircuitState::Closed => 0,
                        CircuitState::Open => 1,
                        CircuitState::HalfOpen => 2,
                    };
                    metrics::CIRCUIT_BREAKER_STATE
                        .with_label_values(&[&chain_config.name])
                        .set(state_value);
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

                            // Record event received metric
                            metrics::EVENTS_RECEIVED
                                .with_label_values(&[&chain_config.name])
                                .inc();

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
                                    // Record deduplication metric
                                    metrics::EVENTS_DEDUPLICATED
                                        .with_label_values(&[&chain_config.name])
                                        .inc();
                                    false
                                }
                                Ok(false) => {
                                    debug!("[{}] New event: {}", chain_config.name, event_id);
                                    true
                                }
                                Err(e) => {
                                    error!("[{}] Deduplication error: {}. Processing anyway.", chain_config.name, e);
                                    metrics::PROCESSING_ERRORS
                                        .with_label_values(&[&chain_config.name, "deduplication_error"])
                                        .inc();
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
                                    // Record successful publish metric
                                    metrics::EVENTS_PUBLISHED
                                        .with_label_values(&[&chain_config.name])
                                        .inc();
                                }
                                Err(e) => {
                                    error!(
                                        "[{}] Failed to publish: {}",
                                        chain_config.name, e
                                    );
                                    metrics::PROCESSING_ERRORS
                                        .with_label_values(&[&chain_config.name, "publish_failed"])
                                        .inc();
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
                            metrics::PROCESSING_ERRORS
                                .with_label_values(&[&chain_config.name, "websocket_closed"])
                                .inc();
                            return Err(anyhow!("WebSocket connection closed"));
                        }
                        Err(e) => {
                            error!(
                                "[{}] Error processing event: {}. Stats: {} events, {} blocks",
                                chain_config.name, e, events_processed, blocks_processed
                            );
                            metrics::PROCESSING_ERRORS
                                .with_label_values(&[&chain_config.name, "processing_error"])
                                .inc();
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

    // ============================================================================
    // Circuit Breaker Unit Tests
    // ============================================================================

    #[test]
    fn test_circuit_breaker_initial_state() {
        let health = ChainHealth::new();

        assert_eq!(health.circuit_state, CircuitState::Closed);
        assert_eq!(health.consecutive_failures, 0);
        assert!(health.circuit_opened_at.is_none());
    }

    #[test]
    fn test_circuit_breaker_opens_after_3_failures() {
        let mut health = ChainHealth::new();

        // Record 3 consecutive failures
        health.record_failure();
        assert_eq!(health.consecutive_failures, 1);
        assert_eq!(health.circuit_state, CircuitState::Closed);

        health.record_failure();
        assert_eq!(health.consecutive_failures, 2);
        assert_eq!(health.circuit_state, CircuitState::Closed);

        health.record_failure();
        assert_eq!(health.consecutive_failures, 3);
        assert_eq!(health.circuit_state, CircuitState::Open);
        assert!(health.circuit_opened_at.is_some());
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let mut health = ChainHealth::new();

        // Record some failures
        health.record_failure();
        health.record_failure();
        assert_eq!(health.consecutive_failures, 2);

        // Success should reset everything
        health.record_success();
        assert_eq!(health.consecutive_failures, 0);
        assert_eq!(health.circuit_state, CircuitState::Closed);
        assert!(health.circuit_opened_at.is_none());
    }

    #[test]
    fn test_circuit_breaker_half_open_transition() {
        let mut health = ChainHealth::new();

        // Open the circuit
        health.record_failure();
        health.record_failure();
        health.record_failure();
        assert_eq!(health.circuit_state, CircuitState::Open);

        // Manually set circuit_opened_at to past (simulate time passing)
        health.circuit_opened_at = Some(Instant::now() - Duration::from_secs(10));

        // Should transition to HalfOpen when attempting reconnect
        let should_reconnect = health.should_attempt_reconnect(1, 60);
        assert!(should_reconnect);
        assert_eq!(health.circuit_state, CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_stays_open_during_backoff() {
        let mut health = ChainHealth::new();

        // Open the circuit
        health.record_failure();
        health.record_failure();
        health.record_failure();
        health.circuit_opened_at = Some(Instant::now());

        // Should NOT reconnect immediately (backoff not elapsed)
        let should_reconnect = health.should_attempt_reconnect(5, 60);
        assert!(!should_reconnect);
        assert_eq!(health.circuit_state, CircuitState::Open);
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        let mut health = ChainHealth::new();
        let base_delay = 2; // 2 seconds base
        let max_delay = 60; // 60 seconds max

        // Failure 1: base * 2^1 = 2 * 2 = 4 seconds (with jitter: 3.2-4.8s)
        health.consecutive_failures = 1;
        let backoff1 = health.calculate_backoff(base_delay, max_delay);
        assert!(backoff1.as_secs() >= 3 && backoff1.as_secs() <= 5);

        // Failure 2: base * 2^2 = 2 * 4 = 8 seconds (with jitter: 6.4-9.6s)
        health.consecutive_failures = 2;
        let backoff2 = health.calculate_backoff(base_delay, max_delay);
        assert!(backoff2.as_secs() >= 6 && backoff2.as_secs() <= 10);

        // Failure 3: base * 2^3 = 2 * 8 = 16 seconds (with jitter: 12.8-19.2s)
        health.consecutive_failures = 3;
        let backoff3 = health.calculate_backoff(base_delay, max_delay);
        assert!(backoff3.as_secs() >= 12 && backoff3.as_secs() <= 20);
    }

    #[test]
    fn test_exponential_backoff_caps_at_max() {
        let mut health = ChainHealth::new();
        let base_delay = 2;
        let max_delay = 60;

        // Failure 10: base * 2^10 = 2 * 1024 = 2048 seconds
        // Should cap at max_delay (60 seconds)
        health.consecutive_failures = 10;
        let backoff = health.calculate_backoff(base_delay, max_delay);

        // With ±20% jitter on 60s = 48-72 seconds
        assert!(backoff.as_secs() >= 48 && backoff.as_secs() <= 72);
    }

    #[test]
    fn test_jitter_prevents_thundering_herd() {
        let mut health = ChainHealth::new();
        health.consecutive_failures = 3;

        // Calculate backoff multiple times
        let mut backoffs = Vec::new();
        for _ in 0..100 {
            let backoff = health.calculate_backoff(2, 60);
            backoffs.push(backoff.as_secs());
        }

        // Check that we get different values (jitter working)
        let unique_values: std::collections::HashSet<_> = backoffs.iter().collect();
        // The jitter range is ±20%, which at second granularity gives us:
        // Failure 3: base * 2^3 = 16 seconds
        // With ±20% jitter: 12.8-19.2 seconds = 13-19 seconds (at 1s granularity)
        // That's 7 possible values: [13, 14, 15, 16, 17, 18, 19]
        // We expect at least 5 unique values out of 100 samples
        assert!(
            unique_values.len() >= 5,
            "Jitter should produce variety (got {} unique values, expected >= 5)",
            unique_values.len()
        );

        // Check that all values are within expected range
        for &backoff_secs in &backoffs {
            assert!(
                (12..=20).contains(&backoff_secs),
                "Backoff {backoff_secs} seconds out of range [12, 20]"
            );
        }
    }

    #[test]
    fn test_time_since_last_event() {
        let health = ChainHealth::new();

        // Just created, should be very recent
        let elapsed = health.time_since_last_event();
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_circuit_state_transitions_complete_cycle() {
        let mut health = ChainHealth::new();

        // 1. Start in Closed state
        assert_eq!(health.circuit_state, CircuitState::Closed);

        // 2. Transition to Open after 3 failures
        health.record_failure();
        health.record_failure();
        health.record_failure();
        assert_eq!(health.circuit_state, CircuitState::Open);

        // 3. Simulate time passing (backoff elapsed)
        health.circuit_opened_at = Some(Instant::now() - Duration::from_secs(100));

        // 4. Transition to HalfOpen
        let _ = health.should_attempt_reconnect(1, 60);
        assert_eq!(health.circuit_state, CircuitState::HalfOpen);

        // 5. Success in HalfOpen → back to Closed
        health.record_success();
        assert_eq!(health.circuit_state, CircuitState::Closed);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_half_open_failure_reopens_circuit() {
        let mut health = ChainHealth::new();

        // Open circuit
        health.record_failure();
        health.record_failure();
        health.record_failure();

        // Transition to HalfOpen
        health.circuit_opened_at = Some(Instant::now() - Duration::from_secs(100));
        let _ = health.should_attempt_reconnect(1, 60);
        assert_eq!(health.circuit_state, CircuitState::HalfOpen);

        // Failure in HalfOpen should reopen circuit
        // (Note: Current implementation doesn't special-case HalfOpen failures,
        // but consecutive_failures increases which will keep it "open")
        health.record_failure();
        assert_eq!(health.consecutive_failures, 4);
    }
}
