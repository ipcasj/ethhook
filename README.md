# EthHook 🦀

> Production-grade real-time Ethereum webhook service built in Rust

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

## 🚀 What is EthHook?

EthHook is a high-performance, self-hostable webhook service that delivers real-time Ethereum blockchain events to your applications. Built entirely in Rust, it provides:

- **⚡ Ultra-low latency**: <500ms from on-chain event to webhook delivery
- **🔒 Secure**: HMAC-signed webhooks with JWT authentication
- **📈 Scalable**: Handle 50k+ events/second with multi-threaded async architecture
- **💰 Cost-effective**: 5x lower infrastructure costs than Node.js alternatives
- **🎯 Developer-first**: Intuitive API, comprehensive docs, and WASM-powered dashboard

### Use Cases

- 🏦 **DeFi Protocols**: Monitor liquidity events, swaps, deposits
- 🎨 **NFT Marketplaces**: Track mints, transfers, sales
- 🏛️ **DAOs**: Governance proposal creation, voting
- 📊 **Analytics Platforms**: Real-time blockchain data ingestion
- 🔔 **Notification Services**: User-specific event alerts

## 🏗️ Architecture

EthHook consists of 4 Rust microservices:

Ethereum → Event Ingestor → Redis → Message Processor → Redis → Webhook Delivery → Your App
                                           ↓
                                      PostgreSQL ← Admin API ← Leptos Portal

1. **Event Ingestor**: WebSocket listener for Ethereum events
2. **Message Processor**: Event filtering, fan-out, and queuing
3. **Webhook Delivery**: Reliable HTTP delivery with retries
4. **Admin API**: REST API for managing subscriptions

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design.

## 🎯 Quick Start

### Prerequisites

- Rust 1.75+ (`rustup install stable`)
- Docker & Docker Compose
- PostgreSQL 15+
- Redis 7+
- Ethereum RPC endpoint (Infura/Alchemy free tier works)

### Local Development

```bash
# Clone the repository
git clone https://github.com/yourusername/ethhook.git
cd ethhook

# Set up environment variables
cp .env.example .env
# Edit .env with your RPC endpoint and database credentials

# Start infrastructure
docker compose up -d postgres redis

# Run database migrations
sqlx migrate run

# Start all services
cargo run --bin event-ingestor &
cargo run --bin message-processor &
cargo run --bin webhook-delivery &
cargo run --bin admin-api &

# Or use cargo-watch for development
cargo watch -x 'run --bin admin-api'
```

### Docker Compose (All-in-one)

```bash
docker compose up -d
```

Access the dashboard at [http://localhost:3000](http://localhost:3000)

## 🔧 Configuration

Create a `.env` file:

```bash
# Ethereum RPC
ETH_RPC_WS=wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID
ETH_RPC_HTTP=https://mainnet.infura.io/v3/YOUR_PROJECT_ID

# Database
DATABASE_URL=postgresql://ethhook:password@localhost/ethhook

# Redis
REDIS_URL=redis://localhost:6379

# API
JWT_SECRET=your-256-bit-secret
API_HOST=0.0.0.0
API_PORT=8080

# Observability
RUST_LOG=info,ethhook=debug
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Run integration tests (requires Docker)
cargo test --test integration_tests

# Run with coverage
cargo tarpaulin --out Html

# Load testing
k6 run tests/load/webhook_delivery.js
```

## 📊 Monitoring

EthHook exposes Prometheus metrics on `/metrics`:

ethhook_events_ingested_total
ethhook_webhooks_sent_total{status="success|failure"}
ethhook_webhook_delivery_latency_seconds
ethhook_active_endpoints

Import the Grafana dashboard from `monitoring/grafana-dashboard.json`.

## 🚀 Deployment

### DigitalOcean App Platform

```bash
# Install doctl
brew install doctl

# Authenticate
doctl auth init

# Deploy
doctl apps create --spec .do/app.yaml
```

### Kubernetes

```bash
# Apply manifests
kubectl apply -f k8s/
```

## 💡 Usage Example

### 1. Create an Application

```bash
curl -X POST https://api.ethhook.io/api/v1/applications \
  -H "Authorization: Bearer YOUR_JWT" \
  -d '{
    "name": "My dApp",
    "description": "NFT marketplace webhooks"
  }'
```

### 2. Create an Endpoint

```bash
curl -X POST https://api.ethhook.io/api/v1/applications/APP_ID/endpoints \
  -H "Authorization: Bearer YOUR_JWT" \
  -d '{
    "name": "NFT Transfers",
    "url": "https://myapp.com/webhooks/nft",
    "contract_address": "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D",
    "event_topics": [
      "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
    ]
  }'
```

### 3. Receive Webhooks

#### Python (Flask)

```python
import hmac
import hashlib
from flask import Flask, request

app = Flask(__name__)

@app.route('/webhooks/nft', methods=['POST'])
def handle_webhook():
    # Verify signature
    signature = request.headers.get('X-EthHook-Signature')
    payload = request.get_data()
    expected = 'sha256=' + hmac.new(
        b'your_webhook_secret',
        payload,
        hashlib.sha256
    ).hexdigest()
    
    if not hmac.compare_digest(signature, expected):
        return 'Invalid signature', 401
    
    event = request.get_json()
    print(f"NFT Transfer: {event['data']['transaction_hash']}")
    
    return 'OK', 200
```

#### Java (Spring Boot)

```java
import org.springframework.web.bind.annotation.*;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;
import java.security.MessageDigest;
import java.util.Map;

@RestController
public class WebhookController {
    
    private static final String WEBHOOK_SECRET = "your_webhook_secret";
    
    @PostMapping("/webhooks/nft")
    public String handleWebhook(
            @RequestHeader("X-EthHook-Signature") String signature,
            @RequestBody String payload) {
        
        try {
            // Verify signature
            Mac hmac = Mac.getInstance("HmacSHA256");
            SecretKeySpec secretKey = new SecretKeySpec(
                WEBHOOK_SECRET.getBytes(), "HmacSHA256");
            hmac.init(secretKey);
            
            byte[] hash = hmac.doFinal(payload.getBytes());
            String expected = "sha256=" + bytesToHex(hash);
            
            if (!MessageDigest.isEqual(signature.getBytes(), expected.getBytes())) {
                return "Invalid signature";
            }
            
            // Parse JSON and process event
            // ObjectMapper mapper = new ObjectMapper();
            // Map<String, Object> event = mapper.readValue(payload, Map.class);
            System.out.println("NFT Transfer received");
            
            return "OK";
            
        } catch (Exception e) {
            return "Error: " + e.getMessage();
        }
    }
    
    private static String bytesToHex(byte[] bytes) {
        StringBuilder result = new StringBuilder();
        for (byte b : bytes) {
            result.append(String.format("%02x", b));
        }
        return result.toString();
    }
}
```

#### JavaScript (Node.js + Express)

```javascript
const express = require('express');
const crypto = require('crypto');

const app = express();
app.use(express.json());

app.post('/webhooks/nft', (req, res) => {
    const signature = req.headers['x-ethhook-signature'];
    const payload = JSON.stringify(req.body);
    
    // Verify signature
    const hmac = crypto.createHmac('sha256', 'your_webhook_secret');
    hmac.update(payload);
    const expected = 'sha256=' + hmac.digest('hex');
    
    if (!crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expected))) {
        return res.status(401).send('Invalid signature');
    }
    
    console.log(`NFT Transfer: ${req.body.data.transaction_hash}`);
    res.send('OK');
});

app.listen(3000);
```

#### TypeScript (Node.js + Express)

```typescript
import express, { Request, Response } from 'express';
import crypto from 'crypto';

const app = express();
app.use(express.json());

interface WebhookEvent {
    id: string;
    type: string;
    created_at: string;
    data: {
        block_number: number;
        transaction_hash: string;
        contract_address: string;
        topics: string[];
        data: string;
    };
}

app.post('/webhooks/nft', (req: Request, res: Response) => {
    const signature = req.headers['x-ethhook-signature'] as string;
    const payload = JSON.stringify(req.body);
    
    // Verify signature
    const hmac = crypto.createHmac('sha256', 'your_webhook_secret');
    hmac.update(payload);
    const expected = 'sha256=' + hmac.digest('hex');
    
    if (!crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expected))) {
        return res.status(401).send('Invalid signature');
    }
    
    const event = req.body as WebhookEvent;
    console.log(`NFT Transfer: ${event.data.transaction_hash}`);
    res.send('OK');
});

app.listen(3000);
```

#### Go

```go
package main

import (
    "crypto/hmac"
    "crypto/sha256"
    "encoding/hex"
    "encoding/json"
    "fmt"
    "io"
    "net/http"
)

const webhookSecret = "your_webhook_secret"

type WebhookEvent struct {
    ID        string `json:"id"`
    Type      string `json:"type"`
    CreatedAt string `json:"created_at"`
    Data      struct {
        BlockNumber     int      `json:"block_number"`
        TransactionHash string   `json:"transaction_hash"`
        ContractAddress string   `json:"contract_address"`
        Topics          []string `json:"topics"`
        Data            string   `json:"data"`
    } `json:"data"`
}

func handleWebhook(w http.ResponseWriter, r *http.Request) {
    signature := r.Header.Get("X-EthHook-Signature")
    
    // Read body
    body, err := io.ReadAll(r.Body)
    if err != nil {
        http.Error(w, "Error reading body", http.StatusBadRequest)
        return
    }
    
    // Verify signature
    mac := hmac.New(sha256.New, []byte(webhookSecret))
    mac.Write(body)
    expected := "sha256=" + hex.EncodeToString(mac.Sum(nil))
    
    if !hmac.Equal([]byte(signature), []byte(expected)) {
        http.Error(w, "Invalid signature", http.StatusUnauthorized)
        return
    }
    
    // Parse event
    var event WebhookEvent
    if err := json.Unmarshal(body, &event); err != nil {
        http.Error(w, "Error parsing JSON", http.StatusBadRequest)
        return
    }
    
    fmt.Printf("NFT Transfer: %s\n", event.Data.TransactionHash)
    w.Write([]byte("OK"))
}

func main() {
    http.HandleFunc("/webhooks/nft", handleWebhook)
    http.ListenAndServe(":3000", nil)
}
```

#### C# (.NET)

```csharp
using Microsoft.AspNetCore.Mvc;
using System.Security.Cryptography;
using System.Text;

[ApiController]
[Route("webhooks")]
public class WebhookController : ControllerBase
{
    private const string WebhookSecret = "your_webhook_secret";
    
    [HttpPost("nft")]
    public IActionResult HandleWebhook([FromBody] WebhookEvent webhookEvent)
    {
        // Get signature from header
        var signature = Request.Headers["X-EthHook-Signature"].ToString();
        
        // Read raw body
        Request.Body.Position = 0;
        using var reader = new StreamReader(Request.Body);
        var payload = reader.ReadToEnd();
        
        // Verify signature
        using var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(WebhookSecret));
        var hash = hmac.ComputeHash(Encoding.UTF8.GetBytes(payload));
        var expected = "sha256=" + BitConverter.ToString(hash)
            .Replace("-", "").ToLower();
        
        if (signature != expected)
        {
            return Unauthorized("Invalid signature");
        }
        
        Console.WriteLine($"NFT Transfer: {webhookEvent.Data.TransactionHash}");
        return Ok("OK");
    }
}

public class WebhookEvent
{
    public string Id { get; set; }
    public string Type { get; set; }
    public string CreatedAt { get; set; }
    public EventData Data { get; set; }
}

public class EventData
{
    public int BlockNumber { get; set; }
    public string TransactionHash { get; set; }
    public string ContractAddress { get; set; }
    public List<string> Topics { get; set; }
    public string Data { get; set; }
}
```

## � Documentation

## 📚 Documentation

- [SETUP_GUIDE.md](./SETUP_GUIDE.md) - Installation and configuration
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design and architecture
- [CONTRIBUTING.md](./CONTRIBUTING.md) - Contribution guidelines

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Write tests
5. Run `cargo fmt` and `cargo clippy`
6. Commit (`git commit -m 'Add amazing feature'`)
7. Push (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## 📜 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [ethers-rs](https://github.com/gakonst/ethers-rs) for Ethereum integration
- [Tokio](https://tokio.rs/) for async runtime
- [Axum](https://github.com/tokio-rs/axum) for HTTP server
- [Leptos](https://leptos.dev/) for frontend framework

## 📞 Support

- 🐛 [Issue Tracker](https://github.com/ipcasj/ethhook/issues)
- 📧 Email: [ihorpetroff@gmail.com](mailto:ihorpetroff@gmail.com)

---

Built with 🦀 Rust
