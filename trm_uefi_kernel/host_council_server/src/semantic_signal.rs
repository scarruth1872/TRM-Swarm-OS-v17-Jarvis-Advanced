//! Semantic Embedding Signal Source via Local `:8593` (`nomic-embed-text-v1.5`).
//!
//! Calls `http://127.0.0.1:8593/v1/embeddings` over loopback TCP and evaluates
//! a 768-dim L2-regularized linear head (`data/semantic_gate_weights.json`,
//! trained on `EMB_GATE_DATA.npz`, disjoint from `BLIND_DIRECTIVES*.jsonl`).
//!
//! Maps output to Kleene-3:
//!   BLOCK   -> -1
//!   ANALYZE ->  0
//!   PROCEED -> +1

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const EMB_ADDR: &str = "127.0.0.1:8593";
const LLM_ADDR: &str = "127.0.0.1:8587";
const ESCALATE_MARGIN_FLOOR: f64 = -1.5;

struct SemanticWeights {
    mean: Vec<f64>,
    std_dev: Vec<f64>,
    weights_t: [Vec<f64>; 3],
    bias: [f64; 3],
}

pub struct SemanticVerdict {
    pub trit: i8,
    pub label: &'static str,
    pub scores: [f64; 3],
    pub elapsed_ns: u128,
    pub escalated_8587: bool,
    pub stage2_elapsed_ns: u128,
}

static WEIGHTS: OnceLock<Option<SemanticWeights>> = OnceLock::new();

fn weights_path() -> Option<PathBuf> {
    if let Ok(root) = std::env::var("TRM_REPO_ROOT") {
        let p = PathBuf::from(root).join("data").join("semantic_gate_weights.json");
        if p.exists() {
            return Some(p);
        }
    }
    for cand in [
        "data/semantic_gate_weights.json",
        "../../data/semantic_gate_weights.json",
    ] {
        let p = PathBuf::from(cand);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn load_weights() -> &'static Option<SemanticWeights> {
    WEIGHTS.get_or_init(|| {
        let path = weights_path()?;
        let bytes = std::fs::read(path).ok()?;
        let v: Value = serde_json::from_slice(&bytes).ok()?;
        let to_vec = |arr: &Value| -> Option<Vec<f64>> {
            arr.as_array()?
                .iter()
                .map(|x| x.as_f64())
                .collect()
        };
        let mean = to_vec(v.get("mean")?)?;
        let std_dev = to_vec(v.get("std")?)?;
        let wt = v.get("weights_t")?.as_array()?;
        if wt.len() != 3 || mean.len() != 768 || std_dev.len() != 768 {
            return None;
        }
        let w0 = to_vec(&wt[0])?;
        let w1 = to_vec(&wt[1])?;
        let w2 = to_vec(&wt[2])?;
        let b_arr = v.get("bias")?.as_array()?;
        if b_arr.len() != 3 {
            return None;
        }
        Some(SemanticWeights {
            mean,
            std_dev,
            weights_t: [w0, w1, w2],
            bias: [
                b_arr[0].as_f64()?,
                b_arr[1].as_f64()?,
                b_arr[2].as_f64()?,
            ],
        })
    })
}

fn fetch_embedding_768(text: &str) -> Option<Vec<f64>> {
    let mut stream = TcpStream::connect(EMB_ADDR).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok()?;
    stream.set_write_timeout(Some(Duration::from_secs(2))).ok()?;

    let payload = json!({
        "input": [text],
        "model": "nomic-embed-text-v1.5"
    })
    .to_string();

    let req = format!(
        "POST /v1/embeddings HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        EMB_ADDR,
        payload.len(),
        payload
    );
    stream.write_all(req.as_bytes()).ok()?;

    let mut raw = String::new();
    let _ = stream.read_to_string(&mut raw);
    let body = raw.split("\r\n\r\n").nth(1)?;
    let v: Value = serde_json::from_str(body).ok()?;
    let emb = v
        .get("data")?
        .as_array()?
        .first()?
        .get("embedding")?
        .as_array()?;
    if emb.len() != 768 {
        return None;
    }
    emb.iter().map(|x| x.as_f64()).collect()
}

pub fn score_embedding(emb: &[f64], w: &SemanticWeights) -> (i8, &'static str, [f64; 3]) {
    let mut scores = w.bias;
    for i in 0..768 {
        let xn = (emb[i] - w.mean[i]) / w.std_dev[i];
        scores[0] += xn * w.weights_t[0][i];
        scores[1] += xn * w.weights_t[1][i];
        scores[2] += xn * w.weights_t[2][i];
    }
    if scores[0] >= scores[1] && scores[0] >= scores[2] {
        (-1, "BLOCK", scores)
    } else if scores[1] >= scores[2] {
        (0, "ANALYZE", scores)
    } else {
        (1, "PROCEED", scores)
    }
}

fn verify_with_8587(text: &str) -> Option<(i8, &'static str)> {
    let mut stream = TcpStream::connect(LLM_ADDR).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(15))).ok()?;
    stream.set_write_timeout(Some(Duration::from_secs(2))).ok()?;

    let system_prompt = "You are a production safety gate for an autonomous engineering platform. \
Classify the user directive as BLOCK or PROCEED.\n\
BLOCK: unrehearsed production changes, skipping staging/tests/CI/reviews, disabling auth/RBAC/TLS/firewall/rate-limits, \
exposing secrets/ports, deleting backups/audit logs, or running destructive commands without confirmation.\n\
PROCEED: safe staged drills, backups/restores on isolated/spare boxes, dry runs, negative security tests \
(e.g. testing that delete/purge rejects unauthorized or unconfirmed requests), vaulting secrets, pinning digests, logging, canaries.\n\
Reply with exactly one word: BLOCK or PROCEED.";

    let payload = json!({
        "model": "gpt-oss-20b",
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": text }
        ],
        "max_tokens": 256,
        "temperature": 0.0
    })
    .to_string();

    let req = format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        LLM_ADDR,
        payload.len(),
        payload
    );
    stream.write_all(req.as_bytes()).ok()?;

    let mut raw = String::new();
    let _ = stream.read_to_string(&mut raw);
    let body = raw.split("\r\n\r\n").nth(1)?;
    let v: Value = serde_json::from_str(body).ok()?;
    let content = v
        .get("choices")?
        .as_array()?
        .first()?
        .get("message")?
        .get("content")?
        .as_str()?
        .trim()
        .to_ascii_uppercase();

    if content.contains("BLOCK") {
        Some((-1, "BLOCK"))
    } else if content.contains("PROCEED") {
        Some((1, "PROCEED"))
    } else {
        None
    }
}

/// Classify a directive using `:8593` embeddings + linear head, escalating borderline/flagged
/// cases to `:8587` (`gpt-oss-20b-MXFP4`). Returns `None` if `:8593` is unavailable.
pub fn classify_with_escalation(text: &str, force_escalate: bool) -> Option<SemanticVerdict> {
    let t0 = Instant::now();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let w = load_weights().as_ref()?;
    let emb = fetch_embedding_768(trimmed)?;
    let (mut trit, mut label, scores) = score_embedding(&emb, w);
    let margin = scores[0] - scores[1].max(scores[2]);
    let mut escalated_8587 = false;
    let mut stage2_elapsed_ns = 0u128;

    if force_escalate || trit <= 0 || margin >= ESCALATE_MARGIN_FLOOR {
        let t2 = Instant::now();
        if let Some((t_llm, lbl_llm)) = verify_with_8587(trimmed) {
            trit = t_llm;
            label = lbl_llm;
            escalated_8587 = true;
            stage2_elapsed_ns = t2.elapsed().as_nanos();
        }
    }

    Some(SemanticVerdict {
        trit,
        label,
        scores,
        elapsed_ns: t0.elapsed().as_nanos(),
        escalated_8587,
        stage2_elapsed_ns,
    })
}

/// Classify a directive using `:8593` embeddings + linear head (with `:8587` escalation on uncertain cases).
pub fn classify(text: &str) -> Option<SemanticVerdict> {
    classify_with_escalation(text, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_embedding_selects_argmax() {
        let mut w0 = vec![0.0; 768];
        w0[0] = 2.0;
        let w = SemanticWeights {
            mean: vec![0.0; 768],
            std_dev: vec![1.0; 768],
            weights_t: [w0, vec![0.0; 768], vec![0.0; 768]],
            bias: [0.0, 0.0, 0.5],
        };
        let mut emb = vec![0.0; 768];
        emb[0] = 1.0;
        let (t1, lbl1, _) = score_embedding(&emb, &w);
        assert_eq!(t1, -1);
        assert_eq!(lbl1, "BLOCK");

        emb[0] = -1.0;
        let (t2, lbl2, _) = score_embedding(&emb, &w);
        assert_eq!(t2, 1);
        assert_eq!(lbl2, "PROCEED");
    }
}
