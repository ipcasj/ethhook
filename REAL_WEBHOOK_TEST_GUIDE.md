# Real Webhook Testing Guide - Live Production Flow

**Goal**: Test EthHook with REAL webhook deliveries, REAL Sepolia events, and see REAL statistics in Grafana

---

## Setup (5 minutes)

### Step 1: Start Real Webhook Receiver

Open a **new terminal** window:

```bash
cd /Users/igor/rust_projects/capstone0

# Start webhook receiver on port 8000
python3 webhook_receiver.py 8000
```

**You'll see**:

```
🚀 REAL WEBHOOK RECEIVER STARTED!
📍 Listening on: http://0.0.0.0:8000
📍 Webhook URL:  http://localhost:8000/webhook
⏳ Waiting for webhooks from EthHook...
```

**Keep this terminal open!** You'll see webhooks arrive here in real-time.

---

### Step 2: Create Real Endpoint in Database

Open another terminal:

```bash
# Connect to PostgreSQL
docker exec -it ethhook-postgres psql -U ethhook -d ethhook
```

**Run this SQL** (creates a real endpoint with your webhook receiver):

```sql
-- Insert a new endpoint for real testing
INSERT INTO endpoints (
    id,
    application_id,
    name,
    webhook_url,
    hmac_secret,
    contract_addresses,
    event_signatures,
    chain_ids,
    is_active,
    description
) VALUES (
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    (SELECT id FROM applications ORDER BY created_at DESC LIMIT 1),  -- Use latest app
    'REAL Production Test',
    'http://host.docker.internal:8000/webhook',  -- Real webhook receiver!
    'test_secret_key_for_hmac_signature_verification',
    ARRAY['0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9'],  -- Sepolia WETH
    ARRAY['Transfer(address,address,uint256)'],
    ARRAY[11155111],  -- Sepolia chain ID
    true,
    'Real webhook endpoint for production testing with actual deliveries'
)
ON CONFLICT (id) DO UPDATE SET
    webhook_url = EXCLUDED.webhook_url,
    is_active = true;

-- Verify it was created
SELECT id, name, webhook_url, is_active, chain_ids
FROM endpoints
WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
```

**Exit psql**: Type `\q` and press Enter

**Why `host.docker.internal`?**
- Your webhook receiver runs on host (port 8000)
- Docker containers need special hostname to reach host
- `host.docker.internal:8000` = your local webhook receiver

---

### Step 3: Verify Event Ingestor is Running

```bash
# Check Event Ingestor logs
docker logs ethhook-event-ingestor --tail 30 | grep "Processing block"
```

**You should see**:
```
INFO [Sepolia Testnet] Processing block #9469712
INFO Block 9469712 processed: 52 transactions, 56 events
```

**If you see blocks being processed** → ✅ Event Ingestor is capturing real Sepolia events!

---

## Real-Time Monitoring (Watch the Magic!)

### Terminal Layout (4 windows)

**Terminal 1: Webhook Receiver** (already running)
```bash
python3 webhook_receiver.py 8000
```

**Terminal 2: Event Ingestor Logs**
```bash
docker logs -f ethhook-event-ingestor | grep -E "Processing block|Transfer|Published"
```

**Terminal 3: Message Processor Logs**
```bash
docker logs -f ethhook-message-processor | grep -E "Matched|endpoint|Published"
```

**Terminal 4: Webhook Delivery Logs**
```bash
docker logs -f ethhook-webhook-delivery | grep -E "Sending|delivered|status"
```

---

## What You'll See (Real-Time Event Flow)

### Terminal 2 (Event Ingestor):
```
INFO [Sepolia Testnet] Processing block #9469715
INFO Block 9469715 processed: 78 transactions, 82 events
INFO Found Transfer event: contract=0x7b79... topics=3
INFO Published event to Redis stream: events:11155111
```

### Terminal 3 (Message Processor):
```
DEBUG Read 1 events from events:11155111
INFO Event matched endpoint: endpoint_id=aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa
INFO Published delivery job: event_id=... endpoint_id=aaaaaaaa...
```

### Terminal 4 (Webhook Delivery):
```
DEBUG Sending webhook to http://host.docker.internal:8000/webhook
INFO ✅ Webhook delivered successfully: status=200 duration=15ms
DEBUG Logged delivery attempt: success=true
```

### Terminal 1 (YOUR Webhook Receiver):
```
================================================================================
🎉 WEBHOOK RECEIVED! [2025-10-22 22:15:34]
================================================================================

📋 HEADERS:
  X-Webhook-Signature: sha256=abc123...
  X-Webhook-Timestamp: 1729653934
  X-Chain-Id: 11155111
  X-Event-Id: 550e8400-e29b-41d4-a716-446655440000

📦 PAYLOAD:
{
  "event_id": "550e8400-e29b-41d4-a716-446655440000",
  "chain_id": 11155111,
  "chain_name": "Sepolia Testnet",
  "block_number": 9469715,
  "transaction_hash": "0xabc123def456...",
  "log_index": 42,
  "contract_address": "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9",
  "event_signature": "Transfer(address,address,uint256)",
  "topics": [...],
  "data": "0x0000...",
  "timestamp": "2025-10-22T22:15:32Z",
  "decoded": {
    "from": "0xsender...",
    "to": "0xreceiver...",
    "value": "1000000000000000000"
  }
}

🔍 EVENT DETAILS:
  Chain ID: 11155111
  Chain Name: Sepolia Testnet
  Block: 9469715
  Transaction: 0xabc123def456...
  Contract: 0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9
  Event: Transfer(address,address,uint256)

  📊 DECODED DATA:
    from: 0xsender...
    to: 0xreceiver...
    value: 1000000000000000000

================================================================================
✅ Total webhooks received: 1
================================================================================
```

**🎉 THIS IS REAL!** You just received an actual Sepolia blockchain event via your webhook!

---

## Verify in Database

While events are flowing, check the database:

### Check Events Table
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "SELECT COUNT(*) as total_events,
          MAX(block_number) as latest_block,
          MAX(ingested_at) as latest_ingestion
   FROM events;"
```

### Check Delivery Attempts
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "SELECT
      COUNT(*) as total_deliveries,
      COUNT(*) FILTER (WHERE success = true) as successful,
      COUNT(*) FILTER (WHERE success = false) as failed,
      AVG(duration_ms) as avg_duration_ms
   FROM delivery_attempts
   WHERE endpoint_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';"
```

### View Recent Successful Deliveries
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "SELECT
      da.id,
      da.http_status_code,
      da.duration_ms,
      da.attempted_at,
      e.block_number,
      e.contract_address
   FROM delivery_attempts da
   JOIN events e ON da.event_id = e.id
   WHERE da.endpoint_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
     AND da.success = true
   ORDER BY da.attempted_at DESC
   LIMIT 10;"
```

**You should see**:
```
total_deliveries | successful | failed | avg_duration_ms
-----------------+------------+--------+----------------
        15       |     15     |   0    |      12.5
```

**100% success rate with real endpoint!** 🎉

---

## Check Redis Streams (Real-Time Event Queue)

### View Events Stream
```bash
# See latest events from Sepolia
redis-cli XREAD COUNT 5 STREAMS events:11155111 0-0
```

**You'll see**:
```
1) 1) "events:11155111"
   2) 1) 1) "1729653934000-0"
         2)  1) "block_number"
             2) "9469715"
             3) "transaction_hash"
             4) "0xabc123..."
             5) "contract_address"
             6) "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9"
             ...
```

### View Delivery Jobs Stream
```bash
# See webhook delivery jobs
redis-cli XREAD COUNT 5 STREAMS webhook:delivery:jobs 0-0
```

### Check Stream Lengths (Queue Depth)
```bash
redis-cli XLEN events:11155111
redis-cli XLEN webhook:delivery:jobs
```

**Healthy system**:
- Events stream: 0-100 (processing fast)
- Delivery jobs: 0-10 (delivering fast)

**If queues are growing** → System can't keep up (unlikely with Sepolia's low volume)

---

## Grafana Dashboards (Real Statistics)

### Open Grafana
```
http://localhost:3001
Login: admin / admin
```

### Check These Dashboards

#### 1. System Overview Dashboard
**Metrics to verify**:
- ✅ Service health (all green)
- ✅ Event ingestion rate (events/sec)
- ✅ Webhook delivery rate (deliveries/sec)
- ✅ Success rate (should be 100%!)

#### 2. Event Pipeline Dashboard
**Metrics to verify**:
- ✅ Events ingested (increasing count)
- ✅ Events matched (number matching your endpoint)
- ✅ Webhooks delivered (successful deliveries)
- ✅ Delivery latency (avg time to deliver)

#### 3. Redis Metrics
**Metrics to verify**:
- ✅ Stream length (events:11155111)
- ✅ Commands/sec
- ✅ Memory usage
- ✅ Connected clients

#### 4. PostgreSQL Metrics
**Metrics to verify**:
- ✅ Active connections
- ✅ Transactions/sec
- ✅ Events table row count
- ✅ Delivery attempts table row count

### Create Custom Query (Prometheus)

Open Prometheus: http://localhost:9090

**Query for event ingestion rate**:
```promql
rate(events_ingested_total[1m])
```

**Query for webhook success rate**:
```promql
rate(webhooks_delivered_success[5m]) / rate(webhooks_delivered_total[5m]) * 100
```

**Query for delivery latency (p95)**:
```promql
histogram_quantile(0.95, rate(webhook_delivery_duration_seconds_bucket[5m]))
```

---

## Advanced Testing

### Test High Volume (Catch More Events)

Update endpoint to catch ALL Sepolia events temporarily:

```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "UPDATE endpoints
   SET contract_addresses = ARRAY['0x0000000000000000000000000000000000000000'],
       event_signatures = ARRAY['Transfer(address,address,uint256)']
   WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';"
```

**This will match ALL Transfer events on Sepolia** (hundreds per block!)

**Watch your webhook receiver flood with events!** 🌊

**Revert when done**:
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "UPDATE endpoints
   SET contract_addresses = ARRAY['0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9']
   WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';"
```

### Test HMAC Signature Verification

In your webhook receiver terminal, you see:
```
X-Webhook-Signature: sha256=abc123...
X-Webhook-Timestamp: 1729653934
```

**Verify the signature** (Python):
```python
import hmac
import hashlib
import json

# From database
endpoint_secret = "test_secret_key_for_hmac_signature_verification"

# From webhook
timestamp = "1729653934"
signature = "sha256=abc123..."
payload = '{"event_id":"..."}' # Full JSON body

# Calculate expected
message = f"{timestamp}.{payload}"
expected = "sha256=" + hmac.new(
    endpoint_secret.encode(),
    message.encode(),
    hashlib.sha256
).hexdigest()

# Compare
if hmac.compare_digest(signature, expected):
    print("✅ Signature VALID - Webhook is authentic!")
else:
    print("❌ Signature INVALID - Possible tampering!")
```

### Trigger Specific Event (Advanced)

If you have Sepolia ETH:

1. **Get Sepolia WETH**: https://sepoliafaucet.com/
2. **Wrap ETH to WETH**:
   ```
   Contract: 0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9
   Method: deposit() payable
   Amount: 0.01 ETH
   ```
3. **Watch your webhook receiver** - you'll see YOUR transaction arrive within 15 seconds!

---

## Success Checklist

After 10-15 minutes of running:

- [ ] ✅ Webhook receiver shows at least 1 webhook received
- [ ] ✅ All 4 terminal windows showing activity
- [ ] ✅ Database shows successful deliveries (100% success rate)
- [ ] ✅ Redis streams show events flowing
- [ ] ✅ Grafana dashboards show metrics
- [ ] ✅ Prometheus queries return data
- [ ] ✅ No errors in any service logs
- [ ] ✅ Delivery latency < 100ms

**If all checked** → 🎉 **YOUR SYSTEM IS PRODUCTION-READY!**

---

## Troubleshooting

### Issue: No webhooks received after 5 minutes

**Check 1: Is endpoint active?**
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "SELECT id, is_active, webhook_url FROM endpoints
   WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';"
```

**Check 2: Are WETH Transfer events happening?**
```bash
# Look for any Transfer events in last 100 blocks
docker logs ethhook-event-ingestor --tail 200 | grep -i transfer | head -10
```

**Check 3: Is Message Processor matching events?**
```bash
docker logs ethhook-message-processor --tail 100 | grep -E "Matched|endpoint"
```

**Check 4: Can Docker reach webhook receiver?**
```bash
# Test from inside Docker container
docker exec ethhook-webhook-delivery curl -X POST http://host.docker.internal:8000/webhook \
  -H "Content-Type: application/json" \
  -d '{"test": "connection test"}'
```
Check your webhook receiver - you should see this test message.

**Check 5: Use wildcard to catch ALL events**
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "UPDATE endpoints
   SET contract_addresses = NULL,  -- Match all contracts
       event_signatures = NULL      -- Match all events
   WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';"
```
This will catch EVERY event on Sepolia (hundreds per block!)

### Issue: Webhook receiver not accessible from Docker

**On macOS**: Use `host.docker.internal`
**On Linux**: Use `172.17.0.1` (Docker bridge IP)

**Update endpoint**:
```bash
# For Linux
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "UPDATE endpoints
   SET webhook_url = 'http://172.17.0.1:8000/webhook'
   WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';"
```

### Issue: Grafana shows no data

**Possible causes**:
1. Prometheus not scraping services
2. Services not exposing metrics
3. Dashboard not configured

**Solution**:
```bash
# Check Prometheus targets
open http://localhost:9090/targets

# Should see:
# - admin-api (port 9090)
# - event-ingestor (port 9090)
# - message-processor (port 9090)
# - webhook-delivery (port 9090)
# - postgres-exporter (port 9187)
# - redis-exporter (port 9121)
```

If targets are down, check service metrics ports in docker-compose.yml

---

## Performance Metrics (Real Data)

After running for 15 minutes, you should see:

### Event Processing
```
Events captured:        ~200-500 (depends on Sepolia activity)
Events matched:         ~10-50 (only WETH Transfers)
Webhooks delivered:     ~10-50
Success rate:           100%
```

### Latency
```
Event ingestion:        < 1 second from block production
Event matching:         < 50ms per event
Webhook delivery:       < 100ms (local network)
End-to-end latency:     < 2 seconds (block → webhook)
```

### Resource Usage
```
CPU:                    < 10% total (all services)
Memory:                 < 1GB total
Disk I/O:               Minimal
Network:                < 1MB/sec
```

**Your system is HIGHLY efficient!** 🚀

---

## What This Proves

By completing this test, you've proven:

1. ✅ **Real blockchain integration** - Sepolia events captured
2. ✅ **Real event processing** - Message processor matching works
3. ✅ **Real webhook delivery** - HTTP POST to your receiver
4. ✅ **Real HMAC security** - Signatures generated
5. ✅ **Real monitoring** - Grafana showing metrics
6. ✅ **Real database persistence** - Events and deliveries stored
7. ✅ **Real audit trail** - Full delivery history
8. ✅ **Real production readiness** - System handles real load

**This is NOT a demo. This is NOT a test.**
**This is a REAL, WORKING webhook delivery platform!** 🎉

---

## Next Steps

**After successful test**:

1. ✅ Stop webhook receiver (Ctrl+C)
2. ✅ Review statistics in Grafana
3. ✅ Check database for complete audit trail
4. ✅ Take screenshots for documentation
5. ✅ Prepare for Railway deployment

**You're ready to deploy!** 🚀

---

## Cleanup (Optional)

**Remove test endpoint**:
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "DELETE FROM endpoints WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';"
```

**Clear Redis streams**:
```bash
redis-cli DEL events:11155111
redis-cli DEL webhook:delivery:jobs
```

**Keep running for production testing!**

---

**Created**: 2025-10-22
**Status**: ✅ Ready for real production testing
**Time required**: 15 minutes
**Result**: Real webhook deliveries with real blockchain data!
