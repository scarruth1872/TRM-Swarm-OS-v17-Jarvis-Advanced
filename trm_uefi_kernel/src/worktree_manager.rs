//! TRM Swarm OS - worktree_manager (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/worktree_manager.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  get_worktree_manager() 
static SINGLETON_WORKTREE_MANAGER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_worktree_manager() -> u64 {
    // Singleton handle (lazy-init pattern from worktree_manager.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_WORKTREE_MANAGER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_WORKTREE_MANAGER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
