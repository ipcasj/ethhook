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

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::client::WebSocketClient;
use crate::config::{ChainConfig, IngestorConfig};

/// Manages ingestion across multiple chains
pub struct ChainIngestionManager {
    config: IngestorConfig,
    shutdown_tx: broadcast::Sender<()>,
}

impl ChainIngestionManager {
    /// Create new chain ingestion manager
    pub async fn new(config: IngestorConfig) -> Result<Self> {
        let (shutdown_tx, _) = broadcast::channel(1);

        Ok(Self {
            config,
            shutdown_tx,
        })
    }

    /// Start ingesting events from all configured chains
    /// 
    /// Spawns a tokio task for each chain. Each task runs independently
    /// and will automatically reconnect on failure.
    pub async fn start_all_chains(&self) -> Result<()> {
        let mut handles = vec![];

        for chain in &self.config.chains {
            let chain_config = chain.clone();
            let mut shutdown_rx = self.shutdown_tx.subscribe();

            // Spawn independent task for this chain
            let handle = tokio::spawn(async move {
                info!(
                    "[{}] Starting chain ingestion task",
                    chain_config.name
                );

                loop {
                    // Check for shutdown signal
                    if shutdown_rx.try_recv().is_ok() {
                        info!("[{}] Shutdown signal received", chain_config.name);
                        break;
                    }

                    // Attempt to ingest events
                    if let Err(e) = Self::ingest_chain_with_retry(&chain_config).await {
                        error!(
                            "[{}] Ingestion failed: {}. Retrying in {} seconds...",
                            chain_config.name, e, chain_config.reconnect_delay_secs
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(
                            chain_config.reconnect_delay_secs,
                        ))
                        .await;
                    }
                }

                info!("[{}] Chain ingestion task stopped", chain_config.name);
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete (they shouldn't unless shutdown)
        for handle in handles {
            if let Err(e) = handle.await {
                warn!("Chain ingestion task panicked: {}", e);
            }
        }

        Ok(())
    }

    /// Ingest events from a single chain with retry logic
    /// 
    /// This is the main ingestion loop for a single chain:
    /// 1. Connect to WebSocket
    /// 2. Subscribe to new blocks
    /// 3. Process events as they arrive
    /// 4. Reconnect on failure
    async fn ingest_chain_with_retry(chain_config: &ChainConfig) -> Result<()> {
        // Connect to WebSocket
        let mut client = WebSocketClient::connect(
            &chain_config.ws_url,
            chain_config.chain_id,
            &chain_config.name,
        )
        .await
        .context("Failed to connect to WebSocket")?;

        info!(
            "[{}] Connected and subscribed to newHeads",
            chain_config.name
        );

        // Process events from WebSocket
        loop {
            match client.next_event().await {
                Ok(Some(event)) => {
                    info!(
                        "[{}] Received event: block={} tx={} contract={}",
                        chain_config.name,
                        event.block_number,
                        event.transaction_hash,
                        event.contract_address
                    );

                    // TODO: Phase 4 - Check deduplication
                    // TODO: Phase 5 - Publish to Redis Stream
                }
                Ok(None) => {
                    warn!(
                        "[{}] WebSocket connection closed, will reconnect",
                        chain_config.name
                    );
                    break;
                }
                Err(e) => {
                    error!(
                        "[{}] Error processing event: {}",
                        chain_config.name, e
                    );
                    break;
                }
            }
        }

        Ok(())
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
