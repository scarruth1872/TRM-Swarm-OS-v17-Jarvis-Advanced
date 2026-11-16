//! Handlers for the two modules whose portable surface was a naming difference and whose aggregate
//! surface was genuinely missing: the cognitive load balancer and the self-fuzzing engine.
//!
//! Both are included from the bare-metal kernel with `#[path]`, so the code answering these routes is
//! the same file the kernel compiles -- not a second implementation. The load balancer is stateful, so
//! instances are NAMED and resettable: that is what lets a comparison run against a fresh oracle
//! instead of against whatever the previous request left behind.

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::{json, Value};

use crate::cognitive_load_balancer as load;
use crate::self_fuzzing_engine as fuzz;

/// Named balancer instances. The source keeps one module-level singleton (`_load_balancer`); the
/// routes keep one PER NAME so a caller can start from a known-empty state deterministically.
static BALS: Mutex<Option<HashMap<String, load::CognitiveLoadBalancer>>> = Mutex::new(None);

fn with_bal<R>(instance: &str, make_if_missing: bool, f: impl FnOnce(&mut load::CognitiveLoadBalancer) -> R) -> Option<R> {
    let mut guard = BALS.lock().ok()?;
    let map = guard.get_or_insert_with(HashMap::new);
    if make_if_missing && !map.contains_key(instance) {
        map.insert(instance.to_string(), load::CognitiveLoadBalancer::new());
    }
    map.get_mut(instance).map(f)
}

fn load_view_json(v: &load::AgentLoadView) -> Value {
    // Key names are the source's own dict keys, so a comparison needs no translation.
    json!({
        "role": v.role,
        "task_count": v.task_count,
        "avg_completion_time": v.avg_completion_time,
        "current_complexity": v.current_complexity,
        "last_active": v.last_active,
        "load_score": v.load_score,
    })
}

/// `POST /native/load/reset` — start a named instance from the source's own initial state.
pub fn load_reset(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let instance = b["instance"].as_str().unwrap_or("default").to_string();
    // The source's `__init__` takes no clock; the epoch entered this port only as the recorded stamp,
    // which `record_task_start` / `record_task_end` still take per call.
    let mut guard = match BALS.lock() {
        Ok(g) => g,
        Err(_) => return Err("balancer instance pool poisoned".to_string()),
    };
    let map = guard.get_or_insert_with(HashMap::new);
    map.insert(instance.clone(), load::CognitiveLoadBalancer::new());
    let bal = &map[&instance];
    Ok(json!({
        "instance": instance,
        "roster_len": bal.roster_len,
        "agents": bal.agent_loads.iter().map(|a| a.role.clone()).collect::<Vec<_>>(),
    }))
}

/// `POST /native/load/start` — source's `record_task_start`.
pub fn load_start(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let instance = b["instance"].as_str().unwrap_or("default");
    let role = b["role"].as_str().unwrap_or("");
    let epoch_s = b["epoch_s"].as_u64().unwrap_or(0);
    Ok(match with_bal(instance, true, |bal| bal.record_task_start(role, epoch_s)) {
        Some(recorded) => json!({"instance": instance, "role": role, "recorded": recorded}),
        None => json!({"error": "no such instance"}),
    })
}

/// `POST /native/load/end` — source's `record_task_end`.
pub fn load_end(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let instance = b["instance"].as_str().unwrap_or("default");
    let role = b["role"].as_str().unwrap_or("");
    let completion_ms = b["completion_ms"].as_f64().unwrap_or(0.0) as f32;
    let complexity = b["complexity"].as_f64().unwrap_or(0.0) as f32;
    let epoch_s = b["epoch_s"].as_u64().unwrap_or(0);
    Ok(match with_bal(instance, true, |bal| bal.record_task_end(role, completion_ms, complexity, epoch_s)) {
        Some(recorded) => json!({"instance": instance, "role": role, "recorded": recorded}),
        None => json!({"error": "no such instance"}),
    })
}

/// `POST /native/load/agent` — source's `get_agent_load`, plus the balancer's own `cognitive_load`.
pub fn load_agent(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let instance = b["instance"].as_str().unwrap_or("default");
    let role = b["role"].as_str().unwrap_or("");
    Ok(match with_bal(instance, true, |bal| {
        let view = bal.get_agent_load(role);
        let cognitive = bal.get_agent_metrics(role).map(|m| m.cognitive_load()).unwrap_or(0.0);
        (view, cognitive)
    }) {
        Some((view, cognitive)) => {
            let mut out = load_view_json(&view);
            out["cognitive_load"] = json!(cognitive);
            out["instance"] = json!(instance);
            out
        }
        None => json!({"error": "no such instance"}),
    })
}

/// `POST /native/load/summary` — source's `get_swarm_load_summary`. The source's `timestamp` field is
/// the host clock and is returned as the caller's own `now_epoch_s` where supplied, never invented.
pub fn load_summary(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let instance = b["instance"].as_str().unwrap_or("default");
    Ok(match with_bal(instance, true, |bal| bal.get_swarm_load_summary()) {
        Some(s) => {
            let mut out = json!({
                "instance": instance,
                "agents": s.agents.iter().map(load_view_json).collect::<Vec<_>>(),
                "total_pending_tasks": s.total_pending_tasks,
                "active_agents": s.active_agents,
                "total_agents": s.total_agents,
                "utilization_pct": s.utilization_pct,
            });
            if let Some(now) = b.get("now_epoch_s").and_then(|v| v.as_f64()) {
                out["timestamp"] = json!(now);
            }
            out
        }
        None => json!({"error": "no such instance"}),
    })
}

/// `POST /native/load/redistribute` — source's `redistribute_task`. `task_description` is accepted and
/// never read: the source passes it straight to a log line and the rule does not consume it, so the
/// port gives it the same non-effect rather than inventing a use for it.
pub fn load_redistribute(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let instance = b["instance"].as_str().unwrap_or("default");
    let task = b["task_description"].as_str().unwrap_or("");
    let preferred: Vec<String> = b["preferred_roles"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).map(String::from).collect())
        .unwrap_or_default();
    Ok(match with_bal(instance, true, |bal| bal.redistribute_task(task, &preferred)) {
        Some(Some(role)) => json!({"instance": instance, "selected": role}),
        Some(None) => json!({"instance": instance, "selected": Value::Null}),
        None => json!({"error": "no such instance"}),
    })
}

/// `POST /native/load/capable` — source's `get_capable_roles`.
pub fn load_capable(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let instance = b["instance"].as_str().unwrap_or("default");
    let capability = b["capability"].as_str().unwrap_or("");
    Ok(match with_bal(instance, true, |bal| bal.get_capable_roles(capability)) {
        Some(roles) => json!({"instance": instance, "capability": capability, "roles": roles}),
        None => json!({"error": "no such instance"}),
    })
}

// --- self-fuzzing engine: the source's file-scan surface ---

fn strings(b: &Value, key: &str) -> Vec<String> {
    b[key]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).map(String::from).collect())
        .unwrap_or_default()
}

fn tally(pairs: &[(String, u64)]) -> Value {
    let mut m = serde_json::Map::new();
    for (k, n) in pairs {
        m.insert(k.clone(), json!(n));
    }
    Value::Object(m)
}

fn findings_json(hits: &[fuzz::Vulnerability]) -> Vec<Value> {
    hits.iter()
        .map(|v| {
            json!({
                "file": v.file, "line": v.line, "line_content": v.line_content,
                "vuln_type": v.vuln_type, "severity": v.severity,
                "antibody_type": v.antibody_type, "fingerprint": v.fingerprint,
            })
        })
        .collect()
}

/// `POST /native/fuzz/scan-lines` — the source's `scan_lines_native` per-line codes.
pub fn fuzz_scan_lines(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let lines = strings(&b, "lines");
    let codes: Vec<u8> = lines.iter().map(|l| fuzz::SelfFuzzingEngine::scan_line_code(l)).collect();
    Ok(json!({"count": codes.len(), "codes": codes}))
}

/// `POST /native/fuzz/scan-text` — the source's `scan_file` scanning step over content the caller read.
pub fn fuzz_scan_text(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let file = b["file"].as_str().unwrap_or("");
    let content = b["content"].as_str().unwrap_or("");
    let mut engine = fuzz::SelfFuzzingEngine::new();
    let hits = engine.scan_text(file, content);
    Ok(json!({
        "file": file,
        "lines_scanned": engine.scanned_lines,
        "count": hits.len(),
        "findings": findings_json(&hits),
    }))
}

/// `POST /native/fuzz/scan-all` — the source's `scan_all` counters over caller-supplied files.
pub fn fuzz_scan_all(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let files: Vec<(String, String)> = b["files"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|f| Some((f["path"].as_str()?.to_string(), f["content"].as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default();
    let mut engine = fuzz::SelfFuzzingEngine::new();
    let s = engine.scan_all(&files);
    Ok(json!({
        "files_scanned": s.files_scanned,
        "lines_scanned": s.lines_scanned,
        "vulnerabilities": s.vulnerabilities,
        "antibodies": s.antibodies_generated,
        "by_severity": tally(&s.by_severity),
        "by_type": tally(&s.by_type),
    }))
}

/// `POST /native/fuzz/save-plan` — the source's whole write path minus the file I/O: scan the content,
/// generate an antibody per finding, then apply the vault's dedup rule and report previous/new/total.
pub fn fuzz_save_plan(doc: &Value) -> Result<Value, String> {
    let b = doc;
    let file = b["file"].as_str().unwrap_or("");
    let content = b["content"].as_str().unwrap_or("");
    let existing = strings(&b, "existing_fingerprints");

    let mut engine = fuzz::SelfFuzzingEngine::new();
    let hits = engine.scan_text(file, content);
    let antibodies: Vec<fuzz::DdrAntibody> = hits.iter().map(|v| engine.create_antibody(v)).collect();
    let plan = engine.save_plan(&existing, &antibodies);
    Ok(json!({
        "previous": plan.previous,
        "new": plan.new,
        "total": plan.total,
        "new_fingerprints": plan.appended.iter().map(|a| a.fingerprint.clone()).collect::<Vec<_>>(),
        "antibody_ids": plan.appended.iter().map(|a| a.antibody_id.clone()).collect::<Vec<_>>(),
    }))
}
