//! TRM Swarm OS — thz_bonsai_gateway (no_std port).
//! Bonsai 27B Ternary Intelligence wired through the THz microkernel
//! gateway — deterministic core from swarm_v2/core/thz_bonsai_gateway.py:
//!   - process_request payload encoding: bytes → 128-dim float vector of
//!     b/255.0 with \x00 padding (`payload[:128].ljust(128, b'\x00')`);
//!   - _to_trits: K3 quantization at ternary_threshold = 0.02;
//!   - route decision: trit_sum → np.sign → {-1:"BLOCK",0:"ANALYZE",1:"PROCEED"};
//!   - route_confidence = sum(|trits|)/len(trits);
//!   - BonsaiTernaryCore stats: total_calls, avg_latency_us
//!     round(total_latency_ns / max(1, calls) / 1000, 2).
//! The Mamba-3 SSM matrices (A/B/C/D — loaded weights or np.random.randn)
//! are not portable → honest `// MANUAL` skeleton, never invented.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// BonsaiTernaryCore dimensions (python __init__ defaults).
pub const BONSAI_DIM: usize = 128;
pub const BONSAI_STATE_DIM: usize = 32;
/// _ternary_threshold — match training threshold (python 0.02).
pub const TERNARY_THRESHOLD: f64 = 0.02;
/// Trit payload width used by process_request encoding.
pub const PAYLOAD_DIM: usize = 128;

fn abs_f64(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// round(x, 2) matching python round() on avg latency (host-safe values).
fn round2(x: f64) -> f64 {
    let t = (x * 100.0 + 0.5).floor_f64_impl();
    t / 100.0
}

trait FloorF64 {
    fn floor_f64_impl(self) -> f64;
}

impl FloorF64 for f64 {
    fn floor_f64_impl(self) -> f64 {
        let t = self as i64 as f64;
        if self < t {
            t - 1.0
        } else {
            t
        }
    }
}

/// process_request payload encoding:
/// `[b / 255.0 for b in payload[:128].ljust(128, b'\x00')]`
pub fn encode_payload(payload: &[u8]) -> Vec<f64> {
    let mut out = vec![0.0f64; PAYLOAD_DIM];
    let n = payload.len().min(PAYLOAD_DIM);
    for i in 0..n {
        out[i] = payload[i] as f64 / 255.0;
    }
    out
}

/// `_to_trits(vector)`: > threshold → 1, < -threshold → -1, else 0.
pub fn to_trits(vector: &[f64]) -> Vec<i8> {
    vector
        .iter()
        .map(|&v| {
            if v > TERNARY_THRESHOLD {
                1
            } else if v < -TERNARY_THRESHOLD {
                -1
            } else {
                0
            }
        })
        .collect()
}

/// `_trit_map.get(np.sign(trit_sum) if trit_sum != 0 else 0, "ANALYZE")`.
pub fn route_decision(trit_sum: i64) -> &'static str {
    let sig = if trit_sum > 0 {
        1
    } else if trit_sum < 0 {
        -1
    } else {
        0
    };
    match sig {
        1 => "PROCEED",
        -1 => "BLOCK",
        _ => "ANALYZE",
    }
}

/// route_confidence = sum(|trits|) / len(trits) (proportion non-zero).
pub fn route_confidence(trits: &[i8]) -> f64 {
    if trits.is_empty() {
        return 0.0;
    }
    let mut s = 0i64;
    for &t in trits {
        s += abs_f64(t as f64) as i64;
    }
    s as f64 / trits.len() as f64
}

/// Full deterministic routing outcome for an encoded/processed latent:
/// trits → sum → dominant kleene label + confidence.
pub fn route_trits(trits: &[i8]) -> (&'static str, f64) {
    let mut sum = 0i64;
    for &t in trits {
        sum += t as i64;
    }
    (route_decision(sum), route_confidence(trits))
}

/// BonsaiTernaryState dataclass (deterministic fields).
#[derive(Debug, Clone)]
pub struct BonsaiTernaryState {
    pub trits: Vec<i8>,
    pub route_confidence: f64,
    /// MANUAL: ponderomotive trap energy requires the h state (RNG-init).
    pub energy: f64,
}

/// BonsaiTernaryCore — MANUAL skeleton for the SSM process (weights are
/// file-loaded or np.random-init matrices). Deterministic core kept here:
/// trit conversion, routing and stats bookkeeping.
#[derive(Debug, Clone)]
pub struct BonsaiTernaryCore {
    pub dim: usize,
    pub state_dim: usize,
    pub total_calls: u64,
    pub total_latency_ns: u64,
}

impl BonsaiTernaryCore {
    pub fn new(dim: usize, state_dim: usize) -> Self {
        BonsaiTernaryCore {
            dim,
            state_dim,
            total_calls: 0,
            total_latency_ns: 0,
        }
    }

    /// MANUAL: full SSM process (A/B/C/D @h + Euler + Ponderomotive trap)
    /// needs the trained/RNG matrices. Deterministic routing over an
    /// externally supplied output vector is exposed instead.
    pub fn process(&mut self, output_latent: &[f64]) -> BonsaiTernaryState {
        let trits = to_trits(output_latent);
        let conf = route_confidence(&trits);
        self.total_calls += 1;
        BonsaiTernaryState {
            trits,
            route_confidence: conf,
            energy: 0.0, // MANUAL
        }
    }

    /// Record measured latency (python accumulates int(elapsed_us * 1000)).
    pub fn record_latency_ns(&mut self, latency_ns: u64) {
        self.total_latency_ns += latency_ns;
    }

    /// get_stats(): avg_latency_us = round(total_latency_ns/max(1,calls)/1000, 2).
    pub fn get_stats(&self) -> (u64, f64) {
        let avg_lat = self.total_latency_ns as f64 / self.total_calls.max(1) as f64 / 1000.0;
        (self.total_calls, round2(avg_lat))
    }
}

/// THzBonsaiMicrokernelGateway — MANUAL wrapper over the Bonsai core;
/// deterministic payload encode + decode string assembly are ported.
#[derive(Debug, Clone)]
pub struct ThzBonsaiMicrokernelGateway {
    pub bonsai: BonsaiTernaryCore,
}

impl ThzBonsaiMicrokernelGateway {
    pub fn new() -> Self {
        ThzBonsaiMicrokernelGateway {
            bonsai: BonsaiTernaryCore::new(BONSAI_DIM, BONSAI_STATE_DIM),
        }
    }

    /// Python decode string (formatting kept 1:1):
    ///   "[Bonsai 27B • THz Microkernel • {lat:.1}μs] {dominant} — Ternary
    ///    K3 routing complete. {conf*100:.0}% confidence."
    pub fn decode_message(&self, dominant: &str, conf: f64, latency_us: f64) -> String {
        alloc::format!(
            "[Bonsai 27B • THz Microkernel • {:.1}μs] {} — Ternary K3 routing complete. {:.0}% confidence.",
            latency_us,
            dominant,
            conf * 100.0
        )
    }
}

impl Default for ThzBonsaiMicrokernelGateway {
    fn default() -> Self {
        Self::new()
    }
}

/// Singleton instance (lazy-init pattern from thz_bonsai_gateway.py).
static SINGLETON_THZ_BONSAI_GATEWAY: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
/// get_thz_bonsai_gateway() singleton handle: 1 once initialized.
pub fn get_thz_bonsai_gateway() -> u64 {
    let v = SINGLETON_THZ_BONSAI_GATEWAY.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_THZ_BONSAI_GATEWAY.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
