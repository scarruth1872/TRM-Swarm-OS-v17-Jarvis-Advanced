use std::env;
fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf".to_string());
    let m = swarm_gguf::parse_file(&path).expect("parse");
    let mut ts: Vec<_> = m.tensors.clone();
    ts.sort_by_key(|t| t.offset);
    for (i, t) in ts.iter().enumerate() {
        if t.name.contains("blk.3.ffn") || (i > 0 && ts[i-1].name.contains("blk.3.ffn")) {
            let actual = if i+1 < ts.len() { ts[i+1].offset - t.offset } else { 0 };
            let n = t.dims.iter().product::<u64>() as usize;
            let computed = match t.ggml_type { 12 => n/256*144, 8 => n/32*34, 6 => n/32*22, 14 => n/256*210, _ => n*4 };
            println!("{:<28} type={:2} dims={:?} offset={} computed={} actual={} {}", t.name, t.ggml_type, t.dims, t.offset, computed, actual, if computed as u64==actual {""} else {"MISMATCH"});
        }
    }
}
