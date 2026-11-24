//! In-process evidence retrieval — the last Python dependency in the decision path, removed.
//!
//! WHY
//! ---
//! `local_inference::evidence_for` used to fetch documents from the Python Council API on
//! :8099. Measured cost of that dependency (same directive, twice each way):
//!
//!     :8099 UP    "cover the watchdog with a regression test"  -> CONSENSUS  A12/R1  (x2)
//!     :8099 DOWN  same directive                               -> REJECTED   A1/R13 (x2)
//!
//! So the council's verdict depended on a Python process reached over HTTP — against the
//! platform's own doctrine — while `/council/propose` reported `evidence_used: 0`, hiding it.
//!
//! HOW
//! ---
//! The Python retriever already persists `data/council_evidence_index.json`: 558 paragraph
//! chunks from 105 files, each with its 768-dim vector. That file is reused as-is, so nothing
//! is re-embedded at startup and the ranking is identical by construction:
//!
//!   * query embedded with nomic-embed-text on :8593 (one vector, same model)
//!   * cosine over the stored vectors, floor 0.30, top-k — same numbers as `retrieve()`
//!
//! An empty result still means "no evidence found", never "evidence supports this".

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::OnceLock;
use std::time::Duration;

const EMB_HOST: &str = "127.0.0.1";
const EMB_PORT: u16 = 8593;
/// Same floor as swarm_v2/core/evidence_retriever.py::retrieve(min_score=0.30).
const MIN_SCORE: f32 = 0.30;

struct Chunk {
    source: String,
    text: String,
    vec: Vec<f32>,
}

static INDEX: OnceLock<Vec<Chunk>> = OnceLock::new();

fn index_path() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("TRM_EVIDENCE_INDEX") {
        let pb = std::path::PathBuf::from(p);
        return pb.exists().then_some(pb);
    }
    // The council binary runs with its own crate directory as cwd, so walk upward.
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..5 {
        let candidate = dir.join("data").join("council_evidence_index.json");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn load() -> Vec<Chunk> {
    let path = match index_path() {
        Some(p) => p,
        None => {
            eprintln!("[evidence] index not found — retrieval disabled, verdicts will run without citations");
            return Vec::new();
        }
    };
    let raw = match std::fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[evidence] cannot read {}: {}", path.display(), e);
            return Vec::new();
        }
    };
    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[evidence] index is not valid JSON: {}", e);
            return Vec::new();
        }
    };
    let arr = match v.get("chunks").and_then(|c| c.as_array()) {
        Some(a) => a,
        None => return Vec::new(),
    };
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let source = item.get("source").and_then(|x| x.as_str()).unwrap_or("?").to_string();
        let text = item.get("text").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let vec: Vec<f32> = match item.get("vec").and_then(|x| x.as_array()) {
            Some(a) => a.iter().filter_map(|n| n.as_f64().map(|f| f as f32)).collect(),
            None => continue,
        };
        if vec.is_empty() || text.is_empty() {
            continue;
        }
        out.push(Chunk { source, text, vec });
    }
    eprintln!("[evidence] native index loaded: {} chunks from {}", out.len(), path.display());
    out
}

fn chunks() -> &'static Vec<Chunk> {
    INDEX.get_or_init(load)
}

fn json_escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push(' '),
            c => o.push(c),
        }
    }
    o
}

/// Embed one query with the same model the index was built with.
fn embed_query(query: &str) -> Option<Vec<f32>> {
    let addr = format!("{}:{}", EMB_HOST, EMB_PORT);
    let mut s = TcpStream::connect(&addr).ok()?;
    let _ = s.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = s.set_write_timeout(Some(Duration::from_secs(10)));
    let body = format!("{{\"input\":[\"{}\"]}}", json_escape(query));
    let req = format!(
        "POST /embeddings HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        addr,
        body.len(),
        body
    );
    s.write_all(req.as_bytes()).ok()?;
    let mut raw = String::new();
    let _ = s.read_to_string(&mut raw);

    let body_start = raw.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = &raw[body_start..];
    let (a, b) = match (body.find('['), body.rfind(']')) {
        (Some(a), Some(b)) if b > a => (a, b + 1),
        _ => return None,
    };
    let v: serde_json::Value = serde_json::from_str(&body[a..b]).ok()?;
    let first = v.get(0)?;
    let mut emb = first.get("embedding")?;
    // llama-server nests the vector: [[768 floats]]
    while emb.as_array().map(|x| x.len() == 1).unwrap_or(false)
        && emb.get(0).map(|x| x.is_array()).unwrap_or(false)
    {
        emb = emb.get(0)?;
    }
    let out: Vec<f32> = emb
        .as_array()?
        .iter()
        .filter_map(|n| n.as_f64().map(|f| f as f32))
        .collect();
    (!out.is_empty()).then_some(out)
}

/// Public wrapper so the continuum path shares the exact same query embedding.
/// Duplicating the llama-server request shape in a second module is how two retrievers drift apart.
pub fn embed_query_pub(query: &str) -> Option<Vec<f32>> {
    embed_query(query)
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// Compact "[source] snippet" block for the top-k chunks.
/// Empty string means retrieval is unavailable or found nothing — callers must treat that as
/// "no evidence", never as "evidence supports this".
/// Which ranker answers: "continuum" (integer facet dot over the K3CX artefact) by default when it is
/// loaded, "f32" to force the previous path. Measured before switching: the continuum keeps 0.944
/// recall@10 of the f32 ranking at 4.0x less memory and served 4/4 platform facts (docs/TRM_Continuum_Rust_2026-11-24.md).
/// Public name of the ranker that would answer right now ("continuum" or "f32"), so the proposal
/// record can state which one produced its citations instead of leaving a reader to infer it.
pub fn ranker_name() -> &'static str {
    if ranker().eq_ignore_ascii_case("f32") {
        "f32"
    } else if crate::continuum::loaded().is_some() {
        "continuum(i8 facets, i32 dot)"
    } else {
        "f32(fallback: no continuum)"
    }
}

fn ranker() -> &'static str {
    std::env::var("TRM_EVIDENCE_RANKER")
        .map(|s| Box::leak(s.into_boxed_str()) as &'static str)
        .unwrap_or("continuum")
}

pub fn evidence_for(query: &str, k: usize) -> String {
    // Caught by `empty_query_returns_no_evidence` after the ranker swap: the embedder happily embeds the
    // empty string, so without this guard an empty query returned a full evidence block from whichever
    // ranker ran first. An empty query must mean "no evidence", never "here is the nearest thing".
    if query.trim().is_empty() {
        return String::new();
    }
    // The continuum path is tried first; an empty result from it is NOT treated as "no evidence" unless
    // it actually answered, so a missing artefact falls through to the f32 index rather than silently
    // blanking every citation.
    if !ranker().eq_ignore_ascii_case("f32") {
        if let Some(c) = crate::continuum::loaded() {
            if let Some(block) = c.evidence_for(query, k, MIN_SCORE) {
                if !block.trim().is_empty() {
                    return block;
                }
            }
        }
    }
    let cs = chunks();
    if cs.is_empty() || query.trim().is_empty() {
        return String::new();
    }
    let qv = match embed_query(query) {
        Some(v) => v,
        None => return String::new(),
    };
    let mut scored: Vec<(f32, &Chunk)> = cs
        .iter()
        .map(|c| (cosine(&qv, &c.vec), c))
        .filter(|(s, _)| *s >= MIN_SCORE)
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut out = String::new();
    for (_, c) in scored.into_iter().take(k.max(1)) {
        let snip: String = c.text.chars().take(240).collect();
        out.push_str(&format!("[{}] {}\n", c.source, snip.replace('\n', " ")));
    }
    out
}

/// Number of indexed chunks, for health reporting.
pub fn index_size() -> usize {
    chunks().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_is_exact_for_identical_vectors() {
        let v = vec![0.5f32, -0.25, 0.75];
        assert!((cosine(&v, &v) - 1.0).abs() < 1e-6);
        let w = vec![-0.5f32, 0.25, -0.75];
        assert!((cosine(&v, &w) + 1.0).abs() < 1e-6);
        assert_eq!(cosine(&v, &[0.0, 0.0, 0.0]), 0.0);
        assert_eq!(cosine(&v, &[1.0]), 0.0);
    }

    #[test]
    fn empty_query_returns_no_evidence() {
        assert_eq!(evidence_for("", 2), "");
    }
}
