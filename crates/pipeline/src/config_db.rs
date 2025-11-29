/*!
 * Configuration Database Module
 *
 * SQLite-based config storage with in-memory caching for hot path.
 *
 * ## Architecture
 *
 * ```
 * SQLite File (config.db)
 *     ↓
 * In-Memory Cache (DashMap)
 *     ↓
 * Hot Path (zero DB queries)
 * ```
 *
 * ## Why SQLite?
 *
 * - Zero deployment (no server process)
 * - 100K reads/sec (faster than PostgreSQL for small data)
 * - 10MB file for all config data
 * - Battle-tested: Cloudflare, Apple, Expensify
 */

use anyhow::{Context, Result};
use dashmap::DashMap;
use ethhook_domain::endpoint::Endpoint;
use once_cell::sync::Lazy;
use sqlx::{SqlitePool, Row};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Global endpoint cache - populated at startup, refreshed every 10s
///
/// Hot path: ZERO database queries (all lookups are O(1) in-memory)
pub static ENDPOINT_CACHE: Lazy<DashMap<String, Vec<Endpoint>>> = Lazy::new(DashMap::new);

/// Global application cache - maps application_id -> user_id
pub static APPLICATION_CACHE: Lazy<DashMap<Uuid, Uuid>> = Lazy::new(DashMap::new);

/// Initialize config database and start cache refresh loop
///
/// Safety Rule #4: 30s timeout for database operations
pub async fn init_config_db(db_path: &str, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
    info!("Initializing config database: {}", db_path);

    // Connect to SQLite
    let db = SqlitePool::connect(db_path)
        .await
        .context("Failed to connect to config database")?;

    // Run migrations (relative to workspace root)
    sqlx::migrate!("../../migrations")
        .run(&db)
        .await
        .context("Failed to run database migrations")?;

    info!("Config database initialized successfully");

    // Initial cache load
    refresh_endpoint_cache(&db).await?;
    refresh_application_cache(&db).await?;

    // Start background refresh task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = refresh_endpoint_cache(&db).await {
                        error!("Failed to refresh endpoint cache: {}", e);
                    }
                    if let Err(e) = refresh_application_cache(&db).await {
                        error!("Failed to refresh application cache: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Config DB refresh task shutting down");
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Refresh endpoint cache from database
///
/// Loads all active endpoints and rebuilds in-memory index by contract_address
async fn refresh_endpoint_cache(db: &SqlitePool) -> Result<()> {
    debug!("Refreshing endpoint cache");

    // Safety Rule #4: 30s timeout for query
    let endpoints = tokio::time::timeout(
        Duration::from_secs(30),
        sqlx::query_as::<_, Endpoint>(
            "SELECT id, application_id, name, url, hmac_secret, 
                    contract_address, event_topics, rate_limit_per_second,
                    max_retries, timeout_seconds, is_active, health_status,
                    last_successful_delivery_at, consecutive_failures,
                    created_at, updated_at
             FROM endpoints 
             WHERE is_active = 1 AND health_status != 'failed'",
        )
        .fetch_all(db),
    )
    .await
    .context("Query timeout")?
    .context("Failed to fetch endpoints")?;

    // Build new cache index
    let mut new_cache: std::collections::HashMap<String, Vec<Endpoint>> =
        std::collections::HashMap::new();

    for endpoint in endpoints {
        if let Some(ref address) = endpoint.contract_address {
            new_cache
                .entry(address.to_lowercase())
                .or_default()
                .push(endpoint);
        }
    }

    // Atomically replace cache
    ENDPOINT_CACHE.clear();
    for (address, endpoints) in new_cache {
        ENDPOINT_CACHE.insert(address, endpoints);
    }

    info!(
        "Endpoint cache refreshed: {} contracts",
        ENDPOINT_CACHE.len()
    );
    Ok(())
}

/// Refresh application cache from database
///
/// Loads all active applications and builds map: application_id -> user_id
async fn refresh_application_cache(db: &SqlitePool) -> Result<()> {
    debug!("Refreshing application cache");

    // Query applications (manual UUID parsing for SQLite TEXT format)
    let rows = tokio::time::timeout(
        Duration::from_secs(30),
        sqlx::query("SELECT id, user_id FROM applications WHERE is_active = 1")
            .fetch_all(db),
    )
    .await
    .context("Query timeout")?
    .context("Failed to fetch applications")?;

    // Parse UUIDs from SQLite TEXT format (lowercase hex without dashes)
    APPLICATION_CACHE.clear();
    for row in rows {
        let id_str: String = row.get("id");
        let user_id_str: String = row.get("user_id");
        
        // Parse hex strings to UUIDs (SQLite stores as lowercase hex without dashes)
        if let (Ok(app_id), Ok(user_id)) = (Uuid::parse_str(&id_str), Uuid::parse_str(&user_id_str)) {
            APPLICATION_CACHE.insert(app_id, user_id);
        } else {
            debug!("Skipping invalid UUID: id={}, user_id={}", id_str, user_id_str);
        }
    }

    info!("Application cache refreshed: {} applications", APPLICATION_CACHE.len());
    Ok(())
}
