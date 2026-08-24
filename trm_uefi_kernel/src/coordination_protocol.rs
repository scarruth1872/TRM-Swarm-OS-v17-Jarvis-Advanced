//! TRM Swarm OS - coordination_protocol (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/coordination_protocol.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  get_coordination_protocol() 
static SINGLETON_COORDINATION_PROTOCOL: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_coordination_protocol() -> u64 {
    // Singleton handle (lazy-init pattern from coordination_protocol.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_COORDINATION_PROTOCOL.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_COORDINATION_PROTOCOL.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
