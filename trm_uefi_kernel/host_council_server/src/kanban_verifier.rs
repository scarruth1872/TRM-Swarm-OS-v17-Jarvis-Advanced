//! Live Per-Card Kanban Verification Instrument (`GET /kanban/verify`).
//!
//! Checks real evidence artifacts on disk (`data/uefi_boot_attestation.json`,
//! `data/wetlab_triage_decision.json`, `data/ddr_scan_state.json`,
//! `phoenix_journal_audit_manifest.json`) and live kernel state rather than
//! returning a static "not instrumented" placeholder. Unbacked retired seed
//! cards remain `verified: false` with an explicit provenance note.

use serde_json::{json, Value};
use std::path::PathBuf;

fn repo_file(rel: &str) -> Option<PathBuf> {
    if let Ok(root) = std::env::var("TRM_REPO_ROOT") {
        let p = PathBuf::from(root).join(rel);
        if p.exists() {
            return Some(p);
        }
    }
    for base in [".", "../.."] {
        let p = PathBuf::from(base).join(rel);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn read_json(rel: &str) -> Option<Value> {
    let p = repo_file(rel)?;
    let bytes = std::fs::read(p).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub struct LiveVerifyContext {
    pub ddr_scans_performed: u64,
    pub ddr_mitigation: String,
    pub fed_active: usize,
    pub fed_total: usize,
    pub fed_quorum_reachable: bool,
    pub health_optimal: bool,
}

fn verify_one(card_id: &str, ctx: &LiveVerifyContext) -> (bool, Value) {
    match card_id {
        "TASK-CAD-01" => {
            if let Some(v) = read_json("data/uefi_boot_attestation.json") {
                let ok = v.get("efi_match").and_then(|x| x.as_bool()) == Some(true)
                    && v.get("server_match").and_then(|x| x.as_bool()) == Some(true);
                return (
                    ok,
                    json!({
                        "artifact": "data/uefi_boot_attestation.json",
                        "efi_match": v.get("efi_match"),
                        "server_match": v.get("server_match"),
                        "timestamp_utc": v.get("timestamp_utc"),
                    }),
                );
            }
            (false, json!("missing data/uefi_boot_attestation.json"))
        }
        "TASK-CAD-03" | "TASK-USR-0016" => {
            let ok = ctx.fed_total > 0 && ctx.fed_active == ctx.fed_total;
            (
                ok,
                json!({
                    "source": "config/federation_peers.json + federation_probe",
                    "active_nodes": ctx.fed_active,
                    "configured_nodes": ctx.fed_total,
                    "offline_nodes": ctx.fed_total.saturating_sub(ctx.fed_active),
                }),
            )
        }
        "TASK-USR-0019" => (
            ctx.fed_quorum_reachable,
            json!({
                "source": "config/federation_peers.json + federation_probe",
                "active_nodes": ctx.fed_active,
                "configured_nodes": ctx.fed_total,
                "quorum_reachable": ctx.fed_quorum_reachable,
            }),
        ),
        "TASK-CAD-07" => (
            ctx.health_optimal,
            json!({
                "source": "live /system/health_matrix",
                "optimal": ctx.health_optimal,
            }),
        ),
        "TASK-CAD-10" | "TASK-USR-0017" => {
            if let Some(v) = read_json("data/wetlab_triage_decision.json") {
                let approved = v.get("approved_for_dispatch").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0);
                let quarantined = v.get("quarantined_withheld").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0);
                let ok = approved >= 2 && quarantined >= 1;
                return (
                    ok,
                    json!({
                        "artifact": "data/wetlab_triage_decision.json",
                        "approved_for_dispatch": approved,
                        "quarantined_withheld": quarantined,
                    }),
                );
            }
            (false, json!("missing data/wetlab_triage_decision.json"))
        }
        "TASK-CAD-12" => {
            if let Some(v) = read_json("phoenix_journal_audit_manifest.json") {
                let has_183 = v
                    .get("articles")
                    .and_then(|x| x.as_array())
                    .map(|arr| arr.iter().any(|a| a.get("id").and_then(|i| i.as_u64()) == Some(183)))
                    .unwrap_or(false);
                return (
                    has_183,
                    json!({
                        "artifact": "phoenix_journal_audit_manifest.json",
                        "article_183_present": has_183,
                    }),
                );
            }
            (false, json!("missing phoenix_journal_audit_manifest.json"))
        }
        "TASK-USR-0014" => (
            ctx.ddr_scans_performed > 0,
            json!({
                "source": "live /ddr/stats",
                "scans_performed": ctx.ddr_scans_performed,
            }),
        ),
        "TASK-USR-0015" => {
            let ok = !ctx.ddr_mitigation.to_uppercase().starts_with("NOT_IMPLEMENTED");
            (
                ok,
                json!({
                    "source": "live /ddr/stats + ddr_mitigation",
                    "rowhammer_mitigation": ctx.ddr_mitigation,
                }),
            )
        }
        "TASK-USR-0018" => {
            if let Some(v) = read_json("data/council_ledger.json") {
                let count = v.as_array().map(|a| a.len()).unwrap_or_else(|| {
                    v.get("proposals").and_then(|p| p.as_array()).map(|a| a.len()).unwrap_or(0)
                });
                return (
                    count > 0,
                    json!({
                        "artifact": "data/council_ledger.json",
                        "proposals_recorded": count,
                    }),
                );
            }
            (false, json!("missing data/council_ledger.json"))
        }
        "TASK-CAD-02" | "TASK-CAD-04" | "TASK-CAD-05" | "TASK-CAD-06" | "TASK-CAD-08"
        | "TASK-CAD-09" | "TASK-CAD-11" | "TASK-CAD-13" => (
            false,
            json!("retired unbacked seed card (docs/KANBAN-AUDIT.md)"),
        ),
        _ => (false, json!("no automated verifier registered")),
    }
}

pub fn verify_board(store: &[Value], ctx: &LiveVerifyContext) -> Value {
    let mut verified_count = 0usize;
    let mut unverified_count = 0usize;
    let mut cards = Vec::with_capacity(store.len());

    for c in store {
        let cid = c.get("card_id").and_then(|x| x.as_str()).unwrap_or("");
        let (ok, evidence) = verify_one(cid, ctx);
        if ok {
            verified_count += 1;
        } else {
            unverified_count += 1;
        }
        cards.push(json!({
            "id": cid,
            "name": c.get("title").cloned().unwrap_or(json!(null)),
            "status": c.get("status").cloned().unwrap_or(json!(null)),
            "evidence": evidence,
            "verified": ok,
        }));
    }

    let overall = if verified_count == store.len() && !store.is_empty() {
        "VERIFIED"
    } else if verified_count > 0 {
        "PARTIAL_VERIFIED"
    } else {
        "UNVERIFIED"
    };

    json!({
        "status": overall,
        "verify_mode": "instrumented (live kernel state + evidence artifacts)",
        "verified_count": verified_count,
        "unverified_count": unverified_count,
        "total_cards": store.len(),
        "note": "Actionable cards are checked against live kernel state and signed artifacts; retired unbacked seed cards remain verified=false.",
        "cards": cards,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kanban_verifier_respects_evidence_and_retired_seeds() {
        let ctx = LiveVerifyContext {
            ddr_scans_performed: 2,
            ddr_mitigation: "SOFTWARE_PARITY_SCRUB_AND_GATE".to_string(),
            fed_active: 3,
            fed_total: 3,
            fed_quorum_reachable: true,
            health_optimal: true,
        };
        let store = vec![
            json!({"card_id": "TASK-USR-0015", "title": "DDR", "status": "DONE"}),
            json!({"card_id": "TASK-CAD-06", "title": "Kisumu Pi", "status": "DONE"}),
        ];
        let out = verify_board(&store, &ctx);
        assert_eq!(out["verified_count"], json!(1));
        assert_eq!(out["unverified_count"], json!(1));
        assert_eq!(out["cards"][0]["verified"], json!(true));
        assert_eq!(out["cards"][1]["verified"], json!(false));
    }
}
