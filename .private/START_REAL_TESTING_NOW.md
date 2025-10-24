# ⚡ START REAL TESTING NOW! ⚡

**Everything is ready!** Test your EthHook system with REAL webhook deliveries in 2 minutes.

---

## Quick Start (2 Minutes)

### Step 1: Start Webhook Receiver

Open terminal and run:

```bash
cd /Users/igor/rust_projects/capstone0
./test_real_webhooks.sh
```

**You'll see**:
```
🚀 EthHook Real Webhook Testing - Quick Start
✅ All services running!
📍 Webhook Endpoint Configuration:
   URL: http://host.docker.internal:8000/webhook
🎯 Starting Real Webhook Receiver on port 8000...
⏳ Waiting for webhooks from EthHook...
```

**That's it!** Now wait for webhooks to arrive.

---

### Step 2: Monitor (Optional)

Open 3 more terminals to watch the event flow:

**Terminal 2 - Event Ingestor:**
```bash
docker logs -f ethhook-event-ingestor | grep -E "Processing block|Transfer"
```

**Terminal 3 - Webhook Delivery:**
```bash
docker logs -f ethhook-webhook-delivery | grep -E "Sending|delivered|success"
```

**Terminal 4 - Grafana:**
```bash
open http://localhost:3001
# Login: admin / admin
```

---

### Step 3: Wait for Webhook (1-10 minutes)

**When a Sepolia WETH Transfer event happens**, you'll see:

```
================================================================================
🎉 WEBHOOK RECEIVED! [2025-10-22 22:30:15]
================================================================================

📋 HEADERS:
  X-Webhook-Signature: sha256=abc123...
  X-Chain-Id: 11155111

📦 PAYLOAD:
{
  "event_id": "550e8400-...",
  "chain_id": 11155111,
  "chain_name": "Sepolia Testnet",
  "block_number": 9469750,
  "contract_address": "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9",
  "event_signature": "Transfer(address,address,uint256)",
  "decoded": {
    "from": "0x...",
    "to": "0x...",
    "value": "1000000000000000000"
  }
}

================================================================================
✅ Total webhooks received: 1
================================================================================
```

**🎉 SUCCESS!** You just received a REAL blockchain event!

---

## What's Configured

✅ **Webhook Receiver**: Running on port 8000
✅ **Database Endpoint**: Created and active
✅ **Contract**: Sepolia WETH (`0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9`)
✅ **Event**: Transfer events
✅ **Chain**: Sepolia Testnet (11155111)
✅ **All Services**: Running and healthy

---

## If No Webhooks After 10 Minutes

### Option 1: Catch ALL Events (High Volume!)

Update endpoint to match ALL Sepolia Transfer events:

```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
UPDATE endpoints
SET contract_addresses = NULL,
    event_signatures = ARRAY['Transfer(address,address,uint256)']
WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
"
```

**Result**: You'll get 50-100 webhooks per minute! 🌊

**Revert when done**:
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
UPDATE endpoints
SET contract_addresses = ARRAY['0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9']
WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
"
```

### Option 2: Check Event Ingestor

```bash
# See if events are being captured
docker logs ethhook-event-ingestor --tail 50 | grep -i transfer
```

If you see Transfer events → System is working, just waiting for WETH transfers specifically.

### Option 3: Test with Manual Event

Trigger your own WETH transfer on Sepolia:

1. Get Sepolia ETH: https://sepoliafaucet.com/
2. Wrap to WETH: Contract `0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9`
3. Transfer WETH to another address
4. See YOUR webhook arrive within 15 seconds!

---

## Verify in Grafana

Open: http://localhost:3001 (admin/admin)

**Check these metrics**:
- Events ingested (should be increasing)
- Webhooks delivered (should match your count)
- Success rate (should be 100%!)
- Delivery latency (should be < 100ms)

---

## Verify in Database

```bash
# Check successful deliveries
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
SELECT
    COUNT(*) as total_deliveries,
    COUNT(*) FILTER (WHERE success = true) as successful,
    AVG(duration_ms) as avg_latency_ms
FROM delivery_attempts
WHERE endpoint_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
  AND attempted_at > NOW() - INTERVAL '1 hour';
"
```

**Expected**:
```
total_deliveries | successful | avg_latency_ms
-----------------+------------+----------------
       15        |     15     |      12.5
```

---

## Success Criteria

After testing for 15-30 minutes:

- [x] ✅ Webhook receiver running on port 8000
- [x] ✅ Database endpoint created and active
- [ ] ⏳ At least 1 webhook received
- [ ] ⏳ 100% delivery success rate
- [ ] ⏳ Grafana showing metrics
- [ ] ⏳ Database shows delivery history
- [ ] ⏳ No errors in service logs

**When all checked** → Your system is PRODUCTION-READY! 🚀

---

## Next Steps After Successful Test

1. ✅ Take screenshots of webhook receiver
2. ✅ Take screenshots of Grafana dashboards
3. ✅ Check database statistics
4. ✅ Read [PRODUCTION_READY_SUMMARY.md](PRODUCTION_READY_SUMMARY.md)
5. ✅ Deploy to Railway this weekend!

---

## Stop Testing

When done testing:

**Stop webhook receiver**: Press `Ctrl+C`

**Keep services running**: Docker containers continue running

**Clean up test endpoint** (optional):
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
DELETE FROM endpoints WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
"
```

---

## Files You Need

- ✅ `webhook_receiver.py` - Real webhook receiver (created)
- ✅ `test_real_webhooks.sh` - Quick start script (created)
- ✅ `REAL_WEBHOOK_TEST_GUIDE.md` - Detailed testing guide (created)
- ✅ Database endpoint - Configured and active (created)

---

## Troubleshooting

**Issue**: Webhook receiver won't start

```bash
# Check if port 8000 is available
lsof -i :8000

# Use different port
python3 webhook_receiver.py 8001

# Update endpoint URL
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
UPDATE endpoints
SET webhook_url = 'http://host.docker.internal:8001/webhook'
WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
"
```

**Issue**: Docker can't reach webhook receiver

```bash
# Test connection from Docker
docker exec ethhook-webhook-delivery curl -X POST \
  http://host.docker.internal:8000/webhook \
  -H "Content-Type: application/json" \
  -d '{"test": "connection"}'
```

Check webhook receiver - you should see this test message.

**If on Linux**, use `172.17.0.1` instead of `host.docker.internal`

---

## Quick Reference

| What | Command |
|------|---------|
| **Start webhook receiver** | `./test_real_webhooks.sh` |
| **View event logs** | `docker logs -f ethhook-event-ingestor` |
| **View delivery logs** | `docker logs -f ethhook-webhook-delivery` |
| **Open Grafana** | `http://localhost:3001` |
| **Check database** | `docker exec -it ethhook-postgres psql -U ethhook` |
| **Check Redis** | `redis-cli XREAD COUNT 10 STREAMS events:11155111 0` |

---

**Ready?** Run this now:

```bash
./test_real_webhooks.sh
```

**Then wait for the magic!** ✨

---

**Created**: 2025-10-22
**Status**: ✅ Ready to test immediately
**Time**: 2 minutes to start, 1-10 minutes to see first webhook
**Result**: Real production webhook delivery with real blockchain data!
