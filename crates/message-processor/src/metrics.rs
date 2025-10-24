/*!
 * Prometheus Metrics for Message Processor
 *
 * Exposes metrics on /metrics endpoint for Prometheus scraping
 */

use lazy_static::lazy_static;
use prometheus::{
    Encoder, HistogramVec, IntCounterVec, IntGaugeVec, TextEncoder, opts, register_histogram_vec,
    register_int_counter_vec, register_int_gauge_vec,
};

lazy_static! {
    /// Events consumed from Redis streams
    pub static ref EVENTS_CONSUMED_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("message_processor_events_consumed_total", "Total number of events consumed from Redis"),
        &["stream"]
    )
    .expect("metric can be created");

    /// Events processed successfully
    pub static ref EVENTS_PROCESSED_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("message_processor_events_processed_total", "Total number of events processed successfully"),
        &["endpoint"]
    )
    .expect("metric can be created");

    /// Processing errors
    pub static ref PROCESSING_ERRORS_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("message_processor_processing_errors_total", "Total number of processing errors"),
        &["error_type"]
    )
    .expect("metric can be created");

    /// Event processing duration
    pub static ref PROCESSING_DURATION: HistogramVec = register_histogram_vec!(
        "message_processor_processing_duration_seconds",
        "Event processing duration in seconds",
        &["operation"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5]
    )
    .expect("metric can be created");

    /// Webhooks published to delivery queue
    pub static ref WEBHOOKS_PUBLISHED_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("message_processor_webhooks_published_total", "Total number of webhooks published to delivery queue"),
        &["endpoint"]
    )
    .expect("metric can be created");

    /// Current Redis queue length (events pending)
    pub static ref REDIS_QUEUE_LENGTH: IntGaugeVec = register_int_gauge_vec!(
        opts!("message_processor_redis_queue_length", "Current Redis stream/queue length"),
        &["queue"]
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
