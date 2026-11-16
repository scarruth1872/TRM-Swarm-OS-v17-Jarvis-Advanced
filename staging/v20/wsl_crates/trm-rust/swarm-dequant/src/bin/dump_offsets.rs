use std::env;
fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf".to_string());
    let m = swarm_gguf::parse_file(&path).expect("parse");
    let mut ts: Vec<_> = m.tensors.clone();
    ts.sort_by_key(|t| t.offset);
    let mut prev = 0u64;
    let mut prev_name = String::new();
    let mut mismatches = 0;
    for t in &ts {
        let n = t.dims.iter().product::<u64>() as usize;
        let computed = match t.ggml_type {
            12 => n / 256 * 144, 8 => n / 32 * 34, 6 => n / 32 * 22, 14 => n / 256 * 210, 1 => n * 2, _ => n * 4,
        };
        let actual = t.offset - prev;
        let ok = prev == 0 || actual == computed as u64;
        if !ok && prev != 0 {
            mismatches += 1;
            println!("{:<28} type={:2} n={:>9} computed={:>9} actual={:>9} MISMATCH (after {})", t.name, t.ggml_type, n, computed, actual, prev_name);
        }
        prev = t.offset;
        prev_name = t.name.clone();
    }
    println!("total mismatches: {mismatches} / {} tensors", ts.len());
}
