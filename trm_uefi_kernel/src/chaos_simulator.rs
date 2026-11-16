//! TRM Swarm OS - chaos_simulator (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/chaos_simulator.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  ChaosConfig 
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    pub simulate_latency_ms: i64,
    pub offline_nodes: Vec<String>,
    pub inject_duplicate_txs: bool,
}

impl ChaosConfig {
    pub fn new() -> Self {
        ChaosConfig {
            simulate_latency_ms: 0,
            offline_nodes: vec![],
            inject_duplicate_txs: false,
        }
    }
}

//  get_chaos_simulator() 
static SINGLETON_CHAOS_SIMULATOR: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_chaos_simulator() -> u64 {
    // Singleton handle (lazy-init pattern from chaos_simulator.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_CHAOS_SIMULATOR.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_CHAOS_SIMULATOR.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
