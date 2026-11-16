// Rollout 9 follow-up: POST /generative-architect/synthesize.
//
// The Python engine's `GenerativeArchitect.synthesize_solution(target, goal)` builds a blueprint by
// calling its own in-process semantic knowledge graph, voltage binder and microkernel spawner, then
// returns generated prose posing as an architectural answer. That engine is the platform's own and
// is offline; this kernel does not embed it, so `studio_handlers` refused the route (501) rather
// than fabricate a blueprint.
//
// This module replaces that refusal with a REAL, deterministic synthesis the kernel can produce
// from its own machinery: it recalls over the platform's sealed local knowledge stores (fabric +
// structured text + clinical catalog) through `recall_router::fused_lookup_text` — a reciprocal-rank
// fusion over file-backed, quantized stores with no model sampled anywhere — and returns the top
// matched documents with their measured scores and source paths, labelled "derived from local
// corpus, no LLM". Every synthesis is appended to swarm_v2_artifacts/architect_synthesis.jsonl.
//
// Nothing here writes text that was not read out of a real local document. A prompt that matches
// nothing returns an empty top_matches list with its basis named, never a score nobody computed.
//
// WIRING (performed in main.rs, not here — this file is added alongside, not in place of, the
// existing handler): declare `mod architect_actions;` next to `mod studio_handlers;`, then in the
// Rollout 9 arm call `architect_actions::handle(&mut request, &url)` BEFORE `studio_handlers::handle`.
// This handler claims only POST /generative-architect/synthesize and returns None for every other
// path, so the GET /status and /skg reads and the other engine-only POSTs keep resolving in
// studio_handlers.

use serde_json::{json, Value};
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response};

type Resp = Response<std::io::Cursor<Vec<u8>>>;

/// JSON 200 with the shared CORS + content headers, per the crate's handler convention.
fn ok(value: Value) -> Resp {
    Response::from_string(value.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// JSON with a non-200 status, same headers. Used only for a malformed request (missing prompt),
/// never to substitute a fabricated answer for a failed recall.
fn err(status: u16, value: Value) -> Resp {
    Response::from_string(value.to_string())
        .with_status_code(status)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// The source path a fused result carries. Fabric and structured hits name their document under
/// `doc` / `key` respectively; the clinical tier is a tabular measurement, so its `doc` is the
/// measurement code rather than a file path — that distinction is preserved, not flattened.
fn match_source(item: &Value) -> String {
    item.get("doc")
        .and_then(|v| v.as_str())
        .or_else(|| item.get("key").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string()
}

/// The measured rank score for a fused result (packed-domain cosine for the vector stores, 1.0 for
/// the lexical clinical exact match). Reported exactly as the recall tier computed it.
fn match_score(item: &Value) -> f64 {
    item.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0)
}

/// ISO-8601 UTC timestamp (same family as Python isoformat), derived from the system clock.
fn iso_now() -> String {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|x| x.as_secs())
        .unwrap_or(0);
    let days = (d / 86400) as i64;
    let secs_of_day = d % 86400;
    let (y, m, day) = civil_from_days(days);
    let (hh, mm, ss) = (
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    );
    format!("{y:04}-{m:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}")
}

/// Howard Hinnant's civil_from_days algorithm (public domain).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as i64, d as i64)
}

/// Append one synthesis record to swarm_v2_artifacts/architect_synthesis.jsonl. Returns the store
/// path and the bytes written, or None when the store could not be opened/written — a failed write
/// is reported, never silently treated as a successful one.
fn persist_synthesis(entry: &Value) -> Option<(String, usize)> {
    let root = crate::k3_retrieval::repo_root()?;
    let path = root.join("swarm_v2_artifacts").join("architect_synthesis.jsonl");
    let line = serde_json::to_string(entry).ok()?;
    let bytes = line.len();
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&path).ok()?;
    writeln!(f, "{line}").ok()?;
    Some((path.display().to_string(), bytes))
}

/// POST /generative-architect/synthesize — a deterministic synthesis from the platform's own local
/// corpus. Claims the route and returns Some(response); returns None for every other request.
pub fn handle(request: &mut Request, url: &str) -> Option<Resp> {
    let method = request.method().clone();
    let path = url.split('?').next().unwrap_or(url).trim_end_matches('/');
    if !(method == Method::Post && path == "/generative-architect/synthesize") {
        return None;
    }

    let parsed: Value = crate::chat_handlers::read_json_or_form(request, 65_536);
    let prompt = parsed
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if prompt.is_empty() {
        return Some(err(
            400,
            json!({
                "status": "error",
                "route": "/generative-architect/synthesize",
                "error": "missing_prompt",
                "detail": "POST {\"prompt\": \"...\"} with a non-empty prompt",
            }),
        ));
    }

    // Real recall over the sealed local knowledge stores. fused_lookup_text is the general-recall
    // surface (fabric + structured text + clinical), fully file-backed and deterministic: the scores
    // are packed-domain cosine / RRF, no model is sampled, and no text is generated.
    let fused = crate::recall_router::fused_lookup_text(&prompt);

    let merged = fused
        .get("merged")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();
    let stores_searched: Vec<Value> = fused
        .get("stores_searched")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();
    let fusion_desc = fused
        .get("fusion")
        .and_then(|s| s.as_str())
        .unwrap_or("reciprocal-rank-fusion (k=60) with cosine tie-break, over fabric + structured + clinical")
        .to_string();

    let top_matches: Vec<Value> = merged
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let source = match_source(item);
            let mut m = json!({
                "rank": i + 1,
                "source": source,
                "score": match_score(item),
                "store": item.get("store").and_then(|v| v.as_str()).unwrap_or(""),
                "rrf": item.get("rrf").and_then(|v| v.as_f64()).unwrap_or(0.0),
            });
            if let Some(basis) = item.get("basis").and_then(|v| v.as_str()) {
                m["basis"] = json!(basis);
            }
            m
        })
        .collect();

    let result_count = top_matches.len();
    let timestamp = iso_now();

    let persisted_entry = json!({
        "timestamp": timestamp,
        "route": "/generative-architect/synthesize",
        "prompt": prompt,
        "derived_from": "local corpus, no LLM",
        "result_count": result_count,
        "stores_searched": stores_searched,
        "top_matches": top_matches,
    });
    let persisted = persist_synthesis(&persisted_entry);

    Some(ok(json!({
        "status": "ok",
        "route": "/generative-architect/synthesize",
        "derived_from": "local corpus, no LLM",
        "doctrine": "the synthesis is the top-ranked recall over the platform's own sealed knowledge; no LLM was sampled to write it",
        "prompt": prompt,
        "result_count": result_count,
        "top_matches": top_matches,
        "stores_searched": stores_searched,
        "fusion": fusion_desc,
        "corpus_root": crate::k3_retrieval::repo_root().map(|p| p.display().to_string()),
        "no_match_note": if result_count == 0 {
            Some("no local corpus document matched the prompt; the recall ran and returned no hits, so nothing is fabricated")
        } else { None },
        "persisted": persisted
            .as_ref()
            .map(|(path, bytes)| json!({ "path": path, "line_bytes": bytes }))
            .unwrap_or(Value::Null),
        "timestamp": timestamp,
    })))
}
