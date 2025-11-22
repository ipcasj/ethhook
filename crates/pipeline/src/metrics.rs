use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge, register_histogram_vec, CounterVec, Gauge, HistogramVec,
};

lazy_static! {
    /// Events received from blockchain WebSockets
    pub static ref EVENTS_RECEIVED: CounterVec = register_counter_vec!(
        "pipeline_events_received_total",
        "Total number of events received from blockchain",
        &["chain"]
    )
    .expect("Failed to create EVENTS_RECEIVED metric");

    /// Events processed by batch processor
    pub static ref EVENTS_PROCESSED: CounterVec = register_counter_vec!(
        "pipeline_events_processed_total",
        "Total number of events processed",
        &["status"] // "matched", "no_match"
    )
    .expect("Failed to create EVENTS_PROCESSED metric");

    /// Webhook deliveries attempted
    pub static ref DELIVERIES_ATTEMPTED: CounterVec = register_counter_vec!(
        "pipeline_deliveries_attempted_total",
        "Total number of webhook deliveries attempted",
        &["status"] // "success", "failure", "retry"
    )
    .expect("Failed to create DELIVERIES_ATTEMPTED metric");

    /// Latency from event receipt to delivery attempt
    pub static ref E2E_LATENCY: HistogramVec = register_histogram_vec!(
        "pipeline_e2e_latency_seconds",
        "End-to-end latency from event receipt to delivery",
        &["chain"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0] // 1ms to 1s
    )
    .expect("Failed to create E2E_LATENCY metric");

    /// Batch processor latency
    pub static ref BATCH_LATENCY: HistogramVec = register_histogram_vec!(
        "pipeline_batch_latency_seconds",
        "Batch processing latency",
        &["operation"], // "match", "db_query"
        vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.025, 0.05, 0.1] // 0.1ms to 100ms
    )
    .expect("Failed to create BATCH_LATENCY metric");

    /// HTTP delivery latency
    pub static ref HTTP_DELIVERY_LATENCY: HistogramVec = register_histogram_vec!(
        "pipeline_http_delivery_latency_seconds",
        "HTTP delivery latency",
        &["status_code"],
        vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0] // 10ms to 10s
    )
    .expect("Failed to create HTTP_DELIVERY_LATENCY metric");

    /// Channel buffer utilization (gauge)
    pub static ref EVENT_CHANNEL_SIZE: Gauge = register_gauge!(
        "pipeline_event_channel_size",
        "Current number of events in event channel"
    )
    .expect("Failed to create EVENT_CHANNEL_SIZE metric");

    pub static ref DELIVERY_CHANNEL_SIZE: Gauge = register_gauge!(
        "pipeline_delivery_channel_size",
        "Current number of deliveries in delivery channel"
    )
    .expect("Failed to create DELIVERY_CHANNEL_SIZE metric");

    /// Active WebSocket connections
    pub static ref ACTIVE_WS_CONNECTIONS: Gauge = register_gauge!(
        "pipeline_active_websocket_connections",
        "Number of active blockchain WebSocket connections"
    )
    .expect("Failed to create ACTIVE_WS_CONNECTIONS metric");

    /// Active HTTP delivery workers
    pub static ref ACTIVE_HTTP_WORKERS: Gauge = register_gauge!(
        "pipeline_active_http_workers",
        "Number of active HTTP delivery workers"
    )
    .expect("Failed to create ACTIVE_HTTP_WORKERS metric");

    /// Database connection pool metrics
    pub static ref DB_POOL_SIZE: Gauge = register_gauge!(
        "pipeline_db_pool_size",
        "Current database connection pool size"
    )
    .expect("Failed to create DB_POOL_SIZE metric");

    pub static ref DB_POOL_IDLE: Gauge = register_gauge!(
        "pipeline_db_pool_idle",
        "Idle database connections in pool"
    )
    .expect("Failed to create DB_POOL_IDLE metric");
}

/// Record event received from blockchain
pub fn record_event_received(chain: &str) {
    EVENTS_RECEIVED.with_label_values(&[chain]).inc();
}

/// Record event processing result
pub fn record_event_processed(matched: bool) {
    let status = if matched { "matched" } else { "no_match" };
    EVENTS_PROCESSED.with_label_values(&[status]).inc();
}

/// Record delivery attempt
pub fn record_delivery_attempt(success: bool) {
    let status = if success { "success" } else { "failure" };
    DELIVERIES_ATTEMPTED.with_label_values(&[status]).inc();
}

/// Record end-to-end latency
pub fn record_e2e_latency(chain: &str, latency_seconds: f64) {
    E2E_LATENCY.with_label_values(&[chain]).observe(latency_seconds);
}

/// Get Prometheus metrics in text format
pub fn get_metrics() -> String {
    use prometheus::TextEncoder;
    
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(metrics) => metrics,
        Err(e) => {
            tracing::error!("Failed to encode metrics: {}", e);
            String::new()
        }
    }
}
