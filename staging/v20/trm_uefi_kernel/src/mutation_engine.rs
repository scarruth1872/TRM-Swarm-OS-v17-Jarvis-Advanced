//! TRM Swarm OS - mutation_engine (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/mutation_engine.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;


//  GeneticProposal 
#[derive(Debug, Clone)]
pub struct GeneticProposal {
    pub proposal_id: String,
    pub target_file: String,
    pub mutation_type: String,
    pub description: String,
    pub original_code: String,
    pub mutated_code: String,
    pub timestamp: u64,
    pub score: f64,
    pub status: String,
}

impl GeneticProposal {
    pub fn new() -> Self {
        GeneticProposal {
            proposal_id: String::new(),
            target_file: String::new(),
            mutation_type: String::new(),
            description: String::new(),
            original_code: String::new(),
            mutated_code: String::new(),
            timestamp: 0,
            score: 0.0,
            status: String::new(),
        }
    }
}

//  get_mutation_engine() 
static SINGLETON_MUTATION_ENGINE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_mutation_engine() -> u64 {
    // Singleton handle (lazy-init pattern from mutation_engine.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_MUTATION_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_MUTATION_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ── The strip's model (native_voltage_strips/strip_mutation_engine.rs) ───────────────────────────
//
// `strip_batch_mutation_score` scores a code-mutation proposal from four figures:
//     diff      = |mut_len - orig_len| / orig_len
//     len_ratio = max(0, 1 - diff)
//     s         = test_cov * 0.4 + len_ratio * 0.3 + (1 - comp_delta) * 0.3
//     trit      = +1 if s >= 0.75 else (0 if s >= 0.50 else -1)
//
// One difference worth naming: the strip returns the RAW score, while the python's fallback rounds it
// to three places before returning. Both are reproduced as they are -- `score_proposals_batch_native`
// below is the strip's, and `score_proposals_rounded` is the python fallback's.

/// Weights, from the oracle's own arithmetic.
pub const MUT_COV_WEIGHT: f64 = 0.4;
pub const MUT_LEN_WEIGHT: f64 = 0.3;
pub const MUT_COMP_WEIGHT: f64 = 0.3;

/// `round(x, 3)` without std.
pub fn round3(v: f64) -> f64 {
    let s = v * 1000.0;
    let r = if s >= 0.0 { (s + 0.5) as i64 } else { (s - 0.5) as i64 };
    (r as f64) / 1000.0
}

/// One proposal's raw score, exactly as the strip computes it.
pub fn mutation_score(orig_len: f64, mut_len: f64, comp_delta: f64, test_cov: f64) -> f64 {
    let diff = (mut_len - orig_len).abs() / orig_len;
    let len_ratio = if 1.0 - diff > 0.0 { 1.0 - diff } else { 0.0 };
    (test_cov * MUT_COV_WEIGHT) + (len_ratio * MUT_LEN_WEIGHT) + ((1.0 - comp_delta) * MUT_COMP_WEIGHT)
}

/// The strip's trit thresholds: `+1` at or above 0.75, `0` at or above 0.50, else `-1`.
pub fn mutation_trit(s: f64) -> i8 {
    if s >= 0.75 {
        1
    } else if s >= 0.50 {
        0
    } else {
        -1
    }
}

/// Batch entry point, mirroring `strip_batch_mutation_score`: (scores, trits).
pub fn score_proposals_batch_native(orig: &[f64], mut_: &[f64], comps: &[f64], covs: &[f64]) -> (Vec<f64>, Vec<i8>) {
    let n = orig.len();
    let mut scores = Vec::with_capacity(n);
    let mut trits = Vec::with_capacity(n);
    for i in 0..n {
        let s = mutation_score(orig[i], mut_[i], comps[i], covs[i]);
        scores.push(s);
        trits.push(mutation_trit(s));
    }
    (scores, trits)
}

/// The python fallback's variant, which rounds the score to three places.
pub fn score_proposals_rounded(orig_len: f64, mut_len: f64, comp_delta: f64, test_cov: f64) -> (f64, i8) {
    let s = round3(mutation_score(orig_len, mut_len, comp_delta, test_cov));
    (s, mutation_trit(mutation_score(orig_len, mut_len, comp_delta, test_cov)))
}

// ── extract_mutation_code (Phase-16 "Robust Code Extraction (v3)" pipeline) ──────────────
//
// Ported 1:1 from `MutationEngine.propose_mutation` (oracle lines 206-236): Pass 1 identifies
// the most likely code block in an agent's raw response, Pass 2 strips `[START]` / `[END]` /
// `WRITE_FILE:` markers (including nested repetitions), Pass 3 removes any remaining markdown
// fences. Pure string surgery over the response — no semantic validation.
//
// Index semantics, matched exactly against the oracle's `str` calls:
//   * `str.split(sep)` (no maxsplit) splits on EVERY occurrence, then `[i]` selects a segment
//     -> `str::split(sep).nth(i)`.   (`str::split` yields an empty first segment when the
//     subject starts with the separator, exactly as Python does.)
//   * `rsplit("[START]", 1)[-1]`  -> `rsplit_once("[START]")`: everything AFTER the LAST marker.
//   * `split("[END]")[0]`         -> `split_once("[END]")`: everything BEFORE the FIRST marker.
//   * `split("\n", 1)[1]`         -> `split_once('\n')`: everything after the first newline.
//   * `.strip()`                  -> `.trim()` (identical for the ASCII whitespace the oracle sees).
//
// Host-side and NOT ported here (listed, not faked):
//   * the `ast.parse` syntax-validity check and the datetime-stamped fallback header that
//     wrap this pipeline in `propose_mutation` (ast / datetime are host-only).
//   * the expert-agent `process_task` calls that produce the raw response in the first place.
pub fn extract_mutation_code(mutated_code: &str) -> String {
    let mut extracted = String::from(mutated_code);

    // Pass 1: identify the most likely code block.
    if extracted.contains("```python") {
        // split("```python")[1] -> split("```")[0] -> strip()
        if let Some(mid) = extracted.split("```python").nth(1) {
            let first = mid.split("```").next().unwrap_or(mid);
            extracted = first.trim().to_string();
        }
    } else if extracted.contains("WRITE_FILE:") {
        // split("WRITE_FILE:")[1]
        if let Some(after) = extracted.split("WRITE_FILE:").nth(1) {
            extracted = after.to_string();
        }
        if extracted.contains('\n') {
            // split("\n", 1)[1].strip()
            extracted = extracted
                .split_once('\n')
                .map(|(_, rest)| rest.trim().to_string())
                .unwrap_or(extracted);
        }
    } else if extracted.contains("```") {
        // split("```")[1] -> split("```")[0] -> strip(); keep only if it looks like code
        let potential: String = extracted
            .split("```")
            .nth(1)
            .and_then(|mid| mid.split("```").next())
            .unwrap_or("")
            .trim()
            .to_string();
        if potential.contains("import ") || potential.contains("def ") || potential.contains("class ") {
            extracted = potential;
        }
    }

    // Pass 2: aggressive marker stripping (handles nesting/repetitions).
    for marker in ["[START]", "[END]", "WRITE_FILE:"] {
        if !extracted.contains(marker) {
            continue;
        }
        match marker {
            "[START]" => {
                // rsplit("[START]", 1)[-1] = everything AFTER the LAST "[START]"
                if let Some((_, after)) = extracted.rsplit_once("[START]") {
                    extracted = after.trim().to_string();
                }
            }
            "[END]" => {
                // split("[END]")[0] = everything BEFORE the FIRST "[END]"
                if let Some((before, _)) = extracted.split_once("[END]") {
                    extracted = before.trim().to_string();
                }
            }
            "WRITE_FILE:" => {
                if extracted.contains('\n') {
                    // split("\n", 1)[1].strip()
                    if let Some((_, rest)) = extracted.split_once('\n') {
                        extracted = rest.trim().to_string();
                    }
                }
            }
            _ => {}
        }
    }

    // Pass 3: clean up any remaining markdown fences.
    if extracted.contains("```") {
        extracted = extracted.replace("```python", "").replace("```", "");
        extracted = extracted.trim().to_string();
    }

    extracted
}

/// `is_python_target` — the pure filename heuristic from `analyze_codebase`:
/// `name.endswith(".py") and "__" not in name`. The companion `os.path.getsize(path) > 15000`
/// size gate is HOST-SIDE (file I/O) and excluded from this port.
pub fn is_python_target(filename: &str) -> bool {
    filename.ends_with(".py") && !filename.contains("__")
}
