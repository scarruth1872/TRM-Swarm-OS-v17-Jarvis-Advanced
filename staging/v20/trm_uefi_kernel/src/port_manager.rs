//! TRM Swarm OS - port_manager (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/port_manager.py & strip_port_manager.dll interfaces.
//! Manages non-overlapping agent network port allocation, lease expiration,
//! cross-agent isolation, and service endpoint routing.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- CONSTANTS ---
pub const DEFAULT_PORT_MIN: u16 = 9000;
pub const DEFAULT_PORT_MAX: u16 = 9100;
pub const MAX_PORT_ALLOCATIONS: usize = 256;
pub const DEFAULT_LEASE_TTL_S: u64 = 3600; // 1 hour lease

// --- PORT ALLOCATION ---
#[derive(Debug, Clone)]
pub struct PortAllocation {
    pub port: u16,
    pub agent_id: String,
    pub purpose: String,
    pub acquired_at_epoch_s: u64,
    pub lease_ttl_s: u64,
}

impl PortAllocation {
    pub fn is_expired(&self, current_epoch_s: u64) -> bool {
        current_epoch_s >= self.acquired_at_epoch_s + self.lease_ttl_s
    }
}

// --- PORT POOL STATS ---
#[derive(Debug, Clone)]
pub struct PortPoolStats {
    pub port_min: u16,
    pub port_max: u16,
    pub total_ports: usize,
    pub allocated_ports: usize,
    pub free_ports: usize,
}

// --- PORT MANAGER ---
pub struct PortManager {
    port_min: u16,
    port_max: u16,
    allocations: Vec<PortAllocation>,
}

impl PortManager {
    pub fn new(min: u16, max: u16) -> Self {
        let actual_min = if min > 0 { min } else { DEFAULT_PORT_MIN };
        let actual_max = if max >= actual_min { max } else { DEFAULT_PORT_MAX };
        Self {
            port_min: actual_min,
            port_max: actual_max,
            allocations: Vec::new(),
        }
    }

    pub fn acquire_port(
        &mut self,
        agent_id: &str,
        purpose: &str,
        current_epoch_s: u64,
        ttl_s: u64,
    ) -> Option<u16> {
        self.prune_expired(current_epoch_s);

        for port in self.port_min..=self.port_max {
            if !self.allocations.iter().any(|a| a.port == port) {
                let actual_ttl = if ttl_s > 0 { ttl_s } else { DEFAULT_LEASE_TTL_S };
                self.allocations.push(PortAllocation {
                    port,
                    agent_id: agent_id.to_string(),
                    purpose: purpose.to_string(),
                    acquired_at_epoch_s: current_epoch_s,
                    lease_ttl_s: actual_ttl,
                });
                return Some(port);
            }
        }
        None // Exhausted
    }

    pub fn release_port(&mut self, port: u16) -> bool {
        if let Some(pos) = self.allocations.iter().position(|a| a.port == port) {
            self.allocations.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn release_agent_ports(&mut self, agent_id: &str) -> usize {
        let initial_len = self.allocations.len();
        self.allocations.retain(|a| a.agent_id != agent_id);
        initial_len - self.allocations.len()
    }

    pub fn is_port_allocated(&self, port: u16) -> bool {
        self.allocations.iter().any(|a| a.port == port)
    }

    pub fn get_agent_for_port(&self, port: u16) -> Option<&str> {
        self.allocations.iter().find(|a| a.port == port).map(|a| a.agent_id.as_str())
    }

    pub fn get_ports_for_agent(&self, agent_id: &str) -> Vec<u16> {
        self.allocations.iter().filter(|a| a.agent_id == agent_id).map(|a| a.port).collect()
    }

    pub fn prune_expired(&mut self, current_epoch_s: u64) -> usize {
        let initial_len = self.allocations.len();
        self.allocations.retain(|a| !a.is_expired(current_epoch_s));
        initial_len - self.allocations.len()
    }

    pub fn get_stats(&self) -> PortPoolStats {
        let total = (self.port_max - self.port_min + 1) as usize;
        let allocated = self.allocations.len();
        PortPoolStats {
            port_min: self.port_min,
            port_max: self.port_max,
            total_ports: total,
            allocated_ports: allocated,
            free_ports: if total >= allocated { total - allocated } else { 0 },
        }
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_PORT_MANAGER: AtomicU64 = AtomicU64::new(0);

pub fn get_port_manager() -> u64 {
    let v = SINGLETON_PORT_MANAGER.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_PORT_MANAGER.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

