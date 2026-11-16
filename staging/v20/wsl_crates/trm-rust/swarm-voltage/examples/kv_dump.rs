//! Dump expert_weights_scale and related KV from the real model.
use swarm_gguf::parse_file;
fn main() {
    let m = parse_file(&swarm_paths::resolve("/home/shawn/models/qwen-moe-2.4b-q4_k_m.gguf")).expect("parse");
    for (k, v) in &m.kvs {
        if k.contains("scale") || k.contains("weights") || k.contains("expert") {
            println!("{k} = {v}");
        }
    }
    println!("--- all expert/ffn keys ---");
    for (k, v) in &m.kvs {
        if k.contains("ffn") || k.contains("expert") {
            println!("{k} = {v}");
        }
    }
}
