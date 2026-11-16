//! TRM Swarm OS — Heartbeat Scheduler (no_std port).
//! Poller and scheduler for proactive agent actions and autonomous pulse dispatching.
//! 1:1 deterministic core from swarm_v2/core/heartbeat_scheduler.py:
//!   - Periodic pulse execution
//!   - Task queue tracking and proactive goals dispatch
//!   - Task status transitions (Pending -> Active -> Completed)

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 1000;
pub const MAX_HEARTBEAT_TASKS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Active,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct HeartbeatTask {
    pub id: u64,
    pub description: String,
    pub status: TaskStatus,
    pub assigned_agent: &'static str,
    pub created_at_tick: u64,
    pub completed_at_tick: u64,
}

pub struct HeartbeatScheduler {
    pub interval_ms: u64,
    pub is_running: bool,
    pub tick_counter: u64,
    pub tasks: Vec<HeartbeatTask>,
    pub next_task_id: u64,
}

impl HeartbeatScheduler {
    pub fn new(interval_ms: u64) -> Self {
        HeartbeatScheduler {
            interval_ms,
            is_running: false,
            tick_counter: 0,
            tasks: Vec::new(),
            next_task_id: 1,
        }
    }

    pub fn start(&mut self) {
        self.is_running = true;
    }

    pub fn stop(&mut self) {
        self.is_running = false;
    }

    pub fn add_task(&mut self, description: &str, assigned_agent: &'static str) -> u64 {
        if self.tasks.len() >= MAX_HEARTBEAT_TASKS {
            self.tasks.retain(|t| t.status != TaskStatus::Completed);
        }

        let id = self.next_task_id;
        self.next_task_id += 1;

        self.tasks.push(HeartbeatTask {
            id,
            description: description.to_string(),
            status: TaskStatus::Pending,
            assigned_agent,
            created_at_tick: self.tick_counter,
            completed_at_tick: 0,
        });

        id
    }

    pub fn pulse(&mut self) -> Option<u64> {
        self.tick_counter += 1;

        // Find next pending task and activate it
        for task in self.tasks.iter_mut() {
            if task.status == TaskStatus::Pending {
                task.status = TaskStatus::Active;
                return Some(task.id);
            }
        }
        None
    }

    pub fn complete_task(&mut self, id: u64) -> bool {
        for task in self.tasks.iter_mut() {
            if task.id == id {
                task.status = TaskStatus::Completed;
                task.completed_at_tick = self.tick_counter;
                return true;
            }
        }
        false
    }
}

static SINGLETON_HEARTBEAT_SCHEDULER: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub fn get_heartbeat_scheduler() -> u64 {
    let v = SINGLETON_HEARTBEAT_SCHEDULER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_HEARTBEAT_SCHEDULER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
