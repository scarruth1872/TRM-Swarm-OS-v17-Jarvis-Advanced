//! Verify the Q8_K-input hypothesis: llama.cpp quantizes `cur` to Q8_K before the Q4_K dot.
//! Quantize h to Q8_K (block 256, scale = max_abs/127) and re-compute the gate — if the
//! divergence against the dump collapses, the ~3% is llama.cpp's input quantization, not our bug.
use std::io::{Read, Seek, SeekFrom};

use swarm_dequant::dequant_q4_k;

const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
const DIR: &str = "D:/llama.cpp-instella/build/bin/Release";

fn load(f: &mut std::fs::File, m: &swarm_gguf::GgufModel, name: &str) -> Vec<f32> {
    let t = m.tensors.iter().find(|t| t.name == name).expect(name);
    let n = t.dims.iter().product::<u64>() as usize;
    let bytes = n / 256 * 144;
    let mut buf = vec![0u8; bytes];
    f.seek(SeekFrom::Start(m.data_start + t.offset)).unwrap();
    f.read_exact(&mut buf).unwrap();
    dequant_q4_k(&buf, n)
}

fn read_dump(path: &str) -> Vec<f32> {
    let text = std::fs::read_to_string(path).expect(path);
    let mut n = 0usize;
    let mut vals = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("attn") || t.starts_with("q-") || t.starts_with("kv_cmpr") {
            if let Some(rest) = t.split("n=").nth(1) {
                n = rest.trim().parse().unwrap_or(0);
            }
            continue;
        }
        if let Ok(v) = t.parse::<f32>() {
            vals.push(v);
            if vals.len() >= n && n > 0 {
                break;
            }
        }
    }
    vals
}

/// Quantize x to Q8_K: 256-value blocks, per-block scale = max_abs/127, q = round(x/scale).
fn quantize_q8_k(x: &[f32]) -> Vec<f32> {
    let mut out = vec![0.0f32; x.len()];
    let mut blocks = x.chunks_exact(256);
    for (bi, blk) in blocks.by_ref().enumerate() {
        let max_abs = blk.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        let scale = (max_abs / 127.0).max(1e-9);
        for (i, &v) in blk.iter().enumerate() {
            let q = (v / scale).round().clamp(-127.0, 127.0);
            out[bi * 256 + i] = q * scale;
        }
    }
    out
}

fn gate(wg: &[f32], h: &[f32]) -> Vec<f32> {
    let mut out = vec![0.0f32; 2048];
    for j in 0..2048 {
        let mut acc = 0.0;
        for i in 0..2048 {
            acc += wg[i + 2048 * j] * h[i];
        }
        out[j] = acc;
    }
    out
}

fn main() {
    let m = swarm_gguf::parse_file(MODEL).expect("parse");
    let mut f = std::fs::File::open(MODEL).expect("open");
    let wg = load(&mut f, &m, "blk.0.attn_gate.weight");
    let h = read_dump(&format!("{DIR}/dump_attn_norm.txt"));
    let ref_gate = read_dump(&format!("{DIR}/dump_attn_gate_proj.txt"));

    let g_exact = gate(&wg, &h);
    let h_q8 = quantize_q8_k(&h);
    let g_q8 = gate(&wg, &h_q8);

    let mut max_exact = 0.0f32;
    let mut max_q8 = 0.0f32;
    for i in 0..2048 {
        max_exact = max_exact.max((g_exact[i] - ref_gate[i]).abs());
        max_q8 = max_q8.max((g_q8[i] - ref_gate[i]).abs());
    }
    println!("gate vs dump: max_abs (exact f32 h) = {max_exact:.6}");
    println!("gate vs dump: max_abs (Q8_K h)    = {max_q8:.6}");
    println!("  gate[0]: exact={:.6} q8={:.6} ref={:.6}", g_exact[0], g_q8[0], ref_gate[0]);
}
