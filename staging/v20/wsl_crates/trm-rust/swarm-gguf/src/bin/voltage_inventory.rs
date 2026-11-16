use swarm_gguf::parse_file;
fn main() {
    let m = parse_file(&swarm_paths::resolve("/home/shawn/models/qwen-moe-2.4b-q4_k_m.gguf"))
        .expect("parse");
    println!("GGUF: {} tensors", m.tensors.len());
    // architecture KV (skip tokenizer arrays)
    for (k, v) in &m.kvs {
        if k.contains("tokenizer") || k == "general.tags" || k == "general.languages"
            || k.contains("base_model") { continue; }
        println!("KV {k} = {v}");
    }
    println!("--- tensor shapes ---");
    // print ALL distinct tensor name patterns with dims, grouped
    let mut seen: Vec<(String, Vec<u64>)> = Vec::new();
    for t in &m.tensors {
        let pat = if t.name.contains("blk.") {
            let rest = t.name.split_once("blk.").unwrap().1;
            let after = rest.split_once('.').map(|(_, r)| r).unwrap_or(rest);
            format!("blk.N.{after}")
        } else { t.name.clone() };
        if !seen.iter().any(|(p, _)| *p == pat) {
            seen.push((pat, t.dims.clone()));
        }
    }
    for (p, d) in &seen { println!("T {p}: {d:?}"); }
}
