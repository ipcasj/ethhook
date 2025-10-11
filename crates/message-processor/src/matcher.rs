/*!
 * Endpoint Matcher
 *
 * Queries PostgreSQL to find webhook endpoints that match blockchain events.
 *
 * ## Matching Logic
 *
 * An endpoint matches an event if:
 * 1. **Active**: endpoint.is_active = true
 * 2. **Contract Match**: endpoint.contract_address = event.contract_address (or NULL for all)
 * 3. **Topic Match**: endpoint.event_topics ⊆ event.topics (or NULL for all)
 *
 * ## Query Performance
 *
 * Indexes used:
 * - `idx_endpoints_contract_address` (B-tree on contract_address WHERE is_active)
 * - `idx_endpoints_event_topics` (GIN on event_topics WHERE is_active)
 *
 * Expected query time: < 5ms for 10,000 endpoints
 *
 * ## Example
 *
 * Event:
 * - contract: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 (USDC)
 * - topics: ["0xddf252ad...", "0x000...alice", "0x000...bob"]
 *
 * Endpoint matches if:
 * - contract_address = 0xA0b... (specific) OR NULL (all contracts)
 * - event_topics = ["0xddf252ad..."] (Transfer signature) OR NULL (all events)
 */

use anyhow::{Context, Result};
use sqlx::{PgPool, Row};
use tracing::{debug, info};
use uuid::Uuid;

use crate::consumer::StreamEvent;

/// Matched endpoint with application context
#[derive(Debug, Clone)]
pub struct MatchedEndpoint {
    /// Endpoint UUID
    pub endpoint_id: Uuid,

    /// Application UUID (for webhook secret lookup)
    pub application_id: Uuid,

    /// Webhook URL to deliver to
    pub url: String,

    /// HMAC secret for signature
    pub hmac_secret: String,

    /// Rate limit (requests per second)
    pub rate_limit_per_second: i32,

    /// Maximum retry attempts
    pub max_retries: i32,

    /// HTTP timeout in seconds
    pub timeout_seconds: i32,
}

/// Endpoint matcher for database queries
pub struct EndpointMatcher {
    /// PostgreSQL connection pool
    pool: PgPool,
}

impl EndpointMatcher {
    /// Create new endpoint matcher
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool from ethhook_common::create_pool
    pub fn new(pool: PgPool) -> Self {
        info!("✅ Endpoint matcher initialized");
        Self { pool }
    }

    /// Find endpoints that match a blockchain event
    ///
    /// # Arguments
    ///
    /// * `event` - Blockchain event from Redis Stream
    ///
    /// # Returns
    ///
    /// Vector of matched endpoints with delivery configuration
    pub async fn find_matching_endpoints(
        &self,
        event: &StreamEvent,
    ) -> Result<Vec<MatchedEndpoint>> {
        // Query database for matching endpoints
        //
        // Match conditions:
        // 1. endpoint.is_active = true
        // 2. (endpoint.contract_address = event.contract OR endpoint.contract_address IS NULL)
        // 3. (endpoint.event_topics <@ event.topics OR endpoint.event_topics IS NULL)
        //    <@ = PostgreSQL array "contained by" operator
        //
        // Example:
        //   event.topics = ['0xddf...', '0x000...alice', '0x000...bob']
        //   endpoint.event_topics = ['0xddf...']
        //   Match: YES (Transfer signature matches)
        //
        //   endpoint.event_topics = ['0xddf...', '0x111...charlie']
        //   Match: NO (charlie not in event topics)

        let rows = sqlx::query(
            r#"
            SELECT 
                e.id AS endpoint_id,
                e.application_id,
                e.webhook_url AS url,
                e.hmac_secret,
                e.rate_limit_per_second,
                e.max_retries,
                e.timeout_seconds
            FROM endpoints e
            WHERE e.is_active = true
              AND (
                  -- Match chain_ids
                  e.chain_ids IS NULL
                  OR $3::INTEGER = ANY(e.chain_ids)
              )
              AND (
                  -- Match contract_addresses
                  e.contract_addresses IS NULL 
                  OR LOWER($1) = ANY(
                      SELECT LOWER(UNNEST(e.contract_addresses))
                  )
              )
              AND (
                  -- Match event_signatures (topics)
                  e.event_signatures IS NULL
                  OR e.event_signatures <@ $2
              )
            "#,
        )
        .bind(&event.contract_address)
        .bind(&event.topics)
        .bind(event.chain_id as i32)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query matching endpoints")?;

        let mut endpoints = Vec::new();

        for row in rows {
            endpoints.push(MatchedEndpoint {
                endpoint_id: row.get("endpoint_id"),
                application_id: row.get("application_id"),
                url: row.get("url"),
                hmac_secret: row.get("hmac_secret"),
                rate_limit_per_second: row.get("rate_limit_per_second"),
                max_retries: row.get("max_retries"),
                timeout_seconds: row.get("timeout_seconds"),
            });
        }

        debug!(
            "Found {} matching endpoints for event {} (contract: {})",
            endpoints.len(),
            event.transaction_hash,
            &event.contract_address[..10]
        );

        Ok(endpoints)
    }

    /// Get endpoint statistics
    ///
    /// Returns total number of active endpoints for monitoring.
    #[allow(dead_code)]
    pub async fn stats(&self) -> Result<EndpointStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) FILTER (WHERE is_active = true) AS active_endpoints,
                COUNT(*) FILTER (WHERE is_active = false) AS inactive_endpoints,
                COUNT(DISTINCT application_id) AS applications
            FROM endpoints
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get endpoint stats")?;

        Ok(EndpointStats {
            active_endpoints: row.get::<i64, _>("active_endpoints") as usize,
            inactive_endpoints: row.get::<i64, _>("inactive_endpoints") as usize,
            applications: row.get::<i64, _>("applications") as usize,
        })
    }
}

/// Endpoint statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EndpointStats {
    pub active_endpoints: usize,
    pub inactive_endpoints: usize,
    pub applications: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_matcher_creation() {
        let pool = sqlx::PgPool::connect("postgresql://localhost/ethhook_test")
            .await
            .unwrap();

        let _matcher = EndpointMatcher::new(pool);
    }

    #[tokio::test]
    #[ignore] // Requires database with data
    async fn test_find_matching_endpoints() {
        let pool = sqlx::PgPool::connect("postgresql://localhost/ethhook_test")
            .await
            .unwrap();

        let matcher = EndpointMatcher::new(pool);

        let event = StreamEvent {
            chain_id: 1,
            block_number: 18000000,
            block_hash: "0xabc123".to_string(),
            transaction_hash: "0xdef456".to_string(),
            log_index: 5,
            contract_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
            topics: vec![
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(), // Transfer
            ],
            data: "0x".to_string(),
            timestamp: 1696800000,
        };

        let endpoints = matcher.find_matching_endpoints(&event).await.unwrap();

        // Should return endpoints that match USDC Transfer events
        println!("Found {} matching endpoints", endpoints.len());
    }
}
