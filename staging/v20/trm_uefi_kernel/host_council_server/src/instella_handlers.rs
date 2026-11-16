//! /instella/* routes — the Instella distill parity primitives, run in pure Rust through the
//! kernel (not a side .exe). swarm-dequant owns the dequant + the int8 dot; this arm only carries
//! the answer, re-implementing no algorithm.

use std::io::{Read, Seek, SeekFrom};

use serde_json::{json, Value};

/// Deterministic SplitMix64 probe — the same sequence the verify harness uses, so the parity
/// number is reproducible across the kernel and the harness.
fn splitmix_probe(n: usize) -> Vec<f32> {
    let mut seed = 42u64;
    (0..n)
        .map(|_| {
            seed = seed.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = seed;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            ((z ^ (z >> 31)) as f64 / u64::MAX as f64 * 2.0 - 1.0) as f32
        })
        .collect()
}

/// /instella/q4k?model=&tensor=&col= — load one Q4_K tensor column from the GGUF, run the int8
/// dot (ggml_vec_dot_q4_K_q8_K shape) against a deterministic probe, and return the parity vs the
/// dequant-to-f32 path. The parity is the quantization rounding, not an error.
pub fn q4k_parity(model_path: &str, tensor: &str, col: usize) -> Value {
    let m = match swarm_gguf::parse_file(model_path) {
        Ok(m) => m,
        Err(e) => {
            return json!({"status":"error","route":"/instella/q4k","error":"parse","detail":e.to_string()})
        }
    };
    let t = match m.tensors.iter().find(|t| t.name == tensor) {
        Some(t) => t.clone(),
        None => {
            return json!({"status":"error","route":"/instella/q4k","error":"missing_tensor","detail":tensor})
        }
    };
    if t.ggml_type != 12 {
        return json!({
            "status":"error","route":"/instella/q4k","error":"wrong_type",
            "detail":format!("tensor {} is ggml_type {}, /instella/q4k serves Q4_K (type 12)", tensor, t.ggml_type)
        });
    }
    let in_dim = t.dims[0] as usize;
    let bytes_per_col = in_dim / 256 * 144; // Q4_K: 256 values = 144 bytes
    let mut f = match std::fs::File::open(model_path) {
        Ok(f) => f,
        Err(e) => {
            return json!({"status":"error","route":"/instella/q4k","error":"open","detail":e.to_string()})
        }
    };
    let abs = m.data_start + t.offset + (col as u64 * bytes_per_col as u64);
    let mut buf = vec![0u8; bytes_per_col];
    if f.seek(SeekFrom::Start(abs)).is_err() || f.read_exact(&mut buf).is_err() {
        return json!({"status":"error","route":"/instella/q4k","error":"read","detail":"seek/read failed"});
    }
    let x = splitmix_probe(in_dim);
    let int8 = swarm_dequant::dot_q4_K_q8_K(&buf, &x);
    let deq = swarm_dequant::dequant_q4_k(&buf, in_dim);
    let xq = swarm_dequant::quantize_q8_k(&x);
    let f32_dot: f32 = deq.iter().zip(&xq).map(|(a, b)| a * b).sum();
    json!({
        "status": "ok",
        "route": "/instella/q4k",
        "engine": "swarm-dequant (pure Rust, in-kernel)",
        "tensor": tensor,
        "col": col,
        "in_dim": in_dim,
        "int8_dot": int8,
        "dequant_f32_dot": f32_dot,
        "parity_abs": (int8 - f32_dot).abs(),
        "note": "int8 dot (ggml_vec_dot_q4_K_q8_K shape) vs dequant-to-f32; parity is the quantization rounding, expected ~1e-3",
    })
}
