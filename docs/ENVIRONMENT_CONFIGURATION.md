# Environment-Based Configuration System

**Date:** October 13, 2025  
**Phase:** Phase 7 - Production Configuration  
**Status:** ✅ Implemented

---

## Overview

The ETHHook platform now supports **environment-based configuration** that allows seamless switching between test networks (Sepolia) and production networks (Ethereum mainnet) using a single `ENVIRONMENT` variable.

This system enables:
- ✅ **Safe MVP testing** on Sepolia testnet (free, no financial risk)
- ✅ **One-line production deployment** (just change ENVIRONMENT=production)
- ✅ **Clear separation** between development, staging, and production
- ✅ **No code changes** needed to switch networks

---

## Configuration Variables

### ENVIRONMENT Variable

The `ENVIRONMENT` variable controls which Ethereum network the event-ingestor uses:

| Environment | Ethereum Network | Chain ID | Use Case |
|------------|------------------|----------|----------|
| `development` | Sepolia Testnet | 11155111 | Local development & MVP demo |
| `staging` | Sepolia Testnet | 11155111 | Pre-production testing |
| `production` | Ethereum Mainnet | 1 | Production deployment (real money) |

**Default:** If `ENVIRONMENT` is not set, defaults to `development` (Sepolia).

---

## How It Works

### 1. Configuration Loading

The event-ingestor reads `ENVIRONMENT` from `.env` and automatically selects the correct chain:

```rust
// From crates/event-ingestor/src/config.rs
let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

let (eth_chain_id, eth_chain_name) = match environment.as_str() {
    "production" => (1_u64, "Ethereum Mainnet"),
    _ => (11155111_u64, "Sepolia Testnet"), // default to testnet
};
```

### 2. Chain Configuration

The selected chain is then used to configure the WebSocket connection:

```rust
ChainConfig {
    name: eth_chain_name.to_string(),    // "Sepolia Testnet" or "Ethereum Mainnet"
    chain_id: eth_chain_id,              // 11155111 or 1
    ws_url: env::var("ETHEREUM_WS_URL")?, // From .env
    max_reconnect_attempts: 10,
    reconnect_delay_secs: 1,
}
```

---

## Usage Examples

### MVP Demo (October 20, 2025)

For the public demo, use **Sepolia testnet** (safe, free, predictable):

```bash
# .env configuration
ENVIRONMENT=development

# Sepolia RPC endpoints (already configured)
ETHEREUM_WS_URL=wss://eth-sepolia.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW

# Start services
docker-compose up -d
cargo run --bin event-ingestor
```

**Result:** Event-ingestor connects to Sepolia testnet (chain_id: 11155111)

### Production Deployment (December 1, 2025+)

For production with real users and real money, switch to **Ethereum mainnet**:

```bash
# .env configuration
ENVIRONMENT=production

# Ethereum Mainnet RPC endpoints
ETHEREUM_WS_URL=wss://eth-mainnet.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW

# Start services
docker-compose up -d
cargo run --bin event-ingestor --release
```

**Result:** Event-ingestor connects to Ethereum mainnet (chain_id: 1)

---

## Migration Path

### Phase 1: MVP Demo (October 13-20, 2025)
- **Network:** Sepolia Testnet
- **Config:** `ENVIRONMENT=development`
- **Goal:** Public demo with test contracts
- **Cost:** $0 (free test ETH from faucets)

### Phase 2: Beta Testing (October 21 - November 30, 2025)
- **Network:** Sepolia Testnet OR Ethereum Mainnet (your choice)
- **Config:** `ENVIRONMENT=staging` (testnet) or `ENVIRONMENT=production` (mainnet)
- **Goal:** Beta users test with real or test contracts
- **Cost:** $0 (testnet) or minimal (mainnet with limited users)

### Phase 3: Production Launch (December 1, 2025+)
- **Network:** Ethereum Mainnet
- **Config:** `ENVIRONMENT=production`
- **Goal:** Full production with real users and real contracts
- **Cost:** Alchemy free tier covers 300M compute units/month

---

## Test Networks vs Production Networks

### Sepolia Testnet (Development/Staging)

**Advantages:**
- ✅ Free test ETH from faucets (no financial risk)
- ✅ Identical API to mainnet (same JSON-RPC, same event structures)
- ✅ Perfect for testing webhooks without spending real money
- ✅ Fast and predictable (great for demos)

**Use Cases:**
- MVP demo (October 20, 2025)
- Developer testing
- CI/CD pipeline testing
- User acceptance testing (UAT)

**Test Contracts on Sepolia:**
- WETH: `0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9`
- USDC: `0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238`

### Ethereum Mainnet (Production)

**Advantages:**
- ✅ Real smart contracts with real value
- ✅ Real users, real transactions
- ✅ Same RPC providers (Alchemy, Infura)

**Requirements:**
- Real ETH needed for gas fees (webhook delivery uses minimal gas)
- Production-grade monitoring (Prometheus, Grafana)
- Backup RPC endpoints configured
- Rate limiting and error handling

---

## RPC Provider Configuration

### Alchemy Free Tier
- **Compute Units:** 300M/month
- **Suitable for:** Up to ~100,000 webhook deliveries/month
- **Networks:** Ethereum, Sepolia, Arbitrum, Optimism, Base

### Current RPC Endpoints

**Sepolia Testnet (Development/Staging):**
```bash
ETHEREUM_WS_URL=wss://eth-sepolia.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW
SEPOLIA_RPC_WS=wss://eth-sepolia.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW
SEPOLIA_RPC_HTTP=https://eth-sepolia.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW
```

**Ethereum Mainnet (Production):**
```bash
ETHEREUM_WS_URL=wss://eth-mainnet.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW
ETH_RPC_WS=wss://eth-mainnet.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW
ETH_RPC_HTTP=https://eth-mainnet.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW
```

---

## Testing the Configuration

### Verify Current Environment

```bash
# Check current ENVIRONMENT setting
grep ENVIRONMENT .env

# Expected output:
# ENVIRONMENT=development
```

### Test Development Mode (Sepolia)

```bash
# Ensure .env has development mode
echo "ENVIRONMENT=development" > .env.test

# Run event-ingestor
cargo run --bin event-ingestor

# Expected log output:
# INFO event_ingestor: Loaded configuration for 4 chains
# INFO event_ingestor: Chain: Sepolia Testnet (chain_id: 11155111)
```

### Test Production Mode (Mainnet)

```bash
# Switch to production mode
sed -i '' 's/ENVIRONMENT=development/ENVIRONMENT=production/' .env

# Run event-ingestor
cargo run --bin event-ingestor

# Expected log output:
# INFO event_ingestor: Chain: Ethereum Mainnet (chain_id: 1)
```

---

## Code Changes Summary

### Modified Files

1. **`crates/event-ingestor/src/config.rs`**
   - Added `ENVIRONMENT` variable detection
   - Implemented environment-based chain selection
   - Updated documentation comments

2. **`.env`**
   - Added `ENVIRONMENT=development` variable
   - Added comprehensive comments explaining usage

### Implementation Details

**Before (Hard-coded Sepolia):**
```rust
let chains = vec![
    ChainConfig {
        name: "Sepolia Testnet".to_string(),
        chain_id: 11155111,
        ws_url: env::var("ETHEREUM_WS_URL")?,
        ...
    },
];
```

**After (Environment-based):**
```rust
let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

let (eth_chain_id, eth_chain_name) = match environment.as_str() {
    "production" => (1_u64, "Ethereum Mainnet"),
    _ => (11155111_u64, "Sepolia Testnet"),
};

let chains = vec![
    ChainConfig {
        name: eth_chain_name.to_string(),
        chain_id: eth_chain_id,
        ws_url: env::var("ETHEREUM_WS_URL")?,
        ...
    },
];
```

---

## Benefits

### For MVP Demo (October 20, 2025)
- ✅ **No risk:** Sepolia testnet is free and safe
- ✅ **Fast setup:** Already configured with test RPC endpoints
- ✅ **Predictable:** Known test contracts and predictable behavior
- ✅ **Professional:** Shows production-ready environment switching

### For Production Deployment
- ✅ **One-line switch:** Change `ENVIRONMENT=production` in `.env`
- ✅ **No code changes:** Same codebase works for testnet and mainnet
- ✅ **Clear separation:** Development, staging, production environments
- ✅ **Easy rollback:** Switch back to testnet if needed

### For Development Team
- ✅ **Local testing:** Developers use Sepolia (no mainnet required)
- ✅ **CI/CD friendly:** Test pipeline uses Sepolia
- ✅ **Cost-effective:** No mainnet gas fees during development
- ✅ **Safe experimentation:** Test new features without financial risk

---

## Security Considerations

### Testnet (Development/Staging)
- ⚠️ **Public test network:** Anyone can view transactions
- ⚠️ **Test ETH only:** No real financial value
- ✅ **Safe for demos:** Perfect for public demos and testing

### Mainnet (Production)
- ⚠️ **Real money:** Requires proper security measures
- ⚠️ **Rate limiting:** Configure rate limits on webhook delivery
- ⚠️ **Monitoring:** Set up alerts for unusual activity
- ⚠️ **Backup RPC:** Configure failover to Infura if Alchemy fails

---

## Next Steps

### Immediate (Phase 7 Completion)
- [ ] Add structured logging (JSON format for production)
- [ ] Implement health check endpoints
- [ ] Add graceful shutdown handling
- [ ] Enable Prometheus metrics

### Before Production Launch
- [ ] Load test with mainnet RPC endpoints
- [ ] Configure backup RPC providers (Infura)
- [ ] Set up monitoring and alerting (Grafana)
- [ ] Document incident response procedures

---

## Troubleshooting

### Issue: Event-ingestor connects to wrong network

**Symptoms:**
```
ERROR: Connected to Ethereum Mainnet but expected Sepolia
```

**Solution:**
```bash
# Check ENVIRONMENT variable
grep ENVIRONMENT .env

# Should show: ENVIRONMENT=development (for Sepolia)
# Or: ENVIRONMENT=production (for Mainnet)

# Fix if needed
sed -i '' 's/ENVIRONMENT=production/ENVIRONMENT=development/' .env
```

### Issue: RPC connection fails

**Symptoms:**
```
ERROR: Failed to connect to wss://eth-sepolia.g.alchemy.com/v2/...
```

**Solution:**
```bash
# Verify RPC URL is correct
grep ETHEREUM_WS_URL .env

# Test connection manually
websocat wss://eth-sepolia.g.alchemy.com/v2/xGCBQXSFxK8qbIwCcSaJW

# Check Alchemy dashboard for rate limits
# https://dashboard.alchemy.com/
```

---

## References

- **Sepolia Testnet:** https://sepolia.etherscan.io/
- **Ethereum Mainnet:** https://etherscan.io/
- **Alchemy Dashboard:** https://dashboard.alchemy.com/
- **Sepolia Faucets:** https://sepoliafaucet.com/

---

**Status:** ✅ Environment-based configuration implemented and tested  
**Compilation:** ✅ Event-ingestor compiles successfully  
**Next Phase:** Phase 7 remaining tasks (logging, health checks, metrics)
