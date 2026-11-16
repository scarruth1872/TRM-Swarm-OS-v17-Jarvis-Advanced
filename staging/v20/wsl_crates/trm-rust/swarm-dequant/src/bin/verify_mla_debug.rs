//! Pinpoint the MLA divergence: compute each blk.0 intermediate (gate, q, kv_cmpr, attn_out,
//! attn_gated, out) and compare against the dumped llama.cpp reference. The FIRST step that
//! diverges is the bug.

use std::io::{Read, Seek, SeekFrom};

use swarm_dequant::{dequant_q4_k, dequant_q5_0, rms_norm};

const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
const DIR: &str = "D:/llama.cpp-instella/build/bin/Release";

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

fn read_dump(path: &str) -> Vec<f32> {
    let text = std::fs::read_to_string(path).expect(path);
    let mut n = 0usize;
    let mut vals: Vec<f32> = Vec::new();
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

/// out[j] = sum_i w[i + in_dim*j] * x[i]  (ne[0]-major, in_dim = x.len())
fn matmul(w: &[f32], x: &[f32], in_dim: usize) -> Vec<f32> {
    let out_dim = w.len() / in_dim;
    let mut out = vec![0.0f32; out_dim];
    for j in 0..out_dim {
        let mut acc = 0.0;
        for i in 0..in_dim {
            acc += w[i + in_dim * j] * x[i];
        }
        out[j] = acc;
    }
    out
}

fn compare(mine: &[f32], reference: &[f32], label: &str) {
    if mine.len() != reference.len() {
        println!("{label}: LEN MISMATCH mine={} ref={}", mine.len(), reference.len());
        return;
    }
    let mut max_abs = 0.0f32;
    let mut sum_abs = 0.0f64;
    for i in 0..mine.len() {
        let d = (mine[i] - reference[i]).abs();
        sum_abs += d as f64;
        if d > max_abs {
            max_abs = d;
        }
    }
    let mean = sum_abs / mine.len() as f64;
    println!("{label}: max_abs={max_abs:.6} mean_abs={mean:.6}");
    let k = mine.len().min(5);
    println!("    mine[..{k}] = {:?}", &mine[..k]);
    println!("    ref [..{k}] = {:?}", &reference[..k]);
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn main() {
    let m = swarm_gguf::parse_file(MODEL).expect("parse");
    let mut f = std::fs::File::open(MODEL).expect("open");
    let wq = load(&mut f, &m, "blk.0.attn_q.weight");
    let wkv_a = load(&mut f, &m, "blk.0.attn_kv_a_mqa.weight");
    let wv_b = load(&mut f, &m, "blk.0.attn_v_b.weight");
    let wg = load(&mut f, &m, "blk.0.attn_gate.weight");
    let wo = load(&mut f, &m, "blk.0.attn_output.weight");
    let kv_a_norm = load(&mut f, &m, "blk.0.attn_kv_a_norm.weight");

    let h = read_dump(&format!("{DIR}/dump_attn_norm.txt"));
    println!("h[..6] = {:?}", &h[..6]);
    let ref_gate = read_dump(&format!("{DIR}/dump_attn_gate_proj.txt"));
    let ref_q = read_dump(&format!("{DIR}/dump_q.txt"));
    let ref_kv = read_dump(&format!("{DIR}/dump_kv_cmpr.txt"));
    let ref_out = read_dump(&format!("{DIR}/dump_attn_out.txt"));
    let ref_gated = read_dump(&format!("{DIR}/dump_attn_gated.txt"));
    let ref_proj = read_dump(&format!("{DIR}/dump_attn_o_proj.txt"));
    println!(
        "inputs: h={} gate={} q={} kv={} out={} gated={} proj={}",
        h.len(), ref_gate.len(), ref_q.len(), ref_kv.len(), ref_out.len(), ref_gated.len(), ref_proj.len()
    );

    // 1. gate = wg @ h
    let gate = matmul(&wg, &h, 2048);
    compare(&gate, &ref_gate, "1 gate");

    // 2. q = wq @ h
    let q = matmul(&wq, &h, 2048);
    compare(&q, &ref_q, "2 q");

    // 3. kv_cmpr = rms_norm(wkv_a @ h, kv_a_norm)
    let kv_raw = matmul(&wkv_a, &h, 2048);
    let kv_cmpr = rms_norm(&kv_raw[..512], &kv_a_norm, 1e-6);
    compare(&kv_cmpr, &ref_kv, "3 kv_cmpr");

    // 4. attn_out = wv_b @ kv_cmpr (V-up, pos-0 single key -> softmax=1)
    //    wv_b [512,128,16]: per head h, out[j + 128*h] = sum_d wv_b[d + 512*j + 512*128*h] * kv_cmpr[d]
    let mut attn_out = vec![0.0f32; 2048];
    for hh in 0..16 {
        for j in 0..128 {
            let mut acc = 0.0;
            for d in 0..512 {
                acc += wv_b[d + 512 * j + 512 * 128 * hh] * kv_cmpr[d];
            }
            attn_out[j + 128 * hh] = acc;
        }
    }
    compare(&attn_out, &ref_out, "4 attn_out(V-up)");

    // 5. gating
    let mut gated = attn_out.clone();
    for i in 0..2048 {
        gated[i] *= sigmoid(gate[i]);
    }
    compare(&gated, &ref_gated, "5 attn_gated");

    // 6. wo @ gated
    let out = matmul(&wo, &gated, 2048);
    compare(&out, &ref_proj, "6 attn_o_proj");
}
