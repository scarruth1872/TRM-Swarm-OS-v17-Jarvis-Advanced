// Layer 1 as routes: the kernel's own SRAM pattern computer, served to operators.
//
// README Layer 1: "15 Virtual SRAM Architectures: Dynamically reconfigures CPU L3 cache lines into 15
// domain-specific compute topologies." The implementations live in trm_uefi_kernel/src/sram_pattern_computer.rs
// and were reachable from nothing — no route, no crate op, no reference. This sibling exposes them: the 15
// topology descriptors, and the Hopfield RELAXATION, which is the recall dynamics the module previously lacked
// (its sram_lut_hopfield_recall is a single synchronous update, and one update is not recall).
//
// Everything is computed in-process from that module, linked by PATH rather than copied, so a /sram answer
// cannot drift from the kernel's own source.

use serde_json::{json, Value};
use tiny_http::{Header, Method, Request, Response};

use crate::sram_pattern_computer as sram;

/// The 15 topologies in the order the pattern strip uses, with the default parameter each factory is called
/// with. Table-driven: adding a topology is a row, not another branch.
const TOPOLOGIES: [(&str, fn(i64) -> sram::ArchDesc, i64); 15] = [
    ("fpga", sram::pattern_fpga, 1024),
    ("cellular", sram::pattern_cellular_automaton, 64),
    ("neural", sram::pattern_neural_network, 4096),
    ("cam", sram::pattern_content_addressable, 1024),
    ("quantum", sram::pattern_quantum_simulator, 12),
    ("ising", sram::pattern_ising_spin_glass, 2048),
    ("hopfield", sram::pattern_hopfield_attractor, 64),
    ("boltzmann", sram::pattern_boltzmann_machine, 512),
    ("gnn", sram::pattern_gnn_accelerator, 256),
    ("tcam", sram::pattern_tcam, 512),
    ("systolic", sram::pattern_systolic_array, 32),
    ("sparse_attention", sram::pattern_sparse_attention, 4096),
    ("hdc", sram::pattern_hyperdimensional_computing, 10000),
    ("holographic", sram::pattern_holographic_associative, 256),
    ("onn", sram::pattern_oscillatory_neural_network, 128),
];

fn ok(value: Value) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(value.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn err(code: u16, status: &str, message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(json!({ "status": status, "message": message }).to_string())
        .with_status_code(code)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// FNV-1a over the expanded pattern bytes — the same hash the deployment strip uses, so the two can be
/// compared instead of trusted independently.
fn fnv1a(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in bytes {
        h ^= *b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

fn desc_json(d: &sram::ArchDesc) -> Value {
    let mut m = serde_json::Map::new();
    let mut pattern: Option<(usize, u32)> = None;
    for (k, f) in &d.fields {
        let v = match f {
            sram::ArchField::I(i) => json!(i),
            sram::ArchField::S(s) => json!(s),
            sram::ArchField::W(s) => json!(s),
            sram::ArchField::L(items) => json!(items),
            sram::ArchField::Rep { period, times } => {
                let mut bytes = Vec::with_capacity(period.len() * times);
                for _ in 0..*times {
                    bytes.extend_from_slice(period);
                }
                pattern = Some((bytes.len(), fnv1a(&bytes)));
                json!({ "period_bytes": period.len(), "times": times, "pattern_len": bytes.len() })
            }
        };
        m.insert((*k).to_string(), v);
    }
    let mut out = Value::Object(m);
    if let Some((len, h)) = pattern {
        out["pattern_len"] = json!(len);
        out["pattern_fnv1a"] = json!(format!("0x{:08x}", h));
    }
    out
}

/// The L3 tier's geometry. The size is MEASURED on the host when the probe can reach the topology;
/// the module's constant is the fallback, and `l3_basis` says which of the two it served. The module
/// declares 32 MB and this host's L3 is 16 MB -- a number that reads as a measurement has to be one.
/// Every other field is derived from the size actually served, so the block cannot contradict itself.
///
/// This mirrors the module of record: `swarm_v2/core/sram_pattern_computer.py` probes the same chain
/// (`/system l3_sram_kb` -> the OS -> the documented target), reports `L3_SIZE_BASIS` and
/// `host_l3_matches_design_target`, and derives `L3_CELLS`/`L3_SETS`/`L3_TOTAL_TRANSISTORS` by the
/// same arithmetic. The Rust port hardcoded the literal; this is the port catching up to it.
pub(crate) fn l3_block() -> Value {
    let measured = crate::host_probe::l3_cache_kb().map(|kb| kb * 1024);
    let size = measured.unwrap_or(sram::L3_SIZE);
    let cells = size * 8; // one bit per cell, the module's own rule
    json!({
        "size_bytes": size,
        "cells": cells,
        "transistors": cells * sram::L3_TRANSISTORS_PER_CELL,
        "line_size": sram::L3_LINE_SIZE,
        "ways": sram::L3_WAYS,
        "slices": sram::L3_SLICES,
        "sets": size / (sram::L3_LINE_SIZE * sram::L3_WAYS * sram::L3_SLICES),
        "l3_basis": match measured {
            Some(_) => "measured: host_probe.rs -- unified L3 summed over the cache instances the \
                        host reports, which is not the same as one instance's size; cells, sets and \
                        transistors derived from it",
            None => "assumed: the module's declared 32 MB default -- the host probe could not read \
                     the cache topology on this target, so this is a declaration, not a measurement",
        },
        // The module of record spells this out too, and it is derived from the two values above.
        "host_l3_matches_design_target": size == sram::L3_SIZE,
        "declared_default": sram::L3_SIZE,
    })
}

fn architectures() -> Value {
    let list: Vec<Value> = TOPOLOGIES
        .iter()
        .map(|(name, f, arg)| {
            json!({ "index": TOPOLOGIES.iter().position(|(n, _, _)| n == name).unwrap(),
                    "name": name, "default_parameter": arg, "descriptor": desc_json(&f(*arg)) })
        })
        .collect();
    json!({
        "layer": "Layer 1 — Virtual SRAM Architectures",
        "count": list.len(),
        "l3": l3_block(),
        "architectures": list,
        "basis": "computed here from trm_uefi_kernel/src/sram_pattern_computer.rs (linked by path); \
                  pattern_fnv1a is FNV-1a over the expanded pattern bytes, the same hash the strip serves",
    })
}

/// The relaxation, from a JSON body. `patterns` (array of ±1 arrays) stores them Hebbian; `weights`
/// (dim×dim flat) skips storage. Returns the settled state and the evidence it settled.
fn relax(doc: &Value) -> Result<Value, String> {
    let dim = doc.get("dim").and_then(|v| v.as_u64()).unwrap_or(64) as usize;
    if dim == 0 || dim > 4096 {
        return Err(format!("dim must be 1..=4096 (got {})", dim));
    }
    let max_sweeps = doc.get("max_sweeps").and_then(|v| v.as_u64()).unwrap_or(64) as usize;
    if max_sweeps == 0 || max_sweeps > 4096 {
        return Err(format!("max_sweeps must be 1..=4096 (got {})", max_sweeps));
    }

    let read_state = |v: &Value, label: &str, want: usize| -> Result<Vec<i8>, String> {
        let a = v.as_array().ok_or_else(|| format!("{} must be an array of ±1 integers", label))?;
        if a.len() != want {
            return Err(format!("{} has {} entries, expected {}", label, a.len(), want));
        }
        a.iter()
            .map(|x| x.as_i64().map(|n| if n >= 0 { 1i8 } else { -1i8 })
                .ok_or_else(|| format!("{} must contain integers", label)))
            .collect()
    };

    let mut stored: Option<Vec<Vec<i8>>> = None;
    let weights: Vec<i8> = if let Some(w) = doc.get("weights") {
        let flat = read_state(w, "weights", dim * dim)?;
        flat
    } else if let Some(p) = doc.get("patterns") {
        let arr = p.as_array().ok_or("patterns must be an array of ±1 arrays")?;
        if arr.is_empty() {
            return Err("patterns is empty; a relaxation needs at least one stored pattern".into());
        }
        let pats: Result<Vec<Vec<i8>>, String> =
            arr.iter().map(|one| read_state(one, "a pattern", dim)).collect();
        let pats = pats?;
        let w = sram::hopfield_weights(&pats, dim);
        stored = Some(pats);
        w
    } else {
        return Err("provide patterns (array of ±1 arrays) to store, or weights (dim×dim flat)".into());
    };

    let state_len = doc.get("state").and_then(|s| s.as_array().map(|a| a.len())).unwrap_or(dim);
    let state: Vec<i8> = match doc.get("state") {
        Some(s) => read_state(s, "state", state_len)?,
        None => (0..dim).map(|i| if i % 3 == 0 { -1 } else { 1 }).collect(),
    };
    if state.len() > dim {
        return Err(format!("state has {} entries but dim is {}", state.len(), dim));
    }

    let r = sram::hopfield_relax(&state, &weights, dim, max_sweeps);
    let monotone = r.energy_trace.windows(2).all(|w| w[0] >= w[1]);
    let mut out = json!({
        "dim": dim,
        "max_sweeps": max_sweeps,
        "sweeps": r.sweeps,
        "converged": r.converged,
        "initial_energy": r.initial_energy,
        "final_energy": r.final_energy,
        "energy_monotone_nonincreasing": monotone,
        "energy_trace": r.energy_trace,
        "settled_state": r.state,
        "basis": "hopfield_relax in trm_uefi_kernel/src/sram_pattern_computer.rs, asynchronous \
                  (Gauss-Seidel) updates to a fixed point; energy is exact-integer E = -Σ_ij w_ij s_i s_j",
    });
    if let Some(pats) = stored {
        let distances: Vec<Value> = pats
            .iter()
            .enumerate()
            .map(|(i, p)| json!({ "pattern_index": i, "hamming_from_settled": sram::hopfield_hamming(&r.state, p),
                                  "recalled_exactly": sram::hopfield_hamming(&r.state, p) == 0 }))
            .collect();
        out["recall"] = json!({ "stored": pats.len(), "distances": distances });
    }
    Ok(out)
}

// ── POST /sram/execute — run a dataset payload through the matching native kernel ──
// The 15 `sram_lut_*` kernels in trm_uefi_kernel/src/sram_pattern_computer.rs accept explicit
// input arrays; this route is the surface that hands each one a real payload. Every arm names
// the kernel it ran and returns that kernel's exact output — nothing inferred, nothing mixed.

fn as_u8s(v: &Value, key: &str) -> Result<Vec<u8>, String> {
    let a = v.get(key)
        .ok_or_else(|| format!("missing \"{key}\" array"))?
        .as_array()
        .ok_or_else(|| format!("\"{key}\" must be an array"))?;
    a.iter().map(|x| x.as_i64().map(|n| n as u8).ok_or_else(|| format!("\"{key}\" entries must be integers"))).collect()
}
fn as_i8s(v: &Value, key: &str) -> Result<Vec<i8>, String> {
    let a = v.get(key)
        .ok_or_else(|| format!("missing \"{key}\" array"))?
        .as_array()
        .ok_or_else(|| format!("\"{key}\" must be an array"))?;
    a.iter().map(|x| x.as_i64().map(|n| n as i8).ok_or_else(|| format!("\"{key}\" entries must be integers"))).collect()
}
fn as_i16s(v: &Value, key: &str) -> Result<Vec<i16>, String> {
    let a = v.get(key)
        .ok_or_else(|| format!("missing \"{key}\" array"))?
        .as_array()
        .ok_or_else(|| format!("\"{key}\" must be an array"))?;
    a.iter().map(|x| x.as_i64().map(|n| n as i16).ok_or_else(|| format!("\"{key}\" entries must be integers"))).collect()
}
fn as_i32s(v: &Value, key: &str) -> Result<Vec<i32>, String> {
    let a = v.get(key)
        .ok_or_else(|| format!("missing \"{key}\" array"))?
        .as_array()
        .ok_or_else(|| format!("\"{key}\" must be an array"))?;
    a.iter().map(|x| x.as_i64().map(|n| n as i32).ok_or_else(|| format!("\"{key}\" entries must be integers"))).collect()
}
fn req_usize(v: &Value, key: &str) -> Result<usize, String> {
    v.get(key).and_then(|x| x.as_u64()).map(|n| n as usize)
        .ok_or_else(|| format!("\"{key}\" (a non-negative integer) is required"))
}

fn execute(doc: &Value) -> Result<Value, String> {
    let arch_raw = doc.get("architecture").and_then(|a| a.as_str())
        .ok_or("provide \"architecture\"")?.to_lowercase();
    let arch: &str = match arch_raw.as_str() {
        "cellular_automaton" => "cellular",
        "quantum_simulator" => "quantum",
        "ising_spin_glass" => "ising",
        "systolic_array" => "systolic",
        "sparse_transformer" | "sparse" => "sparse_attention",
        "hyperdimensional" => "hdc",
        "oscillatory" => "onn",
        "rbm" => "boltzmann",
        "content_addressable" => "cam",
        other => other,
    };
    let p: &Value = doc.get("payload").unwrap_or(doc);
    let (kernel, output): (&str, Value) = match arch {
        "fpga" => ("sram_lut_fpga", json!(sram::sram_lut_fpga(&as_u8s(p, "input")?))),
        "cellular" => ("sram_lut_cellular_automaton", json!(sram::sram_lut_cellular_automaton(&as_u8s(p, "input")?))),
        "neural" => ("sram_lut_neural_network", json!(sram::sram_lut_neural_network(&as_i8s(p, "weights")?, &as_i8s(p, "inputs")?))),
        "cam" => {
            let rows_raw = p.get("rows")
                .ok_or("missing \"rows\" (array of u8 arrays)")?
                .as_array()
                .ok_or("\"rows\" must be an array of arrays")?;
            let mut rows: Vec<Vec<u8>> = Vec::with_capacity(rows_raw.len());
            for r in rows_raw {
                let a = r.as_array().ok_or("each row must be an array of u8")?;
                let mut row = Vec::with_capacity(a.len());
                for x in a { row.push(x.as_i64().map(|n| n as u8).ok_or("row entries must be integers")?); }
                rows.push(row);
            }
            let key = as_u8s(p, "key")?;
            let refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
            ("sram_lut_content_addressable", json!(sram::sram_lut_content_addressable(&refs, &key)))
        }
        "quantum" => ("sram_lut_quantum_simulator", json!(sram::sram_lut_quantum_simulator(&as_u8s(p, "a")?, &as_u8s(p, "b")?))),
        "ising" => ("sram_lut_ising_spin_glass", json!(sram::sram_lut_ising_spin_glass(&as_i8s(p, "spins")?, &as_i8s(p, "couplings")?))),
        "hopfield" => {
            let dim = req_usize(p, "dim")?;
            ("sram_lut_hopfield_recall", json!(sram::sram_lut_hopfield_recall(&as_i8s(p, "state")?, &as_i8s(p, "weights")?, dim)))
        }
        "boltzmann" => {
            let dim = req_usize(p, "dim")?;
            ("sram_lut_boltzmann_energy", json!(sram::sram_lut_boltzmann_energy(&as_i8s(p, "spins")?, &as_i8s(p, "weights")?, dim)))
        }
        "gnn" => {
            let n = req_usize(p, "num_nodes")?;
            ("sram_lut_gnn_aggregate", json!(sram::sram_lut_gnn_aggregate(&as_i32s(p, "node_features")?, &as_u8s(p, "adj_matrix")?, n)))
        }
        "tcam" => ("sram_lut_tcam_match", json!(sram::sram_lut_tcam_match(&as_i8s(p, "stored_trits")?, &as_i8s(p, "query_trits")?, &as_u8s(p, "mask")?))),
        "systolic" => {
            let dim = req_usize(p, "dim")?;
            ("sram_lut_systolic_matmul", json!(sram::sram_lut_systolic_matmul(&as_i32s(p, "a")?, &as_i32s(p, "b")?, dim)))
        }
        "sparse_attention" => {
            let top_k = req_usize(p, "top_k")?;
            ("sram_lut_sparse_attention", json!(sram::sram_lut_sparse_attention(&as_i32s(p, "scores")?, top_k)))
        }
        "hdc" => ("sram_lut_hdc_bind", json!(sram::sram_lut_hdc_bind(&as_u8s(p, "hv_a")?, &as_u8s(p, "hv_b")?))),
        "holographic" => ("sram_lut_holographic_interference", json!(sram::sram_lut_holographic_interference(&as_i16s(p, "a")?, &as_i16s(p, "b")?))),
        "onn" => {
            let k = p.get("coupling_k").and_then(|x| x.as_i64()).unwrap_or(6) as i32;
            ("sram_lut_onn_sync", json!(sram::sram_lut_onn_sync(&as_i32s(p, "phases")?, k)))
        }
        _ => return Err(format!("unknown architecture \"{}\"; GET /sram/architectures lists the 15", arch_raw)),
    };
    Ok(json!({
        "status": "ok",
        "architecture": arch,
        "kernel": kernel,
        "output": output,
        "basis": format!("{kernel} in trm_uefi_kernel/src/sram_pattern_computer.rs (linked by path); exact input arrays -> exact kernel output"),
    }))
}

pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    if !url.starts_with("/sram") {
        return None;
    }
    let method = request.method().clone();

    if url.starts_with("/sram/architectures") {
        if method != Method::Get {
            return Some(err(405, "method_not_allowed", "GET /sram/architectures"));
        }
        return Some(ok(architectures()));
    }

    if url.starts_with("/sram/relax") {
        if method != Method::Post {
            return Some(err(405, "post_only",
                "POST a JSON body: {\"dim\":64,\"patterns\":[[±1,...]],\"state\":[±1,...],\"max_sweeps\":64}"));
        }
        let mut body = String::new();
        if request.as_reader().read_to_string(&mut body).is_err() {
            return Some(err(400, "unreadable_body", "could not read the request body"));
        }
        let doc: Value = match serde_json::from_str(&body) {
            Ok(d) => d,
            Err(e) => return Some(err(400, "invalid_json", &e.to_string())),
        };
        return Some(match relax(&doc) {
            Ok(v) => ok(v),
            Err(m) => err(400, "invalid_request", &m),
        });
    }

    if url.starts_with("/sram/execute") {
        if method != Method::Post {
            return Some(err(405, "post_only",
                "POST a JSON body: {\"architecture\":\"gnn\"|\"tcam\"|\"hdc\"|\"systolic\"|..., \"payload\":{...}}; GET /sram/architectures lists the 15"));
        }
        let mut body = String::new();
        if request.as_reader().read_to_string(&mut body).is_err() {
            return Some(err(400, "unreadable_body", "could not read the request body"));
        }
        let doc: Value = match serde_json::from_str(&body) {
            Ok(d) => d,
            Err(e) => return Some(err(400, "invalid_json", &e.to_string())),
        };
        return Some(match execute(&doc) {
            Ok(v) => ok(v),
            Err(m) => err(400, "invalid_request", &m),
        });
    }

    Some(err(404, "not_found",
        "no such /sram route; try GET /sram/architectures, POST /sram/relax, or POST /sram/execute"))
}
