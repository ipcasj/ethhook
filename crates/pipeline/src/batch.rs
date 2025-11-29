use anyhow::Result;
use clickhouse::Client;
use ethhook_domain::event::BlockchainEvent;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

const BATCH_SIZE: usize = 100;
const BATCH_TIMEOUT_SECS: u64 = 5;

pub async fn start_processor(
    mut event_rx: mpsc::Receiver<BlockchainEvent>,
    shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    info!("Starting batch processor");

    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://clickhouse:8123".to_string());
    
    let clickhouse_db = std::env::var("CLICKHOUSE_DB")
        .unwrap_or_else(|_| "ethhook".to_string());
    
    info!("Connecting to ClickHouse: {}/{}", clickhouse_url, clickhouse_db);

    let client = Client::default()
        .with_url(&clickhouse_url)
        .with_database(&clickhouse_db);

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

async fn flush_batch(client: &Client, batch: &mut Vec<BlockchainEvent>) -> Result<()> {
    let count = batch.len();
    
    info!("Flushing batch of {} events to ClickHouse", count);

    // Insert batch into ClickHouse
    let mut insert = client.insert("events")?;
    
    for event in batch.drain(..) {
        insert.write(&event).await?;
    }
    
    insert.end().await?;
    
    info!("Successfully inserted {} events", count);
    Ok(())
}
