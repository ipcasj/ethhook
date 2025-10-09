//! Redis client with Stream and Queue helpers
//!
//! Provides Redis operations with connection pooling.
//! Similar to Java's Jedis/Lettuce but with async support.

use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, RedisResult};
use serde::Serialize;
use tracing::{error, info};

use crate::error::Result;

/// Redis client wrapper with helper methods
///
/// Java equivalent:
/// ```java
/// JedisPool pool = new JedisPool("localhost", 6379);
/// Jedis jedis = pool.getResource();
/// ```
///
/// Rust:
/// ```rust
/// let client = RedisClient::new("redis://localhost:6379").await?;
/// ```
pub struct RedisClient {
    manager: ConnectionManager,
}

impl RedisClient {
    /// Create a new Redis client
    pub async fn new(redis_url: &str) -> Result<Self> {
        info!("Connecting to Redis at {}", redis_url);

        let client = Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;

        info!("Redis connection established");
        Ok(Self { manager })
    }

    /// Ping Redis to check health
    ///
    /// Java equivalent:
    /// ```java
    /// String pong = jedis.ping();
    /// ```
    pub async fn ping(&mut self) -> Result<String> {
        let result: String = redis::cmd("PING").query_async(&mut self.manager).await?;
        Ok(result)
    }

    /// Set a key-value pair
    ///
    /// Java equivalent:
    /// ```java
    /// jedis.set("key", "value");
    /// ```
    pub async fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.manager.set::<_, _, ()>(key, value).await?;
        Ok(())
    }

    /// Get a value by key
    ///
    /// Java equivalent:
    /// ```java
    /// String value = jedis.get("key");
    /// ```
    pub async fn get(&mut self, key: &str) -> Result<Option<String>> {
        let result: Option<String> = self.manager.get(key).await?;
        Ok(result)
    }

    /// Add entry to Redis Stream (for events)
    ///
    /// Java equivalent:
    /// ```java
    /// jedis.xadd("events:raw", "*", Map.of("data", jsonData));
    /// ```
    ///
    /// Rust:
    /// ```rust
    /// client.xadd("events:raw", &event_data).await?;
    /// ```
    pub async fn xadd<T: Serialize>(&mut self, stream: &str, data: &T) -> Result<String> {
        let json = serde_json::to_string(data)?;

        let id: String = redis::cmd("XADD")
            .arg(stream)
            .arg("*") // Auto-generate ID
            .arg("data")
            .arg(json)
            .query_async(&mut self.manager)
            .await?;

        Ok(id)
    }

    /// Read from Redis Stream
    ///
    /// Java equivalent:
    /// ```java
    /// List<Entry> entries = jedis.xread(
    ///     XReadParams.xReadParams().count(10),
    ///     Map.of("events:raw", "0")
    /// );
    /// ```
    pub async fn xread(
        &mut self,
        stream: &str,
        id: &str,
        count: usize,
    ) -> Result<Vec<StreamEntry>> {
        let result: RedisResult<Vec<(String, Vec<(String, Vec<(String, String)>)>)>> =
            redis::cmd("XREAD")
                .arg("COUNT")
                .arg(count)
                .arg("STREAMS")
                .arg(stream)
                .arg(id)
                .query_async(&mut self.manager)
                .await;

        match result {
            Ok(streams) => {
                let mut entries = Vec::new();
                for (_stream_name, stream_entries) in streams {
                    for (entry_id, fields) in stream_entries {
                        if let Some((_key, json_data)) = fields.first() {
                            entries.push(StreamEntry {
                                id: entry_id,
                                data: json_data.clone(),
                            });
                        }
                    }
                }
                Ok(entries)
            }
            Err(e) => {
                error!("Failed to read from stream: {}", e);
                Err(e.into())
            }
        }
    }

    /// Push to queue (list)
    ///
    /// Java equivalent:
    /// ```java
    /// jedis.lpush("queue:webhooks", jsonData);
    /// ```
    pub async fn lpush<T: Serialize>(&mut self, list: &str, data: &T) -> Result<()> {
        let json = serde_json::to_string(data)?;
        self.manager.lpush::<_, _, ()>(list, json).await?;
        Ok(())
    }

    /// Blocking pop from queue
    ///
    /// Java equivalent:
    /// ```java
    /// List<String> result = jedis.brpop(timeout, "queue:webhooks");
    /// ```
    pub async fn brpop(&mut self, list: &str, timeout: usize) -> Result<Option<String>> {
        let result: Option<(String, String)> = self.manager.brpop(list, timeout as f64).await?;

        Ok(result.map(|(_, value)| value))
    }

    /// Publish message to pub/sub channel
    pub async fn publish(&mut self, channel: &str, message: &str) -> Result<()> {
        self.manager.publish::<_, _, ()>(channel, message).await?;
        Ok(())
    }
}

/// Redis Stream entry
#[derive(Debug, Clone)]
pub struct StreamEntry {
    pub id: String,
    pub data: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_connection() {
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            let result = RedisClient::new(&redis_url).await;
            assert!(result.is_ok(), "Failed to connect: {:?}", result.err());

            let mut client = result.unwrap();
            let pong = client.ping().await;
            assert!(pong.is_ok());
            assert_eq!(pong.unwrap(), "PONG");
        } else {
            println!("Skipping test: REDIS_URL not set");
        }
    }

    #[tokio::test]
    async fn test_redis_set_get() {
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            let mut client = RedisClient::new(&redis_url).await.unwrap();

            client.set("test:key", "test_value").await.unwrap();
            let value = client.get("test:key").await.unwrap();

            assert_eq!(value, Some("test_value".to_string()));
        }
    }

    #[derive(Serialize)]
    struct TestEvent {
        id: u64,
        message: String,
    }

    #[tokio::test]
    async fn test_redis_stream() {
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            let mut client = RedisClient::new(&redis_url).await.unwrap();

            let event = TestEvent {
                id: 123,
                message: "test event".to_string(),
            };

            let id = client.xadd("test:stream", &event).await;
            assert!(id.is_ok(), "Failed to add to stream: {:?}", id.err());
        }
    }
}
