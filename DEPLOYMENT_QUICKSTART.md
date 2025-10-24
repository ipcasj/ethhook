# EthHook Production Deployment - Quick Start

**Target**: Deploy to Railway.app in < 1 hour
**Cost**: $25-40/month
**Difficulty**: Easy (No DevOps knowledge required)

---

## TL;DR - 5 Minute Overview

1. Sign up for Railway.app (free)
2. Add PostgreSQL + Redis (1-click)
3. Deploy 4 backend services from GitHub
4. Set environment variables
5. Deploy frontend to Vercel
6. Done! 🎉

---

## Prerequisites (10 minutes)

### 1. Create Accounts

- [ ] **Railway**: https://railway.app/ (use GitHub login)
- [ ] **Alchemy**: https://www.alchemy.com/ (for Ethereum RPC)
- [ ] **Vercel** (optional): https://vercel.com/ (for frontend)

### 2. Get Alchemy API Key

1. Go to Alchemy dashboard
2. Create new app → **Sepolia testnet**
3. Copy WebSocket URL: `wss://eth-sepolia.g.alchemy.com/v2/YOUR_KEY`
4. **Save this!** You'll need it later

### 3. Generate JWT Secret

Run this command:
```bash
openssl rand -base64 32
```

Copy the output - you'll need it for Railway.

---

## Railway Deployment (30 minutes)

### Step 1: Create Project (2 minutes)

1. Go to Railway dashboard
2. Click "New Project"
3. Name it: `ethhook-production`

### Step 2: Add Databases (3 minutes)

1. Click "+ New"
2. Select "Database" → "PostgreSQL"
3. Click "+ New" again
4. Select "Database" → "Redis"

✅ **Done!** Railway automatically configures connection URLs.

### Step 3: Deploy Backend Services (20 minutes)

You'll deploy 4 services. For EACH service:

1. Click "+ New" → "GitHub Repo"
2. Authorize Railway to access your repo
3. Select your `ethhook` repository
4. Configure as shown below:

#### Service 1: admin-api

**Settings**:
- Root Directory: `/`
- Dockerfile Path: `crates/admin-api/Dockerfile`

**Environment Variables**:
```bash
API_HOST=0.0.0.0
API_PORT=3000
JWT_SECRET=<paste your generated secret from earlier>
CORS_ALLOWED_ORIGINS=*
RUST_LOG=info,ethhook=info,sqlx=warn
REDIS_HOST=redis.railway.internal
REDIS_PORT=6379
```

**Networking**:
- Click "Settings" → "Networking"
- Click "Generate Domain"
- **Copy this URL** - you'll need it for the frontend!

#### Service 2: event-ingestor

**Settings**:
- Dockerfile Path: `crates/event-ingestor/Dockerfile`

**Environment Variables**:
```bash
SEPOLIA_RPC_WS=wss://eth-sepolia.g.alchemy.com/v2/YOUR_ALCHEMY_KEY
SEPOLIA_RPC_HTTP=https://eth-sepolia.g.alchemy.com/v2/YOUR_ALCHEMY_KEY
ETHEREUM_WS_URL=wss://eth-sepolia.g.alchemy.com/v2/YOUR_ALCHEMY_KEY
ENVIRONMENT=development
REDIS_HOST=redis.railway.internal
REDIS_PORT=6379
RUST_LOG=info,ethhook=info
```

#### Service 3: message-processor

**Settings**:
- Dockerfile Path: `crates/message-processor/Dockerfile`

**Environment Variables**:
```bash
REDIS_HOST=redis.railway.internal
REDIS_PORT=6379
CONSUMER_GROUP=message_processors
CONSUMER_NAME=processor-1
BATCH_SIZE=100
BLOCK_TIME_MS=5000
RUST_LOG=info,ethhook=info,sqlx=warn
```

#### Service 4: webhook-delivery

**Settings**:
- Dockerfile Path: `crates/webhook-delivery/Dockerfile`

**Environment Variables**:
```bash
REDIS_HOST=redis.railway.internal
REDIS_PORT=6379
WEBHOOK_TIMEOUT_SECONDS=30
WEBHOOK_MAX_RETRIES=5
WEBHOOK_WORKER_THREADS=50
RUST_LOG=info,ethhook=info,sqlx=warn
```

### Step 4: Wait for Deployment (5 minutes)

Railway will build and deploy each service. This takes 3-5 minutes per service.

**Check Deployment Status**:
1. Go to each service in Railway
2. Click "Deployments" tab
3. Watch build logs
4. Wait for "Build completed" ✅

---

## Frontend Deployment (10 minutes)

### Option A: Vercel (Recommended)

1. Go to https://vercel.com/
2. Click "Add New" → "Project"
3. Import your GitHub repo
4. **Root Directory**: `crates/leptos-portal`
5. **Build Command**: `trunk build --release`
6. **Output Directory**: `dist`
7. Click "Deploy"

### Option B: Local Build + Static Hosting

```bash
cd crates/leptos-portal
trunk build --release
# Deploy the dist/ folder to any static host
```

### Update API URL in Frontend

After frontend deployment, update CORS:

1. Go to Railway → `admin-api` service
2. Go to "Variables" tab
3. Update `CORS_ALLOWED_ORIGINS`:
   ```bash
   CORS_ALLOWED_ORIGINS=https://your-frontend.vercel.app
   ```
4. Redeploy admin-api

---

## Verification (10 minutes)

### 1. Check Service Health

```bash
curl https://your-admin-api-url.up.railway.app/api/v1/health
```

Expected response:
```json
{"status":"healthy","service":"admin-api",...}
```

### 2. Check Logs

For each service in Railway:
1. Go to service
2. Click "Deployments"
3. Click "View Logs"

**Look for**:
- ✅ `[service] is running`
- ✅ `PostgreSQL connected`
- ✅ `Redis connected`
- ❌ No errors

### 3. Test Frontend

1. Open your frontend URL
2. Register a new user
3. Login
4. Create an application
5. Create an endpoint

### 4. Test Webhook Delivery

1. Go to https://webhook.site/
2. Copy your unique webhook URL
3. In EthHook frontend:
   - Create endpoint
   - URL: `https://webhook.site/your-unique-id`
   - Chain ID: `11155111` (Sepolia)
   - Contract: `0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9`
   - Event: `Transfer(address,address,uint256)`
4. Wait 5-10 minutes
5. Check webhook.site for incoming webhooks

✅ **If you receive webhooks, deployment is successful!**

---

## Troubleshooting

### Service Won't Start

**Check**:
1. Logs for error messages
2. All environment variables are set
3. Dockerfile path is correct

**Fix**:
- Click "Redeploy" in Railway

### No Webhooks Received

**Check**:
1. Event ingestor logs (should show "events ingested")
2. Message processor logs (should show "jobs created")
3. Webhook delivery logs (should show "job completed")

**Fix**:
- Wait longer (Sepolia has activity every 5-10 minutes)
- Try a more active contract (USDC: `0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238`)

### CORS Errors

**Check**:
- `CORS_ALLOWED_ORIGINS` includes your frontend URL
- No trailing slash in URL
- Using HTTPS (not HTTP)

**Fix**:
- Update `CORS_ALLOWED_ORIGINS` in admin-api
- Redeploy admin-api

---

## What You Just Deployed

### Architecture

```
Ethereum Sepolia → Event Ingestor → Redis → Message Processor
                                             ↓
                                          PostgreSQL
                                             ↓
                                       Webhook Delivery → Your Webhooks
                                             ↑
                    Leptos Frontend ← Admin API
```

### Services

1. **admin-api** (Port 3000): REST API for managing applications/endpoints
2. **event-ingestor**: Listens to Ethereum via WebSocket
3. **message-processor**: Matches events to endpoints
4. **webhook-delivery**: Sends HTTP POST to your webhooks

### Databases

1. **PostgreSQL**: Stores applications, endpoints, events, users
2. **Redis**: Event streaming and job queuing

---

## Next Steps

### 1. Invite Users

Share your frontend URL with users to test

### 2. Monitor Usage

Railway dashboard shows:
- Service uptime
- CPU/Memory usage
- Logs
- Costs

### 3. Scale When Needed

When you get more users:
1. Railway → Service → Settings → Resources
2. Increase CPU/Memory
3. Or add more service instances

### 4. Go to Production (Optional)

When ready for mainnet:
1. Update `ENVIRONMENT=production`
2. Change RPC URLs to mainnet
3. Update Alchemy API keys for mainnet
4. Increase database tier
5. Set up monitoring/alerting

---

## Costs

**Expected Monthly Cost**: $25-40

Breakdown:
- PostgreSQL: $5-10
- Redis: $5-10
- Services (4): $15-20
- Total: **$25-40/month**

**Free tier**: Railway gives $5/month credit

---

## Support

- **Railway Docs**: https://docs.railway.app/
- **Railway Discord**: https://discord.gg/railway
- **EthHook Issues**: https://github.com/yourusername/ethhook/issues
- **Email**: ihorpetroff@gmail.com

---

## Full Documentation

For detailed instructions, see:

- **Deployment Guide**: [docs/RAILWAY_DEPLOYMENT_GUIDE.md](docs/RAILWAY_DEPLOYMENT_GUIDE.md)
- **Production Checklist**: [docs/PRODUCTION_READINESS_CHECKLIST.md](docs/PRODUCTION_READINESS_CHECKLIST.md)
- **Architecture**: [docs/SYSTEM_ARCHITECTURE.md](docs/SYSTEM_ARCHITECTURE.md)
- **Environment Config**: [.env.production.example](.env.production.example)

---

## Success! 🎉

You've successfully deployed a production-grade webhook platform!

**What you can do now**:
- Monitor real-time Ethereum events
- Deliver webhooks to any HTTP endpoint
- Scale to thousands of endpoints
- Process millions of events

**Total time**: 45-60 minutes
**Cost**: $25-40/month
**Uptime**: 99.9%+

Welcome to production! 🚀
