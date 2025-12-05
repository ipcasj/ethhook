# EthHook C Implementation - Production Deployment Plan

**Decision Date**: December 4, 2025  
**Strategy**: Deploy C implementation as primary, Rust as backup  
**Rationale**: Validate C performance in production with safety net

---

## 🎯 Deployment Strategy

### Primary: C Implementation
- **Why**: 26x smaller binaries, 1.6x faster JSON parsing, proven technology
- **Risk**: Manual memory management, requires careful monitoring
- **Mitigation**: ASAN/UBSAN in staging, comprehensive monitoring, Rust fallback ready

### Backup: Rust Implementation
- **Why**: Memory safety, faster startup, better concurrency
- **Status**: Production-ready, can switch within hours if needed
- **Trigger**: Critical memory bugs, performance issues, or stability concerns

---

## ✅ Pre-Production Checklist

### 1. Build Configuration

- [ ] **Release Build with Optimizations**

  ```bash
  cd ethhook-c/build
  cmake -DCMAKE_BUILD_TYPE=Release \
        -DENABLE_LTO=ON \
        -DCMAKE_C_FLAGS="-O3 -march=native" ..
  make -j$(nproc)
  ```

- [ ] **Strip Debug Symbols** (reduce binary size further)

  ```bash
  strip ethhook-admin-api
  strip ethhook-ingestor
  strip ethhook-processor
  strip ethhook-delivery
  ```

- [ ] **Verify Binary Sizes**

  ```bash
  ls -lh ethhook-* | awk '{print $9, $5}'
  # Expected: 250-350 KB per service
  ```

### 2. Security Hardening

- [ ] **Enable All Security Features**
  - ✅ `-fstack-protector-strong` (already enabled)
  - ✅ `_FORTIFY_SOURCE=2` (already enabled)
  - ✅ `-fPIE -pie` (position-independent executable)
  - ✅ `-Wformat -Wformat-security` (format string protection)

- [ ] **Verify Security Flags**

  ```bash
  checksec --file=ethhook-admin-api
  # Expected: RELRO, Stack Canary, NX, PIE all enabled
  ```

- [ ] **Run Security Audit**

  ```bash
  # Static analysis
  cppcheck --enable=all --error-exitcode=1 ../src/
  clang-tidy ../src/**/*.c
  
  # Dependency audit
  ldd ethhook-admin-api  # Check for vulnerable libs
  ```

### 3. Testing & Validation

- [ ] **Unit Tests** (if available)

  ```bash
  make test
  # All tests must pass
  ```

- [ ] **Integration Tests**

  ```bash
  # Test JWT authentication
  ./ethhook-admin-api --test-jwt
  
  # Test SQLite database connectivity and schema
  sqlite3 ethhook.db ".tables"  # List all tables
  sqlite3 ethhook.db "SELECT COUNT(*) FROM webhooks;"
  sqlite3 ethhook.db "SELECT COUNT(*) FROM subscriptions;"
  sqlite3 ethhook.db "SELECT COUNT(*) FROM events;"
  sqlite3 ethhook.db "PRAGMA integrity_check;"  # Verify database integrity
  sqlite3 ethhook.db "PRAGMA journal_mode;"     # Should return WAL
  
  # Test Redis connectivity and operations
  redis-cli ping
  redis-cli SET test_key "test_value" EX 10
  redis-cli GET test_key
  redis-cli DEL test_key
  ```

- [ ] **Load Testing** (use existing load test scripts)

  ```bash
  cd ../../demo-webhook-receiver
  python3 load_test_receiver.py --duration 60 --concurrency 50
  # Monitor: CPU < 70%, Memory < 500 MB, No crashes
  ```

- [ ] **Memory Leak Testing**

  ```bash
  # Run with Valgrind
  valgrind --leak-check=full --show-leak-kinds=all \
           --track-origins=yes \
           ./ethhook-processor
  # Expected: 0 leaks, 0 errors
  ```

### 4. Monitoring & Observability

- [ ] **Prometheus Metrics Endpoint**
  - Verify `/metrics` endpoint on each service
  - Check metric types: counters, gauges, histograms
  - Key metrics: requests_total, response_time, memory_usage, errors_total

- [ ] **Grafana Dashboards**
  - Import dashboards from `monitoring/grafana/`
  - Verify panels: CPU, Memory, Request Rate, Error Rate, Latency (p50, p95, p99)

- [ ] **Alerting Rules**

  ```yaml
  # monitoring/prometheus/alerts.yml
  - alert: HighErrorRate
    expr: rate(errors_total[5m]) > 0.05
    for: 5m
    
  - alert: HighMemoryUsage
    expr: process_resident_memory_bytes > 500000000  # 500 MB
    for: 10m
    
  - alert: ServiceDown
    expr: up{job="ethhook"} == 0
    for: 2m
  ```

- [ ] **Logging Configuration**
  - Set log level: INFO for production (ERROR for critical paths)
  - Log format: JSON (for structured logging)
  - Log rotation: daily, keep 30 days
  - Log aggregation: Ship to ELK/Loki/CloudWatch

### 5. Deployment Infrastructure

✅ **ALREADY CONFIGURED** - Reuse existing infrastructure!

- [ ] **Docker Images** (Already exist in `ethhook-c/docker/`)
  - ✅ `Dockerfile.admin-api` - Multi-stage Alpine build with security hardening
  - ✅ `Dockerfile.ingestor` - WebSocket event ingestion service
  - ✅ `Dockerfile.processor` - Event matching and processing
  - ✅ `Dockerfile.delivery` - Webhook delivery service
  - All use non-root user (uid 1000), stripped binaries, minimal Alpine base

- [ ] **Docker Compose** (Already configured)
  - ✅ `ethhook-c/docker/docker-compose.yml` - Local development
  - ✅ `ethhook-c/docker/docker-compose.prod.yml` - Production config
  - Services: Redis, all 4 C services with health checks and proper dependencies

- [ ] **CI/CD Pipeline** (Already configured in `.github/workflows/`)
  - ✅ `ci.yml` - Automated testing for Rust (can extend for C)
  - ✅ `deploy-digitalocean.yml` - Automated deployment to production
  - Supports manual triggers, environment management, health checks

- [ ] **Extend Existing CI/CD for C**

  ```yaml
  # k8s/ethhook-c-deployment.yaml
  apiVersion: apps/v1
  kind: Deployment
  metadata:
    name: ethhook-admin-api-c
    labels:
      app: ethhook
      implementation: c
      component: admin-api
  spec:
    replicas: 3
    strategy:
      type: RollingUpdate
      rollingUpdate:
        maxSurge: 1
        maxUnavailable: 0
    selector:
      matchLabels:
        app: ethhook
        implementation: c
        component: admin-api
    template:
      metadata:
        labels:
          app: ethhook
          implementation: c
          component: admin-api
      spec:
        containers:
        - name: admin-api
          image: ethhook-c:latest
          ports:
          - containerPort: 8080
            name: http
          - containerPort: 9090
            name: metrics
          env:
          - name: DATABASE_URL
            valueFrom:
              secretKeyRef:
                name: ethhook-secrets
                key: database-url
          - name: REDIS_URL
            valueFrom:
              secretKeyRef:
                name: ethhook-secrets
                key: redis-url
          - name: JWT_SECRET
            valueFrom:
              secretKeyRef:
                name: ethhook-secrets
                key: jwt-secret
          resources:
            requests:
              memory: "128Mi"
              cpu: "100m"
            limits:
              memory: "512Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          securityContext:
            runAsNonRoot: true
            runAsUser: 1001
            readOnlyRootFilesystem: true
            allowPrivilegeEscalation: false
            capabilities:
              drop:
                - ALL
  ```

### 6. Database & Dependencies

- [ ] **SQLite Setup**
  - Version: 3.35+ (with WAL mode support)
  - Journal mode: WAL (Write-Ahead Logging for better concurrency)
  - Database file: `ethhook.db` (or path from DATABASE_URL)
  - Backup: Automated daily backups using `sqlite3 .backup` or file copy
  - Monitoring: Database size, WAL size, checkpoint frequency
  - Permissions: Ensure write access to database file and directory

- [ ] **Redis Setup**
  - Version: 7+ (for streams support)
  - Persistence: AOF + RDB (hybrid)
  - Max memory: 2 GB with `allkeys-lru` eviction
  - Monitoring: Memory usage, hit rate, connected clients

- [ ] **Run Migrations**

  ```bash
  # Apply schema migrations
  sqlite3 ethhook.db < migrations/001_initial_schema.sql
  sqlite3 ethhook.db < migrations/002_add_indexes.sql
  
  # Enable WAL mode for production
  sqlite3 ethhook.db "PRAGMA journal_mode=WAL;"
  sqlite3 ethhook.db "PRAGMA busy_timeout=5000;"
  
  # Verify: Check table counts and indexes
  sqlite3 ethhook.db ".schema" | head -20
  sqlite3 ethhook.db "SELECT name FROM sqlite_master WHERE type='table';"
  ```

### 7. Environment Configuration

- [ ] **Production Environment Variables**

  ```bash
  # /etc/ethhook/prod.env
  DATABASE_URL=/var/lib/ethhook/ethhook.db
  REDIS_URL=redis://prod-redis:6379/0
  JWT_SECRET=<strong-random-secret-256-bits>
  LOG_LEVEL=info
  METRICS_PORT=9090
  API_PORT=8080
  WORKER_THREADS=4
  MAX_CONNECTIONS=100
  ENABLE_METRICS=true
  ENVIRONMENT=production
  ```

- [ ] **Secrets Management**
  - Use Kubernetes Secrets / AWS Secrets Manager / Vault
  - Rotate secrets quarterly
  - Encrypt secrets at rest
  - Audit secret access

### 8. Deployment Process

- [ ] **Blue-Green Deployment Strategy**
  1. Deploy C implementation to "green" environment
  2. Run smoke tests (health checks, basic API calls)
  3. Gradually shift traffic: 10% → 50% → 100%
  4. Monitor error rates, latency, memory usage
  5. Keep "blue" (Rust) environment running for 24 hours
  6. Rollback procedure: < 2 minutes to switch back

- [ ] **Canary Deployment** (alternative)
  1. Deploy C to 5% of traffic
  2. Monitor for 1 hour
  3. Increase to 25% if healthy
  4. Monitor for 4 hours
  5. Increase to 100% if healthy

- [ ] **Rollback Criteria**
  - Error rate > 1%
  - Response time p99 > 500ms
  - Memory usage > 500 MB per service
  - Any crashes or segfaults
  - Memory leaks detected

### 9. Post-Deployment Monitoring

- [ ] **First 24 Hours - Critical Monitoring**
  - [ ] Check logs every hour for errors
  - [ ] Monitor memory usage trend (should be stable)
  - [ ] Track error rate (should be < 0.1%)
  - [ ] Verify no memory leaks (RSS should plateau)
  - [ ] Check database connection pool health
  - [ ] Monitor Redis memory usage

- [ ] **Week 1 - Daily Checks**
  - [ ] Review Grafana dashboards
  - [ ] Check Prometheus alerts
  - [ ] Analyze slow query logs
  - [ ] Review application logs for warnings
  - [ ] Verify backup success

- [ ] **Week 2-4 - Weekly Checks**
  - [ ] Performance trend analysis
  - [ ] Capacity planning review
  - [ ] Security audit logs
  - [ ] Update runbook with learnings

### 10. Incident Response Plan

- [ ] **On-Call Setup**
  - PagerDuty / Opsgenie integration
  - Escalation policy: L1 (5 min) → L2 (15 min) → L3 (30 min)
  - Runbook: `/docs/runbooks/incident-response.md`

- [ ] **Runbook Sections**
  1. Common Issues & Solutions
  2. How to read logs
  3. How to check metrics
  4. How to restart services
  5. How to rollback to Rust
  6. Emergency contacts

- [ ] **Rollback Procedure** (Practice!)

  ```bash
  # 1. Switch K8s deployment to Rust
  kubectl set image deployment/ethhook-admin-api \
    admin-api=ethhook-rust:latest
  
  # 2. Verify rollback
  kubectl rollout status deployment/ethhook-admin-api
  
  # 3. Check health
  curl https://api.ethhook.com/health
  
  # Expected: < 2 minutes total time
  ```

---

## 🚀 Deployment Commands

✅ **Use existing infrastructure - just switch the implementation flag!**

### Option A: Docker Compose Deployment (Simplest)

```bash
cd /Users/igor/rust_projects/capstone0/ethhook-c/docker

# Build all C services
docker-compose -f docker-compose.prod.yml build

# Start all services
docker-compose -f docker-compose.prod.yml up -d

# Check status
docker-compose -f docker-compose.prod.yml ps

# View logs
docker-compose -f docker-compose.prod.yml logs -f

# Health checks
curl http://localhost:3000/health  # admin-api
docker exec ethhook-processor /app/ethhook-processor --version
```

### Option B: Use Existing GitHub Actions Deployment

```bash
# 1. Update .github/workflows/deploy-digitalocean.yml to use C images
# 2. Configure these secrets (if not already set):
#    - DROPLET_HOST (your server IP)
#    - DROPLET_SSH_KEY (SSH private key)
#    - DROPLET_USER (default: root)

# 3. Trigger deployment
git add -A
git commit -m "Deploy C implementation to production"
git push origin main

# GitHub Actions will automatically:
# - Build Docker images
# - Push to ghcr.io registry
# - Deploy to DigitalOcean droplet
# - Run health checks
```

### Option C: Manual Production Binaries (No Docker)

```bash
cd /Users/igor/rust_projects/capstone0/ethhook-c
mkdir -p build-prod && cd build-prod

# Build for your target architecture
cmake -DCMAKE_BUILD_TYPE=Release \
      -DENABLE_LTO=ON \
      -DCMAKE_C_FLAGS="-O3 -march=x86-64" \
      ..

make -j$(nproc)

# Strip symbols
strip ethhook-admin-api ethhook-ingestor ethhook-processor ethhook-delivery

# Deploy binaries to server
scp ethhook-* user@prod-server:/opt/ethhook/bin/

# Restart services
ssh user@prod-server "systemctl restart ethhook-*"
```

### Switch from Rust to C (Using Existing docker-compose.prod.yml)

```bash
# Edit docker-compose.prod.yml to comment out Rust services and enable C services
cd /Users/igor/rust_projects/capstone0

# Current: Rust services (pipeline, admin-api)
# New: C services (ingestor, processor, delivery, admin-api)

# Update the main docker-compose.prod.yml:
# 1. Comment out 'pipeline' service
# 2. Uncomment C service definitions from ethhook-c/docker/docker-compose.prod.yml
# 3. Or create a docker-compose.override.yml

# Deploy
docker-compose -f docker-compose.prod.yml up -d

# Verify
docker-compose -f docker-compose.prod.yml ps
docker-compose -f docker-compose.prod.yml logs admin-api
```

### Gradual Rollout with Load Balancer

```bash
# If using Nginx/HAProxy/Traefik:
# 1. Run both Rust and C services on different ports
# 2. Configure load balancer weights

# Example nginx config:
upstream ethhook_backend {
    server rust-admin-api:3000 weight=9;  # 90% traffic
    server c-admin-api:3000 weight=1;     # 10% traffic
}

# Gradually adjust weights: 9:1 → 5:5 → 1:9 → 0:10
```

---

## 📊 Success Metrics

### Performance Targets (must meet or exceed)

| Metric | Target | Measurement |
|:-------|-------:|:------------|
| Response Time (p50) | < 50ms | Prometheus histogram |
| Response Time (p95) | < 150ms | Prometheus histogram |
| Response Time (p99) | < 300ms | Prometheus histogram |
| Error Rate | < 0.1% | errors_total / requests_total |
| Memory Usage | < 500 MB | process_resident_memory_bytes |
| CPU Usage | < 70% | container_cpu_usage_seconds_total |
| Throughput | > 1000 req/s | requests_total rate |
| Uptime | > 99.9% | up{job="ethhook"} |

### Comparison with Rust (measure after 1 week)

| Metric | C Implementation | Rust Implementation | Winner |
|:-------|:-----------------|:--------------------|:-------|
| Response Time (p99) | ??? | ??? | TBD |
| Memory Usage | ??? | ??? | TBD |
| Error Rate | ??? | ??? | TBD |
| Throughput | ??? | ??? | TBD |
| Incidents | ??? | ??? | TBD |

---

## 🔍 Monitoring Dashboards

### Grafana Panels (Priority)

1. **Request Rate** (requests/sec over time)
2. **Error Rate** (percentage over time)
3. **Response Time** (p50, p95, p99 over time)
4. **Memory Usage** (RSS in MB over time)
5. **CPU Usage** (percentage over time)
6. **Database Size** (SQLite file size and WAL size over time)
7. **Redis Operations** (commands/sec over time)
8. **Queue Depth** (pending events over time)

### Alerts (PagerDuty Integration)

- **P1 (Critical)**: Service down, error rate > 5%, memory leak detected
- **P2 (High)**: Error rate > 1%, p99 latency > 500ms, memory > 450 MB
- **P3 (Medium)**: Error rate > 0.5%, p99 latency > 300ms, CPU > 80%
- **P4 (Low)**: Warning logs, slow queries, disk space

---

## 📋 Weekly Review Checklist

After 1 week in production, review:

- [ ] **Performance**: Did we meet targets? Any regressions?
- [ ] **Stability**: Any crashes, memory leaks, or deadlocks?
- [ ] **Errors**: What caused errors? Are they fixable?
- [ ] **Resources**: Memory/CPU usage stable? Need tuning?
- [ ] **Incidents**: How many? Average resolution time?
- [ ] **Comparison**: C vs Rust - which performed better?
- [ ] **Decision**: Continue with C, switch to Rust, or run both?

---

## 🎓 Lessons Learned Template

After 1 month, document:

1. **What Went Well**: Wins with C implementation
2. **What Went Wrong**: Issues encountered
3. **Performance Results**: C vs Rust comparison
4. **Team Feedback**: Developer experience, debugging
5. **Cost Analysis**: Compute costs, maintenance costs
6. **Security**: Any vulnerabilities or CVEs?
7. **Final Recommendation**: C, Rust, or hybrid approach?

---

## 🚨 Emergency Rollback (< 2 minutes)

If critical issues arise:

```bash
# 1. Scale down C implementation
kubectl scale deployment/ethhook-c --replicas=0

# 2. Scale up Rust implementation
kubectl scale deployment/ethhook-rust --replicas=3

# 3. Update service to point to Rust
kubectl patch service ethhook -p '{"spec":{"selector":{"implementation":"rust"}}}'

# 4. Verify
kubectl get pods -l app=ethhook
curl https://api.ethhook.com/health

# 5. Notify team
echo "Rolled back to Rust at $(date)" | mail -s "EthHook Rollback" team@example.com
```

---

## 📞 Support & Escalation

- **L1 Support**: DevOps team (monitor alerts, restart services)
- **L2 Support**: Backend engineers (debug logs, tune config)
- **L3 Support**: Senior C developers (memory issues, segfaults)
- **Emergency**: CTO (decide to rollback or continue)

**Escalation Matrix**:
- 0-5 min: L1 investigates
- 5-15 min: L2 joins if unresolved
- 15-30 min: L3 joins if memory/crash issues
- 30-60 min: Consider rollback if business impact
- 60+ min: Mandatory rollback unless explicitly approved

---

## ✅ Go/No-Go Criteria

### GO (Deploy to Production)

- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Load tests show stable performance
- ✅ No memory leaks in Valgrind
- ✅ Static analysis reports zero critical bugs
- ✅ Security audit completed
- ✅ Monitoring dashboards configured
- ✅ Alerts configured and tested
- ✅ Rollback procedure tested
- ✅ Team trained on runbooks
- ✅ Rust backup is production-ready

### NO-GO (Delay Deployment)

- ❌ Any test failures
- ❌ Memory leaks detected
- ❌ Critical bugs in static analysis
- ❌ Security vulnerabilities unresolved
- ❌ Monitoring not ready
- ❌ Rollback procedure not tested
- ❌ Team not trained

---

## 🎯 Success Definition

**After 1 week**: C implementation meets all performance targets with zero critical incidents

**After 1 month**: C implementation shows equal or better reliability than Rust, with acceptable maintenance burden

**Decision Point (1 month)**: 
- **Keep C**: If stable, performant, and team is comfortable
- **Switch to Rust**: If memory bugs, high maintenance, or team prefers safety
- **Hybrid**: Use C for JSON-heavy services, Rust for concurrent services

---

**Status**: Ready for production deployment  
**Next Step**: Execute pre-production checklist and deploy to staging  
**Timeline**: 1 week staging → 1 week production (10%) → 1 week full production → 1 month evaluation
