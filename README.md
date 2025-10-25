<!-- CI Trigger -->

## 🌐 Custom Domain Setup (DigitalOcean)

To use a real project domain (e.g., `ethhook.io`) on DigitalOcean App Platform:

1. **Buy a domain** from a registrar (Cloudflare, Namecheap, Google Domains)
2. **Configure DNS**:
    - Add CNAME records for `api.ethhook.io` (admin-api) and `app.ethhook.io` (frontend)
    - Point to DigitalOcean App Platform endpoints (see DO dashboard)
3. **Add custom domains** in DigitalOcean App Platform dashboard for each service
4. **Wait for DNS propagation and SSL certificate provisioning** (automatic)
5. **Update environment variables**:
    - Set `CORS_ALLOWED_ORIGINS=https://app.ethhook.io` for backend
    - Set `API_URL=https://api.ethhook.io` for frontend
6. **Update documentation** to reference your new domain URLs

See `docs/CUSTOM_DOMAIN_SETUP.md` for a step-by-step guide.
# EthHook 🦀

> Production-grade real-time Ethereum webhook service built in Rust

[![CI](https://github.com/ipcasj/ethhook/workflows/CI/badge.svg)](https://github.com/ipcasj/ethhook/actions)
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
git clone https://github.com/ipcasj/ethhook.git
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

## 🌐 MVP Demo & Monitoring

**Live MVP Demo:**
[https://ethhook-mvp.digitalocean.app](https://ethhook-mvp.digitalocean.app)

**Grafana Dashboard (Live Metrics):**
[https://ethhook-mvp.digitalocean.app/grafana](https://ethhook-mvp.digitalocean.app/grafana)

Access the dashboard locally at [http://localhost:3002](http://localhost:3002)

## 🔧 Configuration

### Environment-Based Network Selection

EthHook supports automatic network switching between **Sepolia testnet** (for development/demo) and **Ethereum mainnet** (for production):

```bash
# ENVIRONMENT controls which Ethereum network to use
# - development: Sepolia Testnet (free, safe for testing)
# - staging: Sepolia Testnet (pre-production)
# - production: Ethereum Mainnet (real money)
ENVIRONMENT=development

# For MVP demo (Oct 20, 2025): Use Sepolia testnet
# For production launch: Change to production
```

**Key Benefits:**

- ✅ **Safe testing**: Use free Sepolia testnet for development/demo
- ✅ **One-line switch**: Change `ENVIRONMENT=production` to go live
- ✅ **No code changes**: Same codebase works for both networks

See [docs/ENVIRONMENT_CONFIGURATION.md](docs/ENVIRONMENT_CONFIGURATION.md) for details.

### Complete .env File

Create a `.env` file:

```bash
# Environment (development=Sepolia, production=Mainnet)
ENVIRONMENT=development

# Ethereum RPC (Sepolia for testing)
ETHEREUM_WS_URL=wss://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
SEPOLIA_RPC_WS=wss://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
SEPOLIA_RPC_HTTP=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# For production, use mainnet endpoints
ETH_RPC_WS=wss://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY
ETH_RPC_HTTP=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY

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
# Run all unit tests
cargo test

# Run end-to-end integration tests (requires PostgreSQL + Redis)
./scripts/run_e2e_tests.sh

# Or manually with infrastructure running
cargo test --package ethhook-e2e-tests -- --ignored

# Run tests for specific service
cargo test --package ethhook-message-processor

# Run with coverage
cargo tarpaulin --out Html
```

See [E2E Test Documentation](tests/README.md) for details on integration testing.

## 📊 Monitoring

EthHook exposes Prometheus metrics on `/metrics`.

**Live Grafana Dashboard:**
[https://ethhook-mvp.digitalocean.app/grafana](https://ethhook-mvp.digitalocean.app/grafana)

**Local Grafana Dashboard:**
[http://localhost:3001/d/ethhook-overview/ethhook-system-overview](http://localhost:3001/d/ethhook-overview/ethhook-system-overview)

Import the dashboard from `monitoring/grafana/dashboards/ethhook-dashboard.json` for custom setups.

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
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;

@RestController
public class WebhookController {
    
    private static final String WEBHOOK_SECRET = "your_webhook_secret";
    
    @PostMapping("/webhooks/nft")
    public ResponseEntity<String> handleWebhook(
            @RequestHeader("X-EthHook-Signature") String signature,
            @RequestBody String payload) {
        
        try {
            // Verify signature
            Mac hmac = Mac.getInstance("HmacSHA256");
            SecretKeySpec secretKey = new SecretKeySpec(
                WEBHOOK_SECRET.getBytes(StandardCharsets.UTF_8), "HmacSHA256");
            hmac.init(secretKey);
            
            byte[] hash = hmac.doFinal(payload.getBytes(StandardCharsets.UTF_8));
            String expected = "sha256=" + bytesToHex(hash);
            
            if (!MessageDigest.isEqual(
                    signature.getBytes(StandardCharsets.UTF_8), 
                    expected.getBytes(StandardCharsets.UTF_8))) {
                return ResponseEntity.status(HttpStatus.UNAUTHORIZED)
                    .body("Invalid signature");
            }
            
            // Parse JSON and process event
            // ObjectMapper mapper = new ObjectMapper();
            // Map<String, Object> event = mapper.readValue(payload, Map.class);
            // String txHash = ((Map<String, Object>) event.get("data"))
            //     .get("transaction_hash").toString();
            System.out.println("NFT Transfer received");
            
            return ResponseEntity.ok("OK");
            
        } catch (Exception e) {
            return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR)
                .body("Error: " + e.getMessage());
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

// Use express.raw() to get the raw body for signature verification
app.use(express.json({
    verify: (req, res, buf) => {
        req.rawBody = buf.toString('utf8');
    }
}));

app.post('/webhooks/nft', (req, res) => {
    const signature = req.headers['x-ethhook-signature'];
    const payload = req.rawBody;
    
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

// Extend Request type to include rawBody
interface WebhookRequest extends Request {
    rawBody?: string;
}

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

// Use express.json() with verify to capture raw body
app.use(express.json({
    verify: (req: WebhookRequest, res, buf) => {
        req.rawBody = buf.toString('utf8');
    }
}));

app.post('/webhooks/nft', (req: WebhookRequest, res: Response) => {
    const signature = req.headers['x-ethhook-signature'] as string;
    const payload = req.rawBody!;
    
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
    "crypto/subtle"
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
    expectedHash := mac.Sum(nil)
    expected := "sha256=" + hex.EncodeToString(expectedHash)
    
    // Use constant-time comparison to prevent timing attacks
    if subtle.ConstantTimeCompare([]byte(signature), []byte(expected)) != 1 {
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
using System.Text.Json;

[ApiController]
[Route("webhooks")]
public class WebhookController : ControllerBase
{
    private const string WebhookSecret = "your_webhook_secret";
    
    [HttpPost("nft")]
    public async Task<IActionResult> HandleWebhook()
    {
        // Get signature from header
        var signature = Request.Headers["X-EthHook-Signature"].ToString();
        
        // Read raw body (must be done before model binding)
        Request.EnableBuffering();
        using var reader = new StreamReader(Request.Body, leaveOpen: true);
        var payload = await reader.ReadToEndAsync();
        Request.Body.Position = 0;
        
        // Verify signature
        using var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(WebhookSecret));
        var hash = hmac.ComputeHash(Encoding.UTF8.GetBytes(payload));
        var expected = "sha256=" + Convert.ToHexString(hash).ToLower();
        
        // Constant-time comparison
        if (!CryptographicOperations.FixedTimeEquals(
                Encoding.UTF8.GetBytes(signature),
                Encoding.UTF8.GetBytes(expected)))
        {
            return Unauthorized("Invalid signature");
        }
        
        // Parse JSON
        var webhookEvent = JsonSerializer.Deserialize<WebhookEvent>(payload);
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
5. **Run CI checks locally** (recommended before pushing):
   ```bash
   ./scripts/ci-check.sh
   ```
   This catches issues before they reach GitHub CI:
   - Format check (`cargo fmt`)
   - Clippy lints (all warnings as errors)
   - Unit tests
   - Build verification
   - SQLX offline mode check
   - Security audit
   
   See [CI_CD_PRECHECK_GUIDE.md](docs/CI_CD_PRECHECK_GUIDE.md) for details.

6. Commit (`git commit -m 'Add amazing feature'`)
7. Push (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Quick Fixes

If CI checks fail:

```bash
# Fix formatting
cargo fmt --all

# Auto-fix clippy warnings
cargo clippy --fix --allow-dirty --allow-staged

# Run tests
cargo test --workspace

# Regenerate SQLX cache (if queries changed)
cargo sqlx prepare --workspace
```

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


---

**Built with 🦀 Rust**
