# Customer Integration Guide

Complete guide for integrating EthHook webhooks into your application.

## Overview

EthHook delivers blockchain events to your webhook endpoint via HTTP POST requests. You deploy your own webhook receiver in your infrastructure, and EthHook sends events to it.

## Quick Start

### 1. Create an Endpoint

Via Admin API or Web Portal:

```bash
POST /api/v1/endpoints
{
  "name": "USDC Transfers",
  "webhook_url": "https://your-api.com/webhooks/ethereum",
  "chain_ids": [1],
  "contract_addresses": ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
  "event_signatures": ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"]
}
```

### 2. Implement Webhook Receiver

Choose your language:

- [Python](#python-example)
- [Node.js / TypeScript](#nodejs-example)
- [Go](#go-example)
- [Rust](#rust-example)
- [PHP](#php-example)

### 3. Verify HMAC Signatures

Always verify webhook signatures to ensure authenticity.

---

## Webhook Format

### HTTP Request

```http
POST https://your-api.com/webhooks/ethereum
Content-Type: application/json
x-webhook-id: 550e8400-e29b-41d4-a716-446655440000
x-webhook-signature: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
x-webhook-attempt: 1
```

### Payload

```json
{
  "block_hash": "0x3a64807006fa59f736cf9c7566b3ead6afc220017df7d689bd66d4594358986f",
  "block_number": 18500000,
  "chain_id": 1,
  "contract_address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
  "data": "0x0000000000000000000000000000000000000000000000000000000005f5e100",
  "log_index": 42,
  "timestamp": 1698898191,
  "topics": [
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
    "0x000000000000000000000000742d35cc6634c0532925a3b844bc9e7595f0beb6",
    "0x000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
  ],
  "transaction_hash": "0xdee13f29a8b8f483a3fc44e533bb745fa3568c7695672ca80c1ee3f353ef294d"
}
```

### Headers

| Header | Description |
|--------|-------------|
| `x-webhook-id` | Unique UUID for this webhook delivery |
| `x-webhook-signature` | HMAC-SHA256 signature of the payload |
| `x-webhook-attempt` | Delivery attempt number (1, 2, 3...) |

---

## Python Example

### Flask

```python
import hmac
import hashlib
from flask import Flask, request, jsonify

app = Flask(__name__)
HMAC_SECRET = "your-hmac-secret-from-ethhook"

def verify_signature(payload: bytes, signature: str) -> bool:
    expected = hmac.new(
        HMAC_SECRET.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(signature, expected)

@app.route('/webhooks/ethereum', methods=['POST'])
def handle_webhook():
    # Verify signature
    payload = request.get_data()
    signature = request.headers.get('x-webhook-signature', '')
    
    if not verify_signature(payload, signature):
        return jsonify({'error': 'Invalid signature'}), 401
    
    # Process event
    data = request.get_json()
    webhook_id = request.headers.get('x-webhook-id')
    
    print(f"Received event: {data['contract_address']} at block {data['block_number']}")
    
    # Your business logic here
    process_blockchain_event(data)
    
    # Return 200 to acknowledge receipt
    return jsonify({'status': 'success', 'webhook_id': webhook_id}), 200

def process_blockchain_event(event):
    """Your business logic"""
    # Example: Save to database
    # db.events.insert(event)
    
    # Example: Send notification
    # notify_user(event['contract_address'], event['transaction_hash'])
    
    # Example: Trigger workflow
    # if event['contract_address'] == USDC_ADDRESS:
    #     process_usdc_transfer(event)
    pass

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

### FastAPI

```python
from fastapi import FastAPI, Request, HTTPException, Header
import hmac
import hashlib

app = FastAPI()
HMAC_SECRET = "your-hmac-secret-from-ethhook"

def verify_signature(payload: bytes, signature: str) -> bool:
    expected = hmac.new(
        HMAC_SECRET.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(signature, expected)

@app.post("/webhooks/ethereum")
async def handle_webhook(
    request: Request,
    x_webhook_id: str = Header(...),
    x_webhook_signature: str = Header(...),
    x_webhook_attempt: int = Header(...)
):
    # Get raw body for signature verification
    body = await request.body()
    
    # Verify signature
    if not verify_signature(body, x_webhook_signature):
        raise HTTPException(status_code=401, detail="Invalid signature")
    
    # Parse JSON
    data = await request.json()
    
    # Process event
    await process_blockchain_event(data)
    
    return {
        "status": "success",
        "webhook_id": x_webhook_id,
        "processed_at": datetime.now().isoformat()
    }
```

---

## Node.js Example

### Express

```javascript
const express = require('express');
const crypto = require('crypto');

const app = express();
const HMAC_SECRET = 'your-hmac-secret-from-ethhook';

// Use raw body for signature verification
app.use(express.json({
  verify: (req, res, buf) => {
    req.rawBody = buf;
  }
}));

function verifySignature(payload, signature) {
  const expected = crypto
    .createHmac('sha256', HMAC_SECRET)
    .update(payload)
    .digest('hex');
  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expected)
  );
}

app.post('/webhooks/ethereum', (req, res) => {
  // Verify signature
  const signature = req.headers['x-webhook-signature'];
  
  if (!verifySignature(req.rawBody, signature)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  // Process event
  const event = req.body;
  const webhookId = req.headers['x-webhook-id'];
  
  console.log(`Received event at block ${event.block_number}`);
  
  // Your business logic
  processBlockchainEvent(event);
  
  // Acknowledge receipt
  res.json({
    status: 'success',
    webhook_id: webhookId,
    processed_at: new Date().toISOString()
  });
});

function processBlockchainEvent(event) {
  // Your business logic here
  // - Save to database
  // - Send notifications
  // - Update state
  // - Trigger workflows
}

app.listen(3000, () => {
  console.log('Webhook receiver listening on port 3000');
});
```

### TypeScript + Express

```typescript
import express, { Request, Response } from 'express';
import crypto from 'crypto';

interface BlockchainEvent {
  block_hash: string;
  block_number: number;
  chain_id: number;
  contract_address: string;
  data: string;
  log_index: number;
  timestamp: number;
  topics: string[];
  transaction_hash: string;
}

const app = express();
const HMAC_SECRET = process.env.HMAC_SECRET || '';

app.use(express.json({
  verify: (req: any, res, buf) => {
    req.rawBody = buf;
  }
}));

function verifySignature(payload: Buffer, signature: string): boolean {
  const expected = crypto
    .createHmac('sha256', HMAC_SECRET)
    .update(payload)
    .digest('hex');
  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expected)
  );
}

app.post('/webhooks/ethereum', async (req: any, res: Response) => {
  const signature = req.headers['x-webhook-signature'] as string;
  const webhookId = req.headers['x-webhook-id'] as string;
  
  if (!verifySignature(req.rawBody, signature)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  const event: BlockchainEvent = req.body;
  
  try {
    await processBlockchainEvent(event);
    
    res.json({
      status: 'success',
      webhook_id: webhookId,
      processed_at: new Date().toISOString()
    });
  } catch (error) {
    console.error('Error processing webhook:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

async function processBlockchainEvent(event: BlockchainEvent): Promise<void> {
  console.log(`Processing event from ${event.contract_address}`);
  
  // Your business logic
  // await db.events.insert(event);
  // await notifyUsers(event);
}

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Webhook receiver listening on port ${PORT}`);
});
```

---

## Go Example

```go
package main

import (
    "crypto/hmac"
    "crypto/sha256"
    "crypto/subtle"
    "encoding/hex"
    "encoding/json"
    "io"
    "log"
    "net/http"
    "time"
)

const hmacSecret = "your-hmac-secret-from-ethhook"

type BlockchainEvent struct {
    BlockHash       string   `json:"block_hash"`
    BlockNumber     int64    `json:"block_number"`
    ChainID         int      `json:"chain_id"`
    ContractAddress string   `json:"contract_address"`
    Data            string   `json:"data"`
    LogIndex        int      `json:"log_index"`
    Timestamp       int64    `json:"timestamp"`
    Topics          []string `json:"topics"`
    TransactionHash string   `json:"transaction_hash"`
}

func verifySignature(payload []byte, signature string) bool {
    mac := hmac.New(sha256.New, []byte(hmacSecret))
    mac.Write(payload)
    expected := hex.EncodeToString(mac.Sum(nil))
    return subtle.ConstantTimeCompare([]byte(signature), []byte(expected)) == 1
}

func webhookHandler(w http.ResponseWriter, r *http.Request) {
    if r.Method != http.MethodPost {
        http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
        return
    }

    // Read body
    body, err := io.ReadAll(r.Body)
    if err != nil {
        http.Error(w, "Failed to read body", http.StatusBadRequest)
        return
    }
    defer r.Body.Close()

    // Verify signature
    signature := r.Header.Get("x-webhook-signature")
    if !verifySignature(body, signature) {
        http.Error(w, "Invalid signature", http.StatusUnauthorized)
        return
    }

    // Parse event
    var event BlockchainEvent
    if err := json.Unmarshal(body, &event); err != nil {
        http.Error(w, "Invalid JSON", http.StatusBadRequest)
        return
    }

    webhookID := r.Header.Get("x-webhook-id")
    log.Printf("Received event %s from contract %s at block %d",
        webhookID, event.ContractAddress, event.BlockNumber)

    // Process event
    if err := processBlockchainEvent(&event); err != nil {
        log.Printf("Error processing event: %v", err)
        http.Error(w, "Internal server error", http.StatusInternalServerError)
        return
    }

    // Respond with success
    response := map[string]interface{}{
        "status":       "success",
        "webhook_id":   webhookID,
        "processed_at": time.Now().Format(time.RFC3339),
    }
    
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(response)
}

func processBlockchainEvent(event *BlockchainEvent) error {
    // Your business logic here
    log.Printf("Processing event from %s", event.ContractAddress)
    
    // Example: Save to database
    // db.SaveEvent(event)
    
    // Example: Send notification
    // notifyUsers(event)
    
    return nil
}

func main() {
    http.HandleFunc("/webhooks/ethereum", webhookHandler)
    
    log.Println("Webhook receiver listening on :8080")
    if err := http.ListenAndServe(":8080", nil); err != nil {
        log.Fatal(err)
    }
}
```

---

## Rust Example

```rust
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
struct BlockchainEvent {
    block_hash: String,
    block_number: u64,
    chain_id: u32,
    contract_address: String,
    data: String,
    log_index: u32,
    timestamp: u64,
    topics: Vec<String>,
    transaction_hash: String,
}

#[derive(Serialize)]
struct WebhookResponse {
    status: String,
    webhook_id: String,
    processed_at: String,
}

fn verify_signature(payload: &[u8], signature: &str, secret: &str) -> bool {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);
    
    let expected = hex::encode(mac.finalize().into_bytes());
    expected == signature
}

async fn webhook_handler(
    req: HttpRequest,
    body: web::Bytes,
) -> actix_web::Result<HttpResponse> {
    // Get headers
    let signature = req
        .headers()
        .get("x-webhook-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing signature"))?;
    
    let webhook_id = req
        .headers()
        .get("x-webhook-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Verify signature
    let secret = env::var("HMAC_SECRET").expect("HMAC_SECRET must be set");
    if !verify_signature(&body, signature, &secret) {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid signature"
        })));
    }

    // Parse event
    let event: BlockchainEvent = serde_json::from_slice(&body)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid JSON"))?;

    println!("Received event from {} at block {}", 
        event.contract_address, event.block_number);

    // Process event
    process_blockchain_event(&event).await?;

    // Respond
    Ok(HttpResponse::Ok().json(WebhookResponse {
        status: "success".to_string(),
        webhook_id: webhook_id.to_string(),
        processed_at: chrono::Utc::now().to_rfc3339(),
    }))
}

async fn process_blockchain_event(event: &BlockchainEvent) -> actix_web::Result<()> {
    // Your business logic here
    println!("Processing event from {}", event.contract_address);
    
    // Example: Save to database
    // db::save_event(event).await?;
    
    // Example: Send notification
    // notify_users(event).await?;
    
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("Webhook receiver listening on 0.0.0.0:8080");
    
    HttpServer::new(|| {
        App::new()
            .route("/webhooks/ethereum", web::post().to(webhook_handler))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

---

## PHP Example

```php
<?php

function verifySignature($payload, $signature, $secret) {
    $expected = hash_hmac('sha256', $payload, $secret);
    return hash_equals($signature, $expected);
}

function processBlockchainEvent($event) {
    // Your business logic here
    error_log("Processing event from " . $event['contract_address']);
    
    // Example: Save to database
    // $db->insert('events', $event);
    
    // Example: Send notification
    // sendNotification($event);
}

// Get raw POST body
$payload = file_get_contents('php://input');
$event = json_decode($payload, true);

// Get headers
$signature = $_SERVER['HTTP_X_WEBHOOK_SIGNATURE'] ?? '';
$webhookId = $_SERVER['HTTP_X_WEBHOOK_ID'] ?? '';
$hmacSecret = getenv('HMAC_SECRET');

// Verify signature
if (!verifySignature($payload, $signature, $hmacSecret)) {
    http_response_code(401);
    echo json_encode(['error' => 'Invalid signature']);
    exit;
}

// Process event
try {
    processBlockchainEvent($event);
    
    http_response_code(200);
    echo json_encode([
        'status' => 'success',
        'webhook_id' => $webhookId,
        'processed_at' => date('c')
    ]);
} catch (Exception $e) {
    error_log("Error processing webhook: " . $e->getMessage());
    http_response_code(500);
    echo json_encode(['error' => 'Internal server error']);
}
```

---

## Best Practices

### 1. Always Verify Signatures

```python
# ✅ Good - Verify before processing
if verify_signature(payload, signature):
    process_event(data)

# ❌ Bad - Processing without verification
process_event(data)
```

### 2. Return 200 Quickly

```python
# ✅ Good - Acknowledge first, process async
@app.route('/webhook', methods=['POST'])
def webhook():
    data = request.json
    # Queue for background processing
    queue.enqueue(process_event, data)
    return {'status': 'accepted'}, 200

# ❌ Bad - Slow processing blocks webhook
@app.route('/webhook', methods=['POST'])
def webhook():
    data = request.json
    slow_database_operation(data)  # Takes 30 seconds
    return {'status': 'success'}, 200
```

### 3. Handle Retries Gracefully

```python
# Use webhook_id for idempotency
@app.route('/webhook', methods=['POST'])
def webhook():
    webhook_id = request.headers.get('x-webhook-id')
    
    # Check if already processed
    if db.webhook_processed(webhook_id):
        return {'status': 'already_processed'}, 200
    
    # Process and mark as done
    process_event(request.json)
    db.mark_processed(webhook_id)
    
    return {'status': 'success'}, 200
```

### 4. Log Everything

```python
import logging

logger = logging.getLogger(__name__)

@app.route('/webhook', methods=['POST'])
def webhook():
    webhook_id = request.headers.get('x-webhook-id')
    attempt = request.headers.get('x-webhook-attempt')
    
    logger.info(f"Received webhook {webhook_id} (attempt {attempt})")
    
    try:
        process_event(request.json)
        logger.info(f"Successfully processed {webhook_id}")
    except Exception as e:
        logger.error(f"Failed to process {webhook_id}: {e}")
        raise
```

### 5. Monitor and Alert

```python
from prometheus_client import Counter

webhook_received = Counter('webhooks_received_total', 'Total webhooks received')
webhook_processed = Counter('webhooks_processed_total', 'Total webhooks processed')
webhook_failed = Counter('webhooks_failed_total', 'Total webhooks failed')

@app.route('/webhook', methods=['POST'])
def webhook():
    webhook_received.inc()
    
    try:
        process_event(request.json)
        webhook_processed.inc()
        return {'status': 'success'}, 200
    except Exception as e:
        webhook_failed.inc()
        raise
```

---

## Testing Your Integration

### 1. Use the EthHook Test Mode

Test endpoints without real blockchain events:

```bash
curl -X POST https://your-api.com/api/v1/endpoints/test \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "endpoint_id": "your-endpoint-id"
  }'
```

### 2. Use webhook.site

For quick testing without deploying:

1. Go to <https://webhook.site>
2. Copy your unique URL
3. Use it as your webhook_url in EthHook
4. View incoming webhooks in real-time

### 3. Use ngrok for Local Testing

```bash
# Expose local port 5000
ngrok http 5000

# Use the ngrok URL in EthHook
# e.g., https://abc123.ngrok.io/webhooks/ethereum
```

---

## Troubleshooting

### Webhook Not Received

1. **Check endpoint URL**: Must be publicly accessible HTTPS
2. **Check firewall**: Allow incoming HTTPS traffic
3. **Check logs**: Look for delivery attempts in EthHook dashboard
4. **Test manually**:

   ```bash
   curl -X POST https://your-api.com/webhooks/ethereum \
     -H "Content-Type: application/json" \
     -d '{"test": true}'
   ```

### Signature Verification Fails

1. **Check secret**: Must match exactly (no extra spaces/newlines)
2. **Check body**: Use raw body bytes, not parsed JSON
3. **Check encoding**: HMAC-SHA256, hex-encoded lowercase
4. **Test signature**:

   ```python
   import hmac, hashlib
   expected = hmac.new(
       secret.encode(),
       payload.encode(),
       hashlib.sha256
   ).hexdigest()
   print(f"Expected: {expected}")
   ```

### Webhooks Timing Out

1. **Process async**: Queue work for background processing
2. **Return 200 quickly**: Acknowledge within 5 seconds
3. **Scale horizontally**: Add more webhook receiver instances
4. **Check logs**: Look for slow database queries or external API calls

---

## Security Considerations

### 1. Always Use HTTPS

```text
✅ https://your-api.com/webhooks
❌ http://your-api.com/webhooks
```

### 2. Verify Signatures

Prevents replay attacks and unauthorized requests.

### 3. Use Environment Variables

```python
# ✅ Good
HMAC_SECRET = os.environ['HMAC_SECRET']

# ❌ Bad - Secret in code
HMAC_SECRET = "my-secret-123"
```

### 4. Rate Limit

```python
from flask_limiter import Limiter

limiter = Limiter(app, default_limits=["100 per minute"])

@app.route('/webhook')
@limiter.limit("60 per minute")
def webhook():
    pass
```

### 5. Validate Input

```python
def validate_event(event):
    required = ['block_number', 'chain_id', 'contract_address']
    for field in required:
        if field not in event:
            raise ValueError(f"Missing field: {field}")
    
    if not isinstance(event['block_number'], int):
        raise ValueError("Invalid block_number type")
    
    if not event['contract_address'].startswith('0x'):
        raise ValueError("Invalid contract address format")
```

---

## Support

Need help integrating webhooks?

- **Documentation**: <https://ethhook.example.com/docs>
- **Discord**: <https://discord.gg/ethhook>
- **Email**: <support@ethhook.example.com>
- **GitHub**: <https://github.com/yourorg/ethhook/issues>

## Examples Repository

Find more examples at:
<https://github.com/yourorg/ethhook-examples>

- Python (Flask, FastAPI, Django)
- Node.js (Express, NestJS, Koa)
- Go (net/http, Gin, Echo)
- Rust (Actix, Axum, Rocket)
- PHP (Laravel, Symfony)
- Ruby (Rails, Sinatra)
- Java (Spring Boot)
- C# (.NET Core)
