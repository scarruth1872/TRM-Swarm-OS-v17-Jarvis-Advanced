//! /infrastructure/status, /infra/health and /infra/history -- the self-healing infrastructure FSM.
//!
//! The FSM is NOT ported here. It is trm_uefi_kernel/src/self_healing_infra.rs, linked by #[path] the
//! way the other modules of record are, and this boundary only feeds it and relays what it returns.
//!
//! Every input is a number the supervisor measured and published in its own status file:
//!
//!   heartbeat     <- last_ping_ms       when that service's port last answered
//!   timeout       <- interval_s         its own tick; a heartbeat older than one tick is stale
//!   restart_count <- restarts
//!   max_attempts  <- max_restarts
//!   cooldown      <- restart_delay_ms
//!   last_restart  <- last_restart_ms
//!
//! ONE input has no source at all, and that is stated rather than papered over: the supervisor keeps
//! no FAILURE counter, so `failures` is 0 and the FSM's DEGRADED branch (f > 2) can never fire here.
//! A 0 in that slot means "no counter exists", not "no failures happened", and every payload says so.
//!
//! `now` is the supervisor's own checked_ms, not this process's clock: the sample and the clock that
//! took it travel together, so an age is never computed across two different time bases.

use serde_json::{json, Value};

use crate::self_healing_infra as fsm;

const SUP_STATUS_REL: &str = "trm_uefi_kernel/host_supervisor/supervisor.status.json";
const SUP_CONTROL_REL: &str = "trm_uefi_kernel/host_supervisor/supervisor.control.json";

pub struct Table {
    pub services: Vec<Value>,
    pub checked_ms: u128,
    pub interval_s: u64,
    pub now_s: f64,
    pub running: bool,
}

/// The supervisor's published table, or None when it cannot be read -- which is NOT "everything is
/// fine" and NOT "there are no services".
pub fn table() -> Option<Table> {
    let path = crate::k3_retrieval::repo_relative(SUP_STATUS_REL)?;
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let services = doc.get("services")?.as_array()?.clone();
    let checked_ms = doc.get("checked_ms").and_then(|v| v.as_u64()).unwrap_or(0) as u128;
    let interval_s = doc.get("interval_s").and_then(|v| v.as_u64()).unwrap_or(10);
    let running = doc.get("running").and_then(|v| v.as_bool()).unwrap_or(true);
    Some(Table {
        services,
        checked_ms,
        interval_s,
        now_s: checked_ms as f64 / 1000.0,
        running,
    })
}

fn wall_now_s() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// The supervisor's table run through the FSM. One row per service, each carrying the verdict AND the
/// inputs it was derived from, so a reader can re-derive rather than trust.
pub fn fsm_rows(t: &Table) -> Vec<Value> {
    let n = t.services.len();
    let mut heartbeat = Vec::with_capacity(n);
    let mut timeout = Vec::with_capacity(n);
    let mut failures = Vec::with_capacity(n);
    let mut last_restart = Vec::with_capacity(n);
    let mut cooldown = Vec::with_capacity(n);
    let mut max_attempts = Vec::with_capacity(n);
    let mut restart_count = Vec::with_capacity(n);

    for s in &t.services {
        let ping_ms = s.get("last_ping_ms").and_then(|v| v.as_u64()).unwrap_or(0) as f64 / 1000.0;
        let restart_ms = s.get("last_restart_ms").and_then(|v| v.as_u64()).unwrap_or(0) as f64 / 1000.0;
        // A service that has never answered has NO heartbeat, and must not be handed the sample clock:
        // that makes elapsed zero, so a service the supervisor has never once seen up scores HEALTHY.
        // The first version of this file did exactly that and reported 7/7 healthy, two of them services
        // the supervisor had never started. Heartbeat 0 is the honest encoding of an absent measurement:
        // elapsed becomes the whole epoch, which exceeds any timeout, so the FSM calls it unresponsive.
        let hb = if ping_ms > 0.0 { ping_ms } else { 0.0 };
        heartbeat.push(hb);
        timeout.push(t.interval_s as f64);
        failures.push(0u32);
        last_restart.push(restart_ms);
        cooldown.push(3.0);
        max_attempts.push(s.get("max_restarts").and_then(|v| v.as_u64()).unwrap_or(10) as u32);
        restart_count.push(s.get("restarts").and_then(|v| v.as_u64()).unwrap_or(0) as u32);
    }

    let (statuses, eligible) =
        fsm::evaluate_services_health(
            &heartbeat, &timeout, &failures, &last_restart, &cooldown, &max_attempts, &restart_count, t.now_s,
        );

    t.services
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let code = statuses.get(i).copied().unwrap_or(0);
            // A service the supervisor will never start (declared, empty argv) is not an unhealthy one and
            // not a restarted one -- it is simply not running, on purpose. Scoring it HEALTHY hid it, and
            // scoring it UNRESPONSIVE would ask for a restart of something with nothing to start. It gets
            // its own state, and it is excluded from the liveness verdict below.
            let startable = s.get("startable").and_then(|v| v.as_bool()).unwrap_or(false);
            let sup_status = s.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let not_started = !startable && sup_status != "ONLINE";
            json!({
                "key": s.get("key").cloned().unwrap_or(Value::Null),
                "name": s.get("name").cloned().unwrap_or(Value::Null),
                "port": s.get("port").cloned().unwrap_or(Value::Null),
                "supervisor_status": s.get("status").cloned().unwrap_or(Value::Null),
                "health": if not_started { "not_started" } else { fsm::status_name(code) },
                "fsm_health": fsm::status_name(code),
                "code": code,
                "restart_eligible": !not_started && eligible.get(i).copied().unwrap_or(0) == 1,
                "restarts": s.get("restarts").cloned().unwrap_or(Value::Null),
                "max_restarts": s.get("max_restarts").cloned().unwrap_or(Value::Null),
                "last_ping_ms": s.get("last_ping_ms").cloned().unwrap_or(Value::Null),
                "last_restart_ms": s.get("last_restart_ms").cloned().unwrap_or(Value::Null),
            })
        })
        .collect()
}

/// The overall verdict, and the rule that produced it -- stated so it can be argued with.
fn overall(rows: &[Value]) -> &'static str {
    let codes: Vec<u8> = rows
        .iter()
        .filter(|r| r.get("health").and_then(|h| h.as_str()) != Some("not_started"))
        .filter_map(|r| r.get("code").and_then(|c| c.as_u64()))
        .map(|c| c as u8)
        .collect();
    if codes.contains(&fsm::SERVICE_STOPPED) {
        "DEGRADED"
    } else if codes.contains(&fsm::SERVICE_UNRESPONSIVE) {
        "HEALING"
    } else if codes.contains(&fsm::SERVICE_DEGRADED) {
        "DEGRADED"
    } else {
        "HEALTHY"
    }
}

fn unreadable() -> Value {
    json!({
        "status": "unknown",
        "error": "supervisor_status_unreadable",
        "detail": format!(
            "{} could not be read or parsed. An unreadable supervisor table is not an empty one and not a \
             healthy one, so neither is claimed.",
            SUP_STATUS_REL
        ),
        "supervisor_status_file": SUP_STATUS_REL,
    })
}

/// /infrastructure/status -- the dashboard's Infrastructure page.
pub fn status_payload() -> Value {
    let Some(t) = table() else {
        return unreadable();
    };
    let rows = fsm_rows(&t);
    let online: Vec<String> = rows
        .iter()
        .filter(|r| r.get("supervisor_status").and_then(|v| v.as_str()) == Some("ONLINE"))
        .filter_map(|r| r.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();

    // Agent load is this process's OWN measured totals divided by its own request count, not the
    // per-agent constant the Python route hardcoded (calls_handled 0, avg_latency_us 500).
    let (calls, nanos) = crate::telemetry_handlers::counters();
    let avg_us = if calls > 0 { (nanos as f64 / calls as f64) / 1000.0 } else { 0.0 };
    let experts: Vec<Value> = crate::mesh_handlers::mesh_nodes()
        .get("nodes")
        .and_then(|v| v.as_array())
        .map(|nodes| {
            nodes
                .iter()
                .map(|n| {
                    json!({
                        "name": n.get("name").cloned().unwrap_or(Value::Null),
                        "role": n.get("role").cloned().unwrap_or(Value::Null),
                        "domain": n.get("domain").cloned().unwrap_or(Value::Null),
                        "status": n.get("status").cloned().unwrap_or(Value::Null),
                        "calls_handled": calls,
                        "avg_latency_us": avg_us.round(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    json!({
        "status": overall(&rows),
        "services": rows,
        "services_total": t.services.len(),
        "services_online": online.len(),
        "online_services": online,
        "experts": experts,
        "active_agents": t.services.iter().filter(|s| s.get("startable").and_then(|v| v.as_bool()).unwrap_or(false)).count(),
        "agent_slots": crate::core_pinning::LOGICAL_THREADS,
        "agent_slots_basis": "measured topology (8C/16T); the real concurrency ceiling, replacing the 584 'digital-twin singleton' the Python route asserted and the reconciliation retired",
        "active_models": Value::Null,
        "active_models_basis": "the supervisor's status table carries service identity, not model identity -- the model a generator loaded lives in its argv in supervisor.services.conf",
        "sample_age_s": (wall_now_s() - t.now_s).max(0.0).round(),
        "supervisor_interval_s": t.interval_s,
        "failures_counter": "absent",
        "failures_basis": "the supervisor keeps no failure counter, so the FSM's DEGRADED branch (failures > 2) is inert here; a 0 in that slot means no counter exists, not that no failure occurred",
        "measured_basis": "the supervisor's own service table, evaluated by trm_uefi_kernel/src/self_healing_infra.rs; agent load is this process's measured request count and handler time",
        "previously_asserted": {
            "calls_handled": 0, "avg_latency_us": 500, "agent_slots": 584,
            "active_models": ["TRM_7M", "Instella_MoE", "Bonsai_27B", "Diffusion_Engine"]
        },
    })
}

/// /infra/health -- the verdict, its inputs, and the rule that maps one to the other.
pub fn health_payload() -> Value {
    let Some(t) = table() else {
        return unreadable();
    };
    let rows = fsm_rows(&t);
    let mut by_health: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for r in &rows {
        if let Some(h) = r.get("health").and_then(|v| v.as_str()) {
            *by_health.entry(h.to_string()).or_insert(0) += 1;
        }
    }
    let eligible: Vec<Value> = rows
        .iter()
        .filter(|r| r.get("restart_eligible").and_then(|v| v.as_bool()) == Some(true))
        .filter_map(|r| r.get("name").cloned())
        .collect();
    json!({
        "status": overall(&rows),
        "running": t.running,
        "running_basis": "the supervisor's own watchdog flag: false means its heal loop is paused (a /infra/stop was issued), true means it is ticking normally",
        "status_rule": "over services the supervisor is actually responsible for starting: DEGRADED if any is STOPPED (restart budget exhausted); HEALING if any is UNRESPONSIVE; DEGRADED if any is DEGRADED; else HEALTHY. Services declared with an empty argv are reported not_started and excluded -- they are not failing, they are simply not running",
        "services_total": rows.len(),
        "by_health": by_health,
        "restart_eligible": eligible,
        "services": rows,
        "sample_age_s": (wall_now_s() - t.now_s).max(0.0).round(),
        "supervisor_interval_s": t.interval_s,
        "failures_counter": "absent",
        "failures_basis": "the supervisor keeps no failure counter, so the FSM's DEGRADED branch (failures > 2) never fires; restart_eligible therefore reflects liveness and restart budget only",
    })
}

/// Write a control action to the supervisor's control file. The Rust host_supervisor reads it on its
/// next tick and consumes it, so the action is applied exactly once. Returns an acknowledgment, not a
/// claim that the action already took effect.
pub fn control_payload(action: &str, key: Option<&str>) -> Value {
    let Some(path) = crate::k3_retrieval::repo_root().map(|r| r.join(SUP_CONTROL_REL)) else {
        return json!({
            "status": "error",
            "error": "control_file_unresolvable",
            "detail": format!("cannot resolve the repo root to place {}", SUP_CONTROL_REL),
        });
    };
    let body = match key {
        Some(k) => json!({ "action": action, "key": k }),
        None => json!({ "action": action }),
    };
    match std::fs::write(&path, body.to_string()) {
        Ok(_) => json!({
            "status": "accepted",
            "action": action,
            "key": key.map(|k| json!(k)).unwrap_or(Value::Null),
            "control_file": SUP_CONTROL_REL,
            "note": "the supervisor applies this on its next tick; poll /infra/health to observe the effect",
        }),
        Err(e) => json!({
            "status": "error",
            "error": "control_write_failed",
            "detail": format!("could not write the control file: {e}"),
        }),
    }
}

/// Samples of the FSM observed by THIS process, most recent last. Deduplicated by the supervisor's own
/// checked_ms, so repeatedly polling one tick does not fill the ring with copies of itself.
static HISTORY: std::sync::Mutex<Vec<Value>> = std::sync::Mutex::new(Vec::new());
const HISTORY_CAP: usize = 240;

/// /infra/history -- what this process has observed, with the limits of that stated.
pub fn history_payload() -> Value {
    let Some(t) = table() else {
        return unreadable();
    };
    let rows = fsm_rows(&t);
    let sample = json!({
        "checked_ms": t.checked_ms,
        "observed_at_s": wall_now_s().round(),
        "status": overall(&rows),
        "services": rows
            .iter()
            .map(|r| json!({
                "key": r.get("key").cloned().unwrap_or(Value::Null),
                "health": r.get("health").cloned().unwrap_or(Value::Null),
                "code": r.get("code").cloned().unwrap_or(Value::Null),
            }))
            .collect::<Vec<Value>>(),
    });

    let mut h = match HISTORY.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let already = h
        .last()
        .and_then(|l| l.get("checked_ms"))
        .and_then(|v| v.as_u64())
        == Some(t.checked_ms as u64);
    if !already {
        h.push(sample);
        if h.len() > HISTORY_CAP {
            h.remove(0);
        }
    }
    let out = h.clone();
    json!({
        "samples": out,
        "count": h.len(),
        "capacity": HISTORY_CAP,
        "distinct_checkpoints": true,
        "basis": "Samples recorded IN THIS PROCESS each time this route is called, deduplicated by the supervisor's checked_ms. This is NOT a background poller: coverage is a function of request traffic, so a quiet dashboard yields a sparse history. The supervisor itself keeps no history -- starting a sampler here would duplicate its job.",
    })
}
