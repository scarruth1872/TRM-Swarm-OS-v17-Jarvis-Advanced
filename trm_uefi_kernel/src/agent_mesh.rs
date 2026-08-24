//! TRM Swarm OS - agent_mesh (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/agent_mesh.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const MESH_DIR: &'static str = "swarm_v2_mesh";

//  MeshNodeSchema 
#[derive(Debug, Clone)]
pub struct MeshNodeSchema {
    pub node_id: String,
    pub name: String,
    pub role: String,
    pub host: String,
    pub port: i64,
    pub specialties: Vec<String>,
    pub skills: Vec<String>,
    pub hardware: Vec<(String, f64)>,
    pub status: String,
    pub last_heartbeat: f64,
    pub joined_at: String,
    pub task_count: i64,
    pub peer_connections: i64,
}

impl MeshNodeSchema {
    pub fn new() -> Self {
        MeshNodeSchema {
            node_id: String::new(),
            name: String::new(),
            role: String::new(),
            host: String::new(),
            port: 0,
            specialties: vec![],
            skills: vec![],
            hardware: vec![],
            status: String::new(),
            last_heartbeat: 0.0,
            joined_at: String::new(),
            task_count: 0,
            peer_connections: 0,
        }
    }
}

//  get_agent_mesh() 
static SINGLETON_AGENT_MESH: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_agent_mesh() -> u64 {
    // Singleton handle (lazy-init pattern from agent_mesh.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_AGENT_MESH.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_AGENT_MESH.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
