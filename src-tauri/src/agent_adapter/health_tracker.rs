// Health Tracker - EWMA-based health monitoring with circuit breaker

use crate::agent_adapter::types::{CircuitBreakerState, HealthMetrics};
use std::time::{SystemTime, UNIX_EPOCH};

/// Health tracker with EWMA and circuit breaker
#[derive(Clone)]
pub struct HealthTracker {
    success_count: u32,
    error_count: u32,
    latencies: Vec<u64>,
    ewma_score: f32,
    circuit_breaker: CircuitBreaker,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub state: CircuitBreakerState,
    pub failure_threshold: u32,    // Failure count to trigger open
    pub success_threshold: u32,    // Success count in half-open to close
    pub timeout_ms: u64,           // Timeout before testing
    pub failure_count: u32,        // Current failure count
    pub success_count: u32,        // Current success count in half-open
}

impl HealthTracker {
    pub fn new() -> Self {
        Self {
            success_count: 0,
            error_count: 0,
            latencies: Vec::new(),
            ewma_score: 100.0,
            circuit_breaker: CircuitBreaker {
                state: CircuitBreakerState::Closed,
                failure_threshold: 5,
                success_threshold: 3,
                timeout_ms: 60000,
                failure_count: 0,
                success_count: 0,
            },
        }
    }

    /// Record a successful execution
    pub fn record_success(&mut self, latency_ms: u64) {
        self.success_count += 1;
        self.latencies.push(latency_ms);
        self.circuit_breaker.failure_count = 0;

        // Update EWMA
        self.update_ewma(true, latency_ms);

        // Update circuit breaker state
        if matches!(self.circuit_breaker.state, CircuitBreakerState::HalfOpen { .. }) {
            self.circuit_breaker.success_count += 1;
            if self.circuit_breaker.success_count >= self.circuit_breaker.success_threshold {
                self.circuit_breaker.state = CircuitBreakerState::Closed;
                self.circuit_breaker.success_count = 0;
            }
        }
    }

    /// Record a failed execution
    pub fn record_error(&mut self) {
        self.error_count += 1;
        self.circuit_breaker.failure_count += 1;

        // Update EWMA
        self.update_ewma(false, 0);

        // Update circuit breaker state
        match self.circuit_breaker.state {
            CircuitBreakerState::Closed => {
                if self.circuit_breaker.failure_count >= self.circuit_breaker.failure_threshold {
                    self.circuit_breaker.state = CircuitBreakerState::Open {
                        opened_at: self.current_timestamp(),
                    };
                }
            }
            CircuitBreakerState::HalfOpen { .. } => {
                // Any failure in half-open goes back to open
                self.circuit_breaker.state = CircuitBreakerState::Open {
                    opened_at: self.current_timestamp(),
                };
                self.circuit_breaker.success_count = 0;
            }
            CircuitBreakerState::Open { .. } => {
                // Already open, check if timeout expired
                self.check_timeout();
            }
        }
    }

    /// Update EWMA score
    fn update_ewma(&mut self, success: bool, latency_ms: u64) {
        let alpha = 0.1; // Smoothing factor

        let new_value = if success {
            // Success: 100 - latency penalty
            100.0 - (latency_ms as f32 / 1000.0).min(50.0)
        } else {
            // Failure: 0
            0.0
        };

        self.ewma_score = alpha * new_value + (1.0 - alpha) * self.ewma_score;
    }

    /// Check if circuit breaker allows request
    pub fn is_allowed(&mut self) -> bool {
        self.check_timeout();

        match self.circuit_breaker.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open { .. } => false,
            CircuitBreakerState::HalfOpen { .. } => true,
        }
    }

    /// Check if circuit breaker is closed (allows requests)
    pub fn is_circuit_breaker_closed(&self) -> bool {
        matches!(self.circuit_breaker.state, CircuitBreakerState::Closed)
    }

    /// Check timeout and transition to half-open if needed
    fn check_timeout(&mut self) {
        if let CircuitBreakerState::Open { opened_at } = self.circuit_breaker.state {
            let now = self.current_timestamp();
            if now - opened_at >= self.circuit_breaker.timeout_ms {
                self.circuit_breaker.state = CircuitBreakerState::HalfOpen { test_requests: 0 };
                self.circuit_breaker.success_count = 0;
            }
        }
    }

    /// Get current health metrics
    pub fn get_metrics(&self) -> HealthMetrics {
        let avg_latency = if self.latencies.is_empty() {
            0
        } else {
            self.latencies.iter().sum::<u64>() / self.latencies.len() as u64
        };

        let success_rate = if self.success_count + self.error_count == 0 {
            1.0
        } else {
            self.success_count as f32 / (self.success_count + self.error_count) as f32
        };

        HealthMetrics {
            health_score: self.ewma_score,
            success_rate,
            avg_latency_ms: avg_latency,
            error_count: self.error_count,
            last_success_at: if self.success_count > 0 { Some(self.current_timestamp()) } else { None },
            last_error_at: if self.error_count > 0 { Some(self.current_timestamp()) } else { None },
            circuit_breaker_state: self.circuit_breaker.state.clone(),
        }
    }

    /// Reset circuit breaker
    pub fn reset(&mut self) {
        self.circuit_breaker.state = CircuitBreakerState::Closed;
        self.circuit_breaker.failure_count = 0;
        self.circuit_breaker.success_count = 0;
        self.ewma_score = 100.0;
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

impl Default for HealthTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_tracker_initial_state() {
        let tracker = HealthTracker::new();
        let metrics = tracker.get_metrics();
        assert_eq!(metrics.health_score, 100.0);
        assert_eq!(metrics.circuit_breaker_state, CircuitBreakerState::Closed);
    }

    #[test]
    fn test_record_success() {
        let mut tracker = HealthTracker::new();
        tracker.record_success(1000);
        let metrics = tracker.get_metrics();
        assert!(metrics.health_score > 0.0);
        assert_eq!(metrics.error_count, 0);
    }

    #[test]
    fn test_record_error() {
        let mut tracker = HealthTracker::new();
        tracker.record_error();
        let metrics = tracker.get_metrics();
        assert!(metrics.health_score < 100.0);
        assert_eq!(metrics.error_count, 1);
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        let mut tracker = HealthTracker::new();
        tracker.circuit_breaker.failure_threshold = 3;

        for _ in 0..3 {
            tracker.record_error();
        }

        let metrics = tracker.get_metrics();
        assert!(matches!(metrics.circuit_breaker_state, CircuitBreakerState::Open { .. }));
    }

    #[test]
    fn test_circuit_breaker_allows_when_closed() {
        let mut tracker = HealthTracker::new();
        assert!(tracker.is_allowed());
    }

    #[test]
    fn test_circuit_breaker_rejects_when_open() {
        let mut tracker = HealthTracker::new();
        tracker.circuit_breaker.failure_threshold = 1;
        tracker.record_error();

        assert!(!tracker.is_allowed());
    }

    #[test]
    fn test_reset() {
        let mut tracker = HealthTracker::new();
        tracker.record_error();
        tracker.record_error();
        tracker.reset();

        let metrics = tracker.get_metrics();
        assert_eq!(metrics.health_score, 100.0);
        assert_eq!(metrics.circuit_breaker_state, CircuitBreakerState::Closed);
    }
}
