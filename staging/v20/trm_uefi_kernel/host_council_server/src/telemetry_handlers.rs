// Rollout 14: the arms that answered 200 with values nobody measured.
//
// Each builder serves a measured or derived value, or null with the reason it cannot be measured.
// What the previous payloads asserted: throughput_ops_sec 4,251,552 and avg_latency_ns 235.2 (with
// active_nodes 2 and a consensus rate of 1.0), three skills "ACTIVE", zero active threats with a
// locked vault, a docking engine "Vina r=0.9097" against a benchmark that fails 0.8 A rotated and
// 1.5 A transverse, and an "O(1) memory bound / 16 recursive steps" presented as measured fact.

use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

static REQUESTS: AtomicU64 = AtomicU64::new(0);
static NANOS: AtomicU64 = AtomicU64::new(0);

/// Called once per handled request from the accept loop, so refusals and reads are both counted.
pub fn note_request(nanos: u64) {
    REQUESTS.fetch_add(1, Ordering::Relaxed);
    NANOS.fetch_add(nanos, Ordering::Relaxed);
}

/// This process's own request count and accumulated handler nanoseconds. Exposed so routes that
/// report per-agent load can divide the real totals instead of asserting a per-call constant.
pub fn counters() -> (u64, u64) {
    (
        REQUESTS.load(Ordering::Relaxed),
        NANOS.load(Ordering::Relaxed),
    )
}

fn read_json(rel: &str) -> Option<Value> {
    let p = crate::k3_retrieval::repo_relative(rel)?;
    serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()
}

/// Throughput and latency are this process's own request count and accumulated handler time over its
/// own uptime -- both measured, so the quotient is too.
pub fn telemetry_payload() -> Value {
    let n = crate::telemetry_handlers::REQUESTS.load(Ordering::Relaxed);
    let up = crate::host_probe::process_uptime_seconds().unwrap_or(0.0).max(0.001);
    let mesh = crate::mesh_handlers::mesh_nodes();
    let agents = mesh.get("nodes").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let peers = mesh
        .get("node_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .saturating_sub(agents as u64);
    let (phi, phi_basis, _clock, _mem) = crate::host_probe::phi_v();
    json!({
        "total_agents": agents,
        "federation_peers": peers,
        "requests_served": n,
        "uptime_seconds": up,
        "throughput_ops_sec": (n as f64 / up).round(),
        "avg_latency_ns": if n > 0 { (NANOS.load(Ordering::Relaxed) as f64 / n as f64).round() } else { 0.0 },
        "measured_basis": "this process's request count and accumulated handler time, over its own uptime",
        "phi_v": phi,
        "phi_v_basis": phi_basis,
        "k3_consensus_rate": null,
        "k3_consensus_rate_reason": "no vote-outcome counter exists; the council's motions are ledger blocks, not a rate",
        "previously_asserted": { "throughput_ops_sec": 4251552, "avg_latency_ns": 235.2, "phi_v": 0.86,
                                 "k3_consensus_rate": 1.0, "active_nodes": 2, "status": "HEALTHY_SOVEREIGN" }
    })
}

/// The real skill modules on disk. The kernel does not execute them, so no status is claimed.
pub fn learning_payload() -> Value {
    let mut skills: Vec<Value> = Vec::new();
    if let Some(d) = crate::k3_retrieval::repo_relative("swarm_v2/skills") {
        if let Ok(rd) = std::fs::read_dir(&d) {
            let mut names: Vec<(String, u64)> = rd
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let p = e.path();
                    let n = p.file_name()?.to_string_lossy().to_string();
                    if !n.ends_with(".py") || n == "__init__.py" {
                        return None;
                    }
                    let b = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                    Some((n, b))
                })
                .collect();
            names.sort();
            skills = names
                .into_iter()
                .map(|(n, b)| {
                    json!({ "id": n.trim_end_matches(".py").to_string(),
                            "module": format!("swarm_v2/skills/{n}"), "bytes": b, "status": "on disk" })
                })
                .collect();
        }
    }
    json!({
        "skills": skills,
        "skill_count": skills.len(),
        "executed_by_kernel": false,
        "basis": "the modules present in swarm_v2/skills; the kernel does not execute them, so it claims no ACTIVE state",
        "previously_asserted": ["mamba3_ssm ACTIVE", "k3_ternary ACTIVE", "sram_15_fabrics ACTIVE"]
    })
}

/// Key material presence is observable; a LOCKED/UNLOCKED state is not. The threat list now comes
/// from the real DDR scan report and the real ingested threat database, both on disk — never an
/// invented count.
pub fn security_payload() -> Value {
    let vault_key = crate::k3_retrieval::repo_relative(".swarm/.vault_key").is_some();
    let scan = read_json("swarm_v2_artifacts/security_scan_report.json");
    let threat_db = read_json("swarm_v2_artifacts/DDR_THREAT_DATABASE.json");

    let get = |v: &Option<Value>, seg: &str, field: &str| -> Option<i64> {
        v.as_ref()
            .and_then(|v| v.get(seg))
            .and_then(|s| s.get(field))
            .and_then(|f| f.as_i64())
    };
    let critical = scan
        .as_ref()
        .and_then(|s| s.get("by_severity"))
        .and_then(|b| b.get("critical"))
        .and_then(|v| v.as_i64());
    let high = scan
        .as_ref()
        .and_then(|s| s.get("by_severity"))
        .and_then(|b| b.get("high"))
        .and_then(|v| v.as_i64());
    let findings_total = scan.as_ref().and_then(|s| s.get("findings")).and_then(|v| v.as_i64());
    let files_scanned = scan.as_ref().and_then(|s| s.get("files_scanned")).and_then(|v| v.as_i64());
    let scanned_at = scan.as_ref().and_then(|s| s.get("scanned_at")).cloned();

    let mitre = |seg: &str| get(&threat_db, "mitre", seg).unwrap_or(0);
    let cwe = get(&threat_db, "cwe", "weaknesses").unwrap_or(0);
    let cve = read_json("swarm_v2_artifacts/DDR_CVE_SUMMARY.json");
    let cve_count = cve.as_ref().and_then(|c| c.get("count")).and_then(|v| v.as_i64()).unwrap_or(0);
    let cve_sev = |sev: &str| {
        cve.as_ref()
            .and_then(|c| c.get("by_severity"))
            .and_then(|b| b.get(sev))
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    };
    let records_ingested = mitre("techniques") + mitre("tactics") + mitre("mitigations") + mitre("malware") + cwe + cve_count;

    // The critical findings of the last scan are the real threat list (bounded for transport).
    let threats: Vec<Value> = scan
        .as_ref()
        .and_then(|s| s.get("findings_list"))
        .and_then(|a| a.as_array())
        .map(|list| {
            list.iter()
                .filter(|f| f.get("severity").and_then(|v| v.as_str()) == Some("critical"))
                .take(100)
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    json!({
        "stats": {
            "antibodies_loaded": crate::actions_handlers::ddr_antibody_count().map(|c| json!(c)).unwrap_or(json!(null)),
            "antibodies_loaded_basis": "swarm_v2_artifacts/DDR_ANTIBODIES.json (the payload asserted 1801)",
            "active_threats": critical.unwrap_or(0),
            "active_threats_basis": "critical findings in swarm_v2_artifacts/security_scan_report.json; 0 means no scan has run or no critical match",
            "vault_key_present": vault_key,
            "vault_key_basis": ".swarm/.vault_key",
            "vault_status": null,
            "vault_status_reason": "nothing on this host tracks a locked/unlocked state; only the key's presence is observable"
        },
        "threat_database": {
            "records_ingested": records_ingested,
            "mitre_techniques": mitre("techniques"),
            "mitre_tactics": mitre("tactics"),
            "mitre_mitigations": mitre("mitigations"),
            "mitre_malware": mitre("malware"),
            "cwe_weaknesses": cwe,
            "cve_records": cve_count,
            "cve_critical": cve_sev("CRITICAL"),
            "cve_high": cve_sev("HIGH"),
            "cve_medium": cve_sev("MEDIUM"),
            "cve_low": cve_sev("LOW"),
            "basis": "swarm_v2_artifacts/DDR_THREAT_DATABASE.json (MITRE ATT&CK + CWE) + DDR_CVE_SUMMARY.json (fkie-cad NVD CVE), downloaded live 2026-10-03"
        },
        "security_scan": {
            "scanned_at": scanned_at.unwrap_or(json!(null)),
            "files_scanned": files_scanned.unwrap_or(0),
            "findings_total": findings_total.unwrap_or(0),
            "critical": critical.unwrap_or(0),
            "high": high.unwrap_or(0),
            "basis": "swarm_v2_artifacts/security_scan_report.json (written by the DDR antibody code scan)"
        },
        "threats": threats,
        "threats_reason": "the critical findings of the last DDR antibody scan; empty when no scan has run"
    })
}

/// The research notes actually on disk, plus the platform's own resolution benchmark. The docking
/// figure this payload used to carry ("Vina r=0.9097") is not what the benchmark reports.
pub fn research_stats_payload() -> Value {
    let mut files = 0u64;
    let mut newest = 0u64;
    let mut newest_name = String::new();
    let mut leads: Vec<String> = Vec::new();
    if let Some(d) = crate::k3_retrieval::repo_relative("swarm_v2_artifacts/research") {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.filter_map(|e| e.ok()) {
                files += 1;
                let name = e.file_name().to_string_lossy().to_string();
                let upper = name.to_uppercase();
                if let Some(pos) = upper.find("TRM-ABX") {
                    let digits: String = upper[pos..]
                        .chars()
                        .skip_while(|c| !c.is_ascii_digit())
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if !digits.is_empty() {
                        let lead = format!("TRM-ABX-{digits}");
                        if !leads.contains(&lead) {
                            leads.push(lead);
                        }
                    }
                }
                if let Ok(md) = e.metadata() {
                    if let Ok(t) = md.modified() {
                        let s = t.duration_since(std::time::UNIX_EPOCH).map(|x| x.as_secs()).unwrap_or(0);
                        if s > newest {
                            newest = s;
                            newest_name = name;
                        }
                    }
                }
            }
        }
    }
    leads.sort();
    let bench = read_json("swarm_v2_artifacts/docking_pose_resolution_benchmark.json");
    json!({
        "research_note_files": files,
        "lead_ids_seen_on_disk": leads,
        "newest_note": if newest_name.is_empty() { json!(null) } else { json!({ "file": newest_name, "unix_ts": newest }) },
        "scorer": bench.as_ref().and_then(|b| b.get("scorer_type")).cloned().unwrap_or(json!(null)),
        "docking_resolution": bench.as_ref().and_then(|b| b.get("resolution_verdict")).cloned().unwrap_or(json!(null)),
        "docking_benchmark_source": "swarm_v2_artifacts/docking_pose_resolution_benchmark.json",
        "clsi_standard": "CLSI M07-A10 Broth Microdilution",
        "clsi_basis": "declared in DRUG_DISCOVERY.md; not measured on this host",
        "pre_registered_leads": null,
        "pre_registered_leads_reason": "the two ids this payload named (TRM-ABX-219/082) have no record on disk",
        "measured_docking_note": "the platform's own benchmark reports fails_0.8A_rotated and fails_1.5A_transverse; the earlier measured figure was 7/11 poses within 2.0 A, median 1.41 A",
        "previously_asserted": { "pre_registered_leads": 2, "docking_engine": "Vina r=0.9097", "status": "WET_LAB_READINESS_SEALED" }
    })
}

/// Declarations are reported as declarations. `python_active` and uptime stay measured.
pub fn brain_status_payload() -> Value {
    json!({
        "status": "ONLINE",
        "python_active": false,
        "python_active_basis": "measured: no Python process is in this server's path",
        "uptime_seconds": crate::host_probe::process_uptime_seconds(),
        "architecture": "Native Rust TRM Brain (Mamba-3 SSM)",
        "memory_bound": "O(1)",
        "max_recursive_steps": 16,
        "k3_logic": "Balanced Ternary (+1/0/-1)",
        "declared_not_measured": ["architecture", "memory_bound", "max_recursive_steps", "k3_logic"],
        "basis": "these four are module declarations carried from the engine's own design; the kernel reports them as declarations rather than measurements"
    })
}
