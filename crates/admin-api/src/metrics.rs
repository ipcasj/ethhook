/*!
 * Prometheus Metrics for Admin API
 *
 * Exposes metrics on /metrics endpoint for Prometheus scraping
 */

use lazy_static::lazy_static;
use prometheus::{
    Encoder, HistogramVec, IntCounterVec, IntGaugeVec, TextEncoder, opts, register_histogram_vec,
    register_int_counter_vec, register_int_gauge_vec,
};

lazy_static! {
    /// HTTP request counter by method, path, and status
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("admin_api_http_requests_total", "Total number of HTTP requests"),
        &["method", "path", "status"]
    )
    .expect("metric can be created");

    /// HTTP request duration histogram
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "admin_api_http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "path"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .expect("metric can be created");

    /// Database query counter
    pub static ref DB_QUERIES_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("admin_api_db_queries_total", "Total number of database queries"),
        &["operation", "table"]
    )
    .expect("metric can be created");

    /// Active database connections
    pub static ref DB_CONNECTIONS_ACTIVE: IntGaugeVec = register_int_gauge_vec!(
        opts!("admin_api_db_connections_active", "Number of active database connections"),
        &["state"]
    )
    .expect("metric can be created");

    /// API key validation attempts
    pub static ref API_KEY_VALIDATIONS_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("admin_api_api_key_validations_total", "Total API key validation attempts"),
        &["result"]
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
