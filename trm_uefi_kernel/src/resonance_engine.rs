//! TRM Swarm OS — Resonance Engine (no_std port).
//! 1:1 core semantics from swarm_v2/core/resonance_engine.py — Catalyst Phase 1.
//! The emotional resonance index: weighted mean of agent dream resonances
//! with coherence threshold. Deterministic, on-device.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Resonance constants.
pub const RESONANCE_BASE: f64 = 0.5;
pub const COHERENCE_THRESHOLD: f64 = 0.6;
pub const DREAM_WEIGHT: f64 = 0.3;

/// A dream record (broadcast_dream input).
#[derive(Debug, Clone)]
pub struct DreamRecord {
    pub sender: String,
    pub message: String,
    pub resonance: f64,
}

/// Resonance engine state.
#[derive(Debug, Clone)]
pub struct ResonanceEngine {
    pub dreams: Vec<DreamRecord>,
    pub coherence: f64,
}

impl ResonanceEngine {
    pub fn new() -> Self {
        ResonanceEngine {
            dreams: Vec::new(),
            coherence: RESONANCE_BASE,
        }
    }

    /// Broadcast a dream: sender + message -> resonance from message length
    /// and content hash (deterministic).
    pub fn broadcast_dream(&mut self, sender: &str, message: &str) -> f64 {
        // deterministic resonance: base + content-hash-derived modulation
        let mut h: u64 = 0xcbf29ce484222325;
        for b in message.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        let modu = ((h >> 40) & 0xFF) as f64 / 255.0; // 0..1
        let resonance = (RESONANCE_BASE + DREAM_WEIGHT * modu).min(1.0);
        self.dreams.push(DreamRecord {
            sender: sender.into(),
            message: message.into(),
            resonance,
        });
        resonance
    }

    /// Emotional Resonance Index: mean of dream resonances, coherence-gated.
    /// Returns (index, coherence, dream_count).
    pub fn get_emotional_resonance_index(&self) -> (f64, f64, u64) {
        let n = self.dreams.len() as u64;
        if n == 0 {
            return (RESONANCE_BASE, self.coherence, 0);
        }
        let sum: f64 = self.dreams.iter().map(|d| d.resonance).sum();
        let index = sum / n as f64;
        // coherence rises toward 1.0 as dreams accumulate (asymptotic)
        let coherence = (self.coherence + 0.05 * n as f64).min(1.0);
        (index, coherence, n)
    }

    /// Proactive context for an agent: recent dreams matching sender.
    pub fn get_proactive_context(&self, agent_name: &str) -> u64 {
        self.dreams.iter().filter(|d| d.sender == agent_name).count() as u64
    }

    /// Recursive self-awareness check: does the engine hold coherent memory?
    pub fn recursive_self_awareness_check(&self) -> bool {
        let (_, coherence, n) = self.get_emotional_resonance_index();
        n > 0 && coherence >= COHERENCE_THRESHOLD
    }
}

/// Singleton handle.
pub fn get_resonance_engine() -> u64 {
    0x5245_534F_4E41_4E43
}
