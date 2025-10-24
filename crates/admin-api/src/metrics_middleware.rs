/*!
 * Metrics Middleware
 *
 * Automatically tracks HTTP request metrics for all endpoints
 */

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

use crate::metrics;

/// Middleware to track HTTP request metrics
pub async fn track_metrics(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Call the next middleware/handler
    let response = next.run(req).await;

    // Record metrics
    let duration = start.elapsed();
    let status = response.status().as_u16().to_string();

    // Increment request counter
    metrics::HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &path, &status])
        .inc();

    // Record request duration
    metrics::HTTP_REQUEST_DURATION
        .with_label_values(&[&method, &path])
        .observe(duration.as_secs_f64());

    response
}
