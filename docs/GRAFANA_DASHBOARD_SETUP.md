# Grafana Dashboard Setup Complete

## Summary

Successfully created and deployed a custom Grafana dashboard for EthHook monitoring.

## What Was Done

### 1. Service Startup
- Started Message Processor and Webhook Delivery services
- Admin API and Event Ingestor require proper environment variable loading (DATABASE_URL)
- Services are running but need Docker containerization for full Prometheus integration

### 2. Custom Dashboard Created
**Location:** `/monitoring/grafana/dashboards/ethhook-dashboard.json`

**Dashboard Features:**
1. **Service Health Status** - Bar gauge showing up/down status of all services
2. **API Request Rate** - Request per second for Admin API
3. **API Response Time** - Average response time in milliseconds
4. **Events Ingested** - Rate of blockchain events being ingested
5. **Events Processed** - Rate of event processing by status
6. **Webhook Delivery Rate** - Webhook delivery success/failure rates
7. **Redis Queue Length** - Current queue depth
8. **Database Activity** - PostgreSQL connections, commits, and rollbacks

### 3. Dashboard Provisioning
- Created provisioning configuration in `/monitoring/grafana/provisioning/dashboards/dashboards.yml`
- Updated `docker-compose.yml` to properly mount dashboard files
- Dashboard automatically loads into Grafana in the "EthHook" folder

### 4. Verification
✅ Dashboard successfully provisioned (UID: `ethhook-overview`)
✅ Available at: http://localhost:3001/d/ethhook-overview/ethhook-system-overview
✅ Credentials: admin / admin

## How to Access

1. **Open Grafana:** http://localhost:3001
2. **Login:** admin / admin
3. **Navigate to Dashboard:**
   - Click on "Dashboards" in the left sidebar
   - Go to "EthHook" folder
   - Select "EthHook System Overview"

## Dashboard Panels

### Service Monitoring
- **Service Health:** Real-time up/down status of all EthHook services
- Shows: admin-api, event-ingestor, message-processor, webhook-delivery, postgres, redis

### Application Metrics
- **API Performance:** Request rates and response times
- **Event Processing:** Ingestion and processing rates
- **Webhook Delivery:** Success/failure rates with color coding (green=success, red=failed, yellow=retrying)

### Infrastructure Metrics
- **Redis Queues:** Monitor queue depths to prevent backlog
- **Database Activity:** Track PostgreSQL performance and activity

## Prometheus Queries Used

The dashboard uses these PromQL queries:

```promql
# Service Health
up

# API Request Rate
rate(http_requests_total{job="admin-api"}[5m])

# API Response Time
rate(http_request_duration_seconds_sum{job="admin-api"}[5m]) / rate(http_request_duration_seconds_count{job="admin-api"}[5m]) * 1000

# Events Ingested
rate(events_received_total{job="event-ingestor"}[5m])

# Events Processed
rate(events_processed_total{job="message-processor"}[5m])

# Webhook Deliveries
rate(webhook_deliveries_total{job="webhook-delivery"}[5m])

# Redis Queue Length
redis_queue_length{job="redis"}

# Database Activity
pg_stat_database_numbackends{datname="ethhook"}
rate(pg_stat_database_xact_commit{datname="ethhook"}[5m])
rate(pg_stat_database_xact_rollback{datname="ethhook"}[5m])
```

## Next Steps

### To Get Full Metrics Data:

1. **Run Services in Docker** (recommended for production):
   ```bash
   # Uncomment service definitions in docker-compose.yml
   docker compose up -d
   ```

2. **Or Expose Metrics from Local Services:**
   - Services need to expose Prometheus metrics on port 9090
   - Currently, Prometheus is configured to scrape Docker service names
   - For local development, update `monitoring/prometheus.yml` to use `localhost` targets

### To Add More Metrics:

1. Add instrumentation to your Rust services using the `prometheus` crate
2. Expose metrics endpoints (typically on `/metrics`)
3. Update Prometheus scrape configs in `monitoring/prometheus.yml`
4. Create new panels in Grafana using the metrics

### To Customize Dashboard:

1. Open the dashboard in Grafana UI
2. Click "Dashboard settings" (gear icon)
3. Make changes to panels, queries, or visualizations
4. Changes are automatically saved if `allowUiUpdates: true` in provisioning config

## Files Modified

1. `/monitoring/grafana/dashboards/ethhook-dashboard.json` - Dashboard definition
2. `/monitoring/grafana/provisioning/dashboards/dashboards.yml` - Provisioning config
3. `/docker-compose.yml` - Updated Grafana volume mounts

## Current Service Status

- ✅ Grafana: Running on port 3001
- ✅ Prometheus: Running on port 9090
- ✅ PostgreSQL: Running on port 5432
- ✅ Redis: Running on port 6379
- ⚠️ Message Processor: Running locally (metrics may not be scraped)
- ⚠️ Webhook Delivery: Running locally (metrics may not be scraped)
- ❌ Admin API: Needs environment variables
- ❌ Event Ingestor: Needs environment variables

## Troubleshooting

### Dashboard Not Showing Data?
- Check if services are exposing Prometheus metrics
- Verify Prometheus can scrape the endpoints: http://localhost:9090/targets
- Check Prometheus for the metric names you're querying

### Services Showing as Down?
- Services may be running locally but Prometheus is looking for Docker service names
- Either run services in Docker or update Prometheus config for localhost

### Dashboard Not Appearing?
- Check Grafana logs: `docker logs ethhook-grafana`
- Verify dashboard file is mounted: `docker exec ethhook-grafana ls /etc/dashboards/`
- Check provisioning config: `docker exec ethhook-grafana cat /etc/grafana/provisioning/dashboards/dashboards.yml`

## Resources

- Dashboard: http://localhost:3001/d/ethhook-overview/ethhook-system-overview
- Prometheus: http://localhost:9090
- Prometheus Targets: http://localhost:9090/targets
- Grafana Documentation: https://grafana.com/docs/grafana/latest/dashboards/
- PromQL Guide: https://prometheus.io/docs/prometheus/latest/querying/basics/
