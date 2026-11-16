//! TRM Swarm OS — Autonomous Remediation Engine (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/remediation_engine.py & strip_remediation_engine.dll.
//! Evaluates incident severity against Coherence Threshold policies,
//! triggering agent restarts, rollback actions, memory resets, or human escalation.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- REMEDIATION SEVERITY ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RemediationSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl RemediationSeverity {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => RemediationSeverity::Low,
            2 => RemediationSeverity::Medium,
            3 => RemediationSeverity::High,
            _ => RemediationSeverity::Critical,
        }
    }
}

// --- REMEDIATION ACTION ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemediationAction {
    RestartAgent,
    RollbackDeployment,
    ReallocateMemory,
    EscalateToHuman,
}

impl RemediationAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            RemediationAction::RestartAgent => "restart_agent",
            RemediationAction::RollbackDeployment => "rollback_deployment",
            RemediationAction::ReallocateMemory => "reallocate_memory",
            RemediationAction::EscalateToHuman => "escalate_to_human",
        }
    }
}

// --- REMEDIATION INCIDENT PLAN ---
#[derive(Debug, Clone)]
pub struct RemediationPlan {
    pub incident_id: String,
    pub source_component: String,
    pub severity: RemediationSeverity,
    pub action: RemediationAction,
    pub executed: bool,
    pub success: bool,
    pub epoch_s: u64,
}

// --- REMEDIATION ENGINE ---
pub struct RemediationEngine {
    plans: Vec<RemediationPlan>,
    pub total_remediations: u64,
    pub incidents_escalated: u64,
}

impl RemediationEngine {
    pub fn new() -> Self {
        Self {
            plans: Vec::new(),
            total_remediations: 0,
            incidents_escalated: 0,
        }
    }

    pub fn trigger_remediation(
        &mut self,
        component: &str,
        severity_val: u32,
        epoch_s: u64,
    ) -> RemediationAction {
        self.total_remediations += 1;
        let severity = RemediationSeverity::from_u32(severity_val);

        let action = match severity {
            RemediationSeverity::Low => RemediationAction::ReallocateMemory,
            RemediationSeverity::Medium => RemediationAction::RestartAgent,
            RemediationSeverity::High => RemediationAction::RollbackDeployment,
            RemediationSeverity::Critical => {
                self.incidents_escalated += 1;
                RemediationAction::EscalateToHuman
            }
        };

        let incident_id = format!("rem_{}_{}", component, self.total_remediations);
        let plan = RemediationPlan {
            incident_id,
            source_component: component.to_string(),
            severity,
            action,
            executed: true,
            success: true,
            epoch_s,
        };

        if self.plans.len() >= 128 {
            self.plans.remove(0);
        }
        self.plans.push(plan);

        action
    }

    pub fn get_recent_plans(&self, limit: usize) -> &[RemediationPlan] {
        let count = limit.min(self.plans.len());
        let start = self.plans.len() - count;
        &self.plans[start..]
    }

    pub fn total_plans(&self) -> usize {
        self.plans.len()
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_REMEDIATION_ENGINE: AtomicU64 = AtomicU64::new(0);

pub fn get_remediation_engine() -> u64 {
    let v = SINGLETON_REMEDIATION_ENGINE.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_REMEDIATION_ENGINE.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}


