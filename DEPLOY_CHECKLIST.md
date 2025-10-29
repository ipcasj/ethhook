# DigitalOcean Deployment Checklist

Quick reference for deploying EthHook to DigitalOcean.

**Full guide:** See [DIGITALOCEAN_DEPLOYMENT.md](./DIGITALOCEAN_DEPLOYMENT.md)

---

## Pre-Deployment (Before You Start)

- [ ] DigitalOcean account created
- [ ] Alchemy account created (for Ethereum RPC)
- [ ] SSH key generated on your machine
- [ ] Credit card added to DigitalOcean (for billing)

---

## DigitalOcean Resources (20 min)

- [ ] **PostgreSQL Database** created
  - 1GB RAM, 10GB disk ($15/month)
  - Note connection string
- [ ] **Redis Database** created
  - 1GB RAM ($15/month)
  - Note host, port, password
- [ ] **Droplet** created
  - Ubuntu 22.04
  - 2 vCPU, 4GB RAM ($24/month)
  - Note IP address: `_________________`
- [ ] **Firewall** configured
  - SSH (22) - Your IP only
  - HTTP (80) - All
  - HTTPS (443) - All
  - API (3000) - All
  - Grafana (3001) - Your IP only

---

## Ethereum RPC (5 min)

- [ ] Alchemy app created
- [ ] API key copied
- [ ] RPC URL: `https://eth-sepolia.g.alchemy.com/v2/_______________`

---

## Environment Configuration (5 min)

- [ ] Copied `.env.digitalocean.example` → `.env.production`
- [ ] DATABASE_URL filled in
- [ ] REDIS_HOST filled in
- [ ] REDIS_PORT filled in
- [ ] REDIS_PASSWORD filled in
- [ ] ETHEREUM_RPC_URL filled in
- [ ] JWT_SECRET generated: `openssl rand -base64 32`
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

## Post-Deployment (Optional)

- [ ] Domain name configured
- [ ] SSL/TLS set up (Caddy)
- [ ] Database backups enabled
- [ ] Monitoring alerts configured
- [ ] Documentation updated with production URLs

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
- PostgreSQL: $15/month
- Redis: $15/month
- **Total: $54/month**

---

## Support

- **Full Guide:** [DIGITALOCEAN_DEPLOYMENT.md](./DIGITALOCEAN_DEPLOYMENT.md)
- **Issues:** Check logs with `docker-compose logs`
- **Help:** Open GitHub issue

---

**Estimated Total Time:** 60 minutes
