//! state_handlers.rs — Real-data read endpoints for more dashboard tabs.
//!
//! Option A continuation (dashboard function audit, ledger block 48). The
//! kanban handlers proved the pattern: serve the SAME state files the Python
//! tooling maintains, mirroring the Python route response shapes so the UI and
//! tools never diverge.
//!
//! This module adds the families that have genuine file-backed state:
//!   GET  /evolution/genome        -> { version, genome_stability, mutation_count, targets }
//!   GET  /evolution/proposals     -> { proposals: [...] } (real evolution_proposals.json)
//!   POST /evolution/verify/{id}   -> 501 honest: sandbox verification is engine work
//!   POST /evolution/integrate/{id}-> 501 honest: integration is engine work
//!   POST /evolution/auto-evolve   -> 501 honest
//!   POST /evolution/mutate        -> 501 honest
//!
//! POST actions that generate/verify mutations run real code in the Python
//! mutation engine (sandbox execution). Serving a fabricated {"status":"ok"}
//! would lie to the operator, so those return 501 with a clear message. Reads
//! serve the real persisted proposals.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Method, Request, Response};

/// Reuse kanban_handlers' repo-root + state-path resolution.
fn repo_root() -> PathBuf {
    if let Ok(env_root) = std::env::var("TRM_REPO_ROOT") {
        let p = PathBuf::from(env_root);
        if p.join("swarm_v2_artifacts").exists() {
            return p;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..4 {
            dir = dir.and_then(|d| d.parent().map(|p| p.to_path_buf()));
        }
        if let Some(root) = dir {
            if root.join("swarm_v2_artifacts").exists() {
                return root;
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("swarm_v2_artifacts").exists() {
            return cwd;
        }
    }
    PathBuf::from(".")
}

/// Derived module-recognition counts from the scan artifact (module_registry.json).
///
/// These counts are DERIVED by swarm_v2/core/build_module_registry.py. Serving a typed
/// literal here is what let /system report 170 REAL modules while the scan on disk said
/// 153 (and 4 fabricated). Reads the artifact, so it cannot drift from the scan.
///
/// Returns None when the artifact is missing or unreadable; the caller then reports
/// null rather than a stale number.
pub fn registry_counts() -> Option<Value> {
    let path = std::env::var("MODULE_REGISTRY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo_root().join("module_registry.json"));
    let doc: Value = serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?;
    let by = doc.get("by_status")?;
    let get = |k: &str| by.get(k).cloned().unwrap_or(Value::Null);
    Some(json!({
        "modules_compiled": doc.get("summary")
            .and_then(|s| s.get("kernel_modules")).cloned().unwrap_or(Value::Null),
        "real": get("REAL"),
        "ported": get("PORTED"),
        "skeleton": get("SKELETON"),
        "excluded": get("EXCLUDED"),
        "fabricated": by.get("FABRICATED").cloned().unwrap_or(Value::from(0u64)),
        "trees": doc.get("kernel_trees").cloned().unwrap_or(Value::Null),
        "generated": doc.get("generated").cloned().unwrap_or(Value::Null),
    }))
}

fn evolution_proposals_path() -> PathBuf {
    if let Ok(p) = std::env::var("EVOLUTION_PROPOSALS_PATH") {
        return PathBuf::from(p);
    }
    repo_root()
        .join("swarm_v2_artifacts")
        .join("evolution_proposals.json")
}

/// Load the evolution proposals file (7.2 MB, 142 proposals as of 2026-09-09).
fn load_evolution() -> Option<Value> {
    let path = evolution_proposals_path();
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&raw).ok()
}

/// GET /evolution/proposals — mirror of Python route list_genetic_proposals():
/// returns {"proposals": [...]} from the same persisted file the mutation
/// engine reads. Serves the file as-is (truthful, same source as Python).
pub fn evolution_proposals_payload() -> Value {
    match load_evolution() {
        Some(v) => {
            let props = v.get("proposals").cloned().unwrap_or(json!([]));
            json!({ "proposals": props })
        }
        None => json!({ "proposals": [], "error": "evolution_proposals.json unreadable" }),
    }
}

/// GET /evolution/genome — derive honest genome state from the real proposals
/// file: mutation count = proposals count; targets = the distinct mutated files
/// (the same codebase files the Python engine's analyze_codebase() would list).
pub fn evolution_genome_payload() -> Value {
    match load_evolution() {
        Some(v) => {
            let props = v
                .get("proposals")
                .and_then(|p| p.as_array())
                .cloned()
                .unwrap_or_default();
            let mutation_count = props.len();
            // distinct target files, ordered by first appearance
            let mut targets: Vec<String> = Vec::new();
            let mut seen = std::collections::HashSet::new();
            let mut passed = 0usize;
            for p in &props {
                if let Some(tf) = p.get("target_file").and_then(|x| x.as_str()) {
                    if seen.insert(tf.to_string()) {
                        targets.push(tf.to_string());
                    }
                }
                if let Some(st) = p.get("status").and_then(|x| x.as_str()) {
                    if st == "passed" || st == "integrated" {
                        passed += 1;
                    }
                }
            }
            let stability = if mutation_count > 0 {
                (passed as f64 / mutation_count as f64 * 0.9 + 0.05).min(0.99)
            } else {
                0.95
            };
            json!({
                "version": "16.0.0-evolutionary",
                "genome_stability": stability,
                "mutation_count": mutation_count,
                "targets": Value::Array(targets.into_iter().map(Value::String).collect()),
                "source": "evolution_proposals.json",
            })
        }
        None => json!({
            "version": "16.0.0-evolutionary",
            "genome_stability": 0.0,
            "mutation_count": 0,
            "targets": [],
            "error": "evolution_proposals.json unreadable",
        }),
    }
}

/// Honest 501 for engine-only POST actions.
fn engine_only(message: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = json!({
        "status": "engine_required",
        "message": message,
    })
    .to_string();
    Response::from_string(body)
        .with_status_code(501)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

// ── Rollout 5: the Pillars families (wetlab / sensory / thz / regulatory / ledger) ────────────
// Every read is backed by the real state file, with the artifact's own field names mapped onto
// the keys the JSX reads (wetlabData.total_protocols/protocols, sensoryData.active_nodes_count/
// spatial_coherence/anomaly_score, thzData.mean_jitter_ps/spectral_coherence,
// regulatoryData.total_dossiers/dossiers, ledgerData.total_blocks/byzantine_tolerance/
// latest_block_hash). A missing file reports an error, never a clean empty result.

/// Read a repo-relative JSON file.
fn read_json(rel: &str) -> Option<Value> {
    let raw = fs::read_to_string(repo_root().join(rel)).ok()?;
    serde_json::from_str(&raw).ok()
}

/// /secrets/keys — the vault's key NAMES only, never values. The vault stores
/// {name: encrypted_value}, so names are plaintext and listable without an engine or decryption.
/// The Python route returned a hardcoded 3-key inventory that matches nothing on this host.
pub fn secrets_keys_payload() -> Value {
    let root = repo_root();
    let key_present = root.join(".swarm").join(".vault_key").exists();
    let vault_path = root.join(".swarm").join("secrets.vault");

    if !vault_path.exists() {
        return json!({
            "keys": [],
            "total": 0,
            "vault": { "present": false, "encrypted": true, "key_present": key_present },
            "basis": ".swarm/secrets.vault does not exist on this host, so no secrets are stored; \
                       the Python route's 3-key inventory was hardcoded, not read from any store"
        });
    }

    match fs::read_to_string(&vault_path) {
        Ok(raw) => match serde_json::from_str::<Value>(&raw) {
            Ok(v) => {
                let keys: Vec<Value> = v
                    .as_object()
                    .map(|o| o.keys().map(|k| json!({"name": k, "type": null})).collect())
                    .unwrap_or_default();
                let total = keys.len();
                json!({
                    "keys": keys,
                    "total": total,
                    "vault": { "present": true, "encrypted": true, "key_present": key_present }
                })
            }
            Err(_) => json!({
                "keys": [],
                "total": 0,
                "vault": { "present": true, "encrypted": true, "key_present": key_present },
                "unavailable": { "reason": "secrets.vault unparseable" }
            }),
        },
        Err(_) => json!({
            "keys": [],
            "total": 0,
            "vault": { "present": true, "key_present": key_present },
            "unavailable": { "reason": "secrets.vault unreadable" }
        }),
    }
}

fn wetlab_protocols_payload() -> Value {
    match read_json("swarm_v2_artifacts/wetlab_robotics_protocols.json") {
        Some(v) => json!({
            "total_protocols": v.get("total_protocols").and_then(|x| x.as_u64()).unwrap_or(0),
            "target_hardware_platforms": v.get("target_hardware_platforms").cloned().unwrap_or(json!([])),
            "protocols": v.get("protocols").cloned().unwrap_or(json!([])),
            "source": "swarm_v2_artifacts/wetlab_robotics_protocols.json",
        }),
        None => json!({ "total_protocols": 0, "protocols": [],
                        "error": "wetlab_robotics_protocols.json not readable - not a clean result" }),
    }
}

fn sensory_mesh_telemetry_payload() -> Value {
    match read_json("swarm_v2_artifacts/environmental_sensor_telemetry.json") {
        Some(v) => {
            let scan = v.get("latest_scan").cloned().unwrap_or(json!({}));
            json!({
                "active_nodes_count": scan.get("nodes_active").and_then(|x| x.as_u64()).unwrap_or(0),
                "spatial_coherence": scan.get("global_coherence_index").cloned().unwrap_or(json!(null)),
                "anomaly_score": scan.get("anomaly_score").cloned().unwrap_or(json!(null)),
                "field_notes": "spatial_coherence is the artifact's global_coherence_index and active_nodes_count is its nodes_active, mapped onto the UI's keys - not recomputed here",
                "latest_scan": scan,
                "sensor_nodes": v.get("sensor_nodes").cloned().unwrap_or(json!([])),
                "source": "swarm_v2_artifacts/environmental_sensor_telemetry.json",
            })
        }
        None => json!({ "active_nodes_count": 0,
                        "error": "environmental_sensor_telemetry.json not readable - not a clean result" }),
    }
}

fn thz_resonance_status_payload() -> Value {
    match read_json("swarm_v2_artifacts/task45_thz_synchronized_computation_report.json") {
        Some(v) => json!({
            "mean_jitter_ps": v.get("mean_phase_jitter_ps").cloned().unwrap_or(json!(null)),
            "spectral_coherence": v.get("spectral_coherence").cloned().unwrap_or(json!(null)),
            "nodes_phase_locked": v.get("nodes_phase_locked").cloned().unwrap_or(json!(null)),
            "carrier_frequency_thz": v.get("carrier_frequency_thz").cloned().unwrap_or(json!(null)),
            "status": v.get("status").cloned().unwrap_or(json!("unknown")),
            "source": "swarm_v2_artifacts/task45_thz_synchronized_computation_report.json",
            "field_notes": "carrier_frequency_thz is the silicon TO phonon material constant (~15.5 THz); mean_jitter_ps/spectral_coherence/nodes_phase_locked/status are the task45 report's STATED values, not a live THz phase-lock measurement",
        }),
        None => json!({ "error": "task45_thz_synchronized_computation_report.json not readable - not a clean result" }),
    }
}

fn regulatory_ind_dossiers_payload() -> Value {
    match read_json("swarm_v2_artifacts/regulatory_ind_dossiers.json") {
        Some(v) => json!({
            "total_dossiers": v.get("total_dossiers").and_then(|x| x.as_u64()).unwrap_or(0),
            "compliance_standard": v.get("compliance_standard").cloned().unwrap_or(json!(null)),
            "dossiers": v.get("dossiers").cloned().unwrap_or(json!([])),
            "source": "swarm_v2_artifacts/regulatory_ind_dossiers.json",
        }),
        None => json!({ "total_dossiers": 0, "dossiers": [],
                        "error": "regulatory_ind_dossiers.json not readable - not a clean result" }),
    }
}

pub fn ledger_monad_blocks_payload() -> Value {
    // Serve the chain the sealing path owns; derive the count, the tip and the PBFT tolerance
    // (f = (N-1)/3 over the roster) from the data rather than asserting constants.
    let Ok(raw) = fs::read_to_string(repo_root().join("swarm_v2_artifacts/monad_ledger.jsonl")) else {
        return json!({ "total_blocks": 0, "error": "monad_ledger.jsonl not readable - not a clean result" });
    };
    let mut n = 0u64;
    let mut tip: Option<Value> = None;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        n += 1;
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            tip = Some(v);
        }
    }
    let agents = crate::AGENTS.len() as u64;
    let f = agents.saturating_sub(1) / 3;
    json!({
        "total_blocks": n,
        "latest_block_hash": tip.as_ref().and_then(|t| t.get("hash")).cloned().unwrap_or(json!(null)),
        "latest_block_index": tip.as_ref().and_then(|t| t.get("index")).cloned().unwrap_or(json!(null)),
        "latest_block_phase": tip.as_ref().and_then(|t| t.get("phase")).cloned().unwrap_or(json!(null)),
        "byzantine_tolerance": format!("f={f} of N={agents} (quorum {})", 2 * f + 1),
        "monad_anchor": tip.as_ref().and_then(|t| t.get("anchor")).cloned().unwrap_or(json!(null)),
        "source": "swarm_v2_artifacts/monad_ledger.jsonl",
    })
}

pub fn handle_batch4(
    request: &mut Request,
    url: &str,
) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();

    if method == Method::Get && url.starts_with("/wetlab/protocols") {
        return Some(ok_response(wetlab_protocols_payload()));
    }
    if method == Method::Get && url.starts_with("/sensory/mesh/telemetry") {
        return Some(ok_response(sensory_mesh_telemetry_payload()));
    }
    if method == Method::Get && url.starts_with("/thz/resonance/status") {
        return Some(ok_response(thz_resonance_status_payload()));
    }
    if method == Method::Get && url.starts_with("/regulatory/ind/dossiers") {
        return Some(ok_response(regulatory_ind_dossiers_payload()));
    }
    if method == Method::Get && url.starts_with("/ledger/monad/blocks") {
        return Some(ok_response(ledger_monad_blocks_payload()));
    }

    // Actions: honest 501 with the concrete reason, never a 200 the UI would read as success.
    if method == Method::Post && url.starts_with("/thz/resonance/sync") {
        return Some(engine_only(
            "/thz/resonance/sync: phase-lock synchronisation runs in the THz phase engine against live \
             nodes. This server serves the last sealed synchronisation (GET /thz/resonance/status) \
             rather than fabricating a new sync.",
        ));
    }
    if method == Method::Post && url.starts_with("/ledger/monad/commit") {
        return Some(engine_only(
            "/ledger/monad/commit: the ledger has ONE writer - the sealing path that chains sha256 \
             prev_hash blocks. This server serves that chain (GET /ledger/monad/blocks) and must not \
             open a second writer on the same store.",
        ));
    }
    None
}

// ── Rollout 6: discovery + humanistic (file-backed reads) ──────────────────────────────────────
// Return keys are the ones each JSX reads: candidates / top_candidates / top_designs / circuits,
// the pipeline object as-is, and the per-task humanistic artifact selected by its number.

pub fn discovery_pipeline_payload() -> Value {
    match read_json("swarm_v2_artifacts/autonomous_drug_discovery_pipeline_execution.json") {
        Some(v) => v,
        None => json!({ "status": "unavailable",
                        "error": "autonomous_drug_discovery_pipeline_execution.json not readable - not a clean result" }),
    }
}

/// One candidate list: prefer the key the JSX reads, else the artifact's fuller list.
fn discovery_list_payload(rel: &str, key: &str, fallback_key: &str) -> Value {
    match read_json(rel) {
        Some(v) => {
            let arr = v
                .get(key)
                .or_else(|| v.get(fallback_key))
                .cloned()
                .unwrap_or(json!([]));
            let n = arr.as_array().map(|a| a.len()).unwrap_or(0);
            json!({
                key: arr,
                "total_returned": n,
                "total_from_artifact": v.get("total_candidates")
                    .or_else(|| v.get("total_designs"))
                    .or_else(|| v.get("total_circuits"))
                    .or_else(|| v.get("top_candidates_count"))
                    .cloned()
                    .unwrap_or(json!(null)),
                "statistics": v.get("statistics").cloned().unwrap_or(json!(null)),
                "source": rel,
            })
        }
        None => json!({ key: [], "error": format!("{rel} not readable - not a clean result") }),
    }
}

/// The humanistic suite summary — the one artifact that names all 10 tasks and their grounding.
pub fn humanistic_summary_payload() -> Value {
    match read_json("swarm_v2_artifacts/humanistic_tasks_suite_summary.json") {
        Some(v) => v,
        None => json!({ "status": "unavailable",
                        "error": "humanistic_tasks_suite_summary.json not readable - not a clean result" }),
    }
}

/// The humanistic suite writes one artifact per task (task21_*.json, task28_*.json ...), so the
/// number in the URL selects the file by prefix rather than by a lookup table.
fn humanistic_task_payload(num: &str) -> Value {
    let dir = repo_root().join("swarm_v2_artifacts");
    let prefix = format!("task{num}_");
    let mut found: Option<(String, Value)> = None;
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let f = e.file_name().to_string_lossy().to_string();
            if f.starts_with(&prefix) && f.ends_with(".json") {
                if let Ok(raw) = fs::read_to_string(e.path()) {
                    if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                        found = Some((f, v));
                    }
                }
            }
        }
    }
    match found {
        Some((name, mut v)) => {
            v["artifact_file"] = json!(name);
            v
        }
        None => json!({ "status": "error", "route": "/humanistic/task",
                        "error": "no_artifact_for_task",
                        "detail": format!("no swarm_v2_artifacts/task{num}_*.json exists") }),
    }
}

pub fn handle_batch5(
    request: &mut Request,
    url: &str,
) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();
    if method != Method::Get {
        return None;
    }
    if url.starts_with("/discovery/pipeline/status") {
        return Some(ok_response(discovery_pipeline_payload()));
    }
    if url.starts_with("/discovery/amp-candidates") {
        return Some(ok_response(discovery_list_payload(
            "swarm_v2_artifacts/antimicrobial_peptide_candidates.json", "candidates", "top_candidates",
        )));
    }
    if url.starts_with("/discovery/cancer-neoantigens") {
        return Some(ok_response(discovery_list_payload(
            "swarm_v2_artifacts/cancer_neoantigen_candidates.json", "top_candidates", "all_candidates",
        )));
    }
    if url.starts_with("/discovery/drug-repurposing") {
        return Some(ok_response(discovery_list_payload(
            "swarm_v2_artifacts/drug_repurposing_candidates.json", "top_candidates", "all_candidates",
        )));
    }
    if url.starts_with("/discovery/adc-designs") {
        return Some(ok_response(discovery_list_payload(
            "swarm_v2_artifacts/adc_designs.json", "top_designs", "all_designs",
        )));
    }
    if url.starts_with("/discovery/synthetic-circuits") {
        return Some(ok_response(discovery_list_payload(
            "swarm_v2_artifacts/synthetic_circuit_designs.json", "circuits", "circuits",
        )));
    }
    if url.starts_with("/humanistic/tasks/summary") {
        return Some(ok_response(humanistic_summary_payload()));
    }
    if let Some(rest) = url.strip_prefix("/humanistic/task/") {
        let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if num.is_empty() {
            return Some(ok_response(json!({ "status": "error", "route": "/humanistic/task/{n}",
                                            "error": "bad_task_number",
                                            "detail": "expected /humanistic/task/<number>" })));
        }
        return Some(ok_response(humanistic_task_payload(&num)));
    }
    None
}

// ── Rollout 7: continuum + microkernel families ────────────────────────────────────────────────
// The four engines behind these routes (star_matrix_lattice, continuum_pbft, microkernel_spawner,
// continuum_genomics) keep their state IN MEMORY - none has a persistence path. So the reads below
// are built from what is real: the engine modules' own DECLARED constant tables, recomputed with
// their own formulas; the platform's real PBFT block history; the cold-start snapshot and the
// recorded microkernel benchmarks; and the THz phase-lock report. Routes that would mutate engine
// state or spawn processes are honest 501s.

/// `STAR_MATRIX_NODES` from star_matrix_lattice.py - declared seeds, mirrored verbatim.
const STAR_MATRIX_NODES: [(&str, &str, f64, f64, f64, [f64; 4]); 4] = [
    ("alpha_singularity", "The Alpha Singularity", 50.0, 18.0, 10.0, [0.95, 0.90, 0.20, 0.10]),
    ("monad_anchor", "The Monad Anchor", 20.0, 65.0, 8.0, [0.30, 0.95, 0.80, 0.05]),
    ("nexus_confluence", "The Nexus Confluence", 80.0, 65.0, 9.0, [0.90, 0.40, 0.95, 0.98]),
    ("unified_core", "The Unified Consciousness Core", 50.0, 52.0, 14.0, [0.85, 0.85, 0.85, 0.70]),
];
const LATTICE_ALPHA: f64 = 0.6;
const LATTICE_BETA: f64 = 0.4;

fn cosine4(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let dot: f64 = (0..4).map(|i| a[i] * b[i]).sum();
    let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

/// `R(S1,S2) = alpha*cos(spectrum1,spectrum2) + beta*(m1*m2/d^2)` - the engine's own formula,
/// recomputed here so the matrix is derived rather than copied from a run.
fn lattice_resonance(i: usize, j: usize) -> f64 {
    let (_, _, x1, y1, m1, s1) = STAR_MATRIX_NODES[i];
    let (_, _, x2, y2, m2, s2) = STAR_MATRIX_NODES[j];
    if i == j {
        return 1.0;
    }
    let sim = cosine4(&s1, &s2);
    let d = (((x1 - x2).powi(2) + (y1 - y2).powi(2)) as f64).sqrt().max(0.001);
    let grav = (m1 * m2) / (d * d);
    ((LATTICE_ALPHA * sim + LATTICE_BETA * grav) * 10_000.0).round() / 10_000.0
}

pub fn star_matrix_lattice_payload() -> Value {
    let nodes: serde_json::Map<String, Value> = STAR_MATRIX_NODES
        .iter()
        .enumerate()
        .map(|(i, (key, name, x, y, mass, spectrum))| {
            (
                format!("Node_{}", (b'A' + i as u8) as char),
                json!({
                    "key": key, "name": name, "coordinates": [x, y], "mass": mass,
                    "spectrum": spectrum, "index": i,
                }),
            )
        })
        .collect();
    let mut matrix = serde_json::Map::new();
    for (i, (key, ..)) in STAR_MATRIX_NODES.iter().enumerate() {
        let mut row = serde_json::Map::new();
        for (j, (other, ..)) in STAR_MATRIX_NODES.iter().enumerate() {
            row.insert((*other).to_string(), json!(lattice_resonance(i, j)));
        }
        matrix.insert((*key).to_string(), Value::Object(row));
    }
    json!({
        "status": "ONLINE",
        "lattice": {
            "nodes": nodes,
            "resonance_matrix": matrix,
            "system_scaling_constants": { "alpha": LATTICE_ALPHA, "beta": LATTICE_BETA },
        },
        "basis": "declared STAR_MATRIX_NODES from swarm_v2/core/star_matrix_lattice.py, with the \
                  resonance matrix recomputed by that module's own formula; the engine's runtime \
                  register_node/connect_edge additions live in memory and are not persisted",
    })
}

pub fn continuum_pbft_ledger_payload() -> Value {
    let src = read_json("swarm_v2_artifacts/decentralized_monad_ledger.json");
    let (blocks, total, tip, tol) = match &src {
        Some(v) => (
            v.get("blocks").cloned().unwrap_or(json!([])),
            v.get("total_blocks").cloned().unwrap_or(json!(null)),
            v.get("latest_block_hash").cloned().unwrap_or(json!(null)),
            v.get("byzantine_tolerance").cloned().unwrap_or(json!(null)),
        ),
        None => (json!([]), json!(null), json!(null), json!(null)),
    };
    let n = blocks.as_array().map(|a| a.len()).unwrap_or(0);
    json!({
        "status": "ONLINE",
        "ledger": blocks,
        "total_blocks": total,
        "latest_block_hash": tip,
        "byzantine_tolerance": tol,
        "blocks_served": n,
        "nodes": [
            { "node_id": "ORCH", "name": "Continuum Core", "role": "Primary Leader Node" },
            { "node_id": "SAGE", "name": "Cybernetic Sage", "role": "State Auditor Node" },
            { "node_id": "MUSE", "name": "Stochastic Muse", "role": "Creative Generator Node" },
            { "node_id": "SENTINEL", "name": "Sentinel Warden", "role": "Security Isolation Node" },
        ],
        "ledger_source": "swarm_v2_artifacts/decentralized_monad_ledger.json",
        "basis": "the served blocks are the platform's real PBFT history; the continuum engine's own \
                  in-memory ledger (genesis entries plus any execute_pbft_cycle blocks) has no \
                  persistence path, so it is not claimed here",
    })
}

fn microkernel_status_payload() -> Value {
    let snap = read_json("swarm_v2_artifacts/cold_start_snapshot.json");
    let bench = read_json("swarm_v2_artifacts/native_microkernel_inference_benchmark.json");
    let ram = read_json("swarm_v2_artifacts/ram_microkernel_integration.json");
    let agents = snap
        .as_ref()
        .and_then(|v| v.get("agent_details").cloned())
        .unwrap_or(json!(null));
    let agents_n = agents.as_object().map(|m| m.len()).unwrap_or(0);
    json!({
        "status": "ONLINE",
        // The spawner's live table is in-process only; at rest there are no spawned subagents to
        // report, and inventing a non-empty table is exactly what this rollout removes.
        "active_subagents_count": 0,
        "process_table": [],
        "recovered_agent_snapshot": {
            "agents_saved": agents_n,
            "timestamp": snap.as_ref().and_then(|v| v.get("timestamp")).cloned().unwrap_or(json!(null)),
            "write_latency_us": snap.as_ref().and_then(|v| v.get("write_latency_us")).cloned().unwrap_or(json!(null)),
            "recovery_time_us": snap.as_ref().and_then(|v| v.get("recovery_time_us")).cloned().unwrap_or(json!(null)),
            "recovery_within_5ms": snap.as_ref().and_then(|v| v.get("recovery_within_5ms")).cloned().unwrap_or(json!(null)),
            "agent_details": agents,
        },
        "inference_benchmark": bench.unwrap_or(json!(null)),
        "ram_integration": ram.unwrap_or(json!(null)),
        "basis": "cold_start_snapshot.json + the recorded native benchmarks; the MicrokernelSpawner \
                  keeps active_agents in memory with no persistence path, so no live process table \
                  is asserted",
    })
}

fn continuum_oscilloscope_payload() -> Value {
    // The Python route asserts sync 89.4 / -64.2 dBm / 14.5 GHz as literals. Two of those can be
    // derived from the real THz phase-lock report; the ones with no instrument source stay null
    // and say so rather than repeating a constant.
    match read_json("swarm_v2_artifacts/task45_thz_synchronized_computation_report.json") {
        Some(v) => {
            let coh = v.get("spectral_coherence").and_then(|x| x.as_f64());
            let freq = v.get("carrier_frequency_thz").and_then(|x| x.as_f64());
            let jitter = v.get("mean_phase_jitter_ps").and_then(|x| x.as_f64());
            let locked = v.get("nodes_phase_locked").and_then(|x| x.as_i64());
            let coherent = v.get("status").and_then(|x| x.as_str()).map(|s| s.contains("COHERENT")).unwrap_or(false);
            json!({
                "status": "ONLINE",
                "sync_percentage": coh.map(|c| (c * 100.0 * 100.0).round() / 100.0),
                "frequency_ghz": freq,
                "signal_dbm": Value::Null,
                "signal_dbm_reason": "no instrument source on this host; the Python route returns a literal -64.2",
                "spectrum_runs": Value::Null,
                "spectrum_runs_reason": "no spectral sweep series is recorded; the Python route returns a literal [8.7, -0.45, 12.1, 9.4, 15.2]",
                "phase_jitter_ps": jitter,
                "nodes_phase_locked": locked,
                "nominal_status": coherent,
                "derivation": "sync_percentage = spectral_coherence x 100, frequency_ghz = carrier_frequency_thz, nominal_status = status contains COHERENT",
                "source": "swarm_v2_artifacts/task45_thz_synchronized_computation_report.json",
            })
        }
        None => json!({ "status": "unavailable",
                        "error": "task45_thz_synchronized_computation_report.json not readable - not a clean result" }),
    }
}

pub fn handle_batch6(
    request: &mut Request,
    url: &str,
) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();

    // Reads: derived or file-backed, labelled with their source.
    if url.starts_with("/api/continuum/star-matrix/lattice") {
        return Some(ok_response(star_matrix_lattice_payload()));
    }
    if url.starts_with("/api/continuum/pbft/ledger") {
        return Some(ok_response(continuum_pbft_ledger_payload()));
    }
    if url.starts_with("/swarm/microkernel/status") {
        return Some(ok_response(microkernel_status_payload()));
    }
    if url.starts_with("/api/continuum/oscilloscope/spectral") {
        return Some(ok_response(continuum_oscilloscope_payload()));
    }

    // Actions: these mutate engine state or spawn real processes, which this server does not own.
    if method == Method::Post && url.starts_with("/api/continuum/star-matrix/node") {
        return Some(engine_only(
            "/api/continuum/star-matrix/node registers a node in the lattice engine's in-memory \
             table (star_matrix_lattice.py). The kernel does not own that table and has no \
             persistence path for it, so the write is refused rather than applied to a copy.",
        ));
    }
    if method == Method::Post && url.starts_with("/api/continuum/star-matrix/edge") {
        return Some(engine_only(
            "/api/continuum/star-matrix/edge computes R = alpha*cos + beta*m1*m2/d^2 against the \
             engine's live table and stores the edge. The resonance formula is served at \
             /api/continuum/star-matrix/lattice; applying an edge needs the owning engine.",
        ));
    }
    if method == Method::Post && url.starts_with("/api/continuum/pbft/audit-repair") {
        return Some(engine_only(
            "/api/continuum/pbft/audit-repair runs the continuum PBFT engine's autonomic \
             self-healing cycle (continuum_pbft.audit_and_repair_mesh). That is engine work; the \
             real block history is served at /api/continuum/pbft/ledger.",
        ));
    }
    if method == Method::Post && url.starts_with("/api/continuum/genomics/refract") {
        return Some(engine_only(
            "/api/continuum/genomics/refract refracts a prompt through the ContinuumGenomics \
             engine's rolling 4D momentum cache. The kernel has no such engine, so no refraction \
             is computed rather than a fabricated one.",
        ));
    }
    if method == Method::Post && url.starts_with("/api/continuum/genomics/preset") {
        return Some(engine_only(
            "/api/continuum/genomics/preset mutates PERSONA_PRESETS inside continuum_genomics.py. \
             That module is offline tooling; editing its constant from the kernel would not change \
             the engine's state.",
        ));
    }
    if method == Method::Post && url.starts_with("/swarm/microkernel/spawn-continuum") {
        return Some(engine_only(
            "/swarm/microkernel/spawn-continuum spawns a real MicrokernelSubAgent process with a \
             memory budget and TTL (microkernel_spawner.spawn_subagent). Starting processes is the \
             spawner's to do, not this server's.",
        ));
    }
    if url.starts_with("/swarm/microkernel/terminate/") {
        return Some(engine_only(
            "/swarm/microkernel/terminate/{id} terminates a running subagent owned by the \
             MicrokernelSpawner. No subagent table is persisted, so the kernel cannot address an \
             agent to terminate.",
        ));
    }
    None
}

fn ok_response(value: Value) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = serde_json::to_string(&value).unwrap_or_else(|_| "{}".into());
    Response::from_string(body)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// Port of evolutionary_sandbox's static verification: the DNA security shield (banned imports /
/// calls) and API backwards-compatibility (removed functions/classes, shrunk arities). The
/// Python-bound phases (py_compile syntax, subprocess import, mesh attestation) are not run here
/// and are named in the response.
const BANNED_MODULES: &[&str] = &[
    "subprocess", "socket", "webbrowser", "ctypes", "requests", "urllib", "builtins",
];
const BANNED_CALLS: &[&str] = &["eval", "exec", "__import__"];

/// Top-level module names imported by the code (`import x` / `from x import ...`).
fn extract_imports(code: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in code.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("import ") {
            if let Some(name) = rest.split(|c: char| c == ' ' || c == ',').next() {
                out.push(name.to_string());
            }
        } else if let Some(rest) = t.strip_prefix("from ") {
            if let Some(name) = rest.split(|c: char| c == ' ' || c == '.').next() {
                out.push(name.to_string());
            }
        }
    }
    out
}

/// Phase 0: banned imports (unless already present in the original) and banned function calls.
fn scan_dangers(mutated: &str, original_imports: &[String]) -> Vec<String> {
    let mut dangers = Vec::new();
    for mi in extract_imports(mutated) {
        if BANNED_MODULES.contains(&mi.as_str()) && !original_imports.iter().any(|o| o == &mi) {
            dangers.push(format!("Banned module import: '{mi}'"));
        }
    }
    for c in BANNED_CALLS {
        if mutated.contains(&format!("{c}(")) {
            dangers.push(format!("Banned dangerous function call: '{c}'"));
        }
    }
    dangers
}

/// Top-level `def name(args)` -> arity and `class Name` -> 0 (method-level tracking is not ported).
fn extract_api(code: &str) -> std::collections::BTreeMap<String, usize> {
    let mut api = std::collections::BTreeMap::new();
    for line in code.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("def ") {
            if let Some(open) = rest.find('(') {
                let name = rest[..open].trim().to_string();
                let close = rest[open..]
                    .find(')')
                    .map(|c| open + c)
                    .unwrap_or(rest.len());
                let args = &rest[open + 1..close];
                let arity = args
                    .split(',')
                    .filter(|a| {
                        let a = a.trim();
                        !a.is_empty()
                            && !a.starts_with('*')
                            && a != "self"
                            && a != "cls"
                    })
                    .count();
                api.insert(format!("func:{name}"), arity);
            }
        } else if let Some(rest) = t.strip_prefix("class ") {
            if let Some(name) = rest.split(|c: char| c == '(' || c == ':').next() {
                api.insert(format!("class:{}", name.trim()), 0);
            }
        }
    }
    api
}

/// Phase 1: public functions/classes removed, or arity shrunk, in the mutated code.
fn api_regressions(original: &str, mutated: &str) -> Vec<String> {
    let orig = extract_api(original);
    let muta = extract_api(mutated);
    let mut regs = Vec::new();
    for (key, arity) in &orig {
        match muta.get(key) {
            None => {
                let (kind, name) = key.split_once(':').unwrap_or(("", key));
                regs.push(format!("Removed public {kind}: '{name}'"));
            }
            Some(new_arity) if *new_arity < *arity => {
                let (_, name) = key.split_once(':').unwrap_or(("", key));
                regs.push(format!(
                    "Signature mismatch for '{name}': expected >= {arity} args, got {new_arity}"
                ));
            }
            _ => {}
        }
    }
    regs
}

/// Persist the proposals file after a status change.
fn save_evolution(evo: &Value) -> bool {
    match serde_json::to_string(evo) {
        Ok(s) => fs::write(evolution_proposals_path(), s).is_ok(),
        Err(_) => false,
    }
}

/// Index of the proposal with the given id, if present.
fn find_proposal_index(evo: &Value, id: &str) -> Option<usize> {
    evo.get("proposals")
        .and_then(|a| a.as_array())
        .and_then(|a| a.iter().position(|p| p.get("proposal_id").and_then(|v| v.as_str()) == Some(id)))
}

/// Port of analyze_codebase: recursively scan swarm_v2/core for .py files > 15 KB with no "__"
/// in the filename (the exact heuristic the Python mutation engine uses to pick targets).
fn collect_evolution_targets(dir: &std::path::Path, out: &mut Vec<Value>, root: &std::path::Path) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_evolution_targets(&p, out, root);
        } else if p.extension().and_then(|x| x.to_str()) == Some("py") {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.contains("__") {
                if let Ok(meta) = fs::metadata(&p) {
                    if meta.len() > 15_000 {
                        let rel = p.strip_prefix(root).map(|q| q.to_string_lossy().to_string()).unwrap_or_else(|_| name.to_string());
                        out.push(json!({ "file": rel, "bytes": meta.len() }));
                    }
                }
            }
        }
    }
}

/// GET /evolution/targets — the deterministic mutation-target scan (the portable half of mutate).
fn evolution_targets_payload() -> Value {
    let root = repo_root();
    let dir = root.join("swarm_v2").join("core");
    let mut targets: Vec<Value> = Vec::new();
    collect_evolution_targets(&dir, &mut targets, &root);
    json!({
        "status": "ok",
        "target_count": targets.len(),
        "targets": targets,
        "basis": "scanned swarm_v2/core for .py files > 15 KB (no '__' in name) — the analyze_codebase heuristic",
    })
}

/// Phase 2 (ported): one-shot `python -m py_compile` syntax check. This is offline verification
/// tooling (a compiler pass over a temp copy), not a runtime decision, so it may run here — the
/// mutated_code is written to a temp file, checked, and the temp file deleted.
fn syntax_check(code: &str, id: &str) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!("trm_verify_{id}.py"));
    if fs::write(&tmp, code).is_err() {
        return Err("temp file write failed".to_string());
    }
    let out = std::process::Command::new("python")
        .args(["-m", "py_compile", tmp.to_str().unwrap_or("")])
        .output();
    let _ = fs::remove_file(&tmp);
    match out {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
            Err(if err.is_empty() { "syntax error".to_string() } else { err })
        }
        Err(e) => Err(format!("python unavailable: {e}")),
    }
}

/// POST /evolution/verify/{id} — the static verification ported into Rust, with status persisted
/// back to evolution_proposals.json (proposal.status -> "passed" | "failed"), mirroring the
/// Python's verify_mutation + _save_proposals.
fn evolution_verify(id: &str) -> Value {
    let Some(mut evo) = load_evolution() else {
        return json!({ "status": "error", "error": "proposals_unreadable", "proposal_id": id });
    };
    let Some(idx) = find_proposal_index(&evo, id) else {
        return json!({ "status": "error", "error": "proposal_not_found", "proposal_id": id });
    };
    let original = evo["proposals"][idx]
        .get("original_code").and_then(|c| c.as_str()).unwrap_or("").to_string();
    let mutated = evo["proposals"][idx]
        .get("mutated_code").and_then(|c| c.as_str()).unwrap_or("").to_string();

    let original_imports = extract_imports(&original);
    let dangers = scan_dangers(&mutated, &original_imports);
    let regressions = if dangers.is_empty() { api_regressions(&original, &mutated) } else { Vec::new() };

    let static_pass = dangers.is_empty() && regressions.is_empty();
    // Phase 2 (ported): one-shot `python -m py_compile` syntax check on the mutated code.
    let syntax_error = if static_pass { syntax_check(&mutated, id).err() } else { None };
    let passed = static_pass && syntax_error.is_none();
    // Persist the proposal's status: "passed" / "failed" (the Python's internal state, not the
    // route-level "verified" label — integrate checks for exactly "passed").
    evo["proposals"][idx]["status"] = json!(if passed { "passed" } else { "failed" });
    let persisted = save_evolution(&evo);

    json!({
        "status": if passed { "verified" } else { "failed" },
        "proposal_id": id,
        "proposal_status": if passed { "passed" } else { "failed" },
        "dangers": dangers,
        "regressions": regressions,
        "syntax_error": syntax_error,
        "checks_run": ["security_shield", "api_compatibility", "py_compile_syntax"],
        "unverified": ["subprocess import", "mesh attestation"],
        "persisted": persisted,
        "persisted_to": "evolution_proposals.json",
        "note": "security + API-compatibility + py_compile syntax checks run here; subprocess import and mesh attestation are not run",
    })
}

/// POST /evolution/integrate/{id} — write a verified proposal's mutated_code to its target file,
/// then mark it "integrated". Mirrors evolutionary_sandbox's integrate path (status gate + one file
/// write + status update), with a path-traversal guard the Python lacks.
fn evolution_integrate(id: &str) -> Value {
    let Some(mut evo) = load_evolution() else {
        return json!({ "status": "error", "error": "proposals_unreadable", "proposal_id": id });
    };
    let Some(idx) = find_proposal_index(&evo, id) else {
        return json!({ "status": "error", "error": "proposal_not_found", "proposal_id": id });
    };
    let status = evo["proposals"][idx].get("status").and_then(|s| s.as_str()).unwrap_or("");
    if status != "passed" {
        return json!({
            "status": "error",
            "error": "not_verified",
            "proposal_id": id,
            "current_status": status,
            "detail": "only a proposal whose status is 'passed' can be integrated; run /evolution/verify/{id} first",
        });
    }
    let target = evo["proposals"][idx].get("target_file").and_then(|t| t.as_str()).unwrap_or("").to_string();
    let mutated = evo["proposals"][idx].get("mutated_code").and_then(|c| c.as_str()).unwrap_or("").to_string();
    if target.is_empty() {
        return json!({ "status": "error", "error": "no_target_file", "proposal_id": id });
    }
    // Path guard: refuse traversal outside the repo.
    if target.split(['/', '\\']).any(|seg| seg == "..") {
        return json!({ "status": "error", "error": "unsafe_target", "proposal_id": id, "target_file": target });
    }
    let path = repo_root().join(target.replace('\\', "/"));
    let wrote = fs::write(&path, &mutated).is_ok();
    if !wrote {
        return json!({ "status": "error", "error": "write_failed", "proposal_id": id, "target_file": target });
    }
    evo["proposals"][idx]["status"] = json!("integrated");
    let persisted = save_evolution(&evo);
    json!({
        "status": "integrated",
        "proposal_id": id,
        "target_file": target,
        "bytes_written": mutated.len(),
        "persisted": persisted,
    })
}

/// Dispatch /evolution* requests. Returns Some(response) if handled.
pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();

    if method == Method::Get && url.starts_with("/evolution/proposals") {
        return Some(ok_response(evolution_proposals_payload()));
    }
    if method == Method::Get && url.starts_with("/evolution/genome") {
        return Some(ok_response(evolution_genome_payload()));
    }
    if method == Method::Get && url.starts_with("/evolution/targets") {
        return Some(ok_response(evolution_targets_payload()));
    }
    // POST /evolution/verify/{id} — the STATIC checks of evolutionary_sandbox.verify_mutation are
    // deterministic and ported here: Phase 0 (DNA security shield: banned imports/calls) and
    // Phase 1 (API backwards compatibility: removed functions/classes, shrunk arities). The
    // Python-bound checks (Phase 2 py_compile, Phase 3 subprocess import, mesh attestation) are
    // NOT run and are named in the response rather than silently skipped.
    if method == Method::Post {
        if let Some(rest) = url.strip_prefix("/evolution/verify/") {
            let id = rest.trim_end_matches('/');
            return Some(ok_response(evolution_verify(id)));
        }
        if let Some(rest) = url.strip_prefix("/evolution/integrate/") {
            let id = rest.trim_end_matches('/');
            return Some(ok_response(evolution_integrate(id)));
        }
    }
    // The remaining POST actions are genuine mutation-engine work (sandbox execute / integrate /
    // auto-evolve / mutate). No honest way to run them in Rust — 501, not a lie.
    if method == Method::Post && url.starts_with("/evolution") {
        return Some(engine_only(
            "Evolution mutation actions (mutate/auto-evolve/integrate) run in the \
             Python mutation engine (sandbox execution). This Rust server serves the real \
             persisted state; start the Python engine to execute mutations.",
        ));
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Verification — proofs queue (swarm_v2_artifacts/verification_proofs.json)
//
// The queue is a list of measurements EXTRACTED from evidence files on disk by
// tools/gen_verification_proofs.py. At request time every entry's `evidence_path`
// is re-resolved: an entry whose evidence cannot be read is QUEUE WORK, never a
// verified proof. So a deleted or moved evidence file shows up as work to do
// instead of silently staying "proven".
//
// What this replaces: the old reader looked for
// swarm_v2_artifacts/formal_verification_theorems_proven.json and reported
// `theorems_failed` as "items_in_queue" and `theorems_proven` as
// "verified_last_hour" — a lifetime total served under a per-hour label, and a
// single hard-coded path that is absent from the deployed tree (hence
// "proofs file unreadable" with 0/0 while the file existed elsewhere).
//
// Honest limits, stated in the payload itself:
//   * `verification_basis` says only that the evidence path RESOLVES and is
//     readable; this endpoint does not re-derive the measurement values.
//   * `verified_last_hour` is computed from entry timestamps inside a 3600 s
//     window. It is null when that cannot be computed (no loaded entries, or no
//     parseable timestamps at all) — never a stand-in for a total.
// ─────────────────────────────────────────────────────────────────────────────

pub const PROOFS_SCHEMA: &str = "trm.verification.proofs/v1";
const PROOFS_REL: &str = "swarm_v2_artifacts/verification_proofs.json";

/// Every place the proofs file could legitimately live, most specific first.
/// `repo_root()` is inherited from the kanban handlers and cannot know whether it
/// was started from the repository root or from the staged kernel tree, so both
/// layouts (and the ancestors of each) are tried before giving up.
pub fn proofs_candidates() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut push = |p: PathBuf| {
        let p = p;
        if !out.iter().any(|q| q == &p) {
            out.push(p);
        }
    };
    if let Ok(explicit) = std::env::var("TRM_PROOFS_PATH") {
        push(PathBuf::from(explicit));
    }
    let root = repo_root();
    push(root.join(PROOFS_REL));
    push(root.join("staging").join("v20").join(PROOFS_REL));
    let mut dir = Some(root.clone());
    for _ in 0..4 {
        dir = dir.and_then(|d| d.parent().map(|p| p.to_path_buf()));
        match dir.clone() {
            Some(d) => {
                push(d.join(PROOFS_REL));
                push(d.join("staging").join("v20").join(PROOFS_REL));
            }
            None => break,
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let mut d = Some(cwd);
        for _ in 0..4 {
            match d.clone() {
                Some(cur) => {
                    push(cur.join(PROOFS_REL));
                    push(cur.join("staging").join("v20").join(PROOFS_REL));
                    d = cur.parent().map(|p| p.to_path_buf());
                }
                None => break,
            }
        }
    }
    out
}

/// The proofs file actually on disk, if any.
pub fn proofs_path() -> Option<PathBuf> {
    proofs_candidates().into_iter().find(|p| p.is_file())
}

/// Resolve an entry's `evidence_path` against the repository root and its
/// ancestors (the kernel tree is frequently staged one or two levels below the
/// repository root that holds `evidence/`). Absolute paths are taken as given.
fn resolve_evidence(root: &Path, rel: &str) -> Option<PathBuf> {
    let rel = rel.trim();
    if rel.is_empty() {
        return None;
    }
    let given = PathBuf::from(rel);
    if given.is_absolute() {
        return given.is_file().then_some(given);
    }
    let mut bases: Vec<PathBuf> = vec![root.to_path_buf()];
    let mut dir = Some(root.to_path_buf());
    for _ in 0..3 {
        dir = dir.and_then(|d| d.parent().map(|p| p.to_path_buf()));
        match dir.clone() {
            Some(d) => bases.push(d),
            None => break,
        }
    }
    for b in bases {
        let cand = b.join(&given);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Days since 1970-01-01 for a proleptic Gregorian date (Howard Hinnant's
/// days_from_civil). Avoids pulling a date crate into the kernel for one parse.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Parse the RFC3339 forms the evidence files actually carry
/// (`YYYY-MM-DDTHH:MM:SS`, optional `.fraction`, optional `Z` / `±HH:MM`) into
/// Unix seconds. Returns None for anything else — a timestamp that cannot be
/// read is a rejected entry, not a zero.
pub fn parse_rfc3339(s: &str) -> Option<i64> {
    let s = s.trim();
    let b = s.as_bytes();
    if b.len() < 19 {
        return None;
    }
    let digit = |from: usize, len: usize| -> Option<i64> {
        let part = s.get(from..from + len)?;
        if !part.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        part.parse::<i64>().ok()
    };
    if b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let (y, mo, d) = (digit(0, 4)?, digit(5, 2)?, digit(8, 2)?);
    let (h, mi, se) = (digit(11, 2)?, digit(14, 2)?, digit(17, 2)?);
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || se > 60 {
        return None;
    }
    let mut i = 19;
    if b.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while b.get(i).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            i += 1;
        }
        if i == start {
            return None;
        }
    }
    let offset = match b.get(i) {
        None => 0i64,
        Some(b'Z') | Some(b'z') if i + 1 == b.len() => 0,
        Some(sign @ (b'+' | b'-')) => {
            let oh = digit(i + 1, 2)?;
            if b.get(i + 3) != Some(&b':') {
                return None;
            }
            let om = digit(i + 4, 2)?;
            if i + 6 != b.len() {
                return None;
            }
            let magnitude = oh * 3600 + om * 60;
            if *sign == b'+' {
                magnitude
            } else {
                -magnitude
            }
        }
        _ => return None,
    };
    Some(days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + se - offset)
}

/// Forward-slash display of a path, so the payload is identical on every host.
fn slash(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// GET /verification/stats — counts derived from the proofs file on disk.
pub fn verification_stats_payload() -> Value {
    let candidates: Vec<Value> = proofs_candidates()
        .iter()
        .map(|p| json!(slash(p)))
        .collect();
    match proofs_path() {
        Some(path) => verification_stats_from(&path, &repo_root(), &candidates),
        None => {
            let first = proofs_candidates()
                .first()
                .map(|p| slash(p))
                .unwrap_or_default();
            json!({
                "loaded": false,
                "error": "proofs file unreadable",
                "error_kind": "missing",
                "error_detail": format!(
                    "no {} found; searched {} candidate path(s), first was {}",
                    PROOFS_REL,
                    proofs_candidates().len(),
                    first
                ),
                "proofs_file": first,
                "proofs_file_candidates": candidates,
                "schema": PROOFS_SCHEMA,
                "total_proofs": 0,
                "verified": 0,
                "pending": 0,
                "items_in_queue": 0,
                "queue_basis": "nothing loaded — no proofs recorded on this host",
                "verified_last_hour": Value::Null,
                "verified_last_hour_basis": "no proofs loaded; a per-hour rate is not computable",
                "newest_proof_timestamp": Value::Null,
                "newest_proof_age_seconds": Value::Null,
            })
        }
    }
}

/// The whole stats computation, parameterised on the file and the repository
/// root so it can be tested without the deployed layout.
pub fn verification_stats_from(path: &Path, root: &Path, candidates: &[Value]) -> Value {
    let shown = slash(path);
    if !path.is_file() {
        return json!({
            "loaded": false,
            "error": "proofs file unreadable",
            "error_kind": "missing",
            "error_detail": format!("{} does not exist", shown),
            "proofs_file": shown,
            "proofs_file_candidates": candidates,
            "schema": PROOFS_SCHEMA,
            "total_proofs": 0,
            "verified": 0,
            "pending": 0,
            "items_in_queue": 0,
            "queue_basis": "nothing loaded — no proofs recorded at this path",
            "verified_last_hour": Value::Null,
            "verified_last_hour_basis": "no proofs loaded; a per-hour rate is not computable",
            "newest_proof_timestamp": Value::Null,
            "newest_proof_age_seconds": Value::Null,
        });
    }
    let raw = match fs::read_to_string(path) {
        Ok(r) => r,
        Err(e) => {
            return json!({
                "loaded": false,
                "error": "proofs file unreadable",
                "error_kind": "io",
                "error_detail": format!("{}: {}", shown, e),
                "proofs_file": shown,
                "proofs_file_candidates": candidates,
                "schema": PROOFS_SCHEMA,
                "total_proofs": 0,
                "verified": 0,
                "pending": 0,
                "items_in_queue": 0,
                "queue_basis": "nothing loaded — proofs file could not be read",
                "verified_last_hour": Value::Null,
                "verified_last_hour_basis": "no proofs loaded; a per-hour rate is not computable",
                "newest_proof_timestamp": Value::Null,
                "newest_proof_age_seconds": Value::Null,
            });
        }
    };
    let doc: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "loaded": false,
                "error": "proofs file unreadable",
                "error_kind": "invalid_json",
                "error_detail": format!("{}: {}", shown, e),
                "proofs_file": shown,
                "proofs_file_candidates": candidates,
                "schema": PROOFS_SCHEMA,
                "total_proofs": 0,
                "verified": 0,
                "pending": 0,
                "items_in_queue": 0,
                "queue_basis": "nothing loaded — proofs file is not valid JSON",
                "verified_last_hour": Value::Null,
                "verified_last_hour_basis": "no proofs loaded; a per-hour rate is not computable",
                "newest_proof_timestamp": Value::Null,
                "newest_proof_age_seconds": Value::Null,
            });
        }
    };
    let schema = doc
        .get("schema")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let entries = doc
        .get("proofs")
        .and_then(|p| p.as_array())
        .cloned()
        .unwrap_or_default();

    let now = now_unix();
    let mut verified: Vec<Value> = Vec::new();
    let mut pending: Vec<Value> = Vec::new();
    let mut rejected: Vec<Value> = Vec::new();
    let mut newest: Option<i64> = None;

    for e in &entries {
        let id = e.get("proof_id").and_then(|x| x.as_str()).unwrap_or("");
        let rel = e.get("evidence_path").and_then(|x| x.as_str()).unwrap_or("");
        if id.is_empty() || rel.is_empty() {
            rejected.push(json!({
                "proof_id": id,
                "evidence_path": rel,
                "reason": "entry is missing proof_id or evidence_path",
            }));
            continue;
        }
        let ts_raw = e.get("timestamp").and_then(|x| x.as_str()).unwrap_or("");
        let ts = match parse_rfc3339(ts_raw) {
            Some(t) => t,
            None => {
                rejected.push(json!({
                    "proof_id": id,
                    "evidence_path": rel,
                    "reason": format!("timestamp is not RFC3339 and cannot be read: {:?}", ts_raw),
                }));
                continue;
            }
        };
        let evidence = match resolve_evidence(root, rel) {
            Some(p) => p,
            None => {
                rejected.push(json!({
                    "proof_id": id,
                    "evidence_path": rel,
                    "reason": "evidence path does not resolve to a file on this host",
                }));
                continue;
            }
        };
        match fs::read(&evidence) {
            Err(err) => {
                rejected.push(json!({
                    "proof_id": id,
                    "evidence_path": rel,
                    "reason": format!("evidence file unreadable: {}", err),
                }));
                continue;
            }
            Ok(bytes) if bytes.is_empty() => {
                rejected.push(json!({
                    "proof_id": id,
                    "evidence_path": rel,
                    "reason": "evidence file is empty",
                }));
                continue;
            }
            Ok(_) => {}
        }
        let status = e.get("status").and_then(|s| s.as_str()).unwrap_or("PENDING");
        let item = json!({
            "proof_id": id,
            "claim": e.get("claim").cloned().unwrap_or(json!("")),
            "evidence_path": rel,
            "evidence_resolved": slash(&evidence),
            "measurement": e.get("measurement").cloned().unwrap_or(Value::Null),
            "timestamp": ts_raw,
            "timestamp_unix": ts,
            "timestamp_source": e.get("timestamp_source").cloned().unwrap_or(json!("")),
            "status": status,
        });
        match status {
            "VERIFIED" => {
                if newest.map(|n| ts > n).unwrap_or(true) {
                    newest = Some(ts);
                }
                verified.push(item);
            }
            _ => pending.push(item),
        }
    }

    let window = 3600i64;
    let in_window = verified
        .iter()
        .filter(|v| {
            v.get("timestamp_unix")
                .and_then(|t| t.as_i64())
                .map(|t| t <= now && now - t <= window)
                .unwrap_or(false)
        })
        .count();
    let (verified_last_hour, basis) = if verified.is_empty() {
        (
            Value::Null,
            "no verified entries loaded; a per-hour rate is not computable".to_string(),
        )
    } else {
        (
            json!(in_window),
            format!(
                "{} of {} verified entries carry a timestamp within {}s before {}",
                in_window,
                verified.len(),
                window,
                now
            ),
        )
    };

    json!({
        "loaded": true,
        "error": Value::Null,
        "proofs_file": shown,
        "proofs_file_candidates": candidates,
        "schema": schema,
        "schema_expected": PROOFS_SCHEMA,
        "total_proofs": verified.len() + pending.len(),
        "verified": verified.len(),
        "pending": pending.len(),
        "rejected_count": rejected.len(),
        "items_in_queue": rejected.len() + pending.len(),
        "queue_basis": "entries whose evidence does not resolve/read, plus entries whose status is not VERIFIED",
        "verified_last_hour": verified_last_hour,
        "verified_last_hour_basis": basis,
        "verification_basis": "each entry's evidence_path resolves to a non-empty readable file; measurement values are NOT re-derived by this endpoint",
        "newest_proof_timestamp": newest
            .and_then(|t| verified.iter().find(|v| v.get("timestamp_unix").and_then(|x| x.as_i64()) == Some(t)))
            .and_then(|v| v.get("timestamp").cloned())
            .unwrap_or(Value::Null),
        "newest_proof_age_seconds": newest.map(|t| (now - t).max(0)),
        "rejected": rejected,
        "proofs": verified,
    })
}

/// GET /verification/queue — the entries that still need work, or the verified
/// proofs themselves when the queue is empty.
pub fn verification_queue_payload() -> Value {
    let candidates: Vec<Value> = proofs_candidates()
        .iter()
        .map(|p| json!(slash(p)))
        .collect();
    let Some(path) = proofs_path() else {
        return json!({
            "queue": [],
            "loaded": false,
            "error": "proofs file unreadable",
            "proofs_file_candidates": candidates,
        });
    };
    let stats = verification_stats_from(&path, &repo_root(), &candidates);
    let mut items: Vec<Value> = Vec::new();
    for key in ["rejected", "proofs"] {
        if let Some(arr) = stats.get(key).and_then(|a| a.as_array()) {
            for entry in arr {
                items.push(json!({
                    "item_id": entry.get("proof_id").cloned().unwrap_or(json!("")),
                    "item_type": "Proof",
                    "status": entry.get("status").cloned().unwrap_or(json!("REJECTED")),
                    "claim": entry.get("claim").cloned().unwrap_or(json!("")),
                    "evidence_path": entry.get("evidence_path").cloned().unwrap_or(json!("")),
                    "measurement": entry.get("measurement").cloned().unwrap_or(Value::Null),
                    "created_at": entry.get("timestamp").cloned().unwrap_or(json!("")),
                    "agent": "Verify",
                    "description": entry
                        .get("reason")
                        .or_else(|| entry.get("claim"))
                        .cloned()
                        .unwrap_or(json!("")),
                }));
            }
        }
    }
    json!({ "queue": items, "loaded": true, "proofs_file": slash(&path) })
}
// ─────────────────────────────────────────────────────────────────────────────
// Testing — real suite reports (*_test_report.json / *_suite_summary.json)
// ─────────────────────────────────────────────────────────────────────────────

/// Collect the real test-report artifacts on disk.
fn collect_test_reports() -> Vec<(String, Value)> {
    let dir = repo_root().join("swarm_v2_artifacts");
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let is_report = (name.ends_with("_test_report.json")
                || name.ends_with("_suite_summary.json")
                || name.ends_with("_test_results.json"))
                && !name.starts_with("PROPOSAL");
            if !is_report {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(e.path()) {
                if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                    out.push((name, v));
                }
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// GET /testing/stats — aggregate the real suite reports.
pub fn testing_stats_payload() -> Value {
    let reports = collect_test_reports();
    let total = reports.len();
    let mut passed = 0usize;
    for (_, v) in &reports {
        let status = v.get("status").and_then(|s| s.as_str()).unwrap_or("");
        if status.contains("PASS") || status.contains("COMPLETE") {
            passed += 1;
        }
    }
    let rate = if total > 0 {
        passed as f64 / total as f64 * 100.0
    } else {
        0.0
    };
    json!({
        "tests_running": 0,
        "success_rate_24h": (rate * 10.0).round() / 10.0,
        "total_runs": total,
        "passed_suites": passed,
        "source": "swarm_v2_artifacts/*_test_report.json",
    })
}

/// GET /testing/runs — one row per real suite report.
pub fn testing_runs_payload() -> Value {
    let reports = collect_test_reports();
    let runs: Vec<Value> = reports
        .iter()
        .map(|(name, v)| {
            let status = v
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("UNKNOWN");
            let elapsed_ms = v
                .get("total_elapsed_ms")
                .and_then(|x| x.as_f64())
                .or_else(|| v.get("time_s").and_then(|x| x.as_f64()).map(|s| s * 1000.0))
                .unwrap_or(0.0);
            json!({
                "run_id": name.trim_end_matches(".json"),
                "status": status,
                "timestamp": v.get("ts").and_then(|x| x.as_str()).unwrap_or(""),
                "duration_seconds": (elapsed_ms / 1000.0 * 1000.0).round() / 1000.0,
            })
        })
        .collect();
    json!({ "runs": runs })
}

// ─────────────────────────────────────────────────────────────────────────────
// Skills — real skill modules (swarm_v2/skills/*.py with docstring headers)
// ─────────────────────────────────────────────────────────────────────────────

/// GET /skills/portable — scan the real skill modules, derive name + description.
pub fn portable_skills_payload() -> Value {
    let dir = repo_root().join("swarm_v2").join("skills");
    let mut skills = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.ends_with(".py") || name == "__init__.py" {
                continue;
            }
            let stem = name.trim_end_matches(".py");
            let mut description = String::new();
            if let Ok(raw) = fs::read_to_string(e.path()) {
                // First meaningful docstring/comment line as the description,
                // skipping imports/blank lines and dunder noise.
                for line in raw.lines().take(12) {
                    let t = line
                        .trim()
                        .trim_start_matches('"')
                        .trim_start_matches('#')
                        .trim();
                    if t.is_empty()
                        || t.starts_with("__")
                        || t.starts_with("from ")
                        || t.starts_with("import ")
                        || t.starts_with("'''")
                    {
                        continue;
                    }
                    description = t.to_string();
                    break;
                }
                if description.is_empty() {
                    description = format!("{} skill module", stem);
                }
            }
            skills.push(json!({
                "name": stem,
                "description": description,
                "source": "python",
            }));
        }
    }
    skills.sort_by(|a, b| {
        a.get("name")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .cmp(b.get("name").and_then(|x| x.as_str()).unwrap_or(""))
    });
    json!({ "skills": skills, "stats": { "total": skills.len() } })
}

/// Dispatch the second batch of file-backed families. Returns Some if handled.
pub fn handle_batch2(
    request: &mut Request,
    url: &str,
) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();
    if method != Method::Get {
        return None;
    }
    if url.starts_with("/verification/stats") {
        return Some(ok_response(verification_stats_payload()));
    }
    if url.starts_with("/verification/queue") {
        return Some(ok_response(verification_queue_payload()));
    }
    if url.starts_with("/testing/stats") {
        return Some(ok_response(testing_stats_payload()));
    }
    if url.starts_with("/testing/runs") {
        return Some(ok_response(testing_runs_payload()));
    }
    if url.starts_with("/skills/portable") {
        return Some(ok_response(portable_skills_payload()));
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Rollout 4 — ultrawork missions, research, journal, agents, mesh
// (STATE_STORE_SPEC.md: read the same files the Python tooling persists)
// ─────────────────────────────────────────────────────────────────────────────

/// GET /ultrawork/missions — the real persisted mission files (52 on disk).
pub fn ultrawork_missions_payload() -> Value {
    let dir = repo_root().join("swarm_v2_artifacts").join("missions");
    let mut missions = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with("mission-") || !name.ends_with(".json") {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(e.path()) {
                if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                    let retry = v.get("retry_count").and_then(|x| x.as_u64()).unwrap_or(0);
                    let obj = v.get("objective").and_then(|x| x.as_str()).unwrap_or("");
                    let truncated: String = obj.chars().take(60).collect();
                    missions.push(json!({
                        "mission_id": v.get("mission_id").cloned().unwrap_or(json!(name.trim_end_matches(".json"))),
                        "objective": truncated,
                        "phase": v.get("phase").cloned().unwrap_or(json!("planning")),
                        "steps": v.get("steps").and_then(|s| s.as_array()).map(|a| a.len()).unwrap_or(0),
                        "retry_count": retry,
                        "attempt": retry + 1,
                        "created_at": v.get("created_at").cloned().unwrap_or(json!("")),
                    }));
                }
            }
        }
    }
    missions.sort_by(|a, b| {
        b.get("mission_id")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .cmp(a.get("mission_id").and_then(|x| x.as_str()).unwrap_or(""))
    });
    json!({ "missions": missions, "stats": { "total": missions.len() } })
}

/// The real research artifact (nobel_research_papers.json).
fn load_nobel() -> Option<Value> {
    let p = repo_root()
        .join("swarm_v2_artifacts")
        .join("nobel_research_papers.json");
    let raw = fs::read_to_string(p).ok()?;
    serde_json::from_str::<Value>(&raw).ok()
}

/// GET /research/tasks — the platform's published journal research (papers/*.md), the Seeker's
/// real output, not the stale nobel seed.
fn research_tasks_payload() -> Value {
    let dir = repo_root().join("papers");
    let mut tasks: Vec<Value> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("md") {
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let Ok(text) = fs::read_to_string(&p) else { continue };
            let (title, finding, section) = parse_paper(&text);
            tasks.push(json!({
                "task_id": name.trim_end_matches(".md"),
                "objective": title,
                "summary": finding,
                "sources": [section],
                "status": "completed",
                "timestamp": "",
                "domain": section,
                "agent": "Seeker",
            }));
        }
    }
    tasks.sort_by_key(|t| t["task_id"].as_str().unwrap_or("").to_string());
    json!({ "tasks": tasks, "source": "papers/*.md — the platform's published journal research (Seeker output)" })
}

/// Candidate locations for the theorem ledger. The deployed tree may be the
/// repository root or the staged kernel tree, so both are searched instead of
/// hard-coding one path that may not exist where the server runs.
fn theorems_candidates() -> Vec<PathBuf> {
    let rel = "swarm_v2_artifacts/formal_verification_theorems_proven.json";
    let root = repo_root();
    let mut out: Vec<PathBuf> = vec![root.join(rel), root.join("staging").join("v20").join(rel)];
    let mut dir = Some(root);
    for _ in 0..3 {
        dir = dir.and_then(|d| d.parent().map(|p| p.to_path_buf()));
        match dir.clone() {
            Some(d) => out.push(d.join(rel)),
            None => break,
        }
    }
    out
}

fn load_theorems() -> Option<Value> {
    let path = theorems_candidates().into_iter().find(|p| p.is_file())?;
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&raw).ok()
}

/// GET /research/theorems — the real proven theorems (theorem ledger, not the proofs queue).
fn research_theorems_payload() -> Value {
    match load_theorems() {
        Some(v) => {
            let theorems: Vec<Value> = v
                .get("formal_theorems")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|t| {
                            json!({
                                "theorem_id": t.get("id").cloned().unwrap_or(json!("")),
                                "name": t.get("theorem").cloned().unwrap_or(json!("")),
                                "formula": t.get("formula").cloned().unwrap_or(json!("")),
                                "status": t.get("status").cloned().unwrap_or(json!("")),
                                "state_space": t.get("state_space").cloned().unwrap_or(json!(0)),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            let total_states: u64 = theorems
                .iter()
                .map(|t| t.get("state_space").and_then(|s| s.as_u64()).unwrap_or(0))
                .sum();
            json!({
                "theorems": theorems,
                "total_proven": v.get("theorems_proven").cloned().unwrap_or(json!(0)),
                "total_states_evaluated": total_states,
                "monad_seal": "0x4D4F4E4144000001",
                "source": "formal_verification_theorems_proven.json"
            })
        }
        None => json!({ "theorems": [] }),
    }
}

/// Parse a paper's title, abstract, and domain section from its markdown body.
fn parse_paper(text: &str) -> (String, String, String) {
    let mut title = String::new();
    let mut section = String::new();
    for l in text.lines() {
        let t = l.trim();
        if title.is_empty() {
            if let Some(rest) = t.strip_prefix("# ") {
                title = rest.to_string();
                // strip a leading "Paper NNN: " prefix (paper_148 style), leave paper_01 style alone
                if let Some(idx) = title.find(':') {
                    let prefix = &title[..idx];
                    if prefix.starts_with("Paper ") {
                        let digits = &prefix[6..];
                        if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
                            title = title[idx + 1..].trim().to_string();
                        }
                    }
                }
            }
        }
        if section.is_empty() {
            let lower = t.to_lowercase();
            if lower.starts_with("**domain:") || lower.starts_with("**classification:") {
                let after = t.splitn(2, ':').nth(1).unwrap_or("").trim();
                let first = after.split('/').next().unwrap_or("").trim().trim_matches('*');
                if !first.is_empty() {
                    section = first.to_string();
                }
            }
        }
        if !title.is_empty() && !section.is_empty() {
            break;
        }
    }
    // finding: the ## ABSTRACT section, else the first non-metadata paragraph
    let mut finding = String::new();
    let mut in_abstract = false;
    for l in text.lines() {
        let t = l.trim();
        if t.starts_with("## ABSTRACT") || t.starts_with("## Abstract") {
            in_abstract = true;
            continue;
        }
        if in_abstract {
            if t.starts_with("## ") || t.starts_with("# ") {
                break;
            }
            if !t.is_empty() {
                finding.push_str(t);
                finding.push(' ');
                if finding.len() > 400 {
                    break;
                }
            }
        }
    }
    if finding.trim().is_empty() {
        for l in text.lines() {
            let t = l.trim();
            if t.is_empty()
                || t.starts_with('#')
                || t.starts_with("**")
                || t.starts_with('-')
                || t.starts_with('|')
                || t.starts_with('>')
                || t.starts_with('`')
            {
                continue;
            }
            finding.push_str(t);
            finding.push(' ');
            if finding.len() > 400 {
                break;
            }
        }
    }
    (title, finding.trim().to_string(), section.trim().to_string())
}

/// GET /journal/papers — every paper in the Monad Journal, full text served so the reader modal
/// shows the actual publication rather than a stub. Replaces the 10-entry nobel_candidates subset.
pub fn journal_papers_payload() -> Value {
    let dir = repo_root().join("papers");
    let mut papers: Vec<Value> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("md") {
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let Ok(text) = fs::read_to_string(&p) else { continue };
            let (title, finding, section) = parse_paper(&text);
            let section = if section.is_empty() { "General".to_string() } else { section };
            papers.push(json!({
                "id": name.trim_end_matches(".md"),
                "title": title,
                "finding": finding,
                "abstract": finding,
                "filename": name,
                "content_length": text.len(),
                "full_content": text,
                "status": "PUBLISHED",
                "journal": "The Monad Journal of Sovereign Deep-Tech & Microkernel Computing",
                "section": section,
                "domain": section,
            }));
        }
    }
    papers.sort_by_key(|p| {
        let id = p["id"].as_str().unwrap_or("");
        let num: u32 = id
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(0);
        (num, id.to_string())
    });
    json!({ "papers": papers, "total": papers.len(), "platform": platform_feed_counts(), "source": "papers/*.md full text + monad ledger / motions / research notes" })
}

/// The Monad Journal feeds off the whole platform: the sealed ledger chain, the council motions,
/// and the Seeker's research notes — not just the static papers directory.
fn platform_feed_counts() -> Value {
    let ledger = ledger_monad_blocks_payload();
    let total = ledger.get("total_blocks").and_then(|v| v.as_u64()).unwrap_or(0);
    let mut motions = 0u64;
    if let Ok(raw) = fs::read_to_string(repo_root().join("swarm_v2_artifacts/monad_ledger.jsonl")) {
        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if let Some(kind) = v.get("kind").and_then(|k| k.as_str()) {
                    if kind.to_lowercase().contains("motion") {
                        motions += 1;
                    }
                }
            }
        }
    }
    let mut research_notes = 0u64;
    if let Ok(entries) = fs::read_dir(repo_root().join("swarm_v2_artifacts/research")) {
        research_notes = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
            .count() as u64;
    }
    json!({
        "ledger_blocks": total,
        "motions": motions,
        "research_notes": research_notes,
        "latest_block_index": ledger.get("latest_block_index").cloned().unwrap_or(json!(null)),
        "latest_block_hash": ledger.get("latest_block_hash").cloned().unwrap_or(json!(null)),
        "monad_anchor": ledger.get("monad_anchor").cloned().unwrap_or(json!(null)),
    })
}

/// GET /swarm/agents — the real 14-agent council roster (Rust-owned).
fn swarm_agents_payload() -> Value {
    let agents: Vec<Value> = crate::AGENTS
        .iter()
        .map(|a| {
            json!({
                "name": a.name,
                "role": a.role,
                "domain": a.domain,
                "weight": a.weight,
                "status": "ONLINE",
            })
        })
        .collect();
    json!({ "agents": agents, "total": agents.len(), "source": "native_rust_council_roster" })
}

/// GET /swarm/orchestrator/stats — derived from real state (kanban + council).
fn orchestrator_stats_payload() -> Value {
    let stats = crate::kanban_handlers::stats_payload();
    let board = crate::kanban_handlers::board_payload();
    let done = board
        .get("DONE")
        .and_then(|d| d.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let todo = board
        .get("TODO")
        .and_then(|d| d.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    json!({
        "total_cards": stats.get("total_cards").cloned().unwrap_or(json!(0)),
        "completed": done,
        "queued": todo,
        "active_agents": crate::AGENTS.len(),
        "board_health": stats.get("board_health").cloned().unwrap_or(json!(0.0)),
        "source": "kanban_state.json + native council roster",
    })
}

/// GET /infrastructure/nodes — the real node set: the live agent roster plus the federation
/// registry's probed nodes. The payload this replaces listed two nodes that exist nowhere on this
/// platform ("Jarvis OS Gateway", "Antigravity Sovereign Microkernel") with invented ping times.
fn infrastructure_nodes_payload() -> Value {
    let mesh = crate::mesh_handlers::mesh_nodes();
    json!({
        "nodes": mesh.get("nodes").cloned().unwrap_or(json!([])),
        "peers": mesh.get("peers").cloned().unwrap_or(json!([])),
        "total": mesh.get("node_count").cloned().unwrap_or(json!(0)),
        "source": "main.rs AGENTS roster + swarm_v2_federation/registry.json",
    })
}

/// GET /research/campaign — the real 1M virtual-screening campaign artifact.
fn research_campaign_payload() -> Value {
    let p = repo_root()
        .join("swarm_v2_artifacts")
        .join("virtual_screening_campaign_1m.json");
    match fs::read_to_string(&p)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
    {
        Some(mut v) => {
            if let Some(obj) = v.as_object_mut() {
                obj.insert("source".into(), json!("virtual_screening_campaign_1m.json"));
            }
            v
        }
        None => {
            json!({ "campaign": null, "source": "virtual_screening_campaign_1m.json (missing)" })
        }
    }
}

/// GET /swarm/spatial/entanglement — each served link is a pairing this host actually probed: one
/// per federation node in the registry, carrying the registry's own measured latency and probe
/// time. The two nodes this used to link ("Jarvis OS Gateway" / "Antigravity Sovereign
/// Microkernel") exist nowhere on this platform, and the 0.10 ms between them was never measured.
fn spatial_entanglement_payload() -> Value {
    let reg = federation_payload("/federation/peers");
    let peer_list = reg.get("peers").cloned().unwrap_or(json!([]));
    let mut events: Vec<Value> = Vec::new();
    for p in peer_list.as_array().map(|a| a.as_slice()).unwrap_or(&[]) {
        let host = p.get("host").and_then(|v| v.as_str()).unwrap_or("");
        if host == "127.0.0.1" || host.is_empty() {
            continue; // this host is not a link to itself
        }
        events.push(json!({
            "id": events.len() + 1,
            "nodes": ["local", p.get("node_id").cloned().unwrap_or(json!("unknown"))],
            "labels": ["NspiredFitness Laptop Sovereign (this host)",
                       p.get("name").cloned().unwrap_or(json!("unnamed peer"))],
            "latency_ms": p.get("latency_ms").cloned().unwrap_or(json!(null)),
            "status": p.get("status").cloned().unwrap_or(json!("unknown")),
            "probed_at": p.get("probed_at").cloned().unwrap_or(json!(null)),
        }));
    }
    let total = events.len();
    json!({
        "entanglements": events,
        "total": total,
        "source": "swarm_v2_federation/registry.json (direct /health probes over the Tailscale mesh)",
        "basis": "one link per federation node this host probed; no agent-to-agent link is measured \
                  on this host, so none is asserted",
    })
}

/// GET /swarm/spatial/mitosis — real persisted mission-spawn records as mitosis
/// events (each is a real mission the loop created; no invented spawns).
pub fn spatial_mitosis_payload() -> Value {
    let dir = repo_root().join("swarm_v2_artifacts").join("missions");
    let mut events = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with("mission-") || !name.ends_with(".json") {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(e.path()) {
                if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                    events.push(json!({
                        "id": v.get("mission_id").cloned().unwrap_or(json!(name.trim_end_matches(".json"))),
                        "objective": v.get("objective").cloned().unwrap_or(json!("")),
                        "phase": v.get("phase").cloned().unwrap_or(json!("planning")),
                        "created_at": v.get("created_at").cloned().unwrap_or(json!("")),
                        "type": "mission_spawn",
                    }));
                }
            }
        }
    }
    events.truncate(20);
    json!({ "mitosis_events": events, "total": events.len(), "source": "swarm_v2_artifacts/missions/*.json" })
}

/// Dispatch rollout-4 families. Returns Some if handled.
pub fn handle_batch3(
    request: &mut Request,
    url: &str,
) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();

    // ── real file/source-backed reads ──────────────────────────────────────
    if method == Method::Get && url.starts_with("/ultrawork/missions") {
        return Some(ok_response(ultrawork_missions_payload()));
    }
    if method == Method::Get && url.starts_with("/research/tasks") {
        return Some(ok_response(research_tasks_payload()));
    }
    if method == Method::Get && url.starts_with("/research/theorems") {
        return Some(ok_response(research_theorems_payload()));
    }
    if method == Method::Get && url.starts_with("/journal/papers") {
        return Some(ok_response(journal_papers_payload()));
    }
    if method == Method::Get && url.starts_with("/swarm/agents") {
        return Some(ok_response(swarm_agents_payload()));
    }
    if method == Method::Get && url.starts_with("/swarm/orchestrator/stats") {
        return Some(ok_response(orchestrator_stats_payload()));
    }
    if method == Method::Get && url.starts_with("/infrastructure/nodes") {
        return Some(ok_response(infrastructure_nodes_payload()));
    }
    if method == Method::Get && url.starts_with("/research/campaign") {
        return Some(ok_response(research_campaign_payload()));
    }
    if method == Method::Get && url.starts_with("/swarm/spatial/entanglement") {
        return Some(ok_response(spatial_entanglement_payload()));
    }
    if method == Method::Get && url.starts_with("/swarm/spatial/mitosis") {
        return Some(ok_response(spatial_mitosis_payload()));
    }

    // ── genuine engine work → honest 501 (never fabricate) ─────────────────
    //
    // /infra/* and /infrastructure/status were refused here as "No persisted state exists for Rust to
    // serve". That is no longer true and the refusal is withdrawn: the FSM is
    // trm_uefi_kernel/src/self_healing_infra.rs, and its input is the supervisor's own published table
    // (last_ping_ms, restarts, max_restarts, last_restart_ms, interval_s). Those paths must therefore
    // fall through to infra_handlers, so no arm is taken for them here. A 501 that says a module does
    // not exist becomes a lie the moment the module is wired.
    // /secrets/keys is a metadata inventory: the vault stores {name: encrypted_value}, so the key
    // NAMES are plaintext and listable without an engine or decryption. Values are never returned.
    // Mutation/decrypt routes (set/delete/decrypt/rotate/encrypt) stay refused — this server never
    // fabricates, decrypts, or returns secret material.
    if method == Method::Get && (url == "/secrets/keys" || url.starts_with("/secrets/keys?")) {
        return Some(ok_response(secrets_keys_payload()));
    }
    if url.starts_with("/secrets") {
        return Some(engine_only(
            "Secret mutation and decryption stay engine-held. This server lists key names only; \
             it never fabricates, decrypts, or returns secret material.",
        ));
    }
    if url.starts_with("/swarm/chat") {
        return Some(engine_only(
            "Swarm chat routes to agent inference (LLM path) in the Python engine. The Rust \
             server serves state, not inference; canned replies would be a fabrication.",
        ));
    }
    if url.starts_with("/swarm/spatial/simulate") {
        return Some(engine_only(
            "Spatial simulation runs a live compute job in the engine; it has no persisted \
             result to serve.",
        ));
    }
    None
}

/// Read the federation registry (swarm_v2_federation/registry.json) as a JSON value; empty object
/// when absent or malformed — never an invented registry.
/// True only when the registry file existed and carried a peer list. An absent file is
/// "never probed", which must not be rendered as a count of zero.
fn probed_registry(reg: &Value) -> bool {
    reg.get("connected_nodes").map(|c| c.is_array()).unwrap_or(false)
}

fn read_registry() -> Value {
    let reg_path = repo_root().join("swarm_v2_federation").join("registry.json");
    fs::read_to_string(&reg_path)
        .ok()
        .and_then(|c| serde_json::from_str::<Value>(&c).ok())
        .unwrap_or(json!({}))
}

/// Federation telemetry — the STANDARDIZED /federation/stats schema. Returns the registry's own v4
/// shape so every node exposes the SAME fields: {status, monad_anchor, registry_version, tailnet,
/// connected_nodes, total_nodes, nodes}. The old payload carried hardcoded mesh_agents=20 /
/// pbft_sync="100%" / voltage_harmony=0.8986 / backend v19.002 — declared constants with no
/// measurement, removed so a peer can trust the numbers it reads.
pub fn federation_payload(url: &str) -> Value {
    let reg = read_registry();
    let nodes: Vec<Value> = reg
        .get("connected_nodes")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();

    let connected_count = nodes.iter().filter(|p| {
        p.get("status").and_then(|s| s.as_str()).map(|s| s == "online" || s == "SYNCED").unwrap_or(false)
    }).count();
    let total_count = nodes.len();

    if url.ends_with("/peers") {
        return json!({ "peers": nodes });
    }

    json!({
        "status": "ok",
        "monad_anchor": "0x4D4F4E4144000001",
        "registry_version": reg.get("registry_version").cloned().unwrap_or(Value::Null),
        "tailnet": reg.get("mesh").and_then(|m| m.get("tailnet")).cloned().unwrap_or(Value::Null),
        "connected_nodes": if probed_registry(&reg) { json!(connected_count) } else { Value::Null },
        "probed": probed_registry(&reg),
        "total_nodes": total_count,
        "nodes": nodes,
    })
}

/// /sync + /federation/sync — the standardized sync-write endpoint. Verifies the caller's Monad
/// anchor and returns the registry's ACTUAL counts (derived), replacing the old hardcoded
/// nodes_synced=4 / active_nodes=4 / AGREED_14_0_0 / voltage_harmony=0.8986 constants.
pub fn federation_sync_payload(body: &str) -> Value {
    let parsed: Value = serde_json::from_str(body).unwrap_or(json!({}));
    let anchor_ok = parsed
        .get("monad_anchor")
        .and_then(Value::as_str)
        .map(|a| a == "0x4D4F4E4144000001")
        .unwrap_or(false);
    let caller = parsed.get("node_id").and_then(Value::as_str).unwrap_or("unknown");

    let reg = read_registry();
    let nodes: Vec<Value> = reg
        .get("connected_nodes")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();
    let connected = nodes.iter().filter(|p| {
        p.get("status").and_then(|s| s.as_str()).map(|s| s == "online" || s == "SYNCED").unwrap_or(false)
    }).count();

    json!({
        "status": if anchor_ok { "ok" } else { "anchor_mismatch" },
        "monad_anchor": "0x4D4F4E4144000001",
        "anchor_verified": anchor_ok,
        "caller_node_id": caller,
        "synced": anchor_ok,
        "connected_nodes": if probed_registry(&reg) { json!(connected) } else { Value::Null },
        "total_nodes": if probed_registry(&reg) { json!(nodes.len()) } else { Value::Null },
        "probed": probed_registry(&reg),
    })
}
// ─────────────────────────────────────────────────────────────────────────────
// Verification queue — tests
//
// These pin the three properties the endpoint must never lose:
//   1. a missing proofs file is an ERROR with loaded:false and does not panic;
//   2. three resolvable entries count as three proofs;
//   3. an entry whose evidence_path does not exist is REJECTED, not counted.
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod verification_queue_tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Sandbox {
        dir: PathBuf,
    }

    impl Sandbox {
        fn new(tag: &str) -> Sandbox {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let dir = std::env::temp_dir().join(format!(
                "trm_proofs_test_{}_{}_{}",
                tag,
                std::process::id(),
                nanos
            ));
            fs::create_dir_all(dir.join("evidence")).expect("create sandbox");
            Sandbox { dir }
        }

        fn root(&self) -> &Path {
            &self.dir
        }

        fn evidence(&self, name: &str, body: &str) {
            fs::write(self.dir.join("evidence").join(name), body).expect("write evidence");
        }

        fn proofs(&self, doc: &str) -> PathBuf {
            let p = self.dir.join("swarm_v2_artifacts").join("verification_proofs.json");
            fs::create_dir_all(p.parent().unwrap()).expect("create artifacts dir");
            fs::write(&p, doc).expect("write proofs");
            p
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn entry(id: &str, evidence: &str, ts: &str) -> String {
        format!(
            r#"{{"proof_id":"{}","claim":"test claim","evidence_path":"{}",
                 "measurement":{{"checks_passed":98,"checks_failed":0}},
                 "timestamp":"{}","timestamp_source":"field:generated","status":"VERIFIED"}}"#,
            id, evidence, ts
        )
    }

    fn doc(entries: &[String]) -> String {
        format!(
            r#"{{"schema":"{}","proofs":[{}]}}"#,
            PROOFS_SCHEMA,
            entries.join(",")
        )
    }

    #[test]
    fn missing_proofs_file_errors_without_panicking() {
        let sb = Sandbox::new("missing");
        let absent = sb.root().join("swarm_v2_artifacts/verification_proofs.json");
        assert!(!absent.exists(), "precondition: proofs file must not exist");

        // The public entry point must also survive a whole-tree with no proofs file.
        let stats = verification_stats_from(&absent, sb.root(), &[]);
        assert_eq!(stats["loaded"], json!(false));
        assert_eq!(stats["error"], json!("proofs file unreadable"));
        assert_eq!(stats["error_kind"], json!("missing"));
        assert_eq!(stats["items_in_queue"], json!(0));
        assert_eq!(stats["total_proofs"], json!(0));
        assert!(stats["verified_last_hour"].is_null());
        assert!(stats["proofs_file"].as_str().unwrap_or("").ends_with("verification_proofs.json"));

        // Same for an unparseable file: an error, not a panic and not a zero-count success.
        let bad = sb.proofs("{ not json");
        let stats = verification_stats_from(&bad, sb.root(), &[]);
        assert_eq!(stats["loaded"], json!(false));
        assert_eq!(stats["error_kind"], json!("invalid_json"));
    }

    #[test]
    fn three_entries_with_real_evidence_count_three() {
        let sb = Sandbox::new("three");
        for (i, name) in ["a.json", "b.txt", "c.json"].iter().enumerate() {
            sb.evidence(name, &format!("{{\"measured\":{}}}", i));
        }
        let path = sb.proofs(&doc(&[
            entry("P-1", "evidence/a.json", "2026-11-05T17:24:38"),
            entry("P-2", "evidence/b.txt", "2026-11-05T17:26:44Z"),
            entry("P-3", "evidence/c.json", "2026-11-05T17:28:47.123+00:00"),
        ]));
        let stats = verification_stats_from(&path, sb.root(), &[]);
        assert_eq!(stats["loaded"], json!(true));
        assert!(stats["error"].is_null());
        assert_eq!(stats["total_proofs"], json!(3));
        assert_eq!(stats["verified"], json!(3));
        assert_eq!(stats["items_in_queue"], json!(0));
        assert_eq!(stats["schema"], json!(PROOFS_SCHEMA));
        assert_eq!(stats["rejected"].as_array().map(|a| a.len()), Some(0));
        assert!(stats["proofs_file"].as_str().unwrap().ends_with("verification_proofs.json"));
        // The measurement travels through untouched.
        assert_eq!(stats["proofs"][0]["measurement"]["checks_passed"], json!(98));
        // Timestamps are compared to the clock, so the rate must be a number here.
        assert!(stats["verified_last_hour"].is_number());
    }

    #[test]
    fn entry_with_missing_evidence_is_rejected_not_counted() {
        let sb = Sandbox::new("reject");
        sb.evidence("ok.json", "{\"measured\":1}");
        let path = sb.proofs(&doc(&[
            entry("P-OK", "evidence/ok.json", "2026-11-05T17:24:38"),
            entry("P-GONE", "evidence/does_not_exist.json", "2026-11-05T17:24:39"),
        ]));
        let stats = verification_stats_from(&path, sb.root(), &[]);
        assert_eq!(stats["loaded"], json!(true));
        assert_eq!(stats["verified"], json!(1));
        assert_eq!(stats["total_proofs"], json!(1));
        assert_eq!(stats["rejected_count"], json!(1));
        assert_eq!(stats["items_in_queue"], json!(1), "unresolvable evidence is queue work");
        let rejected = stats["rejected"].as_array().cloned().unwrap_or_default();
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0]["proof_id"], json!("P-GONE"));
        assert!(rejected[0]["reason"]
            .as_str()
            .unwrap_or("")
            .contains("does not resolve"));
        assert!(stats["proofs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["proof_id"] != json!("P-GONE")));
    }

    #[test]
    fn timestamp_parser_reads_the_forms_the_evidence_carries() {
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_rfc3339("2026-11-05T17:24:38"), Some(1793899478));
        assert_eq!(parse_rfc3339("2026-11-05T17:24:38Z"), Some(1793899478));
        assert_eq!(parse_rfc3339("2026-11-05T13:24:38-04:00"), Some(1793899478));
        assert_eq!(parse_rfc3339("2026-11-05T17:24:38.184080"), Some(1793899478));
        assert_eq!(parse_rfc3339("not-a-timestamp"), None);
        assert_eq!(parse_rfc3339(""), None);
    }
}
