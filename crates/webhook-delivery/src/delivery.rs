/*!
 * Webhook Delivery Module
 *
 * Sends HTTP POST requests to customer webhooks with HMAC signatures.
 */

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::consumer::{DeliveryJob, EventData};

/// Webhook delivery result
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub should_retry: bool,
}

/// Webhook delivery service
pub struct WebhookDelivery {
    /// HTTP client with timeout
    client: Client,
}

impl WebhookDelivery {
    /// Create new webhook delivery service
    ///
    /// # Arguments
    ///
    /// * `http_timeout` - Timeout per HTTP request
    pub fn new(http_timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(http_timeout)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client })
    }

    /// Deliver webhook to endpoint
    ///
    /// # Arguments
    ///
    /// * `job` - Delivery job with endpoint URL and event data
    ///
    /// # Returns
    ///
    /// DeliveryResult with success status and details
    pub async fn deliver(&self, job: &DeliveryJob) -> Result<DeliveryResult> {
        let start = Instant::now();

        // Build webhook payload
        let payload = self.build_payload(&job.event);
        let payload_json =
            serde_json::to_string(&payload).context("Failed to serialize payload")?;

        // Calculate HMAC signature
        let signature = ethhook_common::sign_hmac(&payload_json, &job.hmac_secret);

        debug!(
            "Sending webhook to {} (endpoint: {}, attempt: {})",
            &job.url[..30.min(job.url.len())],
            job.endpoint_id,
            job.attempt
        );

        // Send POST request
        let response_result = self
            .client
            .post(&job.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", signature)
            .header("X-Webhook-Id", job.endpoint_id.to_string())
            .header("X-Webhook-Attempt", job.attempt.to_string())
            .body(payload_json)
            .send()
            .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Process response
        match response_result {
            Ok(response) => {
                let status = response.status();
                let status_code = status.as_u16();

                // Read response body (limit to 10KB)
                let body_result = response.text().await;
                let response_body = body_result
                    .ok()
                    .map(|b| b.chars().take(10000).collect::<String>());

                let success = status.is_success();
                let should_retry = !success && crate::retry::is_retryable_error(Some(status_code));

                if success {
                    info!(
                        "✅ Webhook delivered successfully: endpoint={} status={} duration={}ms",
                        job.endpoint_id, status_code, duration_ms
                    );
                } else {
                    warn!(
                        "⚠️  Webhook delivery failed: endpoint={} status={} duration={}ms retry={}",
                        job.endpoint_id, status_code, duration_ms, should_retry
                    );
                }

                Ok(DeliveryResult {
                    success,
                    status_code: Some(status_code),
                    response_body,
                    error_message: None,
                    duration_ms,
                    should_retry,
                })
            }
            Err(e) => {
                let error_message = e.to_string();

                // Determine if error is retryable
                let should_retry = if e.is_timeout() || e.is_connect() {
                    true // Network errors - retry
                } else if e.is_status() {
                    // Status code error - check if retryable
                    e.status()
                        .map(|s| crate::retry::is_retryable_error(Some(s.as_u16())))
                        .unwrap_or(true)
                } else {
                    true // Unknown error - retry to be safe
                };

                error!(
                    "❌ Webhook delivery error: endpoint={} error={} duration={}ms retry={}",
                    job.endpoint_id, error_message, duration_ms, should_retry
                );

                Ok(DeliveryResult {
                    success: false,
                    status_code: None,
                    response_body: None,
                    error_message: Some(error_message),
                    duration_ms,
                    should_retry,
                })
            }
        }
    }

    /// Build webhook payload from event data
    fn build_payload(&self, event: &EventData) -> serde_json::Value {
        json!({
            "chain_id": event.chain_id,
            "block_number": event.block_number,
            "block_hash": event.block_hash,
            "transaction_hash": event.transaction_hash,
            "log_index": event.log_index,
            "contract_address": event.contract_address,
            "topics": event.topics,
            "data": event.data,
            "timestamp": event.timestamp,
        })
    }
}

/// Log delivery attempt to database
pub async fn log_delivery_attempt(
    pool: &sqlx::PgPool,
    endpoint_id: Uuid,
    _event_tx_hash: &str,
    _event_log_index: u32,
    result: &DeliveryResult,
) -> Result<()> {
    // Note: We're not storing event in database for this demo
    // In production, you might want to:
    // 1. Insert event into events table (if not exists)
    // 2. Insert delivery_attempt record
    // For now, just log to database without foreign key constraint

    sqlx::query(
        r#"
        INSERT INTO delivery_attempts (
            endpoint_id,
            attempt_number,
            http_status_code,
            response_body,
            error_message,
            duration_ms,
            success,
            should_retry
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(endpoint_id)
    .bind(1) // We don't track attempt number in DB for simplicity
    .bind(result.status_code.map(|c| c as i32))
    .bind(&result.response_body)
    .bind(&result.error_message)
    .bind(result.duration_ms as i32)
    .bind(result.success)
    .bind(result.should_retry)
    .execute(pool)
    .await
    .context("Failed to log delivery attempt")?;

    debug!(
        "Logged delivery attempt: endpoint={} success={}",
        endpoint_id, result.success
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webhook_delivery_creation() {
        // Verify the delivery instance can be created successfully
        let result = WebhookDelivery::new(Duration::from_secs(30));
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_payload() {
        let delivery = WebhookDelivery::new(Duration::from_secs(30)).unwrap();

        let event = EventData {
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

        let payload = delivery.build_payload(&event);

        assert_eq!(payload["chain_id"], 1);
        assert_eq!(payload["block_number"], 18000000);
        assert_eq!(payload["transaction_hash"], "0xdef456");
    }
}
