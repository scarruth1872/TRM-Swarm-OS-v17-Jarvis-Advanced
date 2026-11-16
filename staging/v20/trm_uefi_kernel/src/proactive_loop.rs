//! TRM Swarm OS - proactive_loop (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/proactive_loop.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS
//  (host-side API URLs removed — no_std kernel makes no HTTP calls)
pub const PLAN_PATH: &'static str = "swarm_v2_artifacts/QIAE_INTEGRATION_PLAN_V2.md";

// ── The strip's model (swarm_v2_artifacts/native_voltage_strips/strip_proactive_cron.rs) ──────────
//
// `strip_batch_cron_matches` packs each rule as FIVE fields x THREE u32 = 15 words, the fields being
// (minute, hour, day, month, weekday) in that order, each as (rule_type, param1, param2):
//     type 0 = any (params ignored)
//     type 1 = equals param1
//     type 2 = divisible by param1  (and param1 == 0 never matches -- the strip's own guard)
//     type 3 = within [param1, param2] inclusive
// It returns the number of matching rules and writes a 1/0 mask.
//
// Worth recording: `packed_rules` has no constructor anywhere in the python tree -- the caller passes
// a numpy array that nothing builds -- so this native path is callable but currently UNCALLED. The
// matcher below is the strip's semantics, ported so the firmware can evaluate the rule form
// regardless of whether the host wires its caller.

/// Field indices, matching the strip's `vals = [minute, hour, day, month, weekday]`.
pub const CRON_FIELDS: usize = 5;
/// Words per rule: five fields of (type, param1, param2).
pub const CRON_WORDS_PER_RULE: usize = 15;

/// Does one packed rule match the given field values? `rule` is 15 words.
pub fn cron_rule_matches(rule: &[u32], vals: &[u32; 5]) -> bool {
    if rule.len() < CRON_WORDS_PER_RULE {
        return false;
    }
    for f in 0..CRON_FIELDS {
        let b = f * 3;
        let rule_type = rule[b];
        let param1 = rule[b + 1];
        let param2 = rule[b + 2];
        let v = vals[f];
        match rule_type {
            1 => {
                if v != param1 {
                    return false;
                }
            }
            2 => {
                if param1 == 0 || (v % param1) != 0 {
                    return false;
                }
            }
            3 => {
                if v < param1 || v > param2 {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

/// The batch entry point: (match_count, mask), mirroring `strip_batch_cron_matches`.
pub fn cron_matches_batch(rules: &[u32], num_rules: usize, minute: u32, hour: u32, day: u32,
                          month: u32, weekday: u32) -> (usize, Vec<u8>) {
    let vals = [minute, hour, day, month, weekday];
    let mut mask: Vec<u8> = Vec::with_capacity(num_rules);
    let mut count = 0usize;
    for r in 0..num_rules {
        let base = r * CRON_WORDS_PER_RULE;
        if base + CRON_WORDS_PER_RULE > rules.len() {
            mask.push(0);
            continue;
        }
        if cron_rule_matches(&rules[base..base + CRON_WORDS_PER_RULE], &vals) {
            mask.push(1);
            count += 1;
        } else {
            mask.push(0);
        }
    }
    (count, mask)
}

/// The pure-Python `_cron_matches(cron_expr, dt)` string matcher (proactive_loop.py:64-116).
/// Parses a 5-field expression (`minute hour day month weekday`) with `*` (any), `*/N`
/// (divisible by N), or an exact integer. `*/0` and any unparseable field never match — the
/// oracle's `val % 0` raises and an `int()` failure returns False. The weekday value is the
/// caller's (Python `datetime.weekday()`, 0=Monday) — the firmware compares it verbatim.
pub fn cron_matches_expr(
    expr: &str,
    minute: u32,
    hour: u32,
    day: u32,
    month: u32,
    weekday: u32,
) -> bool {
    let vals = [minute as i64, hour as i64, day as i64, month as i64, weekday as i64];
    let parts: Vec<&str> = expr.trim().split_whitespace().collect();
    if parts.len() != 5 {
        return false;
    }
    for (i, part) in parts.iter().enumerate() {
        let v = vals[i];
        if *part == "*" {
            continue;
        }
        if part.starts_with("*/") {
            let step: i64 = match part[2..].parse() {
                Ok(n) => n,
                Err(_) => return false,
            };
            if step == 0 || v % step != 0 {
                return false;
            }
        } else {
            let exact: i64 = match part.parse() {
                Ok(n) => n,
                Err(_) => return false,
            };
            if exact != v {
                return false;
            }
        }
    }
    true
}

//  ScheduledTask 
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub name: String,
    pub cron_expr: String,
    pub handler: u64,
    pub action_description: String,
    pub enabled: bool,
    pub last_run: u64,
    pub run_count: i64,
}

impl ScheduledTask {
    pub fn new() -> Self {
        ScheduledTask {
            name: String::new(),
            cron_expr: String::new(),
            handler: 0,
            action_description: String::new(),
            enabled: false,
            last_run: 0,
            run_count: 0,
        }
    }
}

//  get_proactive_loop(manus_engine) 
static SINGLETON_PROACTIVE_LOOP: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_proactive_loop() -> u64 {
    // Singleton handle (lazy-init pattern from proactive_loop.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_PROACTIVE_LOOP.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_PROACTIVE_LOOP.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
