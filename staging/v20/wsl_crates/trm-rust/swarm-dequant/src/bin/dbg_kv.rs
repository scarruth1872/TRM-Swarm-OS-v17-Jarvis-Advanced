const MODEL: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";
fn main() {
    let m = swarm_gguf::parse_file(MODEL).unwrap();
    for (k, v) in &m.kvs {
        let kl = k.to_lowercase();
        if kl.contains("scale") || kl.contains("norm") || kl.contains("rope") || kl.contains("kq")
            || kl.contains("expert") || kl.contains("epsilon") || kl.contains("theta") {
            println!("KV[{k}] = {}", format!("{:?}", v).chars().take(70).collect::<String>());
        }
    }
}
