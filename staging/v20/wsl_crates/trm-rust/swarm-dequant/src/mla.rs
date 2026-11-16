//! Instella MLA (multi-head latent attention) forward — faithful to the fork's absorbed MLA:
//! q_nope is absorbed into K via wk_b (so it runs as MQA), V is up-projected via wv_b,
//! and the attention output is gated by sigmoid(attn_gate @ x). Weight layout: ne[0]-major,
//! i.e. element (in=i, out=j) at flat index i + in_dim * j.

use crate::{rope, rms_norm, softmax};

pub const N_EMBD: usize = 2048;
pub const N_HEAD: usize = 16;
pub const NOPE: usize = 96; // n_embd_head_qk_nope
pub const ROPE_D: usize = 32; // n_embd_head_qk_rope
pub const KV_LORA: usize = 512;
pub const V_MLA: usize = 128; // n_embd_head_v_mla
pub const FREQ_BASE: f32 = 8_000_000.0;
// YaRN kq_scale pre-scaling, matching llama.cpp's instella-moe.cpp:130-134 exactly.
//   freq_scale = 1/factor = 1/40 = 0.025; rope_yarn_log_mul = 0.1/0.1 = 1.0; ln(1/freq_scale) = ln(40).
//   cparams.yarn_attn_factor is PRE-DIVIDED by (1+0.1*ln(40)) in llama-context.cpp, so it is
//   0.730524, and attn_factor_org = 0.730524 * (1+0.1*ln40) = 1.0.
//   mscale = attn_factor_org * (1+0.1*log_mul*ln40) = 1.368888
//   kq_scale = mscale^2 / sqrt(n_embd_head_k_mla) = 1.368888^2 / sqrt(128) = 1.873854/11.313708
//
// The previous 0.150547 came from assuming attn_factor = 1.0 (giving mscale = 1.87386) AND dividing
// by sqrt(attention.key_length = 544) instead of sqrt(key_length_mla = 128): one extra mscale factor
// and the wrong dimension. Measured 2026-11-01 after the Q5_0 and gating fixes: 0.150547 gives
// logit mean_abs 6.222573 with L26@pos1 rel_l2 0.039950; 0.165626 gives mean_abs 0.217550,
// max_abs 0.589027, cos 0.99999956 and L26@pos1 rel_l2 0.015508. The earlier rejection of this value
// was measured under the broken Q5_0 dequant and was contaminated by it.
// Note kq_scale is invisible at position 0 (softmax over a single key = 1.0 regardless of scale),
// which is why the pos0 trajectory matched perfectly while every position >= 1 carried the offset.
pub const KQ_SCALE: f32 = 0.165_626;

/// sigmoid(x) = 1 / (1 + e^-x) — the gated-attention gate.
pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// One layer's MLA attention, single query at `pos`, with a growing KV cache.
/// h is the post-attn_norm hidden [N_EMBD]. Returns the post-wo output [N_EMBD].
#[allow(clippy::too_many_arguments)]
pub fn mla_forward(
    h: &[f32],
    pos: usize,
    layer: usize,
    k_cache: &mut Vec<Vec<f32>>, // [t][KV_LORA + ROPE_D]
    v_cache: &mut Vec<Vec<f32>>, // [t][KV_LORA]
    wq: &[f32],     // [N_EMBD * N_EMBD]
    wkv_a: &[f32],  // [N_EMBD * (KV_LORA + ROPE_D)]
    kv_a_norm: &[f32], // [KV_LORA]
    wk_b: &[f32],   // [NOPE * KV_LORA * N_HEAD] (f16 -> f32)
    wv_b: &[f32],   // [KV_LORA * V_MLA * N_HEAD]
    wg: &[f32],     // [N_EMBD * N_EMBD] gate
    wo: &[f32],     // [N_EMBD * N_EMBD]
    q8: bool,       // quantize matmul inputs to Q8_K (match llama.cpp's fused dot)
) -> Vec<f32> {
    let hq: Vec<f32> = if q8 { crate::quantize_q8_k(h) } else { h.to_vec() };
    // 1. gate = wg @ h
    let mut gate = vec![0.0f32; N_EMBD];
    for j in 0..N_EMBD {
        gate[j] = crate::dot_fma(&hq, &wg[j * N_EMBD..j * N_EMBD + N_EMBD]);
    }
    crate::tap_write("attn_gate_proj", layer, pos, &gate);
    // 2. q = wq @ h, laid out [dim + 128*head]
    let mut q = vec![0.0f32; N_EMBD];
    for j in 0..N_EMBD {
        q[j] = crate::dot_fma(&hq, &wq[j * N_EMBD..j * N_EMBD + N_EMBD]);
    }
    crate::tap_write("q", layer, pos, &q);
    // 3. kv_cmpr_pe = wkv_a @ h -> [KV_LORA + ROPE_D]
    let mut kv = vec![0.0f32; KV_LORA + ROPE_D];
    for j in 0..KV_LORA + ROPE_D {
        kv[j] = crate::dot_fma(&hq, &wkv_a[j * N_EMBD..j * N_EMBD + N_EMBD]);
    }
    crate::tap_write("kv_cmpr_pe", layer, pos, &kv);
    // 4. RoPE: q_pe per head + k_pe. YARN_ROPE=1 (default) = llama.cpp's rope_yarn WITH the ramp
    // over corr_dims [3,7]; YARN_ROPE=0 keeps the old plain-rope form for A/B only. The ramp is
    // the measured fix: 16/16 greedy tokens with it, divergence at gen 6 without it. mscale is
    // 1.0 in the fork (its own log prints yarn_attn_factor = 1.0000), so the amplitude is identity.
    let yarn = std::env::var("YARN_ROPE").map(|v| v != "0").unwrap_or(true);
    let freq_scale: f32 = std::env::var("YARN_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(0.025);
    let mscale: f32 = std::env::var("YARN_MSCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(1.0);
    let rot32 = |src: &[f32]| -> Vec<f32> {
        if yarn {
            crate::yarn_rope(src, pos, FREQ_BASE, freq_scale, 4096.0, 32.0, 1.0, mscale)
        } else {
            crate::rope_scaled(src, pos, FREQ_BASE, freq_scale)
        }
    };
    let mut q_pe = vec![0.0f32; N_HEAD * ROPE_D];
    for hh in 0..N_HEAD {
        let src: Vec<f32> = (0..ROPE_D).map(|d| q[NOPE + d + (NOPE + ROPE_D) * hh]).collect();
        let rot = rot32(&src);
        for d in 0..ROPE_D {
            q_pe[d + ROPE_D * hh] = rot[d];
        }
    }
    let k_pe = rot32(&kv[KV_LORA..]);
    // 5. norm the compressed KV latent
    let kv_cmpr = rms_norm(&kv[..KV_LORA], kv_a_norm, 1e-6);
    // Names follow the fork's dump semantics: its "kv_cmpr"/"k_pe" taps are the raw projection views
    // (before rms_norm / before RoPE), so the raw split goes under those names. The processed values
    // get distinct crate names so a diff is never ambiguous.
    crate::tap_write("kv_cmpr_raw", layer, pos, &kv[..KV_LORA]);
    crate::tap_write("k_pe_raw", layer, pos, &kv[KV_LORA..]);
    crate::tap_write("kv_cmpr_norm", layer, pos, &kv_cmpr);
    crate::tap_write("k_pe_rope", layer, pos, &k_pe);
    // 6. absorption: q_absorbed[h][j] = sum_d wk_b[d + NOPE*j + NOPE*KV_LORA*h] * q_nope[d + NOPE*h]
    // wk_b (attn_k_b) is Q5_0 -> the q_nope input is Q8_1 (32-blk mean-offset over the [96,16] tensor)
    let mut q_nope = vec![0.0f32; N_HEAD * NOPE];
    for hh in 0..N_HEAD {
        for d in 0..NOPE {
            q_nope[d + NOPE * hh] = q[d + (NOPE + ROPE_D) * hh];
        }
    }
    let q_nope_q: Vec<f32> = if q8 { crate::quantize_q8_1(&q_nope) } else { q_nope };
    let mut q_absorbed = vec![0.0f32; N_HEAD * KV_LORA];
    for hh in 0..N_HEAD {
        for j in 0..KV_LORA {
            q_absorbed[j + KV_LORA * hh] = crate::dot_fma(
                &wk_b[NOPE * j + NOPE * KV_LORA * hh..NOPE * j + NOPE * KV_LORA * hh + NOPE],
                &q_nope_q[NOPE * hh..NOPE * hh + NOPE],
            );
        }
    }
    crate::tap_write("q_absorbed", layer, pos, &q_absorbed);
    // 7/8. Kcur/Vcur into cache
    // llama.cpp's attention runs through ggml_flash_attn_ext, which casts k and v from F32 to F16
    // before the fused kernel (build_attn_mha: "if (k->type == GGML_TYPE_F32) cast to F16", same for
    // v). Rounding those operands the same way is NOT cosmetic: with f32 operands the layer-0
    // attention output disagreed with the fork by 9.0e-3 at position 1; with the f16 rounding it is
    // 2.6e-3, i.e. 2.2x the fork's own FA-vs-nonFA spread of 1.2e-3. The score path is invisible at
    // position 0 (one-key softmax is 1.0), which is why this only ever showed up at position >= 1.
    let mut kcur = vec![0.0f32; KV_LORA + ROPE_D];
    kcur[..KV_LORA].copy_from_slice(&kv_cmpr);
    kcur[KV_LORA..].copy_from_slice(&k_pe);
    for x in kcur.iter_mut() { *x = crate::f16_round(*x); }
    let mut q_absorbed_f16 = q_absorbed.clone();
    for x in q_absorbed_f16.iter_mut() { *x = crate::f16_round(*x); }
    let mut q_pe_f16 = q_pe.clone();
    for x in q_pe_f16.iter_mut() { *x = crate::f16_round(*x); }
    k_cache.push(kcur);
    let mut vcur = kv_cmpr.clone();
    for x in vcur.iter_mut() { *x = crate::f16_round(*x); }
    v_cache.push(vcur);
    let n_t = k_cache.len();
    // 9. per-head attention over the cache
    let mut attn_v = vec![0.0f32; N_HEAD * KV_LORA];
    for hh in 0..N_HEAD {
        let mut scores = vec![0.0f32; n_t];
        for (t, kc) in k_cache.iter().enumerate() {
            let s = crate::dot_fma(&q_absorbed_f16[KV_LORA * hh..KV_LORA * hh + KV_LORA], &kc[..KV_LORA])
                + crate::dot_fma(&q_pe_f16[ROPE_D * hh..ROPE_D * hh + ROPE_D], &kc[KV_LORA..]);
            scores[t] = s * KQ_SCALE;
        }
        let wts = softmax(&scores);
        for d in 0..KV_LORA {
            let mut acc = 0.0;
            for (t, vc) in v_cache.iter().enumerate() {
                acc += wts[t] * vc[d];
            }
            attn_v[d + KV_LORA * hh] = acc;
        }
    }
    crate::tap_write("attn_v", layer, pos, &attn_v);
    // 10. V up-projection: attn_out[h][j] = sum_d wv_b[d + KV_LORA*j + KV_LORA*V_MLA*h] * attn_v
    // wv_b (attn_v_b) is Q4_K -> the attn_v input is Q8_K (256-blk)
    let attn_v_q: Vec<f32> = if q8 { crate::quantize_q8_k(&attn_v) } else { attn_v.clone() };
    let mut attn_out = vec![0.0f32; N_EMBD];
    for hh in 0..N_HEAD {
        for j in 0..V_MLA {
            attn_out[j + V_MLA * hh] = crate::dot_fma(
                &wv_b[KV_LORA * j + KV_LORA * V_MLA * hh..KV_LORA * j + KV_LORA * V_MLA * hh + KV_LORA],
                &attn_v_q[KV_LORA * hh..KV_LORA * hh + KV_LORA],
            );
        }
    }
    // Fork naming: "attn_out" is the attention output BEFORE the gate multiply, "attn_gated" after
    // it, "attn_o_proj" after wo (instella-moe.cpp 262/317/326).
    crate::tap_write("attn_out", layer, pos, &attn_out);
    // 11. gating
    for i in 0..N_EMBD {
        attn_out[i] *= sigmoid(gate[i]);
    }
    // 12. wo @ attn_out
    let attn_q8: Vec<f32> = if q8 { crate::quantize_q8_k(&attn_out) } else { attn_out.clone() };
    let mut out = vec![0.0f32; N_EMBD];
    for j in 0..N_EMBD {
        out[j] = crate::dot_fma(&attn_q8, &wo[j * N_EMBD..j * N_EMBD + N_EMBD]);
    }
    crate::tap_write("attn_gated", layer, pos, &attn_out);
    crate::tap_write("attn_o_proj", layer, pos, &out);
    out
}
