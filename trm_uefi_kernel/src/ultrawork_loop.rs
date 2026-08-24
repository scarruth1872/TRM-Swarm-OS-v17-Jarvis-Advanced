//! TRM Swarm OS - ultrawork_loop (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/ultrawork_loop.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const MISSIONS_DIR: u64 = 0;

//  MissionStep 
#[derive(Debug, Clone)]
pub struct MissionStep {
    pub description: String,
    pub acceptance_criteria: String,
    pub status: String,
    pub result: String,
    pub attempts: i64,
}

impl MissionStep {
    pub fn new() -> Self {
        MissionStep {
            description: String::new(),
            acceptance_criteria: String::new(),
            status: String::new(),
            result: String::new(),
            attempts: 0,
        }
    }
}

//  MissionState 
#[derive(Debug, Clone)]
pub struct MissionState {
    pub mission_id: String,
    pub objective: String,
    pub phase: String,
    pub steps: Vec<Vec<(String, f64)>>,
    pub plan_text: String,
    pub progress_log: Vec<String>,
    pub verification_result: String,
    pub retry_count: i64,
    pub max_retries: i64,
    pub created_at: String,
    pub completed_at: String,
}

impl MissionState {
    pub fn new() -> Self {
        MissionState {
            mission_id: String::new(),
            objective: String::new(),
            phase: String::new(),
            steps: vec![],
            plan_text: String::new(),
            progress_log: vec![],
            verification_result: String::new(),
            retry_count: 0,
            max_retries: 0,
            created_at: String::new(),
            completed_at: String::new(),
        }
    }
}

//  get_ultrawork_loop() 
static SINGLETON_ULTRAWORK_LOOP: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_ultrawork_loop() -> u64 {
    // Singleton handle (lazy-init pattern from ultrawork_loop.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_ULTRAWORK_LOOP.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_ULTRAWORK_LOOP.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
