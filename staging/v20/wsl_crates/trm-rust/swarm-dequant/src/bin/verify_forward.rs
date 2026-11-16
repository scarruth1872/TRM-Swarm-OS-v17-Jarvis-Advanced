//! Full Instella forward: token embedding -> 27 FarSkip layers (MLA attention + sparse MoE +
//! shared experts) -> output norm -> lm_head. Weights dequantized on the fly; only the top-6
//! active experts are dequantized per MoE layer. Prints the logits (softmax over the lm_head)
//! for a fixed prompt, for parity against llama.cpp.

use std::io::{Read, Seek, SeekFrom};

use swarm_dequant::mla::{mla_forward, KV_LORA, N_EMBD, N_HEAD, NOPE, V_MLA};
use swarm_dequant::{dequant_q4_k, dequant_q5_0, dequant_q6_k, dequant_q8_0, f16_to_f32, moe_router, rms_norm, silu, softmax};

const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
const N_LAYER: usize = 27;
const N_VOCAB: usize = 128896;
const EXPERT_DIM: usize = 1408;
const N_EXPERT: usize = 64;
const N_USED: usize = 6;
const SHEXP_DIM: usize = 2816; // 2 * 1408 shared experts
const EXPERT_SCALE: f32 = 2.5;
const USE_Q8: bool = true; // Path A: match llama.cpp's per-weight-type input quant; false = exact f32 (Path B)

struct Model {
    m: swarm_gguf::GgufModel,
    f: std::fs::File,
}

impl Model {
    fn tensor(&self, name: &str) -> &swarm_gguf::TensorInfo {
        self.m.tensors.iter().find(|t| t.name == name).expect(name)
    }
    fn type_of(&self, name: &str) -> u32 {
        self.tensor(name).ggml_type
    }
    fn read(&mut self, t: &swarm_gguf::TensorInfo, n_vals: usize, byte_off: usize) -> Vec<u8> {
        let mut buf = vec![0u8; byte_off];
        self.f.seek(SeekFrom::Start(self.m.data_start + t.offset)).unwrap();
        self.f.read_exact(&mut buf).unwrap();
        buf
    }
    fn bytes_for(t: &swarm_gguf::TensorInfo, n_vals: usize) -> usize {
        match t.ggml_type {
            12 => n_vals / 256 * 144,
            8 => n_vals / 32 * 34,
            6 => n_vals / 32 * 22,
            14 => n_vals / 256 * 210,
            1 => n_vals * 2,
            _ => n_vals * 4,
        }
    }
    fn load(&mut self, name: &str) -> Vec<f32> {
        let t = self.tensor(name).clone();
        let n = t.dims.iter().product::<u64>() as usize;
        let b = Self::bytes_for(&t, n);
        let buf = self.read(&t, n, b);
        match t.ggml_type {
            12 => dequant_q4_k(&buf, n),
            8 => dequant_q8_0(&buf, n),
            6 => dequant_q5_0(&buf, n),
            14 => dequant_q6_k(&buf, n),
            1 => buf.chunks_exact(2).map(|c| f16_to_f32(u16::from_le_bytes([c[0], c[1]]))).collect(),
            _ => buf.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect(),
        }
    }
    /// Load the RAW quantized bytes (no dequant) — for the int8 dot path.
    fn load_raw(&mut self, name: &str) -> Vec<u8> {
        let t = self.tensor(name).clone();
        let n = t.dims.iter().product::<u64>() as usize;
        let b = Self::bytes_for(&t, n);
        self.read(&t, n, b)
    }
    /// Dequantize expert `e` of a [in, out, 64] tensor, returning its [out * in] flat weights.
    fn load_expert(&mut self, name: &str, e: usize, in_dim: usize, out_dim: usize) -> Vec<f32> {
        let t = self.tensor(name).clone();
        let n_per = in_dim * out_dim;
        let bytes_per = Self::bytes_for(&t, n_per);
        let abs = self.m.data_start + t.offset + (e * bytes_per) as u64;
        let mut buf = vec![0u8; bytes_per];
        self.f.seek(SeekFrom::Start(abs)).unwrap();
        self.f.read_exact(&mut buf).unwrap();
        match t.ggml_type {
            12 => dequant_q4_k(&buf, n_per),
            8 => dequant_q8_0(&buf, n_per),
            6 => dequant_q5_0(&buf, n_per),
            14 => dequant_q6_k(&buf, n_per),
            _ => buf.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect(),
        }
    }
    fn embedding(&mut self, token: usize) -> Vec<f32> {
        let t = self.tensor("token_embd.weight").clone();
        // token_embd [2048, 128896] Q4_K: each token = 2048 values = 8 Q4_K blocks = 1152 bytes,
        // aligned to 256-value blocks (2048 = 8 * 256).
        let abs = self.m.data_start + t.offset + (token as u64 * 1152);
        let mut buf = vec![0u8; 1152];
        self.f.seek(SeekFrom::Start(abs)).unwrap();
        self.f.read_exact(&mut buf).unwrap();
        dequant_q4_k(&buf, N_EMBD)
    }
}

fn dense_ffn(h: &[f32], w_gate_raw: &[u8], w_up_raw: &[u8], w_down: &[f32], hidden: usize, ffn_dim: usize, q8: bool, down_type: u32) -> Vec<f32> {
    let hq: Vec<f32> = if q8 { swarm_dequant::quantize_q8_k(h) } else { h.to_vec() };
    let bytes_per_col = hidden / 256 * 144; // Q4_K: 256 values = 144 bytes
    let mut act = vec![0.0f32; ffn_dim];
    for j in 0..ffn_dim {
        let g = swarm_dequant::dot_q4_K_q8_K(&w_gate_raw[j * bytes_per_col..(j + 1) * bytes_per_col], h);
        let u = swarm_dequant::dot_q4_K_q8_K(&w_up_raw[j * bytes_per_col..(j + 1) * bytes_per_col], h);
        act[j] = silu(g) * u;
    }
    let actq: Vec<f32> = if q8 { swarm_dequant::q_input(&act, down_type) } else { act.clone() };
    let mut out = vec![0.0f32; hidden];
    for j in 0..hidden {
        out[j] = swarm_dequant::dot_fma(&actq, &w_down[j * ffn_dim..j * ffn_dim + ffn_dim]);
    }
    out
}

fn forward_token(
    model: &mut Model,
    k_cache: &mut Vec<Vec<Vec<f32>>>,
    v_cache: &mut Vec<Vec<Vec<f32>>>,
    tok: u32,
    tpos: usize,
) -> Vec<f32> {
    let mut res_nort = model.embedding(tok as usize);
    let mut res_full = res_nort.clone();
    for il in 0..N_LAYER {
        let p = |s: &str| format!("blk.{il}.{s}");
        let wq = model.load(&p("attn_q.weight"));
        let wkv_a = model.load(&p("attn_kv_a_mqa.weight"));
        let wk_b = model.load(&p("attn_k_b.weight"));
        let wv_b = model.load(&p("attn_v_b.weight"));
        let wg = model.load(&p("attn_gate.weight"));
        let wo = model.load(&p("attn_output.weight"));
        let kv_a_norm = model.load(&p("attn_kv_a_norm.weight"));
        let attn_norm = model.load(&p("attn_norm.weight"));
        let ffn_norm = model.load(&p("ffn_norm.weight"));

        let h_attn = rms_norm(&res_nort, &attn_norm, 1e-6);
        let attn = mla_forward(&h_attn, tpos, il, &mut k_cache[il], &mut v_cache[il], &wq, &wkv_a, &kv_a_norm, &wk_b, &wv_b, &wg, &wo, USE_Q8);
        let attn_out: Vec<f32> = attn.iter().zip(&res_full).map(|(a, r)| a + r).collect();
        // Fork name "attn_res" = attention output added to the pre-attention residual (instella-moe.cpp
        // 320). Tapping it here makes the whole layer ladder reproducible from crate code.
        swarm_dequant::tap_write("attn_res", il, tpos, &attn_out);

        let h_ffn = rms_norm(&res_full, &ffn_norm, 1e-6);
        if il == 0 {
            // blk.0.ffn_down.weight is Q8_0 in this GGUF; read the type instead of assuming it.
            let dense_dt = model.type_of("blk.0.ffn_down.weight");
            let ffn = dense_ffn(&h_ffn, &model.load_raw("blk.0.ffn_gate.weight"), &model.load_raw("blk.0.ffn_up.weight"), &model.load("blk.0.ffn_down.weight"), N_EMBD, 10944, USE_Q8, dense_dt);
            swarm_dequant::tap_write("ffn_out", il, tpos, &ffn);
            res_full = attn_out.iter().zip(&ffn).map(|(a, f)| a + f).collect();
            res_nort = res_full.clone();
        } else {
            let gate_inp = model.load(&p("ffn_gate_inp.weight"));
            let bias = model.load(&p("exp_probs_b.bias"));
            let (top, weights) = moe_router(&h_ffn, &gate_inp, &bias, N_EXPERT, N_USED, EXPERT_SCALE, false);
            // Router dump for near-tie diagnosis. A top-k boundary flip changes moe_out by O(0.3)
            // while leaving the norm intact, which is indistinguishable from a real bug unless the
            // chosen expert set is printed. Set DUMP_ROUTER=1 (with optional _LAYER/_POS).
            if std::env::var("DUMP_ROUTER").is_ok() {
                let wl: usize = std::env::var("DUMP_ROUTER_LAYER").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
                let wp: usize = std::env::var("DUMP_ROUTER_POS").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
                if il == wl && tpos == wp {
                    eprintln!("ROUTER il={il} pos={tpos} top={top:?} weights={weights:?}");
                    // Full 64-way gating probs: the only like-for-like comparison against the fork's
                    // ffn_moe_probs dump (which is pre-norm, pre-scale raw sigmoid gates).
                    let (_, _, probs) = swarm_dequant::moe_router_probs(&h_ffn, &gate_inp, &bias, N_EXPERT, N_USED, EXPERT_SCALE, false);
                    let mut line = format!("probs-{il}-{tpos} n={} bias", probs.len());
                    for b in &bias { line.push_str(&format!(" {b:.9}")); }
                    line.push('\n');
                    for p in &probs { line.push_str(&format!("{p:.9}\n")); }
                    let _ = std::fs::write("logs/router_probs.txt", line);
                    eprintln!("wrote logs/router_probs.txt ({} probs)", probs.len());
                }
            }
            let mut moe = vec![0.0f32; N_EMBD];
            let down_type = model.type_of(&p("ffn_down_exps.weight"));
            for (k, &e) in top.iter().enumerate() {
                let wg_e = model.load_expert(&p("ffn_gate_exps.weight"), e, N_EMBD, EXPERT_DIM);
                let wu_e = model.load_expert(&p("ffn_up_exps.weight"), e, N_EMBD, EXPERT_DIM);
                let wd_e = model.load_expert(&p("ffn_down_exps.weight"), e, EXPERT_DIM, N_EMBD);
                let out = swarm_dequant::expert_ffn(&h_ffn, &wg_e, &wu_e, &wd_e, N_EMBD, EXPERT_DIM, USE_Q8, down_type);
                for j in 0..N_EMBD { moe[j] += weights[k] * out[j]; }
            }
            // ffn_down_shexp is NOT one type across layers: Q6_K on 12 layers and Q4_K on 14 of
            // them (layer 3 is Q4_K). A hardcoded 14 quantized the activation as Q6_K where
            // llama.cpp uses Q4_K's Q8_K vec_dot input, so read the real per-layer type.
            let shexp_dt = model.type_of(&p("ffn_down_shexp.weight"));
            let shexp = dense_ffn(&h_ffn, &model.load_raw(&p("ffn_gate_shexp.weight")), &model.load_raw(&p("ffn_up_shexp.weight")), &model.load(&p("ffn_down_shexp.weight")), N_EMBD, SHEXP_DIM, USE_Q8, shexp_dt);
            swarm_dequant::tap_write("ffn_moe_out", il, tpos, &moe);
            swarm_dequant::tap_write("ffn_shexp", il, tpos, &shexp);
            let res_nort_new: Vec<f32> = attn_out.iter().zip(&shexp).map(|(a, s)| a + s).collect();
            swarm_dequant::tap_write("l_out_no_routed", il, tpos, &res_nort_new);
            res_full = res_nort_new.iter().zip(&moe).map(|(r, m)| r + m).collect();
            res_nort = res_nort_new;
        }
        // Fork name "l_out" = the layer's full output after both streams (instella-moe.cpp 355).
        swarm_dequant::tap_write("l_out", il, tpos, &res_full);
    }
    let out_norm = model.load("output_norm.weight");
    let h_out = rms_norm(&res_full, &out_norm, 1e-6);
    // output.weight is ggml_type 14 = Q6_K, NOT Q4_K: 210 B per 256 values (1680 B per 2048-wide
    // row). The old raw-Q4_K routing read 1152 B/row, misaligned every row, and produced logits of
    // 3.1e9 — measured on this binary 2026-10-31. dequant_q6_k + the type-14 input quant is the
    // correct path. Instrument repair, not a model change.
    let w_out = model.load("output.weight");
    let h_out_q: Vec<f32> = if USE_Q8 { swarm_dequant::q_input(&h_out, 14) } else { h_out.clone() };
    let mut lg = vec![0.0f32; N_VOCAB];
    for j in 0..N_VOCAB {
        lg[j] = swarm_dequant::dot_fma(&h_out_q, &w_out[j * N_EMBD..(j + 1) * N_EMBD]);
    }
    lg
}

fn main() {
    let mut model = Model {
        m: swarm_gguf::parse_file(MODEL).expect("parse"),
        f: std::fs::File::open(MODEL).expect("open"),
    };
    let mut k_cache: Vec<Vec<Vec<f32>>> = vec![vec![]; N_LAYER];
    let mut v_cache: Vec<Vec<Vec<f32>>> = vec![vec![]; N_LAYER];
    let mut tokens: Vec<u32> = vec![0, 19923]; // BOS, "Hello"

    let mut logits = Vec::new();
    for (tpos, &tok) in tokens.iter().enumerate() {
        logits = forward_token(&mut model, &mut k_cache, &mut v_cache, tok, tpos);
    }

    let probs = softmax(&logits);
    let mut order: Vec<usize> = (0..N_VOCAB).collect();
    order.sort_by(|&a, &b| probs[b].partial_cmp(&probs[a]).unwrap_or(std::cmp::Ordering::Equal));
    println!("top-10 tokens (mine):");
    for k in 0..10 {
        let t = order[k];
        println!("  {k}: token {t}  prob {:.6}  logit {:.4}", probs[t], logits[t]);
    }

    // compare against llama.cpp's dumped logits
    let text = std::fs::read_to_string("D:/llama.cpp-instella/build/bin/Release/dump_logits.txt").unwrap_or_default();
    let mut ref_logits = Vec::new();
    let mut n = 0usize;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("result_output") {
            if let Some(rest) = t.split("n=").nth(1) { n = rest.trim().parse().unwrap_or(0); }
            continue;
        }
        if let Ok(v) = t.parse::<f32>() {
            ref_logits.push(v);
            if ref_logits.len() >= n && n > 0 { break; }
        }
    }
    if ref_logits.len() == N_VOCAB {
        let mut max_abs = 0.0f32;
        let mut sum_abs = 0.0f64;
        let mut max_idx = 0;
        for i in 0..N_VOCAB {
            let d = (logits[i] - ref_logits[i]).abs();
            sum_abs += d as f64;
            if d > max_abs { max_abs = d; max_idx = i; }
        }
        println!("LOGITS PARITY vs llama.cpp: max_abs={max_abs:.6} (idx {max_idx}) mean_abs={:.6}", sum_abs / N_VOCAB as f64);
        println!("  mine[{max_idx}]={:.6} ref[{max_idx}]={:.6}", logits[max_idx], ref_logits[max_idx]);
        let mut rorder: Vec<usize> = (0..N_VOCAB).collect();
        rorder.sort_by(|&a, &b| ref_logits[b].partial_cmp(&ref_logits[a]).unwrap_or(std::cmp::Ordering::Equal));
        print!("  ref top-10 tokens: [");
        for k in 0..10 { print!("{}{}", if k>0 {","} else {""}, rorder[k]); }
        println!("]");
        print!("  mine top-10 tokens: [");
        for k in 0..10 { print!("{}{}", if k>0 {","} else {""}, order[k]); }
        println!("]");
    } else {
        println!("ref logits n={} (expected {N_VOCAB})", ref_logits.len());
    }

    // greedy autoregressive generation (timed)
    let mut sampled: Vec<u32> = Vec::new();
    let t0 = std::time::Instant::now();
    let gen_steps: usize = std::env::var("GEN_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(16);
    for step in 0..gen_steps {
        let (next, pnext) = argmax(&logits);
        tokens.push(next);
        sampled.push(next);
        logits = forward_token(&mut model, &mut k_cache, &mut v_cache, next, tokens.len() - 1);
        println!("gen {step}: token {next} (prob {pnext:.6})");
    }
    let gen_elapsed = t0.elapsed();
    println!("BENCH: 16 tokens in {gen_elapsed:?} = {:.2} tokens/sec ({:.1} ms/token)",
        16.0 / gen_elapsed.as_secs_f64(), gen_elapsed.as_secs_f64() * 1000.0 / 16.0);
    print!("sampled tokens: [");
    for (i, t) in sampled.iter().enumerate() { print!("{}{}", if i>0 {","} else {""}, t); }
    println!("]");
}

fn argmax(logits: &[f32]) -> (u32, f32) {
    let mut best = 0usize;
    let mut bp = f32::NEG_INFINITY;
    for (i, &v) in logits.iter().enumerate() {
        if v > bp { bp = v; best = i; }
    }
    (best as u32, bp)
}
