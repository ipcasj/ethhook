/*!
 * Redis Stream Consumer
 *
 * Consumes events from Redis Streams using consumer groups for horizontal scaling.
 *
 * ## Consumer Groups Architecture
 *
 * ```text
 * Stream: events:1
 *    │
 *    ├──> Consumer Group "message_processors"
 *         │
 *         ├──> processor-1 (this instance)
 *         ├──> processor-2 (another pod/instance)
 *         └──> processor-3 (another pod/instance)
 * ```
 *
 * Each consumer gets a different subset of messages automatically!
 *
 * ## Commands Used
 *
 * - **XGROUP CREATE**: Create consumer group (idempotent)
 * - **XREADGROUP**: Read messages for this consumer
 * - **XACK**: Acknowledge processed messages
 * - **XPENDING**: Check for unprocessed messages
 */

use anyhow::{Context, Result};
use redis::RedisError;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// Processed event from Redis Stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
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

/// Stream entry with ID and data
#[derive(Debug, Clone)]
pub struct StreamEntry {
    pub id: String,
    pub event: StreamEvent,
}

/// Redis Stream consumer with consumer group support
pub struct StreamConsumer {
    /// Redis connection manager
    client: redis::aio::ConnectionManager,

    /// Consumer group name
    group_name: String,

    /// Consumer name (unique per instance)
    consumer_name: String,
}

impl StreamConsumer {
    /// Create new stream consumer
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL
    /// * `group_name` - Consumer group name (e.g., "message_processors")
    /// * `consumer_name` - Unique consumer name (e.g., "processor-1")
    pub async fn new(redis_url: &str, group_name: &str, consumer_name: &str) -> Result<Self> {
        info!(
            "Connecting to Redis at {} (consumer: {})",
            redis_url, consumer_name
        );

        let client = redis::Client::open(redis_url).context("Failed to create Redis client")?;

        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .context("Failed to connect to Redis")?;

        info!("✅ Connected to Redis successfully");

        Ok(Self {
            client: conn,
            group_name: group_name.to_string(),
            consumer_name: consumer_name.to_string(),
        })
    }

    /// Ensure consumer group exists for a stream
    ///
    /// Creates the consumer group if it doesn't exist.
    /// Idempotent - safe to call multiple times.
    ///
    /// # Arguments
    ///
    /// * `stream_name` - Stream name (e.g., "events:1")
    pub async fn ensure_consumer_group(&mut self, stream_name: &str) -> Result<()> {
        // XGROUP CREATE stream_name group_name $ MKSTREAM
        // $ = start reading from new messages only
        // MKSTREAM = create stream if it doesn't exist
        let result: Result<String, RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(stream_name)
            .arg(&self.group_name)
            .arg("$") // Start from latest message
            .arg("MKSTREAM") // Create stream if doesn't exist
            .query_async(&mut self.client)
            .await;

        match result {
            Ok(_) => {
                info!(
                    "✅ Created consumer group '{}' for stream '{}'",
                    self.group_name, stream_name
                );
                Ok(())
            }
            Err(e) => {
                // BUSYGROUP error means group already exists - this is OK!
                if e.to_string().contains("BUSYGROUP") {
                    debug!(
                        "Consumer group '{}' already exists for stream '{}'",
                        self.group_name, stream_name
                    );
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Failed to create consumer group: {e}"))
                }
            }
        }
    }

    /// Read events from stream using consumer group
    ///
    /// Uses XREADGROUP to read messages assigned to this consumer.
    /// Messages are automatically distributed across consumers.
    ///
    /// # Arguments
    ///
    /// * `stream_name` - Stream to read from (e.g., "events:1")
    /// * `count` - Maximum number of messages to read
    /// * `block_ms` - How long to block waiting (0 = wait forever)
    ///
    /// # Returns
    ///
    /// Vector of stream entries with IDs and parsed events
    pub async fn read_events(
        &mut self,
        stream_name: &str,
        count: usize,
        block_ms: usize,
    ) -> Result<Vec<StreamEntry>> {
        // XREADGROUP GROUP group_name consumer_name BLOCK block_ms COUNT count STREAMS stream_name >
        // > = read only new messages not yet delivered to any consumer

        debug!(
            "[{}] Starting XREADGROUP: group={}, consumer={}, block={}ms, count={}",
            stream_name, self.group_name, self.consumer_name, block_ms, count
        );

        // Use redis::Value for flexible parsing (matches integration test approach)
        let response: redis::Value = redis::cmd("XREADGROUP")
            .arg("GROUP")
            .arg(&self.group_name)
            .arg(&self.consumer_name)
            .arg("BLOCK")
            .arg(block_ms)
            .arg("COUNT")
            .arg(count)
            .arg("STREAMS")
            .arg(stream_name)
            .arg(">") // Read new messages only
            .query_async(&mut self.client)
            .await
            .map_err(|e| {
                error!(
                    "XREADGROUP failed for stream '{}': {} (group: {}, consumer: {}, block: {}ms, count: {})",
                    stream_name, e, self.group_name, self.consumer_name, block_ms, count
                );
                anyhow::anyhow!("Failed to read from stream '{stream_name}': {e}")
            })?;

        let mut entries = Vec::new();

        // Parse response: XREADGROUP returns Bulk([Bulk([Data(stream_name), Bulk([entries...])])])
        if let redis::Value::Bulk(streams) = response {
            debug!(
                "[{}] XREADGROUP returned {} streams",
                stream_name,
                streams.len()
            );

            for stream_data in &streams {
                if let redis::Value::Bulk(stream_parts) = stream_data {
                    // stream_parts[0] = stream name (Data)
                    // stream_parts[1] = entries (Bulk)
                    if stream_parts.len() < 2 {
                        continue;
                    }

                    if let redis::Value::Bulk(messages) = &stream_parts[1] {
                        // Each message is Bulk([Data(id), Bulk([Data(key1), Data(val1), ...])])
                        for message in messages {
                            if let redis::Value::Bulk(entry_parts) = message {
                                if entry_parts.len() < 2 {
                                    continue;
                                }

                                // Extract message ID
                                let id = if let redis::Value::Data(id_bytes) = &entry_parts[0] {
                                    String::from_utf8_lossy(id_bytes).to_string()
                                } else {
                                    continue;
                                };

                                // Extract fields (key-value pairs)
                                let mut fields = Vec::new();
                                if let redis::Value::Bulk(field_data) = &entry_parts[1] {
                                    // Fields are alternating: key1, val1, key2, val2, ...
                                    for chunk in field_data.chunks(2) {
                                        if chunk.len() == 2 {
                                            let key = if let redis::Value::Data(k) = &chunk[0] {
                                                String::from_utf8_lossy(k).to_string()
                                            } else {
                                                continue;
                                            };

                                            let val = if let redis::Value::Data(v) = &chunk[1] {
                                                String::from_utf8_lossy(v).to_string()
                                            } else {
                                                continue;
                                            };

                                            fields.push((key, val));
                                        }
                                    }
                                }

                                // Parse fields into StreamEvent
                                let event = Self::parse_stream_event(&fields)?;
                                entries.push(StreamEntry { id, event });
                            }
                        }
                    }
                }
            }
        }

        debug!("Read {} events from {}", entries.len(), stream_name);

        Ok(entries)
    }

    /// Parse Redis Stream fields into StreamEvent
    fn parse_stream_event(fields: &[(String, String)]) -> Result<StreamEvent> {
        let mut chain_id = None;
        let mut block_number = None;
        let mut block_hash = None;
        let mut transaction_hash = None;
        let mut log_index = None;
        let mut contract_address = None;
        let mut topics = None;
        let mut data = None;
        let mut timestamp = None;

        for (key, value) in fields {
            match key.as_str() {
                "chain_id" => chain_id = Some(value.parse::<u64>().context("Invalid chain_id")?),
                "block_number" => {
                    block_number = Some(value.parse::<u64>().context("Invalid block_number")?)
                }
                "block_hash" => block_hash = Some(value.clone()),
                "tx_hash" => transaction_hash = Some(value.clone()),
                "log_index" => log_index = Some(value.parse::<u32>().context("Invalid log_index")?),
                "contract" => contract_address = Some(value.clone()),
                "topics" => {
                    // Parse JSON array of topics
                    topics =
                        Some(serde_json::from_str(value).context("Failed to parse topics JSON")?);
                }
                "data" => data = Some(value.clone()),
                "timestamp" => timestamp = Some(value.parse::<i64>().context("Invalid timestamp")?),
                _ => {
                    warn!("Unknown field in stream: {}", key);
                }
            }
        }

        Ok(StreamEvent {
            chain_id: chain_id.context("Missing chain_id")?,
            block_number: block_number.context("Missing block_number")?,
            block_hash: block_hash.context("Missing block_hash")?,
            transaction_hash: transaction_hash.context("Missing transaction_hash")?,
            log_index: log_index.context("Missing log_index")?,
            contract_address: contract_address.context("Missing contract_address")?,
            topics: topics.context("Missing topics")?,
            data: data.context("Missing data")?,
            timestamp: timestamp.context("Missing timestamp")?,
        })
    }

    /// Acknowledge processed messages
    ///
    /// Removes messages from pending entry list (PEL).
    /// Should be called after successfully processing messages.
    ///
    /// # Arguments
    ///
    /// * `stream_name` - Stream name
    /// * `message_ids` - IDs of messages to acknowledge
    pub async fn ack_messages(&mut self, stream_name: &str, message_ids: &[String]) -> Result<()> {
        if message_ids.is_empty() {
            debug!("[{}] No messages to acknowledge (empty list)", stream_name);
            return Ok(());
        }

        info!(
            "[{}] Acknowledging {} messages...",
            stream_name,
            message_ids.len()
        );

        // XACK stream_name group_name id1 id2 id3...
        let mut cmd = redis::cmd("XACK");
        cmd.arg(stream_name).arg(&self.group_name);

        for id in message_ids {
            cmd.arg(id);
        }

        let acked: usize = cmd
            .query_async(&mut self.client)
            .await
            .context("Failed to acknowledge messages")?;

        info!(
            "[{}] ✅ Acknowledged {} messages (expected {})",
            stream_name,
            acked,
            message_ids.len()
        );

        Ok(())
    }

    /// Get pending messages count
    ///
    /// Returns number of messages that were delivered but not acknowledged.
    /// Useful for monitoring stuck consumers.
    #[allow(dead_code)]
    pub async fn pending_count(&mut self, stream_name: &str) -> Result<usize> {
        // XPENDING stream_name group_name

        // Type alias for complex Redis XPENDING response
        type XPendingResult = (usize, Option<String>, Option<String>, Vec<(String, usize)>);

        let result: XPendingResult = redis::cmd("XPENDING")
            .arg(stream_name)
            .arg(&self.group_name)
            .query_async(&mut self.client)
            .await
            .context("Failed to get pending count")?;

        Ok(result.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_consumer_creation() {
        let consumer = StreamConsumer::new("redis://localhost:6379", "test_group", "test_consumer")
            .await
            .unwrap();

        assert_eq!(consumer.group_name, "test_group");
        assert_eq!(consumer.consumer_name, "test_consumer");
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_ensure_consumer_group() {
        let mut consumer = StreamConsumer::new(
            "redis://localhost:6379",
            "test_group_create",
            "test_consumer",
        )
        .await
        .unwrap();

        // Should succeed on first call
        consumer.ensure_consumer_group("test_stream").await.unwrap();

        // Should succeed on second call (idempotent)
        consumer.ensure_consumer_group("test_stream").await.unwrap();
    }
}
