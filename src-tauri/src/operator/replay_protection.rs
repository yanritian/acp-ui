// Replay Protection Module
// Prevents replay attacks using nonces and timestamps

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Mutex;

/// Replay Protection Configuration
#[derive(Debug, Clone)]
pub struct ReplayProtectionConfig {
    pub enabled: bool,
    pub max_nonce_age_seconds: i64,
    pub max_request_age_seconds: i64,
    pub nonce_cache_size: usize,
}

impl Default for ReplayProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_nonce_age_seconds: 300, // 5 minutes
            max_request_age_seconds: 300, // 5 minutes
            nonce_cache_size: 10000,
        }
    }
}

/// Replay Protection Manager
pub struct ReplayProtectionManager {
    config: ReplayProtectionConfig,
    used_nonces: Mutex<HashSet<String>>,
    last_cleanup: Mutex<DateTime<Utc>>,
}

impl ReplayProtectionManager {
    pub fn new(config: ReplayProtectionConfig) -> Self {
        Self {
            config,
            used_nonces: Mutex::new(HashSet::new()),
            last_cleanup: Mutex::new(Utc::now()),
        }
    }

    /// Validate a request for replay attacks
    pub fn validate_request(
        &self,
        nonce: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<(), ReplayAttackError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check timestamp
        let now = Utc::now();
        let age = now.signed_duration_since(timestamp);

        if age.num_seconds() > self.config.max_request_age_seconds {
            return Err(ReplayAttackError::RequestTooOld {
                age_seconds: age.num_seconds(),
                max_age_seconds: self.config.max_request_age_seconds,
            });
        }

        if age.num_seconds() < -self.config.max_request_age_seconds {
            return Err(ReplayAttackError::RequestFromFuture {
                age_seconds: -age.num_seconds(),
                max_age_seconds: self.config.max_request_age_seconds,
            });
        }

        // Check nonce
        let mut used_nonces = self.used_nonces.lock().map_err(|_| ReplayAttackError::LockError)?;

        if used_nonces.contains(nonce) {
            return Err(ReplayAttackError::NonceReused {
                nonce: nonce.to_string(),
            });
        }

        // Add nonce to cache
        used_nonces.insert(nonce.to_string());

        // Cleanup old nonces if needed
        if used_nonces.len() > self.config.nonce_cache_size {
            drop(used_nonces);
            self.cleanup_old_nonces();
        }

        Ok(())
    }

    /// Generate a unique nonce
    pub fn generate_nonce(&self) -> String {
        let timestamp = Utc::now().timestamp_nanos();
        let random = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_string().as_bytes());
        hasher.update(random.to_string().as_bytes());
        let result = hasher.finalize();

        hex::encode(result)
    }

    /// Cleanup old nonces
    fn cleanup_old_nonces(&self) {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        let now = Utc::now();

        // Only cleanup every minute
        if now.signed_duration_since(*last_cleanup).num_seconds() < 60 {
            return;
        }

        let mut used_nonces = self.used_nonces.lock().unwrap();

        // Clear half of the cache to prevent memory issues
        if used_nonces.len() > self.config.nonce_cache_size {
            let to_remove: Vec<String> = used_nonces.iter().take(self.config.nonce_cache_size / 2).cloned().collect();
            for nonce in to_remove {
                used_nonces.remove(&nonce);
            }
        }

        *last_cleanup = now;
    }

    /// Get statistics
    pub fn get_statistics(&self) -> ReplayProtectionStats {
        let used_nonces = self.used_nonces.lock().unwrap();
        ReplayProtectionStats {
            enabled: self.config.enabled,
            cached_nonces: used_nonces.len(),
            max_cache_size: self.config.nonce_cache_size,
        }
    }
}

impl Default for ReplayProtectionManager {
    fn default() -> Self {
        Self::new(ReplayProtectionConfig::default())
    }
}

/// Replay Protection Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayProtectionStats {
    pub enabled: bool,
    pub cached_nonces: usize,
    pub max_cache_size: usize,
}

/// Replay Attack Errors
#[derive(Debug)]
pub enum ReplayAttackError {
    RequestTooOld {
        age_seconds: i64,
        max_age_seconds: i64,
    },
    RequestFromFuture {
        age_seconds: i64,
        max_age_seconds: i64,
    },
    NonceReused {
        nonce: String,
    },
    LockError,
}

impl std::fmt::Display for ReplayAttackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestTooOld { age_seconds, max_age_seconds } => {
                write!(
                    f,
                    "Request too old: {} seconds (max: {})",
                    age_seconds, max_age_seconds
                )
            }
            Self::RequestFromFuture { age_seconds, max_age_seconds } => {
                write!(
                    f,
                    "Request from future: {} seconds (max: {})",
                    age_seconds, max_age_seconds
                )
            }
            Self::NonceReused { nonce } => {
                write!(f, "Nonce reused: {}", nonce)
            }
            Self::LockError => write!(f, "Failed to acquire lock"),
        }
    }
}

impl std::error::Error for ReplayAttackError {}

/// Request Signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSignature {
    pub nonce: String,
    pub timestamp: DateTime<Utc>,
    pub signature: String,
}

impl RequestSignature {
    pub fn new(nonce: String, timestamp: DateTime<Utc>, secret: &str, payload: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(nonce.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        hasher.update(secret.as_bytes());
        hasher.update(payload.as_bytes());
        let signature = hex::encode(hasher.finalize());

        Self {
            nonce,
            timestamp,
            signature,
        }
    }

    pub fn verify(&self, secret: &str, payload: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(self.nonce.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(secret.as_bytes());
        hasher.update(payload.as_bytes());
        let expected_signature = hex::encode(hasher.finalize());

        self.signature == expected_signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_protection_nonce_validation() {
        let manager = ReplayProtectionManager::new(ReplayProtectionConfig {
            enabled: true,
            max_nonce_age_seconds: 300,
            max_request_age_seconds: 300,
            nonce_cache_size: 100,
        });

        let nonce1 = manager.generate_nonce();
        let timestamp = Utc::now();

        // First use should succeed
        assert!(manager.validate_request(&nonce1, timestamp).is_ok());

        // Second use should fail (nonce reused)
        assert!(manager.validate_request(&nonce1, timestamp).is_err());
    }

    #[test]
    fn test_replay_protection_old_request() {
        let manager = ReplayProtectionManager::new(ReplayProtectionConfig {
            enabled: true,
            max_nonce_age_seconds: 300,
            max_request_age_seconds: 60, // 1 minute
            nonce_cache_size: 100,
        });

        let nonce = manager.generate_nonce();
        let old_timestamp = Utc::now() - Duration::seconds(120); // 2 minutes ago

        // Should fail (request too old)
        assert!(manager.validate_request(&nonce, old_timestamp).is_err());
    }

    #[test]
    fn test_replay_protection_future_request() {
        let manager = ReplayProtectionManager::new(ReplayProtectionConfig {
            enabled: true,
            max_nonce_age_seconds: 300,
            max_request_age_seconds: 60,
            nonce_cache_size: 100,
        });

        let nonce = manager.generate_nonce();
        let future_timestamp = Utc::now() + Duration::seconds(120);

        // Should fail (request from future)
        assert!(manager.validate_request(&nonce, future_timestamp).is_err());
    }

    #[test]
    fn test_request_signature() {
        let nonce = "test_nonce".to_string();
        let timestamp = Utc::now();
        let secret = "test_secret";
        let payload = "test_payload";

        let signature = RequestSignature::new(nonce.clone(), timestamp, secret, payload);

        // Should verify correctly
        assert!(signature.verify(secret, payload));

        // Should fail with wrong secret
        assert!(!signature.verify("wrong_secret", payload));

        // Should fail with wrong payload
        assert!(!signature.verify(secret, "wrong_payload"));
    }

    #[test]
    fn test_replay_protection_disabled() {
        let manager = ReplayProtectionManager::new(ReplayProtectionConfig {
            enabled: false,
            max_nonce_age_seconds: 300,
            max_request_age_seconds: 300,
            nonce_cache_size: 100,
        });

        let nonce = "test_nonce".to_string();
        let timestamp = Utc::now();

        // Should succeed even with duplicate nonce (disabled)
        assert!(manager.validate_request(&nonce, timestamp).is_ok());
        assert!(manager.validate_request(&nonce, timestamp).is_ok());
    }

    #[test]
    fn test_replay_protection_statistics() {
        let manager = ReplayProtectionManager::new(ReplayProtectionConfig::default());

        let stats = manager.get_statistics();
        assert!(stats.enabled);
        assert_eq!(stats.cached_nonces, 0);
        assert_eq!(stats.max_cache_size, 10000);

        // Add some nonces
        for _ in 0..5 {
            let nonce = manager.generate_nonce();
            let _ = manager.validate_request(&nonce, Utc::now());
        }

        let stats = manager.get_statistics();
        assert_eq!(stats.cached_nonces, 5);
    }
}