//! TRM Swarm OS — SIMD Accelerator (no_std port).
//! 1:1 core semantics from swarm_v2/core/simd_accelerator.py — Catalyst.
//! Deterministic vector ops: dot product, elementwise, normalize.
//! The no_std kernel computes scalar (no SIMD intrinsics in UEFI target),
//! but the semantics match the python reference exactly.

extern crate alloc;
use alloc::vec::Vec;
use alloc::vec;

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

// ── The oracle's kernels (swarm_v2/core/simd_accelerator.py + strip_kleene_quantizer.rs) ─────────
//
// Four numeric kernels that the module did not carry:
//
//   `_kleene_quantize(values, peak, valley)` -- the Kleene ternary quantizer: `> peak` is +1,
//   `< valley` is -1, else 0. Identical in the python fallback and in `strip_kleene_quantizer.dll`,
//   which is what the port is checked against bit for bit.
//
//   `_intensity_classify(E_x, peak, valley)` -- `I = |E|^2` per sample, then the same three-way split,
//   counting pass/fail/ambiguous. The trits AND the three counts are the return value.
//
//   `_turbo_2bit_pack(rows, dim)` -- TurboVec 2-bit packing: per row, `scale = 3 / max(max-min, 1e-8)`,
//   then four quantised values per byte with `q = clamp(round((v - min) * scale), 0, 3)` and
//   `byte |= q << (2*k)`. `round` is numpy's round-half-to-EVEN, which the port reproduces -- a
//   half-away-from-zero would differ on exact halves.
//
//   `_sub_agent_reduce(results, weights)` -- a weighted majority vote over trits: accumulate the weight
//   into one of three buckets, then `argmax - 1`. numpy's argmax returns the FIRST maximum, so a tie
//   resolves to -1, then 0, then +1; the port keeps that order.

pub const PACK_EPS: f32 = 1e-8;

/// `_kleene_quantize_avx2`, the oracle's dispatching name for this kernel (its numba variant
/// `_kleene_quantize_avx2_numba` is the same algorithm written for JIT, not a second rule).
pub fn kleene_quantize_avx2(values: &[f32], peak: f32, valley: f32) -> Vec<i8> {
    values
        .iter()
        .map(|v| if *v > peak { 1 } else if *v < valley { -1 } else { 0 })
        .collect()
}

/// `_intensity_classify`: (trits, pass, fail, ambiguous).
pub fn intensity_classify(e_x: &[f32], peak: f32, valley: f32) -> (Vec<i8>, usize, usize, usize) {
    let mut trits: Vec<i8> = Vec::with_capacity(e_x.len());
    let (mut pass, mut fail, mut ambi) = (0usize, 0usize, 0usize);
    for e in e_x.iter() {
        let val = e * e;
        if val > peak {
            trits.push(1);
            pass += 1;
        } else if val < valley {
            trits.push(-1);
            fail += 1;
        } else {
            trits.push(0);
            ambi += 1;
        }
    }
    (trits, pass, fail, ambi)
}

/// `floor` without std -- bare metal has no `f64::floor`, the same gap as `log2`, `round` and `sqrt`.
fn floor_f64(x: f64) -> f64 {
    let t = x as i64 as f64; // truncates toward zero
    if x < 0.0 && t != x { t - 1.0 } else { t }
}

/// numpy's `round`: halfway cases go to the EVEN neighbour.
fn round_half_even(x: f64) -> f64 {
    let f = floor_f64(x);
    let diff = x - f;
    if diff > 0.5 {
        f + 1.0
    } else if diff < 0.5 {
        f
    } else if (f as i64) % 2 == 0 {
        f
    } else {
        f + 1.0
    }
}

/// `_turbo_2bit_pack`: one row of `dim` floats -> `(dim + 3) / 4` packed bytes.
pub fn turbo_2bit_pack_row(v: &[f32]) -> Vec<u8> {
    let dim = v.len();
    let packed_dim = (dim + 3) / 4;
    let mut out: Vec<u8> = vec![0u8; packed_dim];
    if dim == 0 {
        return out;
    }
    let mut v_min = v[0];
    let mut v_max = v[0];
    for x in v.iter() {
        if *x < v_min {
            v_min = *x;
        }
        if *x > v_max {
            v_max = *x;
        }
    }
    let span = if v_max - v_min > PACK_EPS { v_max - v_min } else { PACK_EPS };
    let scale = 3.0f32 / span;
    let mut j = 0usize;
    while j < dim {
        let mut byte_val: u8 = 0;
        for k in 0..4 {
            if j + k < dim {
                let raw = ((v[j + k] - v_min) * scale) as f64;
                let q = round_half_even(raw).min(3.0).max(0.0) as u8;
                byte_val |= q << (k * 2);
            }
        }
        out[j / 4] = byte_val;
        j += 4;
    }
    out
}

/// `_turbo_2bit_pack` over rows of a fixed dimension.
pub fn turbo_2bit_pack(rows: &[f32], dim: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    if dim == 0 {
        return out;
    }
    let mut i = 0usize;
    while i + dim <= rows.len() {
        out.extend_from_slice(&turbo_2bit_pack_row(&rows[i..i + dim]));
        i += dim;
    }
    out
}

/// `_sub_agent_reduce`: the weighted majority trit, ties resolving -1 then 0 then +1.
pub fn sub_agent_reduce(results: &[i8], weights: &[f32]) -> i8 {
    let mut scores = [0.0f32; 3];
    for i in 0..results.len() {
        let idx = results[i] as i32 + 1;
        if idx >= 0 && idx < 3 {
            scores[idx as usize] += weights[i];
        }
    }
    let mut best = 0usize;
    for i in 1..3 {
        if scores[i] > scores[best] {
            best = i;
        }
    }
    best as i8 - 1
}
