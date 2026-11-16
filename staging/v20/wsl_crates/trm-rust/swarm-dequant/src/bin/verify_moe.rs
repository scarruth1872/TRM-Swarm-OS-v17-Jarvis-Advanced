//! Verify the full sparse MoE forward for blk.1: router (gate_inp) -> top-6 -> dequant the active
//! experts -> gated FFN -> combine. Sane output = the native sparse forward works end to end.

use std::io::{Read, Seek};

const HIDDEN: usize = 2048;
const EXPERT_DIM: usize = 1408;
const N_EXPERTS: usize = 64;
const N_USED: usize = 6;
const W_SCALE: f32 = 2.5;

fn read_expert(
    f: &mut std::fs::File,
    m: &swarm_gguf::GgufModel,
    name: &str,
    e: usize,
) -> (Vec<u8>, usize) {
    let t = m.tensors.iter().find(|t| t.name == name).expect("tensor");
    let n_vals = t.dims[0] as usize * t.dims[1] as usize; // per-expert element count
    let block_vals = match t.ggml_type { 12 => 256, 8 => 32, _ => 1 };
    let bytes_per_val = match t.ggml_type { 12 => 144.0 / 256.0, 8 => 34.0 / 32.0, _ => 4.0 };
    let bytes_per_expert = (n_vals as f64 * bytes_per_val) as usize;
    let abs = m.data_start + t.offset + (e as u64) * (bytes_per_expert as u64);
    let mut buf = vec![0u8; bytes_per_expert];
    f.seek(std::io::SeekFrom::Start(abs)).unwrap();
    f.read_exact(&mut buf).unwrap();
    let _ = block_vals;
    (buf, n_vals)
}

fn main() {
    let path = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
    let m = swarm_gguf::parse_file(path).expect("parse");
    let mut f = std::fs::File::open(path).expect("open");

    // gate_inp is f32 [2048, 64] — read directly
    let gi = m.tensors.iter().find(|t| t.name == "blk.1.ffn_gate_inp.weight").expect("gate_inp");
    let gi_vals = gi.dims[0] as usize * gi.dims[1] as usize;
    let mut gate_inp = vec![0u8; gi_vals * 4];
    f.seek(std::io::SeekFrom::Start(m.data_start + gi.offset)).unwrap();
    f.read_exact(&mut gate_inp).unwrap();
    let gate_inp: Vec<f32> = gate_inp.chunks(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
    println!("gate_inp: {} f32", gate_inp.len());

    // sample hidden (deterministic ramp)
    let h: Vec<f32> = (0..HIDDEN).map(|i| (i as f32 - 1024.0) / 1024.0).collect();

    // router
    let (top, weights) = swarm_dequant::moe_router(&h, &gate_inp, &vec![0.0f32; N_EXPERTS], N_EXPERTS, N_USED, W_SCALE, false);
    println!("router top-{}: {:?}  weights: {:?}", N_USED, top, weights.iter().map(|w| format!("{:.4}", w)).collect::<Vec<_>>());

    // for each active expert: dequant + FFN + combine
    let mut moe_out = vec![0.0f32; HIDDEN];
    for (k, &e) in top.iter().enumerate() {
        let (gb, n) = read_expert(&mut f, &m, "blk.1.ffn_gate_exps.weight", e);
        let w_gate = swarm_dequant::dequant_q4_k(&gb, n);
        let (ub, _) = read_expert(&mut f, &m, "blk.1.ffn_up_exps.weight", e);
        let w_up = swarm_dequant::dequant_q4_k(&ub, n);
        let (db, _) = read_expert(&mut f, &m, "blk.1.ffn_down_exps.weight", e);
        let w_down = swarm_dequant::dequant_q8_0(&db, n);
        let out = swarm_dequant::expert_ffn(&h, &w_gate, &w_up, &w_down, HIDDEN, EXPERT_DIM, false, 0);
        for i in 0..HIDDEN {
            moe_out[i] += weights[k] * out[i];
        }
    }

    let mn = moe_out.iter().cloned().fold(f32::INFINITY, f32::min);
    let mx = moe_out.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mean = moe_out.iter().map(|&v| v as f64).sum::<f64>() / moe_out.len() as f64;
    let nan = moe_out.iter().filter(|v| v.is_nan()).count();
    println!("moe_out: n={} min={mn:.5} max={mx:.5} mean={mean:.5} nan={nan}", moe_out.len());
    println!("  first 8: {:?}", &moe_out[..8]);
}
