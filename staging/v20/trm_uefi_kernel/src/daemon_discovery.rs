//! TRM Swarm OS — daemon_discovery (bare-metal no_std + alloc).
//! Phase 35 | Rank 169 | TurboVec 16x BT-Quant | Monad Anchor: 0x4D4F4E4144000001
//! Dynamic Agent Discovery Daemon — Direct IPC Heartbeat & Topology Registry.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const PHASE: u32 = 35;
pub const RANK: u32 = 169;
pub const SCALE_FACTOR: f32 = 31.9654;

pub static TRIT_WEIGHTS: &[i8] = &[
    0, -1, -1, 0, -1, -1, 1, 1, 0, 0, -1, 1, 0, 1, 1, -1, 1, 0, -1, 1, 1, 1, -1, 1, -1, -1, -1, -1, 0, 1, -1, -1,
    -1, -1, -1, -1, -1, -1, 1, 1, 0, 0, -1, 0, 0, 1, 0, -1, 0, 1, -1, 0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 0, -1, 0,
    0, -1, -1, -1, -1, 0, 1, 1, 0, -1, 0, 1, -1, 1, 1, -1, 1, 1, 0, 1, 1, 1, -1, 1, 0, -1, -1, -1, 0, 0, -1, 0,
    0, -1, 0, -1, -1, 0, 1, 1, -1, -1, 0, 1, -1, 0, 1, 0, 0, 0, -1, 0, 0, 1, 0, 0, -1, -1, -1, -1, 0, 1, 0, -1,
    0, 0, 0, 0, -1, -1, 0, 0, 0, -1, -1, 0, 0, 1, 1, -1, 0, 1, -1, 1, 0, 1, 0, 1, -1, -1, -1, -1, -1, 1, -1, -1,
    -1, -1, -1, 0, -1, -1, 1, 1, -1, 0, -1, 0, 0, 1, 1, -1, 1, 1, -1, 0, 1, 1, -1, 1, -1, -1, 0, 0, -1, 1, -1, -1,
    -1, -1, -1, -1, 0, -1, 1, 0, -1, -1, -1, 1, 0, 0, 1, -1, 1, 0, -1, 1, 0, 1, -1, 1, 0, -1, -1, -1, 0, 0, -1, 0,
    0, -1, 0, -1, -1, 0, 1, 1, 0, 0, -1, 1, -1, 1, 1, -1, 1, 1, 0, 1, 0, 1, -1, 1, 0, -1, -1, -1, 0, 1, -1, -1
];

static INVOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static SINGLETON_DAEMON_DISCOVERY: AtomicU64 = AtomicU64::new(0);

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum K3 {
    False = 0,
    Unknown = 1,
    True = 2,
}

impl K3 {
    pub fn and(self, other: K3) -> K3 {
        match (self as u8).min(other as u8) {
            0 => K3::False,
            1 => K3::Unknown,
            _ => K3::True,
        }
    }
}

#[inline(always)]
pub fn ternary_dot(inputs: &[f32]) -> f32 {
    let n = inputs.len().min(TRIT_WEIGHTS.len());
    let mut acc = 0.0f32;
    for i in 0..n {
        acc += inputs[i] * (TRIT_WEIGHTS[i] as f32) * SCALE_FACTOR / 16.0;
    }
    acc
}

pub fn daemon_discovery_execute(inputs: &[f32], output: &mut [f32]) -> u32 {
    if inputs.is_empty() || output.is_empty() {
        return 0xDEAD_BEEF;
    }
    INVOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
    let r = ternary_dot(inputs);
    let k = if r.abs() < 1e-3 {
        K3::Unknown
    } else if r > 0.0 {
        K3::True
    } else {
        K3::False
    };
    let _ = k.and(K3::True);
    output[0] = r;
    0x0000_0001
}

#[derive(Debug, Clone)]
pub struct DiscoveredNode {
    pub node_id: String,
    pub name: String,
    pub role: String,
    pub cpu_load: f32,
    pub memory_usage: f32,
    pub last_seen_secs: u64,
}

pub struct DiscoveryRegistry {
    pub nodes: Vec<DiscoveredNode>,
    pub total_heartbeats: u64,
}

impl DiscoveryRegistry {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            total_heartbeats: 0,
        }
    }

    pub fn register_node(&mut self, node_id: &str, name: &str, role: &str, cpu: f32, mem: f32) {
        self.total_heartbeats += 1;
        for node in self.nodes.iter_mut() {
            if node.node_id == node_id {
                node.cpu_load = cpu;
                node.memory_usage = mem;
                node.last_seen_secs = 0;
                return;
            }
        }
        self.nodes.push(DiscoveredNode {
            node_id: String::from(node_id),
            name: String::from(name),
            role: String::from(role),
            cpu_load: cpu,
            memory_usage: mem,
            last_seen_secs: 0,
        });
    }

    pub fn get_active_count(&self) -> usize {
        self.nodes.len()
    }
}

pub fn get_discovery_daemon() -> u64 {
    let v = SINGLETON_DAEMON_DISCOVERY.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_DAEMON_DISCOVERY.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

pub fn register_discovery_heartbeat(node_id: &str, cpu_load: f32, mem_load: f32) -> u64 {
    let mut reg = DiscoveryRegistry::new();
    reg.register_node(node_id, "agent_local", "worker", cpu_load, mem_load);
    reg.total_heartbeats
}


