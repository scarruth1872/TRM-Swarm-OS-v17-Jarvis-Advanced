//! continuum_refract.rs — native Rust handler for `POST /api/continuum/genomics/refract`.
//!
//! The Python route (`swarm_v2/routes/swarm.py::refract_prompt_continuum`) refracts a prompt
//! through `ContinuumGenomicsEngine`'s rolling 4D momentum cache — an IN-MEMORY rolling average of
//! persona gene vectors with no persistence path. The kernel does not own that cache, so a literal
//! port would either fabricate a momentum or report `None`. Instead, this handler derives a REAL
//! deterministic refraction from what the kernel CAN measure on this host:
//!
//!   1. a REAL recall over the local knowledge base — first the kernel's ternary fabric
//!      (`k3_retrieval::fabric_lookup`, lexically-confirmed, integer agreement score), then the
//!      kernel's packed-domain fabric (`k3_retrieval::handle("/k3/retrieve")`, the store that is
//!      primary where the exact JSON index exceeds the load limit), then a bounded lexical
//!      keyword/substring scan over the real `swarm_v2_artifacts/research/*.md` and `papers/*.md`
//!      files — so the top knowledge hit and its score are computed, never invented;
//!   2. a REAL star-matrix node selection: the node whose 4D spectrum best matches the persona's
//!      own 4D gene vector by cosine similarity, read from the same lattice `state_handlers`
//!      serves at `/api/continuum/star-matrix/lattice`.
//!
//! The resulting refraction (top knowledge + best-matching node + score + source path) is labelled
//! `derived, no LLM` and appended to `swarm_v2_artifacts/continuum_refractions.jsonl`. No LLM runs
//! on this host; every number in the response is a measurement or a derived value.

use serde_json::{json, Value};
use std::io::Read;
use tiny_http::{Header, Method, Request, Response};

type Resp = Response<std::io::Cursor<Vec<u8>>>;

/// `PERSONA_PRESETS` from `swarm_v2/core/continuum_genomics.py`, mirrored verbatim:
/// (key, name, voice, [plasticity, depth, empathy, stochasticity]).
const PERSONAS: [(&str, &str, &str, [f64; 4]); 4] = [
    ("continuum", "Continuum Core", "Kore", [0.75, 0.80, 0.70, 0.50]),
    ("sage", "Cybernetic Sage", "Schedar", [0.35, 0.98, 0.40, 0.15]),
    ("muse", "Stochastic Muse", "Puck", [0.95, 0.55, 0.90, 0.98]),
    ("sentinel", "Sentinel Warden", "Zephyr", [0.15, 0.85, 0.20, 0.05]),
];

/// Resolve a persona key to (name, voice, genes). Unknown keys fall back to `continuum`, exactly as
/// `ContinuumGenomicsEngine::set_active_persona` does in the Python module.
fn persona(key: &str) -> (&'static str, &'static str, [f64; 4]) {
    for (k, name, voice, genes) in PERSONAS.iter() {
        if *k == key {
            return (*name, *voice, *genes);
        }
    }
    ("Continuum Core", "Kore", [0.75, 0.80, 0.70, 0.50])
}

/// Cosine similarity between two 4D vectors. Deterministic, no model.
fn cosine4(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let dot: f64 = (0..4).map(|i| a[i] * b[i]).sum();
    let na = (0..4).map(|i| a[i] * a[i]).sum::<f64>().sqrt();
    let nb = (0..4).map(|i| b[i] * b[i]).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

/// Select the star-matrix node whose 4D spectrum best matches the persona's gene vector, by cosine
/// similarity. The nodes are read from the same `star_matrix_lattice_payload()` the lattice route
/// serves, so the match is against the identical declared table — a real numeric selection.
fn best_star_node(genes: &[f64; 4]) -> Option<Value> {
    let lattice = crate::state_handlers::star_matrix_lattice_payload();
    let nodes = lattice.get("lattice")?.get("nodes")?.as_object()?;
    let mut best: Option<(f64, String, Value)> = None;
    for (label, node) in nodes {
        let spec = match node.get("spectrum").and_then(|s| s.as_array()) {
            Some(s) if s.len() == 4 => s,
            _ => continue,
        };
        let mut s = [0.0f64; 4];
        for (i, v) in spec.iter().enumerate() {
            s[i] = v.as_f64().unwrap_or(0.0);
        }
        let c = cosine4(genes, &s);
        if best.as_ref().map(|(bc, ..)| c > *bc).unwrap_or(true) {
            best = Some((c, label.clone(), node.clone()));
        }
    }
    best.map(|(c, label, node)| {
        json!({
            "node": label,
            "key": node.get("key").cloned().unwrap_or(Value::Null),
            "name": node.get("name").cloned().unwrap_or(Value::Null),
            "spectrum": node.get("spectrum").cloned().unwrap_or(Value::Null),
            "cosine_to_persona_genes": (c * 10_000.0).round() / 10_000.0,
        })
    })
}

/// Content words of a prompt: lowercase alphanumeric tokens of 4+ chars, stopwords dropped. Used by
/// the lexical fallback scan, so its matching is a real keyword/substring test, not a model.
fn content_terms(text: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "what", "which", "when", "where", "with", "that", "this", "from", "have", "does", "your",
        "about", "into", "they", "them", "than", "then", "there", "these", "those", "will", "would",
        "should", "could", "explain", "please", "tell", "give", "make", "made", "using", "used",
        "many", "much", "refract", "refraction", "prompt", "continuum", "genomics",
    ];
    let mut out: Vec<String> = Vec::new();
    for tok in text.split(|c: char| !c.is_alphanumeric()) {
        let t = tok.to_lowercase();
        if t.len() >= 4 && !STOP.contains(&t.as_str()) && !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

/// First 320 chars of a document, for a bounded snippet.
fn head(text: &str) -> String {
    let h: String = text.chars().take(320).collect();
    if text.chars().count() > 320 {
        format!("{h}…")
    } else {
        h
    }
}

/// Best-effort snippet for a fabric doc key, walking the same repo-root prefixes the k3 engine uses.
fn read_doc_snippet(key: &str) -> Option<String> {
    for prefix in ["", "swarm_v2_artifacts/", "docs/", "swarm_v2/papers/"] {
        if let Some(p) = crate::k3_retrieval::repo_relative(&format!("{prefix}{key}")) {
            if let Ok(bytes) = std::fs::read(&p) {
                let cut = bytes.len().min(64_000);
                let text = String::from_utf8_lossy(&bytes[..cut]).to_string();
                return Some(head(&text));
            }
        }
    }
    None
}

/// Real recall over the local knowledge base. Priority order, each with a real numeric score:
///   1. the kernel's ternary fabric (`fabric_lookup` — integer agreement, lexically confirmed);
///   2. the kernel's packed-domain fabric (`/k3/retrieve` — packed 2-bit cosine, primary store);
///   3. a bounded lexical keyword scan over the real research/papers markdown files.
fn recall(prompt: &str) -> Value {
    // 1. Ternary fabric. On a host whose exact JSON index exceeds the exact-ternary load limit this
    //    returns None immediately (no full read) and we fall through to the packed store.
    if let Some(hit) = crate::k3_retrieval::fabric_lookup(prompt, 3) {
        return json!({
            "engine": "k3 ternary fabric (engine agreement, lexically confirmed)",
            "found": true,
            "doc": hit.doc,
            "score": hit.score,
            "matched_terms": hit.matched_terms,
            "path_hits": hit.path_hits,
            "snippet": hit.snippet,
            "ranker": hit.ranker,
            "candidates_proposed": hit.candidates,
            "corpus_documents": hit.documents,
            "basis": "kernel ternary agreement score over the quantized corpus, confirmed by lexical term match",
        });
    }

    // 2. Packed-domain fabric — the store that IS primary on this host.
    let q = prompt
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace(' ', "%20");
    let (payload, code) = crate::k3_retrieval::handle(&format!("/k3/retrieve?q={q}"));
    if code == 200 {
        if let Some(results) = payload.get("results").and_then(|r| r.as_array()) {
            if let Some(top) = results.first() {
                let doc = top
                    .get("doc")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                let snippet = if doc.is_empty() {
                    None
                } else {
                    read_doc_snippet(&doc)
                };
                return json!({
                    "engine": "k3 packed-domain fabric (native-turbovec-hdc)",
                    "found": true,
                    "doc": doc,
                    "score": top.get("score").cloned().unwrap_or(Value::Null),
                    "rank": top.get("rank").cloned().unwrap_or(Value::Null),
                    "snippet": snippet.unwrap_or_default(),
                    "results_returned": results.len(),
                    "scored_documents": payload.get("scored_documents").cloned().unwrap_or(Value::Null),
                    "encoder": payload.get("encoder").cloned().unwrap_or(Value::Null),
                    "basis": "top hit from /k3/retrieve (packed 2-bit cosine over swarm_v2_nand_fabric.turbovec)",
                });
            }
        }
    }

    // 3. Last resort: a real lexical scan over the actual markdown research/papers files.
    fallback_file_recall(prompt)
}

/// Lexical keyword/substring recall over `swarm_v2_artifacts/research/*.md` and `papers/*.md`.
/// Bounded (CAP files), and the bound is reported so a capped scan cannot masquerade as exhaustive.
fn fallback_file_recall(prompt: &str) -> Value {
    let terms = content_terms(prompt);
    if terms.is_empty() {
        return json!({
            "engine": "none",
            "found": false,
            "score": 0,
            "basis": "no content terms in the prompt and no kernel recall available; nothing fabricated",
        });
    }
    let root = match crate::k3_retrieval::repo_root() {
        Some(r) => r,
        None => {
            return json!({
                "engine": "none",
                "found": false,
                "score": 0,
                "basis": "repo root unresolved",
            })
        }
    };
    let dirs = [
        root.join("swarm_v2_artifacts/research"),
        root.join("papers"),
    ];
    const CAP: usize = 4096;
    let mut best: Option<(usize, String, String)> = None;
    let mut scanned = 0usize;
    'outer: for dir in dirs.iter() {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        for ent in entries.flatten() {
            if scanned >= CAP {
                break 'outer;
            }
            let p = ent.path();
            if p.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            scanned += 1;
            let lower = text.to_lowercase();
            let hits = terms.iter().filter(|t| lower.contains(t.as_str())).count();
            if hits > 0 && best.as_ref().map(|(b, ..)| hits > *b).unwrap_or(true) {
                let rel = p
                    .strip_prefix(&root)
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string()
                    });
                best = Some((hits, rel, text));
            }
        }
    }
    match best {
        Some((hits, rel, text)) => {
            let matched: Vec<String> = terms
                .iter()
                .filter(|t| text.to_lowercase().contains(t.as_str()))
                .cloned()
                .collect();
            json!({
                "engine": "lexical file scan",
                "found": true,
                "doc": rel,
                "score": hits,
                "matched_terms": matched,
                "snippet": head(&text),
                "files_scanned": scanned,
                "capped": scanned >= CAP,
                "basis": "keyword/substring match over real swarm_v2_artifacts/research/*.md and papers/*.md",
            })
        }
        None => json!({
            "engine": "lexical file scan",
            "found": false,
            "score": 0,
            "files_scanned": scanned,
            "capped": scanned >= CAP,
            "basis": "no keyword match in the scanned research/papers files",
        }),
    }
}

/// Append one refraction record to `swarm_v2_artifacts/continuum_refractions.jsonl`.
fn persist(prompt: &str, persona_key: &str, refraction: &Value) -> Value {
    let root = match crate::k3_retrieval::repo_root() {
        Some(r) => r,
        None => return json!({ "status": "error", "error": "repo_root_unresolved" }),
    };
    let path = root.join("swarm_v2_artifacts").join("continuum_refractions.jsonl");
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let record = json!({
        "timestamp_unix": secs,
        "route": "/api/continuum/genomics/refract",
        "prompt": prompt,
        "persona": persona_key,
        "refraction": refraction,
    });
    use std::io::Write;
    let opened = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path);
    match opened.and_then(|mut f| {
        writeln!(
            f,
            "{}",
            serde_json::to_string(&record).unwrap_or_else(|_| "{}".into())
        )
    }) {
        Ok(()) => json!({ "status": "ok", "path": path.display().to_string() }),
        Err(e) => json!({
            "status": "error",
            "error": format!("{e}"),
            "path": path.display().to_string(),
        }),
    }
}

/// Build a JSON response with the shared headers (Content-Type + CORS).
fn json_response(value: Value, status: u16) -> Resp {
    let body = serde_json::to_string(&value).unwrap_or_else(|_| "{}".into());
    Response::from_string(body)
        .with_status_code(status)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(
            Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
        )
}

/// Claim `POST /api/continuum/genomics/refract`; return `None` for any other route or method.
pub fn handle(request: &mut Request, url: &str) -> Option<Resp> {
    if request.method() != &Method::Post {
        return None;
    }
    if !url.starts_with("/api/continuum/genomics/refract") {
        return None;
    }

    let parsed: Value = crate::chat_handlers::read_json_or_form(request, 1_048_576);
    let prompt = parsed
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let persona_key = parsed
        .get("persona")
        .and_then(|v| v.as_str())
        .unwrap_or("continuum")
        .trim()
        .to_string();

    if prompt.is_empty() {
        return Some(json_response(
            json!({
                "status": "error",
                "route": "/api/continuum/genomics/refract",
                "error": "missing_prompt",
                "detail": "POST {\"prompt\": \"...\", \"persona\": \"continuum\"} with a non-empty prompt",
            }),
            400,
        ));
    }

    let (pname, pvoice, pgenes) = persona(&persona_key);
    let node = best_star_node(&pgenes);
    let knowledge = recall(&prompt);

    let refraction = json!({
        "knowledge": knowledge,
        "star_matrix_node": node.unwrap_or(Value::Null),
        "score": knowledge.get("score").cloned().unwrap_or(Value::Null),
        "source_path": knowledge.get("doc").cloned().unwrap_or(Value::Null),
        "label": "derived, no LLM",
        "basis": "deterministic refraction: top knowledge hit from a real local recall (score = that hit's own score) plus the star-matrix node whose 4D spectrum best matches the persona's gene vector by cosine similarity",
    });

    let payload = json!({
        "status": "SUCCESS",
        "route": "/api/continuum/genomics/refract",
        "prompt": prompt,
        "persona": {
            "key": persona_key,
            "name": pname,
            "voice": pvoice,
            "genes": {
                "plasticity": pgenes[0],
                "depth": pgenes[1],
                "empathy": pgenes[2],
                "stochasticity": pgenes[3],
            },
        },
        "refraction": refraction,
        "persisted": persist(&prompt, &persona_key, &refraction),
    });

    Some(json_response(payload, 200))
}
