//! swarm-voltage — Build #1: qwen3moe forward pass — the full model.
//!
//! Per block (from llama.cpp's qwen3moe arch + measured tensors):
//!   1. attn_norm (RMSNorm, eps 1e-6) on each position
//!   2. Q = x @ attn_q [1024,2048]  (16 heads × 128)
//!      K = x @ attn_k [1024,1024]  (8 KV heads × 128)
//!      V = x @ attn_v [1024,1024]
//!   3. QK-norm: q_norm [128] per q head, k_norm [128] per kv head
//!   4. RoPE on Q and K (half-split, freq_base 1e6)
//!   5. causal attention (GQA: 16 Q heads share 8 KV heads), scale 1/sqrt(128)
//!   6. out = attn_out @ attn_output [2048,1024] → + residual
//!   7. ffn_norm (RMSNorm)
//!   8. MoE: gate_inp [1024,4] → softmax → top-2 experts
//!      per expert: silu(x@gate_exps) * (x@up_exps) @ down_exps [3072]
//!      weighted sum (top-2 weights) → + residual
//! Final: output_norm → hidden state (NO softmax — the voltage field).

use crate::{matmul, rms_norm, rope_qwen, VoltageModel};

pub const DIM: usize = 1024;
pub const N_LAYERS: usize = 28;
pub const N_HEADS: usize = 16;
pub const N_KV_HEADS: usize = 8;
pub const HEAD_DIM: usize = 128;
pub const N_EXPERTS: usize = 4;
pub const N_EXPERT_USED: usize = 2;
pub const FFN_EXP: usize = 3072;
pub const EPS: f32 = 1e-6;
pub const FREQ_BASE: f32 = 1e6;

fn silu(x: f32) -> f32 {
    x / (1.0 + (-x).exp())
}

/// Run one transformer block over a sequence of hidden states.
/// hs: [seq * DIM]. Returns updated [seq * DIM].
pub fn block(
    hs: &[f32],
    model: &VoltageModel,
    attn_norm_w: &[f32],
    q_w: &[f32],
    k_w: &[f32],
    v_w: &[f32],
    o_w: &[f32],
    q_norm_w: &[f32],
    k_norm_w: &[f32],
    ffn_norm_w: &[f32],
    gate_inp_w: &[f32],
    gate_exps_w: &[f32],
    up_exps_w: &[f32],
    down_exps_w: &[f32],
) -> Vec<f32> {
    let seq = hs.len() / DIM;

    // ── attention ──
    // RMSNorm each position, then QKV
    let mut hs_norm = vec![0f32; hs.len()];
    for s in 0..seq {
        let hh = rms_norm(&hs[s * DIM..(s + 1) * DIM], attn_norm_w, EPS);
        hs_norm[s * DIM..(s + 1) * DIM].copy_from_slice(&hh);
    }
    let q_all = matmul(&hs_norm, q_w, seq, DIM, N_HEADS * HEAD_DIM);
    let k_all = matmul(&hs_norm, k_w, seq, DIM, N_KV_HEADS * HEAD_DIM);
    let v_all = matmul(&hs_norm, v_w, seq, DIM, N_KV_HEADS * HEAD_DIM);

    // QK-norm + RoPE per position
    let mut qn_all = q_all.clone();
    let mut kn_all = k_all.clone();
    for s in 0..seq {
        let qoff = s * N_HEADS * HEAD_DIM;
        let koff = s * N_KV_HEADS * HEAD_DIM;
        for hh in 0..N_HEADS {
            let off = qoff + hh * HEAD_DIM;
            let qh = rms_norm(&qn_all[off..off + HEAD_DIM], &q_norm_w[..HEAD_DIM], EPS);
            qn_all[off..off + HEAD_DIM].copy_from_slice(&qh);
        }
        for hh in 0..N_KV_HEADS {
            let off = koff + hh * HEAD_DIM;
            let kh = rms_norm(&kn_all[off..off + HEAD_DIM], &k_norm_w[..HEAD_DIM], EPS);
            kn_all[off..off + HEAD_DIM].copy_from_slice(&kh);
        }
        rope_qwen(&mut qn_all[qoff..qoff + N_HEADS * HEAD_DIM], s, HEAD_DIM, FREQ_BASE);
        rope_qwen(&mut kn_all[koff..koff + N_KV_HEADS * HEAD_DIM], s, HEAD_DIM, FREQ_BASE);
    }

    // causal attention, GQA
    let scale = 1.0 / (HEAD_DIM as f32).sqrt();
    let mut attn_residual = vec![0f32; seq * DIM];
    for s in 0..seq {
        let mut attn_out = vec![0f32; N_HEADS * HEAD_DIM];
        for hh in 0..N_HEADS {
            let kv = hh % N_KV_HEADS;
            let q_off = s * (N_HEADS * HEAD_DIM) + hh * HEAD_DIM;
            let mut scores = vec![0f32; s + 1];
            for t in 0..=s {
                let k_off = t * (N_KV_HEADS * HEAD_DIM) + kv * HEAD_DIM;
                let mut acc = 0f64;
                for i in 0..HEAD_DIM {
                    acc += qn_all[q_off + i] as f64 * kn_all[k_off + i] as f64;
                }
                scores[t] = (acc as f32 * scale).exp();
            }
            let sum: f32 = scores.iter().sum();
            for t in 0..=s {
                let w = scores[t] / sum;
                let v_off = t * (N_KV_HEADS * HEAD_DIM) + kv * HEAD_DIM;
                for i in 0..HEAD_DIM {
                    attn_out[hh * HEAD_DIM + i] += w * v_all[v_off + i];
                }
            }
        }
        let o = matmul(&attn_out, o_w, 1, N_HEADS * HEAD_DIM, DIM);
        for i in 0..DIM {
            attn_residual[s * DIM + i] = hs[s * DIM + i] + o[i];
        }
    }

    // ── MoE FFN per position ──
    let mut out = vec![0f32; seq * DIM];
    for s in 0..seq {
        let h = &attn_residual[s * DIM..(s + 1) * DIM];
        let hn = rms_norm(h, ffn_norm_w, EPS);
        let logits = matmul(&hn, gate_inp_w, 1, DIM, N_EXPERTS);
        let max_l = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = logits.iter().map(|l| (l - max_l).exp()).collect();
        let sum: f32 = exps.iter().sum();
        let probs: Vec<f32> = exps.iter().map(|e| e / sum).collect();
        let mut idx: Vec<usize> = (0..N_EXPERTS).collect();
        idx.sort_by(|&a, &b| probs[b].partial_cmp(&probs[a]).unwrap());

        // norm_w=true (qwen3moe): renormalize the selected experts' weights
        // by their sum (llama.cpp build_moe_ffn, norm_w branch)
        let mut weights: Vec<f32> = idx.iter().take(N_EXPERT_USED).map(|&e| probs[e]).collect();
        let w_sum: f32 = weights.iter().sum();
        for w in weights.iter_mut() {
            *w /= w_sum;
        }

        let mut ffn_out = vec![0f32; DIM];
        for (ei, &e) in idx.iter().take(N_EXPERT_USED).enumerate() {
            let g = expert_slice(gate_exps_w, e, DIM, FFN_EXP, N_EXPERTS);
            let u = expert_slice(up_exps_w, e, DIM, FFN_EXP, N_EXPERTS);
            let d = expert_slice(down_exps_w, e, FFN_EXP, DIM, N_EXPERTS);
            let gate = matmul(&hn, &g, 1, DIM, FFN_EXP);
            let up = matmul(&hn, &u, 1, DIM, FFN_EXP);
            let mut mid = vec![0f32; FFN_EXP];
            for i in 0..FFN_EXP {
                mid[i] = silu(gate[i]) * up[i];
            }
            let out_e = matmul(&mid, &d, 1, FFN_EXP, DIM);
            let w_e = weights[ei];
            for i in 0..DIM {
                ffn_out[i] += w_e * out_e[i];
            }
        }
        for i in 0..DIM {
            out[s * DIM + i] = h[i] + ffn_out[i];
        }
    }
    out
}

/// Extract expert e from a [R, C, NE] tensor (last dim = expert).
fn expert_slice(w: &[f32], e: usize, r: usize, c: usize, ne: usize) -> Vec<f32> {
    let mut out = vec![0f32; r * c];
    for i in 0..r {
        for j in 0..c {
            out[i * c + j] = w[(i * c + j) * ne + e];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silu_values() {
        assert!((silu(0.0) - 0.0).abs() < 1e-6);
        assert!((silu(1.0) - 0.7310586).abs() < 1e-5);
    }

    #[test]
    fn expert_slice_layout() {
        let r = 2;
        let c = 3;
        let ne = 2;
        let mut w = vec![0f32; r * c * ne];
        for i in 0..r {
            for j in 0..c {
                for e in 0..ne {
                    w[(i * c + j) * ne + e] = (i * 100 + j * 10 + e) as f32;
                }
            }
        }
        let e0 = expert_slice(&w, 0, r, c, ne);
        assert!((e0[0] - 0.0).abs() < 1e-6);
        assert!((e0[1] - 10.0).abs() < 1e-6);
        assert!((e0[2] - 20.0).abs() < 1e-6);
        assert!((e0[3] - 100.0).abs() < 1e-6);
        let e1 = expert_slice(&w, 1, r, c, ne);
        assert!((e1[1] - 11.0).abs() < 1e-6);
    }
}

#[cfg(test)]
mod q6k_tests {
    use super::*;
    use crate::dequantize_q6k_block;

    #[test]
    fn dequant_q6k_synthetic() {
        // 210-byte block: d=1.0, scales=1, ql/qh encode known quants
        let mut data = [0u8; 210];
        data[208..210].copy_from_slice(&[0x00, 0x3C]); // d = 1.0
        for s in 192..208 {
            data[s] = 1; // scales = 1
        }
        // q1 for l=0: ql[0]&0xF=1, qh[0]&3=0 → 1-32 = -31
        data[0] = 0x01;
        let mut out = [0f32; 256];
        dequantize_q6k_block(&data, &mut out);
        // out[0] = d * sc[0] * q1 = 1 * 1 * (-31)
        assert!((out[0] - (-31.0)).abs() < 1e-4);
        // q2 for l=0: ql[32]&0xF=0, qh[0]>>2&3=0 → -32
        assert!((out[32] - (-32.0)).abs() < 1e-4);
    }
}
