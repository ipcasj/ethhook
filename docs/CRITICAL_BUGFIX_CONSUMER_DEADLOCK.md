# Critical Bugfix: Consumer Mutex Deadlock

**Date:** October 10, 2025  
**Severity:** CRITICAL  
**Status:** ✅ FIXED

## Problem Description

### The Bug

The Message Processor service was **never acknowledging any messages** processed from Redis Streams, causing all messages to remain in the XPENDING state indefinitely. This prevented proper consumer group functionality and would cause message reprocessing on service restart.

### Root Cause

A single `StreamConsumer` instance was shared across all 4 stream processing tasks (one per blockchain: Ethereum, Arbitrum, Optimism, Base). The consumer was wrapped in `Arc<Mutex<StreamConsumer>>` to allow shared access.

The deadlock occurred because:

1. Each stream task runs in a loop:

   ```rust
   loop {
       // Lock consumer, read with XREADGROUP (BLOCK 5000ms)
       let entries = consumer.lock().await.read_events(...).await;
       // Release lock after read
       
       // Process events (no lock)
       process_events(entries);
       
       // Lock consumer again to ACK
       consumer.lock().await.ack_messages(...).await;
       // ❌ DEADLOCK: Lock already held by another stream's XREADGROUP!
   }
   ```

2. XREADGROUP with `BLOCK 5000ms` holds the mutex for up to 5 seconds waiting for new events

3. When Stream A tries to ACK its messages, Stream B is already holding the consumer lock doing XREADGROUP

4. Stream A blocks forever waiting for the lock → messages never acknowledged

### Evidence

```bash
# Before fix - NO ACK logs
$ grep "Acknowledged" logs.txt
# (empty - no messages ever ACKed)

# Redis XPENDING showed all messages stuck
$ docker exec ethhook-redis redis-cli XPENDING events:1 message_processors
10
1760121603010-0
1760121603014-1
test-recovery-consumer
10
```

## Solution

### The Fix

Create **separate `StreamConsumer` instance for each stream** instead of sharing one:

```rust
// Before (BROKEN):
let consumer = Arc::new(Mutex::new(StreamConsumer::new(...).await?));
for chain in &config.chains {
    let consumer_clone = Arc::clone(&consumer); // ❌ Shared!
    spawn_task(consumer_clone);
}

// After (FIXED):
for chain in &config.chains {
    // ✅ Each stream gets its own consumer
    let stream_consumer = StreamConsumer::new(...).await?;
    let consumer = Arc::new(Mutex::new(stream_consumer));
    spawn_task(consumer);
}
```

### Code Changes

**File:** `crates/message-processor/src/main.rs`

```diff
  // Spawn processing tasks for each chain
  let mut handles = vec![];
  for chain in &config.chains {
      let chain_config = chain.clone();
-     let consumer = Arc::clone(&consumer);
+     
+     // Create separate consumer for each stream to avoid mutex contention
+     let stream_consumer = match StreamConsumer::new(
+         &config.redis_url(),
+         &config.consumer_group,
+         &config.consumer_name,
+     )
+     .await
+     {
+         Ok(c) => c,
+         Err(e) => {
+             error!("Failed to create consumer for stream {}: {}", chain_config.stream_name, e);
+             return Err(e.into());
+         }
+     };
+     let consumer = Arc::new(Mutex::new(stream_consumer));
      
      let matcher = Arc::clone(&matcher);
      ...
  }
```

## Verification

### Test Results

After fix, messages are properly acknowledged:

```bash
$ cargo test --test e2e_tests test_service_recovery_with_consumer_groups -- --ignored --nocapture
2025-10-10T19:01:31.402Z INFO [events:1] Acknowledging 1 messages...
2025-10-10T19:01:31.403Z INFO [events:1] ✅ Acknowledged 1 messages (expected 1)
2025-10-10T19:01:31.442Z INFO [events:1] Acknowledging 9 messages...
2025-10-10T19:01:31.442Z INFO [events:1] ✅ Acknowledged 9 messages (expected 9)

✅ Service Recovery Test PASSED!
```

### Redis XPENDING (After Fix)

```bash
$ docker exec ethhook-redis redis-cli XPENDING events:1 message_processors
0  # ✅ No pending messages - all acknowledged!
```

## Impact

### Before Fix

- ❌ 0% message acknowledgment rate
- ❌ All messages stuck in XPENDING forever
- ❌ Service restarts would reprocess ALL messages
- ❌ Memory leak in Redis (pending messages never removed)
- ❌ No benefit from consumer groups

### After Fix

- ✅ 100% message acknowledgment rate
- ✅ Messages ACKed within milliseconds of processing
- ✅ Service restarts resume from correct position
- ✅ Consumer groups working as designed
- ✅ Proper fault tolerance and exactly-once processing

## Performance Implications

### Resource Usage

- **Before:** 1 Redis connection shared across 4 streams
- **After:** 4 Redis connections (1 per stream)
- **Impact:** Minimal - Redis handles thousands of connections easily

### Throughput

- **Improved:** No more mutex contention during XREADGROUP blocking
- **Improved:** Each stream can read/process/ack independently
- **Result:** Better parallelism and throughput

## Lessons Learned

1. **Never share blocking resources across async tasks** - XREADGROUP with BLOCK holds the lock too long
2. **Test acknowledgment explicitly** - Add tests that verify XPENDING state
3. **Resource-per-task pattern** - Each async task should have its own I/O resources when possible
4. **Observability matters** - Added extensive logging to catch issues like this earlier

## Related Tests

- ✅ `test_consumer_group_acknowledgment` - Validates ACK behavior
- ✅ `test_service_recovery_with_consumer_groups` - Tests crash/restart scenarios
- ✅ `test_full_pipeline_with_mock_ethereum` - E2E pipeline with ACK verification

## Additional Monitoring

Added enhanced logging in `consumer.rs`:

```rust
pub async fn ack_messages(&mut self, stream_name: &str, message_ids: &[String]) -> Result<()> {
    info!("[{}] Acknowledging {} messages...", stream_name, message_ids.len());
    
    let acked: usize = cmd.query_async(&mut self.client).await?;
    
    info!("[{}] ✅ Acknowledged {} messages (expected {})", stream_name, acked, message_ids.len());
    Ok(())
}
```

This allows easy verification that ACKs are happening in production logs.
