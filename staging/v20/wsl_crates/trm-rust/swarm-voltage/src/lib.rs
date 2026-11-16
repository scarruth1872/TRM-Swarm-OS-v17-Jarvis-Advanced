//! swarm-voltage — Build #1: the qwen3moe forward pass (voltage field).
//!
//! Implements the standard Qwen3 MoE architecture on the REAL tensors:
//!   embed → per-block {RMSNorm → QKV (QK-norm) → RoPE → attention →
//!   residual → RMSNorm → MoE top-2 FFN → residual} ×28 → output
//!   norm → hidden state (NO softmax — the voltage representation).
//!
//! Stage 1 (Q4_K dequant) is verified. This module adds the forward
//! pieces, each independently testable.

use std::collections::HashMap;

pub mod forward;

// ─── Q4_K dequant (Stage 1, verified) ───
pub fn dequantize_q4k_block(data: &[u8], out: &mut [f32]) {
    debug_assert!(data.len() >= 144);
    debug_assert!(out.len() >= 256);
    let d = f16_to_f32(data[0], data[1]);
    let dmin = f16_to_f32(data[2], data[3]);
    let scales = &data[4..16];
    let qs = &data[16..144];
    let mut oi = 0usize;
    let mut q = 0usize;
    for j in (0..8).step_by(2) {
        let (sc1, m1) = get_scale_min_k4(j, scales);
        let (sc2, m2) = get_scale_min_k4(j + 1, scales);
        let d1 = d * sc1 as f32;
        let m1 = dmin * m1 as f32;
        let d2 = d * sc2 as f32;
        let m2 = dmin * m2 as f32;
        for l in 0..32 {
            out[oi] = d1 * (qs[q + l] & 0x0F) as f32 - m1;
            oi += 1;
        }
        for l in 0..32 {
            out[oi] = d2 * (qs[q + l] >> 4) as f32 - m2;
            oi += 1;
        }
        q += 32;
    }
}

fn get_scale_min_k4(j: usize, q: &[u8]) -> (u8, u8) {
    if j < 4 {
        (q[j] & 63, q[j + 4] & 63)
    } else {
        let d = (q[j + 4] & 0x0F) | ((q[j - 4] >> 6) << 4);
        let m = (q[j + 4] >> 4) | ((q[j] >> 6) << 4);
        (d, m)
    }
}

pub fn f16_to_f32(lo: u8, hi: u8) -> f32 {
    let h = ((hi as u16) << 8) | lo as u16;
    let sign = ((h >> 15) & 1) as u32;
    let exp = ((h >> 10) & 0x1F) as u32;
    let mant = (h & 0x3FF) as u32;
    let f = if exp == 0 {
        if mant == 0 {
            0.0
        } else {
            let m = mant as f32 / (1u32 << 10) as f32;
            m * 2f32.powi(-14)
        }
    } else if exp == 31 {
        if mant == 0 {
            f32::INFINITY
        } else {
            f32::NAN
        }
    } else {
        let m = 1.0 + mant as f32 / (1u32 << 10) as f32;
        m * 2f32.powi(exp as i32 - 15)
    };
    if sign == 1 {
        -f
    } else {
        f
    }
}

// ─── Tensor container: name → raw Q4_K bytes ───
pub struct VoltageModel {
    pub tensors: HashMap<String, Vec<u8>>, // raw bytes (Q4_K or f16/f32)
    pub f16_names: Vec<String>,            // names stored as f16 (norms)
}

impl VoltageModel {
    /// Dequantize a Q4_K tensor row-major [nrows, 256] → f32.
    /// rows must be a multiple of 256 (GGUF pads to block size).
    pub fn dequant_tensor(&self, name: &str) -> Option<Vec<f32>> {
        let raw = self.tensors.get(name)?;
        let nblocks = raw.len() / 144;
        let mut out = vec![0f32; nblocks * 256];
        for b in 0..nblocks {
            dequantize_q4k_block(&raw[b * 144..(b + 1) * 144], &mut out[b * 256..(b + 1) * 256]);
        }
        Some(out)
    }

    /// Dequantize a Q6_K tensor row-major [nrows, 256] → f32.
    pub fn dequant_q6k_tensor(&self, name: &str) -> Option<Vec<f32>> {
        let raw = self.tensors.get(name)?;
        let nblocks = raw.len() / 210;
        let mut out = vec![0f32; nblocks * 256];
        for b in 0..nblocks {
            dequantize_q6k_block(
                &raw[b * 210..(b + 1) * 210],
                &mut out[b * 256..(b + 1) * 256],
            );
        }
        Some(out)
    }

    /// Dequantize by ggml_type: 12=Q4_K, 14=Q6_K, 0=F32.
    pub fn dequant_typed(&self, name: &str, ggml_type: u32) -> Option<Vec<f32>> {
        match ggml_type {
            12 => self.dequant_tensor(name),
            14 => self.dequant_q6k_tensor(name),
            0 => {
                let raw = self.tensors.get(name)?;
                Some(
                    raw.chunks_exact(4)
                        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                        .collect(),
                )
            }
            _ => None,
        }
    }

    /// Read an f16 tensor (norms, etc.) → f32.
    pub fn f16_tensor(&self, name: &str) -> Option<Vec<f32>> {
        let raw = self.tensors.get(name)?;
        let n = raw.len() / 2;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            out.push(f16_to_f32(raw[i * 2], raw[i * 2 + 1]));
        }
        Some(out)
    }
}

/// Dequantize one Q6_K super-block (256 weights, 210 bytes) to f32.
/// From llama.cpp dequantize_row_q6_K: ql[128] + qh[64] + scales[16] + d(2).
pub fn dequantize_q6k_block(data: &[u8], out: &mut [f32]) {
    debug_assert!(data.len() >= 210);
    debug_assert!(out.len() >= 256);
    let ql = &data[0..128];
    let qh = &data[128..192];
    let sc = &data[192..208];
    let d = f16_to_f32(data[208], data[209]);

    for n in (0..256).step_by(128) {
        for l in 0..32 {
            let is = l / 16;
            let q1 = (((ql[l + 0] & 0xF) as i32) | (((qh[l] >> 0) & 3) as i32) << 4) - 32;
            let q2 = (((ql[l + 32] & 0xF) as i32) | (((qh[l] >> 2) & 3) as i32) << 4) - 32;
            let q3 = (((ql[l + 0] >> 4) as i32) | (((qh[l] >> 4) & 3) as i32) << 4) - 32;
            let q4 = (((ql[l + 32] >> 4) as i32) | (((qh[l] >> 6) & 3) as i32) << 4) - 32;
            let sc_off = (n / 128) * 8;
            out[n + l + 0] = d * sc[sc_off + is + 0] as f32 * q1 as f32;
            out[n + l + 32] = d * sc[sc_off + is + 2] as f32 * q2 as f32;
            out[n + l + 64] = d * sc[sc_off + is + 4] as f32 * q3 as f32;
            out[n + l + 96] = d * sc[sc_off + is + 6] as f32 * q4 as f32;
        }
    }
}

// ─── Core ops ───
pub fn rms_norm(x: &[f32], w: &[f32], eps: f32) -> Vec<f32> {
    let mut ss = 0f64;
    for &v in x {
        ss += (v as f64) * (v as f64);
    }
    let rms = (ss / x.len() as f64 + eps as f64).sqrt();
    x.iter().zip(w).map(|(&v, &g)| (v as f64 / rms) as f32 * g).collect()
}

/// Row-major matmul: out[i][j] = sum_k a[i][k] * b[k][j] — b stored [rows][cols].
pub fn matmul(a: &[f32], b: &[f32], m: usize, k: usize, n: usize) -> Vec<f32> {
    let mut out = vec![0f32; m * n];
    for i in 0..m {
        for j in 0..n {
            let mut s = 0f64;
            for kk in 0..k {
                s += a[i * k + kk] as f64 * b[kk * n + j] as f64;
            }
            out[i * n + j] = s as f32;
        }
    }
    out
}

/// RoPE (Qwen3: half-split rotary per head, freq_base 1e6).
/// Matches llama.cpp rotate_pairs NEOX: pair (x[i], x[i + head_dim/2]).
/// x: one token's full Q/K vector [heads * head_dim].
pub fn rope_qwen(x: &mut [f32], pos: usize, head_dim: usize, freq_base: f32) {
    let half = head_dim / 2;
    for (h, head) in x.chunks_exact_mut(head_dim).enumerate() {
        for i in 0..half {
            let theta = pos as f32 / freq_base.powf(2.0 * i as f32 / head_dim as f32);
            let (s, c) = theta.sin_cos();
            let a = head[i];
            let b = head[i + half];
            head[i] = a * c - b * s;
            head[i + half] = a * s + b * c;
        }
        let _ = h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rms_norm_known() {
        let x = vec![1.0, 2.0, 3.0];
        let w = vec![1.0, 1.0, 1.0];
        let out = rms_norm(&x, &w, 1e-6);
        let ss: f64 = x.iter().map(|v| (v * v) as f64).sum::<f64>() / 3.0;
        let rms = ss.sqrt();
        for i in 0..3 {
            assert!((out[i] - (x[i] as f64 / rms) as f32).abs() < 1e-5);
        }
    }

    #[test]
    fn matmul_small() {
        let a = vec![1.0, 2.0, 3.0, 4.0]; // 2x2
        let b = vec![1.0, 0.0, 0.0, 1.0]; // identity 2x2
        let out = matmul(&a, &b, 2, 2, 2);
        assert!((out[0] - 1.0).abs() < 1e-6 && (out[3] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn rope_zero_position_is_identity() {
        let mut x = vec![1.0, 0.0, 0.5, -0.5, 2.0, 1.0, -1.0, 0.25];
        let orig = x.clone();
        rope_qwen(&mut x, 0, 4, 1e6);
        for i in 0..8 {
            assert!((x[i] - orig[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn rope_half_split_pairing() {
        // head_dim=4, 1 head. pair = (x[0], x[2]) and (x[1], x[3])
        let mut x = vec![1.0, 0.0, 0.0, 1.0];
        let pos = 1;
        let hd = 4usize;
        let freq = 1e6f32;
        // reference: theta_0 = 1 / 1e6^(0) = 1 → cos1, sin1
        let t0 = 1.0f32 / freq.powf(0.0);
        let (s0, c0) = t0.sin_cos();
        let a = x[0];
        let b = x[2];
        let expect0 = a * c0 - b * s0;
        let expect2 = a * s0 + b * c0;
        rope_qwen(&mut x, pos, hd, freq);
        assert!((x[0] - expect0).abs() < 1e-5);
        assert!((x[2] - expect2).abs() < 1e-5);
    }
}
