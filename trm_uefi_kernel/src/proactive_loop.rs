//! TRM Swarm OS - proactive_loop (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/proactive_loop.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS
//  (host-side API URLs removed — no_std kernel makes no HTTP calls)
pub const PLAN_PATH: &'static str = "swarm_v2_artifacts/QIAE_INTEGRATION_PLAN_V2.md";

//  ScheduledTask 
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub name: String,
    pub cron_expr: String,
    pub handler: u64,
    pub action_description: String,
    pub enabled: bool,
    pub last_run: u64,
    pub run_count: i64,
}

impl ScheduledTask {
    pub fn new() -> Self {
        ScheduledTask {
            name: String::new(),
            cron_expr: String::new(),
            handler: 0,
            action_description: String::new(),
            enabled: false,
            last_run: 0,
            run_count: 0,
        }
    }
}

//  get_proactive_loop(manus_engine) 
static SINGLETON_PROACTIVE_LOOP: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_proactive_loop() -> u64 {
    // Singleton handle (lazy-init pattern from proactive_loop.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_PROACTIVE_LOOP.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_PROACTIVE_LOOP.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
