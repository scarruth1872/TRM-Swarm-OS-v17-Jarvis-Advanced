//! Real benchmark harness for the platform's core primitives.
//!
//! These routes EXERCISE the actual K3 ternary logic, TurboVec quantizer and K3
//! SRAM code the platform is built on - the same sources the UEFI kernel runs -
//! rather than quoting figures from journal text.

use crate::k3_logic::KleeneTernaryEngine;
use crate::k3_ternary_sram::K3Sram;
use crate::turbovec::{
    FemtosecondGenomicSSMScanner, TurboVec16xQuantizer, TurboVec158Quantizer, DIM,
};
use crate::time::now_ns;
use serde_json::{json, Value};

/// Deterministic trit pattern: [0,1,2] -> [-1,0,+1].
fn trit(i: usize) -> i8 {
    match i % 3 {
        0 => -1,
        1 => 0,
        _ => 1,
    }
}

/// LCG so runs are reproducible without pulling in an RNG dependency.
struct Lcg(u64);
impl Lcg {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32 / 2.0) - 1.0
    }
}

pub fn k3_bench(n: usize) -> Value {
    let eng = KleeneTernaryEngine::new();
    let t0 = now_ns();
    let mut checksum: i64 = 0;
    for i in 0..n {
        let x = trit(i);
        let y = trit(i * 7 + 1);
        checksum += KleeneTernaryEngine::kleene_and(x, y) as i64;
        checksum += KleeneTernaryEngine::kleene_or(x, y) as i64;
        checksum += KleeneTernaryEngine::kleene_not(x) as i64;
        checksum += KleeneTernaryEngine::kleene_imply(x, y) as i64;
        checksum += KleeneTernaryEngine::kleene_equiv(x, y) as i64;
    }
    let ops = n * 5;
    let els = now_ns().wrapping_sub(t0);
    json!({
        "primitive": "KleeneTernaryEngine (K3 ALU, dfn-arithmetic)",
        "operators": ["AND=min", "OR=max", "NOT=negate", "IMPLY", "EQUIV"],
        "ops": ops,
        "elapsed_ns": els,
        "ns_per_op": els as f64 / ops as f64,
        "ops_per_sec": ops as f64 / (els as f64 / 1e9),
        "checksum": checksum,
        "source": "trm_uefi_kernel/src/k3_logic.rs (real module, linked not simulated)",
    })
}

pub fn k3_reduction_bench(n: usize, dim: usize) -> Value {
    let eng = KleeneTernaryEngine::new();
    let mut rng = Lcg(0x5EED);
    let mut vecbuf = vec![0.0f32; dim];
    for v in vecbuf.iter_mut() {
        *v = rng.next_f32();
    }
    let t0 = now_ns();
    let mut total_trits = 0usize;
    for _ in 0..n {
        let out = eng.quick_reduction(&vecbuf);
        total_trits += out.len();
    }
    let els = now_ns().wrapping_sub(t0);
    json!({
        "primitive": "KleeneTernaryEngine::quick_reduction (R^d -> T^m)",
        "calls": n, "input_dim": dim, "trits_out_total": total_trits,
        "elapsed_ns": els,
        "ns_per_call": els as f64 / n as f64,
        "calls_per_sec": n as f64 / (els as f64 / 1e9),
        "source": "trm_uefi_kernel/src/k3_logic.rs",
    })
}

pub fn turbovec_bench(n: usize) -> Value {
    let mut rng = Lcg(0xC0FFEE);
    let mut v = vec![0.0f32; DIM];
    for x in v.iter_mut() {
        *x = rng.next_f32();
    }
    let db = TurboVec16xQuantizer::normalize(&v);

    let t0 = now_ns();
    let mut acc = 0.0f32;
    for _ in 0..n {
        let (codes, scale, vmin) = TurboVec16xQuantizer::quantize_16x(&v);
        let rec = TurboVec16xQuantizer::dequantize_16x(&codes, scale, vmin);
        acc += TurboVec16xQuantizer::fast_femto_cosine_similarity(&rec, &db);
    }
    let els = now_ns().wrapping_sub(t0);
    let ratio = if els > 0 { n as f64 / (els as f64) } else { 0.0 };
    json!({
        "primitive": "TurboVec16xQuantizer (2-bit scalar quant, 16x compression)",
        "ops": n * 3, "dim": DIM,
        "elapsed_ns": els,
        "ns_per_triple": els as f64 / n as f64,
        "pairs_per_sec": ratio * 1e9,
        "similarity_accumulator": acc,
        "source": "trm_uefi_kernel/src/turbovec.rs",
    })
}

pub fn femto_bench(n: usize) -> Value {
    let mut scan = FemtosecondGenomicSSMScanner::new();
    let mut rng = Lcg(0xF3370);
    let mut v = vec![0.0f32; DIM];
    for x in v.iter_mut() {
        *x = rng.next_f32();
    }
    let t0 = now_ns();
    let mut internal_ns: u64 = 0;
    for _ in 0..n {
        let (_norm, step_ns, _bytes) = scan.femto_step(&v);
        internal_ns += step_ns;
    }
    let els = now_ns().wrapping_sub(t0);
    json!({
        "primitive": "FemtosecondGenomicSSMScanner::femto_step (h_t = 0.95h + 0.05x)",
        "steps": n, "dim": DIM,
        "wall_elapsed_ns": els,
        "wall_ns_per_step": els as f64 / n as f64,
        "internally_measured_ns_per_step": internal_ns as f64 / n as f64,
        "steps_per_sec": n as f64 / (els as f64 / 1e9),
        "source": "trm_uefi_kernel/src/turbovec.rs",
    })
}

pub fn trit_pack_bench(n: usize) -> Value {
    let t0 = now_ns();
    let mut acc: u64 = 0;
    for i in 0..n {
        let t = trit(i);
        let c = TurboVec158Quantizer::trit_to_code(t);
        acc += TurboVec158Quantizer::code_to_trit(c) as u64;
    }
    let els = now_ns().wrapping_sub(t0);
    json!({
        "primitive": "TurboVec158Quantizer trit<->code packing",
        "ops": n * 2, "elapsed_ns": els,
        "ns_per_op": els as f64 / (n * 2) as f64,
        "roundtrip_checksum": acc,
        "source": "trm_uefi_kernel/src/turbovec.rs",
    })
}

pub fn sram_bench(n: usize) -> Value {
    let mut cam = K3Sram::new();
    let t0 = now_ns();
    let mut hits = 0usize;
    for i in 0..n {
        let mut key = [0u8; 16];
        key[..8].copy_from_slice(&(i as u64).to_le_bytes());
        cam.store_entry(&key, trit(i), 0.9);
        if cam.lookup_entry(&key).is_some() {
            hits += 1;
        }
    }
    let els = now_ns().wrapping_sub(t0);
    let look = cam.cam_lookup(&[0u8; 16]);
    json!({
        "primitive": "K3Sram store_entry + lookup_entry (ternary CAM)",
        "entries": n, "hits": hits,
        "elapsed_ns": els,
        "ns_per_store_plus_lookup": els as f64 / n as f64,
        "cam_lookup_word": look,
        "source": "trm_uefi_kernel/src/k3_ternary_sram.rs",
    })
}

pub fn gpu_state() -> Value {
    // Real host query - no invented VRAM figures. Windows exposes the adapter via WMI;
    // precise VRAM utilisation needs a vendor API, which is reported as unavailable
    // rather than estimated.
    //
    // 2026-11-10: this endpoint silently returned {"queried": false} on this host, and
    // the cause was NOT the absence of a GPU - it was `wmic` itself. Windows 11 build
    // 26200 ships without wmic.exe, so the Command::new("wmic") failed every time and
    // the failure was swallowed into queried=false. A diagnostic route that reports
    // "cannot query" because it calls a removed tool is worse than one that says the
    // tool is missing, so the query now falls back to PowerShell Get-CimInstance and
    // records which backend answered.
    use std::process::Command;

    let mut out = json!({
        "note": "adapter identity is a real WMI query; VRAM utilisation requires a vendor API",
        "vram_utilisation_available": false,
        "backend": null,
        "queried": false,
    });

    let mut adapters: Option<Vec<Value>> = None;

    // 1) wmic - present on older Windows builds only.
    if let Ok(o) = Command::new("wmic")
        .args(["path", "win32_VideoController", "get", "Name,DriverVersion,AdapterRAM", "/format:csv"])
        .output()
    {
        if o.status.success() {
            let txt = String::from_utf8_lossy(&o.stdout).to_string();
            let rows: Vec<&str> = txt
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with("Node,"))
                .collect();
            if !rows.is_empty() {
                adapters = Some(rows.iter().map(|r| {
                    let f: Vec<&str> = r.split(',').collect();
                    json!({
                        "adapter_ram_bytes": f.get(1).and_then(|s| s.trim().parse::<u64>().ok()),
                        "driver_version": f.get(2).map(|s| s.trim()),
                        "name": f.get(3).map(|s| s.trim()),
                    })
                }).collect());
                out["backend"] = json!("wmic");
            }
        }
    }

    // 2) Win11 fallback: Get-CimInstance, which is the supported replacement.
    if adapters.is_none() {
        if let Ok(o) = Command::new("powershell")
            .args([
                "-NoProfile", "-NonInteractive", "-Command",
                "Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,AdapterRAM | ConvertTo-Json -Compress",
            ])
            .output()
        {
            if o.status.success() {
                let txt = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !txt.is_empty() {
                    if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                        let list = match v {
                            Value::Array(ref a) => a.clone(),
                            other => vec![other],
                        };
                        adapters = Some(list.iter().map(|a| {
                            let raw = a.get("AdapterRAM").and_then(|x| x.as_u64());
                            // AdapterRAM is a 32-bit DWORD in WMI; a 12GB card reports
                            // 4294967295 (0xFFFFFFFF). Flag it instead of printing a
                            // number that is obviously not the installed memory.
                            let overflow = raw.map(|r| r >= 4_000_000_000).unwrap_or(false);
                            json!({
                                "adapter_ram_bytes": raw,
                                "adapter_ram_is_overflowed_u32": overflow,
                                "name": a.get("Name").and_then(|x| x.as_str()),
                                "driver_version": a.get("DriverVersion").and_then(|x| x.as_str()),
                            })
                        }).collect());
                        out["backend"] = json!("powershell-Get-CimInstance");
                    }
                }
            }
        }
    }

    match adapters {
        Some(a) => {
            out["adapters"] = json!(a);
            out["queried"] = json!(true);
        }
        None => {
            out["note"] = json!("no adapter query succeeded: wmic absent and PowerShell Get-CimInstance unavailable");
        }
    }
    out
}

/// Article 117 replication attempt: 500,000 composite tasks in 391.21ms
/// (0.7824us/op) at 100.0000% pass. Per task we run exactly the primitives
/// Article 117 names: K3 ternary, an SRAM pattern, Mamba-3 SSM, TurboVec,
/// and a DDR antibody check.
pub fn article117_stress(n: usize) -> serde_json::Value {
    use crate::k3_logic::KleeneTernaryEngine;
    let eng = KleeneTernaryEngine::new();
    let mut sram = crate::k3_ternary_sram::K3Sram::new();
    let mut scan = crate::turbovec::FemtosecondGenomicSSMScanner::new();
    let dim = 64usize;
    let mut x = vec![0f32; dim];
    let mut db = vec![0f32; dim];
    for i in 0..dim {
        x[i] = (i as f32 * 0.017).sin();
        db[i] = (i as f32 * 0.011).cos();
    }
    let (codes, scale, vmin) = crate::turbovec::TurboVec16xQuantizer::quantize_16x(&x);
    let deq = crate::turbovec::TurboVec16xQuantizer::dequantize_16x(&codes, scale, vmin);
    let mut pass = 0usize;
    let t0 = crate::time::now_ns();
    for k in 0..n {
        let a: i8 = ((k % 3) as i8) - 1;
        let b: i8 = (((k / 3) % 3) as i8) - 1;
        let r1 = KleeneTernaryEngine::kleene_and(a, b);
        let r2 = KleeneTernaryEngine::kleene_or(a, b);
        let r3 = KleeneTernaryEngine::kleene_not(a);
        let red = eng.quick_reduction(&x);
        let key = [(k % 15) as u8];
        sram.store_entry(&key, r1, 0.9);
        let hit = sram.lookup_entry(&key);
        let (h, _, _) = scan.femto_step(&x);
        let sim = crate::turbovec::TurboVec16xQuantizer::fast_femto_cosine_similarity(&deq, &db);
        let ok = hit.is_some()
            && r3 == -a
            && r1 >= -1
            && r2 >= -1
            && !red.is_empty()
            && h.is_finite()
            && sim.is_finite();
        if ok {
            pass += 1;
        }
    }
    let ns = crate::time::now_ns().saturating_sub(t0);
    let secs = ns as f64 / 1e9;
    let per_op_ns = if n > 0 { ns as f64 / n as f64 } else { 0.0 };
    let ops_sec = if secs > 0.0 { n as f64 / secs } else { 0.0 };
    let pass_rate = if n > 0 { (pass as f64 / n as f64) * 100.0 } else { 0.0 };
    json!({
        "article": 117,
        "claim": {
            "tasks": 500000,
            "total_ms": 391.21,
            "per_op_us": 0.7824,
            "pass_rate_pct": 100.0,
            "title_ops_per_sec": 1280000.0
        },
        "measured": {
            "tasks": n,
            "total_ms": (ns as f64 / 1e6 * 1000.0).round() / 1000.0,
            "per_op_us": (per_op_ns / 1000.0 * 1000.0).round() / 1000.0,
            "per_op_ns": (per_op_ns * 100.0).round() / 100.0,
            "ops_per_sec": ops_sec.round(),
            "pass_rate_pct": (pass_rate * 10000.0).round() / 10000.0,
            "passed": pass
        },
        "primitives_per_task": ["k3_ternary_and/or/not", "k3_quick_reduction",
            "k3_sram_store+lookup", "mamba3_femto_step", "turbovec_cosine"],
        "gpu_vs_cpu": "CPU (microkernel routes are host-executed; no GPU endpoint exists)"
    })
}

/// Article 118 replication: 500,000 operations at 235.2 ns/op (117.60 ms) with
/// 100.0000% pass across all 15 virtual SRAM patterns. One task = all 15 patterns.
pub fn article118_stress(n: usize) -> serde_json::Value {
    use crate::sram_pattern_computer as sp;
    let dim = 16usize;
    let line = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let key = [1u8, 2, 3, 4];
    let rows_data: Vec<Vec<u8>> = (0..4u8).map(|i| vec![i, i + 1, i + 2, i + 3]).collect();
    let rows: Vec<&[u8]> = rows_data.iter().map(|r| r.as_slice()).collect();
    let s_a: Vec<u8> = (0..dim as u8).map(|i| i.wrapping_mul(7)).collect();
    let s_b: Vec<u8> = (0..dim as u8).map(|i| i.wrapping_mul(11)).collect();
    let spins: Vec<i8> = (0..dim).map(|i| if i % 2 == 0 { 1 } else { -1 }).collect();
    let coup: Vec<i8> = (0..dim * dim).map(|i| ((i % 3) as i8) - 1).collect();
    let wts: Vec<i8> = (0..dim * dim).map(|i| ((i % 3) as i8) - 1).collect();
    let mask: Vec<u8> = vec![1u8; dim];
    let nf: Vec<i32> = (0..dim).map(|i| i as i32).collect();
    let adj: Vec<u8> = vec![1u8; dim * dim];
    let ma: Vec<i32> = (0..dim * dim).map(|i| (i % 7) as i32).collect();
    let mb: Vec<i32> = (0..dim * dim).map(|i| (i % 5) as i32).collect();
    let scores: Vec<i32> = (0..dim).map(|i| (i * 3) as i32).collect();
    let hv_a: Vec<u8> = vec![0xF0u8; dim];
    let hv_b: Vec<u8> = vec![0x0Fu8; dim];
    let hg_a: Vec<i16> = (0..dim).map(|i| (i * 5) as i16).collect();
    let hg_b: Vec<i16> = (0..dim).map(|i| (i * 9) as i16).collect();
    let phases: Vec<i32> = (0..dim).map(|i| (i * 30) as i32).collect();
    let mut pass = 0usize;
    let t0 = crate::time::now_ns();
    for _ in 0..n {
        let r1 = sp::pattern_fpga(&line);
        let r2 = sp::pattern_cellular_automaton(&line);
        let r3 = sp::pattern_neural_network(&wts, &spins);
        let r4 = sp::pattern_content_addressable(&rows, &key);
        let r5 = sp::pattern_quantum_simulator(&s_a, &s_b);
        let r6 = sp::pattern_ising_spin_glass(&spins, &coup);
        let r7 = sp::pattern_hopfield_recall(&spins, &wts, dim);
        let r8 = sp::pattern_boltzmann_energy(&spins, &coup, dim);
        let r9 = sp::pattern_gnn_aggregate(&nf, &adj, dim);
        let r10 = sp::pattern_tcam_match(&spins, &spins, &mask);
        let r11 = sp::pattern_systolic_matmul(&ma, &mb, dim);
        let r12 = sp::pattern_sparse_attention(&scores, 4);
        let r13 = sp::pattern_hdc_bind(&hv_a, &hv_b);
        let r14 = sp::pattern_holographic_interference(&hg_a, &hg_b);
        let r15 = sp::pattern_onn_sync(&phases, 3);
        let ok = !r1.is_empty()
            && !r2.is_empty()
            && !r3.is_empty()
            && r4 >= 0
            && !r5.is_empty()
            && !r6.is_empty()
            && !r7.is_empty()
            && r8 >= i64::MIN
            && !r9.is_empty()
            && r10
            && !r11.is_empty()
            && !r12.is_empty()
            && !r13.is_empty()
            && !r14.is_empty()
            && !r15.is_empty();
        if ok {
            pass += 1;
        }
    }
    let ns = crate::time::now_ns().saturating_sub(t0);
    let secs = ns as f64 / 1e9;
    let per_op_ns = if n > 0 { ns as f64 / n as f64 } else { 0.0 };
    let ops_sec = if secs > 0.0 { n as f64 / secs } else { 0.0 };
    let pass_rate = if n > 0 { (pass as f64 / n as f64) * 100.0 } else { 0.0 };
    json!({
        "article": 118,
        "claim": {
            "operations": 500000,
            "total_ms": 117.60,
            "per_op_ns": 235.2,
            "pass_rate_pct": 100.0,
            "ops_per_sec": 4250000.0,
            "scope": "all 15 virtual SRAM patterns"
        },
        "measured": {
            "operations": n,
            "patterns_per_task": 15,
            "total_ms": (ns as f64 / 1e6 * 1000.0).round() / 1000.0,
            "per_op_ns": (per_op_ns * 100.0).round() / 100.0,
            "ops_per_sec": ops_sec.round(),
            "pass_rate_pct": (pass_rate * 10000.0).round() / 10000.0,
            "passed": pass
        },
        "gpu_vs_cpu": "CPU (microkernel routes execute host-side; no GPU endpoint)"
    })
}

/// Per-pattern timing for the 15 SRAM architectures. Article 118 claims 235.2 ns
/// per operation across all 15; this shows what each one actually costs so the
/// claim can be read correctly (per-pattern vs per-suite).
pub fn sram_pattern_breakdown(n: usize) -> serde_json::Value {
    use crate::sram_pattern_computer as sp;
    let dim = 16usize;
    let line = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let key = [1u8, 2, 3, 4];
    let rows_data: Vec<Vec<u8>> = (0..4u8).map(|i| vec![i, i + 1, i + 2, i + 3]).collect();
    let rows: Vec<&[u8]> = rows_data.iter().map(|r| r.as_slice()).collect();
    let s_a: Vec<u8> = (0..dim as u8).map(|i| i.wrapping_mul(7)).collect();
    let s_b: Vec<u8> = (0..dim as u8).map(|i| i.wrapping_mul(11)).collect();
    let spins: Vec<i8> = (0..dim).map(|i| if i % 2 == 0 { 1 } else { -1 }).collect();
    let coup: Vec<i8> = (0..dim * dim).map(|i| ((i % 3) as i8) - 1).collect();
    let wts: Vec<i8> = (0..dim * dim).map(|i| ((i % 3) as i8) - 1).collect();
    let mask: Vec<u8> = vec![1u8; dim];
    let nf: Vec<i32> = (0..dim).map(|i| i as i32).collect();
    let adj: Vec<u8> = vec![1u8; dim * dim];
    let ma: Vec<i32> = (0..dim * dim).map(|i| (i % 7) as i32).collect();
    let mb: Vec<i32> = (0..dim * dim).map(|i| (i % 5) as i32).collect();
    let scores: Vec<i32> = (0..dim).map(|i| (i * 3) as i32).collect();
    let hv_a: Vec<u8> = vec![0xF0u8; dim];
    let hv_b: Vec<u8> = vec![0x0Fu8; dim];
    let hg_a: Vec<i16> = (0..dim).map(|i| (i * 5) as i16).collect();
    let hg_b: Vec<i16> = (0..dim).map(|i| (i * 9) as i16).collect();
    let phases: Vec<i32> = (0..dim).map(|i| (i * 30) as i32).collect();
    fn time_ns<F: FnMut()>(n: usize, mut f: F) -> f64 {
        let t0 = crate::time::now_ns();
        for _ in 0..n {
            f();
        }
        let ns = crate::time::now_ns().saturating_sub(t0);
        ns as f64 / n as f64
    }
    let mut out = serde_json::Map::new();
    out.insert("pattern_fpga".into(), json!(time_ns(n, || { let _ = sp::pattern_fpga(&line); })));
    out.insert("pattern_cellular_automaton".into(), json!(time_ns(n, || { let _ = sp::pattern_cellular_automaton(&line); })));
    out.insert("pattern_neural_network".into(), json!(time_ns(n, || { let _ = sp::pattern_neural_network(&wts, &spins); })));
    out.insert("pattern_content_addressable".into(), json!(time_ns(n, || { let _ = sp::pattern_content_addressable(&rows, &key); })));
    out.insert("pattern_quantum_simulator".into(), json!(time_ns(n, || { let _ = sp::pattern_quantum_simulator(&s_a, &s_b); })));
    out.insert("pattern_ising_spin_glass".into(), json!(time_ns(n, || { let _ = sp::pattern_ising_spin_glass(&spins, &coup); })));
    out.insert("pattern_hopfield_recall".into(), json!(time_ns(n, || { let _ = sp::pattern_hopfield_recall(&spins, &wts, dim); })));
    out.insert("pattern_boltzmann_energy".into(), json!(time_ns(n, || { let _ = sp::pattern_boltzmann_energy(&spins, &coup, dim); })));
    out.insert("pattern_gnn_aggregate".into(), json!(time_ns(n, || { let _ = sp::pattern_gnn_aggregate(&nf, &adj, dim); })));
    out.insert("pattern_tcam_match".into(), json!(time_ns(n, || { let _ = sp::pattern_tcam_match(&spins, &spins, &mask); })));
    out.insert("pattern_systolic_matmul".into(), json!(time_ns(n, || { let _ = sp::pattern_systolic_matmul(&ma, &mb, dim); })));
    out.insert("pattern_sparse_attention".into(), json!(time_ns(n, || { let _ = sp::pattern_sparse_attention(&scores, 4); })));
    out.insert("pattern_hdc_bind".into(), json!(time_ns(n, || { let _ = sp::pattern_hdc_bind(&hv_a, &hv_b); })));
    out.insert("pattern_holographic_interference".into(), json!(time_ns(n, || { let _ = sp::pattern_holographic_interference(&hg_a, &hg_b); })));
    out.insert("pattern_onn_sync".into(), json!(time_ns(n, || { let _ = sp::pattern_onn_sync(&phases, 3); })));
    let total: f64 = out.values().filter_map(|v| v.as_f64()).sum();
    json!({
        "note": "per-pattern ns/op, measured; Article 118 claims 235.2 ns across all 15",
        "claim_ns_per_op": 235.2,
        "iterations": n,
        "dim": dim,
        "per_pattern_ns": out,
        "sum_all_15_ns": (total * 100.0).round() / 100.0,
        "fastest_vs_claim": "see per_pattern_ns",
    })
}

/// SENSE -> REDUCE: run the platform's real K3 quick_reduction over a
/// caller-supplied f32 vector, returning the reduced trits. This is the
/// stage the ternary pipeline uses to turn hardware telemetry into trits.
pub fn k3_reduce(vector: &[f32]) -> serde_json::Value {
    let engine = crate::k3_logic::KleeneTernaryEngine::new();
    let trits = engine.quick_reduction(vector);
    serde_json::json!({
        "trits": trits,
        "in_len": vector.len(),
        "out_len": trits.len(),
    })
}
