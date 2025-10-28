/*!
 * Mock Ethereum JSON-RPC WebSocket Server
 *
 * Simulates an Ethereum JSON-RPC WebSocket endpoint for E2E testing.
 * Responds to eth_subscribe and sends mock block notifications.
 */

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tracing::{debug, error, info};

/// Mock Ethereum RPC server that accepts WebSocket connections
pub struct MockEthRpcServer {
    addr: SocketAddr,
    shutdown_tx: broadcast::Sender<()>,
}

impl MockEthRpcServer {
    /// Start mock RPC server on a random available port
    pub async fn start() -> anyhow::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let (shutdown_tx, _) = broadcast::channel::<()>(1);

        info!("🔧 Mock Ethereum RPC server starting on {}", addr);

        let shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            if let Err(e) = Self::run_server(listener, shutdown_rx).await {
                error!("Mock RPC server error: {}", e);
            }
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(Self { addr, shutdown_tx })
    }

    /// Get the WebSocket URL for this server
    pub fn url(&self) -> String {
        format!("ws://{}", self.addr)
    }

    /// Stop the mock server
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Run the server loop
    async fn run_server(
        listener: TcpListener,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!("New connection from: {}", addr);
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(stream).await {
                                    error!("Connection handler error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Mock RPC server shutting down");
                    break;
                }
            }
        }
        Ok(())
    }

    /// Handle a single WebSocket connection
    async fn handle_connection(stream: TcpStream) -> anyhow::Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        debug!("WebSocket connection established");

        let mut block_sent = false;

        // Process incoming messages in a loop
        while let Some(msg) = read.next().await {
            let msg = msg?;

            if let Message::Text(text) = msg {
                debug!("Received: {}", text);

                let request: Value = serde_json::from_str(&text)?;

                // Handle eth_subscribe request
                if request["method"] == "eth_subscribe" {
                    // Send subscription confirmation
                    let response = json!({
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": "0xtest-subscription-id"
                    });

                    write.send(Message::Text(response.to_string())).await?;
                    debug!("Sent subscription response");

                    // Send a mock block notification after a delay (only once)
                    if !block_sent {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                        let block_notification = json!({
                            "jsonrpc": "2.0",
                            "method": "eth_subscription",
                            "params": {
                                "subscription": "0xtest-subscription-id",
                                "result": {
                                    "number": "0x112a880",  // 18000000
                                    "hash": "0xabc123def456789abc123def456789abc123def456789abc123def456789abc1",
                                    "timestamp": "0x6543210f",
                                    "parentHash": "0xdef456abc123def456789abc123def456789abc123def456789abc123def456",
                                    "miner": "0x1234567890123456789012345678901234567890"
                                }
                            }
                        });

                        write
                            .send(Message::Text(block_notification.to_string()))
                            .await?;
                        debug!("Sent block notification");
                        block_sent = true;
                    }
                }
                // Handle eth_getBlockByNumber request
                else if request["method"] == "eth_getBlockByNumber" {
                    // Generate unique transaction hash using timestamp (must fit in 66 chars: 0x + 64 hex)
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos();
                    let tx_hash = format!("0x{timestamp:064x}");
                    let block_with_txs = json!({
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": {
                            "number": "0x112a880",
                            "hash": "0xabc123def456789abc123def456789abc123def456789abc123def456789abc1",
                            "timestamp": "0x6543210f",
                            "parentHash": "0xdef456abc123def456789abc123def456789abc123def456789abc123def456",
                            "miner": "0x1234567890123456789012345678901234567890",
                            "transactions": [
                                {
                                    "hash": tx_hash,
                                    "from": "0x742d35cc6634c0532925a3b844bc9e7595f0beb0",
                                    "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",  // USDC contract
                                    "value": "0x0",
                                    "gas": "0x5208",
                                    "gasPrice": "0x3b9aca00",
                                    "input": "0xa9059cbb",  // transfer function signature
                                    "nonce": "0x1",
                                    "transactionIndex": "0x0"
                                }
                            ]
                        }
                    });

                    write
                        .send(Message::Text(block_with_txs.to_string()))
                        .await?;
                    debug!("Sent block with transactions (tx: {})", tx_hash);
                }
                // Handle eth_getTransactionReceipt request
                else if request["method"] == "eth_getTransactionReceipt" {
                    let receipt = json!({
                        "jsonrpc": "2.0",
                        "id": request["id"],
                        "result": {
                            "transactionHash": request["params"][0],
                            "blockNumber": "0x112a880",
                            "blockHash": "0xabc123def456789abc123def456789abc123def456789abc123def456789abc1",
                            "status": "0x1",  // Success
                            "logs": [
                                {
                                    "address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",  // USDC
                                    "topics": [
                                        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"  // Transfer
                                    ],
                                    "data": "0x0000000000000000000000000000000000000000000000000000000000000064",
                                    "blockNumber": "0x112a880",
                                    "transactionHash": request["params"][0],
                                    "logIndex": "0x0",
                                    "transactionIndex": "0x0",
                                    "removed": false
                                }
                            ]
                        }
                    });

                    write.send(Message::Text(receipt.to_string())).await?;
                    debug!("Sent transaction receipt for {}", request["params"][0]);
                }
            }
        }

        debug!("WebSocket connection closed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_starts() {
        let server = MockEthRpcServer::start().await.unwrap();
        assert!(server.url().starts_with("ws://127.0.0.1:"));
        server.shutdown();
    }
}
