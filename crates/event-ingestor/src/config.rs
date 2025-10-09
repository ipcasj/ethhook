/*!
 * Configuration Module
 *
 * Loads environment variables and validates configuration.
 * Similar to Spring's @ConfigurationProperties in Java.
 */

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

/// Main configuration for the Event Ingestor service
#[derive(Debug, Clone)]
pub struct IngestorConfig {
    /// List of chains to ingest from
    pub chains: Vec<ChainConfig>,

    /// Redis connection settings
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: Option<String>,

    /// Metrics server port (for Prometheus)
    pub metrics_port: u16,

    /// Deduplication TTL in seconds (default: 24 hours)
    pub dedup_ttl_seconds: u64,
}

/// Configuration for a single blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Human-readable name (e.g., "Ethereum Mainnet")
    pub name: String,

    /// Chain ID (e.g., 1 for Ethereum, 42161 for Arbitrum)
    pub chain_id: u64,

    /// WebSocket RPC endpoint (e.g., "wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY")
    pub ws_url: String,

    /// Maximum reconnection attempts before giving up
    pub max_reconnect_attempts: u32,

    /// Initial reconnection delay in seconds
    pub reconnect_delay_secs: u64,
}

impl IngestorConfig {
    /// Load configuration from environment variables
    ///
    /// Required environment variables:
    /// - REDIS_HOST
    /// - REDIS_PORT
    /// - ETHEREUM_WS_URL
    /// - ARBITRUM_WS_URL
    /// - OPTIMISM_WS_URL
    /// - BASE_WS_URL
    pub fn from_env() -> Result<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        // Redis configuration
        let redis_host = env::var("REDIS_HOST").context("REDIS_HOST not set")?;
        let redis_port = env::var("REDIS_PORT")
            .context("REDIS_PORT not set")?
            .parse::<u16>()
            .context("REDIS_PORT must be a valid port number")?;
        let redis_password = env::var("REDIS_PASSWORD").ok();

        // Metrics configuration
        let metrics_port = env::var("METRICS_PORT")
            .unwrap_or_else(|_| "9090".to_string())
            .parse::<u16>()
            .context("METRICS_PORT must be a valid port number")?;

        // Deduplication TTL (default: 24 hours)
        let dedup_ttl_seconds = env::var("DEDUP_TTL_SECONDS")
            .unwrap_or_else(|_| "86400".to_string())
            .parse::<u64>()
            .context("DEDUP_TTL_SECONDS must be a valid number")?;

        // Chain configurations
        let chains = vec![
            ChainConfig {
                name: "Ethereum Mainnet".to_string(),
                chain_id: 1,
                ws_url: env::var("ETHEREUM_WS_URL").context("ETHEREUM_WS_URL not set")?,
                max_reconnect_attempts: 10,
                reconnect_delay_secs: 1,
            },
            ChainConfig {
                name: "Arbitrum One".to_string(),
                chain_id: 42161,
                ws_url: env::var("ARBITRUM_WS_URL").context("ARBITRUM_WS_URL not set")?,
                max_reconnect_attempts: 10,
                reconnect_delay_secs: 1,
            },
            ChainConfig {
                name: "Optimism".to_string(),
                chain_id: 10,
                ws_url: env::var("OPTIMISM_WS_URL").context("OPTIMISM_WS_URL not set")?,
                max_reconnect_attempts: 10,
                reconnect_delay_secs: 1,
            },
            ChainConfig {
                name: "Base".to_string(),
                chain_id: 8453,
                ws_url: env::var("BASE_WS_URL").context("BASE_WS_URL not set")?,
                max_reconnect_attempts: 10,
                reconnect_delay_secs: 1,
            },
        ];

        Ok(IngestorConfig {
            chains,
            redis_host,
            redis_port,
            redis_password,
            metrics_port,
            dedup_ttl_seconds,
        })
    }

    /// Get Redis connection URL
    pub fn redis_url(&self) -> String {
        match &self.redis_password {
            Some(password) => format!(
                "redis://:{}@{}:{}/",
                password, self.redis_host, self.redis_port
            ),
            None => format!("redis://{}:{}/", self.redis_host, self.redis_port),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_url_without_password() {
        let config = IngestorConfig {
            chains: vec![],
            redis_host: "localhost".to_string(),
            redis_port: 6379,
            redis_password: None,
            metrics_port: 9090,
            dedup_ttl_seconds: 86400,
        };

        assert_eq!(config.redis_url(), "redis://localhost:6379/");
    }

    #[test]
    fn test_redis_url_with_password() {
        let config = IngestorConfig {
            chains: vec![],
            redis_host: "localhost".to_string(),
            redis_port: 6379,
            redis_password: Some("secret123".to_string()),
            metrics_port: 9090,
            dedup_ttl_seconds: 86400,
        };

        assert_eq!(config.redis_url(), "redis://:secret123@localhost:6379/");
    }
}
