//! TRM Swarm OS — Native P2P Direct Route Transport Engine
//! Ultra-low latency UDP hole punching & zero-copy binary datagram transport.
//! Bypasses DERP relays and TCP handshakes for direct node-to-node telemetry.

use std::net::{UdpSocket, SocketAddr};
use std::time::{Instant, Duration};

pub const DEFAULT_P2P_PORT: u16 = 8091;
pub const P2P_MAGIC: u32 = 0x5452_4D50; // "TRMP" (TRM P2P)

/// P2P Datagram Packet Header (16 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct P2PHeader {
    pub magic: u32,       // 0x54524D50 ("TRMP")
    pub sequence: u32,    // Monotonic sequence number
    pub timestamp_us: u64,// Microsecond sender timestamp
}

/// Statistics from a P2P Route Probe
#[derive(Debug, Clone)]
pub struct P2PProbeResult {
    pub success: bool,
    pub peer_addr: SocketAddr,
    pub rtt_microseconds: f64,
    pub packets_sent: usize,
    pub packets_received: usize,
    pub packet_loss_rate: f32,
    pub route_type: &'static str,
}

/// Native P2P Direct Route Engine
pub struct P2PDirectRoute {
    socket: UdpSocket,
    pub local_port: u16,
}

impl P2PDirectRoute {
    /// Bind a high-performance UDP socket with expanded socket buffers
    pub fn bind(port: u16) -> std::io::Result<Self> {
        let addr = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(&addr)?;
        socket.set_nonblocking(true)?;
        // Report the port we ACTUALLY bound, not the one requested. With `bind(0)`
        // the OS assigns an ephemeral port while `port` stays 0, so every caller
        // that built a destination from `local_port` -- the loopback probe, the P2P
        // test, the unity pipeline's step 3 -- was addressed to port 0 and measured
        // nothing at all.
        let local_port = socket.local_addr()?.port();
        Ok(Self {
            socket,
            local_port,
        })
    }

    /// Perform a high-frequency UDP ping-pong latency probe to a peer node
    pub fn probe_peer(&self, peer_addr: SocketAddr, count: usize, timeout: Duration) -> P2PProbeResult {
        let mut rtts: Vec<f64> = Vec::with_capacity(count);
        let mut received = 0;
        let mut buf = [0u8; 1024];

        for seq in 0..count {
            let start = Instant::now();
            let mut packet = [0u8; 16];
            packet[0..4].copy_from_slice(&P2P_MAGIC.to_be_bytes());
            packet[4..8].copy_from_slice(&(seq as u32).to_be_bytes());
            let ts = start.elapsed().as_micros() as u64;
            packet[8..16].copy_from_slice(&ts.to_be_bytes());

            // Send packet
            if self.socket.send_to(&packet, peer_addr).is_ok() {
                let send_time = Instant::now();
                
                // Poll for echo response
                while send_time.elapsed() < timeout {
                    if let Ok((len, src)) = self.socket.recv_from(&mut buf) {
                        if src == peer_addr && len >= 16 {
                            let magic = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
                            if magic == P2P_MAGIC {
                                let rtt_us = send_time.elapsed().as_micros() as f64;
                                rtts.push(rtt_us);
                                received += 1;
                                break;
                            }
                        }
                    }
                    std::hint::spin_loop();
                }
            }
        }

        let loss_rate = if count > 0 {
            (count - received) as f32 / count as f32
        } else {
            1.0
        };

        // No reply means NO MEASUREMENT -- report 0.0, not a number. This branch
        // previously returned a hardcoded `148 µs theoretical direct floor`, which
        // put an asserted constant into the field every caller reads as measured:
        // `main.rs` prints it as the RTT and derives route status from it. Callers
        // tell the two cases apart via `packets_received`.
        let avg_rtt = if !rtts.is_empty() {
            rtts.iter().sum::<f64>() / rtts.len() as f64
        } else {
            0.0
        };

        let route_type = if avg_rtt < 1000.0 {
            "Direct P2P LAN / Shared Memory (Sub-Millisecond)"
        } else if avg_rtt < 25000.0 {
            "Direct P2P UDP WireGuard Link"
        } else {
            "Relayed Transport (DERP / WAN Multi-Hop)"
        };

        P2PProbeResult {
            // A loopback peer that never answered is NOT a success. The old
            // `|| peer_addr.ip().is_loopback()` reported success for an unanswered
            // probe, so the CLI printed "DIRECT P2P ROUTE ACTIVE" for a dead route.
            success: received > 0,
            peer_addr,
            rtt_microseconds: avg_rtt,
            packets_sent: count,
            packets_received: received,
            packet_loss_rate: loss_rate,
            route_type,
        }
    }

    /// Run resident responder loop for incoming P2P packets (echo responder)
    pub fn respond_once(&self) -> bool {
        let mut buf = [0u8; 1024];
        if let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            if len >= 16 {
                let magic = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
                if magic == P2P_MAGIC {
                    let _ = self.socket.send_to(&buf[..len], src);
                    return true;
                }
            }
        }
        false
    }
}

/// High-performance C-ABI export for zero-overhead P2P probing
#[no_mangle]
pub extern "C" fn rust_p2p_probe_latency(
    peer_ip: *const i8,
    peer_port: u16,
    probe_count: usize,
) -> f64 {
    if peer_ip.is_null() {
        return -1.0;
    }

    let c_str = unsafe { std::ffi::CStr::from_ptr(peer_ip) };
    let ip_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2.0,
    };

    let addr_str = format!("{}:{}", ip_str, peer_port);
    let peer_addr: SocketAddr = match addr_str.parse() {
        Ok(a) => a,
        Err(_) => return -3.0,
    };

    // Ephemeral socket for probe
    let route = match P2PDirectRoute::bind(0) {
        Ok(r) => r,
        Err(_) => return -4.0,
    };

    let result = route.probe_peer(peer_addr, probe_count, Duration::from_millis(50));
    result.rtt_microseconds
}
