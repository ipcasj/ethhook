use anyhow::Result;
use clickhouse::Client;
use ethhook_domain::event::BlockchainEvent;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};

const BATCH_SIZE: usize = 100;
const BATCH_TIMEOUT_SECS: u64 = 5;

pub async fn start_processor(
    mut event_rx: mpsc::Receiver<BlockchainEvent>,
    shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    info!("Starting batch processor");

    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://clickhouse:8123".to_string());
    
    let clickhouse_db = std::env::var("CLICKHOUSE_DATABASE")
        .or_else(|_| std::env::var("CLICKHOUSE_DB"))
        .unwrap_or_else(|_| "ethhook".to_string());
    
    let clickhouse_user = std::env::var("CLICKHOUSE_USER")
        .unwrap_or_else(|_| "default".to_string());
    
    let clickhouse_password = std::env::var("CLICKHOUSE_PASSWORD")
        .unwrap_or_else(|_| String::new());
    
    info!("Connecting to ClickHouse: {}/{} (user: {})", clickhouse_url, clickhouse_db, clickhouse_user);

    let client = Client::default()
        .with_url(&clickhouse_url)
        .with_database(&clickhouse_db)
        .with_user(&clickhouse_user)
        .with_password(&clickhouse_password);

    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let mut shutdown_rx = shutdown;
    let batch_timeout = tokio::time::Duration::from_secs(BATCH_TIMEOUT_SECS);

    loop {
        tokio::select! {
            // Receive events from WebSocket ingestors
            Some(event) = event_rx.recv() => {
                batch.push(event);

                // Flush batch when full
                if batch.len() >= BATCH_SIZE {
                    if let Err(e) = flush_batch(&client, &mut batch).await {
                        error!("Failed to flush batch: {}", e);
                    }
                }
            }

            // Flush partial batches periodically
            _ = tokio::time::sleep(batch_timeout) => {
                if !batch.is_empty() {
                    if let Err(e) = flush_batch(&client, &mut batch).await {
                        error!("Failed to flush batch (timeout): {}", e);
                    }
                }
            }

            // Shutdown signal
            _ = shutdown_rx.recv() => {
                info!("Batch processor shutting down");
                
                // Flush remaining events
                if !batch.is_empty() {
                    info!("Flushing {} remaining events", batch.len());
                    if let Err(e) = flush_batch(&client, &mut batch).await {
                        error!("Failed to flush final batch: {}", e);
                    }
                }
                
                break;
            }
        }
    }

    info!("Batch processor stopped");
    Ok(())
}

async fn flush_batch(_client: &Client, batch: &mut Vec<BlockchainEvent>) -> Result<()> {
    let count = batch.len();
    
    info!("Flushing batch of {} events to ClickHouse", count);

    // Use JSONEachRow format instead of binary to avoid Vec<String> serialization corruption
    // Convert ClickHouse timestamps to string format
    use serde_json::json;
    
    let mut json_lines = Vec::new();
    for event in batch.drain(..) {
        // Manually construct JSON to control timestamp formatting
        let json_obj = json!({
            "id": event.id.to_string(),
            "endpoint_id": event.endpoint_id.map(|u| u.to_string()),
            "application_id": event.application_id.map(|u| u.to_string()),
            "user_id": event.user_id.map(|u| u.to_string()),
            "chain_id": event.chain_id,
            "block_number": event.block_number,
            "block_hash": event.block_hash,
            "transaction_hash": event.transaction_hash,
            "log_index": event.log_index,
            "contract_address": event.contract_address,
            "topics": event.topics,
            "data": event.data,
            "ingested_at": event.ingested_at.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            "processed_at": event.processed_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
        });
        json_lines.push(serde_json::to_string(&json_obj)?);
    }
    
    let json_data = json_lines.join("\n");
    
    // Execute raw HTTP request with JSON format
    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://clickhouse:8123".to_string());
    let clickhouse_db = std::env::var("CLICKHOUSE_DATABASE")
        .or_else(|_| std::env::var("CLICKHOUSE_DB"))
        .unwrap_or_else(|_| "ethhook".to_string());
    let clickhouse_user = std::env::var("CLICKHOUSE_USER")
        .unwrap_or_else(|_| "default".to_string());
    let clickhouse_password = std::env::var("CLICKHOUSE_PASSWORD")
        .unwrap_or_else(|_| String::new());
    
    let http_client = reqwest::Client::new();
    let insert_url = format!(
        "{}/?database={}&query=INSERT%20INTO%20events%20FORMAT%20JSONEachRow",
        clickhouse_url, clickhouse_db
    );
    
    let response = http_client
        .post(&insert_url)
        .basic_auth(&clickhouse_user, Some(&clickhouse_password))
        .body(json_data)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("ClickHouse insert failed: {}", error_text);
    }
    
    info!("Successfully inserted {} events using JSONEachRow format", count);
    Ok(())
}
