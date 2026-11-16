// Rollout 10: the P2P mesh family.
//
// The kernel answered EVERY /mesh/<anything> path with one hardcoded topology - two nodes ("Jarvis OS
// Gateway", "Antigravity Sovereign Microkernel") carrying invented ping times (0.12 / 0.08 ms) and a
// 0.10 ms link - including /mesh/does-not-exist. Nothing on this host measured those numbers. This
// sibling answers per path from real sources (the live agent roster, the federation registry) and
// refuses the actions, so no path in the family can present a fabricated mesh.

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

/// The live roster as mesh nodes. No latency field: this host measures no inter-node RTT.
fn roster_nodes() -> Vec<Value> {
    crate::AGENTS
        .iter()
        .map(|a| {
            json!({
                "id": a.name.to_lowercase(),
                "name": a.name,
                "role": a.role,
                "domain": a.domain,
                "status": "ONLINE",
                "latency_ms": Value::Null,
            })
        })
        .collect()
}

/// The federation registry read lives in one place; reuse it rather than opening a second reader.
fn peers() -> Value {
    let p = crate::state_handlers::federation_payload("/federation/peers");
    p.get("peers").cloned().unwrap_or(json!([]))
}

/// The one place the mesh node set is built: the live agent roster plus the federation registry.
/// No latency for a roster agent - nothing on this host measures inter-agent RTT; federation nodes
/// carry the registry's own probe value.
pub fn mesh_nodes() -> Value {
    let nodes = roster_nodes();
    let peer_list = peers();
    let (n_nodes, n_peers) = (nodes.len(), peer_list.as_array().map(|a| a.len()).unwrap_or(0));
    json!({
        "node_count": n_nodes + n_peers,
        "nodes": nodes,
        "peers": peer_list,
        "p2p_beacon_port": 8022,
        "basis": "agents from the live roster (main.rs); peers from swarm_v2_federation/registry.json. \
                  The payload this replaces asserted ping_ms 0.12 / 0.08 and a 0.10 ms link.",
    })
}

pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();
    let path = url.split('?').next().unwrap_or(url).trim_end_matches('/');
    if !path.starts_with("/mesh") {
        return None;
    }

    if method == Method::Get && (path == "/mesh" || path == "/mesh/topology") {
        let mut t = mesh_nodes();
        t["status"] = json!("ONLINE");
        t["links"] = json!([]);
        t["links_reason"] = json!("no inter-node latency is measured on this host, so no link is asserted");
        return Some(ok(t));
    }
    if method == Method::Get && path == "/mesh/peers" {
        return Some(ok(json!({
            "peers": peers(),
            "source": "swarm_v2_federation/registry.json",
        })));
    }
    if method == Method::Get && path == "/mesh/nodes" {
        let nodes = roster_nodes();
        return Some(ok(json!({
            "nodes": nodes,
            "node_count": nodes.len(),
            "basis": "the live 15-agent roster",
        })));
    }
    if method == Method::Get && path == "/mesh/stats" {
        let peers = peers();
        let n_peers = peers.as_array().map(|a| a.len()).unwrap_or(0);
        return Some(ok(json!({
            "agents": crate::AGENTS.len(),
            "federation_peers": n_peers,
            "p2p_beacon_port": 8022,
            "basis": "counted from the roster and the federation registry; no throughput or latency is \
                      measured here, so none is reported",
        })));
    }
    if method == Method::Get && path == "/mesh/log" {
        return Some(ok(json!({
            "status": "no_persisted_state",
            "events": [],
            "reason": "the mesh logs to the gateway's own log files, and this server keeps no mesh event \
                       log; an empty list with its reason is reported rather than an invented history",
        })));
    }

    if method == Method::Post && path == "/mesh/route" {
        return Some(engine_only(
            "/mesh/route routes a task across the P2P mesh with density scoring over live node state \
             (swarm_v2/routes/mesh.py). This host has no inter-node transport wired into the kernel, so \
             a route is refused instead of a fabricated path. Task routing happens in POST /swarm/chat \
             (the expert-team router), not here.",
        ));
    }
    if method == Method::Post && path == "/mesh/announce" {
        return Some(engine_only(
            "/mesh/announce broadcasts this node's presence to mesh peers over the P2P beacon. No \
             beacon transport is wired here, so the announcement is refused rather than reported as \
             sent.",
        ));
    }
    if method == Method::Post {
        return Some(engine_only(&format!(
            "{path} is not a route this kernel serves. The /mesh family answers topology, peers, nodes, \
             stats and log from real sources; its transport actions are refused rather than faked."
        )));
    }

    // An unmatched GET must not borrow another member's payload.
    Some(ok(json!({
        "status": "unknown_mesh_path",
        "module_path": path,
        "reason": "this path is not part of the mesh family this kernel serves; it answers topology, \
                   peers, nodes, stats and log, and never echoes one member's payload for another",
    })))
}
