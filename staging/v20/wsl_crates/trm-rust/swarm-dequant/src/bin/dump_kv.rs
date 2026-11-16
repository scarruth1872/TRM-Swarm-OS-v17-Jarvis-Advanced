use std::env;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf".to_string()
    });
    let m = swarm_gguf::parse_file(&path).expect("parse");
    for (k, v) in &m.kvs {
        let kl = k.to_lowercase();
        if kl.contains("expert") || kl.contains("moe") || kl.contains("gating") || kl.contains("ffn")
            || kl.contains("n_layer") || kl.contains("n_embd") || kl.contains("n_exp")
        {
            let vv = if v.len() > 80 { &v[..80] } else { v };
            println!("{k} = {vv}");
        }
    }
}
