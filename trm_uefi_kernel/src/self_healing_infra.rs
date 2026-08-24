//! TRM Swarm OS - self_healing_infra (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/self_healing_infra.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  ServiceConfig 
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub health_endpoint: String,
    pub restart_command: Vec<String>,
    pub port: i64,
    pub heartbeat_timeout: i64,
    pub restart_cooldown: i64,
    pub max_restart_attempts: i64,
}

impl ServiceConfig {
    pub fn new() -> Self {
        ServiceConfig {
            name: String::new(),
            health_endpoint: String::new(),
            restart_command: vec![],
            port: 0,
            heartbeat_timeout: 0,
            restart_cooldown: 0,
            max_restart_attempts: 0,
        }
    }
}

//  ServiceHealth 
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub service_name: String,
    pub status: u64,
    pub last_heartbeat: f64,
    pub last_response_time: f64,
    pub consecutive_failures: i64,
    pub restart_count: i64,
    pub last_restart: f64,
    pub error_message: String,
    pub pid: i64,
}

impl ServiceHealth {
    pub fn new() -> Self {
        ServiceHealth {
            service_name: String::new(),
            status: 0,
            last_heartbeat: 0.0,
            last_response_time: 0.0,
            consecutive_failures: 0,
            restart_count: 0,
            last_restart: 0.0,
            error_message: String::new(),
            pid: 0,
        }
    }
}

//  get_self_healing_infra() 
static SINGLETON_SELF_HEALING_INFRA: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_self_healing_infra() -> u64 {
    // Singleton handle (lazy-init pattern from self_healing_infra.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_SELF_HEALING_INFRA.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SELF_HEALING_INFRA.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
