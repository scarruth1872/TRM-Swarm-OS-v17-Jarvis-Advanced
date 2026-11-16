//! TRM Swarm OS - trm_integration (no_std port).
//!
//! The python module is the TRM-enhanced agent -- parallel reasoning, synthesis, metrics. What is
//! portable is its RULE SET, ported here from `swarm_v2/core/trm_integration.py`, plus the token hash
//! it delegates to, ported from `native_voltage_strips/strip_trm_token_hash.rs`.
//!
//! Host-side and not ported: psutil resource sampling, the synthesis model calls, the agent base
//! class. Resource figures reach `should_use_parallel` as arguments instead.

extern crate alloc;
use alloc::vec::Vec;

/// `_analyze_task_complexity`'s high-complexity indicators.
pub const HIGH_COMPLEXITY_KEYWORDS: [&str; 10] = [
    "analyze", "compare", "evaluate", "explain", "interpret", "solve", "calculate", "reason",
    "deduce", "infer",
];
/// `_analyze_task_complexity`'s symbolic indicators.
pub const SYMBOLIC_KEYWORDS: [&str; 9] = [
    "pattern", "sequence", "tokens", "symbol", "logic", "puzzle", "riddle", "algorithm", "formula",
];

/// Case-insensitive substring over ASCII bytes.
fn contains_ci(hay: &str, needle: &str) -> bool {
    let (h, n) = (hay.as_bytes(), needle.as_bytes());
    if n.len() > h.len() {
        return false;
    }
    h.windows(n.len())
        .any(|w| w.iter().zip(n.iter()).all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase()))
}

/// `_analyze_task_complexity`: symbolic wins outright, then high-complexity past 200 characters,
/// then anything past 500, else low. The keywords are matched against the lowercased task while the
/// length thresholds use the original, as in python.
pub fn analyze_task_complexity(task: &str) -> &'static str {
    let symbolic = SYMBOLIC_KEYWORDS.iter().any(|kw| contains_ci(task, kw));
    if symbolic {
        return "symbolic_high";
    }
    let high = HIGH_COMPLEXITY_KEYWORDS.iter().any(|kw| contains_ci(task, kw));
    if high && task.len() > 200 {
        return "text_high";
    }
    if task.len() > 500 {
        return "text_medium";
    }
    "text_low"
}

/// `_should_use_parallel`, with the resource figures supplied by the caller (the python samples them
/// with psutil). The python's gates are `cpu > 80 or memory > 85` and then, by complexity,
/// `len > 100` for the high classes and `len > 300` for medium.
pub fn should_use_parallel(parallel_enabled: bool, cpu_percent: f64, mem_percent: f64,
                           complexity: &str, task_len: usize) -> bool {
    if !parallel_enabled {
        return false;
    }
    if cpu_percent > 80.0 || mem_percent > 85.0 {
        return false;
    }
    if complexity == "symbolic_high" || complexity == "text_high" {
        return task_len > 100;
    }
    if complexity == "text_medium" {
        return task_len > 300;
    }
    false
}

/// `_looks_like_symbolic`'s rule (identical in the python fallback and in `strip_trm_token_hash`):
/// the share of digits, ASCII whitespace and `,.;:|` must exceed 0.6. Empty text is not symbolic.
pub fn looks_like_symbolic(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let b = text.as_bytes();
    let symbolic = b.iter().filter(|c| {
        c.is_ascii_digit() || c.is_ascii_whitespace() || **c == b',' || **c == b'.' || **c == b';'
            || **c == b':' || **c == b'|'
    }).count();
    (symbolic as f64 / b.len() as f64) > 0.6
}

/// `_evaluate_synthesis_quality`: `clamp(success_rate * 0.7 + min(1, len/1000) * 0.3)`, and 0.0 for
/// an empty result set.
pub fn evaluate_synthesis_quality(success_count: usize, total: usize, response_len: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let success_rate = success_count as f64 / total as f64;
    let comprehensive = (response_len as f64 / 1000.0).min(1.0);
    let quality = success_rate * 0.7 + comprehensive * 0.3;
    quality.min(1.0).max(0.0)
}

// ── the token hash (native_voltage_strips/strip_trm_token_hash.rs) ────────────────────────────────
//
// `strip_process_symbolic_partitions` does two things: the symbolic test above, and a tokenizer.
// Each whitespace-separated token is either an INTEGER -- optional `+`/`-` then digits only, parsed
// with saturating arithmetic -- or, if it is not one, the value `hash % 10` where
// `hash = h*31 + byte` accumulated in wrapping u32.
//
// One difference from the python worth naming: the strip's quality is `0.7 * 1.0 + 0.3 * comp` --
// the success factor is hardcoded to 1.0 there, while `_evaluate_synthesis_quality` uses the actual
// success rate. Both are reproduced as they are; the strip's constant is not silently merged into
// the python's rule.

/// The strip's tokenizer: (token values, is_symbolic, quality). Truncates at `max_tokens`, and for
/// empty input returns (empty, false, 0.0) exactly as the strip does.
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

//  _update_trm_metrics() 
// The speedup factor's `total_time` cancels: `estimated/total = (total*partitions*0.7)/total =
// partitions*0.7` whenever `total_time > 0` (the only edge where it doesn't is total_time == 0,
// which python guards with speedup = 1.0). The caller owns the wall clock, exactly as in
// `cleanup_stale_nodes` — this port assumes the normal `total_time > 0` path. `error_reduction`
// is `1 - (1 - success_rate)` = success_rate, and is NOT updated when partition_count == 0.
pub fn update_trm_metrics(
    prev_avg_speedup: f64,
    prev_error_reduction: f64,
    partition_count: usize,
    success_count: usize,
) -> (f64, f64) {
    let speedup = if partition_count > 0 {
        partition_count as f64 * 0.7
    } else {
        1.0
    };
    let avg_speedup = prev_avg_speedup * 0.9 + speedup * 0.1;
    let error_reduction = if partition_count > 0 {
        let success_rate = success_count as f64 / partition_count as f64;
        prev_error_reduction * 0.9 + success_rate * 0.1
    } else {
        prev_error_reduction
    };
    (avg_speedup, error_reduction)
}
