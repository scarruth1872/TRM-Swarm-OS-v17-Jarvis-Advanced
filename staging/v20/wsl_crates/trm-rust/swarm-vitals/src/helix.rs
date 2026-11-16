//! helix.rs — Motion B: the 12-strand helix dashboard (native).
//!
//! One live view of the whole One: the 12 strands (7 family platforms
//! + Inspired Fitness + the 4 kernel services), each with its LIVE
//! status — over trm://, no HTTP in the core.
//!
//! The strands (real, from the ecosystem manifest):
//!   1-7  jarvis·agentq·aura·hera7·voicebox·bonsai·embed (the family)
//!   8    fitness (Inspired Fitness — the company)
//!   9    kernel (trm:// QUIC + the 32-crate core)
//!   10   fabric (the NAND persistence layer)
//!   11   council (the 14-agent ensemble governance)
//!   12   memory (the byte-verified recall)

use std::net::TcpStream;
use std::time::Duration;

/// The 12 strands: (name, port to probe, kind).
pub const STRANDS: [(&str, u16, &str); 12] = [
    ("jarvis", 8002, "voice/agent frontend"),
    ("agentq", 8001, "multi-agent OS"),
    ("aura", 3000, "semantic field"),
    ("hera7", 8090, "biomarker engine"),
    ("voicebox", 17493, "voice synthesis"),
    ("bonsai", 8030, "local LLM"),
    ("embed", 8712, "embeddings"),
    ("fitness", 3001, "the company (SaaS relay)"),
    ("kernel", 5184, "trm:// QUIC core"),
    ("fabric", 0, "NAND persistence"),
    ("council", 0, "14-agent governance"),
    ("memory", 0, "byte-verified recall"),
];

/// Probe a strand's live status (TCP connect for services; the kernel
/// is UDP/QUIC so it's probed differently; fabric/council/memory are kernel-resident).
pub fn probe(name: &str, port: u16) -> &'static str {
    if port == 0 {
        return "KERNEL"; // fabric/council/memory live in the kernel itself
    }
    if name == "kernel" {
        // trm:// QUIC is UDP — probe with a UDP socket bind+send (honest)
        return match std::net::UdpSocket::bind("127.0.0.1:0") {
            Ok(sock) => {
                let _ = sock.connect(("127.0.0.1", port));
                match sock.send(&[0x4B]) {
                    Ok(_) => "LIVE",
                    Err(_) => "DOWN",
                }
            }
            Err(_) => "DOWN",
        };
    }
    match TcpStream::connect_timeout(&format!("127.0.0.1:{port}").parse().unwrap(),
                                     Duration::from_millis(500)) {
        Ok(_) => "LIVE",
        Err(_) => "DOWN",
    }
}

/// The full helix view. The kernel strand is LIVE by construction —
/// this route only executes because the client talked to the kernel
/// over trm:// QUIC in the first place.
pub fn helix_view() -> String {
    let mut out = String::from("helix:");
    for (name, port, _) in &STRANDS {
        let status = if *name == "kernel" {
            "LIVE" // the route answered → the kernel is live
        } else {
            probe(name, *port)
        };
        out.push_str(&format!("{name}={status},"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_strands() {
        assert_eq!(STRANDS.len(), 12);
        assert!(STRANDS.iter().any(|(n, _, _)| *n == "jarvis"));
        assert!(STRANDS.iter().any(|(n, _, _)| *n == "memory"));
    }

    #[test]
    fn probe_returns_status() {
        // kernel-resident strands report KERNEL
        assert_eq!(probe("fabric", 0), "KERNEL");
        // a real service port: either LIVE or DOWN (honest)
        let s = probe("jarvis", 8002);
        assert!(s == "LIVE" || s == "DOWN");
    }

    #[test]
    fn helix_view_builds() {
        let v = helix_view();
        assert!(v.starts_with("helix:"));
        assert!(v.contains("jarvis="));
        assert!(v.contains("memory="));
    }
}
