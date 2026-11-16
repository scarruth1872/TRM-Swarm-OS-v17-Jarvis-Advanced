//! Mamba-3B selective-SSM forward pass over the ternary `.trm` bundle + fp32 passthrough.
//!
//! Pure Mamba (no attention, no FFN — GGUF metadata: head_count 0, feed_forward_length 0).
//! 64 blocks of: RMSNorm -> ssm_in [x,z] -> causal conv1d -> ssm_x [dt,B,C] ->
//! ssm_dt+softplus -> A=-exp(ssm_a) -> selective scan -> ssm_out -> y*silu(z)+residual.
//! Tied embedding: logits = output_norm(h) . token_embd.

use serde_json::{json, Value};
use std::collections::HashMap;

const D_MODEL: usize = 2560;
const D_INNER: usize = 5120;
const D_STATE: usize = 16;
const DT_RANK: usize = 160;
const CONV_KERNEL: usize = 4;
const N_BLOCKS: usize = 64;
const VOCAB: usize = 50280;
const EPS: f32 = 1e-5;

struct Ternary {
    rows: usize,
    cols: usize,
    words: Vec<u32>,
    scale: f32,
}

pub struct Mamba {
    ternary: HashMap<String, Ternary>,
    fp: HashMap<String, Vec<f32>>,
}

fn f16_to_f32(h: u16) -> f32 {
    let sign = ((h >> 15) & 1) as f32;
    let exp = ((h >> 10) & 0x1F) as i32;
    let mant = (h & 0x3FF) as f32;
    let v = if exp == 0 {
        (mant / 1024.0) * 2f32.powi(-14) // subnormal
    } else if exp == 31 {
        if mant == 0.0 { f32::INFINITY } else { f32::NAN }
    } else {
        (mant / 1024.0 + 1.0) * 2f32.powi(exp - 15)
    };
    if sign > 0.0 { -v } else { v }
}

impl Mamba {
    pub fn load(name: &str) -> Result<Self, String> {
        let b = crate::ternary_engine::load_verified(name)?;
        let mut ternary = HashMap::new();
        let mut fp = HashMap::new();

        if let Some(arr) = b.header.get("tensors").and_then(|t| t.as_array()) {
            for t in arr {
                let n = t.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                let rows = t.get("rows").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let cols = t.get("cols").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let nbytes = t.get("nbytes").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let offset = t.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let start = b.data_start + offset;
                let words: Vec<u32> = b.raw[start..start + nbytes]
                    .chunks_exact(4)
                    .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                let scale = b.header.get("scales")
                    .and_then(|s| s.get(&n)).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                ternary.insert(n, Ternary { rows, cols, words, scale });
            }
        }
        if let Some(arr) = b.header.get("passthrough").and_then(|t| t.as_array()) {
            for t in arr {
                let n = t.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                let dtype = t.get("dtype").and_then(|n| n.as_str()).unwrap_or("float32");
                let nbytes = t.get("nbytes").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let offset = t.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let start = b.data_start + offset;
                let bytes = &b.raw[start..start + nbytes];
                let vals: Vec<f32> = if dtype == "float16" {
                    bytes.chunks_exact(2)
                        .map(|c| f16_to_f32(u16::from_le_bytes([c[0], c[1]]))).collect()
                } else {
                    bytes.chunks_exact(4)
                        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
                };
                fp.insert(n, vals);
            }
        }
        Ok(Self { ternary, fp })
    }

    fn gemm_t(&self, name: &str, x: &[f32], rows: usize, cols: usize) -> Vec<f32> {
        self.matmul(name, x, rows, cols)
    }

    /// Matrix-vector multiply over a tensor that may be ternary (packed 2-bit) OR fp32/f16
    /// fallback (below the K3 gate). The gate is per-tensor, so a given weight matrix can be
    /// ternary in some blocks and fp in others.
    fn matmul(&self, name: &str, x: &[f32], rows: usize, cols: usize) -> Vec<f32> {
        if let Some(t) = self.ternary.get(name) {
            let mut y = vec![0f32; rows];
            crate::ternary_engine::gemm(&t.words, rows, cols, x, &mut y);
            for v in &mut y { *v *= t.scale; }
            y
        } else if let Some(w) = self.fp.get(name) {
            let mut y = vec![0f32; rows];
            for r in 0..rows {
                let mut acc = 0f32;
                for c in 0..cols { acc += w[r * cols + c] * x[c]; }
                y[r] = acc;
            }
            y
        } else {
            panic!("tensor {name} not found in ternary or fp store");
        }
    }

    /// Single weight element at (row, col), from either the ternary or fp store.
    fn weight_at(&self, name: &str, row: usize, col: usize, cols: usize) -> f32 {
        if let Some(t) = self.ternary.get(name) {
            let wpr = (cols + 15) / 16;
            trit_at(&t.words, wpr, row, col) * t.scale
        } else if let Some(w) = self.fp.get(name) {
            w[row * cols + col]
        } else {
            0.0
        }
    }

    fn block(&self, b: usize, h: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let t_len = h.len();
        let norm = &self.fp[&format!("blk.{b}.attn_norm.weight")];
        let conv_b = &self.fp[&format!("blk.{b}.ssm_conv1d.bias")];
        let dt_b = &self.fp[&format!("blk.{b}.ssm_dt.bias")];
        let ssm_d = &self.fp[&format!("blk.{b}.ssm_d")];

        // RMSNorm + in_proj -> [x, z] per position
        let mut xs = vec![vec![0f32; D_INNER]; t_len];
        let mut zs = vec![vec![0f32; D_INNER]; t_len];
        for t in 0..t_len {
            let mut xn = h[t].clone();
            rms_norm(&mut xn, norm);
            let xz = self.matmul(&format!("blk.{b}.ssm_in.weight"), &xn, 2 * D_INNER, D_MODEL);
            xs[t].copy_from_slice(&xz[..D_INNER]);
            zs[t].copy_from_slice(&xz[D_INNER..]);
        }

        // causal conv1d (kernel 4) + silu, over the sequence
        let mut xc = vec![vec![0f32; D_INNER]; t_len];
        for t in 0..t_len {
            for c in 0..D_INNER {
                let mut acc = conv_b[c];
                for k in 0..CONV_KERNEL {
                    let tt = t as isize - (CONV_KERNEL as isize - 1 - k as isize);
                    if tt >= 0 {
                        acc += self.weight_at(&format!("blk.{b}.ssm_conv1d.weight"), c, k, CONV_KERNEL)
                            * xs[tt as usize][c];
                    }
                }
                xc[t][c] = silu(acc);
            }
        }

        // A is stored directly (already -exp(A_log), all negative) — no transform.
        let mut a_mat = vec![0f32; D_INNER * D_STATE];
        for c in 0..D_INNER {
            for s in 0..D_STATE {
                a_mat[c * D_STATE + s] = self.weight_at(&format!("blk.{b}.ssm_a"), c, s, D_STATE);
            }
        }

        // selective scan
        let mut state = vec![0f32; D_INNER * D_STATE];
        let mut ys = vec![vec![0f32; D_INNER]; t_len];
        for t in 0..t_len {
            let x192 = self.matmul(&format!("blk.{b}.ssm_x.weight"), &xc[t], DT_RANK + 2 * D_STATE, D_INNER);
            let dt = self.matmul(&format!("blk.{b}.ssm_dt.weight"), &x192[..DT_RANK], D_INNER, DT_RANK);
            let bv = &x192[DT_RANK..DT_RANK + D_STATE];
            let cv = &x192[DT_RANK + D_STATE..];
            for c in 0..D_INNER {
                let dtc = softplus(dt[c] + dt_b[c]);
                let mut acc = 0f32;
                for s in 0..D_STATE {
                    let idx = c * D_STATE + s;
                    let a_bar = (dtc * a_mat[idx]).exp();
                    state[idx] = a_bar * state[idx] + dtc * bv[s] * xc[t][c];
                    acc += state[idx] * cv[s];
                }
                ys[t][c] = acc + ssm_d[c] * xc[t][c];
            }
        }

        // gate y * z, then out_proj -> d_model, + residual
        let mut out = vec![vec![0f32; D_MODEL]; t_len];
        for t in 0..t_len {
            let mut yz = vec![0f32; D_INNER];
            for c in 0..D_INNER { yz[c] = ys[t][c] * silu(zs[t][c]); }
            let o = self.matmul(&format!("blk.{b}.ssm_out.weight"), &yz, D_MODEL, D_INNER);
            for i in 0..D_MODEL { out[t][i] = o[i] + h[t][i]; }
        }
        out
    }

    pub fn forward(&self, tokens: &[u32]) -> Result<Vec<f32>, String> {
        if tokens.is_empty() {
            return Err("empty token sequence".to_string());
        }
        let emb = &self.ternary["token_embd.weight"]; // [VOCAB, D_MODEL]
        let emb_wpr = (D_MODEL + 15) / 16;
        let mut h: Vec<Vec<f32>> = Vec::with_capacity(tokens.len());
        for &tok in tokens {
            if tok as usize >= VOCAB {
                return Err(format!("token {tok} >= vocab {VOCAB}"));
            }
            let mut v = vec![0f32; D_MODEL];
            for i in 0..D_MODEL {
                v[i] = trit_at(&emb.words, emb_wpr, tok as usize, i) * emb.scale;
            }
            h.push(v);
        }
        for b in 0..N_BLOCKS {
            h = self.block(b, &h);
        }
        // output_norm + tied embedding -> logits [VOCAB]
        let on = &self.fp["output_norm.weight"];
        let mut hn = h.last().unwrap().clone();
        rms_norm(&mut hn, on);
        let mut logits = vec![0f32; VOCAB];
        crate::ternary_engine::gemm(&emb.words, VOCAB, D_MODEL, &hn, &mut logits);
        for v in &mut logits { *v *= emb.scale; }
        Ok(logits)
    }
}

fn trit_at(words: &[u32], wpr: usize, row: usize, col: usize) -> f32 {
    let word = words[row * wpr + col / 16];
    let code = ((word >> (2 * (col % 16))) & 3) as u8;
    match code {
        0 => -1.0,
        3 => 1.0,
        _ => 0.0,
    }
}

fn rms_norm(x: &mut [f32], w: &[f32]) {
    let n = x.len();
    let ms = x.iter().map(|v| v * v).sum::<f32>() / n as f32;
    let inv = 1.0 / (ms + EPS).sqrt();
    for i in 0..n { x[i] = x[i] * inv * w[i]; }
}

fn silu(x: f32) -> f32 {
    x / (1.0 + (-x).exp())
}

fn softplus(x: f32) -> f32 {
    (1.0 + x.exp()).ln()
}

fn matvec(w: &[f32], x: &[f32], out: &mut [f32]) {
    let cols = x.len();
    let rows = out.len();
    for r in 0..rows {
        let mut acc = 0f32;
        for c in 0..cols {
            acc += w[r * cols + c] * x[c];
        }
        out[r] = acc;
    }
}
