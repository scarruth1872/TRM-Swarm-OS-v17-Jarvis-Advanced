//! TRM Swarm OS — Telemetry (no_std port).
//! Ported from `swarm_v2/core/telemetry.py` (Phase 4, Emergence Hub).
//!
//! The python class is an **aggregator**: `psutil` supplies system metrics,
//! the manus engine supplies superpositions, the resource arbiter supplies
//! VRAM status, the expert registry supplies cognitive stacks. Those
//! subsystems are host-bound, so they are INJECTED here (`SystemProbe` and
//! per-call arguments) rather than invented.
//!
//! The previous stub in this file hardcoded `agents_online: 14`,
//! `modules_real: 33` and `phi_v: 0.8055`, and returned a formatted string
//! from `get_emergence_report`. None of those values or signatures exist in
//! the python source — it was a fabricated API wearing the real function
//! names. It is replaced by the real aggregation below.
//!
//! Not ported: python's module-level `get_telemetry()` singleton. `no_std` has
//! no module globals; the kernel owns one `SwarmTelemetry` on the boot path,
//! so the cache lives at the call site instead of being faked here.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::coherence_thresholds;

/// One task in superposition — python `manus.active_superpositions[tid]`.
/// `paths` is `len(data["roles"])`, the projection's only derived field.
#[derive(Clone, Debug, PartialEq)]
pub struct Superposition {
    pub task_id: String,
    pub name: String,
    pub coherence: f64,
    pub status: String,
    pub paths: usize,
}

/// One member of `get_expert_team()` — a role paired with an OPTIONAL
/// cognitive stack, mirroring python's `hasattr(agent, "cognitive_stack")`.
#[derive(Clone, Debug, PartialEq)]
pub struct TeamMember {
    pub role: String,
    pub stack_status: Option<String>,
}

/// python's loop over `get_expert_team().items()` in `get_emergence_report`:
/// only agents that actually have a `cognitive_stack` contribute a status.
/// Order follows the team's iteration order.
pub fn collect_stack_statuses(team: &[TeamMember]) -> Vec<String> {
    team.iter()
        .filter_map(|m| m.stack_status.clone())
        .collect()
}

/// Host-bound metric source — python's `psutil` plus `datetime.now()`.
pub trait SystemProbe {
    fn cpu_percent(&self) -> f64;
    fn memory_percent(&self) -> f64;
    fn timestamp_iso(&self) -> String;
}

/// python `system_stats` inside `get_emergence_report`.
///
/// Every metric is written TWICE — `cpu_percent`/`cpu_load` and
/// `memory_percent`/`memory_usage` — because `check_stability` reads the
/// `*_load` / `*_usage` spellings. Both members of each pair come from a
/// single reading, exactly as the python does.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemStats {
    pub cpu_percent: f64,
    pub cpu_load: f64,
    pub memory_percent: f64,
    pub memory_usage: f64,
    pub timestamp: String,
}

impl SystemStats {
    /// Read the probe once, mirror each reading into both spellings.
    pub fn sample(probe: &impl SystemProbe) -> Self {
        let cpu = probe.cpu_percent();
        let memory = probe.memory_percent();
        SystemStats {
            cpu_percent: cpu,
            cpu_load: cpu,
            memory_percent: memory,
            memory_usage: memory,
            timestamp: probe.timestamp_iso(),
        }
    }

    /// The dict python hands to `CoherenceThresholds.check_stability`.
    /// `error_rate` is absent from `system_stats`, so python's
    /// `metrics.get("error_rate", 0)` yields `0`.
    pub fn metrics(&self) -> coherence_thresholds::StabilityMetrics {
        coherence_thresholds::StabilityMetrics {
            cpu_load: self.cpu_load,
            memory_usage: self.memory_usage,
            error_rate: 0.0,
        }
    }
}

/// python `SwarmTelemetry._calculate_global_coherence`.
///
/// An idle swarm is optimal, not undefined — empty input yields `1.0`.
/// Otherwise the plain arithmetic mean of the active coherences.
pub fn calculate_global_coherence(superpositions: &[Superposition]) -> f64 {
    if superpositions.is_empty() {
        return 1.0;
    }
    let total: f64 = superpositions.iter().map(|s| s.coherence).sum();
    total / superpositions.len() as f64
}

/// The dict `get_soul_report` returns.
#[derive(Clone, Debug, PartialEq)]
pub struct SoulReport {
    pub harmony_index: f64,
    pub reflection: String,
    pub autonomous_proposals: Vec<String>,
    pub timestamp: String,
}

/// The dict `get_emergence_report` returns.
#[derive(Clone, Debug, PartialEq)]
pub struct EmergenceReport {
    pub status: &'static str,
    pub system: SystemStats,
    pub superpositions: Vec<Superposition>,
    pub resource_arbiter: String,
    pub distributed_stacks: Vec<String>,
    pub mesh_coherence: f64,
    pub harmony_index: f64,
    pub active_proposals: usize,
}

/// python `SwarmTelemetry` — holds exactly the state the python object holds.
#[derive(Clone, Debug, Default)]
pub struct SwarmTelemetry {
    /// `self.manus.active_superpositions`
    pub superpositions: Vec<Superposition>,
    /// `self.harmony_monitor.get_harmony_index()`
    pub harmony_index: f64,
    /// `self.harmony_monitor.generate_alignment_reflection()`
    pub reflection: String,
    /// `self.proactive.get_active_proposals()`
    pub active_proposals: Vec<String>,
}

impl SwarmTelemetry {
    /// python `__init__`. The subsystems it fetches from module-level
    /// singletons are supplied by the caller instead.
    pub fn new() -> Self {
        Self::default()
    }

    /// python `get_emergence_report()`.
    ///
    /// `arbiter_status` stands in for `self.arbiter.get_status()`; `team` is
    /// `get_expert_team()`, whose stackless agents python skips.
    pub fn get_emergence_report(
        &self,
        probe: &impl SystemProbe,
        arbiter_status: impl Into<String>,
        team: &[TeamMember],
    ) -> EmergenceReport {
        let system = SystemStats::sample(probe);
        EmergenceReport {
            status: coherence_thresholds::check_stability(&system.metrics()).as_str(),
            mesh_coherence: calculate_global_coherence(&self.superpositions),
            system,
            superpositions: self.superpositions.clone(),
            resource_arbiter: arbiter_status.into(),
            distributed_stacks: collect_stack_statuses(team),
            harmony_index: self.harmony_index,
            active_proposals: self.active_proposals.len(),
        }
    }

    /// python `get_soul_report()`.
    pub fn get_soul_report(&self, probe: &impl SystemProbe) -> SoulReport {
        SoulReport {
            harmony_index: self.harmony_index,
            reflection: self.reflection.clone(),
            autonomous_proposals: self.active_proposals.clone(),
            timestamp: probe.timestamp_iso(),
        }
    }

    /// python `record_external_metric` — a documented no-op there too
    /// ("For Phase 17, we simply log it or store in a local ring buffer").
    /// Nothing is buffered in either implementation; the method is kept so the
    /// port's surface matches the source rather than silently dropping a hook.
    pub fn record_external_metric(&mut self, _node_url: &str, _metrics: &str) {}
}
