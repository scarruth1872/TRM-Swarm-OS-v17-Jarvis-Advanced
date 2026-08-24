//! TRM Swarm OS — Heuristic Consensus (no_std port).
//! 1:1 core semantics from swarm_v2/core/heuristic_consensus.py — Catalyst.
//! Agent scoring heuristic: weighted combination of evidence, agreement,
//! and historical reliability. Deterministic.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Weights (1:1 python).
pub const W_EVIDENCE: f64 = 0.5;
pub const W_AGREEMENT: f64 = 0.3;
pub const W_RELIABILITY: f64 = 0.2;

/// Score an agent's contribution: 0..1.
/// evidence: 0..1, agreement: 0..1, reliability: 0..1.
pub fn score_agent(evidence: f64, agreement: f64, reliability: f64) -> f64 {
    (W_EVIDENCE * evidence + W_AGREEMENT * agreement + W_RELIABILITY * reliability).clamp(0.0, 1.0)
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

    /// Evaluate and log an agent.
    pub fn evaluate(&mut self, agent: &str, evidence: f64, agreement: f64, reliability: f64) -> f64 {
        let s = score_agent(evidence, agreement, reliability);
        self.log.push(RouteEntry {
            agent: agent.into(),
            score: s,
        });
        if s > self.best_score {
            self.best_score = s;
        }
        s
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
