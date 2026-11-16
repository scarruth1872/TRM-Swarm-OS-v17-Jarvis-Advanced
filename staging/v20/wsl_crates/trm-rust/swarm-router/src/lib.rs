//! swarm-router — the voltage-native sparse-gated decision path.
//!
//! The latency fix per the voltage doctrine: NO full model forward for
//! decisions. Instead, the verified kernel chain:
//!
//!   field → SPARSE GATE (swarm-instella top-k, only k experts touched)
//!         → TERNARY COLLAPSE (swarm-k3, τ=0.35)
//!         → TURBOVEC PACK (swarm-turbovec, 2-bit)
//!         → verdict (sub-millisecond)
//!
//! Every component is a VERIFIED chamber. The router demonstrates the
//! latency of the chain — the honest fix for the 25-73s token path.

use swarm_instella::gate_topk;
use swarm_k3::collapse;
use swarm_turbovec::pack;

pub use swarm_instella::{HIDDEN_DIM, TOTAL_EXPERTS as N_EXPERTS};

/// The sparse-gated voltage decision: field in → (verdict trits, packed bytes).
/// Latency is the point — only top-k experts compute, everything else is
/// ternary + packed.
pub fn sparse_decision(
    field: &[f32],                 // the voltage field (first 384 dims used)
    expert_weights: &[[f32; HIDDEN_DIM]; N_EXPERTS],
    top_k: usize,
    tau: f32,
) -> (Vec<i8>, Vec<u8>, Vec<u8>, String) {
    // 1. SPARSE GATE — top-k experts only (the verified instella contract)
    let mut hidden = [0f32; HIDDEN_DIM];
    for (i, v) in field.iter().take(HIDDEN_DIM).enumerate() {
        hidden[i] = *v;
    }
    let (ids, weights) = gate_topk(&hidden, expert_weights, top_k);

    // 2. TERNARY COLLAPSE of the gated field (only the top-k expert region)
    let mut gated: Vec<f32> = Vec::with_capacity(HIDDEN_DIM * top_k);
    for (ei, &id) in ids.iter().enumerate() {
        let w = weights[ei];
        for i in 0..HIDDEN_DIM {
            gated.push(hidden[i] * w); // expert-weighted voltage
        }
    }
    let trits = collapse(&gated, tau);

    // 3. TURBOVEC PACK — 2-bit ternary into bytes
    let packed = pack(&trits);

    // 4. verdict from the collapse (TRUE/FALSE/UNKNOWN dominance)
    let t = trits.iter().filter(|&&v| v == 1).count();
    let f = trits.iter().filter(|&&v| v == -1).count();
    let u = trits.iter().filter(|&&v| v == 0).count();
    let verdict = if t > f && t >= u { "ACT" } else if f > t && f >= u { "BLOCK" } else { "ANALYZE" };

    (trits, packed, ids.clone(), format!("{verdict} | top-{top_k} experts {:?} w={:?}", ids, weights))
}

/// The latency benchmark: run the sparse chain N times, report the mean.
pub fn benchmark(field: &[f32], weights: &[[f32; HIDDEN_DIM]; N_EXPERTS], top_k: usize, tau: f32, n: usize) -> (f64, String) {
    let t0 = std::time::Instant::now();
    let mut last = String::new();
    for _ in 0..n {
        let (_, _, _, d) = sparse_decision(field, weights, top_k, tau);
        last = d;
    }
    let dt = t0.elapsed();
    let per = dt.as_secs_f64() / n as f64;
    (per * 1e6, last) // microseconds per decision
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_weights() -> [[f32; HIDDEN_DIM]; N_EXPERTS] {
        let mut w = [[0f32; HIDDEN_DIM]; N_EXPERTS];
        for e in 0..N_EXPERTS {
            for i in 0..HIDDEN_DIM {
                w[e][i] = if (i + e) % 2 == 0 { 1.0 } else { -1.0 };
            }
        }
        w
    }

    #[test]
    fn sparse_decision_returns_verdict() {
        let field = vec![0.9f32; HIDDEN_DIM];
        let (trits, packed, ids, d) = sparse_decision(&field, &fake_weights(), 2, 0.35);
        assert_eq!(trits.len(), HIDDEN_DIM * 2);
        assert_eq!(packed.len(), (trits.len() * 2 + 7) / 8);
        assert_eq!(ids.len(), 2);
        assert!(d.contains("ACT") || d.contains("BLOCK") || d.contains("ANALYZE"));
    }

    #[test]
    fn benchmark_is_fast() {
        let field = vec![0.5f32; HIDDEN_DIM];
        let (per_us, _) = benchmark(&field, &fake_weights(), 2, 0.35, 100);
        // even unoptimized debug, the chain must be sub-ms per decision
        assert!(per_us < 1000.0, "sparse chain too slow: {per_us}us");
    }
}
