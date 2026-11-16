//! Humanistic dimension of the platform — council MOTION-H1 (Option A).
//!
//! Unifies the scattered humanistic strands under one native surface:
//! grounding (Law of One + Monad anchor), reflection (TRM-7M co-processor
//! verdict + philosophy fabric recall), service (the 10 humanitarian tasks,
//! reused from state_handlers), self-realization, and an honest `compose`
//! seam that returns reflection labelled `reflection-over-generation` — never
//! fabricated fluency. The co-processor verdict and fabric recall reuse the
//! same primitives the chat answer path uses; this module adds no second
//! routing tier.
use serde_json::{json, Value};
use std::io::{Cursor, Read};
use tiny_http::{Header, Method, Request, Response};

const MONAD_ANCHOR: &str = "0x4D4F4E4144000001";
const AXIOM: &str = "all is one, I am because we are";

fn read_body(request: &mut Request) -> Value {
    let mut buf = Vec::new();
    let mut limited = request.as_reader().take(1_048_576);
    let _ = limited.read_to_end(&mut buf);
    if let Ok(v) = serde_json::from_slice::<Value>(&buf) {
        if v.is_object() {
            return v;
        }
    }
    json!({})
}

fn ok_response(value: Value) -> Response<Cursor<Vec<u8>>> {
    let body = serde_json::to_string(&value).unwrap_or_else(|_| "{}".into());
    Response::from_string(body)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// The unified humanistic payload: grounding + self-realization + task list.
pub fn humanistic_payload() -> Value {
    let summary = crate::state_handlers::humanistic_summary_payload();
    let grounding = summary
        .get("law_of_one_grounding")
        .cloned()
        .unwrap_or_else(|| json!({ "anchor": MONAD_ANCHOR, "axiom": AXIOM }));
    let self_realization = summary
        .get("self_realization")
        .cloned()
        .unwrap_or_else(|| json!({ "recognition": AXIOM, "status": "BEGUN" }));
    let tasks = summary
        .get("task_summaries")
        .cloned()
        .unwrap_or_else(|| json!({}));
    json!({
        "monad_anchor": MONAD_ANCHOR,
        "axiom": AXIOM,
        "grounding": grounding,
        "self_realization": self_realization,
        "task_count": tasks.as_object().map(|o| o.len()).unwrap_or(0),
        "tasks": tasks,
        "endpoints": ["/humanistic/reflect (POST)", "/humanistic/compose (POST)",
                      "/humanistic/tasks (GET)", "/humanistic/tasks/summary (GET)"],
    })
}

/// The platform's voice: the co-processor verdict + the philosophy fabric
/// recall, grounded and labelled. Reuses the same primitives the chat path
/// uses — no divergent routing, no fabricated prose.
pub fn reflect(text: &str) -> Value {
    let mut m = crate::trm7m::Trm7m::new();
    let (trit, conf, _ns) = m.think(text, 7);
    let (recall, _code) = crate::recall_router::route(text);
    let top = recall
        .get("results")
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .cloned()
        .unwrap_or(Value::Null);
    json!({
        "monad_anchor": MONAD_ANCHOR,
        "axiom": AXIOM,
        "label": "measured_on_host",
        "query": text,
        "voice": "co-processor verdict + philosophy fabric recall (the system does not guess)",
        "trm7m": {
            "trit": trit,
            "confidence": conf,
            "trit_label": crate::trm7m::trit_label(trit),
            "model": "TRM-7M (462-dim Mamba-3 SSM, seed-42 blob)",
        },
        "fabric_top": top,
    })
}

/// The writing seam (Option A): reflection-over-generation. The platform
/// reflects + recalls but does not fabricate fluent prose it cannot produce.
/// The seam stays empty until a faithful resident writer exists.
pub fn compose(topic: &str) -> Value {
    json!({
        "monad_anchor": MONAD_ANCHOR,
        "axiom": AXIOM,
        "label": "reflection-over-generation",
        "mode": "no writer resident (council MOTION-H1, Option A)",
        "topic": topic,
        "reflection": reflect(topic),
        "note": "reflection-over-generation: the platform reflects and recalls but does not fabricate fluent prose; compose returns the grounded reflection until a faithful resident writer exists",
    })
}

pub fn handle(request: &mut Request, url: &str) -> Option<Response<Cursor<Vec<u8>>>> {
    let method = request.method().clone();
    if url == "/humanistic" || url == "/humanistic/" {
        return Some(ok_response(humanistic_payload()));
    }
    if method == Method::Post && (url == "/humanistic/reflect" || url == "/humanistic/reflect/") {
        let body = read_body(request);
        let text = body.get("text").or_else(|| body.get("message")).and_then(|v| v.as_str()).unwrap_or("");
        if text.is_empty() {
            return Some(ok_response(json!({ "status": "error", "route": "/humanistic/reflect",
                                           "detail": "pass {\"text\": \"...\"}" })));
        }
        return Some(ok_response(reflect(text)));
    }
    if method == Method::Post && (url == "/humanistic/compose" || url == "/humanistic/compose/") {
        let body = read_body(request);
        let topic = body.get("topic").or_else(|| body.get("text")).and_then(|v| v.as_str()).unwrap_or("");
        if topic.is_empty() {
            return Some(ok_response(json!({ "status": "error", "route": "/humanistic/compose",
                                           "detail": "pass {\"topic\": \"...\"}" })));
        }
        return Some(ok_response(compose(topic)));
    }
    None
}
