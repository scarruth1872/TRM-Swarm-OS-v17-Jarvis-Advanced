const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
fn main() {
    let m = swarm_gguf::parse_file(MODEL).unwrap();
    for (k, v) in &m.kvs {
        let kl = k.to_lowercase();
        if kl.contains("key_length") || kl.contains("value_length") || kl.contains("head_count")
            || kl.contains("lora") || kl.contains("dimension_count") {
            println!("KV[{k}] = {}", format!("{:?}", v).chars().take(50).collect::<String>());
        }
    }
}
