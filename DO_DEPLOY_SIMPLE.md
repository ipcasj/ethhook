# EthHook DigitalOcean Deployment - All-in-One Droplet

**Ultra-simple deployment guide** for running everything on a single Droplet.

**Cost:** $24/month (just the Droplet!)
**Time:** 30 minutes
**What runs:** PostgreSQL + Redis + All EthHook services (all in Docker containers)

---

## What You Need

- DigitalOcean account
- Alchemy account (for Ethereum RPC - free tier)
- SSH key on your machine

---

## Step 1: Create Droplet (5 min)

1. **Go to https://digitalocean.com** → Sign up / Log in

2. **Create Droplet:**
   - Click "Create" → "Droplets"
   - **Image:** Ubuntu 22.04 LTS
   - **Plan:** Basic
   - **Size:** 2 vCPU, 4 GB RAM, 80 GB SSD (**$24/month**)
   - **Datacenter:** Choose closest to you (NYC, SFO, LON, etc.)
   - **Authentication:** SSH Key (recommended) or Password
   - **Hostname:** `ethhook`
   - Click **Create Droplet**

3. **Note your Droplet IP:** e.g., `147.182.123.456`

4. **Configure Firewall:**
   - Go to Networking → Firewalls → Create Firewall
   - **Inbound Rules:**
     - SSH (22) - Your IP only
     - HTTP (80) - All
     - HTTPS (443) - All
     - Custom (3000) - All (Admin API)
   - **Apply to:** Select your `ethhook` droplet
   - Create

---

## Step 2: Get Alchemy API Key (3 min)

1. **Go to https://alchemy.com** → Sign up (free)

2. **Create App:**
   - Name: "EthHook Sepolia"
   - Chain: Ethereum
   - Network: **Sepolia** (testnet - free)
   - Click "Create app"

3. **Copy API Key:**
   - Dashboard → View Key → Copy **HTTPS URL**
   - Should look like: `https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY`

---

## Step 3: Configure Environment (5 min)

On your **local machine**:

```bash
cd /Users/igor/rust_projects/capstone0

# Copy template
cp .env.digitalocean.example .env.production

# Generate passwords
echo "POSTGRES_PASSWORD=$(openssl rand -base64 32)"
echo "REDIS_PASSWORD=$(openssl rand -base64 32)"
echo "JWT_SECRET=$(openssl rand -base64 32)"
```

**Edit `.env.production`** and fill in:

```bash
# Use the passwords you just generated
POSTGRES_PASSWORD=<paste-generated-password-here>
REDIS_PASSWORD=<paste-generated-password-here>
JWT_SECRET=<paste-generated-password-here>

# Database URL (replace YOUR_POSTGRES_PASSWORD with the password above)
DATABASE_URL=postgresql://ethhook:YOUR_POSTGRES_PASSWORD@postgres:5432/ethhook

# Alchemy API URL from Step 2
ETHEREUM_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# Redis (these stay as-is)
REDIS_HOST=redis
REDIS_PORT=6379

# Other settings (optional, defaults are fine)
WORKER_COUNT=50
ENVIRONMENT=development
RUST_LOG=info
```

---

## Step 4: Deploy (15 min)

```bash
# Run deployment script
./deploy.sh YOUR_DROPLET_IP

# Example:
./deploy.sh 147.182.123.456
```

The script will:
1. ✓ Check environment file
2. ✓ Copy files to Droplet
3. ✓ Install Docker
4. ✓ Build images (~10 min - grab coffee! ☕)
5. ✓ Start all services
6. ✓ Verify health

---

## Step 5: Run Migrations (2 min)

```bash
# SSH into Droplet
ssh root@YOUR_DROPLET_IP

# Go to project directory
cd /root/ethhook

# Run migrations
docker exec ethhook-admin-api sh -c 'sqlx migrate run --source /app/migrations'

# Verify tables created
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "\dt"

# Should see: users, applications, endpoints, events, webhook_deliveries
```

---

## Step 6: Verify (3 min)

```bash
# Check all services running
docker-compose -f docker-compose.prod.yml ps

# Test health endpoints
curl http://YOUR_DROPLET_IP:3000/health      # Admin API
curl http://YOUR_DROPLET_IP:8080/health      # Event Ingestor
curl http://YOUR_DROPLET_IP:8081/health      # Message Processor
curl http://YOUR_DROPLET_IP:8082/health      # Webhook Delivery

# All should return: {"status":"ok"}
```

---

## Step 7: Create First User (2 min)

```bash
# Create admin user
curl -X POST http://YOUR_DROPLET_IP:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "your-secure-password"
  }'

# Returns: {"token":"eyJ..."}
# Save this token!
```

---

## 🎉 Done!

Your EthHook instance is running at:
- **Admin API:** `http://YOUR_DROPLET_IP:3000`
- **Grafana:** `http://YOUR_DROPLET_IP:3001` (admin/admin)

### What's Running:

```
┌─────────────────────────────────────┐
│     DigitalOcean Droplet ($24/mo)  │
│  ┌─────────────────────────────┐   │
│  │  Docker Containers:         │   │
│  │                              │   │
│  │  • PostgreSQL               │   │
│  │  • Redis                    │   │
│  │  • Event Ingestor           │   │
│  │  • Message Processor        │   │
│  │  • Webhook Delivery         │   │
│  │  • Admin API                │   │
│  │  • Prometheus               │   │
│  │  • Grafana                  │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

---

## 🔒 Important: Set Up Backups!

**Your data is in Docker volumes.** Set up automatic backups:

```bash
# SSH into Droplet
ssh root@YOUR_DROPLET_IP

# Test backup script
cd /root/ethhook
./scripts/backup.sh

# Set up automatic daily backups (3 AM)
crontab -e

# Add this line:
0 3 * * * /root/ethhook/scripts/backup.sh >> /var/log/ethhook-backup.log 2>&1
```

Backups are stored in `/root/ethhook-backups/`

**Also consider:**
- DigitalOcean Droplet Snapshots (automatic backups of entire server)
- Copy backups to external storage (S3, DigitalOcean Spaces, etc.)

---

## 📊 Useful Commands

```bash
# SSH into Droplet
ssh root@YOUR_DROPLET_IP

# View logs
cd /root/ethhook
docker-compose -f docker-compose.prod.yml logs -f

# Restart services
docker-compose -f docker-compose.prod.yml restart

# Check status
docker-compose -f docker-compose.prod.yml ps

# Stop all
docker-compose -f docker-compose.prod.yml down

# Start all
docker-compose -f docker-compose.prod.yml up -d

# Backup now
./scripts/backup.sh

# View backups
ls -lh /root/ethhook-backups/
```

---

## 🚀 Next Steps

1. **Set up SSL/TLS** (for HTTPS):
   - Get a domain name
   - Point DNS to your Droplet IP
   - Install Caddy for automatic HTTPS
   - See: [Full Deployment Guide](DIGITALOCEAN_DEPLOYMENT.md#ssl-setup)

2. **Test webhook delivery:**
   - Get test URL from https://webhook.site
   - Create endpoint via Admin API
   - Verify webhooks arrive

3. **Run load tests:**
   ```bash
   ./target/release/load-tester \
     --events 100 \
     --rate 50 \
     --chain-id 11155111 \
     --redis-url redis://:YOUR_REDIS_PASSWORD@YOUR_DROPLET_IP:6379
   ```

4. **Monitor performance:**
   - Open Grafana: `http://YOUR_DROPLET_IP:3001`
   - Login: admin/admin
   - View EthHook dashboards

---

## 💰 Cost Summary

| Item | Monthly Cost |
|------|--------------|
| Droplet (2vCPU, 4GB) | $24 |
| Alchemy (Free tier) | $0 |
| **Total** | **$24/month** |

**Savings vs Managed Services:** $30/month (no separate DB/Redis)

---

## ❓ Troubleshooting

**Services won't start:**
```bash
# Check environment file
cat /root/ethhook/.env.production

# Rebuild
docker-compose -f docker-compose.prod.yml down
docker-compose -f docker-compose.prod.yml build --no-cache
docker-compose -f docker-compose.prod.yml up -d
```

**Out of memory:**
- Upgrade Droplet to 8GB RAM ($48/month)

**Need help:**
- Check logs: `docker-compose logs`
- See full guide: [DIGITALOCEAN_DEPLOYMENT.md](DIGITALOCEAN_DEPLOYMENT.md)

---

**Ready to deploy? Start with Step 1!** 🚀
