//! TRM Swarm OS - evolutionary_sandbox (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/evolutionary_sandbox.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  get_evolutionary_sandbox() 
static SINGLETON_EVOLUTIONARY_SANDBOX: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_evolutionary_sandbox() -> u64 {
    // Singleton handle (lazy-init pattern from evolutionary_sandbox.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_EVOLUTIONARY_SANDBOX.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_EVOLUTIONARY_SANDBOX.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
