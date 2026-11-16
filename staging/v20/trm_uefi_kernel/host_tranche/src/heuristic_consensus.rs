//! TRM Swarm OS — Heuristic Consensus (no_std port).
//! 1:1 core semantics from swarm_v2/core/heuristic_consensus.py — Catalyst.
//! Agent scoring heuristic: weighted combination of evidence, agreement,
//! and historical reliability. Deterministic.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

// ── The platform's scoring model (swarm_v2/core/heuristic_consensus.py) ─────────────────────────
//
// python:  WEIGHTS = {skill_match: 0.35, health_score: 0.25, availability: 0.20, proficiency: 0.20}
//          MAX_CONCURRENT_TASKS = 10
//          skill_match: +10 per specialty named in the task, +5 per skill, +8 if the role is named,
//                       then min(raw / 30, 1)
//          availability: clamp(1 - task_count / max(10, 1), 0, 1)
//          proficiency: 0.5 by default; a task-matched skill's max proficiency / 3.0 (capped 1.0),
//                       else the general maximum / 6.0 (capped 0.5)
//          score: the weighted sum, rounded to 4 decimals
//
// The module previously carried a three-factor model -- evidence 0.5, agreement 0.3, reliability 0.2 --
// that appears nowhere in the oracle, under a header claiming 1:1 core semantics. Different factors and
// different weights is not a partial port; it is a different algorithm, so the constants are replaced
// rather than kept beside the real ones.

/// The oracle's four weights, keyed exactly as the python dict.
pub const W_SKILL_MATCH: f64 = 0.35;
pub const W_HEALTH: f64 = 0.25;
pub const W_AVAILABILITY: f64 = 0.20;
pub const W_PROFICIENCY: f64 = 0.20;
/// The oracle's concurrency ceiling, the availability term's divisor.
pub const MAX_CONCURRENT_TASKS: f64 = 10.0;
/// The oracle's neutral proficiency when nothing is known.
pub const PROFICIENCY_NEUTRAL: f64 = 0.5;
/// Normalisation divisors: a directly matched skill is scored against 3.0, an indirect one against 6.0.
pub const PROFICIENCY_MATCH_DIVISOR: f64 = 3.0;
pub const PROFICIENCY_GENERAL_DIVISOR: f64 = 6.0;

/// `round(score, 4)`. `no_std` has no `f64::round` (the same absence as `log2` in the skill matrix),
/// so this scales, adds half a unit away from zero and truncates. Ties therefore go away from zero
/// where python's `round` is half-to-even; the two differ only when the scaled value lands exactly on
/// `.5`, which for these factor products means an exact tie at the 4th decimal.
pub fn round4(v: f64) -> f64 {
    let scaled = v * 10_000.0;
    let r = if scaled >= 0.0 {
        (scaled + 0.5) as i64
    } else {
        (scaled - 0.5) as i64
    };
    (r as f64) / 10_000.0
}

/// Factor 1: how much of the task names this agent's specialties, skills or role.
pub fn skill_match(task: &str, specialties: &[&str], skills: &[&str], role: &str) -> f64 {
    let t = task.to_lowercase();
    let mut raw = 0i32;
    for s in specialties {
        if !s.is_empty() && t.contains(&s.to_lowercase()) {
            raw += 10;
        }
    }
    for s in skills {
        if !s.is_empty() && t.contains(&s.to_lowercase()) {
            raw += 5;
        }
    }
    if !role.is_empty() && t.contains(&role.to_lowercase()) {
        raw += 8;
    }
    let v = f64::from(raw) / 30.0;
    if v > 1.0 {
        1.0
    } else {
        v
    }
}

/// Factor 3: how much room the node has, against the oracle's ceiling.
pub fn availability(task_count: f64) -> f64 {
    let a = 1.0 - (task_count / if MAX_CONCURRENT_TASKS < 1.0 { 1.0 } else { MAX_CONCURRENT_TASKS });
    a.clamp(0.0, 1.0)
}

/// Factor 4: proficiency from the skill matrix, with the oracle's two normalisations and its default.
/// `matched_max` is the maximum proficiency over skills whose name appears in the task; `general_max`
/// is the maximum over the node's skills when nothing matched.
pub fn proficiency(matched_max: Option<f64>, general_max: Option<f64>) -> f64 {
    if let Some(m) = matched_max {
        let v = m / PROFICIENCY_MATCH_DIVISOR;
        if v > 1.0 {
            1.0
        } else {
            v
        }
    } else if let Some(g) = general_max {
        let v = g / PROFICIENCY_GENERAL_DIVISOR;
        if v > PROFICIENCY_NEUTRAL {
            PROFICIENCY_NEUTRAL
        } else {
            v
        }
    } else {
        PROFICIENCY_NEUTRAL
    }
}

/// The oracle's weighted sum over the four factors, rounded to 4 decimals.
pub fn weighted_score(skill_match: f64, health: f64, availability: f64, proficiency: f64) -> f64 {
    round4(
        skill_match * W_SKILL_MATCH
            + health * W_HEALTH
            + availability * W_AVAILABILITY
            + proficiency * W_PROFICIENCY,
    )
}

/// `score_agent()` without the mesh: the four factors in, the platform's score out.
pub fn score_agent(
    task: &str,
    specialties: &[&str],
    skills: &[&str],
    role: &str,
    health: f64,
    task_count: f64,
    matched_max: Option<f64>,
    general_max: Option<f64>,
) -> f64 {
    weighted_score(
        skill_match(task, specialties, skills, role),
        health,
        availability(task_count),
        proficiency(matched_max, general_max),
    )
}

/// Routing log entry.
#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub agent: String,
    pub score: f64,
}

/// Heuristic consensus engine.
#[derive(Debug, Clone)]
pub struct HeuristicConsensus {
    pub log: Vec<RouteEntry>,
    pub best_score: f64,
}

impl HeuristicConsensus {
    pub fn new() -> Self {
        HeuristicConsensus {
            log: Vec::new(),
            best_score: 0.0,
        }
    }

    /// Evaluate an agent with the platform's four factors and log the outcome. The previous signature
    /// took (evidence, agreement, reliability) -- factors the oracle does not have.
    pub fn evaluate(
        &mut self,
        agent: &str,
        task: &str,
        specialties: &[&str],
        skills: &[&str],
        role: &str,
        health: f64,
        task_count: f64,
        matched_max: Option<f64>,
        general_max: Option<f64>,
    ) -> f64 {
        let s = score_agent(task, specialties, skills, role, health, task_count, matched_max, general_max);
        self.log.push(RouteEntry {
            agent: agent.into(),
            score: s,
        });
        if s > self.best_score {
            self.best_score = s;
        }
        s
    }

    /// `get_routing_log()`: the logged entries, as the oracle exposes them.
    pub fn get_routing_log(&self) -> &[RouteEntry] {
        &self.log
    }

    /// Stats: (entries, best_score).
    pub fn get_stats(&self) -> (u64, f64) {
        (self.log.len() as u64, self.best_score)
    }
}

/// Singleton handle.
pub fn get_heuristic_consensus() -> u64 {
    0x4845_5552_4953_5449
}
