//! Native platform benchmark suite (council MOTION-B1).
//!
//! Measures the in-kernel machinery with real wall-clock timing
//! (`std::time::Instant` + `std::hint::black_box`) and labels every number
//! `measured_on_host`. Nothing here is asserted — each figure is the elapsed
//! time of a real call on this host. Python-side lanes (council deliberation
//! latency, recall P@1 over the labeled set) are measured by the offline
//! harness and merged into the sealed artifact; they are not fabricated here.
use serde_json::{json, Value};
use std::hint::black_box;
use std::time::Instant;

fn ns_since(t0: Instant) -> u64 {
    t0.elapsed().as_nanos() as u64
}

fn measure_trm7m() -> Value {
    let mut m = crate::trm7m::Trm7m::new();
    let t0 = Instant::now();
    let (trit, conf, _ns) = m.think(black_box("benchmark probe: is the platform healthy"), 7);
    let think7_ns = ns_since(t0);
    let t1 = Instant::now();
    let (_t, _c, _) = m.think(black_box("probe"), 1);
    let think1_ns = ns_since(t1);
    json!({
        "label": "measured_on_host",
        "model": "TRM-7M (462-dim Mamba-3 SSM, 3 blocks, seed-42 blob)",
        "think_7turn_ns": think7_ns,
        "think_1turn_ns": think1_ns,
        "decision_trit": trit,
        "confidence": conf,
    })
}

fn measure_recall() -> Value {
    // Warm each store so its lazy load is reported separately, not charged to
    // the timed query.
    let t_cold = Instant::now();
    let _ = crate::recall_router::route("chembl aspirin");
    let chembl_cold_ns = ns_since(t_cold);
    let _ = crate::recall_router::route("warmup");
    let t0 = Instant::now();
    let (v, _code) = crate::recall_router::route(black_box("turbovec"));
    let turbovec_ns = ns_since(t0);
    let t1 = Instant::now();
    let (_v2, _c2) = crate::recall_router::route(black_box("chembl aspirin"));
    let chembl_ns = ns_since(t1);
    json!({
        "label": "measured_on_host",
        "method": "stores warmed, then timed; chembl_cold_load_ns is the first-call store load",
        "chembl_cold_load_ns": chembl_cold_ns,
        "route_turbovec_ns": turbovec_ns,
        "route_chembl_steady_ns": chembl_ns,
        "engine": v.get("engine").cloned().unwrap_or(Value::Null),
    })
}

fn measure_gpu_scan() -> Value {
    let store = match crate::packed_ternary::store() {
        Some(s) => s,
        None => return json!({"label": "measured_on_host", "available": false,
                               "reason": "no packed store loaded"}),
    };
    let qv = crate::native_encoder::encode_f32(
        "protein binding affinity small molecule", crate::gpu_compute::DIM);
    let q = match crate::packed_ternary::quantize(&qv, store.dim()) {
        Some(q) => q,
        None => return json!({"label": "measured_on_host", "available": false,
                               "reason": "query quantize failed"}),
    };
    let t0 = Instant::now();
    let _cpu = black_box(crate::packed_ternary::cpu_scan_in(&store, &q));
    let cpu_us = t0.elapsed().as_secs_f64() * 1e6;
    let (gpu_avail, gpu_us) = if crate::gpu_compute::gpu_available() {
        let t1 = Instant::now();
        let g = crate::gpu_compute::gpu_scan_scores_cached(&q, &store);
        match g {
            Some(_) => (true, Some(t1.elapsed().as_secs_f64() * 1e6)),
            None => (true, None),
        }
    } else {
        (false, None)
    };
    json!({
        "label": "measured_on_host",
        "gpu_available": gpu_avail,
        "store_dim": store.dim(),
        "cpu_scan_us": cpu_us,
        "gpu_scan_cached_us": gpu_us,
        "speedup_cpu_over_gpu": match gpu_us {
            Some(g) if g > 0.0 => Some(cpu_us / g),
            _ => None,
        },
    })
}

pub fn benchmark_payload() -> Value {
    json!({
        "record": "native platform benchmark (council MOTION-B1)",
        "host": "windows-x86_64 (this host)",
        "method": "std::time::Instant + std::hint::black_box; every figure labeled",
        "trm7m": measure_trm7m(),
        "recall": measure_recall(),
        "deterministic": crate::chat_handlers::bench_deterministic(),
        "gpu_scan": measure_gpu_scan(),
    })
}
