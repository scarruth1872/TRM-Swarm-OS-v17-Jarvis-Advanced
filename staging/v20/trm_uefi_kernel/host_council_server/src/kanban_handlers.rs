//! kanban_handlers.rs — Native Rust Kanban board endpoints for the dashboard.
//!
//! Option A execution (dashboard function audit, ledger block 47): the LCARS
//! dashboard's Kanban tab (tab 14) talks ONLY to this Rust server, which was
//! previously stubbing every /kanban/* route with {"status":"ok"} — rendering
//! four empty columns while 640 real cards sat in kanban_state.json.
//!
//! These handlers read/write the SAME state file the Python tooling maintains
//! (swarm_v2_artifacts/kanban_state.json, flat dict keyed by card_id), and
//! mirror the column/transition/priority semantics of swarm_v2/core/kanban_board.py
//! so the UI and the Python tools never diverge.
//!
//! Endpoints implemented:
//!   GET  /kanban/board                         -> { TODO: [..], IN_PROGRESS: [..], REVIEW: [..], DONE: [..], ARCHIVED: [..] }
//!   GET  /kanban/stats                         -> { total_cards, by_status, in_progress_count, critical_todo_count, board_health, active_worktrees, active_ports }
//!   GET  /kanban/verify                        -> { tasks: [{ id, name, status }] }  (verification panel)
//!   POST /kanban/cards                         -> create card { title, assignee, priority }
//!   POST /kanban/cards/{card_id}/move          -> { target_status } with TRANSITIONS validation

use serde_json::{json, Value};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tiny_http::{Header, Method, Request, Response};

/// Serialize access to the shared kanban state file (worker threads race on it).
pub static KANBAN_LOCK: Mutex<()> = Mutex::new(());

/// Port allocator: a monotonic counter so each IN_PROGRESS card gets a distinct port
/// number (the Python PortManager.acquire_port, reduced to an honest allocation). The
/// number is a declared allocation, not a bound socket (socket binding is a later step).
static PORT_COUNTER: AtomicU64 = AtomicU64::new(0);

fn allocate_port() -> u64 {
    20000 + PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Column order for board output.
pub const COLUMNS: [&str; 5] = ["TODO", "IN_PROGRESS", "REVIEW", "DONE", "ARCHIVED"];

/// Transition table mirrored from swarm_v2/core/kanban_board.py TRANSITIONS.
pub const TRANSITIONS: &[(&str, &[&str])] = &[
    ("TODO", &["IN_PROGRESS", "DONE"]),
    ("IN_PROGRESS", &["REVIEW", "DONE", "TODO"]),
    ("REVIEW", &["DONE", "IN_PROGRESS", "TODO"]),
    ("DONE", &["ARCHIVED", "TODO", "IN_PROGRESS"]),
    ("ARCHIVED", &["TODO"]),
];

/// Priority sort order (lower = higher). Mirrors PRIORITY_ORDER.
pub fn priority_rank(prio: &str) -> u8 {
    match prio.trim().to_lowercase().as_str() {
        "critical" | "p0" => 0,
        "high" | "p1" => 1,
        "medium" | "p2" => 2,
        "low" | "p3" => 3,
        _ => 2, // unknown -> medium, same as Python .get(..., 2)
    }
}

/// Council domain ontology (the oracle's AGENT_DOMAINS, in order). First agent with a
/// keyword substring match wins; no match -> "Devo". Mirrors trm_uefi_kernel/src/kanban_board.rs.
pub const AGENT_DOMAINS: [(&str, &[&str]); 14] = [
    ("Verify", &["test", "fuzz", "qa", "assert", "mock", "chaos", "check"]),
    ("Shield", &["security", "vuln", "qkd", "ddr", "antibody", "auth", "waf", "crypto", "protect", "scan"]),
    ("Pulse", &["gpu", "vulkan", "telemetry", "vital", "hardware", "metric", "observab", "power", "thermal", "voltage", "hvbe"]),
    ("Logic", &["logic", "prover", "math", "theorem", "formal", "k3", "kleene", "proof", "inference"]),
    ("Devo", &["ci", "pipeline", "refactor", "build", "compile", "git", "syntax", "format", "lint", "docker"]),
    ("Archi", &["architecture", "spec", "blueprint", "design", "overview", "continuum", "scaffold", "layout"]),
    ("Flow", &["mesh", "k8s", "infra", "container", "network", "route", "p2p", "worktree", "density"]),
    ("Seeker", &["research", "paper", "discovery", "science", "drug", "bio", "exome", "molecule", "reconnaissance", "genomics", "genome", "variant", "trio", "de novo", "heterozygous", "autozygosity", "mitochondrial", "x-linked", "imprinting", "methylation", "phenotype", "diagnostic"]),
    ("EventLedger", &["ledger", "journal", "consensus", "audit", "record", "pbft", "monad"]),
    ("Orchestra", &["orchestrat", "schedule", "dispatch", "multi-agent", "batch", "monitoring"]),
    ("THzGateway", &["thz", "rf", "frequency", "spectrum", "antenna", "carrier"]),
    ("Jarvis", &["memory", "turbovec", "vector", "persona", "embedding"]),
    ("Hermes", &["chat", "comm", "dialogue", "message", "external", "expert"]),
    ("Bridge", &["ffi", "ipc", "socket", "bridge", "adapter", "connector"]),
];

/// The council's PBFT domain vote, wired into kanban routing (step 1).
/// The council (`swarm_council::AGENT_MODELS`, 15 agents) is the authoritative domain table: the
/// motion is the set of council domain terms present in the task text, and the domain expert is
/// the agent with the most matching domains. Returns None when no council domain matches, so the
/// keyword ontology (`AGENT_DOMAINS`) still serves as the fallback.
pub fn council_assign_agent(title: &str, description: &str) -> Option<String> {
    let mut text = String::from(title);
    text.push(' ');
    text.push_str(description);
    let low = text.to_lowercase();

    // Motion = every council domain term that appears in the task text (deduped).
    let mut motion: Vec<&str> = Vec::new();
    for agent in swarm_council::AGENT_MODELS.iter() {
        for d in agent.domains.iter() {
            if low.contains(d) && !motion.contains(d) {
                motion.push(d);
            }
        }
    }
    if motion.is_empty() {
        return None;
    }
    // The domain expert = the agent with the most matching domains.
    let mut best: Option<(&str, usize)> = None;
    for agent in swarm_council::AGENT_MODELS.iter() {
        let ov = swarm_council::overlap(agent, &motion);
        if ov > 0 && best.map_or(true, |(_, b)| ov > b) {
            best = Some((agent.name, ov));
        }
    }
    best.map(|(name, _)| name.to_string())
}

/// The council's top-K domain experts, sorted by overlap (highest first). The first is the
/// single-expert answer; the rest are the fallback experts Orchestra reroutes to when the top
/// expert is overloaded. Empty when no council domain term matches.
pub fn council_top_experts(title: &str, description: &str, k: usize) -> Vec<(String, usize)> {
    let mut text = String::from(title);
    text.push(' ');
    text.push_str(description);
    let low = text.to_lowercase();
    let mut motion: Vec<&str> = Vec::new();
    for agent in swarm_council::AGENT_MODELS.iter() {
        for d in agent.domains.iter() {
            if low.contains(d) && !motion.contains(d) {
                motion.push(d);
            }
        }
    }
    if motion.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(String, usize)> = swarm_council::AGENT_MODELS
        .iter()
        .map(|a| (a.name.to_string(), swarm_council::overlap(a, &motion)))
        .filter(|(_, ov)| *ov > 0)
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.truncate(k);
    scored
}

/// The board's load signal for one silicon execution class (transistor region): how many active
/// cards carry a `silicon_binding` whose task_class matches. This is the voltage doctrine's load
/// axis — the transistor region, not the agent name.
fn silicon_class_load(class: &str) -> usize {
    match load_state() {
        Some(state) => state
            .as_object()
            .map(|obj| {
                obj.values()
                    .filter(|c| {
                        c.get("silicon_binding")
                            .and_then(|sb| sb.get("task_class"))
                            .and_then(|t| t.as_str())
                            == Some(class)
                            && matches!(
                                c.get("status").and_then(|s| s.as_str()).unwrap_or(""),
                                "TODO" | "IN_PROGRESS" | "REVIEW"
                            )
                    })
                    .count()
            })
            .unwrap_or(0),
        None => 0,
    }
}

/// Orchestra's load-aware assignment, transistor-region aware (the voltage doctrine). Fast path:
/// the top expert's silicon region has no active load, so it takes the task — the platform's
/// native compute is fast/accurate and hardly touches capacity outside world-sims. Only when a
/// region is saturated does Orchestra reroute to the top-K expert whose silicon region is
/// least-utilized. Council motion "load-balance by transistor region" (PROCEED).
pub fn orchestra_assign(title: &str, description: &str) -> String {
    let experts = council_top_experts(title, description, 5);
    if experts.is_empty() {
        return "Devo".to_string();
    }
    // Fast path: the top expert's silicon region is idle → keep the expert (fast, accurate, no rebalance).
    let top = &experts[0];
    let top_class = crate::hardware_binder::classify_silicon_task(title, &top.0);
    if silicon_class_load(top_class) == 0 {
        return top.0.clone();
    }
    // Saturated region (world-sim): reroute to the least-utilized silicon region among top-K.
    experts
        .iter()
        .map(|(name, _)| {
            let class = crate::hardware_binder::classify_silicon_task(title, name);
            (name.clone(), silicon_class_load(class))
        })
        .min_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)))
        .map(|(name, _)| name)
        .unwrap_or_else(|| "Devo".to_string())
}

/// The oracle's assignment: council PBFT vote first (load-balanced by Orchestra), then the
/// keyword ontology, else "Devo".
pub fn auto_assign_agent(title: &str, description: &str, tags: &[&str]) -> String {
    // 1. Council top-K experts, load-balanced by Orchestra (least-loaded expert wins).
    if !council_top_experts(title, description, 5).is_empty() {
        return orchestra_assign(title, description);
    }
    // 2. Keyword ontology (AGENT_DOMAINS) — the fallback (carries terms the council table lacks).
    let mut text = String::from(title);
    text.push(' ');
    text.push_str(description);
    for t in tags {
        text.push(' ');
        text.push_str(t);
    }
    let low = text.to_lowercase();
    for (agent, keywords) in AGENT_DOMAINS.iter() {
        for kw in keywords.iter() {
            if low.contains(kw) {
                return (*agent).to_string();
            }
        }
    }
    "Devo".to_string()
}

/// The genomics dispatch (the Rust port of council_cancer_genomics_dispatch.py).
/// For each task in the suite, the council PBFT vote assigns the domain expert (highest
/// swarm_council::overlap) and reports the verdict + the matched domains. This is the ROUTING
/// deliverable — the actual variant calling / annotation remains Python-side (no VCF parser here).
pub fn genomics_dispatch(body: &Value) -> Value {
    let tasks = body
        .get("tasks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut dispatches: Vec<Value> = Vec::new();
    for t in &tasks {
        let title = t.get("title").and_then(|x| x.as_str()).unwrap_or("");
        let desc = t.get("description").and_then(|x| x.as_str()).unwrap_or("");
        let agent = council_assign_agent(title, desc).unwrap_or_else(|| "Devo".to_string());
        let text = format!("{} {}", title, desc).to_lowercase();
        let mut motion: Vec<&str> = Vec::new();
        for a in swarm_council::AGENT_MODELS.iter() {
            for d in a.domains.iter() {
                if text.contains(d) && !motion.contains(d) {
                    motion.push(d);
                }
            }
        }
        let (verdict, _) = swarm_council::council_verdict(&swarm_council::AGENT_MODELS, &motion);
        dispatches.push(json!({
            "task": title,
            "agent": agent,
            "council_verdict": verdict.label(),
            "matched_domains": motion,
        }));
    }
    json!({
        "count": dispatches.len(),
        "dispatches": dispatches,
        "basis": "council PBFT domain vote (swarm_council::council_verdict + highest-overlap domain expert) — the Rust port of council_cancer_genomics_dispatch.py; routing is the deliverable, variant calling remains Python-side",
    })
}

/// Locate the repo root by walking up from the executable, falling back to
/// the current dir, then to an env override TRM_REPO_ROOT.
pub fn repo_root() -> PathBuf {
    if let Ok(env_root) = std::env::var("TRM_REPO_ROOT") {
        let p = PathBuf::from(env_root);
        if p.join("swarm_v2_artifacts").exists() {
            return p;
        }
    }
    // exe is at <repo>/trm_uefi_kernel/host_council_server/target/release/
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
    // Fallback: current working dir
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("swarm_v2_artifacts").exists() {
            return cwd;
        }
    }
    PathBuf::from(".")
}

/// Path to the kanban state file.
pub fn kanban_state_path() -> PathBuf {
    let root = repo_root();
    // env override for the state file itself
    if let Ok(p) = std::env::var("KANBAN_STATE_PATH") {
        return PathBuf::from(p);
    }
    root.join("swarm_v2_artifacts").join("kanban_state.json")
}

/// Load the state file into a JSON object {card_id: card}. Returns None on error.
pub fn load_state() -> Option<Value> {
    let path = kanban_state_path();
    let raw = fs::read_to_string(&path).ok()?;
    serde_json::from_str::<Value>(&raw).ok()
}

/// Write the state object back atomically (tmp + rename).
fn save_state(state: &Value) -> std::io::Result<()> {
    let path = kanban_state_path();
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(state).unwrap_or_else(|_| "{}".into());
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Board grouped by column, each column sorted by priority.
pub fn board_payload() -> Value {
    let mut board = serde_json::Map::new();
    for c in COLUMNS {
        board.insert(c.to_string(), json!([]));
    }
    if let Some(state) = load_state() {
        if let Some(obj) = state.as_object() {
            // collect per column
            let mut per_col: std::collections::HashMap<&str, Vec<&Value>> =
                std::collections::HashMap::new();
            for card in obj.values() {
                let status = card
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("TODO");
                per_col
                    .entry(if COLUMNS.contains(&status) {
                        status
                    } else {
                        "TODO"
                    })
                    .or_default()
                    .push(card);
            }
            for c in COLUMNS {
                if let Some(cards) = per_col.get(c) {
                    let mut owned: Vec<Value> = cards.iter().map(|v| (*v).clone()).collect();
                    owned.sort_by_key(|card| {
                        let p = card
                            .get("priority")
                            .and_then(|x| x.as_str())
                            .unwrap_or("medium");
                        priority_rank(p)
                    });
                    board.insert(c.to_string(), Value::Array(owned));
                }
            }
        }
    }
    Value::Object(board)
}

/// Board statistics, mirroring kanban_board.get_stats() fields the UI reads.
pub fn stats_payload() -> Value {
    let mut by_status = serde_json::Map::new();
    for c in COLUMNS {
        by_status.insert(c.to_string(), json!(0));
    }
    let mut total = 0usize;
    let mut critical_todo = 0usize;
    let mut active_worktrees = 0usize;
    let mut active_ports = 0usize;
    if let Some(state) = load_state() {
        if let Some(obj) = state.as_object() {
            total = obj.len();
            for card in obj.values() {
                let status = card
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("TODO");
                let n = by_status.get(status).and_then(|v| v.as_u64()).unwrap_or(0) + 1;
                by_status.insert(status.to_string(), json!(n));
                let prio = card
                    .get("priority")
                    .and_then(|x| x.as_str())
                    .unwrap_or("medium");
                if status == "TODO" && priority_rank(prio) == 0 {
                    critical_todo += 1;
                }
                let wt = card
                    .get("worktree_path")
                    .and_then(|x| x.as_str())
                    .unwrap_or("");
                if status == "IN_PROGRESS" && !wt.is_empty() {
                    active_worktrees += 1;
                }
                let port = card
                    .get("allocated_port")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0);
                if status == "IN_PROGRESS" && port > 0 {
                    active_ports += 1;
                }
            }
        }
    }
    let done = by_status.get("DONE").and_then(|v| v.as_u64()).unwrap_or(0) as f64;
    let review = by_status
        .get("REVIEW")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as f64;
    let in_prog = by_status
        .get("IN_PROGRESS")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as f64;
    let crit = critical_todo as f64;
    // Same health formula as the Python fallback in kanban_board.get_stats()
    let raw_score = done * 2.0 + review * 1.5 - in_prog * 0.5 - crit * 3.0;
    let health = (50.0 + raw_score * 0.05).clamp(0.0, 100.0);
    json!({
        "total_cards": total,
        "by_status": Value::Object(by_status),
        "in_progress_count": in_prog as u64,
        "critical_todo_count": critical_todo,
        "board_health": health,
        "active_worktrees": active_worktrees,
        "active_ports": active_ports,
    })
}

/// Verification payload: { tasks: [{ id, name, status }] } — the shape the
/// dashboard's card-detail panel matches against (task.id appears in title).
pub fn verify_payload() -> Value {
    let mut tasks = Vec::new();
    if let Some(state) = load_state() {
        if let Some(obj) = state.as_object() {
            for card in obj.values() {
                let title = card.get("title").and_then(|s| s.as_str()).unwrap_or("");
                let status = card
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("TODO");
                // extract bracket tag like [T01] or [task-123] as the id when present
                let id = title
                    .split('[')
                    .nth(1)
                    .and_then(|t| t.split(']').next())
                    .unwrap_or("")
                    .to_string();
                if !id.is_empty() {
                    tasks.push(json!({ "id": id, "name": title, "status": status }));
                }
            }
        }
    }
    json!({ "tasks": tasks, "total": tasks.len() })
}

/// POST /kanban/cards — create a card (mirror kanban_board.create_card).
fn create_card(body: &Value) -> Value {
    let title = body
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if title.is_empty() {
        return json!({ "success": false, "message": "Title is required" });
    }
    let mut assignee = body
        .get("assignee")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let priority = body
        .get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("medium")
        .to_string();
    let desc = body
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tags: Vec<String> = body
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    if assignee.is_empty() {
        // domain-ontology auto-assign (mirrors the Python create_card, not a fixed "Devo")
        let tag_refs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
        assignee = auto_assign_agent(&title, &desc, &tag_refs);
    }

    // generate a card id like the Python side: card-<8 hex chars>
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let card_id = format!("card-{:08x}", (nanos as u64) & 0xFFFF_FFFF);

    let now = iso_now();
    let card = json!({
        "card_id": card_id,
        "title": title,
        "description": desc,
        "status": "TODO",
        "assignee": assignee,
        "priority": priority,
        "tags": body.get("tags").cloned().unwrap_or(json!([])),
        "worktree_path": "",
        "allocated_port": 0,
        "created_at": now,
        "updated_at": now,
        "completed_at": null,
        "history": [ { "action": "created", "timestamp": now, "details": format!("Card created: {} (assigned to {})", title, assignee) } ]
    });

    let _guard = KANBAN_LOCK.lock().unwrap();
    let mut state = load_state().unwrap_or(json!({}));
    if let Some(obj) = state.as_object_mut() {
        obj.insert(card_id.clone(), card);
    }
    if save_state(&state).is_ok() {
        json!({ "success": true, "card_id": card_id })
    } else {
        json!({ "success": false, "message": "Failed to write kanban state" })
    }
}

/// POST /kanban/cards/{card_id}/move — validate + persist a transition.
fn move_card(card_id: &str, body: &Value) -> Value {
    let target = body
        .get("target_status")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !COLUMNS.contains(&target) {
        return json!({ "success": false, "message": format!("Invalid target_status: {}", target) });
    }
    let _guard = KANBAN_LOCK.lock().unwrap();
    let mut state = match load_state() {
        Some(s) => s,
        None => return json!({ "success": false, "message": "Kanban state unreadable" }),
    };
    // Take the card out of the map so we can mutate it freely.
    let mut card = match state.get_mut(card_id).cloned() {
        Some(c) => c,
        None => {
            return json!({ "success": false, "message": format!("Card {} not found", card_id) })
        }
    };
    let current = card
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("TODO")
        .to_string();
    if current == target {
        return json!({ "success": false, "message": format!("Card already in {}", target) });
    }
    // transition validation mirroring TRANSITIONS
    let allowed: &[&str] = TRANSITIONS
        .iter()
        .find(|(from, _)| *from == current.as_str())
        .map(|(_, to)| *to)
        .unwrap_or(&[]);
    if !allowed.contains(&target) {
        let allowed_str = allowed.join(", ");
        return json!({
            "success": false,
            "message": format!("Invalid transition: {} -> {}. Allowed: {}", current, target, allowed_str)
        });
    }
    let now = iso_now();
    card["status"] = json!(target);
    card["updated_at"] = json!(now);
    if target == "IN_PROGRESS" {
        // voltage doctrine: bind the task to the measured host's silicon + allocate
        // its workspace (worktree dir) and port, mirroring the Python _on_start.
        let title = card.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let assignee = card.get("assignee").and_then(|v| v.as_str()).unwrap_or("");
        card["silicon_binding"] = crate::hardware_binder::bind_task_to_silicon(title, assignee);
        let wt = format!("swarm_v2_artifacts/worktrees/{}", card_id);
        let _ = std::fs::create_dir_all(repo_root().join(&wt));
        card["worktree_path"] = json!(wt);
        card["allocated_port"] = json!(allocate_port());
    }
    if target == "DONE" {
        card["completed_at"] = json!(now);
    }
    // append history
    let mut hist = card.get("history").cloned().unwrap_or(json!([]));
    if let Some(arr) = hist.as_array_mut() {
        arr.push(json!({
            "action": "moved",
            "timestamp": now,
            "details": format!("{} -> {}", current, target)
        }));
    }
    card["history"] = hist;
    // write the mutated card back into the map, then persist
    if let Some(obj) = state.as_object_mut() {
        obj.insert(card_id.to_string(), card.clone());
    }
    if save_state(&state).is_ok() {
        json!({ "success": true, "card": card, "message": "OK" })
    } else {
        json!({ "success": false, "message": "Failed to write kanban state" })
    }
}

/// ISO-8601 local-ish timestamp (UTC, same format family as Python isoformat).
fn iso_now() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|x| x.as_secs())
        .unwrap_or(0);
    // days since epoch -> y/m/d (civil-from-days)
    let days = (d / 86400) as i64;
    let secs_of_day = d % 86400;
    let (y, m, day) = civil_from_days(days);
    let (hh, mm, ss) = (
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    );
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", y, m, day, hh, mm, ss)
}

/// Howard Hinnant's civil_from_days algorithm.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as i64, d as i64)
}

/// Read a POST body as JSON (up to 1 MB), falling back to form-urlencoded so the pure-HTML
/// Kanban form (title=...&assignee=...&priority=...) reaches the same handler the JSON caller uses.
fn read_body(request: &mut Request) -> Value {
    let mut buf = Vec::new();
    let mut limited = request.as_reader().take(1_048_576);
    let _ = limited.read_to_end(&mut buf);
    if let Ok(v) = serde_json::from_slice::<Value>(&buf) {
        if v.is_object() {
            return v;
        }
    }
    let text = String::from_utf8_lossy(&buf);
    let mut out = serde_json::Map::new();
    for kv in text.split('&') {
        if let Some((k, v)) = kv.split_once('=') {
            let mut decoded = String::new();
            let b = v.as_bytes();
            let mut i = 0;
            while i < b.len() {
                if b[i] == b'%' && i + 2 < b.len() {
                    if let Ok(hex) = u8::from_str_radix(&v[i + 1..i + 3], 16) {
                        decoded.push(hex as char);
                        i += 3;
                        continue;
                    }
                }
                decoded.push(if b[i] == b'+' { ' ' } else { b[i] as char });
                i += 1;
            }
            out.insert(k.to_string(), json!(decoded));
        }
    }
    json!(out)
}

fn ok_response(value: Value) -> Response<std::io::Cursor<Vec<u8>>> {
    let body = serde_json::to_string(&value).unwrap_or_else(|_| "{}".into());
    Response::from_string(body)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// Dispatch a /kanban* request. Returns Some(response) if handled.
pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let method = request.method().clone();
    // Strip the query string once, here: every route below compares the path, and an exact-match
    // against the full URL silently 404s as soon as a caller appends ?limit=... (the class bug —
    // /kanban/board served 702 KB while /kanban/board?limit=1 returned Not found).
    let url = url.split('?').next().unwrap_or("");

    // GET /kanban/board
    if method == Method::Get && (url == "/kanban/board" || url == "/kanban/board/") {
        let _guard = KANBAN_LOCK.lock().unwrap();
        return Some(ok_response(board_payload()));
    }
    // GET /kanban/stats
    if method == Method::Get && (url == "/kanban/stats" || url == "/kanban/stats/") {
        let _guard = KANBAN_LOCK.lock().unwrap();
        return Some(ok_response(stats_payload()));
    }
    // GET /kanban/verify
    if method == Method::Get && (url == "/kanban/verify" || url == "/kanban/verify/") {
        let _guard = KANBAN_LOCK.lock().unwrap();
        return Some(ok_response(verify_payload()));
    }
    // POST /kanban/cards
    if method == Method::Post && (url == "/kanban/cards" || url == "/kanban/cards/") {
        let body = read_body(request);
        return Some(ok_response(create_card(&body)));
    }
    // POST /kanban/cards/{card_id}/move
    if method == Method::Post {
        if let Some(rest) = url.strip_prefix("/kanban/cards/") {
            if let Some(card_id) = rest
                .strip_suffix("/move")
                .or_else(|| rest.strip_suffix("/move/"))
            {
                let body = read_body(request);
                return Some(ok_response(move_card(card_id, &body)));
            }
        }
    }
    // POST /genomics/dispatch — council PBFT vote over a task suite
    if method == Method::Post && (url == "/genomics/dispatch" || url == "/genomics/dispatch/") {
        let body = read_body(request);
        return Some(ok_response(genomics_dispatch(&body)));
    }
    // anything else under /kanban that isn't handled -> real 404
    if url.starts_with("/kanban") {
        return Some(
            Response::from_string(json!({"error": "Not found"}).to_string())
                .with_status_code(404)
                .with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                ),
        );
    }
    None
}

/// Auto-dispatch: poll HEARTBEAT.md for `- [ ]` tasks, create a kanban card for each
/// (auto-assigned via the domain ontology), mark it `- [x]`. Returns the number of cards
/// created. This is the heartbeat scheduler wired into the runtime — the "auto process"
/// that keeps the board live by turning pending work into cards.
pub fn auto_dispatch_pending() -> usize {
    let heartbeat_path = repo_root().join("HEARTBEAT.md");
    let content = match fs::read_to_string(&heartbeat_path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let mut created = 0usize;
    let mut new_content = content.clone();
    for line in content.lines() {
        let title = match line.trim_start().strip_prefix("- [ ]") {
            Some(rest) => rest.trim(),
            None => continue,
        };
        if title.is_empty() {
            continue;
        }
        let assignee = auto_assign_agent(title, "", &[]);
        let _ = create_card(&json!({ "title": title, "assignee": assignee }));
        let marked = line.replacen("- [ ]", "- [x]", 1);
        new_content = new_content.replacen(line, &marked, 1);
        created += 1;
    }
    if created > 0 {
        let _ = fs::write(&heartbeat_path, new_content);
    }
    created
}

/// The heartbeat auto-dispatch loop: every 60s, poll HEARTBEAT.md and create cards for
/// pending tasks. Wired into the kernel as a background thread.
pub fn heartbeat_auto_loop() {
    loop {
        let n = auto_dispatch_pending();
        if n > 0 {
            eprintln!("[heartbeat] auto-dispatched {} pending task(s) -> kanban", n);
        }
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

/// A concise, measured fact for the ledger from a card's task result: the title + the key measured
/// output (dock dG, k3 tally, deepdta verdict, hot-patch boundary, agent verdict). The fact is what
/// the card actually computed, not a claim the card didn't produce.
fn journal_fact(title: &str, result: &Value) -> String {
    let mode = result.get("mode").and_then(Value::as_str).unwrap_or("recall");
    let compute = result.get("compute");
    let summary = match mode {
        "docking" => compute
            .and_then(|c| c.get("result").and_then(|r| r.get("delta_g_kcal_per_mol")))
            .and_then(Value::as_f64)
            .map(|d| format!("dock dG {:.2} kcal/mol", d))
            .unwrap_or_else(|| "dock result".to_string()),
        "chain" => compute
            .and_then(|c| c.get("dock").and_then(|r| r.get("result").and_then(|x| x.get("delta_g_kcal_per_mol"))))
            .and_then(Value::as_f64)
            .map(|d| format!("recall->dock->journal dG {:.2} kcal/mol", d))
            .unwrap_or_else(|| "recall->dock->journal".to_string()),
        "humanistic" => compute
            .and_then(|c| c.get("k3_tally"))
            .map(|t| format!("k3 tally {}", t))
            .unwrap_or_else(|| "k3 tally".to_string()),
        "deepdta" => compute
            .and_then(|c| c.get("verdict").and_then(Value::as_str))
            .map(|v| v.to_string())
            .unwrap_or_else(|| "deepdta verdict".to_string()),
        "hot_patch" => compute
            .and_then(|c| c.get("status").and_then(Value::as_str))
            .map(|s| format!("hot-patch {}", s))
            .unwrap_or_else(|| "hot-patch boundary".to_string()),
        "census" => compute
            .and_then(|c| c.get("tally"))
            .map(|t| format!("census {}", t))
            .unwrap_or_else(|| "census".to_string()),
        "antibody" => compute
            .and_then(|c| c.get("paratope_contacts"))
            .and_then(Value::as_u64)
            .map(|n| format!("paratope {n} contacts"))
            .unwrap_or_else(|| "antibody score".to_string()),
        "agent_dispatch" => compute
            .and_then(|c| c.get("verdict").and_then(Value::as_str))
            .map(|v| v.to_string())
            .unwrap_or_else(|| "agent dispatch".to_string()),
        other => format!("{} output", other),
    };
    format!("{}: {}", title, summary)
}

/// Execute a card's task through the task executor: route to the native compute that exists
/// (ACMG classify, FASTQ/VCF parse) or the multi-store recall, and return the structured result
/// as a JSON string. The dispatcher lives in task_executor.rs; the loop runs it here and is not
/// time-boxed — a task takes as long as its compute needs.
fn execute_task(title: &str, description: &str) -> String {
    let result = crate::task_executor::execute(title, description);
    serde_json::to_string(&result).unwrap_or_else(|_| "{\"status\":\"error\"}".to_string())
}

/// Run the card's task and write the result back onto the card. The task executes while the card
/// is IN_PROGRESS, so the REVIEW column carries the actual work, not an empty handoff.
fn execute_and_record(card_id: &str) {
    let (title, desc) = {
        let _g = KANBAN_LOCK.lock().unwrap();
        match load_state() {
            Some(st) => st
                .get(card_id)
                .map(|c| {
                    (
                        c.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                        c.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                    )
                })
                .unwrap_or_default(),
            None => return,
        }
    };
    let result = execute_task(&title, &desc);
    // Emit the artifact + intel finding so the task leaves real output behind (not just a card
    // field), and the swarm-intel live feed reflects the work.
    let artifact = write_task_artifacts(card_id, &title, &desc, &result);
    // Auto-journal a card that produced a MEASURED output (any mode but recall). The fact is the
    // measured result; the cited file is the artifact the card wrote, so the ledger block's hash
    // re-derives from a real file. Routing-only cards are not journaled — their fact is just a
    // routing decision, already recorded in task_outputs, and sealing it would inflate the ledger.
    if let Ok(parsed) = serde_json::from_str::<Value>(&result) {
        let mode = parsed.get("mode").and_then(Value::as_str).unwrap_or("recall");
        if mode != "recall" {
            let fact = journal_fact(&title, &parsed);
            let files: Vec<String> = artifact
                .get("artifact")
                .and_then(Value::as_str)
                .map(|s| vec![s.to_string()])
                .unwrap_or_default();
            if !files.is_empty() {
                let _ = crate::crate_ops_handlers::seal_journal("kanban_task", &fact, "", &files);
            }
        }
    }
    let _g = KANBAN_LOCK.lock().unwrap();
    let mut st = match load_state() {
        Some(s) => s,
        None => return,
    };
    if let Some(obj) = st.as_object_mut() {
        if let Some(card) = obj.get_mut(card_id) {
            card["task_result"] = json!(result);
            card["task_artifact"] = artifact;
            card["task_executed_at"] = json!(iso_now());
        }
    }
    let _ = save_state(&st);
}

/// Sanitize a title into a filename-safe slug (alphanumeric + dash, lowercased, capped).
fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars().take(48) {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if c == ' ' || c == '-' || c == '_' {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "task".to_string()
    } else {
        trimmed
    }
}

/// Write a task's result to disk: an artifact JSON under swarm_v2_artifacts/task_outputs/ and a
/// research finding under swarm_v2_artifacts/research/ (which the swarm-intel live feed reads by
/// mtime). Returns the relative paths so the card records where its output lives.
fn write_task_artifacts(card_id: &str, title: &str, description: &str, result: &str) -> Value {
    let slug = slugify(title);
    let root = repo_root();
    let out_dir = root.join("swarm_v2_artifacts").join("task_outputs");
    let research_dir = root.join("swarm_v2_artifacts").join("research");
    let artifact_rel = format!("swarm_v2_artifacts/task_outputs/task_{}_{}.json", card_id, slug);
    let finding_rel = format!("swarm_v2_artifacts/research/task_{}_{}.md", card_id, slug);
    let parsed: Value = serde_json::from_str(result).unwrap_or_else(|_| json!({"raw": result}));
    let doc = json!({
        "task_id": card_id,
        "title": title,
        "description": description,
        "result": parsed,
        "executed_at": iso_now(),
    });
    let _ = fs::create_dir_all(&out_dir);
    let _ = fs::create_dir_all(&research_dir);
    let _ = fs::write(
        out_dir.join(format!("task_{}_{}.json", card_id, slug)),
        serde_json::to_string_pretty(&doc).unwrap_or_default(),
    );
    let md = format!(
        "# Research Finding: {}\n\nTask `{}` executed by the task executor.\n\n```json\n{}\n```\n",
        title, card_id, result
    );
    let _ = fs::write(research_dir.join(format!("task_{}_{}.md", card_id, slug)), md);
    json!({"artifact": artifact_rel, "finding": finding_rel})
}

/// Advance every active card one lifecycle step (TODO -> IN_PROGRESS -> REVIEW -> DONE).
/// MOTION-054: a card IN_PROGRESS has its task EXECUTED (a real on-silicon recall) before it
/// moves to REVIEW, so the board records the work done, not just a tick. REVIEW -> DONE is the
/// council sign-off, kept here as the terminal transition. Returns the number of cards advanced.
fn advance_cards() -> usize {
    // Snapshot the active card ids under the lock (so we don't hold it across the moves).
    let active: Vec<(String, String)> = {
        let _guard = KANBAN_LOCK.lock().unwrap();
        match load_state() {
            Some(state) => state
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, c)| {
                            let s = c.get("status").and_then(|x| x.as_str()).unwrap_or("");
                            if matches!(s, "TODO" | "IN_PROGRESS" | "REVIEW") {
                                Some((k.clone(), s.to_string()))
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default(),
            None => Vec::new(),
        }
    };
    let mut advanced = 0;
    for (cid, status) in active {
        if status == "IN_PROGRESS" {
            // Execute the task before handing it to REVIEW (MOTION-054).
            execute_and_record(&cid);
        }
        let next = match status.as_str() {
            "TODO" => "IN_PROGRESS",
            "IN_PROGRESS" => "REVIEW",
            "REVIEW" => "DONE",
            _ => continue,
        };
        if move_card(&cid, &json!({ "target_status": next })).get("success").and_then(|v| v.as_bool()) == Some(true) {
            advanced += 1;
        }
    }
    advanced
}

/// The agent work loop: every 30s, advance active cards one step so tasks flow through the
/// board with hardware binding. Wired into the kernel as a background thread.
pub fn agent_work_loop() {
    loop {
        let n = advance_cards();
        if n > 0 {
            eprintln!("[kanban] advanced {} active card(s) one lifecycle step", n);
        }
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
}
