# Deployment URL Test Plan - Fix #30 CORS Implementation

## Overview
This document provides a comprehensive test plan to verify all services are accessible and functioning correctly after deploying the CORS fix (Fix #30) to production at **104.248.15.178**.

## Deployment Status
- **Fix #30**: CORS implementation - DEPLOYED
- **Commit**: 3a419f0
- **Changes**: Added CORS headers to all C admin API responses
- **Expected Result**: Browser should successfully communicate with API from UI

---

## Test Plan Checklist

### 1. Core Services Health Checks

#### 1.1 Admin API (C Implementation) - Port 3000
```bash
# Health check
curl -v http://104.248.15.178:3000/health

# Expected Response:
# HTTP/1.1 200 OK
# Content-Type: application/json
# Access-Control-Allow-Origin: *
# Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
# Access-Control-Allow-Headers: Content-Type, Authorization
# Access-Control-Max-Age: 86400
# {"status":"ok"}

# CORS Preflight (OPTIONS)
curl -X OPTIONS http://104.248.15.178:3000/api/v1/auth/login \
  -H "Origin: http://104.248.15.178:3002" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type" \
  -v

# Expected Response:
# HTTP/1.1 204 No Content
# Access-Control-Allow-Origin: *
# Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
# Access-Control-Allow-Headers: Content-Type, Authorization
```

**Result**: [ ] PASS / [ ] FAIL

#### 1.2 UI (Next.js) - Port 3002
```bash
# Homepage
curl -I http://104.248.15.178:3002

# Expected Response:
# HTTP/1.1 200 OK
# Content-Type: text/html
```

**Result**: [ ] PASS / [ ] FAIL

#### 1.3 Pipeline (Rust) - Ports 8080, 9090
```bash
# Health endpoint
curl -v http://104.248.15.178:8080/health

# Expected Response:
# HTTP/1.1 200 OK
# {"status":"healthy"}

# Metrics endpoint
curl -v http://104.248.15.178:9090/metrics

# Expected Response:
# HTTP/1.1 200 OK
# (Prometheus format metrics)
```

**Result**: [ ] PASS / [ ] FAIL

#### 1.4 ClickHouse - Ports 8123, 9000
```bash
# HTTP interface ping
curl -v http://104.248.15.178:8123/ping

# Expected Response:
# HTTP/1.1 200 OK
# Ok.

# Query test
curl 'http://104.248.15.178:8123/?query=SELECT%201'

# Expected Response:
# 1
```

**Result**: [ ] PASS / [ ] FAIL

#### 1.5 Prometheus - Port 9092
```bash
# Ready check
curl -v http://104.248.15.178:9092/-/ready

# Expected Response:
# HTTP/1.1 200 OK
# Prometheus is Ready.

# Targets check
curl -s http://104.248.15.178:9092/api/v1/targets | jq '.data.activeTargets[] | {job: .labels.job, health: .health}'

# Expected Response:
# {
#   "job": "ethhook-pipeline",
#   "health": "up"
# }
```

**Result**: [ ] PASS / [ ] FAIL

#### 1.6 Grafana - Port 3001
```bash
# Homepage
curl -I http://104.248.15.178:3001

# Expected Response:
# HTTP/1.1 200 OK
# (Grafana login page)

# Health API
curl -v http://104.248.15.178:3001/api/health

# Expected Response:
# HTTP/1.1 200 OK
# {"database":"ok","version":"..."}
```

**Result**: [ ] PASS / [ ] FAIL

#### 1.7 Demo Webhook Receiver - Port 8000
```bash
# Health check
curl -v http://104.248.15.178:8000/health

# Expected Response:
# HTTP/1.1 200 OK
# {"status":"ok"}
```

**Result**: [ ] PASS / [ ] FAIL

---

### 2. End-to-End Login Flow Test (CRITICAL - Fix #30)

#### 2.1 Browser Test (Primary Validation)
1. **Open browser**: http://104.248.15.178:3002
2. **Open DevTools**: F12 → Network tab → Preserve log
3. **Navigate to**: Login page
4. **Enter credentials**:
   - Email: `demo@ethhook.com`
   - Password: `demo123`
5. **Click "Login"**

**Expected Behavior**:
- ✅ Browser sends OPTIONS preflight request to `http://104.248.15.178:3000/api/v1/auth/login`
- ✅ Server responds with 204 No Content + CORS headers
- ✅ Browser sends POST request with credentials
- ✅ Server responds with 200 OK + JWT token + CORS headers
- ✅ Browser stores token and redirects to dashboard
- ❌ NO "CORS policy" errors in console
- ❌ NO "Failed to fetch" errors

**DevTools Network Tab Verification**:
```
OPTIONS /api/v1/auth/login → 204 No Content
Headers:
  Access-Control-Allow-Origin: *
  Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
  Access-Control-Allow-Headers: Content-Type, Authorization

POST /api/v1/auth/login → 200 OK
Headers:
  Access-Control-Allow-Origin: *
  Content-Type: application/json
Response:
  {"token":"..."}
```

**Result**: [ ] PASS / [ ] FAIL

#### 2.2 cURL Test (Secondary Validation)
```bash
# Simulate browser CORS preflight
curl -X OPTIONS http://104.248.15.178:3000/api/v1/auth/login \
  -H "Origin: http://104.248.15.178:3002" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type" \
  -i

# Expected: 204 No Content with CORS headers

# Simulate login POST (after preflight succeeds)
curl -X POST http://104.248.15.178:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -H "Origin: http://104.248.15.178:3002" \
  -d '{"email":"demo@ethhook.com","password":"demo123"}' \
  -i

# Expected: 200 OK with JWT token and CORS headers
```

**Result**: [ ] PASS / [ ] FAIL

---

### 3. Dashboard API Endpoints (Post-Login)

After successful login, test dashboard API calls:

#### 3.1 Users Endpoint
```bash
# Browser: Navigate to Users page
# OR cURL with JWT token:
TOKEN="<jwt-from-login>"
curl -H "Authorization: Bearer $TOKEN" \
     -H "Origin: http://104.248.15.178:3002" \
     http://104.248.15.178:3000/api/v1/users
```

**Result**: [ ] PASS / [ ] FAIL

#### 3.2 Applications Endpoint
```bash
curl -H "Authorization: Bearer $TOKEN" \
     -H "Origin: http://104.248.15.178:3002" \
     http://104.248.15.178:3000/api/v1/applications
```

**Result**: [ ] PASS / [ ] FAIL

#### 3.3 Events Endpoint
```bash
curl -H "Authorization: Bearer $TOKEN" \
     -H "Origin: http://104.248.15.178:3002" \
     http://104.248.15.178:3000/api/v1/events
```

**Result**: [ ] PASS / [ ] FAIL

#### 3.4 Deliveries Endpoint
```bash
curl -H "Authorization: Bearer $TOKEN" \
     -H "Origin: http://104.248.15.178:3002" \
     http://104.248.15.178:3000/api/v1/deliveries
```

**Result**: [ ] PASS / [ ] FAIL

---

### 4. Firewall Rules Verification

```bash
# SSH into server
ssh root@104.248.15.178

# Check UFW status
sudo ufw status numbered

# Expected Output:
# Status: active
# To                         Action      From
# --                         ------      ----
# [ 1] 22/tcp                 ALLOW IN    Anywhere
# [ 2] 80/tcp                 ALLOW IN    Anywhere
# [ 3] 443/tcp                ALLOW IN    Anywhere
# [ 4] 3000/tcp               ALLOW IN    Anywhere  # Admin API
# [ 5] 3001/tcp               ALLOW IN    Anywhere  # Grafana
# [ 6] 3002/tcp               ALLOW IN    Anywhere  # UI
# [ 7] 8000/tcp               ALLOW IN    Anywhere  # Demo Receiver
# [ 8] 8080/tcp               ALLOW IN    Anywhere  # Pipeline Health
# [ 9] 8123/tcp               ALLOW IN    Anywhere  # ClickHouse HTTP
# [10] 9000/tcp               ALLOW IN    Anywhere  # ClickHouse Native
# [11] 9090/tcp               ALLOW IN    Anywhere  # Pipeline Metrics
# [12] 9092/tcp               ALLOW IN    Anywhere  # Prometheus
```

**Result**: [ ] PASS / [ ] FAIL

---

### 5. Container Status Verification

```bash
# SSH into server
ssh root@104.248.15.178

# Check all containers running
cd ~/ethhook
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

# Expected Output (all "Up"):
# NAMES                    STATUS              PORTS
# ethhook-admin-api        Up X minutes        0.0.0.0:3000->3000/tcp
# ethhook-ui               Up X minutes        0.0.0.0:3002->3000/tcp
# ethhook-pipeline         Up X minutes        0.0.0.0:8080->8080/tcp, ...
# ethhook-clickhouse       Up X minutes        0.0.0.0:8123->8123/tcp, ...
# ethhook-prometheus       Up X minutes        0.0.0.0:9092->9090/tcp
# ethhook-grafana          Up X minutes        0.0.0.0:3001->3000/tcp
# ethhook-demo-receiver    Up X minutes        0.0.0.0:8000->8000/tcp

# Check admin API logs for CORS requests
docker logs ethhook-admin-api --tail 50

# Expected: No errors, OPTIONS requests logged, CORS headers added
```

**Result**: [ ] PASS / [ ] FAIL

---

### 6. Database Verification

```bash
# SSH into server
ssh root@104.248.15.178

# Check SQLite database
docker exec ethhook-admin-api sqlite3 /data/config.db "SELECT email, is_admin FROM users;"

# Expected Output:
# demo@ethhook.com|0
# admin@ethhook.io|1

# Check database size (should be small)
docker exec ethhook-admin-api ls -lh /data/config.db
# Expected: < 100KB
```

**Result**: [ ] PASS / [ ] FAIL

---

### 7. Monitoring Stack Validation

#### 7.1 Grafana Access
1. Open browser: http://104.248.15.178:3001
2. Login: admin / admin (or GRAFANA_PASSWORD from .env)
3. Navigate to: Connections → Data Sources
4. Verify: Prometheus data source shows "Data source is working"

**Result**: [ ] PASS / [ ] FAIL

#### 7.2 Prometheus Targets
1. Open browser: http://104.248.15.178:9092
2. Navigate to: Status → Targets
3. Verify: ethhook-pipeline target shows "UP" status
4. Verify: Last scrape < 30 seconds ago

**Result**: [ ] PASS / [ ] FAIL

#### 7.3 Metrics Collection
```bash
# Query Prometheus for pipeline metrics
curl -s 'http://104.248.15.178:9092/api/v1/query?query=up{job="ethhook-pipeline"}' | jq

# Expected:
# {
#   "status": "success",
#   "data": {
#     "result": [
#       {
#         "metric": {"job": "ethhook-pipeline"},
#         "value": [<timestamp>, "1"]
#       }
#     ]
#   }
# }
```

**Result**: [ ] PASS / [ ] FAIL

---

## Success Criteria Summary

### Critical (Must Pass):
- ✅ Admin API health check returns 200 with CORS headers
- ✅ OPTIONS preflight requests return 204 with CORS headers
- ✅ Browser login succeeds without CORS errors
- ✅ JWT token received and stored
- ✅ Dashboard loads after login
- ✅ All 7 services accessible from external network

### Important (Should Pass):
- ✅ All firewall rules correctly configured
- ✅ All containers running and healthy
- ✅ Database contains demo users
- ✅ Grafana shows Prometheus data source working
- ✅ Prometheus targets show "UP" status

### Nice to Have (Can Debug Later):
- Dashboard API calls return data (may be empty)
- ClickHouse queries return results
- Metrics graphs show data in Grafana

---

## Troubleshooting Guide

### Issue: CORS errors still appear
**Diagnosis**:
```bash
# Check if new image was built
ssh root@104.248.15.178 "cd ~/ethhook && docker logs ethhook-admin-api | grep CORS"

# Check image build date
ssh root@104.248.15.178 "cd ~/ethhook && docker inspect ethhook-admin-api | jq '.[0].Created'"
```

**Resolution**:
1. Verify deployment workflow completed successfully (check GitHub Actions)
2. Rebuild image: `ssh root@104.248.15.178 "cd ~/ethhook && docker-compose down && docker-compose up -d --build"`
3. Clear browser cache (Ctrl+Shift+R)

### Issue: Firewall blocking ports
**Diagnosis**:
```bash
ssh root@104.248.15.178 "sudo ufw status verbose"
```

**Resolution**:
```bash
# Re-run firewall configuration step from deployment workflow
ssh root@104.248.15.178 << 'EOF'
sudo ufw allow 3000/tcp comment 'Admin API'
sudo ufw allow 3001/tcp comment 'Grafana'
sudo ufw allow 3002/tcp comment 'UI'
sudo ufw allow 8000/tcp comment 'Demo Receiver'
sudo ufw allow 8080/tcp comment 'Pipeline Health'
sudo ufw allow 8123/tcp comment 'ClickHouse HTTP'
sudo ufw allow 9000/tcp comment 'ClickHouse Native'
sudo ufw allow 9090/tcp comment 'Pipeline Metrics'
sudo ufw allow 9092/tcp comment 'Prometheus'
sudo ufw reload
EOF
```

### Issue: Containers not running
**Diagnosis**:
```bash
ssh root@104.248.15.178 "cd ~/ethhook && docker ps -a"
```

**Resolution**:
```bash
ssh root@104.248.15.178 "cd ~/ethhook && docker-compose logs --tail 50"
ssh root@104.248.15.178 "cd ~/ethhook && docker-compose restart"
```

---

## Post-Validation Steps

After all tests pass:

1. **Performance Baseline**: Run basic load test
   ```bash
   ab -n 1000 -c 10 http://104.248.15.178:3000/health
   ```

2. **Memory Check**: Monitor for 10 minutes
   ```bash
   ssh root@104.248.15.178 "docker stats --no-stream"
   ```

3. **Log Review**: Check for errors
   ```bash
   ssh root@104.248.15.178 "cd ~/ethhook && docker-compose logs --tail 200 | grep -i error"
   ```

4. **Document Results**: Update this file with actual test results

5. **24-Hour Monitoring**: Schedule checks tomorrow to verify stability

---

## Test Execution Log

### Execution Date: _______________
### Tester: _______________

**Overall Result**: [ ] ALL PASS / [ ] SOME FAILURES

**Notes**:
- 
- 
- 

**Issues Found**:
1. 
2. 
3. 

**Next Steps**:
- 
- 
- 

---

## Appendix: Quick Test Script

Save as `test_deployment.sh`:

```bash
#!/bin/bash

SERVER="104.248.15.178"
PASS=0
FAIL=0

echo "=== EthHook Deployment Test ==="
echo ""

# Test function
test_endpoint() {
    local name=$1
    local url=$2
    local expected=$3
    
    echo -n "Testing $name... "
    response=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null)
    
    if [ "$response" = "$expected" ]; then
        echo "✅ PASS (HTTP $response)"
        ((PASS++))
    else
        echo "❌ FAIL (HTTP $response, expected $expected)"
        ((FAIL++))
    fi
}

# Run tests
test_endpoint "Admin API Health" "http://$SERVER:3000/health" "200"
test_endpoint "UI Homepage" "http://$SERVER:3002" "200"
test_endpoint "Pipeline Health" "http://$SERVER:8080/health" "200"
test_endpoint "ClickHouse Ping" "http://$SERVER:8123/ping" "200"
test_endpoint "Prometheus Ready" "http://$SERVER:9092/-/ready" "200"
test_endpoint "Grafana Homepage" "http://$SERVER:3001" "200"
test_endpoint "Demo Receiver" "http://$SERVER:8000/health" "200"

# CORS preflight test
echo -n "Testing CORS Preflight... "
cors_response=$(curl -s -o /dev/null -w "%{http_code}" -X OPTIONS \
    -H "Origin: http://$SERVER:3002" \
    -H "Access-Control-Request-Method: POST" \
    "http://$SERVER:3000/api/v1/auth/login" 2>/dev/null)

if [ "$cors_response" = "204" ]; then
    echo "✅ PASS (HTTP 204)"
    ((PASS++))
else
    echo "❌ FAIL (HTTP $cors_response, expected 204)"
    ((FAIL++))
fi

echo ""
echo "=== Summary ==="
echo "✅ PASS: $PASS"
echo "❌ FAIL: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed. Review output above."
    exit 1
fi
```

Run with:
```bash
chmod +x test_deployment.sh
./test_deployment.sh
```

---

**End of Test Plan**
