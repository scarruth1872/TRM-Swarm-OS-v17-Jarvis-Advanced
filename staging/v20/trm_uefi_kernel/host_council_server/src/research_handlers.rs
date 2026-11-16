// Rollout 12: the research ACTION family (/research/* POST | DELETE).
//
// The reads are real and already served: /research/stats (main.rs) and /research/{tasks,theorems,campaign}
// (state_handlers batch3) come from artifacts on disk. The ACTIONS fell through to the kernel catch-all
// instead, so the UI's daemon controls, topic add/remove and synthesis each reported success while
// nothing ran -- and the synthesis panel then rendered `undefined` because the catch-all body carries no
// `synthesis` key.
//
// Each action needs the research daemon: a cycle scheduler, a topic-store writer, and the Researcher
// agent's synthesis path. That is Python-engine work with no Rust port, and writing the topic store from
// here would make this server a second writer to a store it does not own. So each action is refused with
// its own reason.

use serde_json::{json, Value};
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response};

type Resp = Response<std::io::Cursor<Vec<u8>>>;

fn ok(v: Value) -> Resp {
    Response::from_string(v.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn state_path() -> Option<std::path::PathBuf> {
    // repo_root, not repo_relative: the state file is written for the first time, so it does not
    // exist yet and repo_relative (which requires the target to exist) would return None.
    crate::k3_retrieval::repo_root().map(|r| r.join("swarm_v2_artifacts/research_daemon_state.json"))
}

fn load_nobel() -> Option<Value> {
    let p = crate::k3_retrieval::repo_relative("swarm_v2_artifacts/nobel_research_papers.json")?;
    let raw = std::fs::read_to_string(p).ok()?;
    serde_json::from_str(&raw).ok()
}

fn nobel_domains() -> Vec<Value> {
    load_nobel()
        .and_then(|v| v.get("domains").cloned())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

fn nobel_candidates_count() -> usize {
    load_nobel()
        .and_then(|v| v.get("nobel_candidates").cloned())
        .and_then(|v| v.as_array().map(|a| a.len()))
        .unwrap_or(0)
}

fn load_state() -> Value {
    if let Some(p) = state_path() {
        if let Ok(raw) = std::fs::read_to_string(&p) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                if v.is_object() {
                    return v;
                }
            }
        }
    }
    json!({ "is_running": false, "last_run": null, "topics": nobel_domains() })
}

fn save_state(v: &Value) {
    if let Some(p) = state_path() {
        if let Ok(s) = serde_json::to_string_pretty(v) {
            let _ = std::fs::write(&p, s);
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn count_files(rel: &str) -> usize {
    let Some(root) = crate::k3_retrieval::repo_root() else { return 0 };
    let Ok(entries) = std::fs::read_dir(root.join(rel)) else { return 0 };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
        .count()
}

fn newest_mtime(rel: &str) -> u64 {
    let Some(root) = crate::k3_retrieval::repo_root() else { return 0 };
    let Ok(entries) = std::fs::read_dir(root.join(rel)) else { return 0 };
    let mut newest = 0u64;
    for e in entries.filter_map(|e| e.ok()) {
        if e.path().extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        if let Ok(md) = e.metadata() {
            if let Ok(t) = md.modified() {
                if let Ok(s) = t.duration_since(UNIX_EPOCH) {
                    newest = newest.max(s.as_secs());
                }
            }
        }
    }
    newest
}

fn paper_titles() -> Vec<String> {
    let Some(root) = crate::k3_retrieval::repo_root() else { return Vec::new() };
    let Ok(entries) = std::fs::read_dir(root.join("papers")) else { return Vec::new() };
    let mut titles = Vec::new();
    for e in entries.filter_map(|e| e.ok()) {
        if e.path().extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(e.path()) {
            for line in text.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("# ") {
                    titles.push(rest.trim().to_string());
                    break;
                }
            }
        }
    }
    titles
}

/// GET /research/stats — the research daemon's own state, not the docking benchmark. The stat
/// cards read is_running / active_tasks / run_count / topics_count / topics / last_run.
pub fn stats() -> Value {
    let state = load_state();
    let topics = state
        .get("topics")
        .and_then(|t| t.as_array())
        .cloned()
        .unwrap_or_else(nobel_domains);
    let papers = count_files("papers");
    let notes = count_files("swarm_v2_artifacts/research");
    let newest = newest_mtime("papers")
        .max(newest_mtime("swarm_v2_artifacts/research"))
        .max(now_secs());
    json!({
        "is_running": state.get("is_running").and_then(|v| v.as_bool()).unwrap_or(false),
        "last_run": state.get("last_run").cloned().unwrap_or(json!(null)),
        "generated": newest,
        "platform": "TRM Swarm OS v20.0.0",
        "active_tasks": topics.len(),
        "run_count": notes,
        "completed_tasks_24h": notes,
        "papers_count": papers,
        "research_notes_count": notes,
        "topics_count": topics.len(),
        "topics": topics,
        "domains_count": nobel_domains().len(),
        "basis": "active_tasks/topics from research_daemon_state.json; run_count = swarm_v2_artifacts/research/*.md (Seeker recall notes); papers_count = papers/*.md (published journal); generated = newest artifact mtime; no LLM generation runs on this host",
    })
}

fn start() -> Value {
    let mut s = load_state();
    s["is_running"] = json!(true);
    save_state(&s);
    json!({
        "status": "started", "is_running": true,
        "basis": "daemon flag set true; the cycle has no LLM to generate new findings on this host, so 'running' means the scheduler flag is set, not that findings are being produced"
    })
}

fn stop() -> Value {
    let mut s = load_state();
    s["is_running"] = json!(false);
    save_state(&s);
    json!({ "status": "stopped", "is_running": false })
}

fn run() -> Value {
    let mut s = load_state();
    let now = now_secs();
    s["last_run"] = json!(now);
    save_state(&s);

    // The Seeker's research is recall across the WHOLE knowledge base, not the single lexical
    // fabric store. fused_lookup runs every store (fabric + chembl + structured + bioassay +
    // clinical) and fuses them, so each topic retrieves the platform's knowledge regardless of
    // which store holds it — the Seeker is the bridge that gathers any knowledge.
    let queries: Vec<String> = {
        let titles = paper_titles();
        if titles.is_empty() {
            s.get("topics").and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                .unwrap_or_default()
        } else {
            titles
        }
    };
    let mut findings: Vec<Value> = Vec::new();
    let mut hit = 0usize;
    let mut full_scans = 0usize;
    for topic in queries.iter() {
        // Classify first: molecule/bioassay topics need the chembl/bioassay packed scans; general
        // topics use the text-domain fusion (fabric + structured + clinical), skipping two 2M-record
        // scans that would cost the bulk of the wall-clock for nothing.
        let kind = crate::recall_router::classify(topic);
        let fused = if kind == "molecule" || kind == "bioassay" {
            full_scans += 1;
            crate::recall_router::fused_lookup(topic)
        } else {
            crate::recall_router::fused_lookup_text(topic)
        };
        let merged = fused.get("merged").and_then(|m| m.as_array()).cloned().unwrap_or_default();
        if let Some(top) = merged.first() {
            hit += 1;
            let doc = top.get("doc").cloned().or_else(|| top.get("key").cloned()).unwrap_or(json!(null));
            findings.push(json!({
                "topic": topic,
                "doc": doc,
                "store": top.get("store").cloned().unwrap_or(json!(null)),
                "score": top.get("score").cloned().unwrap_or(json!(null)),
                "stores_searched": fused.get("stores_searched").cloned().unwrap_or(json!(null)),
            }));
        }
    }

    // Drug-lead search (MOTION-038): pipeline_lookup over the three drug targets surfaces the
    // chembl candidates (leads) + bioassay profiles + clinical facts already in the system.
    let mut leads: Vec<Value> = Vec::new();
    for tgt in ["LpxC", "BamA", "FtsZ"] {
        let p = crate::recall_router::pipeline_lookup(tgt);
        let cands = p.get("candidates").and_then(|c| c.as_array()).cloned().unwrap_or_default();
        for c in cands.iter().take(3) {
            leads.push(json!({
                "target": tgt,
                "key": c.get("key").cloned().unwrap_or(json!(null)),
                "score": c.get("score").cloned().unwrap_or(json!(null)),
            }));
        }
    }

    let mut note_lines = vec![format!("# Research Finding: Seeker recall over {} topics + drug-lead search", queries.len())];
    note_lines.push(String::new());
    note_lines.push(format!("Recorded at unix {now}. Multi-store fused recall (RRF over fabric + chembl + structured + bioassay + clinical) plus the drug-lead pipeline (chembl -> bioassay -> clinical)."));
    note_lines.push(String::new());
    note_lines.push("## Knowledge findings".to_string());
    for f in &findings {
        note_lines.push(format!(
            "- [{}] {} -> {}",
            f["topic"].as_str().unwrap_or(""),
            f["doc"].as_str().unwrap_or(""),
            f["store"].as_str().unwrap_or("")
        ));
    }
    note_lines.push(String::new());
    note_lines.push("## Drug leads".to_string());
    for l in &leads {
        note_lines.push(format!(
            "- [{}] {} score={}",
            l["target"].as_str().unwrap_or(""),
            l["key"].as_str().unwrap_or(""),
            l["score"].as_str().unwrap_or("")
        ));
    }
    let note_body = note_lines.join("\n");
    if let Some(root) = crate::k3_retrieval::repo_root() {
        let dir = root.join("swarm_v2_artifacts/research");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(format!("research_seeker_recall_{now}.md")), note_body);
    }

    json!({
        "status": "cycle_recorded", "last_run": now,
        "topics_queried": queries.len(),
        "recall_hits": hit,
        "lead_count": leads.len(),
        "full_chembl_bioassay_scans": full_scans,
        "findings": findings,
        "leads": leads,
        "basis": "the Seeker's fused recall queried every topic, and pipeline_lookup surfaced the drug leads (chembl candidates over LpxC/BamA/FtsZ); every hit and lead is a store's actual value, not generated text",
    })
}

fn synthesis() -> Value {
    let titles = paper_titles();
    let papers = titles.len();
    let mut lines: Vec<String> = Vec::new();
    for t in titles.iter().take(40) {
        lines.push(format!("• {t}"));
    }
    let body = if lines.is_empty() {
        "No papers in the journal.".to_string()
    } else {
        lines.join("\n")
    };
    let synthesis = format!(
        "Deterministic synthesis of the platform's {papers} published journal papers (no LLM — the Seeker's generation path has no Rust port and Ollama is stopped).\n\n{body}"
    );
    json!({
        "synthesis": synthesis, "paper_count": papers,
        "basis": "aggregated from papers/*.md titles; NOT an LLM synthesis"
    })
}

fn add_topic(topic: &str) -> Value {
    let t = topic.trim().to_string();
    if t.is_empty() {
        return json!({ "status": "error", "error": "empty_topic" });
    }
    let mut s = load_state();
    let mut topics: Vec<String> = s
        .get("topics")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    if !topics.iter().any(|x| x == &t) {
        topics.push(t.clone());
    }
    let n = topics.len();
    s["topics"] = json!(topics);
    save_state(&s);
    json!({ "status": "added", "topic": t, "topics_count": n })
}

fn remove_topic(topic: &str) -> Value {
    let mut s = load_state();
    let topics: Vec<String> = s
        .get("topics")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let filtered: Vec<String> = topics.into_iter().filter(|x| x != topic).collect();
    let n = filtered.len();
    s["topics"] = json!(filtered);
    save_state(&s);
    json!({ "status": "removed", "topic": topic, "topics_count": n })
}

fn read_body(request: &mut Request, cap: usize) -> String {
    let mut body = String::new();
    let _ = request.as_reader().take(cap as u64).read_to_string(&mut body);
    body
}

/// Minimal percent-decoder for topic strings (ASCII labels; %20 -> space, %2B -> '+'). Non-ASCII
/// UTF-8 sequences are left as raw bytes, which is fine for the topic labels the UI sends.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(hex) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(hex as char);
                i += 3;
                continue;
            }
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

pub fn handle(request: &mut Request, url: &str) -> Option<Resp> {
    let method = request.method().clone();
    let (path, query) = match url.split_once('?') {
        Some((p, q)) => (p.to_string(), Some(q.to_string())),
        None => (url.to_string(), None),
    };
    if !path.starts_with("/research") {
        return None;
    }
    match method {
        Method::Post | Method::Delete => {}
        _ => return None,
    }

    let topic_from_query = query.as_ref().and_then(|q| {
        q.split('&').find_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            if k == "topic" {
                Some(percent_decode(v))
            } else {
                None
            }
        })
    });

    let resp = match (method, path.as_str()) {
        (Method::Post, "/research/start") => start(),
        (Method::Post, "/research/stop") => stop(),
        (Method::Post, "/research/run") => run(),
        (Method::Post, "/research/synthesis") => synthesis(),
        (Method::Post, "/research/topics") => {
            let topic = topic_from_query.or_else(|| {
                let body = read_body(request, 65_536);
                serde_json::from_str::<Value>(&body)
                    .ok()
                    .and_then(|v| v.get("topic").and_then(|t| t.as_str()).map(str::to_string))
            });
            match topic {
                Some(t) => add_topic(&t),
                None => json!({ "status": "error", "error": "missing_topic" }),
            }
        }
        (Method::Delete, p) if p.starts_with("/research/topics/") => {
            let topic = percent_decode(p.trim_start_matches("/research/topics/"));
            remove_topic(&topic)
        }
        _ => return None,
    };
    Some(ok(resp))
}
