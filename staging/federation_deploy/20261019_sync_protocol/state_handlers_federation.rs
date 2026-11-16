// Federation sync-protocol functions — replace the old federation_payload + federation_sync_payload
// in trm_uefi_kernel/host_council_server/src/state_handlers.rs with these three functions.
// NOTE: federation_sync_payload's signature changed from () to (&str) — update its caller in main.rs.

/// Read the federation registry (swarm_v2_federation/registry.json) as a JSON value; empty object
/// when absent or malformed — never an invented registry.
fn read_registry() -> Value {
    let reg_path = repo_root().join("swarm_v2_federation").join("registry.json");
    fs::read_to_string(&reg_path)
        .ok()
        .and_then(|c| serde_json::from_str::<Value>(&c).ok())
        .unwrap_or(json!({}))
}

/// Federation telemetry — the STANDARDIZED /federation/stats schema. Returns the registry's own v4
/// shape so every node exposes the SAME fields: {status, monad_anchor, registry_version, tailnet,
/// connected_nodes, total_nodes, nodes}. The old payload carried hardcoded mesh_agents=20 /
/// pbft_sync="100%" / voltage_harmony=0.8986 / backend v19.002 — declared constants with no
/// measurement, removed so a peer can trust the numbers it reads.
pub fn federation_payload(url: &str) -> Value {
    let reg = read_registry();
    let nodes: Vec<Value> = reg
        .get("connected_nodes")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();

    let connected_count = nodes.iter().filter(|p| {
        p.get("status").and_then(|s| s.as_str()).map(|s| s == "online" || s == "SYNCED").unwrap_or(false)
    }).count();
    let total_count = nodes.len();

    if url.ends_with("/peers") {
        return json!({ "peers": nodes });
    }

    json!({
        "status": "ok",
        "monad_anchor": "0x4D4F4E4144000001",
        "registry_version": reg.get("registry_version").cloned().unwrap_or(Value::Null),
        "tailnet": reg.get("mesh").and_then(|m| m.get("tailnet")).cloned().unwrap_or(Value::Null),
        "connected_nodes": connected_count,
        "total_nodes": total_count,
        "nodes": nodes,
    })
}

/// /sync + /federation/sync — the standardized sync-write endpoint. Verifies the caller's Monad
/// anchor and returns the registry's ACTUAL counts (derived), replacing the old hardcoded
/// nodes_synced=4 / active_nodes=4 / AGREED_14_0_0 / voltage_harmony=0.8986 constants.
pub fn federation_sync_payload(body: &str) -> Value {
    let parsed: Value = serde_json::from_str(body).unwrap_or(json!({}));
    let anchor_ok = parsed
        .get("monad_anchor")
        .and_then(Value::as_str)
        .map(|a| a == "0x4D4F4E4144000001")
        .unwrap_or(false);
    let caller = parsed.get("node_id").and_then(Value::as_str).unwrap_or("unknown");

    let reg = read_registry();
    let nodes: Vec<Value> = reg
        .get("connected_nodes")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();
    let connected = nodes.iter().filter(|p| {
        p.get("status").and_then(|s| s.as_str()).map(|s| s == "online" || s == "SYNCED").unwrap_or(false)
    }).count();

    json!({
        "status": if anchor_ok { "ok" } else { "anchor_mismatch" },
        "monad_anchor": "0x4D4F4E4144000001",
        "anchor_verified": anchor_ok,
        "caller_node_id": caller,
        "synced": anchor_ok,
        "connected_nodes": connected,
        "total_nodes": nodes.len(),
    })
}
