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
