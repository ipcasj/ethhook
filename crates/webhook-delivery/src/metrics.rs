/*!
 * Prometheus Metrics for Webhook Delivery
 *
 * Exposes metrics on /metrics endpoint for Prometheus scraping
 */

use lazy_static::lazy_static;
use prometheus::{
    Encoder, HistogramVec, IntCounterVec, IntGaugeVec, TextEncoder, opts, register_histogram_vec,
    register_int_counter_vec, register_int_gauge_vec,
};

lazy_static! {
    /// Delivery attempts counter
    pub static ref DELIVERY_ATTEMPTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("webhook_delivery_attempts_total", "Total number of delivery attempts"),
        &["endpoint", "success"]
    )
    .expect("metric can be created");

    /// Delivery duration histogram
    pub static ref DELIVERY_DURATION: HistogramVec = register_histogram_vec!(
        "webhook_delivery_duration_seconds",
        "Webhook delivery duration in seconds",
        &["endpoint"],
        vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0]
    )
    .expect("metric can be created");

    /// HTTP status codes received
    pub static ref HTTP_RESPONSES_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("webhook_delivery_http_responses_total", "Total HTTP responses by status code"),
        &["status_code"]
    )
    .expect("metric can be created");

    /// Circuit breaker state
    pub static ref CIRCUIT_BREAKER_STATE: IntGaugeVec = register_int_gauge_vec!(
        opts!("webhook_delivery_circuit_breaker_state", "Circuit breaker state (0=Closed, 1=Open, 2=HalfOpen)"),
        &["endpoint"]
    )
    .expect("metric can be created");

    /// Jobs consumed from queue
    pub static ref JOBS_CONSUMED_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("webhook_delivery_jobs_consumed_total", "Total jobs consumed from delivery queue"),
        &["worker"]
    )
    .expect("metric can be created");

    /// Current delivery queue length
    pub static ref DELIVERY_QUEUE_LENGTH: IntGaugeVec = register_int_gauge_vec!(
        opts!("webhook_delivery_queue_length", "Current delivery queue length"),
        &["queue"]
    )
    .expect("metric can be created");

    /// Retry attempts
    pub static ref RETRY_ATTEMPTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("webhook_delivery_retry_attempts_total", "Total number of retry attempts"),
        &["endpoint"]
    )
    .expect("metric can be created");
}

/// Render metrics in Prometheus format
pub fn render_metrics() -> Result<String, Box<dyn std::error::Error>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}
