# DigitalOcean Deployment Checklist

Quick reference for deploying EthHook to DigitalOcean.

**Simple guide:** See [DO_DEPLOY_SIMPLE.md](./DO_DEPLOY_SIMPLE.md) ← **Start here!**
**Full guide:** See [DIGITALOCEAN_DEPLOYMENT.md](./DIGITALOCEAN_DEPLOYMENT.md)

**Setup:** Everything on one Droplet (PostgreSQL + Redis + all services in Docker)
**Cost:** $24/month

---

## Pre-Deployment (Before You Start)

- [ ] DigitalOcean account created
- [ ] Alchemy account created (for Ethereum RPC)
- [ ] SSH key generated on your machine
- [ ] Credit card added to DigitalOcean (for billing)

---

## DigitalOcean Resources (5 min)

- [ ] **Droplet** created
  - Ubuntu 22.04
  - 2 vCPU, 4GB RAM ($24/month)
  - Note IP address: `_________________`
- [ ] **Firewall** configured
  - SSH (22) - Your IP only
  - HTTP (80) - All
  - HTTPS (443) - All
  - API (3000) - All

**Note:** PostgreSQL and Redis run in Docker containers on the Droplet (not separate services)

---

## Ethereum RPC (5 min)

- [ ] Alchemy app created
- [ ] API key copied
- [ ] RPC URL: `https://eth-sepolia.g.alchemy.com/v2/_______________`

---

## Environment Configuration (5 min)

- [ ] Copied `.env.digitalocean.example` → `.env.production`
- [ ] Generated passwords:
  - [ ] POSTGRES_PASSWORD: `openssl rand -base64 32`
  - [ ] REDIS_PASSWORD: `openssl rand -base64 32`
  - [ ] JWT_SECRET: `openssl rand -base64 32`
- [ ] DATABASE_URL filled in (using POSTGRES_PASSWORD)
- [ ] ETHEREUM_RPC_URL filled in
- [ ] All values double-checked

---

## Deployment (15 min)

- [ ] SSH key added to Droplet
- [ ] Run: `./deploy.sh YOUR_DROPLET_IP`
- [ ] Deployment completed successfully
- [ ] All services showing as "Up" and "healthy"

---

## Database Setup (5 min)

- [ ] SSH into Droplet: `ssh root@YOUR_DROPLET_IP`
- [ ] Run migrations
- [ ] Verify tables created

---

## Testing (10 min)

- [ ] Health checks pass:
  - [ ] `curl http://YOUR_DROPLET_IP:3000/health`
  - [ ] `curl http://YOUR_DROPLET_IP:8080/health`
  - [ ] `curl http://YOUR_DROPLET_IP:8081/health`
  - [ ] `curl http://YOUR_DROPLET_IP:8082/health`
- [ ] Admin user created
- [ ] Application created
- [ ] Test endpoint created
- [ ] Webhook received at webhook.site

---

## Post-Deployment (Important!)

- [ ] **Set up automatic backups** (CRITICAL!)
  - [ ] Test backup: `./scripts/backup.sh`
  - [ ] Set up cron job for daily backups
  - [ ] Verify backups in `/root/ethhook-backups/`
- [ ] Domain name configured (optional)
- [ ] SSL/TLS set up with Caddy (optional)
- [ ] Monitoring alerts configured (optional)

---

## Connection Details

Fill these in as you go:

```
Droplet IP:       _________________
PostgreSQL:       _________________
Redis Host:       _________________
Alchemy API Key:  _________________
JWT Secret:       _________________
```

---

## Quick Commands

```bash
# Deploy
./deploy.sh YOUR_DROPLET_IP

# SSH into Droplet
ssh root@YOUR_DROPLET_IP

# View logs
cd /root/ethhook
docker-compose -f docker-compose.prod.yml logs -f

# Restart services
docker-compose -f docker-compose.prod.yml restart

# Check status
docker-compose -f docker-compose.prod.yml ps

# Run migrations
docker exec ethhook-admin-api sh -c 'cd /app && sqlx migrate run'
```

---

## Costs

- Droplet: $24/month
- Alchemy: $0/month (free tier)
- **Total: $24/month**

**Savings:** $30/month vs managed PostgreSQL + Redis

---

## Support

- **Full Guide:** [DIGITALOCEAN_DEPLOYMENT.md](./DIGITALOCEAN_DEPLOYMENT.md)
- **Issues:** Check logs with `docker-compose logs`
- **Help:** Open GitHub issue

---

**Estimated Total Time:** 60 minutes
