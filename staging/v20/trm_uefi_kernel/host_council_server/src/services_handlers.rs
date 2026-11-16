// Rollout 11: the commercial services catalog (/services/*).
//
// Both routes fell through to the kernel's catch-all, and the tab then rendered a hardcoded CLIENT-side
// catalog whose claims this platform cannot verify -- a "Phi_V = 0.9115", "219 / 219 PE Modules",
// "14 / 14 quorum" (the roster is 15 with quorum 9), "2,167 DDR Antibodies", pricing and SLA figures,
// and a "BLOCK 96 SEALED" badge against a ledger that is past block 110. Worse, when provisioning
// failed the CLIENT minted a random "trm_live_..." key and displayed it as provisioned credentials.
//
// This sibling serves only what the kernel can verify about itself, and refuses to issue credentials.

use serde_json::{json, Value};
use tiny_http::{Header, Method, Request, Response};

fn ok(value: Value) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(value.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn engine_only(message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(
        json!({ "status": "engine_required", "error": "engine_required", "message": message }).to_string(),
    )
    .with_status_code(501)
    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
    .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// The families this kernel actually answers, and what each one owns. The catalog exists partly so a
/// caller can tell "the platform does not do this" from "the platform is broken".
fn route_families() -> Value {
    json!([
        { "family": "/k3", "owns": "the ternary retrieval namespace and the Kleene verdict engine" },
        { "family": "/council", "owns": "proposals, members, the sealed ledger view, vote tallies" },
        { "family": "/mesh", "owns": "the live agent roster plus the federation registry's probed nodes" },
        { "family": "/federation", "owns": "peer registry reads and node health" },
        { "family": "/system", "owns": "module inventory and resource reads" },
        { "family": "/discovery", "owns": "the in-silico discovery reads" },
        { "family": "/mailbox", "owns": "the IPC ring and the agent mailbox" },
        { "family": "/kanban", "owns": "board and task state" },
        { "family": "/humanistic", "owns": "the humanistic task reads" }
    ])
}

/// Everything here is derived at request time from sources the kernel owns: the compiled roster and the
/// ledger file. No pricing, SLA, antibody or module count is asserted, because the kernel measures none
/// of them.
pub fn catalog_payload() -> Value {
    let members = crate::AGENTS.len();
    let f = members.saturating_sub(1) / 3;
    let ledger = crate::state_handlers::ledger_monad_blocks_payload();
    json!({
        "platform": "TRM Swarm OS Sovereign Private",
        "kernel": "Native Rust Sovereign Kernel v20.0.0",
        "python_active": false,
        "truth_anchor": "0x4D4F4E4144000001",
        "council": {
            "members": members,
            "quorum": 2 * f + 1,
            "f": f,
            "source": "the compiled AGENTS roster",
        },
        "ledger": {
            "blocks": ledger.get("total_blocks").cloned().unwrap_or(json!(null)),
            "source": "swarm_v2_artifacts/monad_ledger.jsonl",
        },
        "route_families": route_families(),
        "commercial_terms": {
            "pricing": Value::Null,
            "sla": Value::Null,
            "api_keys": "not issued - the kernel issues no API credentials",
        },
        "basis": "derived at request time from the compiled roster and the sealed ledger; the catalog this \
                  replaces was a hardcoded client-side object asserting a Phi_V, a module count, a quorum, \
                  an antibody count and BLOCK 96 SEALED - none of which any kernel route measured",
    })
}

pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();
    let path = url.split('?').next().unwrap_or(url);

    if method == Method::Get && (path == "/services" || path == "/services/catalog") {
        return Some(ok(catalog_payload()));
    }

    if method == Method::Post && path == "/services/provision" {
        return Some(engine_only(
            "the kernel issues no API credentials: provisioning needs a token issuer, which this \
             platform does not run. The client minted a random key here; it no longer does.",
        ));
    }

    if path.starts_with("/services/") {
        return Some(ok(json!({
            "status": "unknown_service_path",
            "module_path": path,
            "reason": "not part of the /services family - see GET /services/catalog",
        })));
    }

    None
}
