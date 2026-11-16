//! Federation reachability - separating CONFIGURED topology from VERIFIED peers.
//!
//! WHAT WAS WRONG
//! The federation endpoints GENERATED 100 peers from a hardcoded tier table:
//!     ("Tier 1: Core Host & Ingestion", 10, "100.93.191.", 0.45, "PRIMARY_ONLINE"),
//!     ...
//! building each address as `format!("{}{}", ip_prefix, i)`. Every one was served
//! with `"is_active": true` and a status like PRIMARY_ONLINE, plus a synthetic
//! latency of `base_lat + i*0.12`. /federation/stats then asserted
//! `active_nodes: 100`, `offline_nodes: 0`, `data_integrity: "100.0000%"` and a
//! merkle_root, while /swarm/telemetry on the SAME PROCESS reported 2.
//!
//! Measured: this machine's own address is 100.93.191.43 - the tier prefixes are
//! enumerations of its own subnet - and NONE of the 100 answered a 1s TCP connect.
//! So the platform was reporting a 100-node, 100%-integrity, zero-offline
//! federation that it had never contacted and could not reach.
//!
//! THE DISTINCTION THIS MODULE ADDS
//!   CONFIGURED - what the tier table declares. A real, checkable config count.
//!   VERIFIED   - what actually answered a probe. Measured here.
//! They are different numbers and the API now reports both, so "configured" can
//! never be mistaken for "reachable".

use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

pub struct ProbeResult {
    pub configured: usize,
    pub verified: usize,
    pub probed: usize,
    pub method: String,
}

static VERIFIED: AtomicUsize = AtomicUsize::new(0);
static PROBED: AtomicUsize = AtomicUsize::new(0);
static CONFIGURED: AtomicUsize = AtomicUsize::new(0);
static DONE: OnceLock<bool> = OnceLock::new();
/// Per-peer outcomes, index-aligned with the configured list, so request
/// handlers can read reachability WITHOUT re-probing. Re-probing per request
/// cost 100 x 80ms = 8s on every call - the cache must hold the detail, not
/// just the totals.
static PEER_OK: OnceLock<Vec<bool>> = OnceLock::new();

/// Probe one address:port with a short timeout. Returns true only on a completed
/// TCP handshake - i.e. something is listening AND routable.
pub fn probe(host: &str, port: u16, timeout_ms: u64) -> bool {
    let addr = format!("{}:{}", host, port);
    let sock = match addr.to_socket_addrs() {
        Ok(mut it) => match it.next() {
            Some(s) => s,
            None => return false,
        },
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&sock, Duration::from_millis(timeout_ms)).is_ok()
}

/// Probe the whole configured peer set once, in parallel, and cache the counts.
/// 100 sequential 150ms probes would cost 15s; one thread per peer costs ~150ms.
pub fn init_probe(hosts: Vec<(String, u16)>) {
    DONE.get_or_init(|| {
        CONFIGURED.store(hosts.len(), Ordering::Relaxed);
        let handles: Vec<_> = hosts
            .into_iter()
            .map(|(h, p)| {
                std::thread::spawn(move || probe(&h, p, 150))
            })
            .collect();
        let mut ok = 0usize;
        let mut n = 0usize;
        let mut flags: Vec<bool> = Vec::new();
        for t in handles {
            n += 1;
            let hit = matches!(t.join(), Ok(true));
            flags.push(hit);
            if hit {
                ok += 1;
            }
        }
        PROBED.store(n, Ordering::Relaxed);
        VERIFIED.store(ok, Ordering::Relaxed);
        let _ = PEER_OK.set(flags);
        true
    });
}

pub fn result() -> ProbeResult {
    ProbeResult {
        configured: CONFIGURED.load(Ordering::Relaxed),
        verified: VERIFIED.load(Ordering::Relaxed),
        probed: PROBED.load(Ordering::Relaxed),
        method: "TCP connect to council port, 150 ms timeout, one thread per peer".to_string(),
    }
}

/// Per-peer reachability at `index`, from the CACHED probe. Returns None if no
/// probe has run yet, so callers can distinguish "unprobed" from "unreachable"
/// instead of silently reporting `false`.
pub fn peer_reachable(index: usize) -> Option<bool> {
    PEER_OK.get().and_then(|v| v.get(index).copied())
}

/// Cached probe outcome, if a probe has been run.
pub fn cached() -> Option<ProbeResult> {
    if DONE.get().is_some() {
        Some(result())
    } else {
        None
    }
}
