// Rollout 9: the Generative Architect studio and the Mobile Sync viewport.
//
// The architect engine (swarm_v2/core/generative_architect.py) and its semantic knowledge graph keep
// their state in process memory and persist nothing, so the reads report that plainly with the fields
// the UI expects left null, and the actions are refused. Mobile Sync is the sharper case: its Python
// routes answer with ASSERTED constants - peer_ip "192.168.1.104", nodes_synced 14, latency_ms 0.18,
// biometric_token "HW_KEY_SECURE_FINGERPRINT_OK" - and this host has no peer, no WiFi transport and no
// biometric device, so none of that is echoed here.

use serde_json::{json, Value};
use tiny_http::{Header, Method, Request, Response};

fn ok(value: Value) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(value.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn engine_only(message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(json!({ "status": "engine_required", "message": message }).to_string())
        .with_status_code(501)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();
    let path = url.split('?').next().unwrap_or(url).trim_end_matches('/');

    if method == Method::Get && path == "/generative-architect/status" {
        return Some(ok(json!({
            "status": "engine_offline",
            "mode": Value::Null,
            "topology_node_count": Value::Null,
            "total_modifications_verified": Value::Null,
            "engine": "swarm_v2/core/generative_architect.py",
            "reason": "the GenerativeArchitect engine builds its topology in process memory and persists \
                       no artifact; this server does not embed it, so no mode, node count or verified \
                       modification count is asserted",
        })));
    }
    if method == Method::Get && path == "/generative-architect/skg" {
        return Some(ok(json!({
            "status": "engine_offline",
            "nodes": [],
            "edges": [],
            "engine": "swarm_v2/core/semantic_knowledge_graph.py",
            "reason": "the semantic knowledge graph lives in the Python engine's memory and no snapshot is \
                       written to disk, so an empty graph carrying its reason is reported rather than a \
                       fabricated topology",
        })));
    }
    if method == Method::Get && path == "/api/mobile-sync/status" {
        return Some(ok(json!({
            "status": "no_peer_transport",
            "peer_ip": Value::Null,
            "wifi_sync_state": Value::Null,
            "biometric_access": Value::Null,
            "dcas9_nodes_active": Value::Null,
            "context_memory_cap": Value::Null,
            "reason": "no mobile peer, WiFi transport or biometric device is attached to this host. The \
                       python route answers with literals (peer_ip 192.168.1.104, wifi_sync_state READY, \
                       biometric AUTHENTICATED, dcas9_nodes_active 14) that nothing measured",
        })));
    }
    if method == Method::Post && path == "/generative-architect/synthesize" {
        return Some(engine_only(
            "/generative-architect/synthesize runs GenerativeArchitect.synthesize_solution over the \
             engine's live topology. That engine is the platform's own (Python, offline); this server \
             does not embed it, so no synthesis is produced rather than a fabricated blueprint.",
        ));
    }
    if method == Method::Post && path == "/generative-architect/propose-self-modification" {
        return Some(engine_only(
            "/generative-architect/propose-self-modification proposes a code change through the \
             architect engine, which verifies it against its own model before returning. Applying or \
             approving a self-modification is that engine's decision, not this server's.",
        ));
    }
    if method == Method::Post && path == "/api/mobile-sync/push-memory" {
        return Some(engine_only(
            "/api/mobile-sync/push-memory pushes the memory graph to a mobile peer over WiFi. No peer \
             transport exists on this host: the python route returns a hardcoded SUCCESS with \
             nodes_synced 14 and never contacts anything, so the push is refused instead of reporting a \
             sync that did not happen.",
        ));
    }
    if method == Method::Post && path == "/api/mobile-sync/biometric-ping" {
        return Some(engine_only(
            "/api/mobile-sync/biometric-ping verifies an on-device biometric token against a hardware \
             key. There is no biometric device attached here and the python route answers with the \
             literal token HW_KEY_SECURE_FINGERPRINT_OK, so verification is refused rather than faked.",
        ));
    }
    None
}
