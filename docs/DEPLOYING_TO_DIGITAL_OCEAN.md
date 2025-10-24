# Deploying EthHook to Digital Ocean

Complete guide to deploying EthHook on Digital Ocean with Grafana monitoring.

## Overview

This guide covers:

- Setting up a Digital Ocean Droplet
- Deploying all EthHook services with Docker Compose
- Configuring PostgreSQL and Redis
- Setting up Grafana and Prometheus monitoring
- Configuring a custom domain with SSL
- Security best practices

**Estimated time**: 30-45 minutes  
**Cost**: ~$40-60/month for production-ready setup

---

## Prerequisites

- Digital Ocean account (<https://www.digitalocean.com>)
- Domain name (for SSL/custom domain)
- Local machine with:
  - Docker and Docker Compose installed
  - SSH client
  - Git

---

## Architecture

The deployment includes:

- **Droplet**: Ubuntu 22.04 LTS (4 GB Memory, 2 vCPUs, 80 GB SSD)
- **Services**: event-ingestor, message-processor, webhook-delivery, admin-api, leptos-portal, demo-webhook-receiver
- **Database**: PostgreSQL 15 (managed or self-hosted)
- **Cache**: Redis 7 (self-hosted on droplet)
- **Monitoring**: Prometheus + Grafana
- **Reverse Proxy**: Nginx with Let's Encrypt SSL

---

## Step 1: Create Digital Ocean Droplet

### Option A: Using Web Console

1. Log in to Digital Ocean
2. Click **Create** → **Droplets**
3. Choose configuration:
   - **Image**: Ubuntu 22.04 LTS
   - **Plan**: Basic ($24/month - 4 GB / 2 CPUs)
   - **Datacenter**: Choose closest to your users
   - **Authentication**: SSH Key (recommended)
   - **Hostname**: ethhook-production
4. Click **Create Droplet**

### Option B: Using CLI

```bash
# Install doctl
brew install doctl  # macOS
# OR
snap install doctl  # Linux

# Authenticate
doctl auth init

# Create droplet
doctl compute droplet create ethhook-production \
  --region nyc1 \
  --size s-2vcpu-4gb \
  --image ubuntu-22-04-x64 \
  --ssh-keys YOUR_SSH_KEY_ID \
  --wait
```

### Get Droplet IP

```bash
# Using CLI
doctl compute droplet list

# Or check the web console
# Note the public IPv4 address
```

---

## Step 2: Initial Server Setup

### SSH into Droplet

```bash
ssh root@YOUR_DROPLET_IP
```

### Update System

```bash
apt update && apt upgrade -y
```

### Install Docker

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Install Docker Compose
apt install docker-compose-plugin -y

# Verify installation
docker --version
docker compose version
```

### Create Non-Root User (Optional but Recommended)

```bash
# Create user
adduser ethhook

# Add to docker group
usermod -aG docker ethhook

# Add sudo privileges
usermod -aG sudo ethhook

# Switch to new user
su - ethhook
```

---

## Step 3: Setup PostgreSQL

### Option A: Managed Database (Recommended)

1. In Digital Ocean console: **Databases** → **Create Database**
2. Choose:
   - **Database Engine**: PostgreSQL 15
   - **Plan**: Basic ($15/month - 1 GB RAM, 10 GB SSD)
   - **Datacenter**: Same as your droplet
3. Click **Create Database**
4. In **Connection Details**, note:
   - Host
   - Port
   - Username
   - Password
   - Database name
5. Add your droplet to **Trusted Sources**

### Option B: Self-Hosted on Droplet

```bash
# Install PostgreSQL
sudo apt install postgresql postgresql-contrib -y

# Start service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres psql << EOF
CREATE DATABASE ethhook;
CREATE USER ethhook WITH ENCRYPTED PASSWORD 'your-secure-password';
GRANT ALL PRIVILEGES ON DATABASE ethhook TO ethhook;
ALTER DATABASE ethhook OWNER TO ethhook;
\q
EOF

# Allow connections from Docker
sudo nano /etc/postgresql/15/main/pg_hba.conf
# Add line:
# host  all  all  172.16.0.0/12  md5

# Allow listening on all interfaces
sudo nano /etc/postgresql/15/main/postgresql.conf
# Change:
# listen_addresses = '*'

# Restart PostgreSQL
sudo systemctl restart postgresql
```

---

## Step 4: Clone Repository

```bash
cd /home/ethhook  # or your preferred directory

git clone https://github.com/yourorg/ethhook.git
cd ethhook
```

---

## Step 5: Configure Environment

### Create Production Environment File

```bash
cp .env.example .env
nano .env
```

### Configure Variables

```bash
# Database (use managed DB connection string)
DATABASE_URL=postgresql://ethhook:PASSWORD@your-db-host.db.ondigitalocean.com:25060/ethhook?sslmode=require

# Redis (local)
REDIS_URL=redis://redis:6379

# Blockchain
SEPOLIA_WS_URL=wss://ethereum-sepolia-rpc.publicnode.com
MAINNET_WS_URL=wss://ethereum-rpc.publicnode.com
# Note: For production, use dedicated RPC providers (Alchemy, Infura, QuickNode)
# SEPOLIA_WS_URL=wss://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# API Configuration
JWT_SECRET=$(openssl rand -hex 32)
API_KEY_SALT=$(openssl rand -hex 16)
ADMIN_API_PORT=8080
CORS_ALLOWED_ORIGINS=https://your-domain.com

# Webhook Delivery
HMAC_SECRET=$(openssl rand -hex 32)
MAX_RETRIES=5
INITIAL_RETRY_DELAY=60
MAX_RETRY_DELAY=3600

# Demo Webhook Receiver
# This is the secret the demo receiver uses for validation
DEMO_HMAC_SECRET=$(openssl rand -hex 32)

# Monitoring
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000
GRAFANA_ADMIN_PASSWORD=$(openssl rand -base64 12)

# Logging
RUST_LOG=info,ethhook=debug

# Domain (configure later)
DOMAIN=your-domain.com
```

**Important**: Save the generated secrets somewhere secure (password manager).

---

## Step 6: Run Database Migrations

```bash
# Install sqlx-cli (if not in Docker)
cargo install sqlx-cli --no-default-features --features postgres

# Or use Docker
docker run --rm \
  -v $(pwd)/migrations:/migrations \
  --network host \
  -e DATABASE_URL="$DATABASE_URL" \
  ghcr.io/launchbadge/sqlx-cli:latest \
  migrate run --source /migrations

# Or manually using psql
for file in migrations/*.sql; do
  psql "$DATABASE_URL" -f "$file"
done
```

---

## Step 7: Build and Deploy Services

### Build Docker Images

```bash
# Build all services
docker compose build

# This may take 10-15 minutes on first build
# Subsequent builds use cache and are faster
```

### Start Services

```bash
# Start in detached mode
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs -f

# Check specific service
docker compose logs -f event-ingestor
```

### Verify Services

```bash
# Check health endpoints
curl http://localhost:8080/health  # admin-api
curl http://localhost:8000/health  # demo-webhook-receiver

# Check Prometheus metrics
curl http://localhost:9090/metrics

# Check Grafana
curl http://localhost:3000
```

---

## Step 8: Configure Nginx Reverse Proxy

### Install Nginx

```bash
sudo apt install nginx -y
```

### Create Configuration

```bash
sudo nano /etc/nginx/sites-available/ethhook
```

Add configuration:

```nginx
# API Backend
server {
    listen 80;
    server_name api.your-domain.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Web Portal
server {
    listen 80;
    server_name app.your-domain.com;

    location / {
        proxy_pass http://localhost:8081;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}

# Grafana Monitoring
server {
    listen 80;
    server_name grafana.your-domain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}

# Demo Webhook Receiver (for demo only)
server {
    listen 80;
    server_name webhooks.your-domain.com;

    location / {
        proxy_pass http://localhost:8000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Enable Site

```bash
sudo ln -s /etc/nginx/sites-available/ethhook /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

---

## Step 9: Configure DNS

In your domain registrar (Namecheap, GoDaddy, Cloudflare, etc.):

1. Add **A records** pointing to your droplet IP:
   - `api.your-domain.com` → `YOUR_DROPLET_IP`
   - `app.your-domain.com` → `YOUR_DROPLET_IP`
   - `grafana.your-domain.com` → `YOUR_DROPLET_IP`
   - `webhooks.your-domain.com` → `YOUR_DROPLET_IP` (demo only)

2. Wait for DNS propagation (5-30 minutes)

3. Verify:

   ```bash
   nslookup api.your-domain.com
   nslookup app.your-domain.com
   ```

---

## Step 10: Setup SSL with Let's Encrypt

### Install Certbot

```bash
sudo apt install certbot python3-certbot-nginx -y
```

### Obtain Certificates

```bash
# For all subdomains at once
sudo certbot --nginx \
  -d api.your-domain.com \
  -d app.your-domain.com \
  -d grafana.your-domain.com \
  -d webhooks.your-domain.com

# Follow prompts:
# - Enter email
# - Agree to terms
# - Choose redirect HTTP to HTTPS (option 2)
```

### Auto-Renewal

```bash
# Test renewal
sudo certbot renew --dry-run

# Certbot automatically installs cron job for renewal
# Check:
sudo systemctl status certbot.timer
```

---

## Step 11: Configure Grafana

### Access Grafana

1. Open browser: `https://grafana.your-domain.com`
2. Login:
   - Username: `admin`
   - Password: (from GRAFANA_ADMIN_PASSWORD in .env)

### Add Prometheus Data Source

1. Go to **Configuration** → **Data Sources**
2. Click **Add data source**
3. Select **Prometheus**
4. Configure:
   - Name: `Prometheus`
   - URL: `http://prometheus:9090`
5. Click **Save & Test**

### Import Dashboard

1. Go to **Dashboards** → **Import**
2. Upload `monitoring/grafana-dashboard.json`
3. Select Prometheus data source
4. Click **Import**

### Configure Alerts (Optional)

1. Go to **Alerting** → **Notification channels**
2. Add channels (Email, Slack, PagerDuty, etc.)
3. Configure alert rules in dashboard panels

---

## Step 12: Create Initial User

### Using Admin API

```bash
# Create first user
curl -X POST https://api.your-domain.com/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "email": "admin@your-company.com",
    "password": "your-secure-password"
  }'

# Login to get token
curl -X POST https://api.your-domain.com/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "your-secure-password"
  }'

# Save the JWT token
export TOKEN="eyJ..."
```

### Create Application

```bash
curl -X POST https://api.your-domain.com/api/v1/applications \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Demo Application",
    "description": "Production demo for potential customers"
  }'

# Save the API key returned
```

### Create Endpoint

```bash
curl -X POST https://api.your-domain.com/api/v1/endpoints \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Demo USDC Transfers",
    "webhook_url": "https://webhooks.your-domain.com/webhook",
    "chain_ids": [11155111],
    "contract_addresses": ["0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"],
    "event_signatures": ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
    "active": true
  }'
```

---

## Step 13: Verify Deployment

### Check All Services

```bash
# Check Docker containers
docker compose ps

# All services should show "Up" status:
# - postgres (or external)
# - redis
# - event-ingestor
# - message-processor
# - webhook-delivery
# - admin-api
# - leptos-portal
# - demo-webhook-receiver
# - prometheus
# - grafana
```

### Test API

```bash
# Health check
curl https://api.your-domain.com/health

# Get events
curl https://api.your-domain.com/api/v1/events \
  -H "Authorization: Bearer $TOKEN"
```

### Test Web Portal

1. Open `https://app.your-domain.com`
2. Login with credentials
3. Check dashboard shows events and deliveries

### Test Grafana

1. Open `https://grafana.your-domain.com`
2. View EthHook dashboard
3. Verify metrics are flowing:
   - Events ingested
   - Delivery success rate
   - Service health

### Test Webhook Delivery

1. Wait for events to be captured (may take 1-5 minutes)
2. Check demo receiver:

   ```bash
   curl https://webhooks.your-domain.com/webhooks
   ```

3. View in Grafana dashboard

---

## Step 14: Security Hardening

### Firewall

```bash
# Enable UFW
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow ssh

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status
```

### Fail2Ban (SSH Protection)

```bash
# Install
sudo apt install fail2ban -y

# Configure
sudo cp /etc/fail2ban/jail.conf /etc/fail2ban/jail.local
sudo nano /etc/fail2ban/jail.local

# Find [sshd] section and enable
# bantime = 1h
# maxretry = 3

# Restart
sudo systemctl restart fail2ban
```

### Disable Root SSH

```bash
sudo nano /etc/ssh/sshd_config

# Change:
# PermitRootLogin no
# PasswordAuthentication no

sudo systemctl restart sshd
```

### Setup Automatic Updates

```bash
sudo apt install unattended-upgrades -y
sudo dpkg-reconfigure --priority=low unattended-upgrades
```

---

## Step 15: Monitoring and Maintenance

### View Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f event-ingestor

# Last 100 lines
docker compose logs --tail=100 webhook-delivery

# Since timestamp
docker compose logs --since 2024-01-01T00:00:00
```

### System Resources

```bash
# Check disk usage
df -h

# Check memory
free -h

# Check CPU
htop  # or top
```

### Database Maintenance

```bash
# Connect to database
psql "$DATABASE_URL"

# Check table sizes
SELECT
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

# Vacuum (optimize)
VACUUM ANALYZE;
```

### Backup Database

```bash
# Create backup
pg_dump "$DATABASE_URL" > backup_$(date +%Y%m%d).sql

# Or using Docker
docker compose exec postgres pg_dump -U ethhook ethhook > backup.sql

# Upload to DO Spaces or S3
```

### Update Services

```bash
# Pull latest code
git pull origin main

# Rebuild images
docker compose build

# Restart services
docker compose up -d

# Or rolling restart one by one
docker compose up -d --no-deps --build event-ingestor
docker compose up -d --no-deps --build message-processor
```

---

## Scaling Considerations

### Vertical Scaling (More Resources)

```bash
# Resize droplet in DO console
# Database → Resize (for managed DB)
# Droplet → Resize

# Or upgrade to Performance plan for better IOPS
```

### Horizontal Scaling (More Instances)

For high traffic:

1. **Multiple Webhook Delivery Workers**:

   ```yaml
   # docker-compose.yml
   webhook-delivery:
     deploy:
       replicas: 3  # Docker Swarm
   ```

2. **Load Balancer**:
   - Add DO Load Balancer
   - Distribute across multiple droplets
   - Session affinity for admin-api

3. **Redis Cluster**:
   - Migrate from single Redis to cluster mode
   - Or use DO Managed Redis

4. **Database Read Replicas**:
   - Add read replicas in DO managed database
   - Route queries appropriately

---

## Cost Breakdown

### Minimum Production Setup (~$40/month)

- Droplet (4 GB / 2 vCPU): $24/month
- Managed PostgreSQL (1 GB): $15/month
- **Total**: ~$39/month

### Recommended Setup (~$60/month)

- Droplet (8 GB / 4 vCPU): $48/month
- Managed PostgreSQL (2 GB): $30/month
- Domain: ~$10-15/year
- **Total**: ~$78/month + domain

### High-Traffic Setup (~$150-200/month)

- 2x Droplets (8 GB): $96/month
- Load Balancer: $10/month
- Managed PostgreSQL (4 GB): $60/month
- Managed Redis: $15/month
- Backups & Snapshots: $10/month
- **Total**: ~$191/month

---

## Troubleshooting

### Services Won't Start

```bash
# Check logs
docker compose logs

# Check disk space
df -h

# Check memory
free -h

# Restart individual service
docker compose restart event-ingestor
```

### Database Connection Issues

```bash
# Test connection
psql "$DATABASE_URL"

# Check if in trusted sources (managed DB)
# DO Console → Database → Settings → Trusted Sources

# Check firewall (self-hosted)
sudo ufw status
```

### SSL Certificate Issues

```bash
# Check certificate
sudo certbot certificates

# Renew manually
sudo certbot renew --force-renewal

# Check Nginx config
sudo nginx -t
```

### High Memory Usage

```bash
# Check container stats
docker stats

# Restart memory-heavy service
docker compose restart message-processor

# Or increase droplet resources
```

### Events Not Being Captured

```bash
# Check WebSocket connection
docker compose logs event-ingestor | grep -i "connection"

# Check endpoint configuration
curl https://api.your-domain.com/api/v1/endpoints \
  -H "Authorization: Bearer $TOKEN"

# Verify RPC URL works
wscat -c wss://ethereum-sepolia-rpc.publicnode.com
```

### Webhooks Not Being Delivered

```bash
# Check delivery logs
docker compose logs webhook-delivery

# Check circuit breaker status (in Grafana)

# Test webhook URL manually
curl -X POST https://webhooks.your-domain.com/webhook \
  -H "Content-Type: application/json" \
  -d '{"test": true}'
```

---

## Next Steps

1. **Remove Demo Receiver**: For production, customers use their own endpoints
2. **Setup Monitoring Alerts**: Configure Grafana alerts for critical issues
3. **Configure Backup Strategy**: Automated database backups to DO Spaces
4. **Enable CDN**: Use Cloudflare for web portal (optional)
5. **Setup CI/CD**: Automate deployments with GitHub Actions
6. **Document Runbooks**: Create operational procedures for your team

---

## Support Resources

- **Documentation**: `/docs` directory in repository
- **Customer Integration**: `CUSTOMER_INTEGRATION_GUIDE.md`
- **Testing Webhooks**: `TESTING_WEBHOOKS_LOCALLY.md`
- **Architecture**: `SYSTEM_ARCHITECTURE.md`

## Demo Checklist

Before showing to potential customers:

- [ ] All services running (`docker compose ps`)
- [ ] At least 10+ events captured
- [ ] Multiple successful webhook deliveries
- [ ] Grafana dashboard showing metrics
- [ ] Web portal accessible and responsive
- [ ] Demo receiver showing recent webhooks
- [ ] SSL certificates valid (green lock)
- [ ] No errors in logs

---

**Congratulations!** Your EthHook deployment is now live and ready for production use or customer demos.
