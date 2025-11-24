use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ClickHouse client wrapper for events and deliveries
#[derive(Clone)]
pub struct ClickHouseClient {
    client: Client,
}

impl ClickHouseClient {
    /// Create a new ClickHouse client from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let url =
            std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string());
        let user = std::env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".to_string());
        let password = std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_else(|_| "".to_string());
        let database =
            std::env::var("CLICKHOUSE_DATABASE").unwrap_or_else(|_| "ethhook".to_string());

        let client = Client::default()
            .with_url(&url)
            .with_user(&user)
            .with_password(&password)
            .with_database(&database);

        Ok(Self { client })
    }

    /// Get the underlying ClickHouse client
    pub fn client(&self) -> &Client {
        &self.client
    }
}

/// Event row for ClickHouse insertion
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct EventRow {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub chain_id: u32,
    pub block_number: u64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub log_index: u32,
    pub contract_address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub ingested_at: i64, // Unix timestamp milliseconds
    pub processed_at: Option<i64>,
}

/// Delivery attempt row for ClickHouse insertion
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct DeliveryAttemptRow {
    pub id: Uuid,
    pub event_id: Uuid,
    pub endpoint_id: Uuid,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub attempt_number: u8,
    pub status: String,
    pub http_status: u16,
    pub response_body: String,
    pub error_message: String,
    pub attempted_at: i64, // Unix timestamp milliseconds
    pub duration_ms: u32,
    pub webhook_url: String,
}

/// Event query result from ClickHouse
#[derive(Debug, Clone, Deserialize, Row)]
pub struct EventQueryRow {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub chain_id: u32,
    pub block_number: u64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub log_index: u32,
    pub contract_address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub ingested_at: i64,
    pub processed_at: Option<i64>,
}

/// Delivery attempt query result from ClickHouse
#[derive(Debug, Clone, Deserialize, Row)]
pub struct DeliveryAttemptQueryRow {
    pub id: Uuid,
    pub event_id: Uuid,
    pub endpoint_id: Uuid,
    pub application_id: Uuid,
    pub user_id: Uuid,
    pub attempt_number: u8,
    pub status: String,
    pub http_status: u16,
    pub response_body: String,
    pub error_message: String,
    pub attempted_at: i64,
    pub duration_ms: u32,
    pub webhook_url: String,
}
