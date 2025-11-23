//! # EthHook Common Library
//!
//! Shared utilities and infrastructure code used by all EthHook services.
//!
//! This crate provides:
//! - Database connection pooling (SQLite via sqlx)
//! - Custom error types
//! - JWT token creation and validation
//! - Password hashing and verification
//! - HMAC signature helpers for webhooks
//! - Structured logging setup

// Module declarations
pub mod auth;
pub mod clickhouse;
pub mod db;
pub mod error;
pub mod logging;
pub mod types;

// Re-export commonly used types
pub use auth::{create_jwt, hash_password, sign_hmac, verify_hmac, verify_jwt, verify_password};
pub use clickhouse::ClickHouseClient;
pub use db::create_pool;
pub use error::{Error, Result};
pub use logging::init_tracing;
pub use types::{BlockchainEvent, DeliveryJob};
