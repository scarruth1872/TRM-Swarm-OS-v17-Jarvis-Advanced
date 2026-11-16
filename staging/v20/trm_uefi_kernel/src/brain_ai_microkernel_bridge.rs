//! TRM Swarm OS - brain_ai_microkernel_bridge (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/brain_ai_microkernel_bridge.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


// ── The oracle's model (swarm_v2/core/brain_ai_microkernel_bridge.py) ────────────────────────────
//
// `BRAIN_AI_METRIC_MAP` is a six-entry brain-key -> trm-key map in python; here it was the literal
// `0`. `translate_brain_metrics` walks that map and passes unknown keys through unchanged.
// `compute_stability_index_native` scores a six-metric row with
//     weights = [0.2, 0.25, 0.15, 0.1, 0.15, 0.15]     (they sum to 1.0)
//     stability = round(sum(m[k] * weights[k]), 3)
//     k3 = "+1" if s >= 0.70 else ("-1" if s <= 0.35 else "0")
// and the strip path (`strip_batch_brain_metrics`) returns those same two values. The strip DLL for
// this module is absent from swarm_v2_artifacts/native_voltage_strips (the neuro_continuum one is
// present), so the fallback arithmetic above is the oracle that can be checked here.
//
// Note the oracle classifies the trit on the RAW sum and reports the ROUNDED one, so the two are
// computed separately: a raw 0.6996 reports 0.700 and is still "0", not "+1".
//
// Host-side and not ported: `load_checkpoint` / torch `.pt` reads and `try_import`.

//  CONSTANTS 
/// The oracle's `BRAIN_AI_METRIC_MAP`, in its own order.
pub const BRAIN_AI_METRIC_MAP: [(&'static str, &'static str); 6] = [
    ("dimensionalStability", "cosmos3_dimensional_stability"),
    ("layerCoherence", "microkernel_layer_coherence"),
    ("networkFlow", "swarm_network_flow"),
    ("enginePower", "gpu_engine_power_pct"),
    ("realitySync", "monad_truth_sync_pct"),
    ("lineStrength", "cosmos3_line_strength"),
];

/// The oracle's harmonic stability weights (sum 1.0).
pub const STABILITY_WEIGHTS: [f64; 6] = [0.2, 0.25, 0.15, 0.1, 0.15, 0.15];

/// `translate_brain_metrics`: brain key -> TRM key, unknown keys unchanged.
pub fn translate_key(brain_key: &str) -> &str {
    for (k, v) in BRAIN_AI_METRIC_MAP.iter() {
        if *k == brain_key {
            return v;
        }
    }
    brain_key
}

/// `round(x, 3)` without std.
pub fn round3(v: f64) -> f64 {
    let s = v * 1000.0;
    let r = if s >= 0.0 { (s + 0.5) as i64 } else { (s - 0.5) as i64 };
    (r as f64) / 1000.0
}

/// The oracle's raw weighted sum, before its rounding.
pub fn stability_raw(m: &[f64; 6]) -> f64 {
    let mut s = 0.0;
    for k in 0..6 {
        s += m[k] * STABILITY_WEIGHTS[k];
    }
    s
}

/// The reported index: the raw sum rounded to 3.
pub fn stability_index(m: &[f64; 6]) -> f64 {
    round3(stability_raw(m))
}

/// The trit action, classified on the raw sum as the oracle does.
pub fn k3_for(raw: f64) -> &'static str {
    if raw >= 0.70 {
        "+1"
    } else if raw <= 0.35 {
        "-1"
    } else {
        "0"
    }
}

/// One batch row as `compute_stability_index_native` returns it: (stability, k3).
pub fn stability_row(m: &[f64; 6]) -> (f64, &'static str) {
    let raw = stability_raw(m);
    (round3(raw), k3_for(raw))
}
pub const BRAIN_AI_DATA_TYPES: &'static [&'static str] = &["metrics_update", "vector_response", "energy_response", "dimension_response", "voltron_metrics", "analysis_response"];
pub const VERSION: &'static str = "v18.7.0-BRAIN-AI";

//  get_brain_ai_bridge() 
static SINGLETON_BRAIN_AI_BRIDGE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_brain_ai_bridge() -> u64 {
    // Singleton handle (lazy-init pattern from brain_ai_microkernel_bridge.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_BRAIN_AI_BRIDGE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_BRAIN_AI_BRIDGE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
