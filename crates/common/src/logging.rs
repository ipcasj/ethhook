//! Logging and tracing setup
//!
//! Provides structured logging configuration using tracing.
//! Similar to Log4j but with better support for async and structured data.

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize tracing/logging for the application
///
/// Java equivalent (Log4j):
/// ```java
/// Logger logger = LogManager.getLogger(MyClass.class);
/// logger.info("User logged in", Map.of("userId", userId));
/// ```
///
/// Rust (tracing):
/// ```rust
/// use tracing::{info, error};
/// info!(user_id = %user_id, "User logged in");
/// error!(error = %err, "Failed to process");
/// ```
///
/// Usage:
/// ```rust
/// // In main.rs
/// ethhook_common::init_tracing();
///
/// // Then in your code:
/// tracing::info!("Server starting on port 8080");
/// tracing::error!(error = %e, "Database connection failed");
/// ```
pub fn init_tracing() {
    // Read log level from RUST_LOG env var, default to info
    // Examples:
    //   RUST_LOG=debug cargo run     -> debug and higher
    //   RUST_LOG=error cargo run     -> only errors
    //   RUST_LOG=ethhook=trace cargo run  -> trace for ethhook only
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true) // Show module path
                .with_level(true) // Show log level
                .with_thread_ids(true) // Show thread IDs
                .with_file(true) // Show file and line
                .compact(), // Compact format
        )
        .init();
}

/// Initialize tracing with JSON output (for production)
///
/// Outputs logs in JSON format, better for log aggregation systems
/// like Grafana Loki, ELK stack, etc.
///
/// Example output:
/// ```json
/// {
///   "timestamp": "2025-10-03T10:30:45.123Z",
///   "level": "INFO",
///   "target": "ethhook::api",
///   "fields": {
///     "message": "User logged in",
///     "user_id": "user123",
///     "ip": "192.168.1.1"
///   }
/// }
/// ```
pub fn init_tracing_json() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .json() // JSON format
                .with_target(true)
                .with_level(true)
                .with_current_span(true),
        )
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{error, info, warn};

    #[test]
    fn test_tracing_init() {
        // Initialize tracing (only once per test suite)
        // Note: This might fail if already initialized in another test
        let _result = std::panic::catch_unwind(|| {
            init_tracing();
        });

        // Even if init fails (already initialized), we can still log
        info!("Test log message");
        warn!(count = 42, "Warning with structured data");
        error!(error = "test error", "Error message");
    }
}
