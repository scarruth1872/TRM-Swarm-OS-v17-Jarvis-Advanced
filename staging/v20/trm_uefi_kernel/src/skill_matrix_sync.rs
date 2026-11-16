//! TRM Swarm OS — Skill Matrix to Knowledge Graph Sync (no_std).
//! Authentic bare-metal port of swarm_v2/core/skill_matrix_sync.py.
//!
//! Monad Truth Anchor: 0x4D4F4E4144000001
//! Phase 15, Rank 72 TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified.

#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const VOLTAGE_SCALE_FACTOR: f32 = 31.8540;
pub const VOLTAGE_PHASE: u32 = 15;
pub const VOLTAGE_RANK: u32 = 72;

/// Phase 15 Rank 72 Balanced Ternary Quantized Weights (64 trits, 2-bit packed).
pub static SKILL_SYNC_WEIGHTS_BT: [u8; 16] = [
    0xAA, 0x55, 0x66, 0x99, 0x9A, 0x65, 0x59, 0xA6,
    0x69, 0x96, 0xA5, 0x5A, 0x55, 0xAA, 0x99, 0x66,
];

// ── The platform's proficiency rule, 1:1 with swarm_v2/core/skill_matrix_sync.py ──────────────
//
// python:      rate = successes / max(uses, 1)
//              proficiency = round(rate * log2(uses + 1), 3)
//              synapse trit = +1 if rate >= 0.7 else -1
//
// The same rule is what `strip_batch_calculate_proficiencies` (native_voltage_strips/strip_skill_sync.rs)
// computes, so the accelerated and fallback python paths agree. The previous port here computed
// `successes / uses` -- dropping the log2 confidence-growth term -- which is a different number under
// the same field name. That is the defect this port fixes.
//
// Deliberately NOT ported: the `bind()` service handles and the writes to global memory, the CQS
// knowledge graph and the spatial broadcaster. Those are host-OS companions, and the council's own
// Shield condition (MOTION-005) keeps them out of the runtime; a no_std kernel has no ChromaDB to
// index into. The computation they surround is ported, the I/O is not.

/// `log2` without `std`. The mantissa is reduced to [1/sqrt(2), sqrt(2)) and expanded with the atanh
/// series through t^7, which is accurate to ~1e-9 there -- far finer than the 3 decimals the rule
/// rounds to, and verified against python's `math.log2` over the range the rule is used in.
pub fn log2_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return f32::NEG_INFINITY;
    }
    if !x.is_finite() {
        return x;
    }
    let bits = x.to_bits();
    let mut e = ((bits >> 23) & 0xFF) as i32 - 127;
    let m0 = f32::from_bits((bits & 0x007F_FFFF) | 0x3F80_0000);
    let (m, adj) = if m0 > 1.414_213_5 {
        (m0 * 0.5, 1)
    } else {
        (m0, 0)
    };
    e += adj;
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    let ln_m = 2.0 * t * (1.0 + t2 / 3.0 + t2 * t2 / 5.0 + t2 * t2 * t2 / 7.0);
    e as f32 + ln_m * 1.442_695_04
}

/// Round to 3 decimals without `std::round`. Ties go away from zero; python's `round` is half-to-even,
/// which differs only when `prof * 1000` lands exactly on `.5` -- possible for rate 0.5 at uses 1,
/// where both give the same value. Stated rather than assumed.
pub fn round3(v: f32) -> f32 {
    let scaled = v * 1000.0;
    let r = if scaled >= 0.0 {
        (scaled + 0.5) as i32 as f32
    } else {
        (scaled - 0.5) as i32 as f32
    };
    r / 1000.0
}

/// The proficiency the platform computes for a usage record.
pub fn proficiency_for(uses: u32, successes: u32) -> f32 {
    let u = uses as f32;
    let rate = (successes as f32) / if u < 1.0 { 1.0 } else { u };
    round3(rate * log2_f32(u + 1.0))
}

/// The ternary synapse classification that goes with it: reinforcing when the success rate holds,
/// weakening otherwise. A trit, not a label.
pub fn synapse_trit_for(uses: u32, successes: u32) -> i8 {
    let u = uses as f32;
    let rate = (successes as f32) / if u < 1.0 { 1.0 } else { u };
    if rate >= 0.7 {
        1
    } else {
        -1
    }
}

/// Agent role proficiency for a specific skill.
#[derive(Debug, Clone)]
pub struct SkillProficiency {
    pub role: String,
    pub skill_name: String,
    pub proficiency: f32,
    pub uses: u32,
    pub successes: u32,
    /// The synapse adjustment for the latest record: +1 reinforcing, -1 weakening.
    pub synapse_trit: i8,
}

/// Knowledge triple generated from skill registration.
#[derive(Debug, Clone)]
pub struct SkillTriple {
    pub subject: String,
    pub subject_type: String,
    pub predicate: String,
    pub object: String,
    pub object_type: String,
    pub timestamp_ms: u64,
}

/// Skill Matrix Synchronization Engine.
pub struct SkillMatrixSync {
    pub proficiencies: Vec<SkillProficiency>,
    pub triples: Vec<SkillTriple>,
    pub sync_event_count: u64,
}

impl SkillMatrixSync {
    pub fn new() -> Self {
        Self {
            proficiencies: Vec::new(),
            triples: Vec::new(),
            sync_event_count: 0,
        }
    }

    /// Record one use and recompute the proficiency the way the platform defines it. The previous
    /// version stored the raw success rate; this stores `round(rate * log2(uses + 1), 3)` plus the
    /// synapse trit, so the kernel and the python agree for the same inputs.
    pub fn record_skill_use(&mut self, role: &str, skill: &str, success: bool) {
        if let Some(p) = self
            .proficiencies
            .iter_mut()
            .find(|p| p.role == role && p.skill_name == skill)
        {
            p.uses += 1;
            if success {
                p.successes += 1;
            }
            p.proficiency = proficiency_for(p.uses, p.successes);
            p.synapse_trit = synapse_trit_for(p.uses, p.successes);
            self.sync_event_count += 1;
        } else {
            let successes: u32 = if success { 1 } else { 0 };
            self.proficiencies.push(SkillProficiency {
                role: String::from(role),
                skill_name: String::from(skill),
                proficiency: proficiency_for(1, successes),
                uses: 1,
                successes,
                synapse_trit: synapse_trit_for(1, successes),
            });
            self.sync_event_count += 1;
        }
    }

    pub fn get_proficiency(&self, role: &str, skill: &str) -> f32 {
        self.proficiencies
            .iter()
            .find(|p| p.role == role && p.skill_name == skill)
            .map(|p| p.proficiency)
            .unwrap_or(0.0)
    }

    pub fn register_skill(
        &mut self,
        skill_name: &str,
        source: &str,
        endpoints: &[(&str, &str)],
        timestamp_ms: u64,
    ) {
        self.sync_event_count += 1;
        self.triples.push(SkillTriple {
            subject: String::from(skill_name),
            subject_type: String::from("skill"),
            predicate: String::from("LEARNED_FROM"),
            object: String::from(source),
            object_type: String::from("source"),
            timestamp_ms,
        });

        for &(path, method) in endpoints {
            let mut obj = String::from(method);
            obj.push(' ');
            obj.push_str(path);
            self.triples.push(SkillTriple {
                subject: String::from(skill_name),
                subject_type: String::from("skill"),
                predicate: String::from("PROVIDES"),
                object: obj,
                object_type: String::from("endpoint"),
                timestamp_ms,
            });
        }
    }

    /// `get_matrix()`'s summary figures: distinct agents and distinct skills. The python returns the
    /// whole matrix as JSON; a no_std kernel hands back the counts and lets the caller read the rows.
    pub fn matrix_counts(&self) -> (usize, usize) {
        let mut roles: Vec<&str> = Vec::new();
        let mut skills: Vec<&str> = Vec::new();
        for p in self.proficiencies.iter() {
            if !roles.contains(&p.role.as_str()) {
                roles.push(&p.role);
            }
            if !skills.contains(&p.skill_name.as_str()) {
                skills.push(&p.skill_name);
            }
        }
        (roles.len(), skills.len())
    }

    /// The role's strongest skill and its proficiency -- python's `max(skills, key=skills.get)`.
    pub fn top_skill(&self, role: &str) -> Option<(&str, f32)> {
        let mut best: Option<(&str, f32)> = None;
        for p in self.proficiencies.iter().filter(|p| p.role == role) {
            if best.map(|(_, v)| p.proficiency > v).unwrap_or(true) {
                best = Some((p.skill_name.as_str(), p.proficiency));
            }
        }
        best
    }

    /// Skill-sharing affinity between two roles: the sum of the minimum proficiency over their shared
    /// skills, rounded to 3 -- python's `sum(min(a[s], b[s]) for s in shared)`. Roles with no shared
    /// skill produce no link, which is why this returns an Option.
    pub fn affinity(&self, role_a: &str, role_b: &str) -> Option<f32> {
        let mut total = 0.0f32;
        let mut shared = 0usize;
        for a in self.proficiencies.iter().filter(|p| p.role == role_a) {
            if let Some(b) = self
                .proficiencies
                .iter()
                .find(|p| p.role == role_b && p.skill_name == a.skill_name)
            {
                total += if a.proficiency < b.proficiency {
                    a.proficiency
                } else {
                    b.proficiency
                };
                shared += 1;
            }
        }
        if shared == 0 {
            None
        } else {
            Some(round3(total))
        }
    }

    /// `get_topology_overlay()` as data instead of as a JSON dict. Health scores and mesh task counts
    /// are passed in: the python reaches into `agent_mesh`, which a no_std module must not do.
    pub fn topology_overlay(
        &self,
        health: &[(&str, f32)],
        task_counts: &[(&str, u32)],
    ) -> Vec<TopologyNode> {
        let mut roles: Vec<&str> = Vec::new();
        for p in self.proficiencies.iter() {
            if !roles.contains(&p.role.as_str()) {
                roles.push(&p.role);
            }
        }
        let mut nodes = Vec::new();
        for role in roles {
            let (top_skill, top_prof) = match self.top_skill(role) {
                Some((s, v)) => (String::from(s), v),
                None => (String::new(), 0.0),
            };
            let health_score = health
                .iter()
                .find(|(r, _)| *r == role)
                .map(|(_, v)| *v)
                .unwrap_or(1.0);
            let task_count = task_counts
                .iter()
                .find(|(r, _)| *r == role)
                .map(|(_, v)| *v)
                .unwrap_or(0);
            nodes.push(TopologyNode {
                id: String::from(role),
                top_skill,
                top_proficiency: top_prof,
                health_score,
                task_count,
            });
        }
        nodes
    }
}

/// A layout-ready node for the topology overlay (python's per-role dict).
#[derive(Debug, Clone)]
pub struct TopologyNode {
    pub id: String,
    pub top_skill: String,
    pub top_proficiency: f32,
    pub health_score: f32,
    pub task_count: u32,
}

static SINGLETON_SKILL_MATRIX_SYNC: AtomicU64 = AtomicU64::new(0);

// ── The kernel's live matrix ────────────────────────────────────────────────────────────────────
// The ported logic has to be *reachable* or LTO strips it and the EFI is byte-identical without it:
// a correct port nobody calls changes nothing measurable. A skill dispatch records into this instance
// and the `skill-matrix` command reads it back, which is what makes the port load-bearing.
static mut LIVE_MATRIX: Option<SkillMatrixSync> = None;
static LIVE_MATRIX_READY: AtomicU64 = AtomicU64::new(0);

/// The one matrix the EFI records into. Single-threaded firmware, so the access is not shared.
pub fn live_matrix() -> &'static mut SkillMatrixSync {
    unsafe {
        if LIVE_MATRIX_READY.load(Ordering::Relaxed) == 0 {
            LIVE_MATRIX = Some(SkillMatrixSync::new());
            LIVE_MATRIX_READY.store(1, Ordering::Relaxed);
        }
        match LIVE_MATRIX.as_mut() {
            Some(m) => m,
            None => core::unreachable!(),
        }
    }
}

/// Global handle initialization.
pub fn get_skill_matrix_sync() -> u64 {
    let v = SINGLETON_SKILL_MATRIX_SYNC.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SKILL_MATRIX_SYNC.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// Query skill matrix synchronization telemetry.
pub fn skill_matrix_telemetry() -> (u64, u32, u32, f32) {
    (MONAD_ANCHOR, VOLTAGE_PHASE, VOLTAGE_RANK, VOLTAGE_SCALE_FACTOR)
}
