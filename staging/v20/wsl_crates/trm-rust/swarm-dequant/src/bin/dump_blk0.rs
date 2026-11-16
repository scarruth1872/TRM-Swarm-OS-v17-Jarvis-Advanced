use std::env;
fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf".to_string());
    let m = swarm_gguf::parse_file(&path).expect("parse");
    for t in &m.tensors {
        if t.name.starts_with("blk.0.") || t.name.starts_with("blk.1.") || t.name.starts_with("blk.26.") || t.name == "output_norm.weight" {
            println!("  {} dims={:?} type={}", t.name, t.dims, t.ggml_type);
        }
    }
}
