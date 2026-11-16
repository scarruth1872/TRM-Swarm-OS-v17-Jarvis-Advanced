// /aura/* — split by what is real in this repo vs. a separate product.
//
// Aura Studio (`aipro2`, Quantum Consciousness IDE) is a separate application, not a module of this
// platform: not running on this host (:3000 closed, measured), so /aura/execute, /aura/field, and
// /aura/council-integrate stay refused with a 501 naming that product. Answering {"status":"ok"}
// before made its IDE panels look live.
//
// But the CORE transform — text -> metaconcept (keyword score) -> Kleene K3 gate — is real,
// deterministic, offline code in THIS repo (ported from swarm_v2/core/aura_cognition.py). It has no
// model, no embeddings, and no fabricated telemetry once the voltage/THz fields are dropped. It is
// served here: GET /aura/metaconcepts and POST /aura/cognition/branch return derivable results.

use std::io::Read;

use serde_json::{json, Value};
use tiny_http::{Header, Method, Request, Response};

use crate::aura_metaconcepts::METACONCEPTS;
use crate::aura_transform::branch_cognitive_tree;

fn respond_json(status: u16, v: Value) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(v.to_string())
        .with_status_code(status)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn refuse(path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    respond_json(
        501,
        json!({
            "status": "separate_service",
            "error": "engine_required",
            "message": format!(
                "{path} belongs to {} in Aura Studio, a separate application (`aipro2`), not a module of \
                 this platform. That service is not running on this host (:3000 closed), and none of its \
                 state is stored here, so this kernel cannot serve it.",
                owner_of(path)
            ),
        }),
    )
}

/// What each family of paths belongs to, so the refusal is specific rather than generic.
fn owner_of(path: &str) -> &'static str {
    if path.starts_with("/aura/execute") {
        "the AURA DSL executor"
    } else if path.starts_with("/aura/council-integrate") {
        "council integration for AURA directives"
    } else if path.starts_with("/aura/field") {
        "the consciousness-field state"
    } else {
        "Aura Studio state"
    }
}

/// The 100-concept table, real transform fields only — the fields that carried fabricated
/// substrate numbers (density/frequency/wavefunction/voltage_rail) are omitted, and the note says so.
pub fn metaconcepts_payload() -> Value {
    let mut categories: Vec<String> = Vec::new();
    let items: Vec<Value> = METACONCEPTS
        .iter()
        .map(|m| {
            if !categories.iter().any(|c| c == m.category) {
                categories.push(m.category.to_string());
            }
            json!({
                "key": m.key,
                "name": m.name,
                "category": m.category,
                "polarity": m.polarity,
                "description": m.description,
                "intent": m.intent,
                "resonance_links": m.links,
            })
        })
        .collect();
    json!({
        "status": "ok",
        "count": items.len(),
        "categories": categories,
        "metaconcepts": items,
        "honest_note": "real transform fields only; density/frequency/wavefunction/voltage_rail omitted as they carried fabricated substrate numbers",
    })
}

pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let path = url.split('?').next().unwrap_or(url);
    if !path.starts_with("/aura") {
        return None;
    }
    let method = request.method().clone();

    // GET /aura/metaconcepts — the 100-concept table, real fields only.
    if method == Method::Get && path == "/aura/metaconcepts" {
        return Some(respond_json(200, metaconcepts_payload()));
    }

    // POST /aura/cognition/branch — the real transform: directive -> metaconcept -> K3 verdict.
    if method == Method::Post && path == "/aura/cognition/branch" {
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let v: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let directive = v.get("directive").and_then(|d| d.as_str()).unwrap_or("");
        let seeds: Vec<String> = v
            .get("seed_concepts")
            .and_then(|s| s.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let depth = v.get("depth").and_then(|d| d.as_u64()).unwrap_or(2).min(4) as usize;
        if directive.is_empty() {
            return Some(respond_json(400, json!({
                "status": "error",
                "error": "missing_directive",
                "detail": "POST {\"directive\": \"...\", \"seed_concepts\": [...], \"depth\": 2}",
            })));
        }
        return Some(respond_json(200, branch_cognitive_tree(directive, &seeds, depth)));
    }

    // Everything else in /aura belongs to the separate IDE product.
    Some(refuse(path))
}
