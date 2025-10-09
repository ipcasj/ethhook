/*!
 * Configuration Module for Webhook Delivery
 */

use anyhow::{Context, Result};
use std::env;
use std::time::Duration;

/// Main configuration for Webhook Delivery service
#[derive(Debug, Clone)]
pub struct DeliveryConfig {
    /// PostgreSQL connection URL
    pub database_url: String,

    /// Redis connection settings
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: Option<String>,

    /// Queue name to consume from
    pub queue_name: String,

    /// Number of concurrent delivery workers
    pub worker_count: usize,

    /// HTTP timeout per request
    pub http_timeout: Duration,

    /// Maximum retries per delivery
    pub max_retries: u32,

    /// Base delay for exponential backoff (seconds)
    pub retry_base_delay_secs: u64,

    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,

    /// Circuit breaker timeout (seconds)
    pub circuit_breaker_timeout_secs: u64,

    /// Metrics server port
    pub metrics_port: u16,
}

impl DeliveryConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        // Database configuration
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;

        // Redis configuration
        let redis_host = env::var("REDIS_HOST").context("REDIS_HOST not set")?;
        let redis_port = env::var("REDIS_PORT")
            .context("REDIS_PORT not set")?
            .parse::<u16>()
            .context("REDIS_PORT must be a valid port number")?;
        let redis_password = env::var("REDIS_PASSWORD").ok();

        // Queue configuration
        let queue_name = env::var("QUEUE_NAME").unwrap_or_else(|_| "delivery_queue".to_string());

        // Worker configuration
        let worker_count = env::var("WORKER_COUNT")
            .unwrap_or_else(|_| "50".to_string())
            .parse::<usize>()
            .context("WORKER_COUNT must be a valid number")?;

        // HTTP configuration
        let http_timeout = Duration::from_secs(
            env::var("HTTP_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse::<u64>()
                .context("HTTP_TIMEOUT_SECS must be a valid number")?,
        );

        // Retry configuration
        let max_retries = env::var("MAX_RETRIES")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .context("MAX_RETRIES must be a valid number")?;

        let retry_base_delay_secs = env::var("RETRY_BASE_DELAY_SECS")
            .unwrap_or_else(|_| "2".to_string())
            .parse::<u64>()
            .context("RETRY_BASE_DELAY_SECS must be a valid number")?;

        // Circuit breaker configuration
        let circuit_breaker_threshold = env::var("CIRCUIT_BREAKER_THRESHOLD")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .context("CIRCUIT_BREAKER_THRESHOLD must be a valid number")?;

        let circuit_breaker_timeout_secs = env::var("CIRCUIT_BREAKER_TIMEOUT_SECS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .context("CIRCUIT_BREAKER_TIMEOUT_SECS must be a valid number")?;

        // Metrics configuration
        let metrics_port = env::var("METRICS_PORT")
            .unwrap_or_else(|_| "9092".to_string())
            .parse::<u16>()
            .context("METRICS_PORT must be a valid port number")?;

        Ok(Self {
            database_url,
            redis_host,
            redis_port,
            redis_password,
            queue_name,
            worker_count,
            http_timeout,
            max_retries,
            retry_base_delay_secs,
            circuit_breaker_threshold,
            circuit_breaker_timeout_secs,
            metrics_port,
        })
    }

    /// Get Redis connection URL
    pub fn redis_url(&self) -> String {
        if let Some(password) = &self.redis_password {
            format!(
                "redis://:{}@{}:{}",
                password, self.redis_host, self.redis_port
            )
        } else {
            format!("redis://{}:{}", self.redis_host, self.redis_port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_url_without_password() {
        let config = DeliveryConfig {
            database_url: "postgresql://localhost/test".to_string(),
            redis_host: "localhost".to_string(),
            redis_port: 6379,
            redis_password: None,
            queue_name: "test_queue".to_string(),
            worker_count: 50,
            http_timeout: Duration::from_secs(30),
            max_retries: 5,
            retry_base_delay_secs: 2,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_secs: 60,
            metrics_port: 9092,
        };

        assert_eq!(config.redis_url(), "redis://localhost:6379");
    }

    #[test]
    fn test_redis_url_with_password() {
        let config = DeliveryConfig {
            database_url: "postgresql://localhost/test".to_string(),
            redis_host: "localhost".to_string(),
            redis_port: 6379,
            redis_password: Some("secret".to_string()),
            queue_name: "test_queue".to_string(),
            worker_count: 50,
            http_timeout: Duration::from_secs(30),
            max_retries: 5,
            retry_base_delay_secs: 2,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_secs: 60,
            metrics_port: 9092,
        };

        assert_eq!(config.redis_url(), "redis://:secret@localhost:6379");
    }
}
