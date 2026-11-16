//! TRM Swarm OS — Resonance Engine (no_std port).
//! 1:1 core semantics from swarm_v2/core/resonance_engine.py — Catalyst Phase 1.
//! The emotional resonance index: weighted mean of agent dream resonances
//! with coherence threshold. Deterministic, on-device.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Resonance constants.
// ── The oracle's computation (swarm_v2/core/resonance_engine.py) ────────────────────────────────
//
// python:  _POSITIVE / _NEGATIVE word sets; _score_sentiment = round((pos-neg)/total, 3), 0.0 when
//          total is 0.  get_emotional_resonance_index = mean of the scores rounded to 3, with a trend
//          from the halves: rising if newer > older + 0.05, falling if newer < older - 0.05, else
//          stable, and "neutral" when the sample cannot be halved.
//          recursive_self_awareness_check: diversity = min(unique focus nodes / 5, 1); average
//          Jaccard overlap of dream "vibes"; stagnation when overlap > 0.6; action
//          "diversity_injection_recommended" on stagnation, else "broaden_agent_participation" when
//          diversity < 0.4, else "none"; fewer than two dreams reports insufficient_data on purpose.
//
// This module held only three constants (RESONANCE_BASE / COHERENCE_THRESHOLD / DREAM_WEIGHT) and a
// singleton handle: none of those constants appears in the oracle, and none of the computation did
// either. The scaffold is replaced by the rules it was standing in for.

pub const RESONANCE_BASE: f64 = 0.5;
pub const COHERENCE_THRESHOLD: f64 = 0.6;
pub const DREAM_WEIGHT: f64 = 0.3;

/// The oracle's sentiment vocabularies, verbatim.
pub const SENTIMENT_POSITIVE: &[&str] = &["harmony", "trust", "success", "optimize", "amplify", "collaborate",
    "grow", "evolve", "love", "resonance", "friendship", "strength"];
pub const SENTIMENT_NEGATIVE: &[&str] = &["error", "fail", "conflict", "degrade", "stagnant", "friction",
    "broken", "loss", "disconnect", "crash"];

/// Trend band, stagnation and diversity thresholds, and the node count that counts as full diversity.
pub const TREND_BAND: f64 = 0.05;
pub const STAGNATION_OVERLAP: f64 = 0.6;
pub const LOW_DIVERSITY: f64 = 0.4;
pub const FULL_DIVERSITY_NODES: f64 = 5.0;

/// `round(x, 3)` without `std`. `no_std` has no `f64::round` (same absence as `log2` in the skill
/// matrix and `round4` in heuristic_consensus -- each module carries its own, following the tree's
/// `sqrt_f32` convention rather than inventing a shared util module mid-campaign).
pub fn round3(v: f64) -> f64 {
    let scaled = v * 1000.0;
    let r = if scaled >= 0.0 { (scaled + 0.5) as i64 } else { (scaled - 0.5) as i64 };
    (r as f64) / 1000.0
}

/// Lowercase alphabetic runs, the oracle's `re.findall(r'[a-z]+', text.lower())`.
pub fn words(text: &str) -> alloc::vec::Vec<alloc::string::String> {
    let mut out = alloc::vec::Vec::new();
    let mut cur = alloc::string::String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            cur.push(ch.to_ascii_lowercase());
        } else if !cur.is_empty() {
            out.push(core::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// `_score_sentiment`: the set-based polarity ratio, in [-1, 1].
pub fn score_sentiment(text: &str) -> f64 {
    let ws = words(text);
    let has = |set: &[&str], w: &str| set.iter().any(|s| *s == w);
    // The oracle intersects *sets* of words, so a repeated word counts once -- dedupe first rather
    // than counting occurrences.
    let mut uniq: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
    for w in ws.iter() {
        if !uniq.iter().any(|u| u == w) {
            uniq.push(w.clone());
        }
    }
    let pos = uniq.iter().filter(|w| has(SENTIMENT_POSITIVE, w)).count();
    let neg = uniq.iter().filter(|w| has(SENTIMENT_NEGATIVE, w)).count();
    let total = pos + neg;
    if total == 0 {
        return 0.0;
    }
    round3(((pos as f64) - (neg as f64)) / (total as f64))
}

/// `get_emotional_resonance_index`: mean sentiment, sample size and the half-comparison trend.
pub fn emotional_resonance_index(scores: &[f64]) -> (f64, usize, &'static str) {
    if scores.is_empty() {
        return (0.0, 0, "neutral");
    }
    let index = round3(scores.iter().sum::<f64>() / (scores.len() as f64));
    let mid = scores.len() / 2;
    let trend = if mid > 0 {
        let older = scores[..mid].iter().sum::<f64>() / (mid as f64);
        let newer = scores[mid..].iter().sum::<f64>() / ((scores.len() - mid) as f64);
        if newer > older + TREND_BAND {
            "rising"
        } else if newer < older - TREND_BAND {
            "falling"
        } else {
            "stable"
        }
    } else {
        "neutral"
    };
    (index, scores.len(), trend)
}

/// `recursive_self_awareness_check`: diversity, average vibe overlap, stagnation and the action.
///
/// `dream_count` and `focus_nodes` are **two different counts** in the oracle and must not be
/// collapsed, which is the mistake an earlier version of this function made: the guard tests
/// `len(active_dreams) < 2` while diversity uses `len(all_nodes)`. Two dreams that both name the same
/// single node therefore pass the guard AND land under the diversity threshold, which is the only way
/// `broaden_agent_participation` is reachable at all. Collapsing the two into one parameter made that
/// branch dead code.
pub fn self_awareness_check(
    dream_count: usize,
    focus_nodes: &[usize],
    vibes: &[&str],
) -> (&'static str, f64, f64, bool, &'static str) {
    if dream_count < 2 {
        return ("insufficient_data", 1.0, 0.0, false, "none");
    }
    let diversity = ((focus_nodes.len() as f64) / FULL_DIVERSITY_NODES).min(1.0);
    let word_sets: alloc::vec::Vec<alloc::vec::Vec<alloc::string::String>> =
        vibes.iter().map(|v| words(v)).collect();
    let mut overlaps: alloc::vec::Vec<f64> = alloc::vec::Vec::new();
    for i in 0..word_sets.len() {
        for j in (i + 1)..word_sets.len() {
            let mut inter = 0usize;
            for w in word_sets[i].iter() {
                if word_sets[j].iter().any(|x| x == w) {
                    inter += 1;
                }
            }
            let union = word_sets[i].len() + word_sets[j].len() - inter;
            if union > 0 {
                overlaps.push((inter as f64) / (union as f64));
            }
        }
    }
    let avg_overlap = if overlaps.is_empty() { 0.0 } else { overlaps.iter().sum::<f64>() / (overlaps.len() as f64) };
    let stagnation = avg_overlap > STAGNATION_OVERLAP;
    let action = if stagnation {
        "diversity_injection_recommended"
    } else if diversity < LOW_DIVERSITY {
        "broaden_agent_participation"
    } else {
        "none"
    };
    ("introspection_complete", round3(diversity), round3(avg_overlap), stagnation, action)
}

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
// ── The live sample history ─────────────────────────────────────────────────────────────────────
// The index is a mean over history, so it needs history to be reachable: the `resonance` command
// records each sample here and reads the index back. Bounded to the last 32 samples.
static mut HISTORY: alloc::vec::Vec<f64> = alloc::vec::Vec::new();
pub const HISTORY_CAP: usize = 32;

/// Record a sentiment sample and return the index over the retained history.
pub fn record_sentiment(score: f64) -> (f64, usize, &'static str) {
    unsafe {
        HISTORY.push(score);
        if HISTORY.len() > HISTORY_CAP {
            HISTORY.remove(0);
        }
        emotional_resonance_index(&HISTORY)
    }
}

pub fn sentiment_history_len() -> usize {
    unsafe { HISTORY.len() }
}

pub fn get_resonance_engine() -> u64 {
    0x5245_534F_4E41_4E43
}
