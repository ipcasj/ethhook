/*!
 * WebSocket Module - Unified Pipeline
 *
 * Connects to blockchain RPC providers via WebSocket and streams events
 * to in-memory tokio::mpsc channels (NOT Redis).
 *
 * ## Key Differences from event-ingestor
 *
 * 1. **No Redis**: Events go directly to tokio::mpsc channel
 * 2. **No Deduplication**: Done in-memory during batch processing
 * 3. **Safety First**: All operations have timeouts, no .unwrap()
 * 4. **Simplified**: Removed unnecessary abstraction layers
 *
 * ## Performance Target
 *
 * - Latency: < 100ms from block mined to channel send
 * - Throughput: 10,000 events/sec across all chains
 * - Memory: < 20MB per chain connection
 */

use anyhow::{Context, Result};
use chrono::Utc;
use ethhook_domain::event::BlockchainEvent;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config_db::{ENDPOINT_CACHE, APPLICATION_CACHE};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Chain configuration for WebSocket connection
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub name: String,
    pub rpc_ws: String,
    pub chain_id: i32,
}

impl ChainConfig {
    /// Get chain ID from chain name
    pub fn chain_id_from_name(name: &str) -> i32 {
        match name {
            "Ethereum" => 1,
            "Sepolia" => 11155111,
            "Arbitrum" => 42161,
            "Optimism" => 10,
            "Base" => 8453,
            _ => 0,
        }
    }
}

/// Start WebSocket ingestors for all configured chains
///
/// Safety Rule #4: 30s timeout per operation
pub async fn start_ingestor(
    event_tx: mpsc::Sender<BlockchainEvent>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    info!("Starting WebSocket ingestor");

    // Load chain configurations from environment
    let chains = load_chain_configs()?;
    info!("Configured {} chains", chains.len());

    // Spawn a task for each chain
    let mut tasks = Vec::new();
    for chain in chains {
        let tx = event_tx.clone();
        let shutdown = shutdown_rx.resubscribe();

        let task = tokio::spawn(async move { ingest_chain(chain, tx, shutdown).await });

        tasks.push(task);
    }

    // Wait for shutdown signal
    let _ = shutdown_rx.recv().await;
    info!("WebSocket ingestor received shutdown signal");

    // Give tasks 5s to finish gracefully
    tokio::time::timeout(Duration::from_secs(5), async {
        for task in tasks {
            let _ = task.await;
        }
    })
    .await
    .ok();

    info!("WebSocket ingestor shutdown complete");
    Ok(())
}

/// Ingest events from a single blockchain
///
/// Safety: Never panics, always returns Result
async fn ingest_chain(
    chain: ChainConfig,
    event_tx: mpsc::Sender<BlockchainEvent>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    info!("[{}] Starting chain ingestion", chain.name);

    let mut consecutive_failures = 0u32;
    let max_failures = 10;

    loop {
        // Check for shutdown
        if shutdown_rx.try_recv().is_ok() {
            info!("[{}] Shutdown requested", chain.name);
            break;
        }

        // Safety Rule #5: Graceful degradation
        // After 10 consecutive failures, stop trying (prevents infinite loop)
        if consecutive_failures >= max_failures {
            error!(
                "[{}] Too many consecutive failures ({}), stopping ingestion",
                chain.name, consecutive_failures
            );
            break;
        }

        // Attempt to connect and ingest
        match connect_and_ingest(&chain, &event_tx, &mut shutdown_rx).await {
            Ok(_) => {
                info!("[{}] Connection closed gracefully", chain.name);
                consecutive_failures = 0;
            }
            Err(e) => {
                consecutive_failures += 1;
                error!(
                    "[{}] Ingestion error (failure {}/{}): {}",
                    chain.name, consecutive_failures, max_failures, e
                );

                // Exponential backoff: 1s, 2s, 4s, 8s, ..., max 60s
                let backoff_secs = 2u64.saturating_pow(consecutive_failures.min(6));
                let backoff = Duration::from_secs(backoff_secs);

                warn!("[{}] Reconnecting in {:?}", chain.name, backoff);

                // Safety Rule #4: Timeout on sleep (in case of shutdown during backoff)
                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {}
                    _ = shutdown_rx.recv() => {
                        info!("[{}] Shutdown during backoff", chain.name);
                        break;
                    }
                }
            }
        }
    }

    info!("[{}] Chain ingestion stopped", chain.name);
    Ok(())
}

/// Connect to WebSocket and ingest events until error or shutdown
///
/// Safety Rule #1: No .unwrap() - all errors propagated with ?
async fn connect_and_ingest(
    chain: &ChainConfig,
    event_tx: &mpsc::Sender<BlockchainEvent>,
    shutdown_rx: &mut broadcast::Receiver<()>,
) -> Result<()> {
    // Safety Rule #4: 30s timeout for connection
    let mut client = tokio::time::timeout(
        Duration::from_secs(30),
        WebSocketClient::connect(&chain.rpc_ws, &chain.name, chain.chain_id),
    )
    .await
    .context("Connection timeout")?
    .context("Connection failed")?;

    info!("[{}] Connected and subscribed", chain.name);

    // Process events until error or shutdown
    loop {
        tokio::select! {
            // Safety Rule #4: 30s timeout for receiving next event
            result = tokio::time::timeout(Duration::from_secs(30), client.next_event()) => {
                match result {
                    Ok(Ok(Some(events))) => {
                        // Send events to channel
                        for event in events {
                            // Safety Rule #4: 5s timeout for channel send
                            // If batch processor is backed up, don't hang forever
                            match tokio::time::timeout(
                                Duration::from_secs(5),
                                event_tx.send(event)
                            ).await {
                                Ok(Ok(_)) => {
                                    // Success - event sent to batch processor
                                }
                                Ok(Err(e)) => {
                                    error!("[{}] Failed to send event to channel: {}", chain.name, e);
                                    return Err(e.into());
                                }
                                Err(_) => {
                                    error!("[{}] Timeout sending event to channel (batch processor backed up?)", chain.name);
                                    return Err(anyhow::anyhow!("Channel send timeout"));
                                }
                            }
                        }
                    }
                    Ok(Ok(None)) => {
                        // No events in this block (possible but rare)
                        continue;
                    }
                    Ok(Err(e)) => {
                        error!("[{}] Error receiving event: {}", chain.name, e);
                        return Err(e);
                    }
                    Err(_) => {
                        warn!("[{}] No events received in 30s, connection may be stale", chain.name);
                        return Err(anyhow::anyhow!("Event receive timeout"));
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!("[{}] Shutdown signal received", chain.name);
                return Ok(());
            }
        }
    }
}

/// WebSocket client for a single chain
struct WebSocketClient {
    stream: WsStream,
    chain_name: String,
    chain_id: i32,
    subscription_id: Option<String>,
    request_id: u64,
}

impl WebSocketClient {
    /// Connect and subscribe to new block headers
    ///
    /// Safety: No .unwrap(), returns Result
    async fn connect(ws_url: &str, chain_name: &str, chain_id: i32) -> Result<Self> {
        info!("[{}] Connecting to {}", chain_name, ws_url);

        // Connect to WebSocket
        let (stream, _) = connect_async(ws_url)
            .await
            .context("WebSocket connection failed")?;

        let mut client = Self {
            stream,
            chain_name: chain_name.to_string(),
            chain_id,
            subscription_id: None,
            request_id: 1,
        };

        // Subscribe to new block headers
        client.subscribe_to_new_heads().await?;

        Ok(client)
    }

    /// Subscribe to eth_subscribe("newHeads")
    async fn subscribe_to_new_heads(&mut self) -> Result<()> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "eth_subscribe",
            "params": ["newHeads"],
            "id": 1
        });

        debug!("[{}] Sending subscription request", self.chain_name);

        // Send subscription request
        self.stream
            .send(Message::Text(request.to_string()))
            .await
            .context("Failed to send subscription")?;

        // Wait for subscription response
        if let Some(msg) = self.stream.next().await {
            let msg = msg.context("Failed to receive subscription response")?;

            if let Message::Text(text) = msg {
                let response: Value =
                    serde_json::from_str(&text).context("Failed to parse subscription response")?;

                if let Some(sub_id) = response.get("result").and_then(|r| r.as_str()) {
                    self.subscription_id = Some(sub_id.to_string());
                    info!("[{}] Subscribed with ID: {}", self.chain_name, sub_id);
                } else if let Some(error) = response.get("error") {
                    return Err(anyhow::anyhow!("Subscription error: {error}"));
                }
            }
        }

        if self.subscription_id.is_none() {
            return Err(anyhow::anyhow!("No subscription ID received"));
        }

        Ok(())
    }

    /// Receive next block and extract events
    ///
    /// Returns Vec of events (empty vec if no events in block)
    async fn next_event(&mut self) -> Result<Option<Vec<BlockchainEvent>>> {
        // Read next WebSocket message
        let msg = self
            .stream
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("WebSocket stream ended"))??;

        if let Message::Text(text) = msg {
            let message: Value =
                serde_json::from_str(&text).context("Failed to parse WebSocket message")?;

            // Check if this is a subscription notification
            // Note: Using nested if statements instead of let chains for stable Rust compatibility.
            // Clippy suggests collapsing these, but that requires unstable let chains (RFC #53667).
            #[allow(clippy::collapsible_if)]
            if message.get("method").and_then(|m| m.as_str()) == Some("eth_subscription") {
                #[allow(clippy::collapsible_if)]
                if let Some(params) = message.get("params") {
                    if let Some(result) = params.get("result") {
                        return self.process_block_header(result).await;
                    }
                }
            }
        }

        Ok(None)
    }

    /// Process block header and fetch transaction receipts
    async fn process_block_header(&mut self, header: &Value) -> Result<Option<Vec<BlockchainEvent>>> {
        // Extract block number and hash
        let block_number = header
            .get("number")
            .and_then(|n| n.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .ok_or_else(|| anyhow::anyhow!("Invalid block number"))?;

        let block_hash = header
            .get("hash")
            .and_then(|h| h.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing block hash"))?
            .to_string();

        debug!(
            "[{}] Processing block #{} ({})",
            self.chain_name, block_number, block_hash
        );

        // Fetch block receipts via JSON-RPC
        let receipts = match self.fetch_block_receipts(&block_hash).await {
            Ok(receipts) => receipts,
            Err(e) => {
                warn!("[{}] Failed to fetch receipts for block {}: {}", 
                    self.chain_name, block_number, e);
                return Ok(Some(Vec::new()));
            }
        };

        // Extract events from receipts
        let mut events = Vec::new();
        
        for receipt in receipts.as_array().unwrap_or(&vec![]) {
            let tx_hash = receipt
                .get("transactionHash")
                .and_then(|h| h.as_str())
                .unwrap_or("")
                .to_string();
                
            let logs = receipt.get("logs").and_then(|l| l.as_array());
            
            if let Some(logs) = logs {
                for log in logs {
                    // Extract log fields
                    let contract_address = log
                        .get("address")
                        .and_then(|a| a.as_str())
                        .unwrap_or("")
                        .to_lowercase();
                    
                    // Check if this contract is monitored
                    if !ENDPOINT_CACHE.contains_key(&contract_address) {
                        continue;
                    }
                    
                    let log_index = log
                        .get("logIndex")
                        .and_then(|i| i.as_str())
                        .and_then(|s| i32::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                        .unwrap_or(0);
                    
                    let topics: Vec<String> = log
                        .get("topics")
                        .and_then(|t| t.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_default();
                    
                    let mut data = log
                        .get("data")
                        .and_then(|d| d.as_str())
                        .unwrap_or("0x")
                        .to_string();
                    
                    // Truncate data if too large (ClickHouse default limit: 1GB, we use 10MB)
                    const MAX_DATA_SIZE: usize = 10 * 1024 * 1024; // 10MB
                    if data.len() > MAX_DATA_SIZE {
                        warn!(
                            "[{}] Truncating large data field: {} bytes -> {} bytes (tx: {})",
                            self.chain_name,
                            data.len(),
                            MAX_DATA_SIZE,
                            tx_hash
                        );
                        data.truncate(MAX_DATA_SIZE);
                    }
                    
                    // Lookup endpoint metadata from cache
                    let (endpoint_id, application_id, user_id) = ENDPOINT_CACHE
                        .get(&contract_address)
                        .and_then(|endpoints| {
                            endpoints.first().map(|endpoint| {
                                let user_id = APPLICATION_CACHE
                                    .get(&endpoint.application_id)
                                    .map(|entry| *entry.value());
                                (endpoint.id, endpoint.application_id, user_id)
                            })
                        })
                        .map(|(endpoint_id, app_id, user_id)| (Some(endpoint_id), Some(app_id), user_id))
                        .unwrap_or((None, None, None));
                    
                    // Create BlockchainEvent
                    let event = BlockchainEvent {
                        id: Uuid::new_v4(),
                        endpoint_id,
                        application_id,
                        user_id,
                        chain_id: self.chain_id as u32,
                        block_number: block_number as i64,
                        block_hash: block_hash.clone(),
                        transaction_hash: tx_hash.clone(),
                        log_index,
                        contract_address,
                        topics,
                        data,
                        ingested_at: Utc::now(),
                        processed_at: None,
                    };
                    
                    events.push(event);
                }
            }
        }
        
        if !events.is_empty() {
            info!(
                "[{}] Extracted {} events from block #{}",
                self.chain_name,
                events.len(),
                block_number
            );
        }
        
        Ok(Some(events))
    }

    /// Fetch block receipts via JSON-RPC over WebSocket
    async fn fetch_block_receipts(&mut self, block_hash: &str) -> Result<Value> {
        let request_id = self.request_id;
        self.request_id += 1;
        
        let request = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockReceipts",
            "params": [block_hash],
            "id": request_id
        });
        
        // Send request
        self.stream
            .send(Message::Text(request.to_string()))
            .await
            .context("Failed to send eth_getBlockReceipts request")?;
        
        // Wait for response (with timeout)
        let response = tokio::time::timeout(
            Duration::from_secs(10),
            self.read_rpc_response(request_id)
        )
        .await
        .context("Timeout waiting for eth_getBlockReceipts response")??;
        
        Ok(response)
    }

    /// Read JSON-RPC response matching the request ID
    async fn read_rpc_response(&mut self, expected_id: u64) -> Result<Value> {
        // Read messages until we get a response with matching ID
        loop {
            let msg = self
                .stream
                .next()
                .await
                .ok_or_else(|| anyhow::anyhow!("WebSocket stream ended"))??;
            
            if let Message::Text(text) = msg {
                let response: Value = serde_json::from_str(&text)
                    .context("Failed to parse JSON-RPC response")?;
                
                // Check if this is our response (matching ID)
                if let Some(id) = response.get("id").and_then(|i| i.as_u64()) {
                    if id == expected_id {
                        // Check for error
                        if let Some(error) = response.get("error") {
                            return Err(anyhow::anyhow!("RPC error: {}", error));
                        }
                        
                        // Return result
                        return response
                            .get("result")
                            .cloned()
                            .ok_or_else(|| anyhow::anyhow!("Missing result in response"));
                    }
                }
                
                // This might be a subscription notification - ignore and continue
                debug!("[{}] Ignoring non-matching response", self.chain_name);
            }
        }
    }
}

/// Load chain configurations from environment variables
///
/// Safety: Returns Result instead of panicking
fn load_chain_configs() -> Result<Vec<ChainConfig>> {
    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());

    let chains = match environment.as_str() {
        "development" => {
            // Development: Sepolia testnet only
            let name = "Sepolia".to_string();
            vec![ChainConfig {
                chain_id: ChainConfig::chain_id_from_name(&name),
                name,
                rpc_ws: std::env::var("SEPOLIA_WS_URL")
                    .or_else(|_| std::env::var("SEPOLIA_WSS"))
                    .context("SEPOLIA_WS_URL or SEPOLIA_WSS environment variable required")?,
            }]
        }
        _ => {
            // Production: Ethereum, Arbitrum, Optimism, Base
            let mut chains = Vec::new();
            
            // Try each chain, skip if not configured (allows partial deployment)
            if let Ok(url) = std::env::var("ETHEREUM_WS_URL").or_else(|_| std::env::var("ETHEREUM_WSS")) {
                let name = "Ethereum".to_string();
                chains.push(ChainConfig {
                    chain_id: ChainConfig::chain_id_from_name(&name),
                    name,
                    rpc_ws: url,
                });
            }
            
            if let Ok(url) = std::env::var("ARBITRUM_WS_URL").or_else(|_| std::env::var("ARBITRUM_WSS")) {
                let name = "Arbitrum".to_string();
                chains.push(ChainConfig {
                    chain_id: ChainConfig::chain_id_from_name(&name),
                    name,
                    rpc_ws: url,
                });
            }
            
            if let Ok(url) = std::env::var("OPTIMISM_WS_URL").or_else(|_| std::env::var("OPTIMISM_WSS")) {
                let name = "Optimism".to_string();
                chains.push(ChainConfig {
                    chain_id: ChainConfig::chain_id_from_name(&name),
                    name,
                    rpc_ws: url,
                });
            }
            
            if let Ok(url) = std::env::var("BASE_WS_URL").or_else(|_| std::env::var("BASE_WSS")) {
                let name = "Base".to_string();
                chains.push(ChainConfig {
                    chain_id: ChainConfig::chain_id_from_name(&name),
                    name,
                    rpc_ws: url,
                });
            }
            
            if chains.is_empty() {
                warn!("No blockchain RPC WebSocket URLs configured. Set ETHEREUM_WS_URL, ARBITRUM_WS_URL, OPTIMISM_WS_URL, or BASE_WS_URL");
            }
            
            chains
        }
    };

    Ok(chains)
}
