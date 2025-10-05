/*!
 * Integration Tests for Event Ingestor
 * 
 * These tests validate the full pipeline:
 * 1. Deduplication (Redis SET operations)
 * 2. Publishing (Redis STREAM operations)
 * 3. Event ID generation
 * 4. Stream naming conventions
 * 
 * Run with: cargo test --test integration_test -- --ignored
 * (Requires Redis running on localhost:6379)
 */

use redis::AsyncCommands;
use ethhook_event_ingestor::types::ProcessedEvent;

/// Helper to create test ProcessedEvent
fn create_test_event(chain_id: u64, block_number: u64, tx_hash: &str) -> ProcessedEvent {
    ProcessedEvent {
        chain_id,
        block_number,
        block_hash: "0xabc123".to_string(),
        transaction_hash: tx_hash.to_string(),
        log_index: 0,
        contract_address: "0x1234567890123456789012345678901234567890".to_string(),
        topics: vec![
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
        ],
        data: "0x".to_string(),
        timestamp: 1696435200,
    }
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_redis_connection() {
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    
    // Test PING
    let pong: String = redis::cmd("PING").query_async(&mut conn).await.unwrap();
    assert_eq!(pong, "PONG");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_deduplication_stores_event_ids() {
    use ethhook_event_ingestor::deduplicator::Deduplicator;
    
    let mut dedup = Deduplicator::new("redis://127.0.0.1:6379", 3600).await.unwrap(); // 1 hour TTL
    
    let event = create_test_event(1, 18000000, "0xtest123");
    let event_id = event.event_id();
    
    // First check - should be new
    let is_dup1 = dedup.is_duplicate(&event_id).await.unwrap();
    assert!(!is_dup1, "First check should NOT be duplicate");
    
    // Second check - should be duplicate
    let is_dup2 = dedup.is_duplicate(&event_id).await.unwrap();
    assert!(is_dup2, "Second check SHOULD be duplicate");
    
    // Cleanup
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let _: () = conn.del(&event_id).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_deduplication_with_different_events() {
    use ethhook_event_ingestor::deduplicator::Deduplicator;
    
    let mut dedup = Deduplicator::new("redis://127.0.0.1:6379", 3600).await.unwrap();
    
    let event1 = create_test_event(1, 18000000, "0xaaa");
    let event2 = create_test_event(1, 18000001, "0xbbb");
    
    let id1 = event1.event_id();
    let id2 = event2.event_id();
    
    // Both should be new
    assert!(!dedup.is_duplicate(&id1).await.unwrap());
    assert!(!dedup.is_duplicate(&id2).await.unwrap());
    
    // Both should now be duplicates
    assert!(dedup.is_duplicate(&id1).await.unwrap());
    assert!(dedup.is_duplicate(&id2).await.unwrap());
    
    // Cleanup
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let _: () = conn.del(&[&id1, &id2]).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_stream_publishing() {
    use ethhook_event_ingestor::publisher::StreamPublisher;
    
    let mut publisher = StreamPublisher::new("redis://127.0.0.1:6379").await.unwrap();
    
    let event = create_test_event(1, 18000000, "0xintegration_test");
    let stream_id = publisher.publish(&event).await.unwrap();
    
    // Verify stream ID format (should be "timestamp-sequence")
    assert!(stream_id.contains('-'), "Stream ID should contain timestamp-sequence");
    
    // Verify event was published to correct stream
    let stream_name = event.stream_name();
    assert_eq!(stream_name, "eth:events:1");
    
    // Read back from stream to verify data
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let result: redis::streams::StreamReadReply = redis::cmd("XRANGE")
        .arg(&stream_name)
        .arg("-")
        .arg("+")
        .arg("COUNT")
        .arg(1)
        .query_async(&mut conn)
        .await
        .unwrap();
    
    assert!(!result.keys.is_empty(), "Stream should have events");
    let stream_key = &result.keys[0];
    assert_eq!(stream_key.key, stream_name);
    assert!(!stream_key.ids.is_empty(), "Should have at least one event");
    
    // Verify event data
    let entry = &stream_key.ids[0];
    let fields: std::collections::HashMap<String, String> = entry.map.iter()
        .map(|(k, v)| match v {
            redis::Value::BulkString(bytes) => (k.clone(), String::from_utf8_lossy(bytes).to_string()),
            redis::Value::SimpleString(s) => (k.clone(), s.clone()),
            _ => (k.clone(), format!("{:?}", v)),
        })
        .collect();
    
    assert_eq!(fields.get("chain_id").unwrap(), "1");
    assert_eq!(fields.get("block_number").unwrap(), "18000000");
    assert_eq!(fields.get("tx_hash").unwrap(), "0xintegration_test");
    
    // Cleanup
    let _: () = conn.del(&stream_name).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_full_pipeline_dedup_then_publish() {
    use ethhook_event_ingestor::deduplicator::Deduplicator;
    use ethhook_event_ingestor::publisher::StreamPublisher;
    
    let mut dedup = Deduplicator::new("redis://127.0.0.1:6379", 3600).await.unwrap();
    let mut publisher = StreamPublisher::new("redis://127.0.0.1:6379").await.unwrap();
    
    let event = create_test_event(1, 18000000, "0xfull_pipeline_test");
    let event_id = event.event_id();
    let stream_name = event.stream_name();
    
    // Step 1: Check deduplication (should be new)
    let is_dup = dedup.is_duplicate(&event_id).await.unwrap();
    assert!(!is_dup, "Event should be new");
    
    // Step 2: Publish event
    let stream_id = publisher.publish(&event).await.unwrap();
    assert!(!stream_id.is_empty());
    
    // Step 3: Check deduplication again (should be duplicate now)
    let is_dup2 = dedup.is_duplicate(&event_id).await.unwrap();
    assert!(is_dup2, "Event should now be duplicate");
    
    // Step 4: Verify we DON'T publish duplicates
    // (In real code, we'd skip publish, but let's verify the dedup worked)
    
    // Cleanup
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let _: () = conn.del(&[&event_id, &stream_name]).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_multiple_chains_separate_streams() {
    use ethhook_event_ingestor::publisher::StreamPublisher;
    
    let mut publisher = StreamPublisher::new("redis://127.0.0.1:6379").await.unwrap();
    
    // Publish events to different chains
    let eth_event = create_test_event(1, 18000000, "0xeth_test");
    let arb_event = create_test_event(42161, 18000000, "0xarb_test");
    
    let eth_stream_id = publisher.publish(&eth_event).await.unwrap();
    let arb_stream_id = publisher.publish(&arb_event).await.unwrap();
    
    // Verify different streams
    assert_eq!(eth_event.stream_name(), "eth:events:1");
    assert_eq!(arb_event.stream_name(), "eth:events:42161");
    
    // Verify both published successfully
    assert!(!eth_stream_id.is_empty());
    assert!(!arb_stream_id.is_empty());
    
    // Cleanup
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let _: () = conn.del(&["eth:events:1", "eth:events:42161"]).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_event_id_uniqueness() {
    // Event IDs should be unique based on chain_id:block:tx_hash:log_index
    let event1 = create_test_event(1, 18000000, "0xabc");
    let event2 = create_test_event(1, 18000000, "0xdef"); // Different tx
    let event3 = create_test_event(1, 18000001, "0xabc"); // Different block
    let event4 = create_test_event(42161, 18000000, "0xabc"); // Different chain
    
    let id1 = event1.event_id();
    let id2 = event2.event_id();
    let id3 = event3.event_id();
    let id4 = event4.event_id();
    
    // All should be unique
    assert_ne!(id1, id2, "Different tx_hash should create different IDs");
    assert_ne!(id1, id3, "Different block_number should create different IDs");
    assert_ne!(id1, id4, "Different chain_id should create different IDs");
    assert_ne!(id2, id3);
    assert_ne!(id2, id4);
    assert_ne!(id3, id4);
    
    // Same event should produce same ID
    let event1_dup = create_test_event(1, 18000000, "0xabc");
    assert_eq!(event1.event_id(), event1_dup.event_id());
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_stream_name_format() {
    let eth_event = create_test_event(1, 18000000, "0xtest");
    let arb_event = create_test_event(42161, 18000000, "0xtest");
    let opt_event = create_test_event(10, 18000000, "0xtest");
    
    assert_eq!(eth_event.stream_name(), "eth:events:1");
    assert_eq!(arb_event.stream_name(), "eth:events:42161");
    assert_eq!(opt_event.stream_name(), "eth:events:10");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_concurrent_deduplication() {
    use ethhook_event_ingestor::deduplicator::Deduplicator;
    use tokio::task::JoinSet;
    
    let event = create_test_event(1, 18000000, "0xconcurrent_test");
    let event_id = event.event_id();
    
    // Spawn 10 concurrent tasks checking same event
    let mut tasks = JoinSet::new();
    for i in 0..10 {
        let event_id_clone = event_id.clone();
        tasks.spawn(async move {
            let mut dedup = Deduplicator::new("redis://127.0.0.1:6379", 3600).await.unwrap();
            let is_dup = dedup.is_duplicate(&event_id_clone).await.unwrap();
            (i, is_dup)
        });
    }
    
    // Collect results
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.unwrap());
    }
    
    // Exactly ONE task should see it as new (due to Redis SET NX atomicity)
    let new_count = results.iter().filter(|(_, is_dup)| !is_dup).count();
    assert_eq!(new_count, 1, "Exactly one task should see event as new");
    
    // Cleanup
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let _: () = conn.del(&event_id).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_publisher_handles_special_characters() {
    use ethhook_event_ingestor::publisher::StreamPublisher;
    
    let mut publisher = StreamPublisher::new("redis://127.0.0.1:6379").await.unwrap();
    
    // Create event with special characters in data
    let mut event = create_test_event(1, 18000000, "0xspecial_chars");
    event.data = "0x48656c6c6f20576f726c6421".to_string(); // "Hello World!" in hex
    event.contract_address = "0xabcdef1234567890abcdef1234567890abcdef12".to_string();
    
    let stream_id = publisher.publish(&event).await.unwrap();
    assert!(!stream_id.is_empty());
    
    // Read back and verify
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();
    let stream_name = event.stream_name();
    let result: redis::streams::StreamReadReply = redis::cmd("XRANGE")
        .arg(&stream_name)
        .arg("-")
        .arg("+")
        .arg("COUNT")
        .arg(1)
        .query_async(&mut conn)
        .await
        .unwrap();
    
    let entry = &result.keys[0].ids[0];
    let data_field = entry.map.iter()
        .find(|(k, _)| k.as_str() == "data")
        .map(|(_, v)| match v {
            redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
            _ => String::new(),
        })
        .unwrap();
    
    assert_eq!(data_field, "0x48656c6c6f20576f726c6421");
    
    // Cleanup
    let _: () = conn.del(&stream_name).await.unwrap();
}
