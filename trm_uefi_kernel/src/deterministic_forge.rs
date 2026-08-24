//! TRM Swarm OS - deterministic_forge (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/deterministic_forge.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  get_deterministic_forge() 
static SINGLETON_DETERMINISTIC_FORGE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_deterministic_forge() -> u64 {
    // Singleton handle (lazy-init pattern from deterministic_forge.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_DETERMINISTIC_FORGE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_DETERMINISTIC_FORGE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
