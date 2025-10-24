/*!
 * Shared Types
 *
 * Common data structures used across multiple services.
 */

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Blockchain event data
///
/// This represents a processed event from the blockchain.
/// Used by Message Processor and Webhook Delivery services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainEvent {
    pub chain_id: u64,
    pub block_number: u64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub log_index: u32,
    pub contract_address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub timestamp: i64,
}

/// Delivery job for webhook delivery
///
/// Published by Message Processor to Redis Queue.
/// Consumed by Webhook Delivery workers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryJob {
    pub endpoint_id: Uuid,
    pub application_id: Uuid,
    pub url: String,
    pub hmac_secret: String,
    pub event: BlockchainEvent,
    pub attempt: u32,
    pub max_retries: i32,
    pub timeout_seconds: i32,
    pub rate_limit_per_second: i32,
}
