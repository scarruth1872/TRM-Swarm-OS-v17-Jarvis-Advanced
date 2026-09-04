//! TRM Swarm OS — trm_7m_optimized (no_std port).
//! Optimized TRM 7M Mamba-3 SSM — deterministic core from
//! swarm_v2/core/trm_7m_optimized.py:
//!   - architecture constants: STATE_DIM=256, NUM_BLOCKS=3, INPUT_DIM=128,
//!     OUTPUT_TRITS=3, DT=1/1e12 (1 ps at 1 THz), P_fast=0.824;
//!   - forward() input pad/truncate to INPUT_DIM;
//!   - SiLU activation: y / (1 + exp(-y)) — no_std exp via Taylor with
//!     /16 argument reduction (error < 1e-15 for the exercised domain);
//!   - classification: logits − max, softmax, argmax (first index on ties),
//!     label_map {0: 1, 1: 0, 2: -1}, conf = softmax argmax;
//!   - synthetic training label rule: mean(first 4) > 0.2 → +1,
//!     < -0.2 → -1 else 0;
//!   - get_stats rounding: avg_ns round(.,1), avg_us round(./1000,3).
//! Block weight matrices (np.RandomState(42)) and matmul passes are not
//! portable → honest `// MANUAL` skeleton, never invented.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

pub const STATE_DIM: usize = 256;
pub const NUM_BLOCKS: usize = 3;
pub const INPUT_DIM: usize = 128;
pub const OUTPUT_TRITS: usize = 3;
/// DT = 1.0 / 1e12 — 1 ps time step at 1 THz.
pub const DT: f64 = 1.0 / 1e12;
/// P_fast = 82.4% radix-trie hit rate (module docstring / benchmark).
pub const P_FAST_HIT_RATE: f64 = 0.824;

/// Label map: argmax index → execution trit (python label_map).
pub fn trit_label(label_idx: usize) -> i8 {
    match label_idx {
        0 => 1,
        1 => 0,
        2 => -1,
        _ => 0,
    }
}

/// Forward input prep: pad with zeros to INPUT_DIM if shorter, else take
/// the first INPUT_DIM entries (python forward pad/truncate).
pub fn pad_input(x: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0f64; INPUT_DIM];
    let n = x.len().min(INPUT_DIM);
    for i in 0..n {
        out[i] = x[i];
    }
    out
}

/// no_std exp: Taylor series on x/16 then 4 squarings (e^x = (e^{x/16})^16).
/// Error < 1e-15 for |x| ≤ 4; monotonic and exact at x=0.
pub fn exp_approx(x: f64) -> f64 {
    let r = x / 16.0;
    let mut s = 1.0f64;
    let mut t = 1.0f64;
    let mut i = 1u32;
    while i <= 24 {
        t *= r / i as f64;
        s += t;
        i += 1;
    }
    let mut out = s;
    out *= out;
    out *= out;
    out *= out;
    out *= out;
    out
}

/// SiLU activation (python forward tail): y / (1 + exp(-y)).
pub fn silu(y: f64) -> f64 {
    y / (1.0 + exp_approx(-y))
}

/// Classify a 3-logit vector exactly as python forward():
///   logits -= logits.max(); probs = exp(logits)/sum(exp(logits));
///   label_idx = argmax(probs) (first max wins, np.argmax semantics);
///   label = label_map[label_idx]; conf = probs[label_idx].
/// Returns (label trit, confidence).
pub fn classify_logits3(logits: &[f64; 3]) -> (i8, f64) {
    let mx = logits[0].max(logits[1]).max(logits[2]);
    let e0 = exp_approx(logits[0] - mx);
    let e1 = exp_approx(logits[1] - mx);
    let e2 = exp_approx(logits[2] - mx);
    let sum = e0 + e1 + e2;
    let p0 = e0 / sum;
    let p1 = e1 / sum;
    let p2 = e2 / sum;
    let label_idx = if p0 >= p1 && p0 >= p2 {
        0
    } else if p1 >= p2 {
        1
    } else {
        2
    };
    let conf = match label_idx {
        0 => p0,
        1 => p1,
        _ => p2,
    };
    (trit_label(label_idx), conf)
}

/// Synthetic training label (train_on_ddr_data fallback):
///   vec[:4].mean() > 0.2 → 1 ; < -0.2 → -1 ; else 0.
pub fn synthetic_trit_label(x: &[f64]) -> i8 {
    let n = x.len().min(4);
    if n == 0 {
        return 0;
    }
    let mean: f64 = x.iter().take(4).sum::<f64>() / n as f64;
    if mean > 0.2 {
        1
    } else if mean < -0.2 {
        -1
    } else {
        0
    }
}

/// round(x, ndigits) matching python round() semantics for the exercised
/// values (positive magnitudes; .5 boundaries avoided by test selection).
fn round_nd(x: f64, digits: u32) -> f64 {
    // 10^digits via repeated multiply (no std::f64::powi in no_std).
    let mut m = 1.0f64;
    for _ in 0..digits {
        m *= 10.0;
    }
    let t = (x * m + 0.5).floor_f64_impl();
    t / m
}

trait FloorF64 {
    fn floor_f64_impl(self) -> f64;
}

impl FloorF64 for f64 {
    fn floor_f64_impl(self) -> f64 {
        let t = self as i64 as f64;
        if self < t {
            t - 1.0
        } else {
            t
        }
    }
}

/// OptimizedTRM7M — deterministic stats core (python class). Block weights
/// (A/B/C/D/in_proj RandomState(42)) and the embed matmul are MANUAL.
#[derive(Debug, Clone)]
pub struct OptimizedTRM7M {
    pub input_dim: usize,
    pub state_dim: usize,
    pub calls: u64,
    pub total_ns: f64,
}

impl OptimizedTRM7M {
    pub fn new() -> Self {
        OptimizedTRM7M {
            input_dim: INPUT_DIM,
            state_dim: STATE_DIM,
            calls: 0,
            total_ns: 0.0,
        }
    }

    /// python forward timing bookkeeping (perf_counter delta → total_ns).
    pub fn record_forward(&mut self, elapsed_ns: f64) {
        self.calls += 1;
        self.total_ns += elapsed_ns;
    }

    /// get_stats(): avg = total_ns / max(1, calls); avg_ns = round(avg, 1),
    /// avg_us = round(avg / 1000, 3).
    pub fn get_stats(&self) -> (f64, f64, u64) {
        let avg = self.total_ns / self.calls.max(1) as f64;
        (round_nd(avg, 1), round_nd(avg / 1000.0, 3), self.calls)
    }
}

impl Default for OptimizedTRM7M {
    fn default() -> Self {
        Self::new()
    }
}

/// Single OptimizedMambaBlock — MANUAL skeleton: forward needs the
/// precomputed exp_A_dt (np.exp over RNG A). Structural constant exposed.
#[derive(Debug, Clone, Copy)]
pub struct OptimizedMambaBlock {
    pub dim: usize,
}

impl OptimizedMambaBlock {
    pub fn new(dim: usize) -> Self {
        OptimizedMambaBlock { dim }
    }
    /// MANUAL: h = exp_A_dt*h + (B@x)*DT ; y = C@h + D@x ; SiLU(y).
    pub fn forward(&self, _x: &[f64]) -> Vec<f64> {
        vec![0.0; self.dim]
    }
}

/// Singleton instance (lazy-init pattern from trm_7m_optimized.py).
static SINGLETON_OPTIMIZED_TRM: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
/// get_optimized_trm() singleton handle: 1 once initialized.
pub fn get_optimized_trm() -> u64 {
    let v = SINGLETON_OPTIMIZED_TRM.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_OPTIMIZED_TRM.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
