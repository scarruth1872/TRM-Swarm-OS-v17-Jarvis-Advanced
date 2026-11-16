//! TRM Swarm OS v19.001 "Sovereign" — Inter-Machine Mesh Discovery Protocol
//! Zero-config UDP broadcast peer discovery on port 8022 for multi-node bare-metal swarms.
//! Allows multiple physical hardware machines running TRM Swarm OS to auto-federate.

use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::MONAD_TRUTH_ANCHOR;
use crate::agent_runtime::NUM_AGENTS;
use crate::nand_persistence::MANDATORY_TRIT_COUNT;

pub const MESH_DISCOVERY_PORT: u16 = 8022;
pub const MESH_MAGIC: u64 = 0x5452_4D5F_4D45_5348; // "TRM_MESH" in ASCII hex

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshBeaconPacket {
    pub magic: u64,
    pub node_id: u64,
    pub truth_anchor: u64,
    pub agent_count: u16,
    pub trit_count: u32,
    pub timestamp_ms: u64,
}

impl MeshBeaconPacket {
    pub fn new_local(node_id: u64) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        MeshBeaconPacket {
            magic: MESH_MAGIC,
            node_id,
            truth_anchor: MONAD_TRUTH_ANCHOR,
            agent_count: NUM_AGENTS as u16,
            trit_count: MANDATORY_TRIT_COUNT as u32,
            timestamp_ms: ts,
        }
    }

    pub fn serialize(&self) -> [u8; 36] {
        let mut buf = [0u8; 36];
        buf[0..8].copy_from_slice(&self.magic.to_le_bytes());
        buf[8..16].copy_from_slice(&self.node_id.to_le_bytes());
        buf[16..24].copy_from_slice(&self.truth_anchor.to_le_bytes());
        buf[24..26].copy_from_slice(&self.agent_count.to_le_bytes());
        buf[26..30].copy_from_slice(&self.trit_count.to_le_bytes());
        buf[30..38.min(36)].copy_from_slice(&self.timestamp_ms.to_le_bytes()[..6]);
        buf
    }

    pub fn deserialize(buf: &[u8]) -> Option<Self> {
        if buf.len() < 36 { return None; }
        let magic = u64::from_le_bytes(buf[0..8].try_into().ok()?);
        if magic != MESH_MAGIC { return None; }

        let node_id = u64::from_le_bytes(buf[8..16].try_into().ok()?);
        let truth_anchor = u64::from_le_bytes(buf[16..24].try_into().ok()?);
        let agent_count = u16::from_le_bytes(buf[24..26].try_into().ok()?);
        let trit_count = u32::from_le_bytes(buf[26..30].try_into().ok()?);
        let mut ts_bytes = [0u8; 8];
        ts_bytes[..6].copy_from_slice(&buf[30..36]);
        let timestamp_ms = u64::from_le_bytes(ts_bytes);

        Some(MeshBeaconPacket {
            magic,
            node_id,
            truth_anchor,
            agent_count,
            trit_count,
            timestamp_ms,
        })
    }
}

pub struct SovereignMeshDiscovery {
    pub local_node_id: u64,
    pub socket: Option<UdpSocket>,
}

impl SovereignMeshDiscovery {
    pub fn bind(node_id: u64) -> Self {
        let socket = UdpSocket::bind(("0.0.0.0", MESH_DISCOVERY_PORT)).ok();
        if let Some(ref s) = socket {
            let _ = s.set_broadcast(true);
            let _ = s.set_nonblocking(true);
        }

        SovereignMeshDiscovery {
            local_node_id: node_id,
            socket,
        }
    }

    /// Broadcast presence beacon over local subnet
    pub fn broadcast_beacon(&self) -> bool {
        if let Some(ref s) = self.socket {
            let pkt = MeshBeaconPacket::new_local(self.local_node_id);
            let bytes = pkt.serialize();
            s.send_to(&bytes, ("255.255.255.255", MESH_DISCOVERY_PORT)).is_ok()
        } else {
            false
        }
    }

    /// Probe and receive any incoming peer beacons
    pub fn probe_peers(&self) -> Vec<MeshBeaconPacket> {
        let mut peers = Vec::new();
        if let Some(ref s) = self.socket {
            let mut buf = [0u8; 64];
            while let Ok((amt, _src)) = s.recv_from(&mut buf) {
                if let Some(pkt) = MeshBeaconPacket::deserialize(&buf[..amt]) {
                    if pkt.node_id != self.local_node_id && pkt.truth_anchor == MONAD_TRUTH_ANCHOR {
                        peers.push(pkt);
                    }
                }
            }
        }
        peers
    }

    /// Probe peers, accepting ONLY beacons whose HMAC verifies. The legacy
    /// unauthenticated path is NOT used: an unsigned beacon is rejected.
    pub fn probe_peers_authenticated(&self, key: &[u8]) -> Vec<MeshBeaconPacket> {
        let mut peers = Vec::new();
        if let Some(ref s) = self.socket {
            let mut buf = [0u8; 128];
            while let Ok((amt, _src)) = s.recv_from(&mut buf) {
                let Ok(pkt) = verify_beacon_bytes(&buf[..amt], key) else { continue };
                if pkt.node_id != self.local_node_id && pkt.truth_anchor == MONAD_TRUTH_ANCHOR {
                    peers.push(pkt);
                }
            }
        }
        peers
    }
}

// ── Authenticated beacons (combined approach) ───────────────────────────────
//
// The acceptance test below previously read:
//     if pkt.node_id != self.local_node_id && pkt.truth_anchor == MONAD_TRUTH_ANCHOR
// Both MESH_MAGIC and MONAD_TRUTH_ANCHOR are PUBLIC constants (the anchor is even
// rendered on the boot splash), so any LAN host could forge enrollment. No HMAC,
// no signature, no nonce.
//
// The signed form appends HMAC-SHA256(key, body) to the SAME 36-byte body, so an
// existing parser still reads the body and the tag makes forgery detectable.
// Format is shared byte-for-byte with the no_std kernel implementation in
// trm_uefi_kernel/src/hmac_sha256.rs and the AURA tool in
// Agent-Q/agents/skills/aura_native/mesh_auth.py.

use hmac::{Hmac, Mac};
use sha2::Sha256;

pub const BEACON_BODY: usize = 36;
pub const BEACON_TAG: usize = 32;
pub const BEACON_SIGNED: usize = BEACON_BODY + BEACON_TAG;

type HmacSha256 = Hmac<Sha256>;

/// Load the shared mesh secret. Path overridable via TRM_MESH_KEY_FILE.
pub fn load_mesh_key() -> Option<Vec<u8>> {
    let path = std::env::var("TRM_MESH_KEY_FILE")
        .unwrap_or_else(|_| "data/mesh_key.bin".to_string());
    std::fs::read(path).ok().filter(|k| !k.is_empty())
}

pub fn sign_beacon_bytes(node_id: u64, key: &[u8]) -> [u8; BEACON_SIGNED] {
    let body = MeshBeaconPacket::new_local(node_id).serialize();
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac accepts any key length");
    mac.update(&body);
    let tag = mac.finalize().into_bytes();
    let mut out = [0u8; BEACON_SIGNED];
    out[..BEACON_BODY].copy_from_slice(&body);
    out[BEACON_BODY..].copy_from_slice(&tag);
    out
}

/// Verify a signed beacon. The tag is checked BEFORE any field is trusted.
pub fn verify_beacon_bytes(blob: &[u8], key: &[u8]) -> Result<MeshBeaconPacket, &'static str> {
    if blob.len() < BEACON_SIGNED {
        return Err("beacon too short for a signed frame");
    }
    let (body, tag) = blob.split_at(BEACON_BODY);
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| "bad key")?;
    mac.update(body);
    mac.verify_slice(&tag[..BEACON_TAG])
        .map_err(|_| "HMAC mismatch - forged or wrong key")?;
    MeshBeaconPacket::deserialize(body).ok_or("malformed body after tag verified")
}

