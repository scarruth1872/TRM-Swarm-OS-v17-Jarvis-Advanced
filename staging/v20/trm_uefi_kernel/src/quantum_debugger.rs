//! TRM Swarm OS - quantum_debugger (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/quantum_debugger.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  StateSnapshot 
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub timestamp: f64,
    pub mesh_healthy: bool,
    pub total_active_traces: i64,
    pub entities: Vec<(String, f64)>,
}

impl StateSnapshot {
    pub fn new() -> Self {
        StateSnapshot {
            timestamp: 0.0,
            mesh_healthy: false,
            total_active_traces: 0,
            entities: vec![],
        }
    }
}

//  get_quantum_debugger() 
static SINGLETON_QUANTUM_DEBUGGER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_quantum_debugger() -> u64 {
    // Singleton handle (lazy-init pattern from quantum_debugger.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_QUANTUM_DEBUGGER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_QUANTUM_DEBUGGER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
