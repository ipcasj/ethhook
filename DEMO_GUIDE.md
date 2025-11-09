# 🎯 EthHook Demo Guide - Quick Start & Tutorial

**Last Updated:** November 6, 2025  
**Demo URL:** http://104.248.15.178:3002

---

## 🚀 Two Ways to Experience EthHook

### Option 1: **Instant Demo** (Recommended First!)
Just login and see everything already configured with live blockchain data

### Option 2: **Hands-On Tutorial**  
Follow our step-by-step guide to create your own application and endpoints

---

## 📋 Login Credentials

```
URL:      http://104.248.15.178:3002
Email:    demo@ethhook.com
Password: Demo1234!
```

---

## 🎬 OPTION 1: Instant Demo (Just Login & Explore!)

### What's Already Set Up For You

When you login, you'll immediately see:

#### 📊 **Dashboard** 
- Real-time metrics from Ethereum mainnet
- Live event counts updating every few seconds
- 99%+ webhook delivery success rate
- Average delivery time: **8ms** (vs competitors' 150-300ms)

#### 📱 **3 Pre-Configured Applications**

1. **DeFi Protocol Monitor** 
   - USDT Transfers (Tether)
   - USDC Transfers (USD Coin)
   - Uniswap V3 ETH/USDC Swaps
   - WETH Deposits & Withdrawals
   - Aave V3 Supply & Borrow

2. **NFT Marketplace Tracker**
   - OpenSea Order Fulfilled (NFT sales)
   - Blur Order Executed
   - BAYC (Bored Ape) Transfers
   - Azuki NFT Transfers

3. **Multi-Chain Bridge Monitor**
   - Arbitrum Bridge Deposits
   - Optimism Bridge Deposits
   - Base Bridge Deposits

#### 📡 **12 Active Endpoints**
All connected to our demo webhook receiver at port 8000

### 🔍 What To Explore

#### 1. **Dashboard Overview** (Home Page)
```
✓ See total events captured (growing in real-time)
✓ Check active endpoints (12)
✓ View webhook delivery success rate (99%+)
✓ Monitor average response time (8ms)
✓ Recent events section showing latest blockchain activity
```

#### 2. **Applications Page** (Left Sidebar → Applications)
```
✓ See all 3 pre-configured applications
✓ Click each to view associated endpoints
✓ Check API keys (safely displayed, no sensitive production data)
✓ View application descriptions and metrics
```

#### 3. **Endpoints Page** (Left Sidebar → Endpoints)
```
✓ Browse all 12 active endpoints
✓ See which contract addresses being monitored
✓ Check event filters and signatures
✓ View delivery statistics per endpoint
✓ Click "View Details" to see event history
```

#### 4. **Event History** (Any endpoint → Events tab)
```
✓ See real blockchain events captured in real-time
✓ Transaction hashes, block numbers, contract addresses
✓ Event types (Transfer, Swap, Deposit, etc.)
✓ Delivery attempts and success status
✓ Detailed event data and topics
```

### 💡 What Makes This Impressive

**Live Production Data:**
- These are REAL events from Ethereum mainnet
- USDT transfers happening right now
- Uniswap swaps occurring in real-time
- NFT sales as they happen on OpenSea

**Performance:**
- Events captured within 1-2 seconds of blockchain confirmation
- Webhooks delivered in ~8ms average
- 99%+ success rate even with high volume
- Automatic retries on failures

**Scale Demonstrated:**
- Monitoring multiple high-traffic contracts simultaneously
- USDT alone: 100,000+ transfers per day
- Uniswap: Thousands of swaps per hour
- System handles it all effortlessly

---

## 🛠️ OPTION 2: Hands-On Tutorial (Create Your Own!)

Ready to try creating your own application and endpoint? Let's do it!

### Step 1: Navigate to Applications

```
1. Login at http://104.248.15.178:3002
2. Click "Applications" in left sidebar
3. Click "+ Create Application" button (top right)
```

### Step 2: Create Your First Application

Fill in the form:

```
Application Name: 
  "My Test DeFi App"
  
Description:
  "Testing EthHook with custom DeFi monitoring"
  
Click "Create Application"
```

**Result:** You'll get:
- A unique Application ID
- An API key (keep this for your webhook implementation)
- A new entry in your applications list

### Step 3: Create Your First Endpoint

From your new application:

```
1. Click on "My Test DeFi App" 
2. Click "+ Create Endpoint" button
3. Fill in the endpoint details (see below)
```

### Step 4: Configure Your Endpoint

Let's monitor DAI token transfers (another popular stablecoin):

```
Endpoint Name:
  "DAI Token Transfers"

Description:
  "Monitor DAI stablecoin transfer events"

Webhook URL:
  http://104.248.15.178:8000/webhook
  (Use our demo receiver to see webhooks in action!)

Chain Selection:
  ☑ Ethereum Mainnet (Chain ID: 1)

Contract Address:
  0x6b175474e89094c44da98b954eedeac495271d0f
  (This is the DAI token contract)

Event Signature (Transfer event):
  0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
  (Standard ERC20 Transfer event)

Click "Create Endpoint"
```

### Step 5: See Your Endpoint in Action!

```
1. Go back to "Endpoints" page
2. Find "DAI Token Transfers"
3. Click to view details
4. Watch as events start appearing! (DAI is very active)
```

Within **1-2 minutes**, you should see:
- Real DAI transfer events captured
- Webhooks being delivered to demo receiver
- Success/failure status for each delivery
- Full blockchain data (tx hash, block, addresses)

### Step 6: View Webhook Deliveries

Our demo receiver logs all webhooks:

```
Check the webhook receiver logs:
http://104.248.15.178:8000
(If you have access to server logs, you'll see POST requests)

Each webhook contains:
- Event ID
- Block number and hash
- Transaction hash
- Contract address
- Event topics and data
- Timestamp
```

---

## 📚 Understanding What You Just Did

### The Flow

```
1. Ethereum blockchain
      ↓ (New DAI transfer transaction)
2. EthHook Event Ingestor (captures it in real-time)
      ↓ (Filters by your contract address & event signature)
3. Your Endpoint matches!
      ↓ (Prepares webhook payload)
4. Message Processor (ensures delivery)
      ↓ (HTTP POST request)
5. Your Webhook URL (receives the data)
      ↓
6. Your application processes the event!
```

### Key Concepts

**Application:**
- Top-level organization container
- Can have multiple endpoints
- Has one API key
- Use for: grouping related endpoints (e.g., "My DeFi Dashboard")

**Endpoint:**
- Specific webhook configuration
- Belongs to one application
- Filters events by: chain, contract, event type
- Each has unique HMAC secret for security

**Event Filtering:**
- **Chain IDs**: Which blockchain(s) to monitor
- **Contract Addresses**: Which smart contracts to watch
- **Event Signatures**: Which events to capture (Transfer, Swap, etc.)

**Webhook Delivery:**
- HTTP POST to your URL
- Includes full event data
- HMAC signature for verification
- Automatic retries on failure (exponential backoff)

---

## 🎓 Next Steps & Advanced Usage

### Try These Popular Contracts

#### **DeFi Protocols**

```
Uniswap V2 Factory (Pair Created):
  Contract: 0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f
  Event: 0x0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9

Compound cDAI (Mint/Redeem):
  Contract: 0x5d3a536e4d6dbd6114cc1ead35777bab948e3643
  Events: Mint, Redeem, Borrow

MakerDAO Vault (Open/Close):
  Contract: 0x35d1b3f3d7966a1dfe207aa4514c12a259a0492b
  Event: NewCdp (vault opened)
```

#### **NFT Collections**

```
CryptoPunks:
  Contract: 0xb47e3cd837ddf8e4c57f05d70ab865de6e193bbb
  Events: PunkTransfer, PunkBought

Azuki:
  Contract: 0xed5af388653567af2f388e6224dc7c4b3241c544
  Event: Transfer

Pudgy Penguins:
  Contract: 0xbd3531da5cf5857e7cfaa92426877b022e612cf8
  Event: Transfer
```

#### **Bridge & L2s**

```
Polygon Bridge:
  Contract: 0x40ec5b33f54e0e8a33a975908c5ba1c14e5bbbdf
  Events: LockedEther, NewDepositBlock

zkSync Era Bridge:
  Contract: 0x32400084c286cf3e17e7b677ea9583e60a000324
  Events: DepositInitiated, WithdrawalFinalized
```

### Testing Webhook Security

Each webhook includes an HMAC signature in headers:

```javascript
// Verify webhook authenticity (Node.js example)
const crypto = require('crypto');

function verifyWebhook(payload, signature, secret) {
  const hmac = crypto
    .createHmac('sha256', secret)
    .update(JSON.stringify(payload))
    .digest('hex');
    
  return hmac === signature;
}

// In your webhook handler:
const signature = req.headers['x-webhook-signature'];
const isValid = verifyWebhook(req.body, signature, endpoint.hmac_secret);
```

### Using Multiple Filters

Monitor specific contract interactions:

```
Example: Track USDC transfers TO Uniswap

Contract: USDC (0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48)
Event: Transfer (0xddf252...)
Additional Filter: topics[2] = Uniswap Router address

Result: Only USDC being sent to Uniswap for trading
```

---

## 🔍 Common Use Cases

### 1. **DeFi Protocol Monitoring**
```
Track deposits, withdrawals, swaps across DeFi platforms
Alert on large transactions (whale watching)
Monitor your protocol's contracts
Track competitive activity
```

### 2. **NFT Analytics**
```
Monitor NFT sales across marketplaces
Track specific collection transfers
Alert on blue-chip NFT movements
Build real-time floor price trackers
```

### 3. **Wallet Tracking**
```
Monitor specific wallet addresses
Track token transfers in/out
Alert on transactions above threshold
Compliance and fraud detection
```

### 4. **Cross-Chain Monitoring**
```
Track bridge deposits/withdrawals
Monitor multi-chain protocol activity
Alert on cross-chain arbitrage opportunities
Liquidity movement tracking
```

### 5. **Smart Contract Events**
```
Your own contract event monitoring
Integration testing in development
Production event logging
User action analytics
```

---

## 💰 Why This Matters (Sales Points)

### Performance
- **8ms** average delivery time
- **150-300ms faster** than Alchemy webhooks
- **99%+ success rate** on deliveries
- **Real-time** (1-2 seconds from blockchain)

### Reliability
- Automatic retry logic
- Exponential backoff on failures
- Health monitoring and alerts
- No events missed

### Cost
- **$29-499/month** (vs Alchemy $199-999)
- Unlimited endpoints per tier
- No hidden fees
- Pay for what you need

### Simplicity
- **2-minute setup** (as you just experienced!)
- Clean, modern UI
- Comprehensive API
- Excellent documentation

### Features
- Multi-chain support (Ethereum, Arbitrum, Optimism, Base)
- Flexible filtering (chain, contract, event type)
- HMAC webhook security
- Detailed analytics and metrics
- Event history and replay

---

## 🛡️ Security & Best Practices

### For Demo Account

⚠️ **Remember:**
- This is a **public demo account**
- Don't use production webhook URLs
- Don't store sensitive data
- Account may be reset periodically

✅ **Safe to:**
- Create test endpoints
- Use demo webhook receiver
- Explore all features
- Share with potential clients

### For Production Use

When you create your own account:

```
✓ Use HTTPS webhook URLs only
✓ Verify HMAC signatures on all webhooks
✓ Implement idempotency (events may be delivered more than once)
✓ Store endpoint secrets securely
✓ Monitor webhook endpoint health
✓ Implement rate limiting on your side
✓ Log all webhook deliveries for debugging
```

---

## 📞 Getting Help

### Questions While Exploring?

Common scenarios covered:

**"I don't see any events yet"**
- New endpoints take 1-2 minutes to start capturing
- Ensure contract address is correct (0x prefix, lowercase)
- Check event signature matches
- Verify chain ID is correct (1 = Ethereum mainnet)

**"Events captured but webhooks failing"**
- Check webhook URL is accessible from internet
- Our demo receiver (port 8000) works for testing
- Check for HTTPS requirements
- Verify no firewall blocking

**"How do I find event signatures?"**
- Use Etherscan.io → Contract → Events tab
- Look at existing transactions with that event
- Event signature is the first topic (topics[0])
- Or use: keccak256("Transfer(address,address,uint256)")

**"Can I filter by specific addresses?"**
- Yes! In event topics, topics[1] and topics[2] often contain addresses
- Example: Transfer event topics[1]=from, topics[2]=to
- Contact us for advanced filtering setup

### Ready for Production?

When you're ready to move beyond demo:

1. **Create Your Own Account**
   - Sign up for free trial or paid tier
   - Get dedicated resources
   - Production-grade SLAs

2. **Migrate Your Endpoints**
   - Export configurations from demo
   - Re-create in your account
   - Update webhook URLs to your servers

3. **Implement Webhook Handler**
   - Use HMAC verification
   - Handle idempotency
   - Implement retry logic on your side
   - Log for debugging

4. **Monitor & Scale**
   - Watch delivery success rates
   - Check webhook response times
   - Scale webhook handlers as needed
   - Contact support for high-volume needs

---

## 🎉 Summary

You now have everything you need to evaluate EthHook:

✅ **Pre-configured demo** with 12 active endpoints  
✅ **Live blockchain data** from Ethereum mainnet  
✅ **Real-time webhooks** delivered in milliseconds  
✅ **Hands-on tutorial** to create your own endpoints  
✅ **Production-ready** infrastructure you can trust  

### What You've Learned

1. How to navigate the dashboard and see real-time metrics
2. How applications organize multiple endpoints
3. How to create and configure endpoints
4. How to filter events by contract and event type
5. How webhooks are delivered and verified
6. Common use cases across DeFi, NFT, and bridges

### Next Actions

- 🔍 **Explore:** Login and browse the pre-configured applications
- 🛠️ **Build:** Follow the tutorial and create your own endpoint
- 📊 **Monitor:** Watch real events flow through the system
- 💬 **Discuss:** Contact sales to discuss your specific needs

---

**Demo Access:**
- URL: http://104.248.15.178:3002
- Email: demo@ethhook.com
- Password: Demo1234!

**Questions?**
- Documentation: See /docs folder
- Competitive Analysis: docs/COMPETITIVE_ANALYSIS.md
- Architecture: ARCHITECTURE.md

---

*This demo environment runs on production infrastructure with real blockchain data. Experience the same performance and reliability your production application would receive.*
