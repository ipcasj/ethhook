# Phase 2 Analytics - Specification & Implementation Plan

**Version:** 1.0  
**Date:** November 9, 2025  
**Status:** 📋 Planning  
**Prerequisites:** ✅ Phase 1 Complete

---

## 🎯 Overview

Phase 2 expands analytics capabilities with per-application and per-endpoint insights, advanced filtering, real-time updates, and data export features. This phase transforms the dashboard from overview-only to drill-down analytics.

---

## 🎨 Feature Prioritization

### Priority 1: Core Analytics Pages (MUST HAVE)
- Applications Analytics Page
- Endpoints Analytics Page
- Individual application/endpoint detail views

### Priority 2: Enhanced Filtering (SHOULD HAVE)
- Custom date range picker
- Chain/contract filters
- Status filters (success/failed)
- Performance range filters (latency)

### Priority 3: Real-Time Features (NICE TO HAVE)
- WebSocket integration for live updates
- Live event feed
- Real-time chart updates

### Priority 4: Export & Reporting (NICE TO HAVE)
- CSV/JSON data export
- PDF report generation
- Scheduled email reports

---

## 📊 Feature 1: Applications Analytics Page

### Purpose
Provide detailed analytics for each application, allowing users to compare performance across different projects/environments.

### Location
`/dashboard/applications` (enhance existing page)

### UI Components

**1. Applications Overview Card**
- Table/grid of all applications
- Quick stats per application:
  - Total events
  - Active endpoints count
  - Success rate
  - Avg response time
- Sortable columns
- Click to drill into application details

**2. Application Comparison Chart**
- Bar chart comparing applications
- Metrics: events, success rate, avg latency
- Time range selector (24h/7d/30d)
- Hover tooltips with details

**3. Application Detail View** (new route: `/dashboard/applications/[id]`)
- Application header with name, description, created date
- Dedicated metrics section:
  - Total events for this app
  - Endpoints count
  - Success rate
  - Failed deliveries
  - Avg/min/max latency
- Charts:
  - Events over time (area chart)
  - Success rate trend (line chart)
  - Endpoint performance comparison (bar chart)
  - Event distribution by endpoint (pie chart)
- Recent events table filtered by application
- Endpoint list with quick actions

### Backend Requirements

**New/Enhanced Endpoints:**

```
GET /api/v1/applications/{id}/statistics
Response: {
  application_id: uuid,
  events_total: number,
  events_24h: number,
  endpoints_count: number,
  active_endpoints: number,
  total_deliveries: number,
  successful_deliveries: number,
  failed_deliveries: number,
  success_rate: number,
  avg_delivery_time_ms: number,
  min_delivery_time_ms: number,
  max_delivery_time_ms: number,
  first_event_at: timestamp,
  last_event_at: timestamp
}

GET /api/v1/applications/{id}/timeseries?time_range={range}&granularity={unit}
Response: TimeseriesResponse (same structure as Phase 1)

GET /api/v1/applications/{id}/endpoints/performance
Response: {
  endpoints: [{
    endpoint_id: uuid,
    name: string,
    url: string,
    events_count: number,
    success_rate: number,
    avg_latency_ms: number,
    last_event_at: timestamp
  }]
}
```

### Implementation Tasks

**Backend (Rust):**
- [ ] Create `applications/{id}/statistics` handler
- [ ] Create `applications/{id}/timeseries` handler
- [ ] Create `applications/{id}/endpoints/performance` handler
- [ ] Add SQL queries with user_id filtering
- [ ] Add caching for expensive queries (consider Redis)
- [ ] Add tests for new endpoints

**Frontend (TypeScript/React):**
- [ ] Create ApplicationAnalytics component
- [ ] Create ApplicationDetailPage component
- [ ] Add comparison bar chart component
- [ ] Add endpoint performance table
- [ ] Implement sorting and filtering
- [ ] Add loading states and error handling
- [ ] Add navigation breadcrumbs
- [ ] Add tests for new components

**Estimated Effort:** 3-4 days

---

## 🔌 Feature 2: Endpoints Analytics Page

### Purpose
Monitor individual endpoint performance, identify slow/failing endpoints, and debug delivery issues.

### Location
`/dashboard/endpoints` (enhance existing page)

### UI Components

**1. Endpoints Performance Table**
- List all endpoints with stats:
  - Name & URL
  - Status (active/inactive)
  - Events received (24h/total)
  - Success rate
  - Avg response time
  - Last event timestamp
- Sortable by any column
- Search/filter by name
- Status indicators (green/yellow/red)
- Click row to see details

**2. Performance Distribution Chart**
- Scatter plot: success rate vs latency
- Bubble size = event count
- Color = status (green/yellow/red)
- Hover shows endpoint name
- Click to drill down

**3. Endpoint Detail View** (new route: `/dashboard/endpoints/[id]`)
- Endpoint header:
  - Name, URL, status badge
  - Edit/delete actions
  - Test webhook button
- Metrics cards:
  - Total events
  - Success rate
  - Avg/min/max latency
  - Last event time
  - Error rate
- Charts:
  - Response time over time (line chart)
  - Success/failure distribution (pie chart)
  - Hourly traffic pattern (heatmap)
- Recent deliveries table:
  - Timestamp, status, latency, status code
  - Response body preview
  - Retry information
  - Expandable error details
- Configuration section:
  - Webhook URL
  - Retry settings
  - Filter rules

**4. Health Score Card**
- Calculated metric (0-100)
- Based on: success rate, latency, uptime
- Visual indicator (progress bar)
- Breakdown of score components

### Backend Requirements

**New/Enhanced Endpoints:**

```
GET /api/v1/endpoints/{id}/statistics
Response: {
  endpoint_id: uuid,
  name: string,
  url: string,
  status: string,
  events_total: number,
  events_24h: number,
  deliveries_total: number,
  successful_deliveries: number,
  failed_deliveries: number,
  success_rate: number,
  avg_delivery_time_ms: number,
  min_delivery_time_ms: number,
  max_delivery_time_ms: number,
  p50_latency_ms: number,
  p95_latency_ms: number,
  p99_latency_ms: number,
  first_event_at: timestamp,
  last_event_at: timestamp,
  health_score: number
}

GET /api/v1/endpoints/{id}/timeseries?time_range={range}&granularity={unit}
Response: TimeseriesResponse

GET /api/v1/endpoints/{id}/deliveries?limit={n}&offset={n}&status={filter}
Response: {
  deliveries: [{
    id: uuid,
    event_id: uuid,
    attempt_number: number,
    http_status_code: number,
    success: boolean,
    duration_ms: number,
    attempted_at: timestamp,
    error_message: string | null,
    response_body: string | null
  }],
  total: number,
  limit: number,
  offset: number
}

GET /api/v1/endpoints/performance-summary
Response: {
  endpoints: [{
    endpoint_id: uuid,
    name: string,
    url: string,
    success_rate: number,
    avg_latency_ms: number,
    event_count: number,
    status: string
  }]
}
```

### Implementation Tasks

**Backend (Rust):**
- [ ] Create `endpoints/{id}/statistics` handler with health score calculation
- [ ] Create `endpoints/{id}/timeseries` handler
- [ ] Create `endpoints/{id}/deliveries` handler with pagination
- [ ] Create `endpoints/performance-summary` handler
- [ ] Add percentile calculations (p50, p95, p99)
- [ ] Add delivery filtering by status
- [ ] Add tests for new endpoints

**Frontend (TypeScript/React):**
- [ ] Create EndpointAnalytics component
- [ ] Create EndpointDetailPage component
- [ ] Add performance scatter plot
- [ ] Add delivery history table with expandable rows
- [ ] Add health score component
- [ ] Implement test webhook button
- [ ] Add response body preview modal
- [ ] Add tests for new components

**Estimated Effort:** 4-5 days

---

## 🔍 Feature 3: Advanced Filtering

### Purpose
Allow users to narrow down analytics to specific time periods, chains, contracts, and status conditions.

### UI Components

**1. Filter Bar Component** (reusable across all analytics pages)
- Date range picker:
  - Quick presets (Today, Yesterday, Last 7d, Last 30d, Custom)
  - Calendar widget for custom range
  - Time zone selector
- Chain selector:
  - Multi-select dropdown
  - Options: Ethereum, Arbitrum, Optimism, Base, All
  - Shows event count per chain
- Contract address filter:
  - Input field with validation
  - Recent contracts dropdown
  - Clear button
- Status filter:
  - All, Success only, Failed only
  - Error type filter (timeout, 4xx, 5xx)
- Performance filter:
  - Latency range slider (0ms - 5000ms)
  - Success rate threshold
- Apply/Reset buttons
- Save filter preset (for power users)

**2. Active Filters Display**
- Chip/badge for each active filter
- Click to remove individual filter
- Clear all button
- Count of results matching filters

**3. Filter Persistence**
- Save filters in URL query params
- Restore filters on page reload
- Browser back/forward support

### Backend Requirements

**Enhanced Endpoints:**

```
All existing endpoints accept new query parameters:

?start_date={iso8601}
&end_date={iso8601}
&chain_ids[]={id1}&chain_ids[]={id2}
&contract_address={0x...}
&status={all|success|failed}
&min_latency={ms}
&max_latency={ms}
&min_success_rate={percent}

Example:
GET /api/v1/statistics/timeseries
  ?time_range=custom
  &start_date=2025-11-01T00:00:00Z
  &end_date=2025-11-09T23:59:59Z
  &chain_ids[]=1
  &chain_ids[]=42161
  &status=failed
  &min_latency=1000
```

### Implementation Tasks

**Backend (Rust):**
- [ ] Add filter query parameters to all statistics handlers
- [ ] Implement dynamic SQL WHERE clause building
- [ ] Add chain_id filtering support (requires DB schema update?)
- [ ] Add contract_address filtering
- [ ] Add status filtering
- [ ] Add latency range filtering
- [ ] Add input validation for all filters
- [ ] Add tests for filtering logic

**Frontend (TypeScript/React):**
- [ ] Create FilterBar component
- [ ] Create DateRangePicker component
- [ ] Create ChainSelector component
- [ ] Create ActiveFilters display component
- [ ] Implement URL state management
- [ ] Add filter persistence in localStorage
- [ ] Add filter preset management
- [ ] Update all pages to use FilterBar
- [ ] Add tests for filter components

**Database:**
- [ ] Assess if chain_id column needed in events table
- [ ] Add contract_address index if not present
- [ ] Consider materialized views for common filters

**Estimated Effort:** 5-6 days

---

## ⚡ Feature 4: Real-Time Updates

### Purpose
Provide live dashboard updates without manual refresh, enhancing monitoring experience for active systems.

### Approach

**Option A: WebSocket (Preferred)**
- Server pushes updates to connected clients
- Lower latency, more efficient
- Requires WebSocket infrastructure

**Option B: Server-Sent Events (SSE)**
- Simpler than WebSocket
- One-way server→client
- Good for read-only updates

**Option C: Aggressive Polling**
- Reduce refresh interval to 5-10s
- No infrastructure changes
- Higher server load

### WebSocket Implementation Plan

**1. Backend WebSocket Server**
- Add `axum::extract::ws` support
- Create `/ws/events` and `/ws/stats` endpoints
- Authentication via JWT in connection handshake
- Broadcast events to subscribed clients
- Send stat updates on change

**2. Event Types**
```rust
enum WebSocketMessage {
    NewEvent {
        event_id: uuid,
        contract_address: String,
        block_number: i64,
        timestamp: DateTime<Utc>
    },
    StatUpdate {
        metric: String,
        value: f64,
        timestamp: DateTime<Utc>
    },
    DeliveryUpdate {
        event_id: uuid,
        endpoint_id: uuid,
        status: String,
        latency_ms: i32
    }
}
```

**3. Frontend WebSocket Client**
- React hook: `useWebSocket()`
- Auto-reconnect on disconnect
- Handle connection states (connecting, connected, disconnected)
- Update React Query cache on message
- Visual indicator of connection status

**4. Live Components**
- Live event feed (top of dashboard)
- Real-time stat counters (animated)
- Live chart updates (add point without full refresh)
- Notification badge for new events

### Implementation Tasks

**Backend (Rust):**
- [ ] Add WebSocket dependencies (tokio-tungstenite)
- [ ] Create WebSocket connection handler
- [ ] Implement JWT authentication for WS
- [ ] Create broadcast channel for events
- [ ] Add message serialization
- [ ] Implement connection management
- [ ] Add rate limiting
- [ ] Add tests for WebSocket handlers

**Frontend (TypeScript/React):**
- [ ] Create useWebSocket hook
- [ ] Implement auto-reconnect logic
- [ ] Add connection status indicator
- [ ] Create LiveEventFeed component
- [ ] Update charts to accept live data
- [ ] Add notification system
- [ ] Add tests for WebSocket integration

**Infrastructure:**
- [ ] Configure nginx for WebSocket proxy
- [ ] Add WebSocket health check
- [ ] Monitor connection count

**Estimated Effort:** 6-7 days

---

## 📤 Feature 5: Data Export & Reporting

### Purpose
Enable users to extract analytics data for external analysis, reporting, and compliance.

### Export Formats

**1. CSV Export**
- Events list
- Delivery attempts
- Statistics summary
- Custom date range

**2. JSON Export**
- Full API response format
- Useful for programmatic access
- Includes metadata

**3. PDF Report** (Future)
- Executive summary
- Charts as images
- Customizable sections
- Branding support

### UI Components

**1. Export Button** (on all analytics pages)
- Dropdown menu: CSV, JSON, PDF (future)
- "Export Current View" (respects filters)
- "Export All Data" (no filters)
- Shows estimated size/records

**2. Export Configuration Modal**
- Date range selection
- Columns to include (for CSV)
- Format options (timestamps, numbers)
- Download button

**3. Export History** (Settings page)
- List of recent exports
- Download again
- Delete old exports

### Backend Requirements

```
POST /api/v1/export/events
Body: {
  format: "csv" | "json",
  filters: {
    start_date?: string,
    end_date?: string,
    chain_ids?: number[],
    status?: string
  },
  fields?: string[]
}
Response: File download or job_id for async processing

GET /api/v1/export/status/{job_id}
Response: {
  job_id: string,
  status: "pending" | "processing" | "complete" | "failed",
  progress: number,
  download_url?: string,
  error?: string
}
```

### Implementation Tasks

**Backend (Rust):**
- [ ] Create export handler for events
- [ ] Create export handler for deliveries
- [ ] Add CSV serialization
- [ ] Add JSON serialization
- [ ] Implement async export jobs (for large datasets)
- [ ] Add job status tracking
- [ ] Add file cleanup task
- [ ] Add rate limiting
- [ ] Add tests for export functionality

**Frontend (TypeScript/React):**
- [ ] Create ExportButton component
- [ ] Create ExportConfigModal component
- [ ] Implement file download
- [ ] Add progress indicator for large exports
- [ ] Create export history page
- [ ] Add tests for export components

**Estimated Effort:** 3-4 days

---

## 🗓️ Implementation Roadmap

### Sprint 1: Applications Analytics (Week 1)
- **Days 1-2:** Backend endpoints for application statistics
- **Days 3-4:** Frontend ApplicationDetailPage component
- **Day 5:** Testing, bug fixes, documentation

**Deliverable:** Working application analytics page with drill-down

### Sprint 2: Endpoints Analytics (Week 2)
- **Days 1-2:** Backend endpoints for endpoint statistics
- **Days 3-4:** Frontend EndpointDetailPage with health score
- **Day 5:** Performance scatter plot, testing

**Deliverable:** Working endpoint analytics page with performance insights

### Sprint 3: Advanced Filtering (Week 3)
- **Days 1-2:** Backend filter parameter support
- **Days 3-4:** Frontend FilterBar component
- **Day 5:** Integration across all pages, testing

**Deliverable:** Universal filtering system across all analytics

### Sprint 4: Real-Time Updates (Week 4)
- **Days 1-3:** WebSocket server implementation
- **Days 4-5:** Frontend WebSocket client, live components

**Deliverable:** Live dashboard with real-time updates

### Sprint 5: Export & Polish (Week 5)
- **Days 1-2:** CSV/JSON export backend
- **Day 3:** Frontend export UI
- **Days 4-5:** Bug fixes, performance optimization, documentation

**Deliverable:** Complete Phase 2 with export capabilities

---

## 🎯 Success Metrics

**Quantitative:**
- All 5 features implemented and tested
- Page load time < 2s for analytics pages
- WebSocket connection stability > 99%
- Export jobs complete in < 30s for 10k records
- Test coverage > 80%

**Qualitative:**
- Users can drill down from overview to endpoint details
- Filtering is intuitive and fast
- Real-time updates enhance monitoring experience
- Export provides useful data in expected format

---

## 🧪 Testing Strategy

**Unit Tests:**
- All new backend handlers
- All new React components
- Filter logic
- WebSocket message handling

**Integration Tests:**
- API endpoints with various filters
- WebSocket connection lifecycle
- Export job processing

**E2E Tests:**
- Navigate from dashboard to application detail
- Apply filters and verify results
- Export data and download file
- Connect WebSocket and receive updates

**Load Tests:**
- 100 concurrent WebSocket connections
- Export jobs with 100k+ records
- Complex filter queries

---

## 📦 Dependencies

**Backend (Rust):**
- `tokio-tungstenite` - WebSocket support
- `csv` - CSV serialization
- `serde_json` - JSON serialization
- `tokio-task` - Async job processing (optional)

**Frontend:**
- `date-fns` - Date range calculations
- `react-datepicker` - Date range picker
- `@tanstack/react-query` - Enhanced with WebSocket updates
- Native WebSocket API - No additional library needed

**Infrastructure:**
- Redis (optional) - For export job queue and WebSocket pub/sub
- nginx - WebSocket proxy configuration

---

## 🔐 Security Considerations

- All analytics endpoints require authentication
- User can only see their own applications/endpoints
- WebSocket connections require JWT validation
- Export files deleted after 24 hours
- Rate limiting on export requests
- Input validation on all filter parameters

---

## 📝 Documentation Requirements

- [ ] API documentation for new endpoints
- [ ] User guide for analytics features
- [ ] WebSocket connection guide for developers
- [ ] Export format specifications
- [ ] Filter syntax reference
- [ ] Update README with Phase 2 features

---

## 🚀 Post-Phase 2 Ideas

**Phase 3 Possibilities:**
- Alerting system (email/Slack on failures)
- Custom dashboards (drag-drop widgets)
- Anomaly detection (ML-powered)
- Multi-user collaboration (shared dashboards)
- SLA tracking and reporting
- Webhook testing playground
- Historical data archival
- Cross-application analytics

---

## ✅ Definition of Done

Phase 2 is complete when:
- [ ] All 5 features implemented and tested
- [ ] Documentation updated
- [ ] All tests passing (unit, integration, E2E)
- [ ] Code review completed
- [ ] Deployed to production
- [ ] Performance benchmarks met
- [ ] User acceptance testing passed
- [ ] Known issues documented
- [ ] Phase 3 planning initiated

---

**Next Action:** Review and prioritize features with stakeholders
