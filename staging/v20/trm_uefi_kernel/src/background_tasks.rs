//! TRM Swarm OS — background_tasks (bare-metal no_std + alloc).
//! Phase 14 auto-injected native batch artifact evaluator and background task scheduler.
//! High-throughput priority ordering, plan classification, and security skip filters.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;

static TASK_INVOCATIONS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct ArtifactEvaluation {
    pub priority: i32,    // 0: high ("scribe" / "antigravity"), 1: normal
    pub is_plan: i32,     // 1: plan/blueprint/spec, 0: not a plan
    pub skip_scan: i32,   // 1: static content (.md, .txt, .json, .log, etc.), 0: must scan
    pub subdir: i32,      // 0: docs, 1: config, 2: logs, 3: code
}

fn contains_ci(hay: &[u8], needle: &[u8]) -> bool {
    if needle.len() > hay.len() { return false; }
    for w in hay.windows(needle.len()) {
        if w.iter().zip(needle.iter()).all(|(&a, &b)| a.to_ascii_lowercase() == b.to_ascii_lowercase()) {
            return true;
        }
    }
    false
}

fn starts_with_ci(hay: &[u8], needle: &[u8]) -> bool {
    if needle.len() > hay.len() { return false; }
    hay[..needle.len()].iter().zip(needle.iter()).all(|(&a, &b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
}

fn ends_with_ci(hay: &[u8], needle: &[u8]) -> bool {
    if needle.len() > hay.len() { return false; }
    hay[(hay.len() - needle.len())..].iter().zip(needle.iter()).all(|(&a, &b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
}

/// Native batch evaluation of artifact priorities, plan tags, skip scan flags and subdirs.
pub fn evaluate_artifact(s: &[u8]) -> ArtifactEvaluation {
    // 1. Priority: "scribe" or "antigravity"
    let priority = if contains_ci(s, b"scribe") || contains_ci(s, b"antigravity") {
        0
    } else {
        1
    };

    // 2. is_plan
    let mut has_plan_kw = false;
    for kw in &[b"plan" as &[u8], b"blueprint", b"spec"] {
        if starts_with_ci(s, kw) {
            has_plan_kw = true;
            break;
        }
        for prefix in &[b'_', b'-', b'.'] {
            let mut target = Vec::with_capacity(1 + kw.len());
            target.push(*prefix);
            target.extend_from_slice(kw);
            if contains_ci(s, &target) {
                has_plan_kw = true;
                break;
            }
        }
        if has_plan_kw { break; }
    }

    let is_valid_plan = has_plan_kw
        && !contains_ci(s, b"artifact")
        && !starts_with_ci(s, b"research")
        && !contains_ci(s, b"planetary")
        && !s.starts_with(b"PROPOSAL_");

    let is_plan = if is_valid_plan { 1 } else { 0 };

    // 3. Skip scan exts: .md, .txt, .json, .log, .yaml, .yml, .toml, .pt, .bin
    let mut can_skip = false;
    for ext in &[b".md" as &[u8], b".txt", b".json", b".log", b".yaml", b".yml", b".toml", b".pt", b".bin"] {
        if ends_with_ci(s, ext) {
            can_skip = true;
            break;
        }
    }
    let skip_scan = if can_skip { 1 } else { 0 };

    // 4. Subdir mapping: 0:docs, 1:config, 2:logs, 3:code
    let subdir = if ends_with_ci(s, b".md") || ends_with_ci(s, b".txt") {
        0
    } else if ends_with_ci(s, b".json") || ends_with_ci(s, b".yaml") || ends_with_ci(s, b".yml")
           || ends_with_ci(s, b".toml") || ends_with_ci(s, b".cfg") || ends_with_ci(s, b".ini") || ends_with_ci(s, b".env") {
        1
    } else if ends_with_ci(s, b".log") {
        2
    } else {
        3
    };

    ArtifactEvaluation {
        priority,
        is_plan,
        skip_scan,
        subdir,
    }
}

pub struct BackgroundTaskManager {
    pub active_tasks: u64,
    pub completed_tasks: u64,
    pub high_priority_count: u64,
}

impl BackgroundTaskManager {
    pub fn new() -> Self {
        Self {
            active_tasks: 0,
            completed_tasks: 0,
            high_priority_count: 0,
        }
    }

    pub fn schedule_batch(&mut self, filenames: &[&str]) -> Vec<ArtifactEvaluation> {
        let mut results = Vec::with_capacity(filenames.len());
        for &name in filenames {
            let eval = evaluate_artifact(name.as_bytes());
            if eval.priority == 0 {
                self.high_priority_count += 1;
            }
            self.active_tasks += 1;
            results.push(eval);
        }
        results
    }

    pub fn complete_task(&mut self) {
        if self.active_tasks > 0 {
            self.active_tasks -= 1;
            self.completed_tasks += 1;
        }
    }
}

pub fn get_background_task_status() -> (u64, u64, u64) {
    let c = TASK_INVOCATIONS.fetch_add(1, Ordering::Relaxed);
    (c, 0, MONAD_ANCHOR)
}

pub fn evaluate_artifacts_batch(filenames: &[&str]) -> Vec<ArtifactEvaluation> {
    let mut manager = BackgroundTaskManager::new();
    manager.schedule_batch(filenames)
}

pub fn schedule_task(task_type: u64, payload_hash: u64) -> u64 {
    let _ = task_type;
    MONAD_ANCHOR ^ payload_hash ^ TASK_INVOCATIONS.fetch_add(1, Ordering::Relaxed)
}


