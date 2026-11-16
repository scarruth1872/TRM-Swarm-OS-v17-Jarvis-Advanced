//! TRM Swarm OS — Hardware Resource Arbiter (no_std).
//! Authentic bare-metal port of swarm_v2/core/resource_arbiter.py.
//!
//! Monad Truth Anchor: 0x4D4F4E4144000001
//! Phase 29, Rank 136 TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified.

#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const VOLTAGE_SCALE_FACTOR: f32 = 31.9234;
pub const VOLTAGE_PHASE: u32 = 29;
pub const VOLTAGE_RANK: u32 = 136;

pub const DEFAULT_VRAM_BUDGET_GB: f32 = 11.5;

/// Phase 29 Rank 136 Balanced Ternary Quantized Weights (64 trits, 2-bit packed).
pub static RESOURCE_ARBITER_WEIGHTS_BT: [u8; 16] = [
    0xA6, 0x59, 0x96, 0x65, 0xAA, 0x55, 0x69, 0x96,
    0x59, 0xA6, 0x6A, 0x95, 0x9A, 0x65, 0x5A, 0xA6,
];

/// Known model footprint in GB.
pub static KNOWN_MODELS: [(&str, f32); 9] = [
    ("deepseek-r1:8b", 5.2),
    ("gemma4:e2b", 3.3),
    ("deepseek-r1:1.5b", 1.5),
    ("llama3.2:latest", 2.0),
    ("phi3:latest", 2.2),
    ("gemma2:2b", 1.6),
    ("starcoder2:3b", 1.7),
    ("qwen2.5-coder:latest", 4.7),
    ("gemma3:270m", 0.3),
];

/// Active model record in memory.
#[derive(Debug, Clone)]
pub struct ActiveModelRecord {
    pub name: String,
    pub size_gb: f32,
    pub is_busy: bool,
    pub last_used_tick: u64,
}

/// Resource allocation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArbiterError {
    OutOfMemory,
    ModelNotFound,
    ModelBusy,
    AlreadyLoaded,
}

/// Hardware Resource Arbiter.
pub struct ResourceArbiter {
    pub vram_budget_gb: f32,
    pub active_models: Vec<ActiveModelRecord>,
    pub current_tick: u64,
    pub allocation_count: u64,
    pub eviction_count: u64,
}

impl ResourceArbiter {
    pub fn new(vram_budget_gb: f32) -> Self {
        Self {
            vram_budget_gb,
            active_models: Vec::new(),
            current_tick: 0,
            allocation_count: 0,
            eviction_count: 0,
        }
    }

    pub fn active_vram_used(&self) -> f32 {
        self.active_models.iter().map(|m| m.size_gb).sum()
    }

    pub fn vram_available(&self) -> f32 {
        let used = self.active_vram_used();
        if self.vram_budget_gb > used {
            self.vram_budget_gb - used
        } else {
            0.0
        }
    }

    pub fn lookup_model_size(&self, model_name: &str) -> Option<f32> {
        for &(name, size) in &KNOWN_MODELS {
            if name == model_name {
                return Some(size);
            }
        }
        None
    }

    pub fn allocate_model(&mut self, model_name: &str) -> Result<f32, ArbiterError> {
        self.current_tick += 1;
        let size = self.lookup_model_size(model_name).ok_or(ArbiterError::ModelNotFound)?;

        if self.active_models.iter().any(|m| m.name == model_name) {
            return Err(ArbiterError::AlreadyLoaded);
        }

        // Check if eviction is necessary
        while self.vram_available() < size {
            if !self.evict_least_recently_used() {
                return Err(ArbiterError::OutOfMemory);
            }
        }

        self.active_models.push(ActiveModelRecord {
            name: String::from(model_name),
            size_gb: size,
            is_busy: false,
            last_used_tick: self.current_tick,
        });

        self.allocation_count += 1;
        Ok(size)
    }

    pub fn set_model_busy(&mut self, model_name: &str, busy: bool) -> bool {
        if let Some(m) = self.active_models.iter_mut().find(|m| m.name == model_name) {
            m.is_busy = busy;
            if busy {
                self.current_tick += 1;
                m.last_used_tick = self.current_tick;
            }
            true
        } else {
            false
        }
    }

    pub fn evict_least_recently_used(&mut self) -> bool {
        let mut oldest_idx: Option<usize> = None;
        let mut oldest_tick = u64::MAX;

        for (i, m) in self.active_models.iter().enumerate() {
            if !m.is_busy && m.last_used_tick < oldest_tick {
                oldest_tick = m.last_used_tick;
                oldest_idx = Some(i);
            }
        }

        if let Some(idx) = oldest_idx {
            self.active_models.remove(idx);
            self.eviction_count += 1;
            true
        } else {
            false
        }
    }

    pub fn expand_budget(&mut self, additional_gb: f32) {
        if additional_gb > 0.0 {
            self.vram_budget_gb += additional_gb;
        }
    }
}

static SINGLETON_RESOURCE_ARBITER: AtomicU64 = AtomicU64::new(0);

/// Global handle initialization.
pub fn get_resource_arbiter() -> u64 {
    let v = SINGLETON_RESOURCE_ARBITER.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_RESOURCE_ARBITER.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// Query resource arbiter configuration and telemetry.
pub fn resource_arbiter_telemetry() -> (u64, u32, u32, f32) {
    (MONAD_ANCHOR, VOLTAGE_PHASE, VOLTAGE_RANK, VOLTAGE_SCALE_FACTOR)
}
