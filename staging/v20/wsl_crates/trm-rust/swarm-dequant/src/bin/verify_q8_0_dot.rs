use std::io::{Read, Seek, SeekFrom};
use std::time::Instant;
use swarm_dequant::{dequant_q8_0, matmul_q8_0, quantize_q8_0};

const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";

fn main() {
    let m = swarm_gguf::parse_file(MODEL).unwrap();
    let t = m.tensors.iter().find(|t| t.name == "blk.1.ffn_down_exps.weight").unwrap();
    let (in_dim, out_dim) = (t.dims[0] as usize, t.dims[1] as usize);
    assert_eq!(t.ggml_type, 8, "expected Q8_0, got {}", t.ggml_type);
    let bytes_per = in_dim / 32 * out_dim * 34;
    let mut f = std::fs::File::open(MODEL).unwrap();
    f.seek(SeekFrom::Start(m.data_start + t.offset)).unwrap();
    let mut raw = vec![0u8; bytes_per];
    f.read_exact(&mut raw).unwrap();

    let x: Vec<f32> = (0..in_dim).map(|i| (i as f32 - 700.0) / 1400.0).collect();

    // quantized dot (int8, no dequant)
    let t0 = Instant::now();
    let out_q = matmul_q8_0(&raw, in_dim, out_dim, &x);
    let tq = t0.elapsed();

    // f32 path: dequant the weights + quantize x to Q8_0, then f32 matmul
    let t0 = Instant::now();
    let w = dequant_q8_0(&raw, in_dim * out_dim);
    let xq = quantize_q8_0(&x);
    let mut out_f = vec![0.0f32; out_dim];
    for j in 0..out_dim {
        let mut acc = 0.0f32;
        for i in 0..in_dim {
            acc += xq[i] * w[i + in_dim * j];
        }
        out_f[j] = acc;
    }
    let tf = t0.elapsed();

    let mut max_abs = 0.0f32;
    let mut sum_abs = 0.0f64;
    for j in 0..out_dim {
        let d = (out_q[j] - out_f[j]).abs();
        max_abs = max_abs.max(d);
        sum_abs += d as f64;
    }
    println!("Q8_0 GEMV [{}x{}]: max_abs={max_abs:.6} mean_abs={:.6}", out_dim, in_dim, sum_abs / out_dim as f64);
    println!("  quantized (int8, no dequant): {tq:?}");
    println!("  f32 (dequant + matmul):       {tf:?}");
    println!("  speedup: {:.2}x", tf.as_secs_f64() / tq.as_secs_f64().max(1e-9));
}
