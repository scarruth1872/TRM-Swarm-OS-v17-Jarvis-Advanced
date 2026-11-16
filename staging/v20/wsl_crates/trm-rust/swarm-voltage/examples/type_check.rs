use swarm_gguf::parse_file;
fn main() {
    let m = parse_file(&swarm_paths::resolve("/home/shawn/models/qwen-moe-2.4b-q4_k_m.gguf")).unwrap();
    for t in &m.tensors {
        if t.name.contains("blk.0.") || t.name.contains("token_embd") || t.name.contains("output") {
            println!("{} type={} dims={:?}", t.name, t.ggml_type, t.dims);
        }
    }
}
