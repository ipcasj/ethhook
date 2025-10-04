//! Database connection pool management
//!
//! Provides PostgreSQL connection pooling using sqlx.
//! Similar to Java's HikariCP but integrated with Rust's async runtime.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{info, warn};

use crate::error::Result;

/// Create a PostgreSQL connection pool
///
/// Java equivalent:
/// ```java
/// HikariConfig config = new HikariConfig();
/// config.setJdbcUrl("jdbc:postgresql://localhost:5432/ethhook");
/// config.setMaximumPoolSize(20);
/// config.setMinimumIdle(5);
/// config.setConnectionTimeout(30000);
/// HikariDataSource pool = new HikariDataSource(config);
/// ```
///
/// Rust:
/// ```rust
/// let pool = create_pool("postgresql://localhost:5432/ethhook", 20).await?;
/// ```
pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool> {
    info!(
        "Creating database pool with max_connections={}",
        max_connections
    );

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(5) // Keep 5 connections warm
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .connect(database_url)
        .await?;

    info!("Database pool created successfully");

    // Test the connection
    health_check(&pool).await?;

    Ok(pool)
}

/// Check if database is healthy
///
/// Java equivalent:
/// ```java
/// try (Connection conn = pool.getConnection()) {
///     PreparedStatement stmt = conn.prepareStatement("SELECT 1");
///     ResultSet rs = stmt.executeQuery();
///     return rs.next();
/// }
/// ```
///
/// Rust:
/// ```rust
/// health_check(&pool).await?;
/// ```
pub async fn health_check(pool: &PgPool) -> Result<()> {
    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await?;

    if row.0 == 1 {
        info!("Database health check passed");
        Ok(())
    } else {
        warn!("Database health check failed");
        Err(crate::error::Error::Database(sqlx::Error::Protocol(
            "Health check failed".into(),
        )))
    }
}

/// Get pool statistics for monitoring
///
/// Returns: (size, idle, connections)
pub fn pool_stats(pool: &PgPool) -> (u32, usize) {
    let size = pool.size();
    let idle = pool.num_idle();
    (size, idle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        // This test requires DATABASE_URL env var
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            let result = create_pool(&database_url, 5).await;
            assert!(result.is_ok(), "Failed to create pool: {:?}", result.err());

            let pool = result.unwrap();
            let (size, idle) = pool_stats(&pool);
            assert!(size > 0, "Pool should have connections");
            assert!(idle <= size as usize, "Idle should not exceed size");
        } else {
            println!("Skipping test: DATABASE_URL not set");
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            let pool = create_pool(&database_url, 5).await.unwrap();
            let result = health_check(&pool).await;
            assert!(result.is_ok(), "Health check should pass");
        }
    }
}
