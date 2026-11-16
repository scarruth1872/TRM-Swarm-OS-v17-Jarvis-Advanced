//! swarm-harness — the agents' tool-loop CLI.
//!
//! Two capabilities, native Rust, no Python, no HTTP server:
//!   swarm-harness code  "<prompt>"          -> generate code via the tier-2 coder (:8587)
//!   swarm-harness strip <name> "<input>"    -> run a deterministic strip (recall/derive/...)
//!
//! The `code` path uses the same ChatML template the kernel's tier-2 call uses, so the Qwen2.5-Coder
//! answers instead of continuing the persona system prompt.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const CODER_ADDR: &str = "127.0.0.1:8587";
const INSTELLA_ADDR: &str = "127.0.0.1:8588";

/// Minimal HTTP POST to the tier-2 llama-server `/completion`. ChatML template (matches the
/// kernel's `bonsai_vulkan_generate`). Returns the trimmed generated text.
fn coder_generate(prompt: &str) -> Option<String> {
    let combo = format!(
        "<|im_start|>system\nYou are a coding assistant. Return only the requested code, no explanation.\
         <|im_end|>\n<|im_start|>user\n{prompt}<|im_end|>\n<|im_start|>assistant\n"
    );
    let payload = serde_json::json!({
        "prompt": combo,
        "n_predict": 256,
        "temperature": 0.2,
        "stream": false,
    })
    .to_string();

    let mut stream = TcpStream::connect(CODER_ADDR).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(120))).ok()?;
    let req = format!(
        "POST /completion HTTP/1.1\r\nHost: {CODER_ADDR}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );
    stream.write_all(req.as_bytes()).ok()?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw).ok()?;
    let body = raw.split("\r\n\r\n").nth(1)?;
    serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .get("content")?
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Full 16B MoE forward via the fork (:8588) — the raw prompt (the Instella reasoner supplies its
/// own template). Returns the trimmed generated text.
fn instella_generate(prompt: &str) -> Option<String> {
    let payload = serde_json::json!({
        "prompt": prompt,
        "n_predict": 256,
        "temperature": 0.2,
        "stream": false,
    })
    .to_string();
    let mut stream = TcpStream::connect(INSTELLA_ADDR).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(600))).ok()?;
    let req = format!(
        "POST /completion HTTP/1.1\r\nHost: {INSTELLA_ADDR}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );
    stream.write_all(req.as_bytes()).ok()?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw).ok()?;
    let body = raw.split("\r\n\r\n").nth(1)?;
    serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .get("content")?
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Native sparse gating: read `{"hidden":[384], "experts":[[384] x 64], "k":N}` from stdin and run
/// `swarm_instella::gate_topk` + `expert_forward`. No HTTP — the routing layer runs in-process.
fn gate_from_stdin() -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&input).map_err(|e| e.to_string())?;
    let hidden: Vec<f32> = v.get("hidden").and_then(|x| x.as_array())
        .ok_or("hidden must be an array of 384 floats")?
        .iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
    let experts: Vec<Vec<f32>> = v.get("experts").and_then(|x| x.as_array())
        .ok_or("experts must be an array of 64 x 384 floats")?
        .iter().map(|row| row.as_array()
            .map(|r| r.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect())
            .unwrap_or_default()).collect();
    let k = v.get("k").and_then(|x| x.as_u64()).unwrap_or(4) as usize;
    if hidden.len() != swarm_instella::HIDDEN_DIM || experts.len() != swarm_instella::TOTAL_EXPERTS {
        return Err(format!(
            "expected hidden[{}] and experts[{}][{}], got hidden[{}] experts[{}]",
            swarm_instella::HIDDEN_DIM, swarm_instella::TOTAL_EXPERTS, swarm_instella::HIDDEN_DIM,
            hidden.len(), experts.len()
        ));
    }
    let mut h = [0.0f32; swarm_instella::HIDDEN_DIM];
    h.copy_from_slice(&hidden);
    let mut w = [[0.0f32; swarm_instella::HIDDEN_DIM]; swarm_instella::TOTAL_EXPERTS];
    for (i, row) in experts.iter().enumerate() {
        if row.len() == swarm_instella::HIDDEN_DIM {
            w[i].copy_from_slice(row);
        }
    }
    let (ids, weights) = swarm_instella::gate_topk(&h, &w, k);
    let top_forward = swarm_instella::expert_forward(&h, &w[ids[0] as usize]);
    Ok(serde_json::json!({
        "ids": ids, "weights": weights, "top_expert_forward": top_forward,
        "engine": "swarm-instella (native, no HTTP)"
    }).to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("code") => {
            let prompt = args.get(2).cloned().unwrap_or_default();
            if prompt.is_empty() {
                eprintln!("swarm-harness: code needs a prompt");
                std::process::exit(2);
            }
            match coder_generate(&prompt) {
                Some(code) => println!("{code}"),
                None => {
                    eprintln!("harness: coder unreachable on {CODER_ADDR}");
                    std::process::exit(1);
                }
            }
        }
        Some("instella") => {
            let prompt = args.get(2).cloned().unwrap_or_default();
            if prompt.is_empty() {
                eprintln!("swarm-harness: instella needs a prompt");
                std::process::exit(2);
            }
            match instella_generate(&prompt) {
                Some(text) => println!("{text}"),
                None => {
                    eprintln!("harness: instella unreachable on {INSTELLA_ADDR}");
                    std::process::exit(1);
                }
            }
        }
        Some("gate") => match gate_from_stdin() {
            Ok(out) => println!("{out}"),
            Err(e) => {
                eprintln!("harness: gate error: {e}");
                std::process::exit(1);
            }
        },
        Some("strip") => {
            let name = args.get(2).cloned().unwrap_or_default();
            let input = args.get(3).cloned().unwrap_or_default();
            match swarm_strips::run_strip(&name, &input) {
                Some(out) => println!("{out}"),
                None => {
                    eprintln!("harness: no strip '{name}'");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("swarm-harness: code | instella | gate | strip");
            std::process::exit(2);
        }
    }
}
