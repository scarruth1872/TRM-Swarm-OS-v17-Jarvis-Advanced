//! TRM Swarm OS - ddr_self_training (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/ddr_self_training.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  get_self_training() 
static SINGLETON_SELF_TRAINING: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_self_training() -> u64 {
    // Singleton handle (lazy-init pattern from ddr_self_training.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_SELF_TRAINING.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SELF_TRAINING.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
