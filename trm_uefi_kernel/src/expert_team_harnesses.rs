//! 64-Expert Entangled Team Harness Registry for Instella-MoE.
//!
//! Port of `swarm_v2/core/expert_team_harnesses.py`. Maps 64 MoE experts to 6
//! specialised domain harnesses under 1-of-64 single-expert activation:
//!   Trigger  k* = argmax_k g_k(x)
//!   Active set |S| = 1   (only k* fires; the other 63 stay at 0-power standby)
//!   Reduction  R = 64x
//!
//! PORT DEFECT FOUND AND FIXED: the mechanical port emitted HARNESS_DOMAINS with
//! the expert-id RANGE collapsed to a literal 0 - `list(range(1, 11))` became
//! `0` - so the 64-entry registry could never be built and every lookup would
//! have fallen through to expert 1. The ranges are restored here as (first, last).
//!
//! RNG NOTE (honest divergence): the Python seeds numpy's Mersenne Twister with
//! 42 to build a 64x128 embedding matrix. numpy's generator cannot be
//! reproduced without implementing MT19937, so this port uses the platform's own
//! seeded xorshift Rng. The VALUES therefore differ from the Python's. This is
//! safe for the current API because `resolve_entangled_team()` never reads the
//! embeddings - they are built and stored only. If a future cosine-similarity
//! router consumes them, the two implementations will NOT agree and that must be
//! resolved deliberately rather than discovered later.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

use crate::float_math::Rng;

/// One domain harness: name, inclusive expert-id range (1-indexed), agent roles.
pub const HARNESS_DOMAINS: &[(&str, u32, u32, &[&str])] = &[
    ("Security & Cryptography", 1, 10,
     &["Shield", "VulnScanSkill", "CryptoAuditSkill", "NeuralWall", "DDR_Antibody"]),
    ("Logic & Mathematical Proof", 11, 20,
     &["Logic", "MathSolver", "LogicProver", "AttemptSampler", "ConsensusEngine"]),
    ("Code Engineering & Refactoring", 21, 30,
     &["Devo", "RefactorSkill", "TestGeneratorSkill", "FuzzTestSkill", "ToolForge"]),
    ("Telemetry & Memory Synchronization", 31, 40,
     &["Pulse", "GlobalMemory", "SystemMetricsSkill", "MemorySynchronizer", "MonitoringSkill"]),
    ("Synthesis & Chain-of-Thought", 41, 50,
     &["Archi", "CoCreatorSkill", "SynthesisPipeline", "CognitiveOrchestrator", "IntentBuffer"]),
    ("Autonomous Healing & Resilience", 51, 64,
     &["SelfHealing", "ChaosSimulator", "Sentinel", "MicrokernelSpawner", "StarMatrix"]),
];

/// Activation threshold applied to every expert (python literal 0.85).
pub const ACTIVATION_THRESHOLD: f64 = 0.85;
/// Standby power for a dark expert (python literal 0.0 mW).
pub const STANDBY_POWER_MW: f64 = 0.0;

/// One registry entry, mirroring the python dict.
#[derive(Clone, Debug, PartialEq)]
pub struct HarnessEntry {
    pub expert_id: u32,
    pub domain: String,
    pub entangled_agents: Vec<String>,
    pub primary_role: String,
    pub activation_threshold: f64,
    pub standby_power_mw: f64,
}

/// The 6 domain harnesses expanded to 64 expert entries, ascending by id.
pub fn build_registry() -> Vec<HarnessEntry> {
    let mut out = Vec::with_capacity(64);
    for (domain, first, last, roles) in HARNESS_DOMAINS {
        let mut eid = *first;
        while eid <= *last {
            out.push(HarnessEntry {
                expert_id: eid,
                domain: domain.to_string(),
                entangled_agents: roles.iter().map(|r| r.to_string()).collect(),
                primary_role: roles[0].to_string(),
                activation_threshold: ACTIVATION_THRESHOLD,
                standby_power_mw: STANDBY_POWER_MW,
            });
            eid += 1;
        }
    }
    out
}

/// The python `EXPERT_HARNESS_REGISTRY` dict: id -> entry.
pub fn registry_map() -> Vec<(u32, HarnessEntry)> {
    build_registry().into_iter().map(|e| (e.expert_id, e)).collect()
}

/// The python `resolve_entangled_team()` return dict.
#[derive(Clone, Debug, PartialEq)]
pub struct EntangledTeam {
    pub activated_expert_id: u32,
    pub expert_index_0_based: u32,
    pub domain: String,
    pub primary_role: String,
    pub entangled_team: Vec<String>,
    pub compute_overhead_reduction: String,
    pub dark_experts_in_standby: u32,
    pub entanglement_status: String,
}

/// Manages 1-of-64 single-expert entangled activation.
#[derive(Clone, Debug)]
pub struct ExpertTeamHarnessManager {
    pub num_experts: u32,
    pub d_model: u32,
    /// Row-major [num_experts x d_model], each row normalised to the unit sphere.
    pub expert_embeddings: Vec<f32>,
    pub registry: Vec<(u32, HarnessEntry)>,
}

impl ExpertTeamHarnessManager {
    /// Python seeded numpy with 42; this uses the platform Rng (see the RNG NOTE
    /// at the top of the file - the values are NOT the same as numpy's).
    pub fn new(num_experts: u32, d_model: u32) -> Self {
        let mut rng = Rng::new(42);
        let n = (num_experts * d_model) as usize;
        let mut emb: Vec<f32> = Vec::with_capacity(n);
        for _ in 0..n {
            // Approximate a standard normal by the Irwin-Hall construction: the
            // sum of 12 uniforms minus 6 has mean 0 and variance 1. This avoids
            // pulling ln/cos into float_math for values the current API never
            // reads - but note it is NOT numpy's Mersenne Twister, so the
            // numbers differ from the Python's regardless.
            let mut s = 0.0f64;
            for _ in 0..12 { s += rng.next_f64(); }
            emb.push((s - 6.0) as f32);
        }
        // normalise each row to the unit sphere; zero-norm rows divide by 1.0
        let dm = d_model as usize;
        for row in 0..num_experts as usize {
            let base = row * dm;
            let mut sum = 0.0f32;
            for j in 0..dm { sum += emb[base + j] * emb[base + j]; }
            let norm = sqrt_f32(sum);
            let denom = if norm == 0.0 { 1.0 } else { norm };
            for j in 0..dm { emb[base + j] /= denom; }
        }
        Self { num_experts, d_model, expert_embeddings: emb, registry: registry_map() }
    }

    /// Default manager: 64 experts, d_model 128 (the python defaults).
    pub fn default_manager() -> Self { Self::new(64, 128) }

    fn lookup(&self, eid: u32) -> &HarnessEntry {
        self.registry
            .iter()
            .find(|(id, _)| *id == eid)
            .map(|(_, e)| e)
            .unwrap_or(&self.registry[0].1)
    }

    /// Map a 0-indexed expert id (0..63) to the 1-indexed MoE id (1..64) and
    /// return its entangled team. Mirrors `resolve_entangled_team`.
    pub fn resolve_entangled_team(&self, expert_id_0_indexed: u32) -> EntangledTeam {
        let eid = (expert_id_0_indexed % self.num_experts) + 1;
        let h = self.lookup(eid);
        EntangledTeam {
            activated_expert_id: eid,
            expert_index_0_based: expert_id_0_indexed,
            domain: h.domain.clone(),
            primary_role: h.primary_role.clone(),
            entangled_team: h.entangled_agents.clone(),
            compute_overhead_reduction: format!(
                "{}x (1 active, {} dark standby)", self.num_experts, self.num_experts - 1),
            dark_experts_in_standby: self.num_experts - 1,
            entanglement_status: "ENTANGLED_PREWARMED".to_string(),
        }
    }

    /// Entangled agent roles for an expert id, as a plain list.
    pub fn teammates_for(&self, expert_id_0_indexed: u32) -> Vec<String> {
        self.resolve_entangled_team(expert_id_0_indexed).entangled_team
    }
}

/// Returns the default manager handle. The python used a module-level global
/// singleton; the kernel has no OS to own global mutable state, so callers hold
/// a manager (see `default_manager`). Retained for API-shape compatibility and
/// used as a live call site so the module is not linker-eliminated.
pub fn get_harness_manager() -> ExpertTeamHarnessManager {
    ExpertTeamHarnessManager::default_manager()
}

/// sqrt for f32 by widening to the verified f64 implementation.
fn sqrt_f32(x: f32) -> f32 { crate::float_math::sqrt_f64(x as f64) as f32 }
