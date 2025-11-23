//! Error types for EthHook
//!
//! Custom error enum that wraps all possible error types in the application.
//! Similar to Java exception hierarchy, but using Rust's Result type.

use thiserror::Error;

/// Main error type for EthHook operations
///
/// Java equivalent: Custom exception hierarchy
/// ```java
/// class EthHookException extends Exception { }
/// class DatabaseException extends EthHookException { }
/// ```
#[derive(Debug, Error)]
pub enum Error {
    /// Database errors (SQLite via sqlx)
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Authentication errors (JWT, bcrypt)
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid JWT token
    #[error("Invalid JWT token: {0}")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),

    /// Password hashing errors
    #[error("Password hashing error: {0}")]
    PasswordHash(#[from] bcrypt::BcryptError),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// External service errors (HTTP, RPC providers)
    #[error("External service error: {0}")]
    External(String),

    /// Generic errors
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Result type alias for EthHook operations
///
/// Java equivalent: try/catch but as a type
/// ```java
/// try {
///     return someOperation();
/// } catch (Exception e) {
///     throw new EthHookException(e);
/// }
/// ```
///
/// Rust:
/// ```rust
/// fn some_operation() -> Result<T> {
///     Ok(value)
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Validation("Email is required".to_string());
        assert_eq!(err.to_string(), "Validation error: Email is required");
    }

    #[test]
    fn test_error_conversion() {
        // Test that sqlx errors convert properly
        let sql_err = sqlx::Error::RowNotFound;
        let our_err: Error = sql_err.into();
        assert!(matches!(our_err, Error::Database(_)));
    }
}
