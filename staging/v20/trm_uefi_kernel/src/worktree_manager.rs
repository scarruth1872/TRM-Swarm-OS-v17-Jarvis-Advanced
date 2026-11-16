//! TRM Swarm OS - worktree_manager (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/worktree_manager.py & strip_worktree_manager.dll interfaces.
//! Manages isolated Git worktrees and task branches for agent execution workspaces (Isolation Level 0),
//! lock state tracking, merge cleanup, and atomic worktree leasing.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- CONSTANTS ---
pub const DEFAULT_WORKTREES_ROOT: &'static str = "swarm_v2_worktrees";
pub const MAX_ISOLATED_WORKTREES: usize = 64;

// --- WORKTREE STATUS ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorktreeStatus {
    Active,
    Locked,
    Merged,
    Pruned,
}

impl WorktreeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorktreeStatus::Active => "active",
            WorktreeStatus::Locked => "locked",
            WorktreeStatus::Merged => "merged",
            WorktreeStatus::Pruned => "pruned",
        }
    }
}

// --- WORKTREE DESCRIPTOR ---
#[derive(Debug, Clone)]
pub struct WorktreeDescriptor {
    pub task_id: String,
    pub branch_name: String,
    pub relative_path: String,
    pub status: WorktreeStatus,
    pub created_at_epoch_s: u64,
    pub locked: bool,
    pub branch_hash: u64,
}

impl WorktreeDescriptor {
    pub fn compute_branch_hash(branch: &str, task_id: &str) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in branch.as_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        for &b in task_id.as_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
}

// --- WORKTREE MANAGER ---
pub struct WorktreeManager {
    base_project_path: String,
    worktrees_root: String,
    worktrees: Vec<WorktreeDescriptor>,
}

impl WorktreeManager {
    pub fn new(base_path: &str, worktrees_root: Option<&str>) -> Self {
        let root = worktrees_root.unwrap_or(DEFAULT_WORKTREES_ROOT);
        Self {
            base_project_path: base_path.to_string(),
            worktrees_root: root.to_string(),
            worktrees: Vec::new(),
        }
    }

    pub fn create_worktree(
        &mut self,
        task_id: &str,
        base_branch: &str,
        epoch_s: u64,
    ) -> Option<String> {
        let rel_path = format!("{}/task_{}", self.worktrees_root, task_id);
        let branch = format!("task/{}", task_id);

        if let Some(existing) = self.worktrees.iter().find(|w| w.task_id == task_id) {
            return Some(existing.relative_path.clone());
        }

        if self.worktrees.len() >= MAX_ISOLATED_WORKTREES {
            // Prune any merged or pruned worktrees before refusing
            self.prune_stale();
            if self.worktrees.len() >= MAX_ISOLATED_WORKTREES {
                return None;
            }
        }

        let hash = WorktreeDescriptor::compute_branch_hash(&branch, task_id);
        let desc = WorktreeDescriptor {
            task_id: task_id.to_string(),
            branch_name: branch,
            relative_path: rel_path.clone(),
            status: WorktreeStatus::Active,
            created_at_epoch_s: epoch_s,
            locked: false,
            branch_hash: hash,
        };

        self.worktrees.push(desc);
        Some(rel_path)
    }

    pub fn lock_worktree(&mut self, task_id: &str) -> bool {
        if let Some(w) = self.worktrees.iter_mut().find(|w| w.task_id == task_id) {
            w.locked = true;
            w.status = WorktreeStatus::Locked;
            true
        } else {
            false
        }
    }

    pub fn unlock_worktree(&mut self, task_id: &str) -> bool {
        if let Some(w) = self.worktrees.iter_mut().find(|w| w.task_id == task_id) {
            w.locked = false;
            w.status = WorktreeStatus::Active;
            true
        } else {
            false
        }
    }

    pub fn mark_merged(&mut self, task_id: &str) -> bool {
        if let Some(w) = self.worktrees.iter_mut().find(|w| w.task_id == task_id) {
            w.status = WorktreeStatus::Merged;
            true
        } else {
            false
        }
    }

    pub fn remove_worktree(&mut self, task_id: &str) -> bool {
        if let Some(pos) = self.worktrees.iter().position(|w| w.task_id == task_id) {
            if !self.worktrees[pos].locked {
                self.worktrees.remove(pos);
                return true;
            }
        }
        false
    }

    pub fn prune_stale(&mut self) -> usize {
        let initial_len = self.worktrees.len();
        self.worktrees.retain(|w| w.status != WorktreeStatus::Merged && w.status != WorktreeStatus::Pruned && !w.locked);
        initial_len - self.worktrees.len()
    }

    pub fn find_by_task(&self, task_id: &str) -> Option<&WorktreeDescriptor> {
        self.worktrees.iter().find(|w| w.task_id == task_id)
    }

    pub fn count_active(&self) -> usize {
        self.worktrees.iter().filter(|w| w.status == WorktreeStatus::Active || w.status == WorktreeStatus::Locked).count()
    }

    pub fn total_worktrees(&self) -> usize {
        self.worktrees.len()
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_WORKTREE_MANAGER: AtomicU64 = AtomicU64::new(0);

pub fn get_worktree_manager() -> u64 {
    let v = SINGLETON_WORKTREE_MANAGER.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_WORKTREE_MANAGER.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

