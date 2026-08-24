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
}
