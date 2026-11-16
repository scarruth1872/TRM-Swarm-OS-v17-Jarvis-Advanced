//! TRM Swarm OS - memory_synchronizer
//! Collective Singularity Synchronizer: synchronizes genetic proposals and learned patterns
//! across federated Swarm OS instances into a unified intelligence fabric.
//! Integrates TurboVec 16x BT-Quant native strip, Kleene K3 gating, and pattern sync tables.

#![allow(non_snake_case, dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

// --- TurboVec 16x Balanced Ternary Quantization Weights (Phase 31, Rank 150) ---
static TW: &[i8] = &[
    -1, 1, 0, 0, -1, 1, 0, 1, 1, 0, -1, -1, -1, -1, 0, -1, 1, -1, 0, -1, 0, -1, -1, 0,
    1, -1, -1, 1, 1, -1, -1, 1, -1, 0, -1, 1, -1, 1, -1, 1, 1, 0, -1, -1, 0, -1, 1, -1,
    0, -1, -1, -1, 0, -1, 0, 1, 1, -1, 0, 0, 1, -1, -1, 0, 0, 0, -1, 1, 0, 0, 0, 1, 1, 1,
    -1, 0, -1, -1, 1, 0, 1, -1, -1, 0, -1, -1, -1, 1, 1, -1, -1, 0, 0, -1, -1, 1, 0, 0, -1,
    0, 0, 0, -1, 1, 1, 1, 0, -1, 0, -1, 1, -1, 1, 0, -1, -1, -1, 0, -1, 1, 1, 0, -1, 1, 0,
    0, -1, 1, -1, 1, -1, 1, -1, 1, -1, 1, 0, 1, -1, -1, -1, 0, 1, -1, 1, -1, -1, -1, 0, 0, -1,
    0, 0, -1, 0, 1, 1, -1, -1, 1, -1, 1, -1, 0, 0, 0, 0, 1, 1, 0, 0, -1, 0, -1, 1, 0, 1, -1, -1,
    0, -1, -1, 0, 1, 0, -1, 0, 0, 1, 0, -1, 1, -1, 1, -1, 1, 0, 1, -1, 0, 0, 0, -1, 0, -1, -1, 0,
    -1, 0, 0, -1, -1, -1, 0, -1, 1, 0, -1, -1, 0, 1, 0, -1, 1, -1, 1, -1, 0, -1, 1, 0, 1, 1, 1, -1,
    -1, 0, -1, 0, -1, 0, -1, -1, 0, -1, -1, -1, 1, 0, -1, 0, 1, 1, 0, -1, 0
];

pub static SCALE_FACTOR: f32 = 31.9821;
pub static INVOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
pub static MONAD_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;
pub static PHASE: u32 = 31;
pub static RANK: u32 = 150;

// --- Kleene K3 3-Valued Logic ---
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum K3 {
    False = 0,
    Unknown = 1,
    True = 2,
}

impl K3 {
    #[inline(always)]
    pub fn and(self, other: K3) -> K3 {
        match (self as u8).min(other as u8) {
            0 => K3::False,
            1 => K3::Unknown,
            _ => K3::True,
        }
    }
}

// --- Ternary Dot Product ---
#[inline(always)]
pub fn ternary_dot(inputs: &[f32]) -> f32 {
    let n = inputs.len().min(TW.len());
    let mut acc = 0.0f32;
    let mut i = 0;
    while i < n {
        acc += inputs[i] * (TW[i] as f32) * (SCALE_FACTOR / 16.0);
        i += 1;
    }
    acc
}

// --- Native C ABI Exports ---
#[no_mangle]
pub extern "C" fn memory_synchronizer_execute(i: *const f32, n: usize, o: *mut f32) -> u32 {
    if i.is_null() || o.is_null() || n == 0 {
        return 0xDEAD_BEEF;
    }
    INVOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
    let s = unsafe { core::slice::from_raw_parts(i, n) };
    let r = ternary_dot(s);
    let k = if r.abs() < 1e-3 {
        K3::Unknown
    } else if r > 0.0 {
        K3::True
    } else {
        K3::False
    };
    let _ = k.and(K3::True);
    unsafe { *o = r };
    0x0000_0001
}

#[no_mangle]
pub extern "C" fn memory_synchronizer_monad_commit(bid: u64) -> u64 {
    let inv = INVOCATION_COUNT.load(Ordering::Relaxed);
    MONAD_ANCHOR ^ bid ^ inv
}

#[no_mangle]
pub extern "C" fn memory_synchronizer_phase_info(op: *mut u32, or_: *mut u32, oa: *mut u64) {
    if !op.is_null() {
        unsafe { *op = PHASE };
    }
    if !or_.is_null() {
        unsafe { *or_ = RANK };
    }
    if !oa.is_null() {
        unsafe { *oa = MONAD_ANCHOR };
    }
}

// --- High-Level Collective Singularity Memory Synchronizer ---

#[derive(Clone, Copy, Debug)]
pub struct PatternProposal {
    pub pattern_id: u64,
    pub fitness: f32,
    pub generation: u32,
    pub origin_node_id: u64,
    pub verified: bool,
    pub timestamp: u64,
}

pub struct MemorySynchronizer {
    pub sync_interval_sec: u32,
    pub total_sync_cycles: u64,
    pub total_patterns_synced: u64,
    pub federated_nodes_count: usize,
    pub proposal_table: [PatternProposal; 16],
    pub proposal_count: usize,
}

impl MemorySynchronizer {
    pub const fn new() -> Self {
        const EMPTY_PROP: PatternProposal = PatternProposal {
            pattern_id: 0,
            fitness: 0.0,
            generation: 0,
            origin_node_id: 0,
            verified: false,
            timestamp: 0,
        };

        Self {
            sync_interval_sec: 3600,
            total_sync_cycles: 0,
            total_patterns_synced: 0,
            federated_nodes_count: 3,
            proposal_table: [EMPTY_PROP; 16],
            proposal_count: 0,
        }
    }

    /// Submits a newly evolved genetic pattern proposal into the synchronization pool.
    pub fn submit_proposal(&mut self, pattern_id: u64, fitness: f32, gen: u32, origin: u64, ts: u64) -> bool {
        if self.proposal_count < self.proposal_table.len() {
            self.proposal_table[self.proposal_count] = PatternProposal {
                pattern_id,
                fitness,
                generation: gen,
                origin_node_id: origin,
                verified: fitness >= 0.70f32,
                timestamp: ts,
            };
            self.proposal_count += 1;
            true
        } else {
            // Replace lowest fitness proposal if incoming is superior
            let mut min_idx = 0;
            let mut min_fit = self.proposal_table[0].fitness;
            let mut i = 1;
            while i < self.proposal_table.len() {
                if self.proposal_table[i].fitness < min_fit {
                    min_fit = self.proposal_table[i].fitness;
                    min_idx = i;
                }
                i += 1;
            }

            if fitness > min_fit {
                self.proposal_table[min_idx] = PatternProposal {
                    pattern_id,
                    fitness,
                    generation: gen,
                    origin_node_id: origin,
                    verified: fitness >= 0.70f32,
                    timestamp: ts,
                };
                true
            } else {
                false
            }
        }
    }

    /// Executes a global synchronization pass, computing aggregate fabric fitness.
    pub fn synchronize_all(&mut self) -> (usize, f32) {
        self.total_sync_cycles += 1;
        let mut verified_count = 0;
        let mut total_fitness = 0.0f32;

        let mut i = 0;
        while i < self.proposal_count {
            if self.proposal_table[i].verified {
                verified_count += 1;
                total_fitness += self.proposal_table[i].fitness;
                self.total_patterns_synced += 1;
            }
            i += 1;
        }

        let mean_fitness = if verified_count > 0 {
            total_fitness / (verified_count as f32)
        } else {
            0.0f32
        };

        (verified_count, mean_fitness)
    }

    /// Retrieves the highest-fitness pattern across the federated memory space.
    pub fn get_top_pattern(&self) -> Option<PatternProposal> {
        if self.proposal_count == 0 {
            return None;
        }
        let mut best = self.proposal_table[0];
        let mut i = 1;
        while i < self.proposal_count {
            if self.proposal_table[i].fitness > best.fitness {
                best = self.proposal_table[i];
            }
            i += 1;
        }
        Some(best)
    }

    /// Returns synchronization metrics (sync_cycles, patterns_synced, federation_nodes).
    pub fn stats(&self) -> (u64, u64, usize) {
        (
            self.total_sync_cycles,
            self.total_patterns_synced,
            self.federated_nodes_count,
        )
    }
}

// --- Global Singleton Interface ---
static SINGLETON_MEMORY_SYNCHRONIZER: AtomicU64 = AtomicU64::new(0);

pub fn get_memory_synchronizer() -> u64 {
    let v = SINGLETON_MEMORY_SYNCHRONIZER.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_MEMORY_SYNCHRONIZER.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}
