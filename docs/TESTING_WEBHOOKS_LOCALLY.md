# Testing Webhooks Locally

This guide shows you how to test EthHook webhook deliveries on your local machine before deploying your own webhook endpoint.

## Quick Start

EthHook includes a sample webhook receiver script for local testing. This helps you:
- ✅ Verify webhook payloads
- ✅ Test HMAC signature verification
- ✅ Debug your integration locally
- ✅ See real-time blockchain events

## Starting the Webhook Receiver

### Option 1: Using the Included Script (Recommended)

```bash
# From the project root
python3 .private/webhook_receiver.py 8000
```

You should see:

```
🚀 REAL WEBHOOK RECEIVER STARTED!
📍 Listening on: http://0.0.0.0:8000
📍 Webhook URL:  http://localhost:8000/webhook

⏳ Waiting for webhooks from EthHook...
```

### Option 2: Using Docker Compose (for testing with containers)

If you're running EthHook in Docker, use `host.docker.internal:8000` as your webhook URL in endpoint configuration.

## Configuring an Endpoint

1. Open the Leptos Portal: http://localhost:3000
2. Navigate to **Endpoints**
3. Create a new endpoint with:
   - **Webhook URL**: `http://host.docker.internal:8000/webhook` (Docker) or `http://localhost:8000/webhook` (local)
   - **Contract Address**: e.g., `0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238` (USDC on Sepolia)
   - **Event Signature**: `0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef` (Transfer event)
   - **Chain ID**: `11155111` (Sepolia testnet)

## Example Webhook Payload

When a matching blockchain event occurs, you'll receive:

```json
{
  "block_hash": "0x3a64807006fa59f736cf9c7566b3ead6afc220017df7d689bd66d4594358986f",
  "block_number": 9481634,
  "chain_id": 11155111,
  "contract_address": "0x6815183051ec7c56e24c5915931b483f1cb4aacf",
  "data": "0x",
  "log_index": 0,
  "timestamp": 1761327480,
  "topics": [
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
    "0x000000000000000000000000786896bb6f6ff73c9fba9651d20a4a536ecd0bef",
    "0x00000000000000000000000026e8897215c4f4890590e532bf6e3eadfde8b245",
    "0x00000000000000000000000000000000000000000000000000000000000003e6"
  ],
  "transaction_hash": "0xdee13f29a8b8f483a3fc44e533bb745fa3568c7695672ca80c1ee3f353ef294d"
}
```

## Webhook Headers

Every webhook includes security headers:

```
X-Webhook-Signature: 6e9b8c52500096463e677739900be5349a11ea658546d6c1272c4442d3b2750e
X-Webhook-Id: 73ba15a8-09ff-4eec-87e0-0efe7f499384
X-Webhook-Attempt: 1
```

### Verifying HMAC Signature

To verify the webhook signature:

```python
import hmac
import hashlib

def verify_webhook(payload_bytes, signature, secret):
    """Verify webhook HMAC signature"""
    expected = hmac.new(
        secret.encode(),
        payload_bytes,
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(expected, signature)

# In your webhook handler:
signature = request.headers.get('X-Webhook-Signature')
timestamp = request.headers.get('X-Webhook-Timestamp')
payload = request.body

if verify_webhook(payload, signature, YOUR_HMAC_SECRET):
    # Webhook is authentic
    process_event(payload)
else:
    # Invalid signature - reject
    return 401
```

## Production Deployment

⚠️ **Important**: The included `webhook_receiver.py` is for **local testing only**.

In production, you should:

1. **Deploy Your Own Webhook Endpoint**
   - AWS Lambda / Azure Functions / Google Cloud Functions
   - Express.js / FastAPI / Flask server
   - Message queue (SQS, Pub/Sub, etc.)

2. **Use HTTPS**
   - EthHook supports HTTPS webhook URLs
   - Recommended for production security

3. **Implement Retry Logic**
   - EthHook retries failed deliveries with exponential backoff
   - Your endpoint should be idempotent (handle duplicate deliveries)

4. **Monitor Delivery Status**
   - Check the EthHook dashboard for delivery statistics
   - Set up alerts for failed deliveries

## Example Production Endpoints

### AWS Lambda (Python)
```python
import json
import hmac
import hashlib

def lambda_handler(event, context):
    """Handle EthHook webhook delivery"""
    
    # Verify signature
    signature = event['headers']['x-webhook-signature']
    secret = os.environ['ETHHOOK_SECRET']
    payload = event['body']
    
    if not verify_signature(payload, signature, secret):
        return {'statusCode': 401, 'body': 'Invalid signature'}
    
    # Process blockchain event
    data = json.loads(payload)
    process_transfer_event(data)
    
    return {'statusCode': 200, 'body': 'OK'}
```

### Express.js (Node.js)
```javascript
const express = require('express');
const crypto = require('crypto');

app.post('/webhook', express.raw({type: 'application/json'}), (req, res) => {
    // Verify HMAC signature
    const signature = req.headers['x-webhook-signature'];
    const secret = process.env.ETHHOOK_SECRET;
    
    const hmac = crypto.createHmac('sha256', secret);
    hmac.update(req.body);
    const expected = hmac.digest('hex');
    
    if (!crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expected))) {
        return res.status(401).send('Invalid signature');
    }
    
    // Process event
    const data = JSON.parse(req.body.toString());
    processBlockchainEvent(data);
    
    res.status(200).send('OK');
});
```

## Troubleshooting

### Webhooks Not Arriving?

1. **Check endpoint is active**
   ```sql
   SELECT name, is_active FROM endpoints WHERE webhook_url LIKE '%localhost%';
   ```

2. **Check for matching events**
   ```sql
   SELECT COUNT(*) FROM events 
   WHERE contract_address = '0x...' 
   AND ingested_at > NOW() - INTERVAL '10 minutes';
   ```

3. **Check delivery attempts**
   ```sql
   SELECT * FROM delivery_attempts 
   ORDER BY attempted_at DESC 
   LIMIT 10;
   ```

4. **Restart message-processor** (reprocesses recent events)
   ```bash
   docker restart ethhook-message-processor
   ```

### Connection Refused Error?

If you see "error sending request for url", your webhook receiver is not running:

```bash
# Start the receiver
python3 .private/webhook_receiver.py 8000
```

### Circuit Breaker Open?

After 5 consecutive failures, the circuit breaker opens for 30 seconds:

```bash
# Reset by restarting webhook-delivery
docker restart ethhook-webhook-delivery
```

## Next Steps

- 📖 Read [Event Types Documentation](POPULAR_ETHEREUM_EVENTS.md)
- 🔧 Explore [Admin API](../README.md#admin-api)
- 🚀 Deploy to production: [Deployment Guide](DEPLOYMENT_QUICKSTART.md)
