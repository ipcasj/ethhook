use anyhow::{Context, Result};
use std::env;

/// Configuration for the Admin API server
#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP server configuration
    pub server_host: String,
    pub server_port: u16,

    /// Database connection (SQLite for config)
    pub database_url: String,
    pub database_max_connections: u32,

    /// ClickHouse connection (for events/deliveries)
    pub clickhouse_url: String,
    pub clickhouse_user: String,
    pub clickhouse_password: String,
    pub clickhouse_database: String,

    /// JWT configuration
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,

    /// API key configuration
    #[allow(dead_code)]
    pub api_key_prefix: String,

    /// Rate limiting
    #[allow(dead_code)]
    pub rate_limit_per_minute: u32,

    /// CORS configuration
    #[allow(dead_code)]
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            server_host: env::var("ADMIN_API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("ADMIN_API_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("Failed to parse ADMIN_API_PORT")?,

            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:config.db".to_string()),
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string()) // SQLite: reduce from 20 to 5 (single file)
                .parse()
                .context("Failed to parse DATABASE_MAX_CONNECTIONS")?,

            clickhouse_url: env::var("CLICKHOUSE_URL")
                .unwrap_or_else(|_| "http://localhost:8123".to_string()),
            clickhouse_user: env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".to_string()),
            clickhouse_password: env::var("CLICKHOUSE_PASSWORD").unwrap_or_else(|_| "".to_string()),
            clickhouse_database: env::var("CLICKHOUSE_DATABASE")
                .unwrap_or_else(|_| "ethhook".to_string()),

            jwt_secret: env::var("JWT_SECRET").context("JWT_SECRET must be set")?,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .context("Failed to parse JWT_EXPIRATION_HOURS")?,

            api_key_prefix: env::var("API_KEY_PREFIX").unwrap_or_else(|_| "ethk".to_string()),

            rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .context("Failed to parse RATE_LIMIT_PER_MINUTE")?,

            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        // This test requires DATABASE_URL and JWT_SECRET
        unsafe {
            env::set_var("DATABASE_URL", "postgresql://localhost/test");
            env::set_var("JWT_SECRET", "test-secret-key");
        }

        let config = Config::from_env().unwrap();
        assert_eq!(config.server_host, "0.0.0.0");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.jwt_expiration_hours, 24);
        assert_eq!(config.api_key_prefix, "ethk");
    }
}
