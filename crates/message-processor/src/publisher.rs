#![allow(dead_code)]
/*!
 * Delivery Job Publisher
 *
 * Publishes webhook delivery jobs to Redis Queue for Webhook Delivery service.
 *
 * ## Architecture
 *
 * ```text
 * Message Processor          Redis Queue          Webhook Delivery
 * ─────────────────         ───────────          ────────────────
 *         │                       │                       │
 *         ├─ LPUSH delivery_queue │                       │
 *         │  {                    │                       │
 *         │    endpoint_id,       │                       │
 *         │    event_data,        │                       │
 *         │    attempt: 1         │                       │
 *         │  }                    │                       │
 *         │                       │                       │
 *         │                       │<──────────────────────┤
 *         │                       │  BRPOP delivery_queue │
 *         │                       │  timeout=5            │
 *         │                       │                       │
 *         │                       ├───────────────────────>
 *         │                       │  Return job           │
 * ```
 *
 * ## Job Format
 *
 * ```json
 * {
 *   "endpoint_id": "550e8400-e29b-41d4-a716-446655440000",
 *   "url": "https://example.com/webhook",
 *   "hmac_secret": "secret123",
 *   "event": {
 *     "chain_id": 1,
 *     "block_number": 18000000,
 *     "transaction_hash": "0xabc...",
 *     "contract_address": "0xA0b...",
 *     "topics": ["0xddf..."],
 *     "data": "0x...",
 *     "timestamp": 1696800000
 *   },
 *   "attempt": 1,
 *   "max_retries": 5,
 *   "timeout_seconds": 30
 * }
 * ```
 *
 * ## Why Redis Queue vs Stream?
 *
 * - **Queue (LIST)**: FIFO, blocking pop, simpler for worker pattern
 * - **Stream**: Ordered log, consumer groups, replay capability
 *
 * We use Queue here because:
 * 1. Simple worker pool pattern (Webhook Delivery workers)
 * 2. Don't need replay (failed jobs handled via retries)
 * 3. BRPOP is simpler than XREADGROUP
 */

use anyhow::{Context, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use crate::consumer::StreamEvent;
use crate::matcher::MatchedEndpoint;

/// Delivery job for webhook delivery service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryJob {
    /// Endpoint UUID
    pub endpoint_id: Uuid,

    /// Application UUID
    pub application_id: Uuid,

    /// Webhook URL
    pub url: String,

    /// HMAC secret for signature
    pub hmac_secret: String,

    /// Blockchain event data
    pub event: StreamEvent,

    /// Current attempt number
    pub attempt: u32,

    /// Maximum retry attempts
    pub max_retries: i32,

    /// HTTP timeout in seconds
    pub timeout_seconds: i32,

    /// Rate limit (requests per second)
    pub rate_limit_per_second: i32,
}

/// Redis Queue publisher for delivery jobs
pub struct DeliveryPublisher {
    /// Redis connection manager
    client: redis::aio::ConnectionManager,

    /// Queue name
    queue_name: String,
}

impl DeliveryPublisher {
    /// Create new delivery publisher
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL
    /// * `queue_name` - Queue name (default: "delivery_queue")
    pub async fn new(redis_url: &str, queue_name: &str) -> Result<Self> {
        info!(
            "Connecting to Redis for delivery publishing at {}",
            redis_url
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

    /// Publish delivery job to queue
    ///
    /// Uses LPUSH to add job to the left side of the queue.
    /// Webhook Delivery service uses BRPOP to read from right side (FIFO).
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Matched endpoint configuration
    /// * `event` - Blockchain event to deliver
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Job published successfully
    /// * `Err(_)` - Redis connection or serialization error
    pub async fn publish(&mut self, endpoint: &MatchedEndpoint, event: &StreamEvent) -> Result<()> {
        let job = DeliveryJob {
            endpoint_id: endpoint.endpoint_id,
            application_id: endpoint.application_id,
            url: endpoint.url.clone(),
            hmac_secret: endpoint.hmac_secret.clone(),
            event: event.clone(),
            attempt: 1,
            max_retries: endpoint.max_retries,
            timeout_seconds: endpoint.timeout_seconds,
            rate_limit_per_second: endpoint.rate_limit_per_second,
        };

        // Serialize job to JSON
        let job_json = serde_json::to_string(&job).context("Failed to serialize delivery job")?;

        // LPUSH queue_name job_json
        let _: () = self
            .client
            .lpush(&self.queue_name, &job_json)
            .await
            .context("Failed to push job to queue")?;

        debug!(
            "Published delivery job: endpoint={} event={}",
            endpoint.endpoint_id, event.transaction_hash
        );

        Ok(())
    }

    /// Publish multiple jobs in a pipeline (batch operation)
    ///
    /// More efficient than calling publish() in a loop.
    /// Uses Redis pipelining to send all LPUSH commands at once.
    ///
    /// # Arguments
    ///
    /// * `jobs` - Vector of (endpoint, event) tuples
    ///
    /// # Returns
    ///
    /// Number of jobs published
    #[allow(dead_code)]
    pub async fn publish_batch(
        &mut self,
        jobs: Vec<(&MatchedEndpoint, &StreamEvent)>,
    ) -> Result<usize> {
        if jobs.is_empty() {
            return Ok(0);
        }

        let mut pipe = redis::pipe();

        for (endpoint, event) in &jobs {
            let job = DeliveryJob {
                endpoint_id: endpoint.endpoint_id,
                application_id: endpoint.application_id,
                url: endpoint.url.clone(),
                hmac_secret: endpoint.hmac_secret.clone(),
                event: (*event).clone(),
                attempt: 1,
                max_retries: endpoint.max_retries,
                timeout_seconds: endpoint.timeout_seconds,
                rate_limit_per_second: endpoint.rate_limit_per_second,
            };

            let job_json =
                serde_json::to_string(&job).context("Failed to serialize delivery job")?;

            pipe.lpush(&self.queue_name, &job_json);
        }

        // Execute pipeline
        let _: () = pipe
            .query_async(&mut self.client)
            .await
            .context("Failed to execute pipeline")?;

        let count = jobs.len();
        debug!("Published {} delivery jobs in batch", count);

        Ok(count)
    }

    /// Get queue statistics
    ///
    /// Returns number of jobs waiting in queue.
    pub async fn queue_length(&mut self) -> Result<usize> {
        let length: usize = self
            .client
            .llen(&self.queue_name)
            .await
            .context("Failed to get queue length")?;

        Ok(length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_publisher_creation() {
        let publisher = DeliveryPublisher::new("redis://localhost:6379", "test_delivery_queue")
            .await
            .unwrap();

        assert_eq!(publisher.queue_name, "test_delivery_queue");
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_publish_job() {
        let mut publisher = DeliveryPublisher::new("redis://localhost:6379", "test_delivery_queue")
            .await
            .unwrap();

        let endpoint = MatchedEndpoint {
            endpoint_id: Uuid::new_v4(),
            application_id: Uuid::new_v4(),
            url: "https://example.com/webhook".to_string(),
            hmac_secret: "secret123".to_string(),
            rate_limit_per_second: 10,
            max_retries: 5,
            timeout_seconds: 30,
        };

        let event = StreamEvent {
            chain_id: 1,
            block_number: 18000000,
            block_hash: "0xabc123".to_string(),
            transaction_hash: "0xdef456".to_string(),
            log_index: 5,
            contract_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            topics: vec![
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
            ],
            data: "0x".to_string(),
            timestamp: 1696800000,
        };

        publisher.publish(&endpoint, &event).await.unwrap();

        let length = publisher.queue_length().await.unwrap();
        assert!(length > 0);
    }
}
