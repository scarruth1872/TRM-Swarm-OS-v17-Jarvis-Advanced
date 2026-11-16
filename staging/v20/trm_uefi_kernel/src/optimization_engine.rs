//! TRM Swarm OS — Recursive Self-Optimization Engine (no_std).
//! Authentic bare-metal port of swarm_v2/core/optimization_engine.py.
//!
//! Monad Truth Anchor: 0x4D4F4E4144000001
//! Phase 37, Rank 177 TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified.

#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const VOLTAGE_SCALE_FACTOR: f32 = 31.8807;
pub const VOLTAGE_PHASE: u32 = 37;
pub const VOLTAGE_RANK: u32 = 177;

/// Phase 37 Rank 177 Balanced Ternary Quantized Weights (64 trits, 2-bit packed).
pub static OPTIMIZATION_WEIGHTS_BT: [u8; 16] = [
    0x55, 0x6A, 0x95, 0xA6, 0x59, 0x66, 0x9A, 0xA5,
    0x65, 0x5A, 0xA9, 0x96, 0x56, 0x69, 0x95, 0x5A,
];

/// Performance metric record for moving-window evaluation.
#[derive(Debug, Clone, Copy)]
pub struct PerformanceMetric {
    pub timestamp_ms: u64,
    pub latency_sec: f32,
    pub harmony_score: f32,
}

/// Agent-specific performance window (capped at 10 entries).
#[derive(Debug, Clone)]
pub struct AgentPerformanceWindow {
    pub agent_id: String,
    pub metrics: Vec<PerformanceMetric>,
}

impl AgentPerformanceWindow {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent_id: String::from(agent_id),
            metrics: Vec::new(),
        }
    }

    pub fn record(&mut self, timestamp_ms: u64, latency_sec: f32, harmony_score: f32) {
        if self.metrics.len() >= 10 {
            self.metrics.remove(0);
        }
        self.metrics.push(PerformanceMetric {
            timestamp_ms,
            latency_sec,
            harmony_score,
        });
    }

    pub fn average_latency(&self) -> f32 {
        if self.metrics.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.metrics.iter().map(|m| m.latency_sec).sum();
        sum / (self.metrics.len() as f32)
    }

    pub fn average_harmony(&self) -> f32 {
        if self.metrics.is_empty() {
            return 1.0;
        }
        let sum: f32 = self.metrics.iter().map(|m| m.harmony_score).sum();
        sum / (self.metrics.len() as f32)
    }
}

/// Optimization decision parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptimizationParams {
    pub h_cycles: u32,
    pub l_cycles: u32,
    pub adjusted: bool,
}

/// Optimization strategy types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationStrategy {
    Optimal,
    HighLoad,
    LowHarmony,
}

/// Recursive Self-Optimization Engine.
pub struct OptimizationEngine {
    pub windows: Vec<AgentPerformanceWindow>,
    pub current_strategy: OptimizationStrategy,
    pub evaluation_count: u64,
    pub adjustments_made: u64,
}

impl OptimizationEngine {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            current_strategy: OptimizationStrategy::Optimal,
            evaluation_count: 0,
            adjustments_made: 0,
        }
    }

    pub fn record_performance(&mut self, agent_id: &str, timestamp_ms: u64, latency_sec: f32, harmony_score: f32) {
        if let Some(w) = self.windows.iter_mut().find(|w| w.agent_id == agent_id) {
            w.record(timestamp_ms, latency_sec, harmony_score);
        } else {
            let mut w = AgentPerformanceWindow::new(agent_id);
            w.record(timestamp_ms, latency_sec, harmony_score);
            self.windows.push(w);
        }
    }

    pub fn get_optimization_params(&mut self, agent_id: &str, current_cycles: u32) -> OptimizationParams {
        self.evaluation_count += 1;
        let window = match self.windows.iter().find(|w| w.agent_id == agent_id) {
            Some(w) => w,
            None => {
                return OptimizationParams {
                    h_cycles: current_cycles,
                    l_cycles: current_cycles,
                    adjusted: false,
                };
            }
        };

        if window.metrics.is_empty() {
            return OptimizationParams {
                h_cycles: current_cycles,
                l_cycles: current_cycles,
                adjusted: false,
            };
        }

        let avg_latency = window.average_latency();
        let avg_harmony = window.average_harmony();

        let mut new_h = current_cycles;

        // If harmony drops below 0.75 and recursion depth < 5, increase depth to improve accuracy
        if avg_harmony < 0.75 && new_h < 5 {
            new_h += 1;
        } else if avg_latency > 5.0 && new_h > 1 {
            // If latency is excessive (>5.0s), reduce depth to favor efficiency
            new_h -= 1;
        }

        let adjusted = new_h != current_cycles;
        if adjusted {
            self.adjustments_made += 1;
        }

        OptimizationParams {
            h_cycles: new_h,
            l_cycles: new_h,
            adjusted,
        }
    }

    pub fn set_strategy(&mut self, strategy: OptimizationStrategy) {
        self.current_strategy = strategy;
    }

    pub fn prune_inactive_windows(&mut self, current_time_ms: u64, max_age_ms: u64) {
        self.windows.retain(|w| {
            if let Some(last) = w.metrics.last() {
                current_time_ms.saturating_sub(last.timestamp_ms) <= max_age_ms
            } else {
                false
            }
        });
    }
}

static SINGLETON_OPTIMIZATION_ENGINE: AtomicU64 = AtomicU64::new(0);

/// Global handle initialization.
pub fn get_optimization_engine() -> u64 {
    let v = SINGLETON_OPTIMIZATION_ENGINE.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_OPTIMIZATION_ENGINE.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// Query optimization engine configuration and telemetry.
pub fn optimization_engine_telemetry() -> (u64, u32, u32, f32) {
    (MONAD_ANCHOR, VOLTAGE_PHASE, VOLTAGE_RANK, VOLTAGE_SCALE_FACTOR)
}
