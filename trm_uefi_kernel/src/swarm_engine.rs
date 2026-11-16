//! TRM Swarm OS — Swarm Engine (no_std port).
//! 1:1 core semantics from swarm_v2/core/swarm_engine.py — Catalyst.
//! Compliance modes: whether a task requires security audit / approval gates.

extern crate alloc;

/// Compliance modes (1:1 python).
pub const MODE_STRICT: u64 = 0;
pub const MODE_STANDARD: u64 = 1;
pub const MODE_PERMISSIVE: u64 = 2;

/// Swarm engine compliance state.
#[derive(Debug, Clone)]
pub struct SwarmEngine {
    pub mode: u64,
}

impl SwarmEngine {
    pub fn new() -> Self {
        SwarmEngine {
            mode: MODE_STANDARD,
        }
    }

    pub fn set_compliance_mode(&mut self, mode: u64) {
        self.mode = mode.min(MODE_PERMISSIVE);
    }

    /// Does this task require a security audit?
    pub fn requires_security_audit(&self, task_severity: u64) -> bool {
        // severity 3+ (high/critical) always; strict mode audits everything 1+
        task_severity >= 3 || (self.mode == MODE_STRICT && task_severity >= 1)
    }

    /// Does this task require approval gates?
    pub fn requires_approval_gates(&self, task_severity: u64) -> bool {
        task_severity >= 4 || (self.mode == MODE_STRICT && task_severity >= 3)
    }

    /// Status: (mode, audit_required_for_high, gates_for_critical).
    pub fn get_status(&self) -> (u64, bool, bool) {
        (self.mode, self.requires_security_audit(3), self.requires_approval_gates(4))
    }
}

/// Singleton handle.
pub fn get_swarm_engine() -> u64 {
    0x5357_4152_4D45_4E47
}
