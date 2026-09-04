//! TRM Swarm OS — worktree_manager (no_std port).
//! Deterministic core from swarm_v2/core/worktree_manager.py — Isolation 0:
//! manages isolated git worktrees for agent tasks. Path/branch construction,
//! existence checks, task tracking are ported 1:1; the git subprocess calls
//! (host filesystem) become MANUAL skeletons.

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Default worktrees root name (1:1 python default arg).
pub const WORKTREES_ROOT_DEFAULT: &str = "swarm_v2_worktrees";

/// WorktreeManager with in-memory task registry.
#[derive(Debug, Clone)]
pub struct WorktreeManager {
    pub base_project_path: String,
    pub worktrees_root: String,
    /// task_id -> (created, force_removed); mirrors on-disk existence state.
    tasks: BTreeMap<String, bool>,
}

impl WorktreeManager {
    pub fn new(base_project_path: &str) -> Self {
        let root = format!("{}/{}", base_project_path, WORKTREES_ROOT_DEFAULT);
        WorktreeManager {
            base_project_path: base_project_path.to_string(),
            worktrees_root: root,
            tasks: BTreeMap::new(),
        }
    }

    /// os.path.join(worktrees_root, f"task_{task_id}") — deterministic path.
    pub fn worktree_path(&self, task_id: &str) -> String {
        format!("{}/task_{}", self.worktrees_root, task_id)
    }

    /// Branch name used by the git command: f"task/{task_id}".
    pub fn branch_name(task_id: &str) -> String {
        format!("task/{}", task_id)
    }

    /// create_worktree(): if the path already exists, return it unchanged;
    /// otherwise mark the task as created and return the fresh path.
    pub fn create_worktree(&mut self, task_id: &str, _branch: &str) -> Option<String> {
        let path = self.worktree_path(task_id);
        match self.tasks.get(task_id) {
            Some(_) => Some(path), // already exists -> reuse
            None => {
                // MANUAL: `git worktree add -b task/{id} {path} {branch}` runs
                // on the host. Kernel tracks intent; subprocess errors map to None.
                self.tasks.insert(task_id.to_string(), true);
                Some(path)
            }
        }
    }

    /// remove_worktree(): no-op (returns false) when the task is unknown.
    pub fn remove_worktree(&mut self, task_id: &str, force: bool) -> bool {
        match self.tasks.get(task_id) {
            None => false,
            Some(_) => {
                // MANUAL: `git worktree remove [--force] {path}` on host.
                let _ = force;
                self.tasks.remove(task_id);
                true
            }
        }
    }

    /// list_worktrees(): python shells out to `git worktree list --porcelain`;
    /// kernel-side the equivalent registry is the set of known task ids.
    pub fn list_worktrees(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    pub fn task_count(&self) -> u64 {
        self.tasks.len() as u64
    }
}

/// Singleton handle (lazy-init pattern from get_worktree_manager()).
static SINGLETON_WORKTREE_MANAGER: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
pub fn get_worktree_manager() -> u64 {
    let v = SINGLETON_WORKTREE_MANAGER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_WORKTREE_MANAGER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
