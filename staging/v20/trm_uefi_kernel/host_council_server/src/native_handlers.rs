// /native -- the proven native modules served as ROUTES, linked by path from the kernel's own source.
//
// Each of these modules was reachable only from a python caller, and before that from nothing at all: their
// strips are bit-exact against their oracles and wired into the python modules, but an operator had no route
// to ask them anything. These routes re-implement nothing -- they call the SAME kernel functions the strips
// wrap, so a route answer and a strip answer cannot disagree, and the python rule remains the oracle both
// were proved against.
//
// Honest limits, stated on the payloads rather than here:
//   * the persona prompt signature is SHA3-256 and this kernel tree has no SHA3, so the route returns the
//     UNSIGNED body and says the signature is computed by its python owner.
//   * the GPU pattern math is the design target's arithmetic; /system reports this host's own L3, which is a
//     different quantity.

use serde_json::{json, Value};
use tiny_http::{Header, Method, Request, Response};

use crate::chaos_engine as chaos;
use crate::docking_handlers as docking;
use crate::antibody_handlers as antibody;
use crate::drug_handlers as drug;
use crate::intel_handlers as intel;
use crate::fit_handlers as fit;
use crate::brain_handlers as brain;
use crate::expert_registry as experts;
use crate::gpu_infinity_computer as gpu;
use crate::persona_matrix as persona;
use crate::load_fuzz_handlers as lf;

fn ok(value: Value) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(value.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn err(code: u16, kind: &str, message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(json!({ "status": kind, "message": message }).to_string())
        .with_status_code(code)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn root() -> std::path::PathBuf {
    crate::kanban_handlers::repo_root()
}

fn read_json(rel: &str) -> Option<Value> {
    let raw = std::fs::read_to_string(root().join(rel)).ok()?;
    serde_json::from_str::<Value>(&raw).ok()
}

fn num(doc: &Value, key: &str) -> Option<f64> {
    doc.get(key).and_then(|v| v.as_f64())
}

fn strings(doc: &Value, key: &str) -> Vec<String> {
    doc.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
        .unwrap_or_default()
}

fn post(
    request: &mut Request,
    f: fn(&Value) -> Result<Value, String>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut raw = String::new();
    if request.as_reader().read_to_string(&mut raw).is_err() {
        return err(400, "unreadable_body", "the request body could not be read");
    }
    let doc: Value = if raw.trim().is_empty() {
        json!({})
    } else {
        match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => return err(400, "invalid_json", &e.to_string()),
        }
    };
    match f(&doc) {
        Ok(v) => ok(v),
        Err(m) => err(400, "invalid_request", &m),
    }
}

/// Which modules this namespace serves, with their source size MEASURED now and their parity figures read from
/// The operation table. ONE source for the route list a caller is handed (GET /native/routes), for the 405 and
/// 404 messages, and for the signature the ruler checks against the dispatch arms below -- so a route added to
/// the dispatcher without being added here (or vice versa) is a measurable difference, not a silent one.
pub const ROUTES: &[(&str, &str, &str)] = &[
    ("GET", "/native", "the module census: every entry measures its own source at request time"),
    ("GET", "/native/modules", "alias of /native"),
    ("GET", "/native/routes", "this table"),
    ("GET", "/native/gpu/architectures", "the measured Vulkan device list"),
    ("POST", "/native/chaos/drop", "the strip's drop decision for an explicit draw"),
    ("POST", "/native/load/reset", "a named balancer instance, from the source's own initial state"),
    ("POST", "/native/load/start", "the source's record_task_start"),
    ("POST", "/native/load/end", "the source's record_task_end"),
    ("POST", "/native/load/agent", "the source's get_agent_load"),
    ("POST", "/native/load/summary", "the source's get_swarm_load_summary"),
    ("POST", "/native/load/redistribute", "the source's redistribute_task"),
    ("POST", "/native/load/capable", "the source's get_capable_roles"),
    ("POST", "/native/fuzz/scan-lines", "the source's scan_lines_native per-line codes"),
    ("POST", "/native/fuzz/scan-text", "the source's scan_file step, over caller-supplied content"),
    ("POST", "/native/fuzz/scan-all", "the source's scan_all counters"),
    ("POST", "/native/fuzz/save-plan", "the source's save_antibodies dedup rule"),
    ("POST", "/native/chaos/clamp", "the strip's delay clamp"),
    ("POST", "/native/expert/resolve", "the registry's identifier resolution (an index, not an object)"),
    ("POST", "/native/persona/check", "the matrix's enforcement decision for one output"),
    ("POST", "/native/persona/prompt", "the compiled prompt body for a persona"),
    ("POST", "/native/gpu/pattern", "the design target's integer pattern math"),
    ("POST", "/native/drug/triage", "swarm-drug's triage"),
    ("POST", "/native/drug/sdf", "swarm-drug's SDF parse"),
    ("POST", "/native/docking/score-pair", "swarm-docking's pair score"),
    ("POST", "/native/docking/dock", "swarm-docking's conformer dock"),
    ("POST", "/native/antibody/chains", "list the chains of a SAbDab2 mmCIF complex"),
    ("POST", "/native/antibody/score", "score a candidate antibody's paratope against an antigen (SAbDab2 geometry)"),
    ("GET", "/native/intel/agents", "swarm-intel's agent roster"),
    ("POST", "/native/intel/route", "swarm-intel's domain routing"),
    ("POST", "/native/intel/vote", "swarm-intel's council vote"),
    ("POST", "/native/intel/decide", "swarm-intel's decide"),
    ("POST", "/native/intel/ensemble", "swarm-intel's ensemble field"),
    ("POST", "/native/fit/encode", "swarm-fit's K3 record encoding"),
    ("POST", "/native/fit/verdict", "swarm-fit's Booked/Blocked/Analyze rule"),
    ("POST", "/native/fit/serve", "swarm-fit's NAND append + read-back"),
    ("POST", "/native/brain/connectome", "swarm-brain's scale-free connectome store"),
    ("POST", "/native/brain/neural", "swarm-brain's sparse neural timestep"),
    ("POST", "/native/brain/collapse", "swarm-brain's K3 neuron collapse"),
    ("POST", "/native/brain/switch", "swarm-brain's four-architecture switch"),
];

fn route_list() -> String {
    ROUTES
        .iter()
        .map(|(m, p, _)| format!("{} {}", m, p))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn routes() -> Value {
    let ops: Vec<Value> = ROUTES
        .iter()
        .map(|(m, p, basis)| json!({ "method": m, "path": p, "basis": basis }))
        .collect();
    json!({
        "count": ops.len(),
        "operations": ops,
        "basis": "this table is the kernel's single source for the /native surface: the 404/405 messages are \
                  built from it and the ruler compares it against the dispatch arms",
    })
}

/// each module's own artifact. Nothing in this payload is a literal that could drift from the tree.
pub fn census() -> Value {
    let mods = [
        ("sram_pattern_computer", "trm_uefi_kernel/src/sram_pattern_computer.rs", "/sram",
         "swarm_v2_artifacts/route_probe.json"),
        ("gpu_infinity_computer", "trm_uefi_kernel/src/gpu_infinity_computer.rs", "/native/gpu/*",
         "swarm_v2_artifacts/gpu_strip_parity.json"),
        ("chaos_engine", "trm_uefi_kernel/src/chaos_engine.rs", "/native/chaos/*",
         "swarm_v2_artifacts/chaos_strip_parity.json"),
        ("expert_registry", "trm_uefi_kernel/src/expert_registry.rs", "/native/expert/*",
         "swarm_v2_artifacts/expert_strip_parity.json"),
        ("persona_matrix", "trm_uefi_kernel/src/persona_matrix.rs", "/native/persona/*",
         "swarm_v2_artifacts/persona_strip_parity.json"),
        // not a kernel module: the v18 crate itself decides, so its own source is the one measured here
        ("swarm_drug", "wsl_crates/trm-rust/swarm-drug/src/lib.rs", "/native/drug/*",
         "swarm_v2_artifacts/drug_wiring_ruler.json"),
        ("swarm_docking", "wsl_crates/trm-rust/swarm-docking/src/lib.rs", "/native/docking/*",
         "swarm_v2_artifacts/docking_wiring_ruler.json"),
        ("swarm_intel", "wsl_crates/trm-rust/swarm-intel/src/lib.rs", "/native/intel/*",
         "swarm_v2_artifacts/intel_wiring_ruler.json"),
        // the council and the ensemble field live in a second module of the same crate: measured too
        ("swarm_intel::ensemble_council", "wsl_crates/trm-rust/swarm-intel/src/ensemble_council.rs",
         "/native/intel/ensemble", "swarm_v2_artifacts/intel_wiring_ruler.json"),
        ("swarm_fit", "wsl_crates/trm-rust/swarm-fit/src/lib.rs", "/native/fit/*",
         "swarm_v2_artifacts/fit_wiring_ruler.json"),
        ("swarm_brain", "wsl_crates/trm-rust/swarm-brain/src/lib.rs", "/native/brain/*",
         "swarm_v2_artifacts/brain_wiring_ruler.json"),
        ("swarm_brain::neural", "wsl_crates/trm-rust/swarm-brain/src/neural.rs", "/native/brain/neural",
         "swarm_v2_artifacts/brain_wiring_ruler.json"),
        ("swarm_brain::pattern_switch", "wsl_crates/trm-rust/swarm-brain/src/pattern_switch.rs",
         "/native/brain/switch", "swarm_v2_artifacts/brain_wiring_ruler.json"),
        // kernel modules whose portable surface was a rename and whose aggregate surface was missing
        ("cognitive_load_balancer", "trm_uefi_kernel/src/cognitive_load_balancer.rs", "/native/load/*",
         "swarm_v2_artifacts/load_fuzz_parity_ruler.json"),
        ("self_fuzzing_engine", "trm_uefi_kernel/src/self_fuzzing_engine.rs", "/native/fuzz/*",
         "swarm_v2_artifacts/load_fuzz_parity_ruler.json"),
    ];
    let entries: Vec<Value> = mods
        .iter()
        .map(|(name, src, routes, artifact)| {
            let bytes = std::fs::metadata(root().join(src)).map(|m| m.len()).unwrap_or(0);
            let art = read_json(artifact);
            json!({
                "module": name,
                "source": src,
                "source_bytes": bytes,
                "routes": routes,
                "parity_artifact": artifact,
                "parity_checks_passed": art.as_ref().and_then(|v| v.get("passed")).and_then(|v| v.as_u64()),
                "parity_checks_failed": art.as_ref().and_then(|v| v.get("failed")).and_then(|v| v.as_u64()),
            })
        })
        .collect();
    json!({
        "protocol": "NATIVE_MODULE_ROUTES",
        "count": entries.len(),
        "modules": entries,
        "basis": "source_bytes is measured at request time and every parity figure is read from that module's \
                  own artifact; the python rule each module was proved against is the oracle of record",
        "limit": "the persona signature is not computed here (no SHA3 in this kernel tree): that route returns \
                  the unsigned body and names its owner",
    })
}

/// The architectures this namespace serves math for. The NAMES and per-architecture hashes are owned by the
/// generated strip, so they are pointed at rather than restated -- a second copy here could only drift.
fn gpu_architectures() -> Value {
    let cap = read_json("swarm_v2_artifacts/vulkan_capability.json");
    let vulkan = match &cap {
        Some(c) => {
            let devs: Vec<Value> = c
                .get("devices")
                .and_then(|d| d.get("devices"))
                .and_then(|d| d.as_array())
                .map(|a| {
                    a.iter()
                        .map(|d| {
                            json!({
                                "name": d.get("name"),
                                "type": d.get("type"),
                                "api_version": d.get("api_version"),
                                "device_local_bytes": d.get("device_local_bytes"),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            let w = c.get("write").cloned().unwrap_or(Value::Null);
            json!({
                "measured_by": "python vulkan_capability, through the system vulkan-1.dll loader",
                "devices": devs,
                "write": {
                    "ok": w.get("ok"),
                    "device": w.get("device"),
                    "bytes": w.get("bytes"),
                    "verified": w.get("verified"),
                    "write_us_median": w.get("write_us_median"),
                },
                "note": "this kernel does not run Vulkan -- a no_std image has no loader or driver -- so the route \
                         reports the probe's measurement, re-read from its artifact at request time rather than \
                         restated here",
            })
        }
        None => json!({
            "measured_by": "python vulkan_capability",
            "available": false,
            "reason": "no artifact at swarm_v2_artifacts/vulkan_capability.json; run the probe to measure",
        }),
    };
    json!({
        "arches": ["fpga", "neural", "cellular", "k3"],
        "math_source": "trm_uefi_kernel/src/gpu_infinity_computer.rs (this route calls it directly)",
        "names_and_hashes_owned_by": "swarm_v2_artifacts/native_voltage_strips/strip_gpu_infinity.rs, GENERATED \
                                      from the oracle's ARCHITECTURES_GPU; read them from strip_gpu_arch_name / \
                                      strip_gpu_arch_hash rather than restating them here",
        "note": "this route lists the architectures whose arithmetic it serves; the strip owns their identity",
        "vulkan": vulkan,
    })
}

/// The drop decision for an explicit (enabled, rate, sample) -- the same stateless function the python module
/// calls, so the route holds no engine state and cannot answer from a stale one.
fn chaos_drop(doc: &Value) -> Result<Value, String> {
    let enabled = doc.get("enabled").and_then(|v| v.as_bool()).ok_or("`enabled` (bool) is required")?;
    let rate = num(doc, "rate").ok_or("`rate` (number) is required")?;
    let sample = num(doc, "sample").ok_or("`sample` (number, the uniform draw) is required")?;
    let engine = chaos::ChaosEngine { network_delay_sec: 0.0, dropout_rate: rate, enabled };
    Ok(json!({
        "drop": engine.should_drop_connection_with(sample),
        "enabled": enabled,
        "rate": rate,
        "sample": sample,
        "basis": "the kernel's own should_drop_connection_with, the function strip_chaos_engine.dll wraps",
    }))
}

/// `set_dropout_rate` clamps to [0,1]; this returns both the request and what was stored, so a caller cannot
/// believe an out-of-range rate stuck.
fn chaos_clamp(doc: &Value) -> Result<Value, String> {
    let rate = num(doc, "rate").ok_or("`rate` (number) is required")?;
    let mut e = chaos::ChaosEngine::new();
    e.set_dropout_rate(rate);
    Ok(json!({
        "requested": rate,
        "stored": e.dropout_rate,
        "basis": "python: dropout_rate = max(0.0, min(1.0, rate)) -- the kernel's set_dropout_rate",
    }))
}

/// Resolve an identifier against a caller-supplied team, by the kernel's three-stage rule. Agents do not
/// cross this boundary: the route answers with an INDEX into the team the caller sent, exactly as the strip
/// does, and the caller keeps the object.
fn expert_resolve(doc: &Value) -> Result<Value, String> {
    let id = doc.get("identifier").and_then(|v| v.as_str()).ok_or("`identifier` (string) is required")?;
    let rows = doc.get("team").and_then(|v| v.as_array()).ok_or("`team` (array) is required")?;
    let (mut keys, mut names, mut roles) = (Vec::new(), Vec::new(), Vec::new());
    for (i, r) in rows.iter().enumerate() {
        let k = r.get("key").and_then(|v| v.as_str()).ok_or_else(|| format!("team[{}].key is required", i))?;
        keys.push(k.to_string());
        // accept {key, persona:{name,role}} or the flat {key, name, role}
        let p = r.get("persona").unwrap_or(r);
        names.push(p.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string());
        roles.push(p.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string());
    }
    let idx = experts::resolve_identifier(id, &keys, &names, &roles);
    Ok(json!({
        "index": idx,
        "resolved_key": if idx >= 0 { json!(keys[idx as usize]) } else { Value::Null },
        "sanitized_identifier": experts::sanitize_identifier(id),
        "team_size": keys.len(),
        "basis": "the kernel's three-stage rule (exact key, case-insensitive key, then persona name/role \
                  containment), the function strip_expert_registry.dll wraps; the ROW ORDER is the caller's \
                  because the first match wins",
    }))
}

fn state_name(code: i32) -> &'static str {
    match code {
        persona::PERSONA_IDLE => "IDLE",
        persona::PERSONA_VALIDATING => "VALIDATING",
        persona::PERSONA_GENERATING => "GENERATING",
        persona::PERSONA_PENALIZING => "PENALIZING",
        persona::PERSONA_VERIFYING => "VERIFYING",
        persona::PERSONA_FAILED => "FAILED",
        persona::PERSONA_RESET => "RESET",
        persona::PERSONA_LOCKED => "LOCKED",
        _ => "UNKNOWN",
    }
}

fn status_name(code: i32) -> &'static str {
    match code {
        persona::GEN_SUCCESS => "SUCCESS",
        persona::GEN_FAILED => "FAILED",
        persona::GEN_RESET => "RESET",
        _ => "UNKNOWN",
    }
}

/// The enforcement decision for one output. The clock is passed IN (`secs_since_reset`) rather than read, so
/// the same case is replayable here, in the strip, and in the python oracle.
fn persona_check(doc: &Value) -> Result<Value, String> {
    let output = doc.get("output").and_then(|v| v.as_str()).ok_or("`output` (string) is required")?;
    let forbidden = strings(doc, "forbidden");
    let max_violations = doc.get("max_violations").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    let locked = doc.get("locked").and_then(|v| v.as_bool()).unwrap_or(false);
    let secs = num(doc, "secs_since_reset").unwrap_or(0.0);
    let window = num(doc, "window").unwrap_or(60.0);
    let (state, status, content) =
        persona::enforce_decision(locked, secs, window, max_violations, output, &forbidden);
    Ok(json!({
        "violations": persona::count_violations(output, &forbidden),
        "state": state,
        "state_name": state_name(state),
        "status": status,
        "status_name": status_name(status),
        "content": content,
        "basis": "the kernel's count_violations + enforce_decision, the functions strip_persona_matrix.dll wraps",
        "notes": [
            "an empty entry in `forbidden` counts as a violation, as `'' in text` does in python",
            "state/status codes are the python enum.auto() values (IDLE=1 ... LOCKED=8, SUCCESS=1 ...)",
        ],
    }))
}

/// The compiled prompt BODY for a persona under a caller-supplied identity kernel -- unsigned, and it says so.
fn persona_prompt(doc: &Value) -> Result<Value, String> {
    let name = doc.get("persona").and_then(|v| v.as_str()).ok_or("`persona` (string) is required")?;
    let axioms = strings(doc, "axioms");
    let forbidden = strings(doc, "forbidden");
    let body = persona::compiled_prompt(name, &axioms, &forbidden);
    Ok(json!({
        "body": body,
        "body_bytes": body.len(),
        "signature": Value::Null,
        "signature_owner": "python -- persona_matrix.build_compiled_prompt",
        "signature_basis": "the signed form is `<!-- SIGNATURE:{sha3_256(body)} -->\\n{body}`; SHA3-256 is not \
                            implemented in this kernel tree, so the signature keeps a single owner rather \
                            than gaining a second implementation that could disagree",
    }))
}

/// Enforce a COUNCIL agent's persona (the 14-role identity table in council_personas) on one output.
fn council_check(doc: &Value) -> Result<Value, String> {
    let agent = doc.get("agent").and_then(|v| v.as_str()).ok_or("`agent` (string) is required")?;
    let output = doc.get("output").and_then(|v| v.as_str()).ok_or("`output` (string) is required")?;
    let locked = doc.get("locked").and_then(|v| v.as_bool()).unwrap_or(false);
    let secs = num(doc, "secs_since_reset").unwrap_or(0.0);
    crate::council_personas::check(agent, locked, secs, output).ok_or_else(|| {
        let known: Vec<&str> = crate::council_personas::PERSONAS.iter().map(|p| p.0).collect();
        format!("unknown agent `{agent}`; known: {}", known.join(", "))
    })
}

/// The compiled identity block for a council agent, from the table rather than a caller kernel.
fn council_prompt(doc: &Value) -> Result<Value, String> {
    let agent = doc.get("agent").and_then(|v| v.as_str()).ok_or("`agent` (string) is required")?;
    let body = crate::council_personas::compile(agent).ok_or_else(|| format!("unknown agent `{agent}`"))?;
    Ok(json!({
        "agent": agent,
        "body": body,
        "body_bytes": body.len(),
        "basis": "council_personas -> persona_matrix::compiled_prompt (unsigned; the signature keeps its single python owner)",
    }))
}

/// The pure-integer pattern math for one GPU architecture. This is the design target's arithmetic.
fn gpu_pattern(doc: &Value) -> Result<Value, String> {
    let arch = doc.get("arch").and_then(|v| v.as_str()).ok_or("`arch` is required: fpga|neural|cellular|k3")?;
    let size_mb = doc.get("size_mb").and_then(|v| v.as_u64()).unwrap_or(96);
    let pattern = match arch {
        "fpga" => {
            let (lines, lut) = gpu::sram_lut_fpga(size_mb);
            json!({ "logic_cells": lines, "lut_size": lut })
        }
        "neural" => {
            let (n, syn) = gpu::pattern_neural(size_mb);
            json!({ "neurons": n, "synapses": syn })
        }
        "cellular" => {
            let (cells, updates) = gpu::pattern_cellular(size_mb);
            json!({ "cells": cells, "updates_per_second": updates })
        }
        "k3" => {
            let (cells, trits, tput) = gpu::pattern_k3_ternary();
            json!({ "cells_used": cells, "trits": trits, "throughput_tps": tput })
        }
        other => return Err(format!("unknown arch `{}`; expected fpga, neural, cellular or k3", other)),
    };
    Ok(json!({
        "arch": arch,
        "size_mb": size_mb,
        "pattern": pattern,
        "basis": "the kernel's own pattern math, the functions strip_gpu_infinity.dll wraps",
        "note": "GPU_SPECS in the python module describes a DESIGN TARGET (RX 6700 XT, 96MB Infinity Cache); \
                 this is that target's arithmetic, not a measurement of this host's GPU",
    }))
}

pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    if !url.starts_with("/native") {
        return None;
    }
    let trimmed = url.split('?').next().unwrap_or(url).trim_end_matches('/');
    let path = if trimmed.is_empty() { "/native" } else { trimmed };
    let method = request.method().clone();
    Some(match (method, path) {
        (Method::Get, "/native") | (Method::Get, "/native/modules") => ok(census()),
        (Method::Get, "/native/routes") => ok(routes()),
        (Method::Get, "/native/gpu/architectures") => ok(gpu_architectures()),
        (Method::Get, "/native/phonic") => ok(crate::phonic::report(3.3)),
        (Method::Get, "/native/ambient") => ok(crate::ambient::report()),
        (Method::Get, "/native/shadow-bios") => ok(crate::shadow_bios::report()),
        (Method::Post, "/native/chaos/drop") => post(request, chaos_drop),
        (Method::Post, "/native/chaos/clamp") => post(request, chaos_clamp),
        (Method::Post, "/native/expert/resolve") => post(request, expert_resolve),
        (Method::Post, "/native/persona/check") => post(request, persona_check),
        (Method::Post, "/native/persona/prompt") => post(request, persona_prompt),
        (Method::Post, "/native/persona/council/check") => post(request, council_check),
        (Method::Post, "/native/persona/council/prompt") => post(request, council_prompt),
        (Method::Post, "/native/gpu/pattern") => post(request, gpu_pattern),
        (Method::Post, "/native/drug/triage") => post(request, drug::triage),
        (Method::Post, "/native/drug/sdf") => post(request, drug::sdf),
        (Method::Post, "/native/docking/score-pair") => post(request, docking::score_pair),
        (Method::Post, "/native/docking/dock") => post(request, docking::dock),
        (Method::Post, "/native/docking/contacts") => post(request, docking::contacts),
        (Method::Post, "/native/antibody/chains") => post(request, antibody::list_chains),
        (Method::Post, "/native/antibody/score") => post(request, antibody::score_cdr_antigen),
        (Method::Get, "/native/intel/agents") => ok(intel::agents()),
        (Method::Post, "/native/intel/route") => post(request, intel::route),
        (Method::Post, "/native/intel/vote") => post(request, intel::vote),
        (Method::Post, "/native/intel/decide") => post(request, intel::decide),
        (Method::Post, "/native/intel/ensemble") => post(request, intel::ensemble),
        (Method::Post, "/native/fit/encode") => post(request, fit::encode),
        (Method::Post, "/native/fit/verdict") => post(request, fit::verdict),
        (Method::Post, "/native/fit/serve") => post(request, fit::serve),
        (Method::Post, "/native/brain/connectome") => post(request, brain::connectome),
        (Method::Post, "/native/brain/neural") => post(request, brain::neural_step),
        (Method::Post, "/native/brain/collapse") => post(request, brain::collapse),
        (Method::Post, "/native/brain/switch") => post(request, brain::switch),
        (Method::Post, "/native/load/reset") => post(request, lf::load_reset),
        (Method::Post, "/native/load/start") => post(request, lf::load_start),
        (Method::Post, "/native/load/end") => post(request, lf::load_end),
        (Method::Post, "/native/load/agent") => post(request, lf::load_agent),
        (Method::Post, "/native/load/summary") => post(request, lf::load_summary),
        (Method::Post, "/native/load/redistribute") => post(request, lf::load_redistribute),
        (Method::Post, "/native/load/capable") => post(request, lf::load_capable),
        (Method::Post, "/native/fuzz/scan-lines") => post(request, lf::fuzz_scan_lines),
        (Method::Post, "/native/fuzz/scan-text") => post(request, lf::fuzz_scan_text),
        (Method::Post, "/native/fuzz/scan-all") => post(request, lf::fuzz_scan_all),
        (Method::Post, "/native/fuzz/save-plan") => post(request, lf::fuzz_save_plan),
        (Method::Get, _) => err(
            405,
            "method_not_allowed",
            &format!(
                "GET is served on {}; the rest are POST",
                ROUTES.iter()
                    .filter(|(m, _, _)| *m == "GET")
                    .map(|(_, p, _)| *p)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ),
        _ => err(
            404,
            "not_found",
            &format!("no such /native route; served: {}", route_list()),
        ),
    })
}
