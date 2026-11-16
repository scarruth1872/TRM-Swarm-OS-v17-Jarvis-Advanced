//! TRM Swarm OS — Task Arbiter (no_std port).
//! 1:1 core semantics from swarm_v2/core/task_arbiter.py — Catalyst.
//! Maintenance task scheduling: due-check with dynamic priority.

extern crate alloc;
use alloc::string::String;
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

    /// Dynamic priority: base + urgency from overdue ratio.
    pub fn get_dynamic_priority(&self, name: &str) -> u64 {
        for t in &self.tasks {
            if t.name == name {
                let overdue = self.tick.saturating_sub(t.last_run);
                let urgency = if t.interval_ticks == 0 {
                    0
                } else {
                    (overdue / t.interval_ticks).min(3)
                };
                return (t.priority + urgency).min(MAX_DYNAMIC_PRIORITY);
            }
        }
        0
    }

    /// Stats: (tasks, tick).
    pub fn get_stats(&self) -> (u64, u64) {
        (self.tasks.len() as u64, self.tick)
    }
}

/// Singleton handle.
pub fn get_task_arbiter() -> u64 {
    0x5441_534B_4152_4249
}

// ── ported from the v20 base tree (G189) ─────────────────────────────────────────────────────────
// 18 column-0 item(s) only: nested fn definitions and `self` receivers are excluded, and the
// statics/consts each function names travel with it (grafting bare functions was the first failure).
// imports: use alloc::vec;

use alloc::vec;

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

pub fn decentralized_consensus(scores_desc: &[i64]) -> bool {
    !(scores_desc.len() > 1 && scores_desc[0] - scores_desc[1] < 2)
}

pub fn decentralized_vote_score(role: &str, task_description: &str, history_len: usize) -> i64 {
    let role_in_task = contains_ci(task_description, role);
    (if role_in_task { 10 } else { 0 }) + core::cmp::min(history_len, 10) as i64 + 5
}

pub fn dynamic_priority(base: u64, window: &str, elapsed_sec: u64, interval_sec: u64) -> i64 {
    let mut adj = window_adjustment(window);
    if interval_sec > 0 {
        let overdue = ((elapsed_sec / interval_sec) as i64) * 5;
        adj += overdue.min(20);
    }
    core::cmp::max(0, base as i64 + adj)
}

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

pub fn task_priority_lt(a: i64, b: i64) -> bool {
    a < b
}

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

pub fn window_adjustment(window: &str) -> i64 {
    match window {
        "LOW_USAGE" => 50,
        "NORMAL_USAGE" => 10,
        "HIGH_USAGE" => -20,
        "CRITICAL_USAGE" => -50,
        _ => 0,
    }
}

pub struct DecentralizedOutcome {
    pub method: &'static str,
    pub assigned_to: Option<String>,
    pub reason: &'static str,
    pub votes: Vec<(String, i64)>,
}

pub struct DecentralizedCandidate {
    pub agent_id: String,
    pub role: String,
    pub is_busy: bool,
    pub task_history_len: usize,
}
