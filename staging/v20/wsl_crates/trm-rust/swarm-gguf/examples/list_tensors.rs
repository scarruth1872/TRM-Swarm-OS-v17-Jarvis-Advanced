use std::env;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf".to_string()
    });
    let m = swarm_gguf::parse_file(&path).expect("parse failed");
    println!("tensors: {}", m.tensors.len());
    let kw = ["ffn", "exps", "expert", "gate", "mo_e", "moe"];
    let mut shown = 0;
    for t in &m.tensors {
        let low = t.name.to_lowercase();
        if kw.iter().any(|k| low.contains(k)) {
            println!("{:60} dims={:?} type={}", t.name, t.dims, t.ggml_type);
            shown += 1;
            if shown >= 20 {
                break;
            }
        }
    }
    println!("--- blk.0 full names ---");
    for t in m.tensors.iter().filter(|t| t.name.starts_with("blk.0.")) {
        println!("  {} dims={:?}", t.name, t.dims);
    }
}
