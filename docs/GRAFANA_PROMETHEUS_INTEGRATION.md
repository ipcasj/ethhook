# Grafana and Prometheus Integration Guide

## Overview

EthHook uses Prometheus for metrics collection and Grafana for visualization. Both services run in Docker containers and are automatically configured to work together.

## Access Information

### Grafana
- **URL:** http://localhost:3001
- **Username:** `admin`
- **Password:** `admin`

### Prometheus
- **URL:** http://localhost:9090
- **Health Check:** http://localhost:9090/-/healthy

## Architecture

```
┌──────────────────┐
│  Grafana:3001    │
│  (Visualization) │
└────────┬─────────┘
         │
         │ http://prometheus:9090
         │
┌────────▼─────────┐
│ Prometheus:9090  │
│  (Metrics DB)    │
└────────┬─────────┘
         │
         │ scrapes metrics from
         │
┌────────▼─────────┐
│  EthHook Services│
│  (metrics port)  │
└──────────────────┘
```

## Configuration

### Prometheus Datasource

The Prometheus datasource is automatically configured via provisioning files:

**File:** `monitoring/grafana/provisioning/datasources/prometheus.yml`

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090  # Uses Docker service name
    isDefault: true
    editable: true
    jsonData:
      timeInterval: 5s
```

**Key Points:**
- Uses `http://prometheus:9090` (Docker service name) instead of `localhost:9090`
- Docker networking allows containers to communicate using service names
- `access: proxy` means Grafana backend queries Prometheus (recommended)

### Docker Compose Configuration

**File:** `docker-compose.yml`

```yaml
services:
  prometheus:
    image: prom/prometheus:latest
    container_name: ethhook-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/grafana/provisioning/prometheus.yml
      - prometheus_data:/prometheus

  grafana:
    image: grafana/grafana:latest
    container_name: ethhook-grafana
    ports:
      - "3001:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: admin
      GF_USERS_ALLOW_SIGN_UP: false
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/provisioning:/etc/grafana/provisioning
    depends_on:
      - prometheus
```

## Verifying the Integration

### 1. Check Prometheus Health

```bash
curl http://localhost:9090/-/healthy
# Expected output: Prometheus Server is Healthy.
```

### 2. Check Grafana Datasource

1. Open Grafana: http://localhost:3001
2. Login with `admin` / `admin`
3. Go to: Configuration → Data Sources
4. You should see "Prometheus" datasource listed
5. Click on it and scroll down
6. Click "Test" button
7. Should see green "Data source is working"

### 3. Check Logs

```bash
# Check if Grafana loaded the datasource
docker logs ethhook-grafana 2>&1 | grep "inserting datasource from configuration"

# Should see something like:
# level=info msg="inserting datasource from configuration" name=Prometheus uid=...
```

## Troubleshooting

### Error: "dial tcp [::1]:9090: connect: connection refused"

**Cause:** Grafana is trying to connect to `localhost:9090` instead of `prometheus:9090`

**Solution:** 
1. Ensure the datasource URL is `http://prometheus:9090` (Docker service name)
2. Restart Grafana: `docker compose restart grafana`

### Datasource Not Showing

**Cause:** Provisioning files not mounted correctly

**Solution:**
1. Check if files exist:
   ```bash
   docker exec ethhook-grafana ls -la /etc/grafana/provisioning/datasources/
   ```
2. Should see `prometheus.yml` file
3. If not, restart Docker compose:
   ```bash
   docker compose down
   docker compose up -d
   ```

### Can't Access Grafana

**Cause:** Port 3001 might be in use

**Solution:**
1. Check what's using port 3001:
   ```bash
   lsof -i :3001
   ```
2. Either stop the conflicting service or change Grafana's port in `docker-compose.yml`

## Creating Dashboards

### Option 1: Import Pre-built Dashboard

1. In Grafana, go to: Dashboards → Import
2. Enter dashboard ID or upload JSON file
3. Select "Prometheus" as the datasource
4. Click "Import"

### Option 2: Create Custom Dashboard

1. In Grafana, go to: Dashboards → New Dashboard
2. Click "Add visualization"
3. Select "Prometheus" as datasource
4. Use PromQL queries, for example:
   ```promql
   # Request rate
   rate(http_requests_total[5m])
   
   # Memory usage
   process_resident_memory_bytes
   
   # Custom metrics from EthHook
   ethhook_events_processed_total
   ethhook_webhook_deliveries_total
   ```

## Useful PromQL Queries for EthHook

```promql
# Total events processed
sum(ethhook_events_processed_total)

# Events processed per second
rate(ethhook_events_processed_total[1m])

# Webhook delivery success rate
sum(rate(ethhook_webhook_deliveries_success_total[5m])) 
/ 
sum(rate(ethhook_webhook_deliveries_total[5m]))

# Average delivery latency
histogram_quantile(0.95, 
  rate(ethhook_webhook_delivery_duration_seconds_bucket[5m])
)
```

## Maintenance

### Restart Services

```bash
# Restart both services
docker compose restart prometheus grafana

# Restart only Grafana
docker compose restart grafana
```

### View Logs

```bash
# Prometheus logs
docker logs ethhook-prometheus

# Grafana logs
docker logs ethhook-grafana

# Follow logs in real-time
docker logs -f ethhook-grafana
```

### Backup Data

```bash
# Backup Prometheus data
docker cp ethhook-prometheus:/prometheus ./backup/prometheus-data

# Backup Grafana data
docker cp ethhook-grafana:/var/lib/grafana ./backup/grafana-data
```

## Next Steps

1. **Add Metrics to Your Services**: Instrument your Rust services with Prometheus metrics
2. **Configure Prometheus Scraping**: Update `monitoring/prometheus.yml` to scrape your services
3. **Create Dashboards**: Build custom dashboards for EthHook metrics
4. **Set Up Alerts**: Configure alerting rules in Prometheus for critical metrics

## Additional Resources

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/grafana/latest/)
- [PromQL Tutorial](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Grafana Dashboards Library](https://grafana.com/grafana/dashboards/)
