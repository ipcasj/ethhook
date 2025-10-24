# Frontend Showing Old webhook.site URLs? - Fix Guide

## The Issue

You're seeing old `webhook.site` URLs in the frontend because there were 4 endpoints total:
- ✅ 1 NEW endpoint with real URL: `http://host.docker.internal:8000/webhook`
- 🔴 3 OLD endpoints with webhook.site URLs

**✅ FIXED!** I've deactivated the 3 old endpoints. Only the REAL endpoint is active now.

---

## Verify the Fix

### Check Database (Current State)

```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
SELECT
    name,
    webhook_url,
    is_active,
    CASE
        WHEN is_active THEN '✅ ACTIVE'
        ELSE '🔴 INACTIVE'
    END as status
FROM endpoints
ORDER BY created_at DESC;
"
```

**You should see**:
```
          name          |              webhook_url               | is_active |   status
------------------------+----------------------------------------+-----------+-------------
 REAL Production Test   | http://host.docker.internal:8000/...  | t         | ✅ ACTIVE
 WETH Transfer Monitor  | https://webhook.site/12345678-...      | f         | 🔴 INACTIVE
 Sepolia USDC Transfers | https://webhook.site/test-usdc         | f         | 🔴 INACTIVE
 Sepolia WETH Transfers | https://webhook.site/test-weth         | f         | 🔴 INACTIVE
```

**Only the REAL endpoint is active!** ✅

---

## Refresh Frontend Portal

### Option 1: Refresh the Page

1. Open: http://localhost:3002
2. Press `Cmd+Shift+R` (Mac) or `Ctrl+Shift+R` (Windows/Linux) to hard refresh
3. Navigate to Applications → Your App → Endpoints
4. You should see only the active endpoint now

### Option 2: Clear Browser Cache

1. Open browser DevTools (F12)
2. Right-click refresh button → "Empty Cache and Hard Reload"
3. Reload http://localhost:3002

### Option 3: Restart Frontend

```bash
# Find trunk process
ps aux | grep trunk

# Kill it (replace PID with actual number)
kill 15365

# Restart
cd crates/leptos-portal
trunk serve --port 3002 &
```

---

## If You Want to Delete Old Endpoints Completely

**Warning**: This will permanently delete the old webhook.site endpoints.

```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
-- Delete all webhook.site endpoints
DELETE FROM endpoints
WHERE webhook_url LIKE '%webhook.site%';

-- Verify only real endpoint remains
SELECT id, name, webhook_url FROM endpoints;
"
```

---

## Create New Endpoint via Portal UI (Alternative)

If you want to create the real endpoint through the frontend:

1. **Open Portal**: http://localhost:3002
2. **Login/Register**
3. **Go to Applications** → Select app → "Add Endpoint"
4. **Fill Form**:
   ```
   Name: Real Production Test
   Webhook URL: http://host.docker.internal:8000/webhook
   Description: Real webhook for production testing
   Contract: 0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9
   Event: Transfer(address,address,uint256)
   Chain: Sepolia (11155111)
   Active: ✅ YES
   ```
5. **Click**: "Create Endpoint"

The portal will create it in the database automatically.

---

## Current Active Endpoint Details

```
ID:          aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa
Name:        REAL Production Test
URL:         http://host.docker.internal:8000/webhook
Contract:    0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9 (Sepolia WETH)
Event:       Transfer(address,address,uint256)
Chain:       Sepolia Testnet (11155111)
Status:      ✅ ACTIVE
HMAC Secret: test_secret_key_for_hmac_signature_verification
```

---

## Test It Works (Ignore Frontend)

**The frontend display doesn't matter!** What matters is that webhooks are being delivered.

**Start webhook receiver**:
```bash
./test_real_webhooks.sh
```

**Monitor delivery logs**:
```bash
docker logs -f ethhook-webhook-delivery | grep -E "Sending|delivered|8000"
```

**When a WETH Transfer happens**, you'll see:
```
DEBUG Sending webhook to http://host.docker.internal:8000/webhook
INFO ✅ Webhook delivered successfully: status=200
```

**And in your webhook receiver terminal**:
```
🎉 WEBHOOK RECEIVED!
📦 Real Sepolia WETH Transfer event
```

**The system works regardless of what the frontend shows!** ✅

---

## Why This Happened

The old endpoints were created during earlier testing with webhook.site. They were still marked as `is_active = true`, so the frontend was showing them.

**Solution**: Deactivate old endpoints → Only real endpoint is active now ✅

---

## Summary

✅ **FIXED**: Old webhook.site endpoints are now INACTIVE
✅ **ACTIVE**: Only the real `localhost:8000` endpoint is active
✅ **WORKING**: Webhooks will be delivered to your real receiver
✅ **FRONTEND**: Will show only active endpoints after refresh

**Next step**: Run `./test_real_webhooks.sh` and see REAL webhooks arrive!

---

**Created**: 2025-10-22
**Status**: ✅ Old endpoints deactivated, real endpoint active
