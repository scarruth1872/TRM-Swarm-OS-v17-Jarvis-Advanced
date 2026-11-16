//! TRM Swarm OS - federation (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/federation.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const FEDERATION_DIR: u64 = 0;
pub const FEDERATION_API_PREFIX: &'static str = "/federation";

//  SwarmNode 
#[derive(Debug, Clone)]
pub struct SwarmNode {
    pub node_id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub api_key: String,
    pub status: String,
    pub last_seen: String,
    pub capabilities: Vec<String>,
    pub memory_count: i64,
}

impl SwarmNode {
    pub fn new() -> Self {
        SwarmNode {
            node_id: String::new(),
            name: String::new(),
            host: String::new(),
            port: 0,
            api_key: String::new(),
            status: String::new(),
            last_seen: String::new(),
            capabilities: vec![],
            memory_count: 0,
        }
    }
}

//  get_federation() 
static SINGLETON_FEDERATION: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_federation() -> u64 {
    // Singleton handle (lazy-init pattern from federation.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_FEDERATION.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_FEDERATION.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
