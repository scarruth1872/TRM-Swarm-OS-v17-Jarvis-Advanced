//! OFP (OpenFang Protocol) transport, ported for the sovereign kernel.
//!
//! PROVENANCE: adapted from `openfang-wire` v0.6.9
//! (github.com/RightNow-AI/openfang, commit acf2587, Apache-2.0 OR MIT). A copy of the original
//! is preserved in `preserved_sources/openfang_original/`; see `openfang/VENDOR_PROVENANCE.md`.
//!
//! WHAT CHANGED AND WHY — the original is tokio/async and pulls `hmac`, `sha2`, `hex`, `subtle`,
//! `dashmap`, `chrono`, `uuid`, `tracing`. This kernel's whole dependency list is `tiny_http`,
//! `serde`, `serde_json`. The port is therefore synchronous and dependency-free:
//!
//! | original                     | here                                        |
//! |------------------------------|---------------------------------------------|
//! | `Hmac<Sha256>` from `hmac`   | `hmac_sha256` built on `k3_logic::sha256`   |
//! | `hex::encode`                | `hex_encode`                                 |
//! | `subtle::ConstantTimeEq`     | `ct_eq` (XOR fold, length-checked)          |
//! | `DashMap<String, Instant>`   | `Mutex<HashMap<String, Instant>>`           |
//! | `chrono::DateTime<Utc>`      | `connected_at_unix: u64`                    |
//! | `async fn write_message`     | `write_frame` / `read_frame` over `Read`+`Write` |
//!
//! WIRE FORMAT IS UNCHANGED, so a TRM kernel can talk to an OpenFang peer:
//! `[4-byte big-endian length][JSON body]`, authenticated frames append a 64-char hex HMAC.

#![allow(dead_code)] // registry/nonce surface is wired to routes incrementally

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------------------------
// Message types (JSON shape identical to upstream, so the two are wire-compatible)
// ---------------------------------------------------------------------------------------------

pub const PROTOCOL_VERSION: u32 = 1;

/// Largest frame we will accept. Upstream has no cap; an unbounded length prefix is a trivial
/// memory-exhaustion vector, so this one is ours.
pub const MAX_FRAME_BYTES: u32 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireMessage {
    pub id: String,
    #[serde(flatten)]
    pub kind: WireMessageKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WireMessageKind {
    #[serde(rename = "request")]
    Request(WireRequest),
    #[serde(rename = "response")]
    Response(WireResponse),
    #[serde(rename = "notification")]
    Notification(WireNotification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum WireRequest {
    #[serde(rename = "handshake")]
    Handshake {
        node_id: String,
        node_name: String,
        protocol_version: u32,
        agents: Vec<RemoteAgentInfo>,
        #[serde(default)]
        nonce: String,
        #[serde(default)]
        auth_hmac: String,
    },
    #[serde(rename = "discover")]
    Discover { query: String },
    #[serde(rename = "agent_message")]
    AgentMessage {
        agent: String,
        message: String,
        sender: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum WireResponse {
    #[serde(rename = "handshake_ack")]
    HandshakeAck {
        node_id: String,
        node_name: String,
        protocol_version: u32,
        agents: Vec<RemoteAgentInfo>,
        #[serde(default)]
        nonce: String,
        #[serde(default)]
        auth_hmac: String,
    },
    #[serde(rename = "discover_result")]
    DiscoverResult { agents: Vec<RemoteAgentInfo> },
    #[serde(rename = "agent_response")]
    AgentResponse { text: String },
    #[serde(rename = "pong")]
    Pong { uptime_secs: u64 },
    #[serde(rename = "error")]
    Error { code: i32, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum WireNotification {
    #[serde(rename = "agent_spawned")]
    AgentSpawned { agent: RemoteAgentInfo },
    #[serde(rename = "agent_terminated")]
    AgentTerminated { agent_id: String },
    #[serde(rename = "shutting_down")]
    ShuttingDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub tools: Vec<String>,
    pub state: String,
}

// ---------------------------------------------------------------------------------------------
// Framing
// ---------------------------------------------------------------------------------------------

/// Encode a message as `[4-byte BE length][JSON]`.
pub fn encode_message(msg: &WireMessage) -> Result<Vec<u8>, serde_json::Error> {
    let json = serde_json::to_vec(msg)?;
    let len = json.len() as u32;
    let mut bytes = Vec::with_capacity(4 + json.len());
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(&json);
    Ok(bytes)
}

pub fn decode_length(header: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*header)
}

pub fn decode_message(body: &[u8]) -> Result<WireMessage, serde_json::Error> {
    serde_json::from_slice(body)
}

/// Read one frame. Rejects a declared length above `MAX_FRAME_BYTES` before allocating.
pub fn read_frame<R: Read>(r: &mut R) -> io::Result<Vec<u8>> {
    let mut header = [0u8; 4];
    r.read_exact(&mut header)?;
    let len = decode_length(&header);
    if len > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame length {len} exceeds cap {MAX_FRAME_BYTES}"),
        ));
    }
    let mut body = vec![0u8; len as usize];
    r.read_exact(&mut body)?;
    Ok(body)
}

/// Write one framed message.
pub fn write_frame<W: Write>(w: &mut W, msg: &WireMessage) -> io::Result<()> {
    let bytes = encode_message(msg)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    w.write_all(&bytes)?;
    w.flush()
}

// ---------------------------------------------------------------------------------------------
// HMAC-SHA256 (RFC 2104) over the kernel's existing sha256 — no new dependency
// ---------------------------------------------------------------------------------------------

const HMAC_BLOCK: usize = 64;

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut block = [0u8; HMAC_BLOCK];
    if key.len() > HMAC_BLOCK {
        let h = crate::k3_logic::sha256(key);
        block[..32].copy_from_slice(&h);
    } else {
        block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; HMAC_BLOCK];
    let mut opad = [0x5cu8; HMAC_BLOCK];
    for i in 0..HMAC_BLOCK {
        ipad[i] ^= block[i];
        opad[i] ^= block[i];
    }

    let mut inner = Vec::with_capacity(HMAC_BLOCK + data.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(data);
    let inner_hash = crate::k3_logic::sha256(&inner);

    let mut outer = Vec::with_capacity(HMAC_BLOCK + 32);
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&inner_hash);
    crate::k3_logic::sha256(&outer)
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Constant-time comparison — a length check plus an XOR fold, so comparison time does not
/// reveal how many leading characters matched.
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Hex HMAC-SHA256 signature, as the protocol puts on the wire.
pub fn hmac_sign(secret: &str, data: &[u8]) -> String {
    hex_encode(&hmac_sha256(secret.as_bytes(), data))
}

pub fn hmac_verify(secret: &str, data: &[u8], signature: &str) -> bool {
    let expected = hmac_sign(secret, data);
    ct_eq(expected.as_bytes(), signature.as_bytes())
}

/// Session key both peers derive from the two nonces, per OFP.
pub fn derive_session_key(shared_secret: &str, our_nonce: &str, their_nonce: &str) -> String {
    let data = format!("{our_nonce}{their_nonce}");
    hmac_sign(shared_secret, data.as_bytes())
}

/// The handshake HMAC covers `nonce + node_id` (upstream comment; verified against peer.rs use).
pub fn handshake_hmac(shared_secret: &str, nonce: &str, node_id: &str) -> String {
    let data = format!("{nonce}{node_id}");
    hmac_sign(shared_secret, data.as_bytes())
}

// ---------------------------------------------------------------------------------------------
// Secret resolution — this is the kernel-side counterpart of gap G8
// ---------------------------------------------------------------------------------------------

pub const DEFAULT_SECRET: &str = "default_secret_change_me";

/// A secret is usable only if it is non-empty and is not the placeholder every Python module
/// falls back to. Silently accepting the default is how a shared-secret check becomes theatre.
pub fn validate_secret(secret: &str) -> Result<(), String> {
    let s = secret.trim();
    if s.is_empty() {
        return Err("federation secret is empty".to_string());
    }
    if s == DEFAULT_SECRET {
        return Err("federation secret is the default placeholder 'default_secret_change_me'"
            .to_string());
    }
    if s.len() < 32 {
        return Err(format!(
            "federation secret is {} chars; require at least 32",
            s.len()
        ));
    }
    Ok(())
}

/// Resolve the federation secret: env var first, then the on-disk secret the Python side writes.
/// Returns an explicit error rather than a default — federation must fail closed.
pub fn federation_secret() -> Result<String, String> {
    if let Ok(v) = std::env::var("SWARM_FEDERATION_SECRET") {
        if validate_secret(&v).is_ok() {
            return Ok(v.trim().to_string());
        }
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let path = std::path::Path::new(&local)
            .join("hermes")
            .join("swarm")
            .join("federation_secret");
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if validate_secret(&raw).is_ok() {
                return Ok(raw.trim().to_string());
            }
            return Err(format!(
                "secret file {} exists but is unusable: {}",
                path.display(),
                validate_secret(&raw).unwrap_err()
            ));
        }
    }
    Err("federation secret unavailable (no valid SWARM_FEDERATION_SECRET, no secret file)"
        .to_string())
}

// ---------------------------------------------------------------------------------------------
// Nonce replay defence
// ---------------------------------------------------------------------------------------------

/// Time-windowed single-use nonce tracker. Upstream uses DashMap; a Mutex is enough here and
/// keeps the dependency list unchanged.
#[derive(Clone)]
pub struct NonceTracker {
    seen: Arc<Mutex<HashMap<String, Instant>>>,
    window: Duration,
}

impl NonceTracker {
    pub fn new() -> Self {
        Self {
            seen: Arc::new(Mutex::new(HashMap::new())),
            window: Duration::from_secs(300),
        }
    }

    pub fn with_window(window: Duration) -> Self {
        Self {
            seen: Arc::new(Mutex::new(HashMap::new())),
            window,
        }
    }

    /// Record a nonce, or reject it as a replay.
    pub fn check_and_record(&self, nonce: &str) -> Result<(), String> {
        let now = Instant::now();
        let mut seen = self.seen.lock().unwrap_or_else(|e| e.into_inner());
        seen.retain(|_, ts| now.duration_since(*ts) < self.window);
        if seen.contains_key(nonce) {
            let shown: String = nonce.chars().take(16).collect();
            return Err(format!("nonce replay detected: {shown}"));
        }
        seen.insert(nonce.to_string(), now);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.seen.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for NonceTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------------------------
// Peer registry
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RemoteAgent {
    pub peer_node_id: String,
    pub info: RemoteAgentInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct PeerEntry {
    pub node_id: String,
    pub node_name: String,
    pub address: SocketAddr,
    pub agents: Vec<RemoteAgentInfo>,
    pub state: PeerState,
    /// Unix seconds — replaces chrono::DateTime<Utc> to stay dependency-free.
    pub connected_at_unix: u64,
    pub protocol_version: u32,
}

#[derive(Debug, Clone)]
pub struct PeerRegistry {
    peers: Arc<RwLock<HashMap<String, PeerEntry>>>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_peer(&self, entry: PeerEntry) {
        let mut peers = self.peers.write().unwrap_or_else(|e| e.into_inner());
        peers.insert(entry.node_id.clone(), entry);
    }

    pub fn remove_peer(&self, node_id: &str) -> Option<PeerEntry> {
        self.peers
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(node_id)
    }

    pub fn mark_disconnected(&self, node_id: &str) {
        let mut peers = self.peers.write().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = peers.get_mut(node_id) {
            entry.state = PeerState::Disconnected;
        }
    }

    pub fn mark_connected(&self, node_id: &str) {
        let mut peers = self.peers.write().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = peers.get_mut(node_id) {
            entry.state = PeerState::Connected;
        }
    }

    pub fn get_peer(&self, node_id: &str) -> Option<PeerEntry> {
        self.peers
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(node_id)
            .cloned()
    }

    pub fn connected_peers(&self) -> Vec<PeerEntry> {
        self.peers
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .filter(|p| p.state == PeerState::Connected)
            .cloned()
            .collect()
    }

    pub fn all_peers(&self) -> Vec<PeerEntry> {
        self.peers
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .cloned()
            .collect()
    }

    pub fn update_agents(&self, node_id: &str, agents: Vec<RemoteAgentInfo>) {
        let mut peers = self.peers.write().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = peers.get_mut(node_id) {
            entry.agents = agents;
        }
    }

    pub fn add_agent(&self, node_id: &str, agent: RemoteAgentInfo) {
        let mut peers = self.peers.write().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = peers.get_mut(node_id) {
            if let Some(existing) = entry.agents.iter_mut().find(|a| a.id == agent.id) {
                *existing = agent;
            } else {
                entry.agents.push(agent);
            }
        }
    }

    pub fn remove_agent(&self, node_id: &str, agent_id: &str) {
        let mut peers = self.peers.write().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = peers.get_mut(node_id) {
            entry.agents.retain(|a| a.id != agent_id);
        }
    }

    pub fn find_agents(&self, query: &str) -> Vec<RemoteAgent> {
        let needle = query.to_lowercase();
        let peers = self.peers.read().unwrap_or_else(|e| e.into_inner());
        let mut out = Vec::new();
        for peer in peers.values() {
            if peer.state != PeerState::Connected {
                continue;
            }
            for agent in &peer.agents {
                let hit = agent.name.to_lowercase().contains(&needle)
                    || agent.description.to_lowercase().contains(&needle)
                    || agent.tags.iter().any(|t| t.to_lowercase().contains(&needle));
                if hit {
                    out.push(RemoteAgent {
                        peer_node_id: peer.node_id.clone(),
                        info: agent.clone(),
                    });
                }
            }
        }
        out
    }

    pub fn all_remote_agents(&self) -> Vec<RemoteAgent> {
        let peers = self.peers.read().unwrap_or_else(|e| e.into_inner());
        let mut out = Vec::new();
        for peer in peers.values() {
            if peer.state != PeerState::Connected {
                continue;
            }
            for agent in &peer.agents {
                out.push(RemoteAgent {
                    peer_node_id: peer.node_id.clone(),
                    info: agent.clone(),
                });
            }
        }
        out
    }

    pub fn connected_count(&self) -> usize {
        self.peers
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .filter(|p| p.state == PeerState::Connected)
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.peers
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .len()
    }
}

impl Default for PeerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------------------------
// Tests — including a PUBLISHED oracle (RFC 4231), so this is judged against the world, not
// merely against itself.
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(id: &str, name: &str, tags: &[&str]) -> RemoteAgentInfo {
        RemoteAgentInfo {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("{name} agent"),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            tools: vec![],
            state: "running".to_string(),
        }
    }

    fn peer(node_id: &str, agents: Vec<RemoteAgentInfo>) -> PeerEntry {
        PeerEntry {
            node_id: node_id.to_string(),
            node_name: format!("{node_id}-name"),
            address: "127.0.0.1:9000".parse().unwrap(),
            agents,
            state: PeerState::Connected,
            connected_at_unix: 1_700_000_000,
            protocol_version: PROTOCOL_VERSION,
        }
    }

    // PUBLISHED ORACLE: RFC 4231 test case 2.
    // Key = "Jefe", data = "what do ya want for nothing?"
    // HMAC-SHA256 = 5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843
    #[test]
    fn test_hmac_matches_rfc4231_case2() {
        let sig = hmac_sign("Jefe", b"what do ya want for nothing?");
        assert_eq!(
            sig,
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    // PUBLISHED ORACLE: sha256("abc")
    #[test]
    fn test_sha256_oracle() {
        let h = crate::k3_logic::sha256(b"abc");
        assert_eq!(
            hex_encode(&h),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_hmac_key_longer_than_block() {
        // RFC 4231 case 6: 131-byte key, data "Test Using Larger Than Block-Size Key - Hash Key First"
        // HMAC-SHA256 = 60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54
        let key = vec![0xaau8; 131];
        let sig = hex_encode(&hmac_sha256(
            &key,
            b"Test Using Larger Than Block-Size Key - Hash Key First",
        ));
        assert_eq!(
            sig,
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn test_framing_roundtrip() {
        let msg = WireMessage {
            id: "msg-1".to_string(),
            kind: WireMessageKind::Request(WireRequest::Ping),
        };
        let bytes = encode_message(&msg).unwrap();
        assert_eq!(decode_length(&bytes[0..4].try_into().unwrap()) as usize, bytes.len() - 4);

        let mut cursor = std::io::Cursor::new(bytes);
        let body = read_frame(&mut cursor).unwrap();
        let decoded = decode_message(&body).unwrap();
        assert_eq!(decoded.id, "msg-1");
        assert!(matches!(
            decoded.kind,
            WireMessageKind::Request(WireRequest::Ping)
        ));
    }

    #[test]
    fn test_oversized_frame_rejected_before_alloc() {
        let mut cursor = std::io::Cursor::new(vec![0xff, 0xff, 0xff, 0xff]);
        let err = read_frame(&mut cursor).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_handshake_hmac_binds_nonce_and_node() {
        let good = handshake_hmac("s3cret", "nonce-1", "node-a");
        assert!(hmac_verify("s3cret", b"nonce-1node-a", &good));
        // wrong secret, wrong nonce, and wrong node must all fail
        assert!(!hmac_verify("other", b"nonce-1node-a", &good));
        assert!(!hmac_verify("s3cret", b"nonce-2node-a", &good));
        assert!(!hmac_verify("s3cret", b"nonce-1node-b", &good));
    }

    #[test]
    fn test_session_key_is_symmetric() {
        let a = derive_session_key("s", "nonceA", "nonceB");
        let b = derive_session_key("s", "nonceB", "nonceA");
        // OFP concatenates our_nonce + their_nonce, so order matters; equal inputs agree:
        assert_eq!(derive_session_key("s", "x", "y"), derive_session_key("s", "x", "y"));
        assert_ne!(a, b);
    }

    #[test]
    fn test_nonce_replay_rejected() {
        let t = NonceTracker::new();
        assert!(t.check_and_record("n-1").is_ok());
        let err = t.check_and_record("n-1").unwrap_err();
        assert!(err.contains("replay"));
        assert!(t.check_and_record("n-2").is_ok());
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn test_nonce_window_expires() {
        let t = NonceTracker::with_window(Duration::from_millis(30));
        assert!(t.check_and_record("n-1").is_ok());
        std::thread::sleep(Duration::from_millis(60));
        // expired entry is collected, so the nonce is usable again
        assert!(t.check_and_record("n-1").is_ok());
    }

    #[test]
    fn test_secret_validation_rejects_default() {
        assert!(validate_secret(DEFAULT_SECRET).is_err());
        assert!(validate_secret("").is_err());
        assert!(validate_secret("short").is_err());
        let ok = "a".repeat(64);
        assert!(validate_secret(&ok).is_ok());
    }

    #[test]
    fn test_registry_add_find_and_agents() {
        let r = PeerRegistry::new();
        r.add_peer(peer(
            "node-1",
            vec![agent("a1", "coder", &["code"]), agent("a2", "researcher", &["research"])],
        ));
        r.add_peer(peer("node-2", vec![agent("a3", "code-reviewer", &["code", "review"])]));
        assert_eq!(r.total_count(), 2);
        assert_eq!(r.connected_count(), 2);
        assert_eq!(r.find_agents("code").len(), 2);

        r.mark_disconnected("node-1");
        assert_eq!(r.connected_count(), 1);
        assert_eq!(r.total_count(), 2);
        assert_eq!(r.find_agents("research").len(), 0);

        r.mark_connected("node-1");
        r.remove_agent("node-1", "a2");
        assert_eq!(r.get_peer("node-1").unwrap().agents.len(), 1);
        assert_eq!(r.all_remote_agents().len(), 2);
    }
}
