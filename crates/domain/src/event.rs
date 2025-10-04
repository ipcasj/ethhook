use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockchainEvent {
    pub id: Uuid,
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub log_index: i32,
    pub contract_address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub ingested_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub created_at: DateTime<Utc>,
    pub data: EventData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventData {
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub log_index: i32,
    pub contract_address: String,
    pub topics: Vec<String>,
    pub data: String,
}

impl BlockchainEvent {
    pub fn to_webhook_payload(&self) -> WebhookPayload {
        WebhookPayload {
            id: format!("evt_{}", self.id.simple()),
            event_type: "ethereum.log".to_string(),
            created_at: self.ingested_at,
            data: EventData {
                block_number: self.block_number,
                block_hash: self.block_hash.clone(),
                transaction_hash: self.transaction_hash.clone(),
                log_index: self.log_index,
                contract_address: self.contract_address.clone(),
                topics: self.topics.clone(),
                data: self.data.clone(),
            },
        }
    }
}
