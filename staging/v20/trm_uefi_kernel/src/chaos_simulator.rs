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

// ── ChaosSimulator predicates (chaos_simulator.py:73-83) ────────────────────────────────────────
// `process_latency` and `is_node_isolated` are pure config accessors: the session-active flag and
// the injected latency/offline set arrive as arguments (firmware holds no simulator singleton).

/// `process_latency`: `simulate_latency_ms / 1000.0` when a session is active and the latency is
/// positive, else 0.0.
pub fn process_latency(active_session: bool, simulate_latency_ms: i64) -> f64 {
    if active_session && simulate_latency_ms > 0 {
        simulate_latency_ms as f64 / 1000.0
    } else {
        0.0
    }
}

/// `is_node_isolated`: `node_id in offline_nodes` when a session is active, else false.
pub fn is_node_isolated(active_session: bool, offline_nodes: &[&str], node_id: &str) -> bool {
    active_session && offline_nodes.contains(&node_id)
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
