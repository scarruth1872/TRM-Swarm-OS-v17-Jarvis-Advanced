//! TRM Swarm OS - federation (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/federation.py.
//! Functions marked // MANUAL need human verification of math semantics.
//!
//! Verified against the Python oracle swarm_v2/core/federation.py (differential
//! vectors: swarm_v2_artifacts/port_tests/vectors/federation.json):
//!   * SwarmNode.base_url()  - the `http://{host}:{port}` property.
//!   * FederationRegistry    - the pure registry state machine: add_node (self /
//!     already-online rejection + insertion-order-preserving update), remove_node,
//!     get_node, get_online_nodes, mark_online, mark_offline.
//!   * handle_peer_request() - FederationProtocol's online-peers filter (excludes
//!     the requester and the local node).
//!   * federation_stats()    - FederationProtocol.get_federation_stats() composition.
//!
//! Host-side, deliberately NOT ported (file I/O, wall clock, crypto, network,
//! threading, and the global-memory dependency):
//!   * _load_registry / _save_registry / os.makedirs(FEDERATION_DIR)      (file I/O)
//!   * datetime.now().isoformat()  -> last_seen, handshake/sync timestamps,
//!     and the sha256 api-key nonce (wall clock)
//!   * _generate_hmac / _verify_hmac  (hmac + sha256; this crate has no SHA-256)
//!   * handshake / discover_peers / sync_memories  (requests + asyncio)
//!   * start_heartbeat / stop_heartbeat  (Thread + time.sleep)
//!   * get_global_memory() / contribute / get_recent_entries / get_stats
//!     (semantic store -> a port returns a needs_host flag, not fake data)
//!   * SwarmNode.to_dict()/from_dict() JSON (de)serialization (the struct fields
//!     map 1:1 to the dict keys; no JSON codec on bare metal)
//!
//! The two HMAC-related native-strip pieces below (constant_time_eq and the batch
//! presence mask) predate this port and are kept unchanged.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const FEDERATION_DIR: u64 = 0;
pub const FEDERATION_API_PREFIX: &'static str = "/federation";

// ── The strip's model (swarm_v2_artifacts/native_voltage_strips/strip_federation_handshake.rs) ────
//
// The strip carries two things: `constant_time_eq`, a genuine byte-wise comparison, and
// `strip_batch_verify_hmac_signatures`, which -- read exactly -- checks that each 32-byte signature
// contains at least one non-zero byte and returns how many do. That is a signature-PRESENCE check:
// it does NOT establish that an HMAC is correct, and this comment says so rather than letting the
// function name imply otherwise. The real digest comparison lives host-side (python's
// `_verify_hmac` with `hmac.compare_digest`), and this crate has no SHA-256 to compute a digest with.
//
// Both are ported for what they are: the comparison is useful on its own, and the mask is the
// platform's actual native verification stage.

/// Byte-wise equality that does not short-circuit on the first difference (the strip's inline fn):
/// every byte is compared regardless, so timing does not reveal the matching prefix length. Length
/// mismatch still returns false immediately -- as in the strip.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut xor = 0u8;
    for i in 0..a.len() {
        xor |= a[i] ^ b[i];
    }
    xor == 0
}

/// The strip's batch mask over `count` 32-byte signatures: (valid_total, mask), where a signature
/// "valid" means it is not entirely zero. See the note above -- this is presence, not correctness.
pub fn batch_verify_hmac_signatures(sigs: &[u8], count: usize) -> (usize, Vec<u8>) {
    let mut mask: Vec<u8> = Vec::with_capacity(count);
    let mut valid_total = 0usize;
    for i in 0..count {
        let b = i * 32;
        if b + 32 > sigs.len() {
            mask.push(0);
            continue;
        }
        let sig = &sigs[b..b + 32];
        let is_valid = sig.iter().any(|&x| x != 0);
        mask.push(if is_valid { 1 } else { 0 });
        if is_valid {
            valid_total += 1;
        }
    }
    (valid_total, mask)
}

//  SwarmNode 
#[derive(Debug, Clone)]
pub struct SwarmNode {
    pub node_id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub api_key: String,
    pub status: String,
    pub last_seen: String,
    pub capabilities: Vec<String>,
    pub memory_count: i64,
}

impl SwarmNode {
    pub fn new() -> Self {
        SwarmNode {
            node_id: String::new(),
            name: String::new(),
            host: String::new(),
            port: 0,
            api_key: String::new(),
            status: String::new(),
            last_seen: String::new(),
            capabilities: vec![],
            memory_count: 0,
        }
    }

    /// Python `SwarmNode.base_url` property: `http://{host}:{port}`.
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

//  FederationRegistry (pure state machine) 
/// Port of `federation.py`'s `FederationRegistry`, minus the disk (`_load_registry` /
/// `_save_registry`) and wall-clock (`datetime.now()`) side effects. `connected_nodes`
/// is a Python dict keyed by node_id, so the no_std side keeps `Vec<SwarmNode>` in
/// FIRST-SEEN order -- never a sorted map (see the learned pitfalls).
#[derive(Debug, Clone)]
pub struct FederationRegistry {
    pub local_node: Option<SwarmNode>,
    pub connected_nodes: Vec<SwarmNode>,
}

impl FederationRegistry {
    pub fn new() -> Self {
        FederationRegistry {
            local_node: None,
            connected_nodes: Vec::new(),
        }
    }

    /// `FederationRegistry.add_node(node) -> bool`.
    ///
    /// Rejects self (node_id == local id) and a node that is already connected AND online.
    /// Otherwise inserts, preserving the Python dict semantics: an existing key is updated
    /// IN PLACE (keeps its original position), a new key is appended. The Python side then
    /// calls `_save_registry()` -- host-side file I/O, not ported.
    pub fn add_node(&mut self, node: SwarmNode) -> bool {
        // Don't add self. Python raises AttributeError if local_node is None; here an
        // unset local node simply skips the self check (local is always set in the real flow).
        if let Some(local) = &self.local_node {
            if node.node_id == local.node_id {
                return false;
            }
        }
        // Already connected (existing and online) -> reject.
        if let Some(existing) = self.connected_nodes.iter().find(|n| n.node_id == node.node_id) {
            if existing.status == "online" {
                return false;
            }
        }
        // self.connected_nodes[node.node_id] = node  (dict update: in-place if present)
        match self
            .connected_nodes
            .iter_mut()
            .find(|n| n.node_id == node.node_id)
        {
            Some(slot) => *slot = node,
            None => self.connected_nodes.push(node),
        }
        true
    }

    /// `FederationRegistry.remove_node(node_id)`; deletes only if present. `_save_registry()`
    /// afterwards is host-side.
    pub fn remove_node(&mut self, node_id: &str) {
        if let Some(pos) = self
            .connected_nodes
            .iter()
            .position(|n| n.node_id == node_id)
        {
            self.connected_nodes.remove(pos);
        }
    }

    /// `FederationRegistry.get_node(node_id) -> Optional[SwarmNode]`.
    pub fn get_node(&self, node_id: &str) -> Option<&SwarmNode> {
        self.connected_nodes.iter().find(|n| n.node_id == node_id)
    }

    /// `FederationRegistry.get_online_nodes()`: status == "online", insertion order preserved.
    pub fn get_online_nodes(&self) -> Vec<&SwarmNode> {
        self.connected_nodes
            .iter()
            .filter(|n| n.status == "online")
            .collect()
    }

    /// `FederationRegistry.mark_online(node_id)`: sets status to "online". The Python side also
    /// stamps `last_seen = datetime.now().isoformat()` and saves -- both host-side.
    pub fn mark_online(&mut self, node_id: &str) {
        if let Some(n) = self
            .connected_nodes
            .iter_mut()
            .find(|n| n.node_id == node_id)
        {
            n.status = String::from("online");
        }
    }

    /// `FederationRegistry.mark_offline(node_id)`: sets status to "offline". `_save_registry()`
    /// afterwards is host-side.
    pub fn mark_offline(&mut self, node_id: &str) {
        if let Some(n) = self
            .connected_nodes
            .iter_mut()
            .find(|n| n.node_id == node_id)
        {
            n.status = String::from("offline");
        }
    }
}

//  FederationProtocol.handle_peer_request() 
/// Port of `FederationProtocol.handle_peer_request(requester_node_id)`'s pure filter: return the
/// online connected nodes, excluding the requester and the local node. Python returns
/// `{"peers": [n.to_dict() ...]}`; the no_std side returns the node ids in first-seen order
/// (JSON serialization of the full dict is host-side).
pub fn handle_peer_request(
    requester_node_id: &str,
    local_node_id: &str,
    connected: &[SwarmNode],
) -> Vec<String> {
    let mut peers: Vec<String> = Vec::new();
    for n in connected {
        if n.status == "online" && n.node_id != requester_node_id && n.node_id != local_node_id {
            peers.push(n.node_id.clone());
        }
    }
    peers
}

/// `FederationProtocol.get_federation_stats()` result, flattened for no_std.
/// `local_node_id` is the local node's id (None when no local node is registered);
/// `connected_nodes` is the online count; `total_nodes` is the registry size;
/// `node_list` is the online node ids in first-seen order.
#[derive(Debug, Clone)]
pub struct FederationStats {
    pub local_node_id: Option<String>,
    pub connected_nodes: usize,
    pub total_nodes: usize,
    pub node_list: Vec<String>,
}

/// Port of `FederationProtocol.get_federation_stats()`: pure composition over the registry
/// (online count == node_list length, total count == registry size).
pub fn federation_stats(local_node: Option<&SwarmNode>, connected: &[SwarmNode]) -> FederationStats {
    let node_list: Vec<String> = connected
        .iter()
        .filter(|n| n.status == "online")
        .map(|n| n.node_id.clone())
        .collect();
    FederationStats {
        local_node_id: local_node.map(|n| n.node_id.clone()),
        connected_nodes: node_list.len(),
        total_nodes: connected.len(),
        node_list,
    }
}

//  get_federation() 
static SINGLETON_FEDERATION: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_federation() -> u64 {
    // Singleton handle (lazy-init pattern from federation.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_FEDERATION.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_FEDERATION.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
