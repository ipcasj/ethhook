/*!
 * Retry Logic with Exponential Backoff
 */

use rand::Rng;
use std::time::Duration;

/// Calculate exponential backoff with jitter
/// 
/// Formula: min(base * 2^attempt, max) + jitter
/// Jitter: ±20% randomness to prevent thundering herd
/// 
/// # Arguments
/// 
/// * `attempt` - Current attempt number (0-indexed)
/// * `base_delay_secs` - Base delay in seconds (e.g., 2)
/// * `max_delay_secs` - Maximum delay in seconds (e.g., 60)
/// 
/// # Returns
/// 
/// Duration to wait before next attempt
pub fn calculate_backoff(attempt: u32, base_delay_secs: u64, max_delay_secs: u64) -> Duration {
    // Exponential: base * 2^attempt
    let exponential_delay = base_delay_secs.saturating_mul(2u64.saturating_pow(attempt));
    
    // Cap at max
    let capped_delay = exponential_delay.min(max_delay_secs);
    
    // Add jitter: ±20%
    let mut rng = rand::thread_rng();
    let jitter_factor = rng.gen_range(0.8..1.2);
    let delay_with_jitter = (capped_delay as f64 * jitter_factor) as u64;
    
    Duration::from_secs(delay_with_jitter)
}

/// Check if error is retryable
/// 
/// # Returns
/// 
/// * `true` - Transient error, should retry
/// * `false` - Permanent error, don't retry
pub fn is_retryable_error(status: Option<u16>) -> bool {
    match status {
        // No status (network error, timeout) - retry
        None => true,
        
        // 2xx - success, don't retry
        Some(status) if status >= 200 && status < 300 => false,
        
        // 4xx - client error
        Some(400) => false, // Bad Request - permanent
        Some(401) => false, // Unauthorized - permanent
        Some(403) => false, // Forbidden - permanent
        Some(404) => false, // Not Found - permanent
        Some(405) => false, // Method Not Allowed - permanent
        Some(410) => false, // Gone - permanent
        Some(429) => true,  // Too Many Requests - retry after backoff
        Some(status) if status >= 400 && status < 500 => false, // Other 4xx - permanent
        
        // 5xx - server error, retry
        Some(status) if status >= 500 && status < 600 => true,
        
        // Unknown status - retry to be safe
        Some(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_backoff() {
        // Attempt 0: 2 seconds
        let delay0 = calculate_backoff(0, 2, 60);
        assert!(delay0.as_secs() >= 1 && delay0.as_secs() <= 3); // ~2s ± 20%
        
        // Attempt 1: 4 seconds
        let delay1 = calculate_backoff(1, 2, 60);
        assert!(delay1.as_secs() >= 3 && delay1.as_secs() <= 5); // ~4s ± 20%
        
        // Attempt 2: 8 seconds
        let delay2 = calculate_backoff(2, 2, 60);
        assert!(delay2.as_secs() >= 6 && delay2.as_secs() <= 10); // ~8s ± 20%
        
        // High attempt: capped at max (60s)
        let delay_high = calculate_backoff(10, 2, 60);
        assert!(delay_high.as_secs() >= 48 && delay_high.as_secs() <= 72); // ~60s ± 20%
    }

    #[test]
    fn test_is_retryable_error() {
        // Network errors (no status) - retry
        assert!(is_retryable_error(None));
        
        // 2xx success - don't retry
        assert!(!is_retryable_error(Some(200)));
        assert!(!is_retryable_error(Some(201)));
        
        // 4xx client errors - mostly don't retry
        assert!(!is_retryable_error(Some(400))); // Bad Request
        assert!(!is_retryable_error(Some(401))); // Unauthorized
        assert!(!is_retryable_error(Some(403))); // Forbidden
        assert!(!is_retryable_error(Some(404))); // Not Found
        assert!(is_retryable_error(Some(429)));  // Too Many Requests - retry
        
        // 5xx server errors - retry
        assert!(is_retryable_error(Some(500))); // Internal Server Error
        assert!(is_retryable_error(Some(502))); // Bad Gateway
        assert!(is_retryable_error(Some(503))); // Service Unavailable
        assert!(is_retryable_error(Some(504))); // Gateway Timeout
    }
}
