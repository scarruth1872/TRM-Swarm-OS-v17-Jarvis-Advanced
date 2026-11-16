//! TRM Swarm OS — monitor_daemon (bare-metal no_std + alloc).
//! Phase 31 | Rank 149 | TurboVec 16x BT-Quant | Monad Anchor: 0x4D4F4E4144000001
//! Agent mesh health sensory loop and coherence watchdog.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const PHASE: u32 = 31;
pub const RANK: u32 = 149;
pub const SCALE_FACTOR: f32 = 31.8251;

pub static TRIT_WEIGHTS: &[i8] = &[
    -1, 1, -1, 0, 1, -1, -1, 1, 1, -1, 0, -1, 0, 0, -1, 1, 1, -1, -1, -1, 1, -1, 0, -1, -1, -1, 0, -1, 0, -1, 0, -1,
    -1, 0, -1, 0, 1, -1, -1, 1, 0, -1, 1, 0, -1, 0, 0, 1, 1, -1, 0, -1, 1, -1, -1, -1, -1, -1, -1, 0, 0, 0, -1, -1,
    -1, 1, -1, 1, 1, -1, -1, 0, 1, -1, 0, 0, -1, 1, -1, 1, 0, -1, 0, -1, 0, -1, -1, -1, 0, -1, -1, -1, 0, -1, 0, -1,
    0, 1, -1, 1, 0, 0, 0, 0, 0, -1, 1, 0, 0, 0, -1, 0, 1, 0, 0, -1, 0, -1, -1, -1, -1, -1, -1, 0, 1, -1, -1, -1,
    -1, 0, -1, 0, 1, -1, -1, 1, 0, 0, 1, -1, 0, 1, 0, 1, 1, -1, 0, -1, 1, 0, -1, -1, -1, -1, -1, 0, 1, -1, -1, -1,
    -1, 0, 0, 0, 1, -1, -1, 1, 1, 0, 1, 0, 0, 1, -1, 1, 0, 0, 0, 0, 0, -1, -1, -1, -1, -1, -1, 0, 1, -1, -1, -1,
    -1, 1, -1, 1, 0, -1, 0, 1, 1, -1, 1, -1, -1, 0, -1, 1, 1, -1, -1, -1, 0, -1, 0, -1, 0, -1, 0, -1, 1, 0, -1, -1,
    -1, 1, -1, 1, 1, -1, -1, 1, 0, -1, 0, -1, -1, 0, -1, 1, 1, -1, 0, -1, 1, -1, -1, 0, -1, -1, -1, -1, 1, 0, -1, -1
];

static INVOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static SINGLETON_MONITOR_DAEMON: AtomicU64 = AtomicU64::new(0);

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

pub fn monitor_daemon_execute(inputs: &[f32], output: &mut [f32]) -> u32 {
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

pub struct CoherenceThresholds {
    pub heartbeat_timeout_secs: u64,
    pub min_active_agents: usize,
    pub max_drift_cycles: u64,
}

impl CoherenceThresholds {
    pub fn default_limits() -> Self {
        Self {
            heartbeat_timeout_secs: 300,
            min_active_agents: 3,
            max_drift_cycles: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonitorStatus {
    pub is_healthy: bool,
    pub active_agent_count: usize,
    pub coherence_score: f64,
    pub checks_completed: u64,
}

pub struct MonitorDaemon {
    pub thresholds: CoherenceThresholds,
    pub checks: u64,
    pub anomalies_detected: u64,
}

impl MonitorDaemon {
    pub fn new() -> Self {
        Self {
            thresholds: CoherenceThresholds::default_limits(),
            checks: 0,
            anomalies_detected: 0,
        }
    }

    pub fn inspect_mesh(&mut self, active_nodes: usize, heartbeat_age_secs: u64) -> MonitorStatus {
        self.checks += 1;
        let mut healthy = true;

        if active_nodes < self.thresholds.min_active_agents {
            healthy = false;
            self.anomalies_detected += 1;
        }

        if heartbeat_age_secs > self.thresholds.heartbeat_timeout_secs {
            healthy = false;
            self.anomalies_detected += 1;
        }

        let coherence = if healthy {
            1.0 - (heartbeat_age_secs as f64 / (self.thresholds.heartbeat_timeout_secs as f64 * 2.0)).min(0.5)
        } else {
            0.2
        };

        MonitorStatus {
            is_healthy: healthy,
            active_agent_count: active_nodes,
            coherence_score: coherence,
            checks_completed: self.checks,
        }
    }
}

pub fn get_monitor_daemon() -> u64 {
    let v = SINGLETON_MONITOR_DAEMON.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_MONITOR_DAEMON.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

pub fn check_mesh_coherence(active_nodes: usize, heartbeat_age_secs: u64) -> (bool, f64) {
    let mut daemon = MonitorDaemon::new();
    let status = daemon.inspect_mesh(active_nodes, heartbeat_age_secs);
    (status.is_healthy, status.coherence_score)
}


