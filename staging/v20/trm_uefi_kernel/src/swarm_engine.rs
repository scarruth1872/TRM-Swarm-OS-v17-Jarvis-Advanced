//! TRM Swarm OS — Swarm Engine (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/swarm_engine.py & strip_swarm_engine.dll.
//! Manages QIAE compliance modes (Solo-Ninja, Agile-Squad, Software-Factory),
//! security audit gates, approval gates, and multi-agent task routing.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- COMPLIANCE MODES (Legacy integer constants + Modern Enum) ---
pub const MODE_STRICT: u64 = 0;
pub const MODE_STANDARD: u64 = 1;
pub const MODE_PERMISSIVE: u64 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplianceMode {
    SoloNinja,
    AgileSquad,
    SoftwareFactory,
}

impl ComplianceMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComplianceMode::SoloNinja => "solo_ninja",
            ComplianceMode::AgileSquad => "agile_squad",
            ComplianceMode::SoftwareFactory => "software_factory",
        }
    }

    pub fn from_u64(v: u64) -> Self {
        match v {
            MODE_STRICT => ComplianceMode::SoftwareFactory,
            MODE_PERMISSIVE => ComplianceMode::SoloNinja,
            _ => ComplianceMode::AgileSquad,
        }
    }

    pub fn to_u64(&self) -> u64 {
        match self {
            ComplianceMode::SoftwareFactory => MODE_STRICT,
            ComplianceMode::SoloNinja => MODE_PERMISSIVE,
            ComplianceMode::AgileSquad => MODE_STANDARD,
        }
    }
}

// --- MODE CONFIGURATION ---
#[derive(Debug, Clone, Copy)]
pub struct ModeConfig {
    pub mode: ComplianceMode,
    pub security_audit: bool,
    pub approval_gates: bool,
    pub pipeline_gates: bool,
    pub team_size_descriptor: &'static str,
}

impl ModeConfig {
    pub fn for_mode(mode: ComplianceMode) -> Self {
        match mode {
            ComplianceMode::SoloNinja => Self {
                mode,
                security_audit: false,
                approval_gates: false,
                pipeline_gates: false,
                team_size_descriptor: "minimal",
            },
            ComplianceMode::AgileSquad => Self {
                mode,
                security_audit: true,
                approval_gates: false,
                pipeline_gates: true,
                team_size_descriptor: "standard",
            },
            ComplianceMode::SoftwareFactory => Self {
                mode,
                security_audit: true,
                approval_gates: true,
                pipeline_gates: true,
                team_size_descriptor: "full",
            },
        }
    }
}

// --- TASK ROUTE DECISION ---
#[derive(Debug, Clone)]
pub struct SwarmTaskRoute {
    pub target_agent: String,
    pub requires_audit: bool,
    pub requires_gates: bool,
    pub auto_approved: bool,
}

// --- SWARM ENGINE STATE ---
#[derive(Debug, Clone)]
pub struct SwarmEngine {
    pub mode: u64,
    pub compliance_mode: ComplianceMode,
    pub tasks_processed: u64,
    pub security_audits_count: u64,
    pub approval_gates_count: u64,
    registered_agents: Vec<String>,
}

impl SwarmEngine {
    pub fn new() -> Self {
        let default_mode = ComplianceMode::AgileSquad;
        let mut engine = Self {
            mode: default_mode.to_u64(),
            compliance_mode: default_mode,
            tasks_processed: 0,
            security_audits_count: 0,
            approval_gates_count: 0,
            registered_agents: Vec::new(),
        };

        // Populate default 14 Council agent identities
        engine.register_agent("Architect");
        engine.register_agent("Logic");
        engine.register_agent("Devo");
        engine.register_agent("Shield");
        engine.register_agent("Seeker");
        engine.register_agent("Bridge");
        engine.register_agent("Flow");
        engine.register_agent("Scribe");
        engine.register_agent("Validator");
        engine.register_agent("Operator");
        engine.register_agent("Sentinel");
        engine.register_agent("Chronos");
        engine.register_agent("Nexus");
        engine.register_agent("Quantum");

        engine
    }

    pub fn register_agent(&mut self, name: &str) {
        if !self.registered_agents.iter().any(|a| a == name) {
            self.registered_agents.push(name.to_string());
        }
    }

    pub fn set_compliance_mode(&mut self, mode_val: u64) {
        let clamped = mode_val.min(MODE_PERMISSIVE);
        self.mode = clamped;
        self.compliance_mode = ComplianceMode::from_u64(clamped);
    }

    pub fn set_mode(&mut self, mode: ComplianceMode) {
        self.compliance_mode = mode;
        self.mode = mode.to_u64();
    }

    pub fn get_mode_config(&self) -> ModeConfig {
        ModeConfig::for_mode(self.compliance_mode)
    }

    pub fn requires_security_audit(&self, task_severity: u64) -> bool {
        let cfg = self.get_mode_config();
        if !cfg.security_audit {
            return false;
        }
        task_severity >= 3 || (self.compliance_mode == ComplianceMode::SoftwareFactory && task_severity >= 1)
    }

    pub fn requires_approval_gates(&self, task_severity: u64) -> bool {
        let cfg = self.get_mode_config();
        if !cfg.approval_gates {
            return false;
        }
        task_severity >= 4 || (self.compliance_mode == ComplianceMode::SoftwareFactory && task_severity >= 2)
    }

    pub fn evaluate_task_route(&mut self, task_desc: &str, task_severity: u64) -> SwarmTaskRoute {
        self.tasks_processed += 1;
        let needs_audit = self.requires_security_audit(task_severity);
        let needs_gates = self.requires_approval_gates(task_severity);

        if needs_audit {
            self.security_audits_count += 1;
        }
        if needs_gates {
            self.approval_gates_count += 1;
        }

        let target = if self.compliance_mode == ComplianceMode::SoloNinja {
            "Architect".to_string()
        } else if task_desc.contains("security") || task_desc.contains("audit") || task_severity >= 4 {
            "Shield".to_string()
        } else if task_desc.contains("code") || task_desc.contains("implement") {
            "Devo".to_string()
        } else if task_desc.contains("math") || task_desc.contains("logic") || task_desc.contains("verify") {
            "Logic".to_string()
        } else {
            "Architect".to_string()
        };

        SwarmTaskRoute {
            target_agent: target,
            requires_audit: needs_audit,
            requires_gates: needs_gates,
            auto_approved: !needs_gates,
        }
    }

    pub fn get_status(&self) -> (u64, bool, bool) {
        (
            self.mode,
            self.requires_security_audit(3),
            self.requires_approval_gates(4),
        )
    }

    pub fn registered_agent_count(&self) -> usize {
        self.registered_agents.len()
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_SWARM_ENGINE: AtomicU64 = AtomicU64::new(0);

pub fn get_swarm_engine() -> u64 {
    let v = SINGLETON_SWARM_ENGINE.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SWARM_ENGINE.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

