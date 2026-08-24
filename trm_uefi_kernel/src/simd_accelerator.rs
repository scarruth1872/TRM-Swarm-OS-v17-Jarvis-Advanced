//! TRM Swarm OS — SIMD Accelerator (no_std port).
//! 1:1 core semantics from swarm_v2/core/simd_accelerator.py — Catalyst.
//! Deterministic vector ops: dot product, elementwise, normalize.
//! The no_std kernel computes scalar (no SIMD intrinsics in UEFI target),
//! but the semantics match the python reference exactly.

extern crate alloc;
use alloc::vec::Vec;

/// Dot product of two vectors.
pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    let mut s = 0.0f64;
    for i in 0..n {
        s += a[i] * b[i];
    }
    s
}

/// Elementwise multiply into a new vector.
pub fn elementwise_mul(a: &[f64], b: &[f64]) -> Vec<f64> {
    let n = a.len().min(b.len());
    (0..n).map(|i| a[i] * b[i]).collect()
}

/// L2 normalize in place.
pub fn normalize(v: &mut [f64]) {
    let mut s = 0.0f64;
    for x in v.iter() {
        s += x * x;
    }
    let norm = sqrt_approx(s);
    if norm > 1e-12 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

/// no_std f64 sqrt (Newton-Raphson on bit representation — UEFI has no libm).
fn sqrt_approx(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let bits = x.to_bits() as u64;
    let seed_bits: u64 = ((bits + (1023u64 << 52)) >> 1) as u64;
    let mut r = f64::from_bits(seed_bits as u64);
    for _ in 0..5 {
        r = 0.5 * (r + x / r);
    }
    r
}

/// Vectorized classification: weighted sum + threshold.
/// Returns (score, decision) where decision = score >= threshold.
pub fn classify(features: &[f64], weights: &[f64], threshold: f64) -> (f64, bool) {
    let score = dot(features, weights);
    (score, score >= threshold)
}
