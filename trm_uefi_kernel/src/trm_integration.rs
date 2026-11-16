//! TRM Swarm OS — trm_integration (no_std port).
//! Deterministic core ported 1:1 from swarm_v2/core/trm_integration.py:
//! task-complexity analysis (keyword + length bands), parallel-vs-sequential
//! decision, symbolic detection (0.6 numeric/separator ratio), synthesis
//! quality scoring, speedup/EMA metric updates, recursion-depth clamp, the
//! TRMEnhancedAgent configuration defaults and the factory functions.
//! Async/LLM/psutil bodies are // MANUAL (no_std has no executor or LLM).

#![allow(dead_code)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Complexity bands returned by _analyze_task_complexity (python strings).
pub const CMP_SYMBOLIC_HIGH: u8 = 0; // "symbolic_high"
pub const CMP_TEXT_HIGH: u8 = 1; // "text_high"
pub const CMP_TEXT_MEDIUM: u8 = 2; // "text_medium"
pub const CMP_TEXT_LOW: u8 = 3; // "text_low"

/// Processing-mode discriminators used in result dicts.
pub const MODE_PARALLEL: u8 = 0; // "parallel"
pub const MODE_SEQUENTIAL: u8 = 1; // "sequential"
pub const MODE_PARALLEL_FAILED: u8 = 2; // "parallel_failed"
pub const MODE_PARALLEL_AGGREGATED: u8 = 3; // "parallel_aggregated"

/// Result type discriminators (symbolic / text / error).
pub const RES_SYMBOLIC: u8 = 0;
pub const RES_TEXT: u8 = 1;
pub const RES_ERROR: u8 = 2;

/// Deterministic replacement for Python's `hash(part)` fallback (see
/// trm_orchestrator::hash_mod10). Kept local so modules stay independent.
fn hash_mod10(s: &str) -> i64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (h % 10) as i64
}

// ---------------------------------------------------------------------------
// TRMEnhancedAgent configuration + metrics (deterministic state core)
// ---------------------------------------------------------------------------

/// TRMEnhancedAgent trm_metrics dict as a plain struct.
#[derive(Clone, Debug)]
pub struct TrmMetrics {
    pub total_trm_tasks: u64,
    pub parallel_executions: u64,
    pub avg_speedup: f64,
    pub error_reduction: f64,
}

impl Default for TrmMetrics {
    fn default() -> Self {
        TrmMetrics {
            total_trm_tasks: 0,
            parallel_executions: 0,
            avg_speedup: 1.0,
            error_reduction: 0.0,
        }
    }
}

/// TRM-enhanced agent configuration snapshot.
#[derive(Clone, Debug)]
pub struct TrmAgentConfig {
    pub agent_id: u64,
    pub parallel_enabled: bool,
    pub max_recursion_depth: u64,
    pub confidence_threshold: f64,
}

/// Agent identity used by the factories: persona handle + skill list.
#[derive(Clone, Debug)]
pub struct AgentIdentity {
    pub persona: u64,
    pub skills: Vec<u64>,
}

// ---------------------------------------------------------------------------
// _analyze_task_complexity — deterministic keyword + length classifier
// ---------------------------------------------------------------------------

/// High-complexity indicators from the python source.
pub const HIGH_COMPLEXITY_KEYWORDS: [&str; 10] = [
    "analyze", "compare", "evaluate", "explain", "interpret", "solve",
    "calculate", "reason", "deduce", "infer",
];

/// Symbolic-processing indicators from the python source.
pub const SYMBOLIC_KEYWORDS: [&str; 9] = [
    "pattern", "sequence", "tokens", "symbol", "logic", "puzzle", "riddle",
    "algorithm", "formula",
];

/// _analyze_task_complexity: returns one of the CMP_* constants.
pub fn analyze_task_complexity(task: &str) -> u8 {
    let task_lower = task.to_lowercase();
    let high_complexity = HIGH_COMPLEXITY_KEYWORDS
        .iter()
        .any(|kw| task_lower.contains(kw));
    let symbolic = SYMBOLIC_KEYWORDS.iter().any(|kw| task_lower.contains(kw));
    let len = task.chars().count();

    if symbolic {
        CMP_SYMBOLIC_HIGH
    } else if high_complexity && len > 200 {
        CMP_TEXT_HIGH
    } else if len > 500 {
        CMP_TEXT_MEDIUM
    } else {
        CMP_TEXT_LOW
    }
}

// ---------------------------------------------------------------------------
// _should_use_parallel — deterministic decision (resource check injected)
// ---------------------------------------------------------------------------

/// _should_use_parallel with the psutil resource probe replaced by an
/// injected `resources_constrained` flag (// MANUAL in no_std).
pub fn should_use_parallel(complexity: u8, task: &str, resources_constrained: bool) -> bool {
    if resources_constrained {
        return false;
    }
    let len = task.chars().count();
    match complexity {
        CMP_SYMBOLIC_HIGH | CMP_TEXT_HIGH => len > 100,
        CMP_TEXT_MEDIUM => len > 300,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// _looks_like_symbolic — 0.6 numeric/separator ratio (text variant)
// ---------------------------------------------------------------------------

/// _looks_like_symbolic of TRMEnhancedAgent: digits, whitespace and
/// ",.;:|" separators; ratio > 0.6.
pub fn looks_like_symbolic(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let total = text.chars().count() as f64;
    if total == 0.0 {
        return false;
    }
    let mut symbolic_chars = 0u64;
    for c in text.chars() {
        if c.is_ascii_digit()
            || c.is_ascii_whitespace()
            || c == ','
            || c == '.'
            || c == ';'
            || c == ':'
            || c == '|'
        {
            symbolic_chars += 1;
        }
    }
    (symbolic_chars as f64) / total > 0.6
}

/// _context_to_tokens: whitespace split with int()/hash%10 fallback.
pub fn context_to_tokens(context: &str) -> Vec<i64> {
    let mut tokens = vec![];
    for part in context.split_whitespace() {
        match part.parse::<i64>() {
            Ok(v) => tokens.push(v),
            Err(_) => tokens.push(hash_mod10(part)),
        }
    }
    tokens
}

// ---------------------------------------------------------------------------
// _evaluate_synthesis_quality — success-rate + response-length heuristic
// ---------------------------------------------------------------------------

/// _evaluate_synthesis_quality: min(1, max(0, success_rate*0.7 +
/// min(1, len/1000)*0.3)).
pub fn evaluate_synthesis_quality(success_count: u64, total_results: u64, response_len: u64) -> f64 {
    if total_results == 0 {
        return 0.0;
    }
    let success_rate = (success_count as f64) / (total_results as f64);
    let comprehensive = ((response_len as f64) / 1000.0).min(1.0);
    let quality = success_rate * 0.7 + comprehensive * 0.3;
    quality.min(1.0).max(0.0)
}

// ---------------------------------------------------------------------------
// _update_trm_metrics — speedup + EMA updates (deterministic)
// ---------------------------------------------------------------------------

/// _update_trm_metrics given total elapsed time: speedup =
/// total_time*partitions*0.7 / total_time (== 0.7*partitions, or 1.0 when
/// total_time is 0); avg_speedup and error_reduction are 90/10 EMAs.
pub fn update_trm_metrics(
    m: &mut TrmMetrics,
    total_time: f64,
    partition_count: u64,
    success_count: u64,
) {
    let speedup = if total_time > 0.0 {
        let estimated = total_time * (partition_count as f64) * 0.7;
        if estimated > 0.0 {
            estimated / total_time
        } else {
            1.0
        }
    } else {
        1.0
    };
    m.total_trm_tasks += 1;
    if partition_count > 1 {
        m.parallel_executions += 1;
    }
    m.avg_speedup = m.avg_speedup * 0.9 + speedup * 0.1;
    if partition_count > 0 {
        let success_rate = (success_count as f64) / (partition_count as f64);
        let error_reduction = 1.0 - (1.0 - success_rate); // == success_rate
        m.error_reduction = m.error_reduction * 0.9 + error_reduction * 0.1;
    }
}

/// set_recursion_depth: clamp depth to 1..=5.
pub fn set_recursion_depth(depth: u64) -> u64 {
    depth.max(1).min(5)
}

// ---------------------------------------------------------------------------
// Factories — create_trm_enhanced_agent / enhance_existing_agent
// ---------------------------------------------------------------------------

/// Static agent-id counter mirroring BaseAgent.agent_id allocation.
static AGENT_ID_COUNTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Create a TRM-enhanced agent: mirrors TRMEnhancedAgent(persona, skills)
/// plus the constructor defaults.
pub fn create_trm_enhanced_agent(persona: u64, skills: &[u64]) -> AgentIdentity {
    let id = AGENT_ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    AgentIdentity {
        persona,
        skills: skills.to_vec(),
    }
}

/// Enhance an existing agent: the enhanced agent keeps the persona and a
/// copy of the skill list; richer object state (memory, subagents, logs) is
/// // MANUAL in no_std and summarized by the shared counter below.
pub fn enhance_existing_agent(base_persona: u64, base_skills: &[u64]) -> AgentIdentity {
    let id = AGENT_ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    AgentIdentity {
        persona: base_persona,
        skills: base_skills.to_vec(),
    }
}

/// TRMEnhancedAgent defaults used by __init__.
pub fn enhanced_agent_defaults() -> TrmAgentConfig {
    TrmAgentConfig {
        agent_id: 0,
        parallel_enabled: true,
        max_recursion_depth: 3,
        confidence_threshold: 0.75,
    }
}

/// Deterministic emulation of one `spawn_sub_reasoning` batch over a set of
/// text/symbolic partitions at depth 0. Returns (success_count, collapsed).
/// Text partitions never collapse at depth 0 (conf 0.7 < 0.75 threshold);
/// symbolic partitions carry conf 0.7 as well, so a pure batch is
/// inconclusive — mirroring python for these deterministic inputs.
pub fn emulated_batch_outcome(partitions: &[String]) -> (u64, bool) {
    let mut success = 0u64;
    let mut max_conf = 0.0f64;
    for part in partitions {
        if looks_like_symbolic(part) {
            let _ = context_to_tokens(part); // symbolic route (trm reason)
            max_conf = max_conf.max(0.7);
            success += 1;
        } else {
            // text route produces an interpretation at confidence 0.7
            max_conf = max_conf.max(0.7);
            success += 1;
        }
    }
    (success, max_conf >= 0.75)
}

// ---------------------------------------------------------------------------
// // MANUAL — async orchestration, LLM synthesis and psutil probes
// ---------------------------------------------------------------------------

/// process_with_trm / _execute_parallel_reasoning / _process_partition /
/// _synthesize_results require an async executor and LLM. // MANUAL -> false.
pub fn process_with_trm_manual() -> bool {
    false
}

/// _should_use_parallel's psutil.cpu_percent / virtual_memory probes. // MANUAL.
pub fn system_resources_constrained_manual() -> bool {
    false
}

/// _llm_generate / _execute_action_tags / _fallback_aggregation. // MANUAL -> "".
pub fn llm_synthesize_manual() -> String {
    String::new()
}

// ── ported from the v20 base tree (G189) ─────────────────────────────────────────────────────────
// 2 column-0 item(s) only: nested fn definitions and `self` receivers are excluded, and the
// statics/consts each function names travel with it (grafting bare functions was the first failure).

fn contains_ci(hay: &str, needle: &str) -> bool {
    let (h, n) = (hay.as_bytes(), needle.as_bytes());
    if n.len() > h.len() {
        return false;
    }
    h.windows(n.len())
        .any(|w| w.iter().zip(n.iter()).all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase()))
}

pub fn process_symbolic_partitions(text: &[u8], max_tokens: usize) -> (Vec<i64>, bool, f64) {
    if text.is_empty() {
        return (Vec::new(), false, 0.0);
    }
    let symbolic = {
        let n = text.iter().filter(|c| {
            c.is_ascii_digit() || c.is_ascii_whitespace() || **c == b',' || **c == b'.'
                || **c == b';' || **c == b':' || **c == b'|'
        }).count();
        (n as f64 / text.len() as f64) > 0.6
    };

    let mut tokens: Vec<i64> = Vec::new();
    let mut idx = 0usize;
    while idx < text.len() && tokens.len() < max_tokens {
        while idx < text.len() && text[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= text.len() {
            break;
        }
        let start = idx;
        while idx < text.len() && !text[idx].is_ascii_whitespace() {
            idx += 1;
        }
        let part = &text[start..idx];

        let mut p_idx = 0usize;
        let mut sign = 1i64;
        if part[0] == b'-' {
            sign = -1;
            p_idx = 1;
        } else if part[0] == b'+' {
            p_idx = 1;
        }

        let mut is_num = p_idx < part.len();
        let mut num_val: i64 = 0;
        if is_num {
            for &c in &part[p_idx..] {
                if c.is_ascii_digit() {
                    num_val = num_val.saturating_mul(10).saturating_add((c - b'0') as i64);
                } else {
                    is_num = false;
                    break;
                }
            }
        }

        if is_num {
            tokens.push(num_val * sign);
        } else {
            let mut h: u32 = 0;
            for &c in part {
                h = h.wrapping_mul(31).wrapping_add(c as u32);
            }
            tokens.push((h % 10) as i64);
        }
    }

    let resp_len = tokens.len() * 4;
    let comp = (resp_len as f64 / 1000.0).min(1.0);
    let quality = 0.7 * 1.0 + 0.3 * comp;
    (tokens, symbolic, quality)
}
