use std::env;
fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf".to_string());
    let m = swarm_gguf::parse_file(&path).expect("parse");
    for (k, v) in &m.kvs {
        println!("{k} = {v}");
    }
}
