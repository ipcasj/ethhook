# EthHook DigitalOcean Deployment Guide

Complete step-by-step guide to deploy EthHook to DigitalOcean.

## 📋 Prerequisites

- [ ] DigitalOcean account (sign up at https://digitalocean.com)
- [ ] Alchemy account for Ethereum RPC (sign up at https://alchemy.com)
- [ ] SSH key configured on your machine
- [ ] Basic command line knowledge

**Estimated Time:** 45-60 minutes
**Monthly Cost:** ~$54 ($24 Droplet + $15 PostgreSQL + $15 Redis)

---

## Part 1: Create DigitalOcean Resources (20 min)

### Step 1: Create Managed PostgreSQL Database

1. **Go to DigitalOcean Console** → Databases → Create Database
   - **Engine:** PostgreSQL 15
   - **Plan:** Basic ($15/month)
     - 1 GB RAM
     - 10 GB Disk
     - 1 vCPU
   - **Datacenter:** Choose closest to you (e.g., NYC1, SFO3, AMS3)
   - **Database name:** `ethhook`
   - Click **Create Database**

2. **Wait 3-5 minutes** for database to provision

3. **Get Connection Details:**
   - Go to database → Connection Details
   - Copy these values for later:
     - Host
     - Port (usually 25060)
     - Username
     - Password
     - Database name

4. **Connection String will look like:**
   ```
   postgresql://username:password@host-do-user-xxx.db.ondigitalocean.com:25060/ethhook?sslmode=require
   ```

### Step 2: Create Managed Redis

1. **Go to Databases** → Create Database
   - **Engine:** Redis
   - **Plan:** Basic ($15/month)
     - 1 GB RAM
     - 1 vCPU
   - **Datacenter:** Same as PostgreSQL
   - Click **Create Database**

2. **Wait 3-5 minutes** for Redis to provision

3. **Get Connection Details:**
   - Go to Redis → Connection Details
   - Copy these values:
     - Host
     - Port (usually 25061)
     - Password

### Step 3: Create Droplet (VM)

1. **Go to Droplets** → Create Droplet
   - **Image:** Ubuntu 22.04 LTS
   - **Plan:** Basic
     - **Size:** 2 vCPU, 4 GB RAM, 80 GB SSD ($24/month)
   - **Datacenter:** Same region as DB/Redis
   - **Authentication:** SSH Key (recommended) or Password
   - **Hostname:** `ethhook-production`
   - Click **Create Droplet**

2. **Wait 1-2 minutes** for Droplet to provision

3. **Note the IP address** (e.g., 147.182.123.456)

### Step 4: Configure Firewall (Security)

1. **Go to Networking** → Firewalls → Create Firewall
   - **Name:** `ethhook-firewall`

2. **Inbound Rules:**
   ```
   - SSH (22)          - Your IP only
   - HTTP (80)         - All IPv4, All IPv6
   - HTTPS (443)       - All IPv4, All IPv6
   - Custom (3000)     - All IPv4, All IPv6  (Admin API)
   - Custom (3001)     - Your IP only        (Grafana)
   ```

3. **Apply to Droplets:** Select `ethhook-production`

4. **Create Firewall**

---

## Part 2: Get Ethereum RPC Access (5 min)

### Step 5: Create Alchemy Account

1. **Go to https://alchemy.com** → Sign up (free)

2. **Create App:**
   - Click **Create App**
   - **Name:** EthHook Sepolia
   - **Chain:** Ethereum
   - **Network:** Sepolia (testnet - for demo)
   - Click **Create app**

3. **Get API Key:**
   - Go to app dashboard
   - Click **View Key**
   - Copy **HTTPS URL**
   - Should look like: `https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY`

**Note:** Alchemy free tier gives you 300M compute units/month (enough for testing)

---

## Part 3: Configure Environment (5 min)

### Step 6: Create Production Environment File

On your **local machine**:

```bash
cd /Users/igor/rust_projects/capstone0

# Copy example file
cp .env.digitalocean.example .env.production

# Edit with your values
nano .env.production  # or use your favorite editor
```

### Step 7: Fill in Environment Variables

Edit `.env.production` with your actual values:

```bash
# Database (from Step 1)
DATABASE_URL=postgresql://username:password@host.db.ondigitalocean.com:25060/ethhook?sslmode=require

# Redis (from Step 2)
REDIS_HOST=your-redis-host.db.ondigitalocean.com
REDIS_PORT=25061
REDIS_PASSWORD=your-redis-password

# Ethereum RPC (from Step 5)
ETHEREUM_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# Generate JWT secret
JWT_SECRET=$(openssl rand -base64 32)

# Environment
ENVIRONMENT=development  # Use 'development' for Sepolia testnet
```

**To generate JWT_SECRET:**
```bash
openssl rand -base64 32
```

---

## Part 4: Deploy to DigitalOcean (15 min)

### Step 8: SSH Key Setup (if not done)

```bash
# Check if you have SSH key
ls ~/.ssh/id_rsa.pub

# If not, generate one
ssh-keygen -t rsa -b 4096 -C "your_email@example.com"

# Copy to DigitalOcean (if using password initially)
ssh-copy-id root@YOUR_DROPLET_IP
```

### Step 9: Run Deployment Script

```bash
# Make sure you're in project root
cd /Users/igor/rust_projects/capstone0

# Run deployment
./deploy.sh YOUR_DROPLET_IP

# Example:
./deploy.sh 147.182.123.456
```

The script will:
1. ✓ Check .env.production exists
2. ✓ Test SSH connection
3. ✓ Create project directory
4. ✓ Copy files to Droplet
5. ✓ Install Docker & Docker Compose
6. ✓ Build Docker images (10-15 min)
7. ✓ Start all services
8. ✓ Verify deployment

**Note:** First build takes 10-15 minutes. Grab a coffee! ☕

---

## Part 5: Run Database Migrations (5 min)

### Step 10: SSH into Droplet and Run Migrations

```bash
# SSH into your Droplet
ssh root@YOUR_DROPLET_IP

# Navigate to project directory
cd /root/ethhook

# Check if migrations directory exists
ls migrations/

# Run migrations using admin-api container
docker exec ethhook-admin-api sh -c '
  cd /app &&
  sqlx migrate run --source /app/migrations
'

# Verify tables created
docker exec ethhook-postgres psql -U ethhook -d ethhook -c "\dt"
```

You should see tables:
- users
- applications
- endpoints
- events
- webhook_deliveries

---

## Part 6: Verify Deployment (5 min)

### Step 11: Check Service Health

```bash
# Check all services are running
docker-compose -f docker-compose.prod.yml ps

# Should show all services as "Up" and "healthy"
```

### Step 12: Test Health Endpoints

```bash
# From your local machine
curl http://YOUR_DROPLET_IP:3000/health      # Admin API
curl http://YOUR_DROPLET_IP:8080/health      # Event Ingestor
curl http://YOUR_DROPLET_IP:8081/health      # Message Processor
curl http://YOUR_DROPLET_IP:8082/health      # Webhook Delivery

# All should return: {"status":"ok"}
```

### Step 13: View Logs

```bash
# SSH into Droplet
ssh root@YOUR_DROPLET_IP
cd /root/ethhook

# View logs from all services
docker-compose -f docker-compose.prod.yml logs -f

# View specific service
docker-compose -f docker-compose.prod.yml logs -f admin-api

# Exit logs with Ctrl+C
```

---

## Part 7: Create First User & Test (5 min)

### Step 14: Create Admin User

```bash
# From your local machine
curl -X POST http://YOUR_DROPLET_IP:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "your-secure-password"
  }'

# Should return: {"token":"eyJ..."}
```

### Step 15: Create Application

```bash
# Save token from previous step
TOKEN="eyJ..."

# Create application
curl -X POST http://YOUR_DROPLET_IP:3000/applications \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Test App"
  }'

# Returns application_id and api_key
```

### Step 16: Create Webhook Endpoint

```bash
APP_ID="<from previous step>"

# Create endpoint for USDC transfers on Sepolia
curl -X POST http://YOUR_DROPLET_IP:3000/applications/$APP_ID/endpoints \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "USDC Transfers",
    "webhook_url": "https://webhook.site/YOUR-UNIQUE-URL",
    "contract_addresses": ["0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"],
    "event_signatures": ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
    "chain_ids": [11155111]
  }'
```

**Note:** Get a test webhook URL at https://webhook.site

---

## Part 8: Run Load Test Against Production (5 min)

### Step 17: Update Load Tester Configuration

On your **local machine**:

```bash
# Test against production
./target/release/load-tester \
  --events 100 \
  --rate 50 \
  --chain-id 11155111 \
  --redis-url redis://YOUR_REDIS_PASSWORD@YOUR_REDIS_HOST:25061
```

**Note:** You'll need to allow your IP in Redis firewall or use DO VPC

---

## 🎉 Deployment Complete!

Your EthHook instance is now running in production!

### Service URLs

- **Admin API:** `http://YOUR_DROPLET_IP:3000`
- **Grafana:** `http://YOUR_DROPLET_IP:3001` (login: admin/admin)
- **Prometheus:** `http://YOUR_DROPLET_IP:9092`

### Next Steps

1. **Set up SSL/TLS:**
   ```bash
   # Install Caddy for automatic HTTPS
   sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
   curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
   curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
   sudo apt update
   sudo apt install caddy
   ```

2. **Point your domain:** Configure DNS A record to your Droplet IP

3. **Configure Caddy:**
   ```bash
   sudo nano /etc/caddy/Caddyfile
   ```
   ```
   api.yourdomain.com {
       reverse_proxy localhost:3000
   }
   ```

4. **Enable automatic backups** in DigitalOcean console for database

5. **Set up monitoring alerts** in Grafana

---

## 📊 Monitoring & Troubleshooting

### View Service Status
```bash
ssh root@YOUR_DROPLET_IP
cd /root/ethhook
docker-compose -f docker-compose.prod.yml ps
```

### Restart Services
```bash
docker-compose -f docker-compose.prod.yml restart
```

### View Logs
```bash
docker-compose -f docker-compose.prod.yml logs -f admin-api
```

### Check Resource Usage
```bash
docker stats
```

### Database Queries
```bash
docker exec ethhook-postgres psql -U ethhook -d ethhook
```

---

## 💰 Cost Breakdown

| Resource | Specs | Monthly Cost |
|----------|-------|--------------|
| Droplet | 2 vCPU, 4GB RAM | $24 |
| PostgreSQL | 1GB RAM, 10GB disk | $15 |
| Redis | 1GB RAM | $15 |
| Alchemy | Free tier | $0 |
| **Total** | | **$54/month** |

**Additional costs:**
- Domain name: ~$12/year
- SSL certificate: Free (Let's Encrypt)
- Backups: $5-10/month (optional)

---

## 🆘 Common Issues

### Issue: Services won't start

**Solution:**
```bash
# Check logs
docker-compose -f docker-compose.prod.yml logs

# Check environment variables
cat .env.production

# Rebuild
docker-compose -f docker-compose.prod.yml down
docker-compose -f docker-compose.prod.yml build --no-cache
docker-compose -f docker-compose.prod.yml up -d
```

### Issue: Cannot connect to database

**Solution:**
1. Check database is running in DO console
2. Verify connection string in `.env.production`
3. Ensure Droplet IP is in database's trusted sources (DO console → Database → Settings → Trusted Sources)

### Issue: Out of memory

**Solution:**
Upgrade Droplet to 8GB RAM ($48/month)

---

## 📚 Additional Resources

- [DigitalOcean Documentation](https://docs.digitalocean.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Caddy Server](https://caddyserver.com/docs/)
- [Alchemy Documentation](https://docs.alchemy.com/)

---

## 🔒 Security Checklist

- [ ] Changed default passwords
- [ ] JWT_SECRET is randomly generated (32+ characters)
- [ ] Firewall configured properly
- [ ] Database SSL enabled (sslmode=require)
- [ ] SSH key authentication (no password login)
- [ ] Regular security updates: `apt update && apt upgrade`
- [ ] Database backups enabled
- [ ] Monitoring alerts configured

---

**Need help?** Open an issue on GitHub or check the troubleshooting section above.

Good luck with your deployment! 🚀
