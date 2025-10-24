/*!
 * Type Definitions
 *
 * Data structures for blockchain events and RPC responses.
 * These map to Ethereum JSON-RPC response formats.
 */

use serde::{Deserialize, Serialize};

/// A blockchain block header
///
/// Example JSON from Alchemy:
/// ```json
/// {
///   "number": "0x112a880",
///   "hash": "0xabc123...",
///   "timestamp": "0x65f12a80",
///   "miner": "0x...",
///   ...
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    /// Block number (hex string like "0x112a880")
    pub number: String,

    /// Block hash (hex string like "0xabc123...")
    pub hash: String,

    /// Unix timestamp (hex string)
    pub timestamp: String,

    /// Miner address
    pub miner: Option<String>,

    /// Parent block hash
    pub parent_hash: String,
}

/// A transaction log (emitted event)
///
/// Example: USDC Transfer event
/// ```json
/// {
///   "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
///   "topics": [
///     "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
///     "0x000000000000000000000000742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
///     "0x000000000000000000000000d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
///   ],
///   "data": "0x0000000000000000000000000000000000000000000000000000000000989680",
///   "blockNumber": "0x112a880",
///   "transactionHash": "0xdef456...",
///   "logIndex": "0x5"
/// }
/// ```
///
/// This represents:
/// - Event: Transfer (topic[0] is the event signature hash)
/// - From: 0x742d... (topic[1])
/// - To: 0xd8dA... (topic[2])
/// - Value: 10000000 (data field, which is 10 USDC with 6 decimals)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    /// Smart contract address that emitted the event
    pub address: String,

    /// Indexed event parameters (up to 3, plus event signature as topic[0])
    pub topics: Vec<String>,

    /// Non-indexed event parameters (ABI-encoded)
    pub data: String,

    /// Block number where this log was emitted
    pub block_number: String,

    /// Transaction hash that produced this log
    pub transaction_hash: String,

    /// Log index within the block (for uniqueness)
    pub log_index: String,

    /// Transaction index within the block
    pub transaction_index: Option<String>,

    /// Whether this log was removed (due to chain reorg)
    #[serde(default)]
    pub removed: bool,

    /// Capture any additional fields without failing
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: serde_json::Value,
}

/// RPC subscription message for newHeads
///
/// When we subscribe via WebSocket:
/// ```json
/// {"jsonrpc":"2.0","method":"eth_subscribe","params":["newHeads"]}
/// ```
///
/// We receive messages like:
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "eth_subscription",
///   "params": {
///     "subscription": "0xabc123",
///     "result": { ...block header... }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionMessage {
    pub jsonrpc: String,
    pub method: String,
    pub params: SubscriptionParams,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionParams {
    pub subscription: String,
    pub result: Block,
}

/// RPC request to get block with full transaction details
///
/// Request:
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "eth_getBlockByNumber",
///   "params": ["0x112a880", true],
///   "id": 1
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)] // RPC request type for future use
pub struct GetBlockRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: (String, bool),
    pub id: u64,
}

/// RPC response for eth_getBlockByNumber
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // JSON-RPC response fields
pub struct BlockResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<BlockWithTransactions>,
}

/// Block with full transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockWithTransactions {
    pub number: String,
    pub hash: String,
    pub timestamp: String,
    pub transactions: Vec<Transaction>,
}

/// A transaction within a block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas: String,
    pub gas_price: Option<String>,
    pub input: String,
    pub nonce: String,
    pub transaction_index: Option<String>,
}

/// RPC request to get transaction receipt (includes logs)
///
/// Request:
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "eth_getTransactionReceipt",
///   "params": ["0xdef456..."],
///   "id": 1
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)] // RPC request type for future use
pub struct GetReceiptRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
}

/// RPC response for eth_getTransactionReceipt
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // JSON-RPC response fields
pub struct ReceiptResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<TransactionReceipt>,
}

/// Transaction receipt with logs (events)
///
/// This struct is lenient - it only requires the fields we actually use (logs).
/// All other fields from the RPC response are ignored via #[serde(flatten)].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    pub transaction_hash: Option<String>,
    pub block_number: Option<String>,
    pub block_hash: Option<String>,
    pub logs: Vec<Log>,
    pub status: Option<String>,

    // Capture all other fields without failing
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: serde_json::Value,
}

/// Processed event ready for Redis Stream
///
/// This is our internal format after parsing blockchain logs.
/// We'll serialize this to JSON and publish to Redis Stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedEvent {
    /// Chain ID (1 = Ethereum, 42161 = Arbitrum, etc.)
    pub chain_id: u64,

    /// Block number (decimal, not hex)
    pub block_number: u64,

    /// Block hash
    pub block_hash: String,

    /// Transaction hash
    pub transaction_hash: String,

    /// Log index within the transaction
    pub log_index: u64,

    /// Smart contract address that emitted the event
    pub contract_address: String,

    /// Event topics (indexed parameters)
    pub topics: Vec<String>,

    /// Event data (non-indexed parameters)
    pub data: String,

    /// Unix timestamp when the block was mined
    pub timestamp: u64,
}

impl ProcessedEvent {
    /// Generate unique event ID for deduplication
    ///
    /// Format: "event:{chain_id}:{tx_hash}:{log_index}"
    /// Example: "event:1:0xabc123...:5"
    ///
    /// This ID is unique across:
    /// - Chains (different chain_id)
    /// - Transactions (different tx_hash)
    /// - Logs within transaction (different log_index)
    pub fn event_id(&self) -> String {
        format!(
            "event:{}:{}:{}",
            self.chain_id, self.transaction_hash, self.log_index
        )
    }

    /// Get Redis Stream name for this event
    ///
    /// Format: "events:{chain_id}"
    /// Examples:
    /// - "events:1" (Ethereum)
    /// - "events:42161" (Arbitrum)
    /// - "events:10" (Optimism)
    /// - "events:8453" (Base)
    pub fn stream_name(&self) -> String {
        format!("events:{}", self.chain_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_generation() {
        let event = ProcessedEvent {
            chain_id: 1,
            block_number: 18000000,
            block_hash: "0xabc".to_string(),
            transaction_hash: "0xdef456".to_string(),
            log_index: 5,
            contract_address: "0x123".to_string(),
            topics: vec![],
            data: "0x".to_string(),
            timestamp: 1234567890,
        };

        assert_eq!(event.event_id(), "event:1:0xdef456:5");
    }

    #[test]
    fn test_stream_name_generation() {
        let event = ProcessedEvent {
            chain_id: 42161,
            block_number: 100,
            block_hash: "0x".to_string(),
            transaction_hash: "0x".to_string(),
            log_index: 0,
            contract_address: "0x".to_string(),
            topics: vec![],
            data: "0x".to_string(),
            timestamp: 0,
        };

        assert_eq!(event.stream_name(), "events:42161");
    }
}
