/*!
 * Application State
 *
 * Shared state for the admin API server.
 */

use crate::config::Config;
use ethhook_common::ClickHouseClient;
use sqlx::SqlitePool;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub clickhouse: ClickHouseClient,
    pub config: Config,
}

// Implement FromRef to allow extracting individual pieces from AppState
impl axum::extract::FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl axum::extract::FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}
