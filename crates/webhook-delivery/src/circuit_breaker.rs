/*!
 * Circuit Breaker for Endpoint Health Tracking
 * 
 * Prevents hammering unhealthy endpoints with requests.
 */

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};
use uuid::Uuid;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - endpoint is healthy
    Closed,
    /// Too many failures - waiting before retry
    Open,
    /// Testing if endpoint recovered
    HalfOpen,
}

/// Health tracking for an endpoint
#[derive(Debug, Clone)]
struct EndpointHealth {
    /// Current circuit breaker state
    state: CircuitState,
    /// Number of consecutive failures
    consecutive_failures: u32,
    /// Time when circuit opened
    opened_at: Option<Instant>,
    /// Last successful delivery
    last_success: Option<Instant>,
}

impl EndpointHealth {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            opened_at: None,
            last_success: None,
        }
    }
}

/// Circuit breaker manager for all endpoints
pub struct CircuitBreakerManager {
    /// Health state per endpoint
    endpoints: Arc<Mutex<HashMap<Uuid, EndpointHealth>>>,
    /// Failure threshold before opening circuit
    threshold: u32,
    /// How long to wait before testing recovery
    timeout: Duration,
}

impl CircuitBreakerManager {
    /// Create new circuit breaker manager
    /// 
    /// # Arguments
    /// 
    /// * `threshold` - Number of failures before opening circuit (e.g., 5)
    /// * `timeout` - How long to wait before testing recovery (e.g., 60 seconds)
    pub fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            endpoints: Arc::new(Mutex::new(HashMap::new())),
            threshold,
            timeout,
        }
    }
    
    /// Check if request should be allowed for this endpoint
    /// 
    /// # Returns
    /// 
    /// * `true` - Request allowed (circuit closed or half-open)
    /// * `false` - Request blocked (circuit open, still in timeout)
    pub async fn should_allow_request(&self, endpoint_id: Uuid) -> bool {
        let mut endpoints = self.endpoints.lock().await;
        
        let health = endpoints
            .entry(endpoint_id)
            .or_insert_with(EndpointHealth::new);
        
        match health.state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(opened_at) = health.opened_at {
                    if opened_at.elapsed() >= self.timeout {
                        // Transition to half-open
                        debug!(
                            "Circuit breaker transitioning to HALF-OPEN for endpoint {}",
                            endpoint_id
                        );
                        health.state = CircuitState::HalfOpen;
                        true
                    } else {
                        // Still in timeout
                        false
                    }
                } else {
                    // No opened_at timestamp? Allow request
                    true
                }
            }
        }
    }
    
    /// Record successful delivery
    pub async fn record_success(&self, endpoint_id: Uuid) {
        let mut endpoints = self.endpoints.lock().await;
        
        let health = endpoints
            .entry(endpoint_id)
            .or_insert_with(EndpointHealth::new);
        
        // Reset failure count
        health.consecutive_failures = 0;
        health.last_success = Some(Instant::now());
        
        // Close circuit if it was open/half-open
        if health.state != CircuitState::Closed {
            debug!(
                "Circuit breaker closing for endpoint {} (success after failures)",
                endpoint_id
            );
            health.state = CircuitState::Closed;
            health.opened_at = None;
        }
    }
    
    /// Record failed delivery
    pub async fn record_failure(&self, endpoint_id: Uuid) {
        let mut endpoints = self.endpoints.lock().await;
        
        let health = endpoints
            .entry(endpoint_id)
            .or_insert_with(EndpointHealth::new);
        
        health.consecutive_failures += 1;
        
        // Check if we should open the circuit
        if health.consecutive_failures >= self.threshold {
            match health.state {
                CircuitState::Closed => {
                    warn!(
                        "Circuit breaker OPENING for endpoint {} ({} consecutive failures)",
                        endpoint_id, health.consecutive_failures
                    );
                    health.state = CircuitState::Open;
                    health.opened_at = Some(Instant::now());
                }
                CircuitState::HalfOpen => {
                    warn!(
                        "Circuit breaker RE-OPENING for endpoint {} (half-open test failed)",
                        endpoint_id
                    );
                    health.state = CircuitState::Open;
                    health.opened_at = Some(Instant::now());
                }
                CircuitState::Open => {
                    // Already open, update timestamp
                    health.opened_at = Some(Instant::now());
                }
            }
        }
    }
    
    /// Get current state for an endpoint
    pub async fn get_state(&self, endpoint_id: Uuid) -> CircuitState {
        let endpoints = self.endpoints.lock().await;
        
        endpoints
            .get(&endpoint_id)
            .map(|h| h.state)
            .unwrap_or(CircuitState::Closed)
    }
    
    /// Get statistics for monitoring
    pub async fn stats(&self) -> CircuitBreakerStats {
        let endpoints = self.endpoints.lock().await;
        
        let mut closed = 0;
        let mut open = 0;
        let mut half_open = 0;
        
        for health in endpoints.values() {
            match health.state {
                CircuitState::Closed => closed += 1,
                CircuitState::Open => open += 1,
                CircuitState::HalfOpen => half_open += 1,
            }
        }
        
        CircuitBreakerStats {
            total_endpoints: endpoints.len(),
            closed,
            open,
            half_open,
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub total_endpoints: usize,
    pub closed: usize,
    pub open: usize,
    pub half_open: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_initial_state() {
        let manager = CircuitBreakerManager::new(3, Duration::from_secs(60));
        let endpoint_id = Uuid::new_v4();
        
        // Initial state should be closed
        assert_eq!(manager.get_state(endpoint_id).await, CircuitState::Closed);
        assert!(manager.should_allow_request(endpoint_id).await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let manager = CircuitBreakerManager::new(3, Duration::from_secs(60));
        let endpoint_id = Uuid::new_v4();
        
        // Record 3 failures
        manager.record_failure(endpoint_id).await;
        manager.record_failure(endpoint_id).await;
        manager.record_failure(endpoint_id).await;
        
        // Circuit should be open
        assert_eq!(manager.get_state(endpoint_id).await, CircuitState::Open);
        assert!(!manager.should_allow_request(endpoint_id).await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_on_success() {
        let manager = CircuitBreakerManager::new(3, Duration::from_secs(60));
        let endpoint_id = Uuid::new_v4();
        
        // Open circuit
        manager.record_failure(endpoint_id).await;
        manager.record_failure(endpoint_id).await;
        manager.record_failure(endpoint_id).await;
        assert_eq!(manager.get_state(endpoint_id).await, CircuitState::Open);
        
        // Manually transition to half-open (simulate timeout elapsed)
        {
            let mut endpoints = manager.endpoints.lock().await;
            endpoints.get_mut(&endpoint_id).unwrap().state = CircuitState::HalfOpen;
        }
        
        // Record success
        manager.record_success(endpoint_id).await;
        
        // Circuit should close
        assert_eq!(manager.get_state(endpoint_id).await, CircuitState::Closed);
        assert!(manager.should_allow_request(endpoint_id).await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let manager = CircuitBreakerManager::new(3, Duration::from_secs(60));
        
        let endpoint1 = Uuid::new_v4();
        let endpoint2 = Uuid::new_v4();
        let endpoint3 = Uuid::new_v4();
        
        // endpoint1: closed (healthy)
        manager.record_success(endpoint1).await;
        
        // endpoint2: open (unhealthy)
        manager.record_failure(endpoint2).await;
        manager.record_failure(endpoint2).await;
        manager.record_failure(endpoint2).await;
        
        // endpoint3: closed (healthy)
        manager.record_success(endpoint3).await;
        
        let stats = manager.stats().await;
        assert_eq!(stats.total_endpoints, 3);
        assert_eq!(stats.closed, 2);
        assert_eq!(stats.open, 1);
        assert_eq!(stats.half_open, 0);
    }
}
