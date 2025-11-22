# Deployment Guide

## DigitalOcean App Platform (Recommended for Quick Start)

### Prerequisites

1. DigitalOcean account
2. GitHub repository
3. `doctl` CLI tool

### Steps

```bash
# 1. Install doctl
brew install doctl  # macOS
# or
snap install doctl  # Linux

# 2. Authenticate
doctl auth init

# 3. Create app
doctl apps create --spec .do/app.yaml

# 4. Get app URL
doctl apps list
```

### Environment Variables

Set these secrets in the DigitalOcean dashboard:

- `ETHEREUM_WS_URL` - Your Alchemy/Infura WebSocket URL
- `JWT_SECRET` - Random 256-bit secret for JWT signing

### Cost

Estimated monthly cost: **$30-50** (depending on traffic)

- Database: $15/month (dev tier)
- Redis: $5/month (basic-xxs)
- Services: $5-10/month each (basic-xs)

## Docker Compose (Development)

```bash
# 1. Clone repository
git clone https://github.com/ipcasj/ethhook-c.git
cd ethhook-c

# 2. Set environment variables
cp .env.example .env
# Edit .env with your RPC URLs

# 3. Start all services
docker compose up -d

# 4. Check health
curl http://localhost:8080/health

# 5. View logs
docker compose logs -f

# 6. Stop services
docker compose down
```

## Kubernetes

### Prerequisites

- Kubernetes cluster (v1.24+)
- `kubectl` configured
- PostgreSQL and Redis (in-cluster or managed)

### Deploy

```bash
# 1. Create namespace
kubectl create namespace ethhook

# 2. Create secrets
kubectl create secret generic ethhook-secrets \
  --from-literal=ethereum-ws-url="wss://..." \
  --from-literal=jwt-secret="..." \
  -n ethhook

# 3. Apply manifests
kubectl apply -f k8s/ -n ethhook

# 4. Check status
kubectl get pods -n ethhook

# 5. Get external IP
kubectl get svc admin-api -n ethhook
```

## AWS ECS/Fargate

### Using AWS CLI

```bash
# 1. Create ECS cluster
aws ecs create-cluster --cluster-name ethhook-cluster

# 2. Register task definitions
aws ecs register-task-definition --cli-input-json file://aws/task-definition.json

# 3. Create services
aws ecs create-service \
  --cluster ethhook-cluster \
  --service-name event-ingestor \
  --task-definition ethhook-ingestor:1 \
  --desired-count 1

# 4. Repeat for other services
```

## Bare Metal / VPS

### Prerequisites

- Ubuntu 22.04 or similar
- PostgreSQL 15+
- Redis 7+
- sudo access

### Installation

```bash
# 1. Install dependencies
./scripts/install-deps.sh

# 2. Build services
./scripts/build.sh Release

# 3. Install binaries
sudo make install

# 4. Create systemd services
sudo cp systemd/*.service /etc/systemd/system/

# 5. Start services
sudo systemctl enable --now ethhook-ingestor
sudo systemctl enable --now ethhook-processor
sudo systemctl enable --now ethhook-delivery
sudo systemctl enable --now ethhook-admin-api

# 6. Check status
sudo systemctl status ethhook-*
```

## Monitoring

### Prometheus

Scrape metrics from each service:

```yaml
scrape_configs:
  - job_name: 'ethhook'
    static_configs:
      - targets: ['ingestor:9090', 'processor:9090', 'delivery:9090', 'api:9090']
```

### Grafana

Import dashboard from `monitoring/grafana/ethhook-dashboard.json`

## Scaling

### Horizontal Scaling

Scale stateless services independently:

```bash
# Docker Compose
docker compose up -d --scale webhook-delivery=3

# Kubernetes
kubectl scale deployment webhook-delivery --replicas=3

# DigitalOcean
doctl apps update APP_ID --spec .do/app.yaml  # Edit replicas in YAML
```

### Vertical Scaling

Increase resource limits:

```yaml
# Kubernetes
resources:
  limits:
    cpu: 1000m
    memory: 512Mi
```

## High Availability

### Multi-Region Deployment

1. Deploy to multiple regions
2. Use geo-DNS routing
3. Replicate PostgreSQL (read replicas)
4. Use Redis Cluster for caching

### Database Failover

Use PostgreSQL streaming replication:

```bash
# Primary-standby setup
pg_basebackup -h primary -D /var/lib/postgresql/data -U replication -Fp -Xs -P -R
```

## Security

### Production Checklist

- [ ] Use managed PostgreSQL (RDS, Cloud SQL, etc.)
- [ ] Enable SSL/TLS for all connections
- [ ] Rotate JWT secrets regularly
- [ ] Use secrets manager (AWS Secrets Manager, Vault)
- [ ] Enable network policies (Kubernetes)
- [ ] Run as non-root user
- [ ] Keep images updated
- [ ] Enable audit logging

## Troubleshooting

### Service won't start

```bash
# Check logs
docker compose logs service-name
kubectl logs -f pod-name
journalctl -u ethhook-service-name

# Common issues:
# - Missing environment variables
# - Can't connect to database/Redis
# - Port already in use
```

### High memory usage

```bash
# Check memory stats
docker stats
kubectl top pods

# Solutions:
# - Reduce arena sizes
# - Limit concurrent connections
# - Increase swap space
```

### High latency

```bash
# Check metrics
curl http://localhost:9090/metrics

# Solutions:
# - Scale horizontally
# - Optimize database queries
# - Use connection pooling
# - Add caching
```

## Support

- GitHub Issues: https://github.com/ipcasj/ethhook-c/issues
- Discussions: https://github.com/ipcasj/ethhook-c/discussions
- Email: ihorpetroff@gmail.com
