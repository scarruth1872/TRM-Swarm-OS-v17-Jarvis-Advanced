//! Verify the gated expert FFN: dequant expert 0 of blk.1 (gate/up Q4_K, down Q8_0) from the real
//! Instella GGUF and run the FFN on a sample hidden state. Sane output = the forward works.

use std::io::{Read, Seek};

const HIDDEN: usize = 2048;
const EXPERT_DIM: usize = 1408;

fn read_tensor(f: &mut std::fs::File, m: &swarm_gguf::GgufModel, name: &str) -> (Vec<u8>, usize) {
    let t = m.tensors.iter().find(|t| t.name == name).expect("tensor not found");
    let abs = m.data_start + t.offset;
    let n_vals = t.dims.iter().product::<u64>() as usize;
    let n_expert = *t.dims.last().unwrap() as usize;
    let per_expert = n_vals / n_expert; // expert 0 is the first slice
    let bytes_per = match t.ggml_type {
        12 => (per_expert / 256) * 144, // Q4_K
        8 => (per_expert / 32) * 34,    // Q8_0
        _ => per_expert * 4,
    };
    let mut buf = vec![0u8; bytes_per];
    f.seek(std::io::SeekFrom::Start(abs)).unwrap();
    f.read_exact(&mut buf).unwrap();
    (buf, per_expert)
}

fn main() {
    let path = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
    let m = swarm_gguf::parse_file(path).expect("parse");
    let mut f = std::fs::File::open(path).expect("open");

    let (gate_b, n) = read_tensor(&mut f, &m, "blk.1.ffn_gate_exps.weight");
    let w_gate = swarm_dequant::dequant_q4_k(&gate_b, n);
    let (up_b, _) = read_tensor(&mut f, &m, "blk.1.ffn_up_exps.weight");
    let w_up = swarm_dequant::dequant_q4_k(&up_b, n);
    let (down_b, _) = read_tensor(&mut f, &m, "blk.1.ffn_down_exps.weight");
    let w_down = swarm_dequant::dequant_q8_0(&down_b, n);
    println!("dequantized: gate {} up {} down {} f32", w_gate.len(), w_up.len(), w_down.len());

    // sample hidden: a deterministic ramp, normalized
    let h: Vec<f32> = (0..HIDDEN).map(|i| (i as f32 - 1024.0) / 1024.0).collect();
    let t0 = std::time::Instant::now();
    let out = swarm_dequant::expert_ffn(&h, &w_gate, &w_up, &w_down, HIDDEN, EXPERT_DIM, false, 0);
    let el = t0.elapsed();

    let mn = out.iter().cloned().fold(f32::INFINITY, f32::min);
    let mx = out.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mean = out.iter().map(|&v| v as f64).sum::<f64>() / out.len() as f64;
    let nan = out.iter().filter(|v| v.is_nan()).count();
    println!("expert_ffn: n={} min={mn:.5} max={mx:.5} mean={mean:.5} nan={nan} time={el:?}", out.len());
    println!("  first 8: {:?}", &out[..8]);
}
