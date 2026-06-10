//! Queen Lease - ZooKeeper-inspired lease election for swarm coordination
//!
//! Implements lease-based election for Queen Agent (coordinator) role:
//! 1. Queen obtains a lease with TTL (default 30 seconds)
//! 2. Queen must renew lease periodically (default every 10 seconds)
//! 3. If lease expires, new election is triggered
//! 4. Workers only accept commands from valid lease-holder
//!
//! This prevents split-brain scenarios and ensures single Queen at any time.

use crate::swarm_types::{WorkerId, HealthStatus};
use crate::swarm_adapters::{WorkerCapabilities, SwarmError, InstantWrapper, now_ms};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Queen Lease Configuration
// ---------------------------------------------------------------------------

/// Configuration for Queen lease system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueenLeaseConfig {
    /// Lease TTL in seconds (default 30)
    pub ttl_seconds: u64,
    /// Renew interval in seconds (default 10)
    pub renew_interval_seconds: u64,
    /// Grace period before election after lease expires (default 5)
    pub election_grace_period_seconds: u64,
    /// Minimum workers required for election (default 1)
    pub min_workers_for_election: u32,
    /// Enable adaptive queen upgrade based on task complexity
    pub enable_adaptive_upgrade: bool,
}

impl Default for QueenLeaseConfig {
    fn default() -> Self {
        Self {
            ttl_seconds: 30,
            renew_interval_seconds: 10,
            election_grace_period_seconds: 5,
            min_workers_for_election: 1,
            enable_adaptive_upgrade: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Queen Lease
// ---------------------------------------------------------------------------

/// Queen lease - grants exclusive coordination rights
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueenLease {
    /// ID of the lease (unique identifier)
    pub lease_id: String,
    /// Worker holding this lease (Queen)
    pub queen_id: WorkerId,
    /// When the lease was granted (UNIX ms)
    pub granted_at: InstantWrapper,
    /// Lease TTL in seconds
    pub ttl_seconds: u64,
    /// When the lease expires (UNIX ms)
    pub expires_at: InstantWrapper,
    /// Last renewal time (UNIX ms)
    pub last_renewed_at: Option<InstantWrapper>,
    /// Number of renewals so far
    pub renewal_count: u32,
    /// Whether lease is currently valid
    pub is_valid: bool,
    /// Reason if lease became invalid
    pub invalid_reason: Option<String>,
    /// Current task complexity being handled (for adaptive upgrade)
    pub current_complexity: Option<u8>,
}

impl QueenLease {
    /// Create a new lease for a Queen
    pub fn new(lease_id: String, queen_id: WorkerId, ttl_seconds: u64) -> Self {
        let granted_ms = now_ms();
        let expires_ms = granted_ms + ttl_seconds * 1000;

        Self {
            lease_id,
            queen_id,
            granted_at: InstantWrapper::from_ms(granted_ms),
            ttl_seconds,
            expires_at: InstantWrapper::from_ms(expires_ms),
            last_renewed_at: None,
            renewal_count: 0,
            is_valid: true,
            invalid_reason: None,
            current_complexity: None,
        }
    }

    /// Check if lease is still valid (compares absolute timestamps)
    pub fn check_validity(&mut self) -> bool {
        let current_ms = now_ms();
        if current_ms >= self.expires_at.timestamp_ms {
            self.is_valid = false;
            self.invalid_reason = Some("Lease expired".to_string());
            return false;
        }
        true
    }

    /// Renew the lease — extends expiry by ttl_seconds from NOW
    pub fn renew(&mut self) -> Result<(), SwarmError> {
        if !self.is_valid {
            return Err(SwarmError::InternalError("Cannot renew invalid lease".to_string()));
        }

        let renewed_ms = now_ms();
        self.expires_at = InstantWrapper::from_ms(renewed_ms + self.ttl_seconds * 1000);
        self.last_renewed_at = Some(InstantWrapper::from_ms(renewed_ms));
        self.renewal_count += 1;

        Ok(())
    }

    /// Invalidate the lease (Queen crashed or stepped down)
    pub fn invalidate(&mut self, reason: String) {
        self.is_valid = false;
        self.invalid_reason = Some(reason);
    }

    /// Get remaining time until expiry (in seconds)
    pub fn remaining_time(&self) -> u64 {
        let current_ms = now_ms();
        self.expires_at
            .timestamp_ms
            .saturating_sub(current_ms)
            / 1000
    }
}

// ---------------------------------------------------------------------------
// Election State
// ---------------------------------------------------------------------------

/// State of Queen election
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElectionState {
    /// No active Queen, election pending
    NoQueen,
    /// Queen has valid lease
    QueenActive,
    /// Queen lease expired, election in progress
    ElectionInProgress,
    /// Multiple candidates competing
    Competing,
    /// Election completed, new Queen selected
    ElectionComplete,
}

/// Candidate for Queen election
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueenCandidate {
    /// Worker ID
    pub worker_id: WorkerId,
    /// Worker capabilities
    pub capabilities: WorkerCapabilities,
    /// Election score (based on capabilities and health)
    pub score: u32,
    /// Whether worker is healthy
    pub is_healthy: bool,
    /// Whether worker wants to be Queen
    pub wants_queen: bool,
}

impl QueenCandidate {
    /// Calculate election score
    pub fn calculate_score(&self) -> u32 {
        if !self.is_healthy || !self.wants_queen {
            return 0;
        }

        // Score based on:
        // - Max complexity (0-50 points)
        // - Number of capabilities (0-30 points)
        // - Supports streaming (+10)
        // - Supports cancel (+10)

        let complexity_score = self.capabilities.max_complexity as u32 * 10;
        let capability_score = self.capabilities.capabilities.len() as u32 * 3;
        let streaming_score = if self.capabilities.supports_streaming { 10 } else { 0 };
        let cancel_score = if self.capabilities.supports_cancel { 10 } else { 0 };

        complexity_score + capability_score + streaming_score + cancel_score
    }
}

// ---------------------------------------------------------------------------
// Queen Election Manager
// ---------------------------------------------------------------------------

/// Manager for Queen election and lease management
pub struct QueenElectionManager {
    /// Current lease (if any)
    current_lease: Arc<Mutex<Option<QueenLease>>>,
    /// Election state
    election_state: Arc<Mutex<ElectionState>>,
    /// All registered workers
    workers: Arc<Mutex<HashMap<WorkerId, WorkerCapabilities>>>,
    /// Worker health status
    worker_health: Arc<Mutex<HashMap<WorkerId, HealthStatus>>>,
    /// Configuration
    config: QueenLeaseConfig,
    /// Last election time (UNIX ms timestamp)
    last_election: Arc<Mutex<Option<u64>>>,
    /// Election candidates
    candidates: Arc<Mutex<Vec<QueenCandidate>>>,
}

impl QueenElectionManager {
    /// Create a new election manager
    pub fn new(config: QueenLeaseConfig) -> Self {
        Self {
            current_lease: Arc::new(Mutex::new(None)),
            election_state: Arc::new(Mutex::new(ElectionState::NoQueen)),
            workers: Arc::new(Mutex::new(HashMap::new())),
            worker_health: Arc::new(Mutex::new(HashMap::new())),
            config,
            last_election: Arc::new(Mutex::new(None)),
            candidates: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a worker for potential Queen role
    pub fn register_worker(&self, worker_id: WorkerId, capabilities: WorkerCapabilities) {
        self.workers.lock().unwrap().insert(worker_id.clone(), capabilities);
        self.worker_health.lock().unwrap().insert(worker_id, HealthStatus::Offline);
    }

    /// Update worker health status
    pub fn update_health(&self, worker_id: &WorkerId, health: HealthStatus) {
        self.worker_health.lock().unwrap().insert(worker_id.clone(), health);
    }

    /// Start election process
    pub fn start_election(&self) -> Result<WorkerId, SwarmError> {
        // Check if we have enough workers
        let worker_count = self.workers.lock().unwrap().len() as u32;
        if worker_count < self.config.min_workers_for_election {
            return Err(SwarmError::InternalError(
                format!("Not enough workers for election: {} < {}", worker_count, self.config.min_workers_for_election)
            ));
        }

        // Update state
        *self.election_state.lock().unwrap() = ElectionState::ElectionInProgress;

        // Gather candidates
        self.gather_candidates();

        // Run election
        let winner = self.run_election()?;

        // Create lease for winner
        let lease = QueenLease::new(
            format!("lease-{}", now_ms()),
            winner.clone(),
            self.config.ttl_seconds,
        );

        *self.current_lease.lock().unwrap() = Some(lease);
        *self.election_state.lock().unwrap() = ElectionState::QueenActive;
        *self.last_election.lock().unwrap() = Some(now_ms());

        println!("✅ Queen elected: {}", winner);
        Ok(winner)
    }

    /// Gather candidates for election
    fn gather_candidates(&self) {
        // Clone data from each lock separately to avoid holding two locks simultaneously
        // (which risks deadlock if another thread acquires them in a different order).
        let workers_snapshot: Vec<(WorkerId, WorkerCapabilities)> = {
            let workers = self.workers.lock().unwrap();
            workers.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        };

        let health_snapshot: HashMap<WorkerId, HealthStatus> = {
            let health = self.worker_health.lock().unwrap();
            health.clone()
        };

        let mut candidates = Vec::new();

        for (worker_id, capabilities) in &workers_snapshot {
            let worker_health = health_snapshot.get(worker_id).copied().unwrap_or(HealthStatus::Offline);
            let is_healthy = matches!(worker_health, HealthStatus::Healthy | HealthStatus::Busy);

            let candidate = QueenCandidate {
                worker_id: worker_id.clone(),
                capabilities: capabilities.clone(),
                score: 0, // Calculated below
                is_healthy,
                wants_queen: true, // Assume all want to be Queen for now
            };

            candidates.push(candidate);
        }

        // Calculate scores
        for candidate in &mut candidates {
            candidate.score = candidate.calculate_score();
        }

        // Sort by score (highest first)
        candidates.sort_by(|a, b| b.score.cmp(&a.score));

        *self.candidates.lock().unwrap() = candidates;
    }

    /// Run election and select winner
    fn run_election(&self) -> Result<WorkerId, SwarmError> {
        let candidates = self.candidates.lock().unwrap();

        if candidates.is_empty() {
            return Err(SwarmError::InternalError("No candidates for election".to_string()));
        }

        // Top candidate wins
        let winner = candidates.first()
            .filter(|c| c.score > 0)
            .map(|c| c.worker_id.clone())
            .ok_or_else(|| SwarmError::InternalError("No valid candidate with positive score".to_string()))?;

        Ok(winner)
    }

    /// Check lease validity and trigger election if expired
    pub fn check_and_renew(&self) -> Result<Option<WorkerId>, SwarmError> {
        let mut lease_guard = self.current_lease.lock().unwrap();

        if let Some(ref mut lease) = *lease_guard {
            // Try to renew if still valid
            if lease.check_validity() {
                lease.renew()?;
                return Ok(None); // No new election
            }
        }

        // Lease expired - need new election
        drop(lease_guard); // Release lock before election

        *self.election_state.lock().unwrap() = ElectionState::NoQueen;
        let new_queen = self.start_election()?;
        Ok(Some(new_queen))
    }

    /// Get current Queen (if lease valid)
    pub fn get_current_queen(&self) -> Option<WorkerId> {
        let lease = self.current_lease.lock().unwrap();
        lease.as_ref()
            .filter(|l| l.is_valid)
            .map(|l| l.queen_id.clone())
    }

    /// Get election state
    pub fn get_state(&self) -> ElectionState {
        *self.election_state.lock().unwrap()
    }

    /// Get current lease info
    pub fn get_lease_info(&self) -> Option<QueenLease> {
        self.current_lease.lock().unwrap().clone()
    }

    /// Check if a command from this worker should be accepted
    pub fn should_accept_command(&self, from_worker: &WorkerId) -> bool {
        let queen = self.get_current_queen();
        queen.as_ref() == Some(from_worker)
    }

    /// Adaptive Queen upgrade - switch to stronger Queen for complex tasks
    pub fn adaptive_upgrade(&self, required_complexity: u8) -> Result<Option<WorkerId>, SwarmError> {
        if !self.config.enable_adaptive_upgrade {
            return Ok(None);
        }

        let current_queen = self.get_current_queen();
        if current_queen.is_none() {
            return Ok(None);
        }

        let queen_id = current_queen.unwrap();

        // Find better queen candidate
        let better_queen = {
            let workers = self.workers.lock().unwrap();

            let current_max = workers.get(&queen_id)
                .map(|c| c.max_complexity)
                .unwrap_or(0);

            // Check if current Queen can handle this complexity
            if current_max >= required_complexity {
                return Ok(None); // No upgrade needed
            }

            // Find a better Queen
            workers.iter()
                .filter(|(_, c)| c.max_complexity >= required_complexity && c.max_complexity > current_max)
                .max_by_key(|(_, c)| c.max_complexity)
                .map(|(id, _)| id.clone())
        }; // workers lock released here

        if let Some(_new_queen_id) = better_queen {
            // Invalidate current lease
            {
                let mut lease = self.current_lease.lock().unwrap();
                if let Some(ref mut l) = *lease {
                    l.invalidate(format!("Adaptive upgrade: need complexity {}", required_complexity));
                }
            }

            // Start new election (better queen will likely win due to higher complexity)
            let winner = self.start_election()?;
            println!("✅ Adaptive Queen upgrade: {} -> {}", queen_id, winner);
            return Ok(Some(winner));
        }

        Ok(None)
    }

    /// Get all candidates with scores
    pub fn get_candidates(&self) -> Vec<QueenCandidate> {
        self.candidates.lock().unwrap().clone()
    }
}

impl Default for QueenElectionManager {
    fn default() -> Self {
        Self::new(QueenLeaseConfig::default())
    }
}

// ---------------------------------------------------------------------------
// Anti-Split-Brain Validator
// ---------------------------------------------------------------------------

/// Validator to prevent split-brain scenarios
///
/// Workers should only accept commands from the valid lease-holder.
/// This validator checks lease validity before processing commands.
pub struct AntiSplitBrainValidator {
    /// Reference to election manager
    election_manager: Arc<Mutex<QueenElectionManager>>,
}

impl AntiSplitBrainValidator {
    pub fn new(election_manager: Arc<Mutex<QueenElectionManager>>) -> Self {
        Self { election_manager }
    }

    /// Validate if a command should be accepted
    pub fn validate_command(&self, sender_id: &WorkerId, command_type: &str) -> ValidationResult {
        let manager = self.election_manager.lock().unwrap();

        // Check if sender is the current Queen with valid lease
        let queen = manager.get_current_queen();

        if queen.as_ref() == Some(sender_id) {
            // Verify lease is still valid
            if let Some(lease) = manager.get_lease_info() {
                if lease.is_valid {
                    return ValidationResult::Accepted;
                } else {
                    return ValidationResult::Rejected {
                        reason: format!("Queen lease expired: {}", lease.invalid_reason.unwrap_or_default()),
                    };
                }
            }
        }

        // Non-Queen sender
        match command_type {
            // Workers can submit results without Queen permission
            "submit_result" | "heartbeat" | "status_report" => ValidationResult::Accepted,
            // All other commands require Queen authority
            _ => ValidationResult::Rejected {
                reason: format!("Only Queen can issue '{}' commands", command_type),
            },
        }
    }
}

/// Result of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationResult {
    /// Command accepted
    Accepted,
    /// Command rejected
    Rejected { reason: String },
}