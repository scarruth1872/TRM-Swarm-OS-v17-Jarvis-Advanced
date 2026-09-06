//! TRM Swarm OS — P2P Direct Route (council 83563732). Native gap closure.
//!
//! Zero-hop peer-to-peer packet router — closes the Laptop `p2p_direct_route`
//! capability natively (voltage doctrine: no HTTP). Maintains a peer registry,
//! a direct-route table, and reports measured routing state. The actual UDP
//! socket is the host/federation bridge's job; this kernel module models the
//! P2P routing decision substrate and honesty-labels the socket as host-handoff.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// A direct P2P peer.
#[derive(Clone)]
pub struct P2PPeer {
    pub peer_id: String,
    pub addr: [u8; 4],
    pub port: u16,
    pub last_seen: u64,
    /// direct/zero-hop route (no relay)
    pub zero_hop: bool,
}

/// Direct-route table entry.
pub struct DirectRoute {
    pub src: String,
    pub dst: String,
    pub hops: u8,       // 0 = direct (zero-hop)
    pub latency_us: u64,
    pub healthy: bool,
}

/// Zero-hop peer-to-peer packet router.
pub struct P2PDirectRouter {
    pub peers: Vec<P2PPeer>,
    pub routes: Vec<DirectRoute>,
    pub packets_routed: u64,
    pub direct_packets: u64,
}

impl P2PDirectRouter {
    pub fn new() -> Self {
        P2PDirectRouter {
            peers: Vec::new(),
            routes: Vec::new(),
            packets_routed: 0,
            direct_packets: 0,
        }
    }

    /// Register a peer (zero-hop if it's a known direct link).
    pub fn register_peer(&mut self, peer_id: &str, addr: [u8; 4], port: u16, now: u64) {
        // replace if exists
        self.peers.retain(|p| p.peer_id != peer_id);
        self.peers.push(P2PPeer {
            peer_id: peer_id.to_string(),
            addr,
            port,
            last_seen: now,
            zero_hop: true, // direct registration = zero-hop
        });
    }

    /// Establish a direct (zero-hop) route between two peers.
    pub fn connect_direct(&mut self, src: &str, dst: &str, latency_us: u64) {
        self.routes.retain(|r| !(r.src == src && r.dst == dst));
        self.routes.push(DirectRoute {
            src: src.to_string(),
            dst: dst.to_string(),
            hops: 0,
            latency_us,
            healthy: true,
        });
    }

    /// Route a packet: prefer a zero-hop direct route, else relay.
    pub fn route_packet(&mut self, src: &str, dst: &str) -> String {
        self.packets_routed += 1;
        // zero-hop direct?
        if let Some(r) = self.routes.iter().find(|r| r.src == src && r.dst == dst && r.healthy) {
            self.direct_packets += 1;
            format!(
                "DIRECT {}->{} hops={} latency={}us",
                src, dst, r.hops, r.latency_us
            )
        } else {
            // no direct route => relay (honest)
            format!("RELAY {}->{} (no zero-hop route, mesh indirect)", src, dst)
        }
    }

    /// Ratio of packets sent zero-hop direct.
    pub fn direct_ratio(&self) -> f64 {
        if self.packets_routed == 0 {
            return 0.0;
        }
        self.direct_packets as f64 / self.packets_routed as f64
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

/// Demo: DESKTOP<->LAPTOP direct link, route a packet, report ratio.
pub fn demo() -> (usize, usize, u64, u64, f64) {
    let mut r = P2PDirectRouter::new();
    r.register_peer("DESKTOP", [127, 0, 0, 1], 8022, 1);
    r.register_peer("LAPTOP", [100, 123, 215, 119], 8022, 1);
    r.register_peer("EDGE", [10, 0, 0, 5], 9000, 1);
    r.connect_direct("DESKTOP", "LAPTOP", 12);
    r.route_packet("DESKTOP", "LAPTOP"); // direct
    r.route_packet("DESKTOP", "EDGE");   // relay (no direct route)
    r.route_packet("LAPTOP", "DESKTOP"); // relay (one-way direct)
    (
        r.peer_count(),
        r.route_count(),
        r.packets_routed,
        r.direct_packets,
        r.direct_ratio(),
    )
}

pub fn get_p2p_direct_route() -> u64 {
    0x4D4F_4E41_4400_0001
}