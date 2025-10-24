# ⚡ Test EthHook RIGHT NOW (5 Minutes) ⚡

**Your system is LIVE!** Test it as your first real client.

---

## Step 1: Open Frontend (30 seconds)

```
http://localhost:3002
```

**You'll see**: EthHook Portal with modern gradient UI

---

## Step 2: Login/Register (1 minute)

**Option A**: Use existing account
- Email: `demo@ethhook.com` or any from database

**Option B**: Create YOUR account
- Click "Register"
- Name: Igor
- Email: your email
- Password: (your choice)

---

## Step 3: Get Webhook Test URL (30 seconds)

Open new tab:
```
https://webhook.site/
```

**Copy the unique URL** that appears:
```
https://webhook.site/abc123-def456-...
```

**Keep this tab open!** You'll see webhooks arrive here.

---

## Step 4: Create Endpoint (2 minutes)

In EthHook portal:

1. **Click** "Create Application" or use existing
2. **Click** "Add Endpoint"
3. **Fill in**:
   ```
   Webhook URL:
   (paste from webhook.site)

   Description:
   My first real webhook test

   Chain:
   Sepolia Testnet (11155111)

   Contract Address:
   0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9

   Event Signature:
   Transfer(address,address,uint256)

   Active:
   ✅ YES
   ```
4. **Click** "Create"

---

## Step 5: Wait for Event (1-5 minutes)

Your system is NOW watching Sepolia blockchain for WETH Transfer events!

**Watch logs in real-time**:
```bash
# Terminal 1: Event capture
docker logs -f ethhook-event-ingestor | grep Transfer

# Terminal 2: Webhook delivery
docker logs -f ethhook-webhook-delivery | grep "Sending webhook"
```

**What you'll see**:
```
INFO [Sepolia Testnet] Processing block #9469712
INFO Found Transfer event: contract=0x7b79...
DEBUG Sending webhook to https://webhook.site/abc123...
INFO ✅ Webhook delivered successfully: status=200
```

---

## Step 6: Check Webhook Received! 🎉

**Go back to webhook.site tab**

**You'll see**:
```
NEW REQUEST RECEIVED!
POST https://webhook.site/abc123...
```

**JSON Body**:
```json
{
  "event_id": "...",
  "chain_id": 11155111,
  "chain_name": "Sepolia Testnet",
  "block_number": 9469712,
  "transaction_hash": "0x...",
  "contract_address": "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9",
  "event_signature": "Transfer(address,address,uint256)",
  "decoded": {
    "from": "0x...",
    "to": "0x...",
    "value": "1000000000000000000"
  }
}
```

**Headers**:
```
X-Webhook-Signature: sha256=...
X-Chain-Id: 11155111
X-Event-Id: ...
```

---

## ✅ SUCCESS!

**You just**:
- ✅ Used EthHook as a real client
- ✅ Configured a webhook endpoint
- ✅ Received a real blockchain event
- ✅ Verified HMAC signature present
- ✅ Proved the system works end-to-end

---

## Troubleshooting

### No webhook after 5 minutes?

**Option 1**: Check logs for delivery
```bash
docker logs ethhook-webhook-delivery --tail 50
```

**Option 2**: Use wildcard contract (catches ALL events)
```
Contract Address: 0x0000000000000000000000000000000000000000
```

**Option 3**: Check endpoint is active
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c \
  "SELECT id, webhook_url, is_active FROM endpoints ORDER BY created_at DESC LIMIT 1;"
```

**Option 4**: Manually test webhook.site URL
```bash
curl -X POST https://webhook.site/YOUR-ID \
  -H "Content-Type: application/json" \
  -d '{"test": "manual test"}'
```
Check webhook.site - you should see this test request.

---

## What's Next?

**After successful test**:
1. ✅ Read `PRODUCTION_READY_SUMMARY.md`
2. ✅ Deploy to Railway this weekend
3. ✅ Invite 5 beta users next week
4. ✅ Get first paying clients next month
5. ✅ Build your SaaS business! 🚀

---

**System Status**: 🟢 ALL SERVICES RUNNING
**Your Status**: 🎯 FIRST REAL CLIENT
**Time to test**: ⏱️ 5 MINUTES

**GO!** 🏃‍♂️💨
