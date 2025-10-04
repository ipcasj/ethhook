/// Configuration management for EthHook
/// 
/// This module handles loading and validating configuration from environment variables.
/// 
/// Java Equivalent:
/// - @ConfigurationProperties in Spring Boot
/// - System.getenv() for environment variables
/// - application.yml for configuration files

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

/// Main configuration struct for the entire application
/// 
/// Java equivalent: @ConfigurationProperties(prefix = "ethhook")
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub chains: Vec<ChainConfig>,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub api: ApiConfig,
    pub webhook: WebhookConfig,
    pub observability: ObservabilityConfig,
}

/// Configuration for a single blockchain network
/// 
/// Supports multiple chains (Ethereum, Arbitrum, Optimism, Base)
/// All are EVM-compatible, so same API works for all
#[derive(Debug, Clone, Deserialize)]
pub struct ChainConfig {
    /// Chain ID (1 = Ethereum, 42161 = Arbitrum, 10 = Optimism, 8453 = Base)
    pub chain_id: u64,
    
    /// Human-readable name for logging
    pub name: String,
    
    /// WebSocket RPC endpoint (for real-time events)
    /// Example: wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
    pub rpc_ws: String,
    
    /// HTTP RPC endpoint (for fallback and queries)
    /// Example: https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
    pub rpc_http: String,
    
    /// Average block time in milliseconds (Ethereum: 12000, Base: 2000)
    pub block_time_ms: u64,
    
    /// Block explorer URL for linking in UI
    pub explorer_url: String,
}

/// PostgreSQL database configuration
/// 
/// Java equivalent: spring.datasource.*
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Connection string
    /// Format: postgresql://user:password@host:port/database
    pub url: String,
    
    /// Maximum number of connections in the pool
    /// Java equivalent: spring.datasource.hikari.maximum-pool-size
    pub max_connections: u32,
    
    /// Minimum number of connections to maintain
    pub min_connections: u32,
}

/// Redis configuration for streams and queues
/// 
/// Java equivalent: spring.redis.*
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Connection URL
    /// Format: redis://host:port or redis://user:password@host:port
    pub url: String,
    
    /// Connection pool size
    pub pool_size: usize,
}

/// REST API server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    /// Host to bind to (0.0.0.0 for all interfaces)
    pub host: String,
    
    /// Port to listen on
    pub port: u16,
    
    /// JWT secret for token signing (min 256 bits)
    pub jwt_secret: String,
    
    /// JWT expiration in hours
    pub jwt_expiration_hours: i64,
    
    /// Rate limit: requests per minute per user
    pub rate_limit_per_minute: u32,
}

/// Webhook delivery configuration
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    /// HTTP timeout in seconds
    pub timeout_seconds: u64,
    
    /// Maximum retry attempts for failed deliveries
    pub max_retries: u32,
    
    /// Number of worker threads for parallel delivery
    pub worker_threads: usize,
}

/// Observability configuration (metrics, logs, tracing)
#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    /// Rust log level (trace, debug, info, warn, error)
    /// Java equivalent: logging.level.root
    pub rust_log: String,
    
    /// OpenTelemetry collector endpoint
    /// Example: http://localhost:4317
    pub otel_endpoint: Option<String>,
    
    /// Prometheus metrics port
    pub metrics_port: u16,
}

impl Config {
    /// Load configuration from environment variables
    /// 
    /// Java equivalent:
    /// @Autowired
    /// private Environment env;
    /// 
    /// Rust pattern: Load from .env file, validate, return Result
    pub fn load() -> Result<Self> {
        // Load .env file (like Spring Boot's application.properties)
        dotenvy::dotenv().ok();
        
        Ok(Config {
            chains: Self::load_chains()?,
            database: Self::load_database()?,
            redis: Self::load_redis()?,
            api: Self::load_api()?,
            webhook: Self::load_webhook()?,
            observability: Self::load_observability()?,
        })
    }
    
    /// Load blockchain configurations
    /// 
    /// Pattern: Load all supported chains, skip if env vars not set
    fn load_chains() -> Result<Vec<ChainConfig>> {
        let mut chains = Vec::new();
        
        // Ethereum Mainnet
        if let (Ok(ws), Ok(http)) = (
            env::var("ETH_MAINNET_WS"),
            env::var("ETH_MAINNET_HTTP"),
        ) {
            chains.push(ChainConfig {
                chain_id: 1,
                name: "Ethereum".to_string(),
                rpc_ws: ws,
                rpc_http: http,
                block_time_ms: 12000,
                explorer_url: "https://etherscan.io".to_string(),
            });
        }
        
        // Arbitrum
        if let (Ok(ws), Ok(http)) = (
            env::var("ARBITRUM_WS"),
            env::var("ARBITRUM_HTTP"),
        ) {
            chains.push(ChainConfig {
                chain_id: 42161,
                name: "Arbitrum".to_string(),
                rpc_ws: ws,
                rpc_http: http,
                block_time_ms: 250,
                explorer_url: "https://arbiscan.io".to_string(),
            });
        }
        
        // Optimism
        if let (Ok(ws), Ok(http)) = (
            env::var("OPTIMISM_WS"),
            env::var("OPTIMISM_HTTP"),
        ) {
            chains.push(ChainConfig {
                chain_id: 10,
                name: "Optimism".to_string(),
                rpc_ws: ws,
                rpc_http: http,
                block_time_ms: 2000,
                explorer_url: "https://optimistic.etherscan.io".to_string(),
            });
        }
        
        // Base
        if let (Ok(ws), Ok(http)) = (
            env::var("BASE_WS"),
            env::var("BASE_HTTP"),
        ) {
            chains.push(ChainConfig {
                chain_id: 8453,
                name: "Base".to_string(),
                rpc_ws: ws,
                rpc_http: http,
                block_time_ms: 2000,
                explorer_url: "https://basescan.org".to_string(),
            });
        }
        
        // Ensure at least one chain is configured
        if chains.is_empty() {
            anyhow::bail!("No blockchain chains configured. Set at least one of: ETH_MAINNET_WS, ARBITRUM_WS, OPTIMISM_WS, BASE_WS");
        }
        
        Ok(chains)
    }
    
    fn load_database() -> Result<DatabaseConfig> {
        Ok(DatabaseConfig {
            url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20),
            min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        })
    }
    
    fn load_redis() -> Result<RedisConfig> {
        Ok(RedisConfig {
            url: env::var("REDIS_URL")
                .context("REDIS_URL must be set")?,
            pool_size: env::var("REDIS_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        })
    }
    
    fn load_api() -> Result<ApiConfig> {
        let jwt_secret = env::var("JWT_SECRET")
            .context("JWT_SECRET must be set")?;
        
        // Validate JWT secret length (min 32 bytes for security)
        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters");
        }
        
        Ok(ApiConfig {
            host: env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("API_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            jwt_secret,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(24),
            rate_limit_per_minute: env::var("API_RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
        })
    }
    
    fn load_webhook() -> Result<WebhookConfig> {
        Ok(WebhookConfig {
            timeout_seconds: env::var("WEBHOOK_TIMEOUT_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            max_retries: env::var("WEBHOOK_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            worker_threads: env::var("WEBHOOK_WORKER_THREADS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        })
    }
    
    fn load_observability() -> Result<ObservabilityConfig> {
        Ok(ObservabilityConfig {
            rust_log: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,ethhook=debug".to_string()),
            otel_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            metrics_port: env::var("PROMETHEUS_METRICS_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(9090),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        // Test JWT secret validation
        env::set_var("JWT_SECRET", "short");
        env::set_var("DATABASE_URL", "postgresql://localhost/test");
        env::set_var("REDIS_URL", "redis://localhost");
        env::set_var("ETH_MAINNET_WS", "wss://test");
        env::set_var("ETH_MAINNET_HTTP", "https://test");
        
        let result = Config::load();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32 characters"));
    }
}
