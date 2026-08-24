//! TRM Swarm OS - architectural_rl_agent (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/architectural_rl_agent.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  PolicyState 
#[derive(Debug, Clone)]
pub struct PolicyState {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub active_nodes: i64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
}

impl PolicyState {
    pub fn new() -> Self {
        PolicyState {
            cpu_percent: 0.0,
            memory_percent: 0.0,
            active_nodes: 0,
            avg_latency_ms: 0.0,
            error_rate: 0.0,
        }
    }
}

//  ArchitecturalAction 
#[derive(Debug, Clone)]
pub struct ArchitecturalAction {
    pub action_type: String,
    pub parameters: Vec<(String, f64)>,
    pub expected_reward: f64,
}

impl ArchitecturalAction {
    pub fn new() -> Self {
        ArchitecturalAction {
            action_type: String::new(),
            parameters: vec![],
            expected_reward: 0.0,
        }
    }
}

//  get_architectural_rl_agent() 
static SINGLETON_ARCHITECTURAL_RL_AGENT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_architectural_rl_agent() -> u64 {
    // Singleton handle (lazy-init pattern from architectural_rl_agent.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_ARCHITECTURAL_RL_AGENT.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_ARCHITECTURAL_RL_AGENT.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
