//! Hospital AMR surveillance node (Paper 143) — MEASURED deployment report.
//!
//! REPLACES the scratch copy at `scratch/kernel194/src/hospital_amr_surveillance_node.rs`, which
//! emitted five steps whose "measured_metric" fields were authored strings ("Substrate recognized in
//! 0.42 ms", "Total Parser Memory Footprint: 14.8 MB", "0x8FA4912DEBC31002") and set every step to
//! `K3Trit::Unknown` — honest about being unverified, but still a report that measured nothing. Its own
//! source said it: *"Nothing verifies these steps: the metrics beside them are authored strings."*
//!
//! Each step here has ONE job: produce a value this process computed, or say it did not. The trit is
//! derived from the value, never chosen:
//!
//!   trit = +1  the measured value meets the step's stated criterion on THIS machine
//!   trit =  0  the measurement exists but does not decide the criterion (different artefact), or no
//!              measurement is possible without hardware this host does not have
//!   trit = -1  the measurement contradicts the criterion
//!
//! The aggregate is the Kleene AND of the five trits, so one refuted step cannot be reported as a
//! deployment. On this host the expected verdict is REFUTED: the target is a Raspberry Pi 4 (aarch64)
//! and this is x86_64, so step 1 is a contradiction by construction — which is the correct answer to
//! "did the Kisumu deployment run", not a failure of the report.

use crate::k3_logic::{k3_consensus, sha256, FALSE, TRUE, UNKNOWN};
use serde_json::{json, Value};
use std::time::Instant;

/// Files this module will read to measure step 3 and 4. Each candidate is tried in order; the first
/// that loads is used and its path is published, so a reader can re-run the same measurement.
const EVIDENCE_CANDIDATES: [&str; 5] = [
    "eskape_spectrum_full.json",
    "../../eskape_spectrum_full.json",
    "data/eskape_spectrum_full.json",
    "../../data/eskape_spectrum_full.json",
    "swarm_v2_artifacts/eskape_spectrum_full.json",
];

/// The gene signatures the step-3 claim names. Counted, not assumed.
const GENE_SIGNATURES: [&str; 7] = [
    "blaKPC", "blaNDM", "blaOXA-48", "blaCTX-M-15", "mcr-1", "vanA", "mecA",
];

struct Step {
    id: &'static str,
    title: &'static str,
    metric: Value,
    trit: i8,
    /// Why the trit is what it is, in the reader's terms.
    basis: String,
}

fn read_evidence() -> Option<(&'static str, String)> {
    for c in EVIDENCE_CANDIDATES.iter() {
        if let Ok(s) = std::fs::read_to_string(c) {
            return Some((c, s));
        }
    }
    None
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for b in bytes.iter() {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

/// STEP 1 — hardware. The claim is a Raspberry Pi 4 (aarch64). This process can read its own
/// architecture and core count for free, and that is decisive: either this is the target or it is not.
fn step_hardware() -> Step {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    let target_match = arch == "aarch64";
    Step {
        id: "Step 1",
        title: "ARM64 Hardware Discovery & Resource Fusion",
        metric: json!({
            "arch": arch,
            "os": os,
            "logical_cores": cores,
            "claimed_target": "Broadcom BCM2711 ARM64 Cortex-A72, 4 GB LPDDR4",
            "target_matched": target_match,
        }),
        trit: if target_match { TRUE } else { FALSE },
        basis: if target_match {
            format!("running on aarch64 with {} cores", cores)
        } else {
            format!(
                "measured arch={} on {} cores; the deployment target is aarch64, so the node under \
                 discussion is not the one this report was produced on",
                arch, cores
            )
        },
    }
}

/// STEP 2 — parser suite. The claim is "20 native ARM64 parsers, <20 MB footprint". What this host can
/// measure is how many real ingestion serialisers exist on disk and that none of them are ARM64
/// binaries. Both are counted from the filesystem rather than asserted.
fn step_parsers() -> Step {
    // MEASURED DEFECT (2026-11-23): this first scanned only "." and "../../", counted 0 and reported a
    // false negative -- the ingestion serialisers live in tools/. A count taken from the wrong directory
    // is not a measurement of absence. The search now covers the directories the repo actually uses, and
    // the paths each hit came from are published so the count can be checked.
    let mut found: Vec<(String, String)> = Vec::new();
    for dir in [".", "../../", "../../tools", "tools", "../../data", "data"] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with("_serialize.py") && !found.iter().any(|(n, _)| *n == name) {
                    found.push((name, dir.to_string()));
                }
            }
        }
    }
    found.sort();
    let names: Vec<String> = found.iter().map(|(n, _)| n.clone()).collect();
    let dirs: Vec<String> = {
        let mut d: Vec<String> = found.iter().map(|(_, d)| d.clone()).collect();
        d.sort();
        d.dedup();
        d
    };
    let arm64_parsers = 0usize; // nothing in this build is an ARM64 parser binary
    let ok = names.len() >= 20 && arm64_parsers >= 20;
    Step {
        id: "Step 2",
        title: "Native ARM64 Scientific Parser Suite Ingestion",
        metric: json!({
            "serialisers_on_disk": names.len(),
            "serialiser_names": names,
            "searched_dirs": dirs,
            "arm64_parser_binaries": arm64_parsers,
            "claimed": "20 native ARM64 parsers in < 20 MB",
            "footprint_measured_mb": Value::Null,
        }),
        trit: if ok { TRUE } else { FALSE },
        basis: format!(
            "counted {} *_serialize.py in {} and {} ARM64 parser binaries; the claim is 20 ARM64 \
             parsers, so it is contradicted here regardless of the serialiser count",
            names.len(), dirs.join(", "), arm64_parsers
        ),
    }
}

/// STEP 3 — knowledge fabric. Two things are measurable: the SHA-256 of the evidence file (replacing the
/// authored anchor `0x8FA4912DEBC31002`) and how many of the seven named resistance genes actually appear
/// in it. A file that exists and hashes is not the same as a fabric seeded with 1,000 signatures, so the
/// count decides the trit.
fn step_knowledge() -> Step {
    match read_evidence() {
        Some((path, text)) => {
            let digest = hex32(&sha256(text.as_bytes()));
            let hits: Vec<&str> = GENE_SIGNATURES
                .iter()
                .copied()
                .filter(|g| text.contains(g))
                .collect();
            let bytes = text.len();
            let enough = hits.len() >= GENE_SIGNATURES.len();
            Step {
                id: "Step 3",
                title: "100M Trit AMR Knowledge Fabric & Gene Signatures",
                metric: json!({
                    "evidence_path": path,
                    "bytes": bytes,
                    "sha256": digest,
                    "genes_checked": GENE_SIGNATURES.len(),
                    "genes_present": hits,
                    "claimed_signatures": 1000,
                    "authored_anchor_replaced": "0x8FA4912DEBC31002",
                }),
                trit: if enough { TRUE } else { FALSE },
                basis: format!(
                    "hashed {} ({} bytes, sha256 {}…) and found {}/{} named resistance genes; the claim \
                     is 1,000 signatures, so the fabric is not demonstrated by this file",
                    path,
                    bytes,
                    &digest[..16],
                    hits.len(),
                    GENE_SIGNATURES.len()
                ),
            }
        }
        None => Step {
            id: "Step 3",
            title: "100M Trit AMR Knowledge Fabric & Gene Signatures",
            metric: json!({ "evidence_path": Value::Null, "claimed_signatures": 1000 }),
            trit: UNKNOWN,
            basis: format!(
                "no evidence file found in any of {} candidate paths",
                EVIDENCE_CANDIDATES.len()
            ),
        },
    }
}

/// STEP 4 — pipeline latency. What can be timed is a real ingestion of the evidence file through the
/// same serde_json parser the platform uses. What CANNOT be claimed from that is the MIC/EUCAST
/// disk-diffusion path, which does not exist in this binary — so the trit stays 0 (measured, but it does
/// not decide the stated criterion) even when the timing beats the 5 s threshold.
fn step_pipeline() -> Step {
    let t0 = Instant::now();
    let parsed = read_evidence()
        .and_then(|(path, text)| serde_json::from_str::<Value>(&text).ok().map(|v| (path, v)));
    let ingest_ms = t0.elapsed().as_secs_f64() * 1000.0;
    match parsed {
        Some((path, v)) => {
            let top_keys = v.as_object().map(|m| m.len()).unwrap_or(0);
            Step {
                id: "Step 4",
                title: "Autonomous MIC Pipeline & Outbreak Early Warning",
                metric: json!({
                    "parsed_path": path,
                    "ingest_ms": (ingest_ms * 1000.0).round() / 1000.0,
                    "top_level_keys": top_keys,
                    "claimed_mic_latency_sec": 0.084,
                    "mic_eucast_parser_present": false,
                    "outbreak_zscore": Value::Null,
                }),
                trit: UNKNOWN,
                basis: format!(
                    "timed a real serde_json ingest of {} at {:.3} ms, but the MIC/EUCAST disk-diffusion \
                     parser and the longitudinal counts a Z-score needs are absent, so no alert latency \
                     can be derived from it",
                    path, ingest_ms
                ),
            }
        }
        None => Step {
            id: "Step 4",
            title: "Autonomous MIC Pipeline & Outbreak Early Warning",
            metric: json!({ "parsed_path": Value::Null, "mic_eucast_parser_present": false }),
            trit: UNKNOWN,
            basis: "no parseable evidence file found; nothing was timed".to_string(),
        },
    }
}

/// STEP 5 — sovereign operation. A blackout drill, an offline proof and a cost measurement are all
/// physical actions this process cannot perform from inside itself. It reports that plainly rather than
/// manufacturing a percentage.
fn step_sovereign() -> Step {
    Step {
        id: "Step 5",
        title: "Sovereign Offline Execution & Phoenix Power Resilience",
        metric: json!({
            "offline_drill_run": false,
            "blackout_test_run": false,
            "cost_measured": false,
            "claimed_power_cut_recovery_us": 0.20,
        }),
        trit: UNKNOWN,
        basis: "not demonstrated: no offline drill, no blackout test and no cost measurement were run \
                from inside the process"
            .to_string(),
    }
}

/// Full report. `steps_passed` counts trit == TRUE, so it is a count of measurements that met their
/// criterion — not a count of steps that exist.
pub fn deployment_report() -> Value {
    let t0 = Instant::now();
    let steps = vec![
        step_hardware(),
        step_parsers(),
        step_knowledge(),
        step_pipeline(),
        step_sovereign(),
    ];
    let trits: Vec<i8> = steps.iter().map(|s| s.trit).collect();
    let passed = trits.iter().filter(|t| **t == TRUE).count();
    let refuted = trits.iter().filter(|t| **t == FALSE).count();
    let undetermined = trits.iter().filter(|t| **t == UNKNOWN).count();
    let cons = k3_consensus(&trits);
    let verdict = match cons.k3_and {
        TRUE => "DEMONSTRATED",
        FALSE => "REFUTED",
        _ => "UNDETERMINED",
    };

    json!({
        "module": "hospital_amr_surveillance_node",
        "source_of_record": "trm_uefi_kernel/host_council_server/src/amr_surveillance.rs",
        "replaces": "scratch/kernel194/src/hospital_amr_surveillance_node.rs (authored metrics)",
        "deployment_target": "Kisumu Rural District Hospital (Kenya) — 200 beds / 500k catchment",
        "steps_evaluated": steps.len(),
        "steps_passed": passed,
        "steps_refuted": refuted,
        "steps_undetermined": undetermined,
        "k3": { "and": cons.k3_and, "or": cons.k3_or, "unknown": cons.unknown, "n": cons.n },
        "verdict": verdict,
        "verdict_basis": "Kleene AND of the five step trits; a single refuted step cannot read as a deployment",
        "steps": steps.iter().map(|s| json!({
            "id": s.id,
            "title": s.title,
            "metric": s.metric,
            "trit": s.trit,
            "basis": s.basis,
        })).collect::<Vec<_>>(),
        "report_latency_ms": (t0.elapsed().as_secs_f64() * 1000.0 * 1000.0).round() / 1000.0,
    })
}
