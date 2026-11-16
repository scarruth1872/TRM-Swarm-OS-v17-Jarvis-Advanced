//! TRM Swarm OS — Predictive Watchdog (no_std port).
//! 1:1 core semantics from swarm_v2/core/predictive_watchdog.py — Catalyst.
//! Predicts restart risk from consecutive failure counts.

extern crate alloc;

/// Watchdog thresholds (1:1 python).
pub const WATCHDOG_RESTART_AFTER: u64 = 3;
pub const WATCHDOG_PREDICT_AT: u64 = 2;

/// Predictive watchdog state.
#[derive(Debug, Clone)]
pub struct Watchdog {
    pub consecutive_failures: u64,
    pub restarts: u64,
}

impl Watchdog {
    pub fn new() -> Self {
        Watchdog {
            consecutive_failures: 0,
            restarts: 0,
        }
    }

    /// Record a failure; returns restart_predicted (>= 2 consecutive).
    pub fn record_failure(&mut self) -> bool {
        self.consecutive_failures += 1;
        self.consecutive_failures >= WATCHDOG_PREDICT_AT
    }

    /// Record a success (resets the streak).
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
    }

    /// Execute a restart (reset + count).
    pub fn restart(&mut self) -> u64 {
        self.restarts += 1;
        self.consecutive_failures = 0;
        self.restarts
    }

    /// Is a restart predicted right now?
    pub fn is_restart_predicted(&self) -> bool {
        self.consecutive_failures >= WATCHDOG_RESTART_AFTER
    }

    /// Status: (failures, restarts, predicted).
    pub fn get_status(&self) -> (u64, u64, bool) {
        (self.consecutive_failures, self.restarts, self.is_restart_predicted())
    }
}

/// Singleton handle.
pub fn get_watchdog() -> u64 {
    0x5741_5443_4844_4F47
}
