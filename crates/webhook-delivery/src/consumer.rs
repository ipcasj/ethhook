/*!
 * Delivery Job Consumer
 *
 * Consumes delivery jobs from Redis Queue using BRPOP (blocking pop).
 */

use anyhow::{Context, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

/// Blockchain event data (matches StreamEvent from message-processor)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
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

/// Delivery job from message processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryJob {
    pub endpoint_id: Uuid,
    pub application_id: Uuid,
    pub url: String,
    pub hmac_secret: String,
    pub event: EventData,
    pub attempt: u32,
    pub max_retries: i32,
    pub timeout_seconds: i32,
    pub rate_limit_per_second: i32,
}

/// Redis Queue consumer for delivery jobs
pub struct JobConsumer {
    /// Redis connection manager
    client: redis::aio::ConnectionManager,

    /// Queue name to consume from
    queue_name: String,
}

impl JobConsumer {
    /// Create new job consumer
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL
    /// * `queue_name` - Queue name (e.g., "delivery_queue")
    pub async fn new(redis_url: &str, queue_name: &str) -> Result<Self> {
        info!(
            "Connecting to Redis at {} (queue: {})",
            redis_url, queue_name
        );

        let client = redis::Client::open(redis_url).context("Failed to create Redis client")?;

        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .context("Failed to connect to Redis")?;

        info!("✅ Connected to Redis Queue successfully");

        Ok(Self {
            client: conn,
            queue_name: queue_name.to_string(),
        })
    }

    /// Consume next delivery job from queue (blocking)
    ///
    /// Uses BRPOP to block until a job is available.
    /// This is more efficient than polling with RPOP.
    ///
    /// # Arguments
    ///
    /// * `timeout_secs` - How long to block waiting (0 = wait forever)
    ///
    /// # Returns
    ///
    /// * `Ok(Some(job))` - Job received
    /// * `Ok(None)` - Timeout (if timeout > 0)
    /// * `Err(_)` - Redis connection or parsing error
    pub async fn consume(&mut self, timeout_secs: usize) -> Result<Option<DeliveryJob>> {
        // BRPOP queue_name timeout
        // Returns: (queue_name, value) or None if timeout
        let result: Option<(String, String)> = self
            .client
            .brpop(&self.queue_name, timeout_secs as f64)
            .await
            .context("Failed to pop from queue")?;

        match result {
            Some((_queue, job_json)) => {
                // Parse JSON into DeliveryJob
                let job: DeliveryJob =
                    serde_json::from_str(&job_json).context("Failed to parse delivery job JSON")?;

                debug!(
                    "Consumed job: endpoint={} attempt={} url={}",
                    job.endpoint_id,
                    job.attempt,
                    &job.url[..30.min(job.url.len())]
                );

                Ok(Some(job))
            }
            None => {
                // Timeout - no jobs available
                Ok(None)
            }
        }
    }

    /// Get queue length (number of pending jobs)
    /// Reserved for future monitoring/metrics
    #[allow(dead_code)]
    pub async fn queue_length(&mut self) -> Result<usize> {
        let length: usize = self
            .client
            .llen(&self.queue_name)
            .await
            .context("Failed to get queue length")?;

        Ok(length)
    }

    /// Re-queue a job for retry (add to left side of queue)
    ///
    /// Used when delivery fails and should be retried.
    /// Reserved for future manual retry functionality
    #[allow(dead_code)]
    pub async fn requeue(&mut self, mut job: DeliveryJob) -> Result<()> {
        // Increment attempt counter
        job.attempt += 1;

        // Serialize back to JSON
        let job_json =
            serde_json::to_string(&job).context("Failed to serialize job for requeue")?;

        // LPUSH (add to left side, so it gets processed again)
        let _: () = self
            .client
            .lpush(&self.queue_name, &job_json)
            .await
            .context("Failed to requeue job")?;

        debug!(
            "Requeued job: endpoint={} attempt={}",
            job.endpoint_id, job.attempt
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_consumer_creation() {
        let consumer = JobConsumer::new("redis://localhost:6379", "test_delivery_queue")
            .await
            .unwrap();

        assert_eq!(consumer.queue_name, "test_delivery_queue");
    }

    #[tokio::test]
    #[ignore] // Requires Redis with data
    async fn test_consume_timeout() {
        let mut consumer = JobConsumer::new("redis://localhost:6379", "test_delivery_queue_empty")
            .await
            .unwrap();

        // Should timeout after 1 second
        let result = consumer.consume(1).await.unwrap();
        assert!(result.is_none());
    }
}
