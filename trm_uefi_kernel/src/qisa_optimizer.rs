//! TRM Swarm OS - qisa_optimizer (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/qisa_optimizer.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  AnnealingConfig 
#[derive(Debug, Clone)]
pub struct AnnealingConfig {
    pub initial_temperature: f64,
    pub cooling_rate: f64,
    pub min_temperature: f64,
    pub max_cycles: i64,
    pub tunnel_probability: f64,
    pub tunnel_strength: f64,
    pub convergence_threshold: f64,
}

impl AnnealingConfig {
    pub fn new() -> Self {
        AnnealingConfig {
            initial_temperature: 0.0,
            cooling_rate: 0.0,
            min_temperature: 0.0,
            max_cycles: 0,
            tunnel_probability: 0.0,
            tunnel_strength: 0.0,
            convergence_threshold: 0.0,
        }
    }
}

//  AnnealingResult 
#[derive(Debug, Clone)]
pub struct AnnealingResult {
    pub best_index: i64,
    pub best_score: f64,
    pub initial_best_score: f64,
    pub cycles_run: i64,
    pub tunneling_events: i64,
    pub temperature_final: f64,
    pub history: Vec<Vec<(String, f64)>>,
}

impl AnnealingResult {
    pub fn new() -> Self {
        AnnealingResult {
            best_index: 0,
            best_score: 0.0,
            initial_best_score: 0.0,
            cycles_run: 0,
            tunneling_events: 0,
            temperature_final: 0.0,
            history: vec![],
        }
    }
}

//  get_qisa_optimizer(config) 
static SINGLETON_QISA_OPTIMIZER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_qisa_optimizer() -> u64 {
    // Singleton handle (lazy-init pattern from qisa_optimizer.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_QISA_OPTIMIZER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_QISA_OPTIMIZER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
