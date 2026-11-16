//! Verify dequant against the real Instella GGUF: read an expert tensor's bytes at its offset,
//! dequantize, and print stats (min/max/mean + the first few values) — sane floats, not NaN/zero.

use std::io::{Read, Seek};

fn main() {
    let path = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
    let m = swarm_gguf::parse_file(path).expect("parse failed");
    let mut f = std::fs::File::open(path).expect("open");

    // 1. Q8_0 tensor: blk.1.ffn_down_exps.weight [1408, 2048, 64] type 8
    let q8 = m.tensors.iter().find(|t| t.name == "blk.1.ffn_down_exps.weight").expect("q8 tensor");
    let abs = m.data_start + q8.offset;
    let n_vals = q8.dims.iter().product::<u64>() as usize;
    let mut buf = vec![0u8; (n_vals / 32) * 34 + 64];
    f.seek(std::io::SeekFrom::Start(abs)).unwrap();
    f.read_exact(&mut buf[..(n_vals / 32) * 34]).unwrap();
    let q8_vals = swarm_dequant::dequant_q8_0(&buf, n_vals.min(64));
    print_stats("Q8_0 ffn_down_exps (first 64)", &q8_vals);

    // 2. Q4_K tensor: blk.1.ffn_gate_exps.weight [2048, 1408, 64] type 12
    let q4 = m.tensors.iter().find(|t| t.name == "blk.1.ffn_gate_exps.weight").expect("q4 tensor");
    let abs4 = m.data_start + q4.offset;
    let n4 = q4.dims.iter().product::<u64>() as usize;
    let mut buf4 = vec![0u8; (n4 / 256) * 144 + 256];
    f.seek(std::io::SeekFrom::Start(abs4)).unwrap();
    f.read_exact(&mut buf4[..(n4 / 256) * 144]).unwrap();
    let q4_vals = swarm_dequant::dequant_q4_k(&buf4, n4.min(256));
    print_stats("Q4_K ffn_gate_exps (first 256)", &q4_vals);
}

fn print_stats(label: &str, vals: &[f32]) {
    let n = vals.len();
    let mut mn = f32::INFINITY;
    let mut mx = f32::NEG_INFINITY;
    let mut sum = 0.0f64;
    for &v in vals {
        if v < mn { mn = v; }
        if v > mx { mx = v; }
        sum += v as f64;
    }
    let mean = sum / n as f64;
    let nan = vals.iter().filter(|v| v.is_nan()).count();
    let zero = vals.iter().filter(|v| **v == 0.0).count();
    println!("{label}: n={n} min={mn:.5} max={mx:.5} mean={mean:.5} nan={nan} zero={zero}");
    println!("  first 8: {:?}", &vals[..8.min(n)]);
}
