/*!
 * Configuration Module for Message Processor
 *
 * Loads environment variables and validates configuration.
 */

use anyhow::{Context, Result};
use std::env;

/// Main configuration for the Message Processor service
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// PostgreSQL connection URL
    pub database_url: String,

    /// Redis connection settings
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: Option<String>,

    /// Consumer group name for Redis Streams
    pub consumer_group: String,

    /// Consumer name (usually hostname or pod name)
    pub consumer_name: String,

    /// List of chains to process
    pub chains: Vec<ChainToProcess>,

    /// Batch size for XREAD (how many events to read at once)
    pub batch_size: usize,

    /// Block time in milliseconds for XREAD (0 = wait forever)
    pub block_time_ms: usize,

    /// Metrics server port (for Prometheus)
    #[allow(dead_code)]
    pub metrics_port: u16,
}

/// Chain configuration for processing
#[derive(Debug, Clone)]
pub struct ChainToProcess {
    /// Chain ID (1 = Ethereum, 42161 = Arbitrum, etc.)
    #[allow(dead_code)]
    pub chain_id: u64,

    /// Stream name (e.g., "events:1")
    pub stream_name: String,
}

impl ProcessorConfig {
    /// Load configuration from environment variables
    ///
    /// Required environment variables:
    /// - DATABASE_URL
    /// - REDIS_HOST
    /// - REDIS_PORT
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

        // Consumer configuration
        let consumer_group =
            env::var("CONSUMER_GROUP").unwrap_or_else(|_| "message_processors".to_string());
        let consumer_name = env::var("CONSUMER_NAME").unwrap_or_else(|_| {
            // Use hostname as consumer name
            hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "processor-1".to_string())
        });

        // Processing configuration
        let batch_size = env::var("BATCH_SIZE")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .context("BATCH_SIZE must be a valid number")?;

        let block_time_ms = env::var("BLOCK_TIME_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse::<usize>()
            .context("BLOCK_TIME_MS must be a valid number")?;

        // Metrics configuration
        let metrics_port = env::var("METRICS_PORT")
            .unwrap_or_else(|_| "9091".to_string())
            .parse::<u16>()
            .context("METRICS_PORT must be a valid port number")?;

        // Determine environment (default to development = Sepolia)
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        // Chain configurations based on environment
        let chains = if environment == "production" {
            // Production: Mainnet chains
            vec![
                ChainToProcess {
                    chain_id: 1,
                    stream_name: "events:1".to_string(),
                },
                ChainToProcess {
                    chain_id: 42161,
                    stream_name: "events:42161".to_string(),
                },
                ChainToProcess {
                    chain_id: 10,
                    stream_name: "events:10".to_string(),
                },
                ChainToProcess {
                    chain_id: 8453,
                    stream_name: "events:8453".to_string(),
                },
            ]
        } else {
            // Development/Staging: Sepolia testnet only
            // Note: For testing/development, we only monitor Sepolia to keep tests simple
            // In a real staging environment, you might want to add testnet versions of L2s
            vec![ChainToProcess {
                chain_id: 11155111, // Sepolia Testnet
                stream_name: "events:11155111".to_string(),
            }]
        };

        Ok(Self {
            database_url,
            redis_host,
            redis_port,
            redis_password,
            consumer_group,
            consumer_name,
            chains,
            batch_size,
            block_time_ms,
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
        let config = ProcessorConfig {
            database_url: "postgresql://localhost/test".to_string(),
            redis_host: "localhost".to_string(),
            redis_port: 6379,
            redis_password: None,
            consumer_group: "test".to_string(),
            consumer_name: "test-1".to_string(),
            chains: vec![],
            batch_size: 100,
            block_time_ms: 5000,
            metrics_port: 9091,
        };

        assert_eq!(config.redis_url(), "redis://localhost:6379");
    }

    #[test]
    fn test_redis_url_with_password() {
        let config = ProcessorConfig {
            database_url: "postgresql://localhost/test".to_string(),
            redis_host: "localhost".to_string(),
            redis_port: 6379,
            redis_password: Some("secret".to_string()),
            consumer_group: "test".to_string(),
            consumer_name: "test-1".to_string(),
            chains: vec![],
            batch_size: 100,
            block_time_ms: 5000,
            metrics_port: 9091,
        };

        assert_eq!(config.redis_url(), "redis://:secret@localhost:6379");
    }
}
