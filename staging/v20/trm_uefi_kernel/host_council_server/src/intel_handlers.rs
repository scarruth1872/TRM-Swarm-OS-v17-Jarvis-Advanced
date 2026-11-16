// /native/intel -- the council decided by swarm-intel itself.
//
// swarm-intel is the v18 crate that absorbed the intelligence layer into Rust: the 14-agent council, the mesh
// routing and the act disposition. It is wired as a path dependency the way swarm-proof, swarm-drug and
// swarm-docking are: the crate decides and this boundary adds no rule, threshold, default or re-derivation.
//
// Recorded rather than smoothed:
//   * `EnsembleField::load()` reads two ABSOLUTE WSL paths (/home/shawn/trm-rust/field_qwen.bin and
//     field_bonsai.bin). On this host those captures do not exist, so the field is the crate's own
//     deterministic fallback and `from_gpu` is false. The route reports that flag and the dimension count
//     instead of quietly presenting the fallback as a GPU field.
//   * `route()` returns ("hermes", 0) when no domain overlaps, i.e. a zero-overlap match is reported as
//     overlap 0 rather than being hidden as a confident owner.
//   * `council_vote` votes on the K3 collapse of [domain-fit, 0.9, 0.7] at tau 0.35 -- the crate's own
//     signals. The boundary passes the task through and returns the trits it produced.

use serde_json::{json, Value};

use swarm_intel::{council_vote, decide as intel_decide, ensemble_council, route as intel_route,
                  Task, AGENTS};

fn task(doc: &Value) -> Result<Task, String> {
    let class = doc.get("class").and_then(|v| v.as_str())
        .ok_or("`class` (string, e.g. \"mitigate\" | \"respond\" | \"triage\") is required")?;
    let domain = doc.get("domain").and_then(|v| v.as_str())
        .ok_or("`domain` (string) is required: it decides the routing and the domain-fit signal")?;
    Ok(Task {
        class: class.to_string(),
        reference: doc.get("reference").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        domain: domain.to_string(),
    })
}

/// The council roster, straight from the crate's own AGENTS table.
pub fn agents() -> Value {
    let rows: Vec<Value> = AGENTS
        .iter()
        .map(|(name, doms)| json!({"agent": name, "domains": doms}))
        .collect();
    json!({
        "count": rows.len(),
        "agents": rows,
        "basis": "swarm_intel::AGENTS -- the crate's own table, relayed without a copy",
    })
}

/// Mesh routing: the owning agent and the measured overlap.
pub fn route(doc: &Value) -> Result<Value, String> {
    let domain = doc.get("domain").and_then(|v| v.as_str())
        .ok_or("`domain` (string) is required")?;
    let (agent, overlap) = intel_route(domain);
    Ok(json!({
        "domain": domain,
        "agent": agent,
        "overlap": overlap,
        "basis": "swarm_intel::route -- real domain overlap; overlap 0 means no agent's domains matched",
    }))
}

/// The K3 council vote on one task.
pub fn vote(doc: &Value) -> Result<Value, String> {
    let t = task(doc)?;
    let (verdict, trits) = council_vote(&t);
    Ok(json!({
        "verdict": verdict.as_str(),
        "trits": trits,
        "signals": "swarm_intel::council_vote collapses [domain-fit, 0.9, 0.7] at tau 0.35 (the crate's)",
        "task": {"class": t.class, "reference": t.reference, "domain": t.domain},
    }))
}

/// The full loop on one task: route -> council -> disposition (the crate's own string).
pub fn decide(doc: &Value) -> Result<Value, String> {
    let t = task(doc)?;
    Ok(json!({
        "disposition": intel_decide(&t),
        "basis": "swarm_intel::decide -- the crate formats the disposition; this route relays it verbatim",
    }))
}

/// The ensemble-governed vote: the 2-model voltage field as the council's prior.
pub fn ensemble(doc: &Value) -> Result<Value, String> {
    let domain = doc.get("domain").and_then(|v| v.as_str())
        .ok_or("`domain` (string) is required")?;
    let tau = doc.get("tau").and_then(|v| v.as_f64()).unwrap_or(0.35) as f32;
    let field = ensemble_council::EnsembleField::load();
    let signal = field.signal(tau);
    let (verdict, trits) = ensemble_council::ensemble_vote(&field, domain, tau);
    Ok(json!({
        "domain": domain,
        "tau": tau,
        "verdict": verdict,
        "trits": trits,
        "field_signal": signal,
        "field": {
            "dims": field.values.len(),
            "from_gpu": field.from_gpu,
            "note": if field.from_gpu {
                "the GPU captures were read from /home/shawn/trm-rust/field_*.bin"
            } else {
                "the crate's captures at /home/shawn/trm-rust/field_*.bin are absent on this host, so this is \
                 the crate's deterministic fallback field -- reported, not presented as a GPU capture"
            },
        },
        "basis": "swarm_intel::ensemble_council::ensemble_vote -- domain-fit + the field's collapse at tau",
    }))
}
