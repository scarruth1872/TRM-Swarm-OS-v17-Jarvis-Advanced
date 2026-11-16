//! TRM Swarm OS — Task Arbiter (no_std port).
//! 1:1 core semantics from swarm_v2/core/task_arbiter.py — Catalyst.
//! Maintenance task scheduling: due-check with dynamic priority.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// Arbiter constants.
pub const DEFAULT_INTERVAL_TICKS: u64 = 100;
pub const MAX_DYNAMIC_PRIORITY: u64 = 10;

/// A scheduled maintenance task.
#[derive(Debug, Clone)]
pub struct MaintTask {
    pub name: String,
    pub interval_ticks: u64,
    pub last_run: u64,
    pub priority: u64,
}

/// Task arbiter.
#[derive(Debug, Clone)]
pub struct TaskArbiter {
    pub tasks: Vec<MaintTask>,
    pub tick: u64,
}

impl TaskArbiter {
    pub fn new() -> Self {
        TaskArbiter {
            tasks: Vec::new(),
            tick: 0,
        }
    }

    /// Register a maintenance task.
    pub fn register_maintenance_task(&mut self, name: &str, interval_ticks: u64, priority: u64) {
        self.tasks.push(MaintTask {
            name: name.into(),
            interval_ticks: if interval_ticks == 0 { DEFAULT_INTERVAL_TICKS } else { interval_ticks },
            last_run: 0,
            priority: priority.min(MAX_DYNAMIC_PRIORITY),
        });
    }

    /// Advance the clock; returns names of tasks now due.
    pub fn tick_and_collect_due(&mut self) -> Vec<String> {
        self.tick += 1;
        let mut due = Vec::new();
        for t in self.tasks.iter_mut() {
            if self.tick.saturating_sub(t.last_run) >= t.interval_ticks {
                due.push(t.name.clone());
                t.last_run = self.tick;
            }
        }
        due
    }

    /// Is a task due right now (without consuming)?
    pub fn is_due(&self, name: &str) -> bool {
        self.tasks
            .iter()
            .any(|t| t.name == name && self.tick.saturating_sub(t.last_run) >= t.interval_ticks)
    }

    /// Stats: (tasks, tick).
    pub fn get_stats(&self) -> (u64, u64) {
        (self.tasks.len() as u64, self.tick)
    }
}

// ── The oracle's model (swarm_v2/core/task_arbiter.py + strip_task_arbiter.rs) ────────────────────
//
// The module's `get_dynamic_priority` was `(priority + min(overdue/interval, 3)).min(10)`: a clamp at
// ten on a value the oracle lets reach 150. The oracle's `DynamicPriorityArbiter` is
//     dynamic = max(0, base + PRIORITY_ADJUSTMENTS[window] + min(20, int(elapsed/interval) * 5))
// with a base-priority TABLE by maintenance class and a usage window from the average CPU percent.
// The three pieces below are that model, plus the allocation scorer the module never had.

/// Base priority by maintenance class (oracle `_get_base_priority`). Default 10.
pub fn base_priority(class: &str) -> u64 {
    match class {
        "HEALTH_CHECK" => 80,
        "MEMORY_PRUNING" => 60,
        "GARBAGE_COLLECTION" => 50,
        "METRICS_COLLECTION" => 40,
        "CACHE_CLEANUP" => 30,
        _ => 10,
    }
}

/// Usage window from an average CPU percent: < 20 LOW, < 60 NORMAL, < 80 HIGH, else CRITICAL.
pub fn usage_window(avg_cpu: f64) -> &'static str {
    if avg_cpu < 20.0 {
        "LOW_USAGE"
    } else if avg_cpu < 60.0 {
        "NORMAL_USAGE"
    } else if avg_cpu < 80.0 {
        "HIGH_USAGE"
    } else {
        "CRITICAL_USAGE"
    }
}

/// `PRIORITY_ADJUSTMENTS`: LOW +50, NORMAL +10, HIGH -20, CRITICAL -50 (0 for anything else).
pub fn window_adjustment(window: &str) -> i64 {
    match window {
        "LOW_USAGE" => 50,
        "NORMAL_USAGE" => 10,
        "HIGH_USAGE" => -20,
        "CRITICAL_USAGE" => -50,
        _ => 0,
    }
}

/// The oracle's dynamic priority. `elapsed`/`interval` are in seconds as on the python side; the
/// overdue boost is `min(20, int(elapsed / interval) * 5)` and the result floors at 0 -- it is NOT
/// clamped at `MAX_DYNAMIC_PRIORITY`, which was this module's invention.
pub fn dynamic_priority(base: u64, window: &str, elapsed_sec: u64, interval_sec: u64) -> i64 {
    let mut adj = window_adjustment(window);
    if interval_sec > 0 {
        let overdue = ((elapsed_sec / interval_sec) as i64) * 5;
        adj += overdue.min(20);
    }
    core::cmp::max(0, base as i64 + adj)
}

/// `should_run_maintenance`: disabled never runs; CRITICAL defers; only essential classes run in a
/// HIGH window; otherwise the task's due-ness decides.
pub fn should_run(enabled: bool, window: &str, is_due: bool, class: &str) -> bool {
    if !enabled {
        return false;
    }
    if window == "CRITICAL_USAGE" {
        return false;
    }
    if !is_due {
        return false;
    }
    if window == "HIGH_USAGE" {
        return matches!(class, "HEALTH_CHECK" | "MEMORY_PRUNING" | "GARBAGE_COLLECTION");
    }
    true
}

/// The oracle's multi-agent allocation scorer: for each task, the agent maximising
///     (sum_k caps[a][k] * reqs[t][k]) * urgency[t] / (1.0 + loads[a])
/// over eight capability dimensions. `best_score` starts at -1.0e9 and the comparison is strict, so a
/// tie keeps the FIRST agent -- both as in `strip_task_arbiter.rs`. f32 throughout, as the strip is.
/// Returns (assignments, priorities), one entry per task.
pub fn score_allocation(loads: &[f32], caps: &[f32], reqs: &[f32], urgencies: &[f32]) -> (alloc::vec::Vec<i32>, alloc::vec::Vec<f32>) {
    let agents = loads.len();
    let tasks = urgencies.len();
    let mut assignments: alloc::vec::Vec<i32> = alloc::vec::Vec::with_capacity(tasks);
    let mut priorities: alloc::vec::Vec<f32> = alloc::vec::Vec::with_capacity(tasks);
    for t in 0..tasks {
        let req_base = t * 8;
        let urg = urgencies[t];
        let mut best_agent: i32 = 0;
        let mut best_score: f32 = -1.0e9;
        for a in 0..agents {
            let cap_base = a * 8;
            if cap_base + 8 > caps.len() || req_base + 8 > reqs.len() {
                continue;
            }
            let mut m: f32 = 0.0;
            for k in 0..8 {
                m += caps[cap_base + k] * reqs[req_base + k];
            }
            let score = (m * urg) / (1.0 + loads[a]);
            if score > best_score {
                best_score = score;
                best_agent = a as i32;
            }
        }
        assignments.push(best_agent);
        priorities.push(best_score);
    }
    (assignments, priorities)
}

/// Singleton handle.
pub fn get_task_arbiter() -> u64 {
    0x5441_534B_4152_4249
}

// ── Missing pure functions ported from swarm_v2/core/task_arbiter.py ──────────────────────────
// Everything below is PURE and portable. Host-side (deliberately NOT ported): psutil
// (cpu_percent/cpu_count/virtual_memory/Process.cpu_affinity), datetime.now() timestamps,
// asyncio (PriorityQueue, create_task, run_maintenance_cycle), os/glob/gc file I/O, logging,
// the Phase-15 federation HTTP delegation, the Phase-14 attempt_sampler, the Phase-16
// mutation_engine / expert_registry, and the get_task_arbiter / route_task_to_arbiter
// singletons (import-singletons are host-side, per the port policy).

/// Case-insensitive ASCII substring test (Python `needle.lower() in haystack.lower()`).
/// no_std has no Unicode case tables, so lowering is ASCII-only — the same MANUAL caveat as
/// ultrawork_loop's `is_python_isdigit`. All differential vectors are ASCII, where the two
/// agree exactly. An empty needle matches (Python `"" in s` is always True).
fn contains_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let hb = haystack.as_bytes();
    let nb = needle.as_bytes();
    if nb.len() > hb.len() {
        return false;
    }
    'outer: for i in 0..=(hb.len() - nb.len()) {
        for j in 0..nb.len() {
            if hb[i + j].to_ascii_lowercase() != nb[j].to_ascii_lowercase() {
                continue 'outer;
            }
        }
        return true;
    }
    false
}

/// `TaskArbiter.classify_task(task, context=None)` — the `context` arg is unused by the
/// oracle, so this is a pure function of the task string. Returns "low" | "medium" | "high"
/// (never "critical": the oracle's ladder has no CRITICAL return). Note the phrase check
/// (`"code"`, `"write file"`, `"create file"`) runs BEFORE the high/low keyword test and
/// uses substring semantics, so `"codebase"` matches `"code"` and `len(task) > 100` counts
/// characters (Python `len`), not bytes.
pub fn classify_task(task: &str) -> &'static str {
    const HIGH_KWS: [&str; 14] = [
        "analyze", "audit", "review", "research", "generate", "implement", "design",
        "architect", "create", "build", "reason", "explain", "compare", "evaluate",
    ];
    const LOW_KWS: [&str; 10] = [
        "list", "show", "get", "read", "check", "status", "ping", "echo", "count", "simple",
    ];

    if contains_ci(task, "write file") || contains_ci(task, "create file") || contains_ci(task, "code") {
        return "high";
    }
    let high = HIGH_KWS.iter().any(|kw| contains_ci(task, kw));
    let low = LOW_KWS.iter().any(|kw| contains_ci(task, kw));
    let len = task.chars().count();
    if high && len > 100 {
        "high"
    } else if low {
        "low"
    } else if len > 200 {
        "medium"
    } else {
        "low"
    }
}

/// `TaskArbiter.get_optimal_compute_target(task, system_load)` decision ladder, taking the
/// task complexity string and the system load. Returns "cpu_cores" | "gpu_model" | "hybrid".
/// The Phase-15 `REMOTE_SWARM` branch is host-side (federation lookup) AND unreachable: it is
/// only entered for complexity not HIGH/CRITICAL/MEDIUM and `system_load > 0.85`, but LOW with
/// `system_load > 0.8` already returned CPU_CORES one guard earlier. Dead code, not ported.
pub fn optimal_compute_target(complexity: &str, system_load: f64) -> &'static str {
    if system_load > 0.8 && complexity == "low" {
        return "cpu_cores";
    }
    if complexity == "high" || complexity == "critical" {
        return "gpu_model";
    }
    if complexity == "medium" {
        if system_load < 0.5 {
            return "hybrid";
        }
        return "cpu_cores";
    }
    "cpu_cores"
}

/// `TaskArbiter._calculate_autonomous_priority(task, agent_id)`. `registered` is whether the
/// agent exists in `agent_workloads` (the oracle returns the base priority untouched when it
/// does not). Role matching is case-sensitive, exactly as the oracle's set membership.
pub fn autonomous_priority(
    registered: bool,
    base: i64,
    role: &str,
    task_type: &str,
    complexity: &str,
) -> i64 {
    if !registered {
        return base;
    }
    let mut p = base;
    if matches!(role, "Architect" | "Reasoning Engine" | "Security Auditor" | "Swarm Manager") {
        p += 10;
    }
    // Single `or` in the oracle: +20 once, even if both halves are true.
    if task_type == "remediation" || complexity == "critical" {
        p += 20;
    }
    p
}

/// `TaskArbiter._setup_cpu_affinity()` for a given `cpu_count`. Returns (os, llm, agent, io)
/// core lists. The `>= 8` and `< 8` branches are exact; `>= 16` builds io as `range(20, n)`.
pub fn setup_cpu_affinity(cpu_count: u32) -> (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>) {
    if cpu_count >= 16 {
        let os: Vec<u32> = (0..4).collect();
        let llm: Vec<u32> = (4..12).collect();
        let agent: Vec<u32> = (12..20).collect();
        let io: Vec<u32> = (20..cpu_count).collect();
        (os, llm, agent, io)
    } else if cpu_count >= 8 {
        (vec![0], vec![1, 2, 3], vec![4, 5, 6], vec![7])
    } else {
        let os = vec![0];
        let shared: Vec<u32> = (1..cpu_count).collect();
        (os, shared.clone(), shared.clone(), shared)
    }
}

/// `TaskArbiter._get_affinity_for_role(role)`, given the llm/agent/io core lists the oracle
/// reads from `self`. High-load roles take `llm[:3]` (else all of agent); I/O roles take
/// `io[:2]` (else agent); anything else takes `agent[:2]` (else llm). Role matching is
/// case-sensitive set membership.
pub fn affinity_for_role(role: &str, llm: &[u32], agent: &[u32], io: &[u32]) -> Vec<u32> {
    let high_load = matches!(role, "Lead Developer" | "Reasoning Engine" | "Researcher");
    let io_role = matches!(role, "Technical Writer" | "QA Engineer" | "ui_ux Designer");
    if high_load {
        if llm.len() >= 3 {
            llm[..3].to_vec()
        } else {
            agent.to_vec()
        }
    } else if io_role {
        if io.len() >= 2 {
            io[..2].to_vec()
        } else {
            agent.to_vec()
        }
    } else if agent.len() >= 2 {
        agent[..2].to_vec()
    } else {
        llm.to_vec()
    }
}

/// Per-candidate vote score of `TaskArbiter.decentralized_assign` (Phase 7):
///     (10 if role.lower() in task_description.lower() else 0) + min(history_len, 10) + 5
/// The "+5 not-busy bonus" is baked in because busy candidates are skipped before scoring.
pub fn decentralized_vote_score(role: &str, task_description: &str, history_len: usize) -> i64 {
    let role_in_task = contains_ci(task_description, role);
    (if role_in_task { 10 } else { 0 }) + core::cmp::min(history_len, 10) as i64 + 5
}

/// Consensus decision over scores already sorted descending (the oracle's
/// `votes[0][0] - votes[1][0] < 2`). True => "decentralized"/"consensus",
/// false => "centralized_fallback"/"no_consensus". A single vote (or none) is consensus.
pub fn decentralized_consensus(scores_desc: &[i64]) -> bool {
    !(scores_desc.len() > 1 && scores_desc[0] - scores_desc[1] < 2)
}

/// A candidate agent for `decentralized_assign`. `agent_id`/`role` are owned because the
/// outcome borrows from them; the oracle's `not wl` (unknown agent) case is host-side and
/// cannot arise here — every candidate is a known agent.
#[derive(Debug, Clone)]
pub struct DecentralizedCandidate {
    pub agent_id: String,
    pub role: String,
    pub is_busy: bool,
    pub task_history_len: usize,
}

/// Outcome of `decentralized_assign`: method ("fallback" | "centralized_fallback" |
/// "decentralized"), the winner (None on fallback), the reason, and the top-5 (agent, score)
/// votes in descending score order (insertion order preserved on ties — Python's stable sort).
#[derive(Debug, Clone)]
pub struct DecentralizedOutcome {
    pub method: &'static str,
    pub assigned_to: Option<String>,
    pub reason: &'static str,
    pub votes: Vec<(String, i64)>,
}

/// `TaskArbiter.decentralized_assign(task_description, candidates)` — the pure voting core.
/// `candidates` is the explicit list (the oracle's `candidates is None` default to all agent
/// keys is host-side). Busy candidates are skipped; the winner is the top score after a
/// stable descending sort; a <2-point lead over second place falls back to centralized.
pub fn decentralized_assign(
    task_description: &str,
    candidates: &[DecentralizedCandidate],
) -> DecentralizedOutcome {
    if candidates.is_empty() {
        return DecentralizedOutcome {
            method: "fallback",
            assigned_to: None,
            reason: "no_candidates",
            votes: Vec::new(),
        };
    }

    // (score, candidate index) for non-busy candidates, in first-seen order.
    let mut scored: Vec<(i64, usize)> = Vec::new();
    for (idx, c) in candidates.iter().enumerate() {
        if c.is_busy {
            continue;
        }
        let score = decentralized_vote_score(&c.role, task_description, c.task_history_len);
        scored.push((score, idx));
    }

    if scored.is_empty() {
        return DecentralizedOutcome {
            method: "fallback",
            assigned_to: None,
            reason: "all_busy",
            votes: Vec::new(),
        };
    }

    // Stable descending sort by score (Python `votes.sort(key=lambda x: -x[0])`).
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    let winner = candidates[scored[0].1].agent_id.clone();
    let scores_desc: Vec<i64> = scored.iter().map(|s| s.0).collect();
    let consensus = decentralized_consensus(&scores_desc);

    let votes: Vec<(String, i64)> = scored
        .iter()
        .take(5)
        .map(|(s, idx)| (candidates[*idx].agent_id.clone(), *s))
        .collect();

    DecentralizedOutcome {
        method: if consensus {
            "decentralized"
        } else {
            "centralized_fallback"
        },
        assigned_to: Some(winner),
        reason: if consensus { "consensus" } else { "no_consensus" },
        votes,
    }
}

/// `Task.__lt__(other)` (the `@total_ordering` dataclass orders by priority only).
pub fn task_priority_lt(a: i64, b: i64) -> bool {
    a < b
}
