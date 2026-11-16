//! Verify the Q4_K dequantizer against REAL model bytes.
//! Reads token_embd.weight from the actual GGUF, dequantizes the first
//! 256-weight block, and checks the value distribution is sane
//! (finite, reasonable magnitude — real embeddings, not zeros/garbage).
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use swarm_gguf::parse_file;
use swarm_voltage::dequantize_q4k_block;

fn main() {
    let path = swarm_paths::resolve("/home/shawn/models/qwen-moe-2.4b-q4_k_m.gguf");
    let m = parse_file(&path).expect("parse gguf");

    // find token_embd.weight
    let te = m.tensors.iter().find(|t| t.name == "token_embd.weight")
        .expect("token_embd.weight");
    println!("token_embd.weight: dims={:?} offset={} (rel data_start)",
             te.dims, te.offset);

    // file_type 15 = Q4_K. Block size = 144 bytes per 256 weights.
    // Read the first 144 bytes of the tensor data.
    let abs = m.data_start + te.offset;
    let mut f = File::open(path).expect("open");
    f.seek(SeekFrom::Start(abs)).expect("seek");
    let mut block = [0u8; 144];
    f.read_exact(&mut block).expect("read block");

    let mut w = [0f32; 256];
    dequantize_q4k_block(&block, &mut w);

    // sanity: finite, nonzero spread, reasonable magnitude
    let mut finite = 0;
    let mut max_abs = 0.0f32;
    let mut sum = 0.0f64;
    for &v in &w {
        if v.is_finite() { finite += 1; }
        max_abs = max_abs.max(v.abs());
        sum += v as f64;
    }
    println!("dequantized 256 real weights:");
    println!("  finite: {finite}/256");
    println!("  max|v|: {max_abs:.4}");
    println!("  mean: {:.6}", sum / 256.0);
    println!("  first 8: {:?}", &w[..8]);

    let ok = finite == 256 && max_abs > 0.01 && max_abs < 100.0;
    println!("\nREAL-DATA DEQUANT: {}", if ok { "VERIFIED" } else { "FAILED" });
}
