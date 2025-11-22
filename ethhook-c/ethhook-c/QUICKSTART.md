# Quick Start Guide

Get ETHhook-C running in 5 minutes!

## Option 1: Docker (Easiest)

```bash
# 1. Clone repository
git clone https://github.com/ipcasj/ethhook-c.git
cd ethhook-c

# 2. Set environment variables
cp .env.example .env
# Edit .env and add your Ethereum RPC WebSocket URL

# 3. Start all services
docker compose up -d

# 4. Check status
docker compose ps
docker compose logs -f

# 5. Test API
curl http://localhost:8080/health
```

**That's it!** Your ETHhook-C stack is running.

---

## Option 2: Build from Source (macOS)

```bash
# 1. Install dependencies
./scripts/install-deps.sh

# 2. Build
make build

# 3. Start infrastructure (PostgreSQL + Redis)
docker compose up -d postgres redis

# 4. Set environment
export ETHEREUM_WS_URL="wss://eth-sepolia.g.alchemy.com/v2/YOUR_KEY"
export DATABASE_URL="postgresql://ethhook:password@localhost:5432/ethhook"
export REDIS_URL="redis://localhost:6379"

# 5. Run services
./build/bin/event-ingestor &
./build/bin/message-processor &
./build/bin/webhook-delivery &
./build/bin/admin-api &

# 6. Check logs
tail -f *.log
```

---

## Option 3: Deploy to DigitalOcean (Production)

```bash
# 1. Install doctl
brew install doctl  # macOS
snap install doctl  # Linux

# 2. Authenticate
doctl auth init

# 3. Set secrets
# Edit .do/app.yaml and add your secrets

# 4. Deploy
doctl apps create --spec .do/app.yaml

# 5. Get app URL
doctl apps list
```

**Cost**: $30-50/month for full stack

---

## Testing Your Setup

### 1. Health Check

```bash
curl http://localhost:8080/health
```

Expected response:
```json
{
  "status": "healthy",
  "service": "admin-api",
  "version": "1.0.0"
}
```

### 2. Create Test Application

```bash
curl -X POST http://localhost:8080/api/v1/applications \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My dApp",
    "description": "Test application"
  }'
```

### 3. View Metrics

```bash
curl http://localhost:9090/metrics
```

---

## Common Issues

### "Can't connect to PostgreSQL"

```bash
# Check PostgreSQL is running
docker compose ps postgres

# Check connection
docker compose exec postgres psql -U ethhook -d ethhook -c "SELECT 1;"
```

### "Can't connect to Redis"

```bash
# Check Redis is running
docker compose ps redis

# Test connection
docker compose exec redis redis-cli ping
```

### "Build fails"

```bash
# Clean and rebuild
make clean
make build

# Check dependencies
./scripts/install-deps.sh
```

---

## Next Steps

1. **Read the docs**: [README.md](README.md)
2. **Understand the architecture**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
3. **Deploy to production**: [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)
4. **Contribute**: [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Need Help?

- **GitHub Issues**: https://github.com/ipcasj/ethhook-c/issues
- **Email**: ihorpetroff@gmail.com

---

**You're ready to go!** 🚀
