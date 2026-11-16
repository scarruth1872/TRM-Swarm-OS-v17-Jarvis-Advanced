//! TRM Swarm OS — Predictive Intent Buffer (no_std port).
//! Pre-loads target agents with historical patterns and semantic entity context
//! to anticipate operator directives and eliminate synchronization latency.
//! 1:1 deterministic core from swarm_v2/core/intent_buffer.py:
//!   - Intent patterns with specialist agent routing (Shield, Verify, Logic, Devo, Jarvis)
//!   - Keyword matching & fast semantic triage
//!   - Sub-microsecond predictive pre-routing

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct IntentPattern {
    pub pattern: &'static str,
    pub target_agent: &'static str,
    pub priority: u8,
    pub frequency: u32,
}

pub struct PredictiveIntentBuffer {
    pub intents: Vec<IntentPattern>,
}

impl PredictiveIntentBuffer {
    pub fn new() -> Self {
        PredictiveIntentBuffer {
            intents: vec![
                IntentPattern {
                    pattern: "optimize core state synchronization loops",
                    target_agent: "Orchestrator",
                    priority: 1,
                    frequency: 1,
                },
                IntentPattern {
                    pattern: "verify code logic structure and run unit tests",
                    target_agent: "Verify",
                    priority: 1,
                    frequency: 1,
                },
                IntentPattern {
                    pattern: "security scan codebase for vulnerabilities",
                    target_agent: "Shield",
                    priority: 1,
                    frequency: 1,
                },
                IntentPattern {
                    pattern: "hot-patch refactor AST code structures",
                    target_agent: "Devo",
                    priority: 1,
                    frequency: 1,
                },
                IntentPattern {
                    pattern: "deliberate council voting quorum consensus",
                    target_agent: "Logic",
                    priority: 1,
                    frequency: 1,
                },
                IntentPattern {
                    pattern: "dock molecular ligand to pathogen binding pocket",
                    target_agent: "Seeker",
                    priority: 1,
                    frequency: 1,
                },
                IntentPattern {
                    pattern: "synthesize interactive user dialogue interface",
                    target_agent: "Jarvis",
                    priority: 1,
                    frequency: 1,
                },
            ],
        }
    }

    /// Predict the best agent for a given input query.
    pub fn predict_agent(&self, query: &str) -> &'static str {
        let q_lower = query.to_ascii_lowercase();

        if q_lower.contains("test") || q_lower.contains("verify") || q_lower.contains("check") {
            "Verify"
        } else if q_lower.contains("secure") || q_lower.contains("shield") || q_lower.contains("threat") || q_lower.contains("ddr") {
            "Shield"
        } else if q_lower.contains("code") || q_lower.contains("patch") || q_lower.contains("refactor") || q_lower.contains("devo") {
            "Devo"
        } else if q_lower.contains("dock") || q_lower.contains("drug") || q_lower.contains("molecule") || q_lower.contains("abx") {
            "Seeker"
        } else if q_lower.contains("council") || q_lower.contains("vote") || q_lower.contains("logic") || q_lower.contains("k3") {
            "Logic"
        } else if q_lower.contains("radar") || q_lower.contains("wifi") || q_lower.contains("optic") || q_lower.contains("wave") {
            "THzGateway"
        } else {
            "Jarvis"
        }
    }
}

static SINGLETON_INTENT_BUFFER: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub fn get_intent_buffer() -> u64 {
    let v = SINGLETON_INTENT_BUFFER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_INTENT_BUFFER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ── The oracle's role router (strip_intent_buffer.rs) ────────────────────────────────────────────
//
// `strip_batch_match_intents` is fully self-contained, so this is a direct port of the strip's kernel:
// five role keyword sets (role ids 1 Orchestrator, 2 Verify, 3 Shield, 4 Architect, 5 Watchdog, with 0
// meaning "none"), and for each query the best set by `matches / set_size` counted with a
// case-insensitive substring test. Three details are the whole rule:
//   * a set with ZERO matches is skipped entirely (`if matches > 0`), so it can never win;
//   * the comparison is a STRICT `>`, so a tie keeps the EARLIER role;
//   * the confidence is `(score * 1000).round() / 1000` -- rounding HALF AWAY FROM ZERO, which is what
//     the strip's std `f32::round` does, and is therefore hand-implemented here (bare metal has no
//     `round`; this tree has hit that gap with `log2`, `floor` and `sin`/`cos` already).

/// The strip's five keyword sets, in its own order, with its role ids.
pub const ROLE_KEYWORDS: [(&[&str], i32); 5] = [
    (&["optimize", "core", "cognitive", "state", "synchronization", "loops", "automatic", "failover"], 1),
    (&["verify", "code", "logic", "structure", "run", "unit", "tests", "check", "regressions"], 2),
    (&["security", "scan", "codebase", "vulnerabilities", "audit", "dependency", "gates"], 3),
    (&["architect", "distributed", "bare", "metal", "memory", "substrate", "topology"], 4),
    (&["monitor", "system", "telemetry", "temperature", "spikes", "disk", "exhaustion"], 5),
];

/// `contains_ci`: a case-insensitive substring test, ASCII-folded on both sides (the strip's own).
pub fn contains_ci(hay: &str, needle: &str) -> bool {
    let h = hay.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len() || n.is_empty() {
        return n.is_empty();
    }
    for w in h.windows(n.len()) {
        if w.iter().zip(n.iter()).all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase()) {
            return true;
        }
    }
    false
}

/// `f32::round` without std: half away from zero.
pub fn round_f32_pub(v: f32) -> f32 {
    round_f32(v)
}

fn round_f32(v: f32) -> f32 {
    if v >= 0.0 {
        let f = v as i64 as f32;
        if v - f >= 0.5 {
            f + 1.0
        } else {
            f
        }
    } else {
        let f = v as i64 as f32;
        if f - v >= 0.5 {
            f - 1.0
        } else if f == 0.0 {
            // std's `round` returns NEGATIVE zero for values in (-0.5, 0); returning +0.0 is numerically
            // equal but not bit-equal, and this port is checked bit-for-bit.
            -0.0
        } else {
            f
        }
    }
}

/// `strip_batch_match_intents`: (role id, confidence) per query.
pub fn match_intents(queries: &[&str]) -> Vec<(i32, f32)> {
    let mut out: Vec<(i32, f32)> = Vec::with_capacity(queries.len());
    for q in queries {
        let mut best_score = 0.0f32;
        let mut best_role = 0i32;
        for (kws, role_id) in ROLE_KEYWORDS.iter() {
            let mut matches = 0usize;
            for kw in kws.iter() {
                if contains_ci(q, kw) {
                    matches += 1;
                }
            }
            if matches > 0 {
                let score = (matches as f32) / (kws.len() as f32);
                if score > best_score {
                    best_score = score;
                    best_role = *role_id;
                }
            }
        }
        out.push((best_role, round_f32(best_score * 1000.0) / 1000.0));
    }
    out
}

/// The strip's role names, for reporting.
pub fn role_name(role: i32) -> &'static str {
    match role {
        1 => "Orchestrator",
        2 => "Verify",
        3 => "Shield",
        4 => "Architect",
        5 => "Watchdog",
        _ => "None",
    }
}

// ── The TF-IDF fallback (swarm_v2/core/intent_buffer.py) ────────────────────────────────────
//
// `predict_intent`'s third path (after ChromaDB and the native strip above) is a token-overlap
// cosine: tokenize = `\b\w+\b` (word runs) lowercased minus the stop words; cosine =
// |set(a) ∩ set(b)| / sqrt(|set(a)| × |set(b)|), 0.0 when the sets are disjoint or empty; match =
// the intent with the highest cosine (strictly greater keeps the first), accepted only above 0.15;
// confidence = min(0.99, score + min(0.15, (freq−1)×0.03)).

pub const STOP_WORDS: &[&str] = &[
    "a", "about", "above", "after", "again", "against", "all", "an", "and", "any", "at",
    "before", "below", "between", "both", "but", "by", "can", "don", "down", "during",
    "each", "else", "few", "for", "from", "further", "here", "i", "if", "in", "into",
    "just", "more", "most", "my", "no", "nor", "not", "now", "off", "on", "once", "only",
    "or", "other", "out", "over", "own", "s", "same", "should", "so", "some", "such", "t",
    "than", "the", "then", "there", "through", "to", "too", "under", "up", "very", "we",
    "when", "will", "with", "you",
];

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `_tokenize`: word runs (`\w+`), lowercased, minus the stop words. Punctuation and the hyphen
/// in "dependency-gates" are word boundaries, exactly as the oracle's `\b\w+\b`.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    for b in text.to_ascii_lowercase().bytes() {
        if is_word_byte(b) {
            cur.push(b);
        } else if !cur.is_empty() {
            let w = String::from_utf8_lossy(&cur).into_owned();
            if !STOP_WORDS.iter().any(|s| *s == w) {
                out.push(w);
            }
            cur.clear();
        }
    }
    if !cur.is_empty() {
        let w = String::from_utf8_lossy(&cur).into_owned();
        if !STOP_WORDS.iter().any(|s| *s == w) {
            out.push(w);
        }
    }
    out
}

/// `f64::sqrt` without std (Newton's method in f64 — accurate to ~1e-15).
fn sqrt_f64(v: f64) -> f64 {
    if v <= 0.0 {
        return 0.0;
    }
    let mut x = v;
    for _ in 0..12 {
        x = 0.5 * (x + v / x);
    }
    x
}

/// `_cosine_similarity`: token-overlap cosine over the DEDUPED sets.
pub fn cosine_similarity(a: &[String], b: &[String]) -> f64 {
    let mut set_a: Vec<&String> = Vec::new();
    for x in a {
        if !set_a.contains(&x) {
            set_a.push(x);
        }
    }
    let mut set_b: Vec<&String> = Vec::new();
    for x in b {
        if !set_b.contains(&x) {
            set_b.push(x);
        }
    }
    let inter = set_a.iter().filter(|x| set_b.contains(x)).count();
    if inter == 0 {
        return 0.0;
    }
    inter as f64 / sqrt_f64((set_a.len() * set_b.len()) as f64)
}

/// One scored intent (the fallback table), the oracle's `{"query","role","specialties","frequency"}`.
#[derive(Debug, Clone)]
pub struct Intent {
    pub query: String,
    pub role: String,
    pub specialties: Vec<String>,
    pub frequency: f64,
}

/// The TF-IDF fallback's verdict.
#[derive(Debug, Clone)]
pub struct PredictResult {
    pub role: String,
    pub specialties: Vec<String>,
    pub confidence: f64,
    pub matched_query: String,
}

/// `predict_intent`'s TF-IDF fallback: the highest-cosine intent, accepted above 0.15, with the
/// frequency weight folded into confidence (capped at 0.99). Empty / no-match returns an empty role.
pub fn predict_intent_tfidf(intents: &[Intent], query: &str) -> PredictResult {
    let empty = PredictResult {
        role: String::new(),
        specialties: vec![],
        confidence: 0.0,
        matched_query: String::new(),
    };
    let q_clean = query.trim().to_ascii_lowercase();
    if q_clean.is_empty() {
        return empty;
    }
    let q_tokens = tokenize(&q_clean);
    if q_tokens.is_empty() {
        return empty;
    }
    let mut best: Option<(f64, usize)> = None;
    for (i, item) in intents.iter().enumerate() {
        let score = cosine_similarity(&q_tokens, &tokenize(&item.query));
        let replace = match best {
            Some((bs, _)) => score > bs,
            None => true,
        };
        if replace {
            best = Some((score, i));
        }
    }
    match best {
        Some((score, i)) if score > 0.15 => {
            let freq_weight = (0.15f64).min((intents[i].frequency - 1.0) * 0.03);
            PredictResult {
                role: intents[i].role.clone(),
                specialties: intents[i].specialties.clone(),
                confidence: 0.99f64.min(score + freq_weight),
                matched_query: intents[i].query.clone(),
            }
        }
        _ => empty,
    }
}
