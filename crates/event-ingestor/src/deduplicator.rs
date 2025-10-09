#![allow(dead_code)]
/*!
 * Deduplication Module
 *
 * Prevents duplicate webhook deliveries during chain reorganizations.
 *
 * ## How It Works
 *
 * Blockchains can experience "reorganizations" (reorgs) where blocks are replaced:
 *
 * ```text
 * Original chain:
 *   Block 100 → Block 101a → Block 102a
 *
 * After reorg:
 *   Block 100 → Block 101b → Block 102b
 *                    ↑
 *              Block 101a is "uncle" (orphaned)
 * ```
 *
 * Without deduplication:
 * 1. We process Block 101a, send webhook
 * 2. Reorg happens, Block 101b becomes canonical
 * 3. We process Block 101b, send DUPLICATE webhook
 * 4. Customer gets confused: "Why same event twice?"
 *
 * With deduplication:
 * 1. Process Block 101a → Store event ID in Redis SET
 * 2. Reorg happens, Block 101b received
 * 3. Try to add event ID → Already exists → Skip
 * 4. Customer gets exactly one webhook ✅
 *
 * ## Implementation
 *
 * We use Redis SET with TTL:
 * - **Key**: `seen_events` (single SET for all chains)
 * - **Members**: `event:{chain_id}:{tx_hash}:{log_index}`
 * - **TTL**: 24 hours (reorgs can't go back further than this)
 *
 * ## Performance
 *
 * - **SADD**: O(1) operation
 * - **Memory**: ~100 bytes per event × 86,400 events/day × 4 chains ≈ 35 MB
 * - **Throughput**: Redis can handle 100,000+ SADD/sec
 */

use anyhow::{Context, Result};
use redis::AsyncCommands;
use tracing::{debug, info};

/// Deduplicator using Redis SET for event tracking
///
/// Uses Redis SET with TTL to track which events we've already processed.
/// This prevents duplicate webhook deliveries during blockchain reorganizations.
pub struct Deduplicator {
    /// Redis connection manager (automatically handles reconnections)
    client: redis::aio::ConnectionManager,

    /// TTL for event tracking in seconds (default: 24 hours)
    ttl_seconds: u64,
}

impl Deduplicator {
    /// Create new deduplicator with Redis connection
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
    /// * `ttl_seconds` - How long to remember events (default: 86400 = 24 hours)
    ///
    /// # Example
    ///
    /// ```no_run
    /// let dedup = Deduplicator::new("redis://localhost:6379", 86400).await?;
    /// ```
    pub async fn new(redis_url: &str, ttl_seconds: u64) -> Result<Self> {
        info!("Connecting to Redis at {}", redis_url);

        let client = redis::Client::open(redis_url).context("Failed to create Redis client")?;

        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .context("Failed to connect to Redis")?;

        info!("✅ Connected to Redis successfully");

        Ok(Self {
            client: conn,
            ttl_seconds,
        })
    }

    /// Check if an event has already been processed
    ///
    /// # Arguments
    ///
    /// * `event_id` - Unique event identifier (format: "event:{chain}:{tx}:{log}")
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Event is a duplicate (already processed)
    /// * `Ok(false)` - Event is new (should be processed)
    /// * `Err(_)` - Redis connection error
    ///
    /// # Example
    ///
    /// ```no_run
    /// let event_id = "event:1:0xabc123...:5";
    /// if dedup.is_duplicate(event_id).await? {
    ///     println!("Skip duplicate event");
    ///     return;
    /// }
    /// ```
    pub async fn is_duplicate(&mut self, event_id: &str) -> Result<bool> {
        // Try to add event ID to Redis SET
        // SADD returns:
        // - 1 if element was added (new event)
        // - 0 if element already exists (duplicate)
        let added: i32 = self
            .client
            .sadd("seen_events", event_id)
            .await
            .context("Failed to add event to Redis SET")?;

        if added == 1 {
            // New event - set expiration on the SET
            // Note: We set TTL on the entire SET, not individual members
            // This is fine because we want to forget ALL old events after 24 hours
            let _: () = self
                .client
                .expire("seen_events", self.ttl_seconds as i64)
                .await
                .context("Failed to set TTL on seen_events")?;

            debug!("✅ New event: {}", event_id);
            Ok(false) // Not a duplicate
        } else {
            debug!("⚠️  Duplicate event detected: {}", event_id);
            Ok(true) // Is a duplicate
        }
    }

    /// Get statistics about deduplication
    ///
    /// Returns the number of events currently being tracked.
    pub async fn stats(&mut self) -> Result<DeduplicationStats> {
        let count: usize = self
            .client
            .scard("seen_events")
            .await
            .context("Failed to get SET cardinality")?;

        let ttl: i64 = self.client.ttl("seen_events").await.unwrap_or(-1);

        Ok(DeduplicationStats {
            tracked_events: count,
            ttl_seconds: if ttl >= 0 { Some(ttl as u64) } else { None },
        })
    }

    /// Clear all tracked events (for testing)
    #[cfg(test)]
    pub async fn clear(&mut self) -> Result<()> {
        let _: () = self
            .client
            .del("seen_events")
            .await
            .context("Failed to clear seen_events")?;
        Ok(())
    }
}

/// Statistics about deduplication
#[derive(Debug, Clone)]
pub struct DeduplicationStats {
    /// Number of events currently being tracked
    pub tracked_events: usize,

    /// TTL remaining on the SET (None if no TTL)
    pub ttl_seconds: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Run with: cargo test --package ethhook-event-ingestor -- --ignored
    async fn test_deduplication_flow() {
        // This test requires Redis running on localhost:6379
        let mut dedup = Deduplicator::new("redis://localhost:6379", 60)
            .await
            .expect("Failed to connect to Redis");

        // Clear any existing data
        dedup.clear().await.expect("Failed to clear");

        let event_id = "event:1:0xtest123:0";

        // First time: should NOT be duplicate
        let is_dup1 = dedup.is_duplicate(event_id).await.expect("Failed");
        assert!(!is_dup1, "First event should not be duplicate");

        // Second time: SHOULD be duplicate
        let is_dup2 = dedup.is_duplicate(event_id).await.expect("Failed");
        assert!(is_dup2, "Second event should be duplicate");

        // Stats should show 1 tracked event
        let stats = dedup.stats().await.expect("Failed to get stats");
        assert_eq!(stats.tracked_events, 1);
        assert!(stats.ttl_seconds.is_some());

        // Cleanup
        dedup.clear().await.expect("Failed to clear");
    }

    #[tokio::test]
    #[ignore]
    async fn test_different_events_not_duplicates() {
        let mut dedup = Deduplicator::new("redis://localhost:6379", 60)
            .await
            .expect("Failed to connect to Redis");

        dedup.clear().await.expect("Failed to clear");

        let event1 = "event:1:0xaaa:0";
        let event2 = "event:1:0xaaa:1"; // Different log_index
        let event3 = "event:1:0xbbb:0"; // Different tx_hash
        let event4 = "event:42161:0xaaa:0"; // Different chain_id

        // All should be unique
        assert!(!dedup.is_duplicate(event1).await.unwrap());
        assert!(!dedup.is_duplicate(event2).await.unwrap());
        assert!(!dedup.is_duplicate(event3).await.unwrap());
        assert!(!dedup.is_duplicate(event4).await.unwrap());

        // Stats should show 4 tracked events
        let stats = dedup.stats().await.expect("Failed to get stats");
        assert_eq!(stats.tracked_events, 4);

        dedup.clear().await.expect("Failed to clear");
    }
}
