//! TRM Swarm OS — Custom Silicon Kernel Instructions Emulation (no_std).
//! Authentic bare-metal port of swarm_v2/core/kernel_instructions.py.
//!
//! Monad Truth Anchor: 0x4D4F4E4144000001
//! Phase 3 Custom Instruction Accelerator: 8 proposed silicon instructions
//! benchmarked and emulated for zero-human test generation and runtime speedup.

#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const INSTRUCTION_COUNT: usize = 8;

pub static INSTRUCTION_NAMES: [&str; 8] = [
    "K3CLASS",
    "MAMBA3STEP",
    "PONDEROMOTIVE",
    "LORA_FUSED",
    "RADIX_TRIE_WALK",
    "DDR_MATCH",
    "TURBOVEC_DOT",
    "INSTELLA_GATE",
];

/// Instruction 1: K3CLASS
/// Ternary K3 classification: {-1, 0, +1} from 256-bit vector in 1 cycle.
pub fn k3_classify(x: &[f32], threshold: f32, out: &mut [i8]) -> usize {
    let n = x.len().min(out.len());
    for i in 0..n {
        if x[i] > threshold {
            out[i] = 1;
        } else if x[i] < -threshold {
            out[i] = -1;
        } else {
            out[i] = 0;
        }
    }
    n
}

/// `_native_custom_kernel_instructions` (kernel_instructions.py:30-48): the combined native entry —
/// K3 classify (`sign(x) * (|x| > threshold)`) AND the ponderomotive trap (`>0.5` -> 1, `<-0.5` -> -1)
/// in one pass. Both halves are already ported (`k3_classify`, `ponderomotive_trap`); this is the
/// single-call wrapper the strip exposes.
pub fn custom_kernel_instructions(x: &[f32], threshold: f32) -> (Vec<i8>, Vec<i8>) {
    let n = x.len();
    let mut k3 = vec![0i8; n];
    let mut trap = vec![0i8; n];
    k3_classify(x, threshold, &mut k3);
    ponderomotive_trap(x, &mut trap);
    (k3, trap)
}

/// Instruction 2: MAMBA3STEP
/// Single Mamba-3 SSM state evolution step: h(t+dt) = h(t) + dt*(A@h + B@x), out = C@h + D@x.
pub fn mamba3_step_scalar(
    dim: usize,
    state_dim: usize,
    x: &[f32],
    h: &mut [f32],
    a: &[f32],
    b: &[f32],
    c: &[f32],
    d: &[f32],
    out: &mut [f32],
) {
    if x.len() < dim || h.len() < state_dim || out.len() < dim {
        return;
    }
    // Update hidden state: dh = A@h + B@x
    for i in 0..state_dim {
        let mut dh = 0.0f32;
        for j in 0..state_dim {
            dh += a[i * state_dim + j] * h[j];
        }
        for j in 0..dim {
            dh += b[i * dim + j] * x[j];
        }
        h[i] += 0.05 * dh;
    }
    // Compute output: out = C@h + D@x
    for i in 0..dim {
        let mut y = 0.0f32;
        for j in 0..state_dim {
            y += c[i * state_dim + j] * h[j];
        }
        for j in 0..dim {
            y += d[i * dim + j] * x[j];
        }
        out[i] = y;
    }
}

/// Instruction 3: PONDEROMOTIVE
/// Ponderomotive dark-node trap: classify envelope into {-1, 0, +1} buckets.
pub fn ponderomotive_trap(envelope: &[f32], out: &mut [i8]) -> usize {
    let n = envelope.len().min(out.len());
    for i in 0..n {
        if envelope[i] > 0.5 {
            out[i] = 1;
        } else if envelope[i] < -0.5 {
            out[i] = -1;
        } else {
            out[i] = 0;
        }
    }
    n
}

/// Instruction 4: LORA_FUSED
/// Fused LoRA forward pass: out = base@x + B@(A@x).
pub fn lora_fused_forward(
    dim: usize,
    rank: usize,
    x: &[f32],
    base: &[f32],
    a: &[f32],
    b: &[f32],
    out: &mut [f32],
) {
    if x.len() < dim || out.len() < dim {
        return;
    }
    // 1. ax = A @ x  (rank-dim)
    let mut ax = [0.0f32; 16]; // support up to rank 16 on bare-metal stack
    let r = rank.min(16);
    for i in 0..r {
        let mut sum = 0.0f32;
        for j in 0..dim {
            sum += a[i * dim + j] * x[j];
        }
        ax[i] = sum;
    }
    // 2. lora = B @ ax + base @ x
    for i in 0..dim {
        let mut sum = 0.0f32;
        for j in 0..dim {
            sum += base[i * dim + j] * x[j];
        }
        for j in 0..r {
            sum += b[i * rank + j] * ax[j];
        }
        out[i] = sum;
    }
}

/// Instruction 5: RADIX_TRIE_WALK
/// Single-cycle radix trie walk on a flat transition table.
pub fn radix_trie_walk_step(flat_trie: &[i32], current_node: usize, nibble: u8) -> Option<usize> {
    let idx = current_node * 16 + ((nibble & 0x0F) as usize);
    if idx < flat_trie.len() {
        let next = flat_trie[idx];
        if next >= 0 {
            Some(next as usize)
        } else {
            None
        }
    } else {
        None
    }
}

/// Instruction 6: DDR_MATCH
/// Parallel DDR antibody needle matching.
pub fn ddr_match_count(text: &str, patterns: &[&str]) -> usize {
    let mut count = 0;
    for &pat in patterns {
        if !pat.is_empty() && text.contains(pat) {
            count += 1;
        }
    }
    count
}

/// Instruction 7: TURBOVEC_DOT
/// Cache-blocked 384-dim TurboVec dot product.
pub fn turbovec_blocked_dot(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let mut sum = 0.0f32;
    let chunk_size = 64;
    for i in (0..n).step_by(chunk_size) {
        let end = (i + chunk_size).min(n);
        for j in i..end {
            sum += a[j] * b[j];
        }
    }
    sum
}

/// Instruction 8: INSTELLA_GATE
/// Instella MoE gating: top-2 indices and score.
pub fn instella_gate_top2(scores: &[f32]) -> (usize, usize, f32, f32) {
    if scores.is_empty() {
        return (0, 0, 0.0, 0.0);
    }
    let mut top1_idx = 0;
    let mut top1_val = scores[0];
    for (i, &s) in scores.iter().enumerate().skip(1) {
        if s > top1_val {
            top1_val = s;
            top1_idx = i;
        }
    }
    let mut top2_idx = if top1_idx == 0 { 1.min(scores.len() - 1) } else { 0 };
    let mut top2_val = scores[top2_idx];
    for (i, &s) in scores.iter().enumerate() {
        if i != top1_idx && s > top2_val {
            top2_val = s;
            top2_idx = i;
        }
    }
    (top1_idx, top2_idx, top1_val, top2_val)
}

static SINGLETON_INSTRUCTIONS: AtomicU64 = AtomicU64::new(0);

/// Global handle initialization.
pub fn get_kernel_instructions() -> u64 {
    let v = SINGLETON_INSTRUCTIONS.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_INSTRUCTIONS.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// Query instruction emulator configuration.
pub fn kernel_instructions_telemetry() -> (u64, usize) {
    (MONAD_ANCHOR, INSTRUCTION_COUNT)
}
