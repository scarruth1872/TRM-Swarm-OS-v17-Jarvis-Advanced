//! TRM Swarm OS — Jarvis Cognitive Toolkit (no_std port).
//! 1:1 from Jarvis-Advanced-main:
//!  - scripts/bonsai_router.py: complexity classification (math_words,
//!    query length -> casual/reasoning/deep routing)
//!  - core.py: JARVIS cognitive tool schema (terminal + file tools)
//!  - scripts/local_codebase_indexer.py: codebase scan/parse stats
//!  - turbovec_search.py: normalized cosine search over item vectors

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Bonsai router complexity classification.
/// Matches bonsai_router.py: casual (<35 chars + greeting words),
/// reasoning (math words or len>250), else default.
pub const MATH_WORDS: &'static [&'static str] = &[
    "solve", "probability", "cohomology", "recurrent", "consensus",
    "integral", "formula", "equation", "balls",
];
pub const GREETING_WORDS: &'static [&'static str] = &[
    "hello", "hi", "hey", "greet", "how are you", "who are you",
];

/// Route classification: 0=casual(1.7B), 1=default(4B), 2=reasoning(8B).
pub fn classify_complexity(query: &str) -> u64 {
    let qlen = query.len() as u64;
    let lower = query.to_lowercase();
    let is_reasoning = MATH_WORDS.iter().any(|w| lower.contains(w)) || qlen > 250;
    let is_casual = qlen < 35 && GREETING_WORDS.iter().any(|w| lower.contains(w));
    if is_casual {
        0
    } else if is_reasoning {
        2
    } else {
        1
    }
}

/// JARVIS cognitive tool schema (from core.py).
/// Two tools: run_terminal_command, read_file_contents.
pub const COGNITIVE_TOOLS: &'static [&'static str] = &[
    "run_terminal_command — Executes a terminal or shell command on the host computer",
    "read_file_contents — Reads the entire contents of any file on the local computer",
];

/// Codebase indexer: count files + languages scanned.
/// Matches local_codebase_indexer.py scan stats.
pub fn codebase_scan_stats(files_by_ext: &[(&str, u64)]) -> (u64, u64) {
    let total: u64 = files_by_ext.iter().map(|(_, n)| n).sum();
    (total, files_by_ext.len() as u64)
}

/// TurboVec normalized-cosine search: L2-normalize rows + query, dot-product.
/// Matches turbovec_search.py. Returns the best-matching row index.
pub fn normalized_cosine_search(query: &[f32], rows: &[Vec<f32>]) -> u64 {
    let q_norm = l2_norm(query);
    let mut best_idx = 0u64;
    let mut best_score = -1.0f32;
    for (i, row) in rows.iter().enumerate() {
        let r_norm = l2_norm(row);
        if q_norm == 0.0 || r_norm == 0.0 {
            continue;
        }
        let mut dot = 0.0f32;
        for (a, b) in query.iter().zip(row.iter()) {
            dot += a * b;
        }
        let score = dot / (q_norm * r_norm);
        if score > best_score {
            best_score = score;
            best_idx = i as u64;
        }
    }
    best_idx
}

fn l2_norm(v: &[f32]) -> f32 {
    let mut s = 0.0f32;
    for x in v {
        s += x * x;
    }
    s.sqrt_approx()
}

/// no_std sqrt (Newton-Raphson on bit representation).
trait SqrtApprox {
    fn sqrt_approx(self) -> f32;
}
impl SqrtApprox for f32 {
    fn sqrt_approx(self) -> f32 {
        if self <= 0.0 {
            return 0.0;
        }
        let bits = self.to_bits();
        let seed_bits = ((bits + (127u32 << 23)) >> 1) as u32;
        let mut x = f32::from_bits(seed_bits);
        for _ in 0..4 {
            x = 0.5 * (x + self / x);
        }
        x
    }
}
