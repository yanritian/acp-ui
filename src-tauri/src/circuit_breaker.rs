//! Circuit Breaker - Three-state failure protection
//!
//! Implements Closed → Open → HalfOpen pattern for agent failure handling

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Failing, rejecting requests
    HalfOpen,   // Testing recovery
}

impl CircuitState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CircuitState::Closed => "closed",
            CircuitState::Open => "open",
            CircuitState::HalfOpen => "half_open",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "closed" => Ok(CircuitState::Closed),
            "open" => Ok(CircuitState::Open),
            "half_open" | "halfopen" => Ok(CircuitState::HalfOpen),
            other => Err(format!("Unknown circuit state: {}", other)),
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,      // Failures before opening
    pub success_threshold: u32,      // Successes to close from half-open
    pub timeout_ms: u64,             // Time before half-open
    pub half_open_max_calls: u32,    // Max calls in half-open state
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_ms: 30000, // 30 seconds
            half_open_max_calls: 5,
        }
    }
}

/// Circuit breaker record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerRecord {
    pub id: String,
    pub target: String,              // Agent or service name
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub cool_down_until: Option<DateTime<Utc>>,
    pub config: CircuitBreakerConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Circuit breaker manager
pub struct CircuitBreakerManager {
    breakers: Arc<Mutex<HashMap<String, CircuitBreakerRecord>>>,
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(Mutex::new(HashMap::new())),
            default_config: CircuitBreakerConfig::default(),
        }
    }

    /// Get or create circuit breaker for a target
    pub fn get_breaker(&self, target: &str) -> CircuitBreakerRecord {
        let breakers = self.breakers.lock().unwrap();

        if let Some(breaker) = breakers.get(target) {
            breaker.clone()
        } else {
            // Create new breaker
            CircuitBreakerRecord {
                id: uuid::Uuid::new_v4().to_string(),
                target: target.to_string(),
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_at: None,
                cool_down_until: None,
                config: self.default_config.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }
    }

    /// Check if requests are allowed
    pub fn is_allowed(&self, target: &str) -> bool {
        let breaker = self.get_breaker(target);

        match breaker.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if cool down has passed
                if let Some(cool_down) = breaker.cool_down_until {
                    if Utc::now() >= cool_down {
                        // Transition to half-open
                        self.transition_to_half_open(target);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited calls in half-open
                breaker.success_count < breaker.config.half_open_max_calls
            }
        }
    }

    /// Record a success
    pub fn record_success(&self, target: &str) {
        let mut breakers = self.breakers.lock().unwrap();

        if let Some(breaker) = breakers.get_mut(target) {
            breaker.success_count += 1;
            breaker.updated_at = Utc::now();

            // In half-open, check if enough successes to close
            if breaker.state == CircuitState::HalfOpen
                && breaker.success_count >= breaker.config.success_threshold {
                    breaker.state = CircuitState::Closed;
                    breaker.failure_count = 0;
                    breaker.success_count = 0;
                    breaker.cool_down_until = None;
            }
        }
    }

    /// Record a failure
    pub fn record_failure(&self, target: &str) {
        let mut breakers = self.breakers.lock().unwrap();

        if let Some(breaker) = breakers.get_mut(target) {
            breaker.failure_count += 1;
            breaker.last_failure_at = Some(Utc::now());
            breaker.updated_at = Utc::now();

            // In closed state, check if threshold reached
            if breaker.state == CircuitState::Closed
                && breaker.failure_count >= breaker.config.failure_threshold {
                    breaker.state = CircuitState::Open;
                    breaker.cool_down_until = Some(
                        Utc::now() + chrono::Duration::milliseconds(breaker.config.timeout_ms as i64)
                    );
            }

            // In half-open, any failure returns to open
            if breaker.state == CircuitState::HalfOpen {
                breaker.state = CircuitState::Open;
                breaker.success_count = 0;
                breaker.cool_down_until = Some(
                    Utc::now() + chrono::Duration::milliseconds(breaker.config.timeout_ms as i64)
                );
            }
        }
    }

    /// Transition to half-open state
    fn transition_to_half_open(&self, target: &str) {
        let mut breakers = self.breakers.lock().unwrap();

        if let Some(breaker) = breakers.get_mut(target) {
            breaker.state = CircuitState::HalfOpen;
            breaker.success_count = 0;
            breaker.updated_at = Utc::now();
        }
    }

    /// Force reset a circuit breaker
    pub fn reset(&self, target: &str) {
        let mut breakers = self.breakers.lock().unwrap();

        if let Some(breaker) = breakers.get_mut(target) {
            breaker.state = CircuitState::Closed;
            breaker.failure_count = 0;
            breaker.success_count = 0;
            breaker.cool_down_until = None;
            breaker.updated_at = Utc::now();
        }
    }

    /// Get all circuit breaker states
    pub fn get_all_states(&self) -> Vec<CircuitBreakerRecord> {
        let breakers = self.breakers.lock().unwrap();
        breakers.values().cloned().collect()
    }

    /// Count open circuit breakers
    pub fn count_open(&self) -> u32 {
        let breakers = self.breakers.lock().unwrap();
        breakers.values()
            .filter(|b| b.state == CircuitState::Open)
            .count() as u32
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}