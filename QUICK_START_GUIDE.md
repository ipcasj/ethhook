# Quick Start: Populating Your EthHook Instance

**For:** Testing and Demo Purposes  
**Time:** 5 minutes  
**Date:** November 2, 2025

---

## Why Are Events and Endpoints Empty?

✅ **This is completely normal!** Here's the data flow:

```
User → Creates Application → Creates Endpoint → Blockchain Events Flow In
```

**Current State:**
- **Empty Endpoints** = You haven't configured any webhook endpoints yet
- **Empty Events** = No endpoints configured, so no events to capture

---

## Quick Setup Guide

### Step 1: Check Your Applications (1 minute)

1. Navigate to **Applications** page
2. You should see existing applications (if using demo user)
3. If empty, click **"Create Application"**:
   - **Name:** `My Test App`
   - **Description:** `Testing webhook events`
   - Click **"Create Application"**
4. ✅ Copy your **API Key** and **Webhook Secret** (you'll need these later)

### Step 2: Create an Endpoint (2 minutes)

1. Navigate to **Endpoints** page
2. Click **"Add Endpoint"** button
3. Fill in the form:

   **Basic Info:**
   - **Application:** Select the app you just created
   - **Name:** `Ethereum Transfers`
   - **Webhook URL:** `http://localhost:5001/webhook` (demo receiver)

   **Blockchain Configuration:**
   - **Select Chains:** ✅ Ethereum Mainnet, ✅ Sepolia Testnet
   
   **Filters (Optional but Recommended):**
   - **Contract Address:** `0xdAC17F958D2ee523a2206206994597C13D831ec7` (USDT Token)
   - **Event Signature:** `Transfer(address,address,uint256)`

4. Click **"Add Endpoint"** to save

✅ **Your endpoint is now configured!**

### Step 3: Wait for Events (Automatic)

**How Events Flow In:**

```
Blockchain → Event Ingestor → Message Queue → Webhook Delivery → Your Endpoint
```

**Timeline:**
- **Real Blockchain:** Events appear when matching on-chain activity occurs (could be minutes to hours)
- **Test Data:** If you want immediate results, you need the full backend stack running

---

## Verify Your Setup

### Check Endpoint Configuration

Navigate to **Endpoints** page and verify you see:

```
✅ Name: Ethereum Transfers
✅ Status: Active
✅ Chains: 2 selected (Ethereum, Sepolia)
✅ Webhook URL: http://localhost:5001/webhook
```

### Monitor for Events

Navigate to **Events** page:

- **Empty initially** = Normal, waiting for blockchain activity
- **Events appear** = Your endpoint is working!

---

## Current System Status

Let me show you what's running:

✅ **UI:** http://localhost:3000 (running)  
✅ **Admin API:** Running (backend)  
✅ **Demo Webhook Receiver:** http://localhost:5001 (running)

⚠️ **For events to flow, you also need:**
- Event Ingestor service (monitors blockchain)
- Message Processor service (processes events)
- Webhook Delivery service (sends to your endpoint)

---

## Testing with Real Data

### Option A: Wait for Real Blockchain Events
**Pro:** Authentic test  
**Con:** Could take time (depends on blockchain activity matching your filters)

### Option B: Full Local Stack
**Requires:**
1. PostgreSQL running
2. Redis running
3. Event Ingestor running: `cargo run -p event-ingestor`
4. Message Processor running: `cargo run -p message-processor`
5. Webhook Delivery running: `cargo run -p webhook-delivery`

### Option C: Mock Testing
**For development only:**
- Inject test events directly into the database
- Verify UI displays them correctly
- Test webhook delivery to demo receiver

---

## Expected Results

### After 15-30 Minutes (with full stack):

**Applications Page:**
```
✅ My Test App
   API Key: eth_xxxxxxxxxxxxx
   Webhook Secret: whsec_xxxxxxxxxx
   Status: Active
```

**Endpoints Page:**
```
✅ Ethereum Transfers
   Application: My Test App
   URL: http://localhost:5001/webhook
   Chains: Ethereum Mainnet, Sepolia Testnet
   Status: Active
```

**Events Page:**
```
✅ Transfer Event - Chain: Ethereum
   Contract: 0xdAC17F958D2ee523a2206206994597C13D831ec7
   Block: 18923456
   Timestamp: 2 minutes ago
   Status: Delivered ✓
```

**Dashboard:**
```
Applications: 1
Endpoints: 1  
Events (24h): 5
Deliveries (24h): 5
```

---

## Demo User Credentials

If testing with the demo account:

```
Email: demo@ethhook.com
Password: Demo1234!
```

This account should already have test data pre-populated.

---

## Troubleshooting

### ❌ "No applications found"
**Solution:** Click "Create Application" button and set one up

### ❌ "No endpoints configured"  
**Solution:** Click "Add Endpoint" and configure one (see Step 2 above)

### ❌ "No events yet" (after 30+ minutes)
**Possible Causes:**
1. Event Ingestor not running → Start with `cargo run -p event-ingestor`
2. No matching blockchain activity → Broaden your filters (remove contract address/event signature)
3. Webhook delivery failing → Check demo receiver logs
4. Database not connected → Verify PostgreSQL is running

### ❌ Events showing but "Delivery Failed"
**Check:**
1. Webhook URL is reachable: `curl http://localhost:5001/webhook`
2. Demo receiver is running: `ps aux | grep receiver.py`
3. Check webhook delivery logs for errors

---

## Next Steps

Once you have events flowing:

1. ✅ **Monitor Dashboard** - See real-time statistics
2. ✅ **View Event Details** - Click any event to see full payload
3. ✅ **Test Webhook Delivery** - Check demo receiver logs
4. ✅ **Edit Endpoints** - Update filters to capture different events
5. ✅ **Create More Apps** - Test multi-application scenarios

---

## Production Considerations

When deploying to production:

1. **Replace demo webhook URL** with your actual endpoint
2. **Secure your webhook** using the HMAC secret
3. **Monitor delivery status** for failed webhooks
4. **Set up alerts** for high failure rates
5. **Scale horizontally** as event volume increases

---

**Current Status:** ✅ Empty state is normal and expected  
**Action Required:** Create your first endpoint to start capturing events  
**Time to First Event:** 15-30 minutes (with full stack running)
