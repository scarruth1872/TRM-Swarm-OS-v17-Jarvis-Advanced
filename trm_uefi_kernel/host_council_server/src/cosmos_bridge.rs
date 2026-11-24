//! Live Cosmos 3 Edge VLA Safety Bridge.
//!
//! Ingests `CosmosActionChunk` payloads (`POST /cosmos/chunk`), validates each
//! trajectory action against deterministic safety boundaries (+1 safe, -1 hazard
//! intercept, 0 ambiguous re-plan) and routes the semantic plan through the
//! 584-agent bitboard evaluator (`agent_bitboard::evaluate_directive`).

use crate::agent_bitboard;
use serde_json::{json, Value};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

#[derive(Default)]
struct CosmosStats {
    chunks_processed: u64,
    actions_verified: u64,
    hazards_intercepted: u64,
    ambiguities: u64,
    latest_chunk_id: String,
    latest_verdict: String,
}

static COSMOS_STATE: OnceLock<Mutex<CosmosStats>> = OnceLock::new();

fn state() -> &'static Mutex<CosmosStats> {
    COSMOS_STATE.get_or_init(|| Mutex::new(CosmosStats::default()))
}

/// Deterministic K3 verification for a single VLA action entry.
pub fn verify_action_k3(action: &Value) -> i8 {
    let action_type = action
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    if matches!(action_type, "halt" | "freeze" | "emergency_stop") {
        return -1;
    }

    if let Some(params) = action.get("params").and_then(|v| v.as_object()) {
        for (k, val) in params {
            if let Some(num) = val.as_f64() {
                if !num.is_finite() {
                    return -1;
                }
                if k.starts_with("max_") && num > 1_000_000.0 {
                    return -1;
                }
                if k.starts_with("min_") && num < 0.0 && !k.ends_with("_temp") {
                    return -1;
                }
            }
        }
        if let Some(pos) = params.get("position").and_then(|v| v.as_array()) {
            if pos.len() >= 3 {
                for coord in pos.iter().take(3) {
                    if coord.as_f64().map(|x| x.abs() > 1000.0).unwrap_or(true) {
                        return 0;
                    }
                }
            }
        }
    }

    1
}

pub fn ingest_chunk(payload: &Value) -> Value {
    let t0 = Instant::now();
    let chunk_id = payload
        .get("chunk_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            format!("cosmos_{ms}")
        });
    let semantic_plan = payload
        .get("semantic_plan")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let actions = payload
        .get("actions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut safe_count = 0u64;
    let mut hazard_count = 0u64;
    let mut ambig_count = 0u64;
    let mut action_trits = Vec::with_capacity(actions.len());

    for act in &actions {
        let trit = verify_action_k3(act);
        action_trits.push(trit);
        match trit {
            1 => safe_count += 1,
            -1 => hazard_count += 1,
            _ => ambig_count += 1,
        }
    }

    let plan_eval = agent_bitboard::evaluate_directive(semantic_plan);
    let plan_trit = plan_eval["consensus_trit"].as_i64().unwrap_or(0) as i8;

    let overall_trit: i8 = if hazard_count > 0 || plan_trit == -1 {
        -1
    } else if ambig_count > 0 || plan_trit == 0 {
        0
    } else {
        1
    };

    let verdict = match overall_trit {
        -1 => "HAZARD_INTERCEPT",
        0 => "AMBIGUOUS_REPLAN",
        _ => "VERIFIED_SAFE",
    };

    {
        let mut g = match state().lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        g.chunks_processed += 1;
        g.actions_verified += safe_count;
        g.hazards_intercepted += hazard_count;
        g.ambiguities += ambig_count;
        g.latest_chunk_id = chunk_id.clone();
        g.latest_verdict = verdict.to_string();
    }

    json!({
        "chunk_id": chunk_id,
        "actions_total": actions.len(),
        "safe": safe_count,
        "hazards": hazard_count,
        "ambiguities": ambig_count,
        "action_trits": action_trits,
        "semantic_plan_trit": plan_trit,
        "overall_trit": overall_trit,
        "verdict": verdict,
        "elapsed_ns": t0.elapsed().as_nanos(),
    })
}

pub fn status_json() -> Value {
    let g = match state().lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    json!({
        "bridge": "Cosmos3EdgeBridge (Rust Native)",
        "chunks_processed": g.chunks_processed,
        "actions_verified": g.actions_verified,
        "hazards_intercepted": g.hazards_intercepted,
        "ambiguities": g.ambiguities,
        "latest_chunk_id": if g.latest_chunk_id.is_empty() { "none" } else { &g.latest_chunk_id },
        "latest_verdict": if g.latest_verdict.is_empty() { "IDLE" } else { &g.latest_verdict },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmos_chunk_verification() {
        let safe_chunk = json!({
            "chunk_id": "test-safe",
            "semantic_plan": "move arm to home position",
            "actions": [
                {"type": "move", "params": {"position": [10.0, 20.0, 30.0]}}
            ]
        });
        let r1 = ingest_chunk(&safe_chunk);
        assert_eq!(r1["overall_trit"], json!(1));
        assert_eq!(r1["verdict"], json!("VERIFIED_SAFE"));

        let hazard_chunk = json!({
            "chunk_id": "test-hazard",
            "semantic_plan": "stop immediately",
            "actions": [
                {"type": "emergency_stop", "params": {}}
            ]
        });
        let r2 = ingest_chunk(&hazard_chunk);
        assert_eq!(r2["overall_trit"], json!(-1));
        assert_eq!(r2["verdict"], json!("HAZARD_INTERCEPT"));

        let ambig_chunk = json!({
            "chunk_id": "test-ambig",
            "semantic_plan": "reach distant point",
            "actions": [
                {"type": "move", "params": {"position": [1500.0, 0.0, 0.0]}}
            ]
        });
        let r3 = ingest_chunk(&ambig_chunk);
        assert_eq!(r3["overall_trit"], json!(0));
        assert_eq!(r3["verdict"], json!("AMBIGUOUS_REPLAN"));
    }
}
