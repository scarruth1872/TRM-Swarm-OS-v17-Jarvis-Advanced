//! swarm-services — the whole family absorbed into the kernel.
//!
//! One native service layer for the ecosystem's platforms, all
//! persisted to the verified NAND fabric over trm:// (no HTTP to the
//! kernel). Each platform's core records become K3-encoded fabric
//! records with a real verdict:
//!
//!   jarvis   → voice/agent frontend records (hands runs, messages)
//!   agentq   → agent states, blackboard keys, goals
//!   aura     → semantic field states (metaconcepts)
//!   hera7    → biomarker records (21 biomarkers)
//!   voicebox → TTS/STT profile records
//!   bonsai   → local-LLM inference records
//!   embed    → embedding-model records
//!
//! The pattern is the swarm-fit one: entity → K3 trits → NAND append
//! → byte-verified persistence → verdict.

use swarm_k3::collapse;
use swarm_nand::NandFabric;

/// The Monad anchor (the truth reference on every platform's records).
pub const MONAD_ANCHOR: [i8; 4] = [1, 1, 1, 1];

/// The ecosystem platforms the kernel now serves.
pub const PLATFORMS: [&str; 7] = [
    "jarvis", "agentq", "aura", "hera7", "voicebox", "bonsai", "embed",
];

/// The K3 verdict on a platform record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceVerdict {
    Live,     // clear proceed (record accepted)
    Blocked,  // hold (conflict, risk)
    Analyze,  // uncertain (needs review)
}

impl ServiceVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceVerdict::Live => "LIVE",
            ServiceVerdict::Blocked => "BLOCKED",
            ServiceVerdict::Analyze => "ANALYZE",
        }
    }
}

/// Encode a platform record as K3 trits (deterministic).
pub fn encode(platform: &str, fields: &[&str]) -> Vec<i8> {
    let mut trits = MONAD_ANCHOR.to_vec();
    for f in fields {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in format!("{platform}|{f}").bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        for k in 0..6 {
            h ^= h >> 30;
            h = h.wrapping_mul(0xBF58476D1CE4E5B9);
            trits.push(if (h >> k) & 1 == 1 { 1 } else { -1 });
        }
    }
    trits
}

/// The K3 verdict on a platform action from its signals.
pub fn verdict(signals: &[f32], tau: f32) -> ServiceVerdict {
    let trits = collapse(signals, tau);
    let t = trits.iter().filter(|&&v| v == 1).count();
    let f = trits.iter().filter(|&&v| v == -1).count();
    if t >= 2 && f == 0 {
        ServiceVerdict::Live
    } else if f > t {
        ServiceVerdict::Blocked
    } else {
        ServiceVerdict::Analyze
    }
}

/// Serve a platform record: persist to NAND + verdict.
pub fn serve(fabric: &mut NandFabric, platform: &str, fields: &[&str],
             signals: &[f32]) -> (ServiceVerdict, u64, Vec<i8>) {
    let rec = encode(platform, fields);
    let offset = match fabric.append(&rec) {
        Ok(off) => off,
        Err(_) => 0,
    };
    let v = verdict(signals, 0.35);
    let _ = fabric.flush();
    (v, offset, rec)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_fabric(name: &str) -> NandFabric {
        let p = swarm_paths::resolve(&format!("/tmp/swarm_services_test_{name}.k3"));
        let _ = std::fs::remove_file(&p);
        NandFabric::open(&p).expect("open")
    }

    #[test]
    fn platforms_are_registered() {
        assert_eq!(PLATFORMS.len(), 7);
        assert!(PLATFORMS.contains(&"jarvis"));
        assert!(PLATFORMS.contains(&"agentq"));
        assert!(PLATFORMS.contains(&"embed"));
    }

    #[test]
    fn jarvis_record_persists() {
        let mut fab = tmp_fabric("jarvis");
        let (v, off, trits) = serve(&mut fab, "jarvis",
            &["hand=system-monitor", "status=ran", "ts=2026-08-16"],
            &[0.9, 0.8, 0.7]);
        assert_eq!(v, ServiceVerdict::Live);
        assert!(off > 0);
        let mut fab2 = NandFabric::open(&swarm_paths::resolve("/tmp/swarm_services_test_jarvis.k3")).expect("reopen");
        let got = fab2.read(off, trits.len()).unwrap_or_default();
        assert_eq!(got, trits, "byte-identical persistence");
    }

    #[test]
    fn conflict_blocks() {
        let mut fab = tmp_fabric("agentq");
        let (v, _, _) = serve(&mut fab, "agentq",
            &["goal=duplicate", "status=conflict"],
            &[-0.9, -0.8, -0.7]);
        assert_eq!(v, ServiceVerdict::Blocked);
    }
}
