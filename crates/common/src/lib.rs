//! # EthHook Common Library
//!
//! Shared utilities and infrastructure code used by all EthHook services.
//!
//! This crate provides:
//! - Database connection pooling (PostgreSQL via sqlx)
//! - Redis client with Stream and Queue helpers
//! - Custom error types
//! - JWT token creation and validation
//! - Password hashing and verification
//! - HMAC signature helpers for webhooks
//! - Structured logging setup

// Module declarations
pub mod db;
pub mod redis_client;
pub mod error;
pub mod auth;
pub mod logging;

// Re-export commonly used types
pub use db::create_pool;
pub use redis_client::RedisClient;
pub use error::{Error, Result};
pub use auth::{create_jwt, verify_jwt, hash_password, verify_password, sign_hmac, verify_hmac};
pub use logging::init_tracing;
