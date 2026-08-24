//! TRM Swarm OS - phoenix_protocol_engine (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/phoenix_protocol_engine.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  get_phoenix_engine() 
static SINGLETON_PHOENIX_ENGINE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_phoenix_engine() -> u64 {
    // Singleton handle (lazy-init pattern from phoenix_protocol_engine.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_PHOENIX_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_PHOENIX_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
