//! Parity check: run the Rust MLA forward on llama.cpp's actual blk.0 attn_norm output and
//! compare against llama.cpp's attn_o_proj (the reference). Max/mean abs diff = the distill's
//! correctness measure. This is the gold standard, not the "sane output" smoke test.

use std::io::{Read, Seek, SeekFrom};

use swarm_dequant::mla::{mla_forward, N_EMBD};
use swarm_dequant::{dequant_q4_k, dequant_q5_0};

const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
const NORM_DUMP: &str = "D:/llama.cpp-instella/build/bin/Release/dump_attn_norm.txt";
const PROJ_DUMP: &str = "D:/llama.cpp-instella/build/bin/Release/dump_attn_o_proj.txt";

fn n_bytes(t: &swarm_gguf::TensorInfo) -> usize {
    let n = t.dims.iter().product::<u64>() as usize;
    match t.ggml_type {
        12 => n / 256 * 144,
        8 => n / 32 * 34,
        6 => n / 32 * 22,
        1 => n * 2,
        _ => n * 4,
    }
}

fn load(f: &mut std::fs::File, m: &swarm_gguf::GgufModel, name: &str) -> Vec<f32> {
    let t = m.tensors.iter().find(|t| t.name == name).expect(name);
    let n = t.dims.iter().product::<u64>() as usize;
    let mut buf = vec![0u8; n_bytes(t)];
    f.seek(SeekFrom::Start(m.data_start + t.offset)).unwrap();
    f.read_exact(&mut buf).unwrap();
    match t.ggml_type {
        12 => dequant_q4_k(&buf, n),
        6 => dequant_q5_0(&buf, n),
        _ => buf.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect(),
    }
}

/// Parse a dump file: skip the "attn_...-0 n=N" headers, read the first 2048 values (token 0
/// of blk.0 — the dump holds all 27 layers × 2 tokens, we only compare the first token).
fn read_dump(path: &str) -> Vec<f32> {
    let text = std::fs::read_to_string(path).expect(path);
    let mut vals = Vec::with_capacity(2048);
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("attn") {
            continue;
        }
        if let Ok(v) = t.parse::<f32>() {
            vals.push(v);
            if vals.len() >= 2048 {
                break;
            }
        }
    }
    vals
}

fn main() {
    let m = swarm_gguf::parse_file(MODEL).expect("parse");
    let mut f = std::fs::File::open(MODEL).expect("open");
    let wq = load(&mut f, &m, "blk.0.attn_q.weight");
    let wkv_a = load(&mut f, &m, "blk.0.attn_kv_a_mqa.weight");
    let wk_b = load(&mut f, &m, "blk.0.attn_k_b.weight");
    let wv_b = load(&mut f, &m, "blk.0.attn_v_b.weight");
    let wg = load(&mut f, &m, "blk.0.attn_gate.weight");
    let wo = load(&mut f, &m, "blk.0.attn_output.weight");
    let kv_a_norm = load(&mut f, &m, "blk.0.attn_kv_a_norm.weight");

    let h = read_dump(NORM_DUMP);
    let reference = read_dump(PROJ_DUMP);
    println!("input h n={}  reference n={}", h.len(), reference.len());
    assert_eq!(h.len(), N_EMBD, "attn_norm output must be 2048");

    let mut k_cache: Vec<Vec<f32>> = vec![];
    let mut v_cache: Vec<Vec<f32>> = vec![];
    let out = mla_forward(&h, 0, 0, &mut k_cache, &mut v_cache, &wq, &wkv_a, &kv_a_norm, &wk_b, &wv_b, &wg, &wo, false);

    assert_eq!(out.len(), reference.len(), "output dim mismatch");
    let mut max_abs = 0.0f32;
    let mut sum_abs = 0.0f64;
    let mut max_idx = 0;
    for i in 0..out.len() {
        let d = (out[i] - reference[i]).abs();
        sum_abs += d as f64;
        if d > max_abs {
            max_abs = d;
            max_idx = i;
        }
    }
    let mean_abs = sum_abs / out.len() as f64;
    println!("PARITY: max_abs={max_abs:.6} (idx {max_idx}) mean_abs={mean_abs:.6}");
    println!("  mine[..8]   = {:?}", &out[..8]);
    println!("  ref[..8]    = {:?}", &reference[..8]);
    println!("  ref[{max_idx}]={} mine[{max_idx}]={}", reference[max_idx], out[max_idx]);
    if max_abs < 1e-4 {
        println!("PASS: MLA attention matches llama.cpp (distill is faithful)");
    } else {
        println!("FAIL: MLA diverges from llama.cpp — the distill is not yet faithful");
    }
}
