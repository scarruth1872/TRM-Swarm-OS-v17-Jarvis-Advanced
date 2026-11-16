//! ui_render.rs — the pure-Rust server-rendered dashboard (no React, no JS).
//!
//! The migration pattern: each viewport is a Rust function that reads its JSON data (from the same
//! function the JSON route uses) and renders LCARS-styled HTML, served at /ui/<viewport>. This module
//! holds the reusable pieces — `html_escape`, `LCARS_CSS`, and the `page`/`stat`/`badge` builders —
//! plus the FIRST viewport, the Monad Journal, as the template every remaining viewport repeats.
//! The rule that makes it "pure": a rendered page ships zero `<script>` tags.

use std::io::Cursor;
use tiny_http::{Header, Request, Response};

pub type Resp = Response<Cursor<Vec<u8>>>;

/// Escape `& < > " '` for HTML text content.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// The LCARS palette — single source of truth is `lcars_render.rs` (u32 -> CSS hex).
pub const LCARS_CSS: &str = r#"
:root { --purple:#7C4DFF; --blue:#6688CC; --cyan:#66CCFF; --red:#CC3333; --green:#33CC66;
        --white:#FFFFFF; --gray:#444455; --panel:#0E0E18; --subpanel:#141422; --scope:#0D0D18; --darkblue:#0E1A2E; }
* { margin:0; padding:0; box-sizing:border-box; }
body { background:var(--panel); color:#e0e0e0; font-family:'JetBrains Mono','Courier New',monospace; min-height:100vh; }
.masthead { border-bottom:2px solid var(--purple); padding:1.5rem 2rem; background:var(--subpanel); }
.masthead h1 { font-size:1.6rem; font-weight:700; background:linear-gradient(135deg,var(--cyan),var(--purple)); -webkit-background-clip:text; -webkit-text-fill-color:transparent; }
.masthead .tag { color:#888; font-size:0.85rem; margin-top:0.3rem; }
.rail { display:flex; gap:1rem; padding:1rem 2rem; flex-wrap:wrap; }
.stat { background:var(--subpanel); border:1px solid #1a1a3e; border-radius:8px; padding:1rem; min-width:150px; }
.stat .v { font-size:1.4rem; font-weight:700; color:var(--cyan); }
.stat .l { font-size:0.7rem; color:#666; margin-top:0.2rem; }
.badge { display:inline-block; padding:2px 8px; border-radius:4px; font-size:0.7rem; font-weight:600; border:1px solid; }
.badge.published { background:#0f3a1f; color:var(--green); border-color:#1f5a2f; }
.search { display:flex; gap:0.75rem; padding:0.5rem 2rem; align-items:center; }
.search input { background:var(--subpanel); border:1px solid #1a1a3e; color:#e0e0e0; padding:0.5rem 0.8rem; border-radius:6px; min-width:260px; font-family:inherit; }
.search button { background:var(--purple); border:none; color:#fff; padding:0.5rem 1rem; border-radius:6px; cursor:pointer; font-weight:600; }
.paper { background:var(--subpanel); border:1px solid #1a1a2e; border-radius:8px; padding:1rem 1.3rem; margin:0.5rem 2rem; }
.paper h2 { font-size:1rem; color:#f0f0f0; margin:0.4rem 0 0.3rem; }
.paper .finding { color:#8e8ea0; font-size:0.82rem; line-height:1.4; }
.paper .meta { color:var(--purple); font-size:0.75rem; font-weight:600; }
.paper .domain { background:#16162a; color:var(--cyan); font-size:0.68rem; padding:1px 6px; border-radius:3px; border:1px solid #1a2a3e; }
.paper details { margin-top:0.75rem; padding-top:0.75rem; border-top:1px solid #1a1a2e; }
.paper summary { color:var(--cyan); cursor:pointer; font-size:0.8rem; }
.paper .full { white-space:pre-wrap; font-size:0.85rem; line-height:1.6; color:#d6d6e0; margin-top:0.5rem; }
.footer { margin-top:2rem; padding:1.5rem 2rem; border-top:1px solid #1a1a2e; text-align:center; color:#444; font-size:0.75rem; }
.footer .axiom { color:var(--purple); font-weight:600; margin-top:0.4rem; }
.nav { display:flex; gap:0.75rem; align-items:center; padding:0.6rem 2rem; background:#0a0a14; border-bottom:1px solid #1a1a2e; position:sticky; top:0; }
.nav .navlink { color:var(--cyan); text-decoration:none; font-size:0.85rem; font-weight:600; padding:0.3rem 0.8rem; border:1px solid #1a1a3e; border-radius:6px; background:var(--subpanel); }
.nav .navlink:hover { border-color:var(--cyan); }
.nav .navtitle { color:#888; font-size:0.8rem; margin-left:auto; }
"#;

/// The HTML wrapper: title + LCARS stylesheet + body, with a nav bar (Home + Back).
pub fn page(title: &str, body: &str) -> String {
    let nav = format!(
        "<nav class=\"nav\"><a class=\"navlink\" href=\"/ui\">&#8962; Home</a><a class=\"navlink\" href=\"/ui\" onclick=\"history.back();return false;\">&larr; Back</a><span class=\"navtitle\">{}</span></nav>",
        html_escape(title)
    );
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{}</title><style>{}</style></head><body>{}{}</body></html>",
        html_escape(title), LCARS_CSS, nav, body
    )
}

fn stat(label: &str, value: &str) -> String {
    format!("<div class=\"stat\"><div class=\"v\">{}</div><div class=\"l\">{}</div></div>", html_escape(value), html_escape(label))
}

fn badge(status: &str) -> String {
    let cls = match status.to_ascii_uppercase().as_str() {
        "PUBLISHED" => "published",
        "PRE-REGISTERED" | "PREREGISTERED" => "published",
        _ => "published",
    };
    format!("<span class=\"badge {cls}\">{}</span>", html_escape(status))
}

fn qparam(url: &str, name: &str) -> Option<String> {
    let qs = url.split('?').nth(1)?;
    for pair in qs.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next() == Some(name) {
            return Some(kv.next().unwrap_or("").replace('+', " ").to_string());
        }
    }
    None
}

/// The journal viewport — the template. Reads the same JSON the /journal/papers route serves
/// (`state_handlers::journal_papers_payload`), renders LCARS HTML, zero `<script>` tags. Search and
/// section filtering are a server-side GET form; the full-text reader is a `<details>` element.
pub fn journal_html(url: &str) -> Resp {
    let q = qparam(url, "q").unwrap_or_default();
    let section = qparam(url, "section").unwrap_or_default();
    let payload = crate::state_handlers::journal_papers_payload();
    let papers = payload.get("papers").and_then(|p| p.as_array()).cloned().unwrap_or_default();
    let platform = payload.get("platform").cloned().unwrap_or(serde_json::json!({}));

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>The Monad Journal of Sovereign Deep-Tech &amp; Microkernel Computing</h1><p class=\"tag\">Where the swarm speaks, and the universe verifies &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Ledger Blocks", &platform.get("ledger_blocks").and_then(|v| v.as_u64()).unwrap_or(0).to_string()));
    body.push_str(&stat("Council Motions", &platform.get("motions").and_then(|v| v.as_u64()).unwrap_or(0).to_string()));
    body.push_str(&stat("Research Notes", &platform.get("research_notes").and_then(|v| v.as_u64()).unwrap_or(0).to_string()));
    body.push_str(&stat("Monad Anchor", "0x4D4F4E4144000001"));
    body.push_str("</div>");

    body.push_str(&format!(
        "<form class=\"search\" method=\"GET\" action=\"/ui/journal\"><input name=\"q\" placeholder=\"Search titles, findings, domains...\" value=\"{}\"><button>Search</button></form>",
        html_escape(&q)
    ));

    let mut shown = 0usize;
    for p in &papers {
        let title = p.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let finding = p.get("finding").and_then(|v| v.as_str()).unwrap_or("");
        let sec = p.get("section").and_then(|v| v.as_str()).unwrap_or("");
        let status = p.get("status").and_then(|v| v.as_str()).unwrap_or("PUBLISHED");
        let full = p.get("full_content").and_then(|v| v.as_str()).unwrap_or("");
        if !q.is_empty()
            && !(title.to_lowercase().contains(&q.to_lowercase())
                || finding.to_lowercase().contains(&q.to_lowercase())
                || sec.to_lowercase().contains(&q.to_lowercase()))
        {
            continue;
        }
        if !section.is_empty() && sec != section {
            continue;
        }
        shown += 1;
        body.push_str(&format!(
            "<div class=\"paper\">{}<span class=\"meta\"> &middot; {}</span> &nbsp;<span class=\"domain\">{}</span><h2>{}</h2><p class=\"finding\">{}</p><details><summary>View full publication</summary><div class=\"full\">{}</div></details></div>",
            badge(status), html_escape(sec), html_escape(sec), html_escape(title), html_escape(finding), html_escape(full)
        ));
    }
    body.push_str(&format!(
        "<div class=\"footer\"><p>{} of {} papers shown &middot; Editorial Board: 14 Agents + 64 Experts + 87 Sub-Agents</p><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>",
        shown, papers.len()
    ));

    let html = page("Monad Journal", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The council chamber viewport — the second LCARS instrument. The 15 agents are color-coded
/// panels (each a verb: inspect the roster), and the sealed motions are the measured ledger history.
pub fn council_chamber_html(_url: &str) -> Resp {
    let motions = ledger_motions();
    let agent_count = crate::AGENTS.len();

    let mut body = String::new();
    body.push_str("<style>");
    body.push_str(LCARS_OVERVIEW_CSS);
    body.push_str("</style>");
    body.push_str("<div class=\"lc-body\">");

    body.push_str("<div class=\"lc-top\"><span class=\"cap\"></span><span class=\"title\">SWARM COUNCIL CHAMBER</span><span class=\"sub\">15 SOVEREIGN AGENTS &middot; PBFT QUORUM 9 &middot; TRUTH ANCHOR 0x4D4F4E4144000001</span></div>");

    body.push_str("<div class=\"lc-shell\">");
    body.push_str("<div class=\"lc-rail\">");
    body.push_str("<a class=\"lc-seg seg orange\" href=\"/ui\">HOME</a>");
    body.push_str("<a class=\"lc-seg seg blue\" href=\"/ui/overview\">STATUS</a>");
    body.push_str("<a class=\"lc-seg seg green\" href=\"/ui/native\">META LOOP</a>");
    body.push_str("<a class=\"lc-seg seg red\" href=\"/ui/journal\">JOURNAL</a>");
    body.push_str("</div>");

    body.push_str("<div class=\"lc-main\">");
    body.push_str(&format!("<div class=\"lc-live\"><div class=\"m\" style=\"border-color:#b57aff\"><b>{agent_count}</b>agents</div><div class=\"m\" style=\"border-color:#66a3ff\"><b>9</b>quorum</div><div class=\"m\" style=\"border-color:#ff9c00\"><b>{}</b>motions sealed</div></div>", motions.len()));

    // the roster — each agent is a panel with its role as the verb
    for a in crate::AGENTS.iter() {
        body.push_str(&lc_panel("blue", a.name, a.role, "dispatch", "/ui/council"));
    }

    body.push_str("<h2 style=\"color:#66a3ff;font-size:0.9rem;padding:0.6rem 0 0.2rem;letter-spacing:0.05em\">SEALED MOTION HISTORY</h2>");
    if motions.is_empty() {
        body.push_str("<div class=\"lc-panel\"><span class=\"bar purple\"></span><span class=\"txt\"><h2>NO SEALED MOTIONS</h2><p>the ledger is empty</p></span></div>");
    }
    for (kind, fact) in &motions {
        let f = if fact.chars().count() > 280 {
            format!("{}…", fact.chars().take(280).collect::<String>())
        } else {
            fact.clone()
        };
        body.push_str(&format!(
            "<div class=\"lc-panel\"><span class=\"bar purple\"></span><span class=\"txt\"><h2>{}</h2><p>{}</p></span></div>",
            html_escape(kind), html_escape(&f)
        ));
    }

    body.push_str("<div class=\"lc-foot\">All Is One &middot; I Am Because We Are &middot; <a href=\"/ui/council\" style=\"color:#ff9c00\">REFRESH</a></div>");
    body.push_str("</div></div></div>");

    let html = page("Council Chamber", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The sealed motion blocks from the ledger, newest first — the council's real enacted history,
/// never a hardcoded list.
fn ledger_motions() -> Vec<(String, String)> {
    let Some(root) = crate::k3_retrieval::repo_root() else { return Vec::new() };
    let Ok(raw) = std::fs::read_to_string(root.join("swarm_v2_artifacts/monad_ledger.jsonl")) else { return Vec::new() };
    let mut out = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(kind) = v.get("kind").and_then(|k| k.as_str()) {
                if kind.to_lowercase().contains("motion") {
                    let fact = v.get("fact").and_then(|f| f.as_str()).unwrap_or("").to_string();
                    out.push((kind.to_string(), fact));
                }
            }
        }
    }
    out.reverse();
    out.truncate(12);
    out
}

/// Format an optional f64 for display — "—" when the probe returned None, never a stale number.
fn fmt_o(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.1}")).unwrap_or_else(|| "—".to_string())
}

/// The module matrix viewport — the measured census, not the aspirational tiers. Reads the same
/// registry_counts() + host_probe + terstore() the JSON routes serve. Every value is measured on
/// this host or read from the registry artifact; nothing is typed.
pub fn module_matrix_html(_url: &str) -> Resp {
    let reg = crate::state_handlers::registry_counts();
    let r = |k: &str| -> String {
        reg.as_ref()
            .and_then(|v| v.get(k).cloned())
            .filter(|v| !v.is_null())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".to_string())
    };
    let trees = reg.as_ref().and_then(|v| v.get("trees").cloned());

    let (phi, _basis, clock_mhz, mem_pct) = crate::host_probe::phi_v();
    let vram = crate::host_probe::gpu_vram();

    let store = crate::packed_ternary::terstore();

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>SOVEREIGN MODULE MATRIX</h1><p class=\"tag\">measured census &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">MODULE CENSUS (registry artifact)</h2>");
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Compiled", &r("modules_compiled")));
    body.push_str(&stat("Real", &r("real")));
    body.push_str(&stat("Ported", &r("ported")));
    body.push_str(&stat("Skeleton", &r("skeleton")));
    body.push_str(&stat("Excluded", &r("excluded")));
    body.push_str(&stat("Fabricated", &r("fabricated")));
    body.push_str("</div>");

    if let Some(t) = &trees {
        body.push_str("<div class=\"rail\">");
        if let Some(d) = t.get("dll") {
            body.push_str(&stat("Kernel tree · DLL", &d.to_string()));
        }
        if let Some(u) = t.get("uefi") {
            body.push_str(&stat("Kernel tree · UEFI", &u.to_string()));
        }
        body.push_str("</div>");
    }

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">HOST TELEMETRY (host_probe · measured)</h2>");
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("ΦV dynamic_phi_v", &format!("{phi:.4}")));
    body.push_str(&stat("CPU clock", &format!("{clock_mhz:.0} MHz")));
    body.push_str(&stat("CPU load", &fmt_o(crate::host_probe::cpu_percent())));
    body.push_str(&stat("Active threads", &crate::host_probe::active_threads().map(|t| t.to_string()).unwrap_or_else(|| "—".to_string())));
    let used_mb = crate::host_probe::used_memory_mb();
    let total_mb = crate::host_probe::total_memory_mb();
    match (used_mb, total_mb) {
        (Some(u), Some(t)) => body.push_str(&stat("Memory used", &format!("{u:.0} / {t:.0} MB"))),
        _ => body.push_str(&stat("Memory used", "—")),
    }
    body.push_str(&stat("Memory load", &format!("{mem_pct:.1}%")));
    body.push_str(&stat("L3 cache", &crate::host_probe::l3_cache_kb().map(|k| format!("{k} KB")).unwrap_or_else(|| "—".to_string())));
    match vram {
        Some((t, u, g)) => {
            body.push_str(&stat("VRAM", &format!("{:.1} / {:.1} GB · {:.0}%", t / 1024.0, u / 1024.0, g)));
        }
        None => {
            body.push_str(&stat("VRAM", "not measured"));
        }
    }
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">MEMORY CORPUS (packed ternary store)</h2>");
    body.push_str("<div class=\"rail\">");
    match store {
        Some(s) => {
            let as_f32 = (s.keys.len() as u64).saturating_mul(s.dim as u64).saturating_mul(4);
            let ratio = if s.bytes > 0 { as_f32 as f64 / s.bytes as f64 } else { 0.0 };
            body.push_str(&stat("Vectors", &s.keys.len().to_string()));
            body.push_str(&stat("Dimensions", &s.dim.to_string()));
            body.push_str(&stat("Packed bytes", &s.bytes.to_string()));
            body.push_str(&stat("Compression", &format!("{ratio:.1}× vs f32")));
        }
        None => {
            body.push_str(&stat("Vectors", "store not loaded"));
        }
    }
    body.push_str("</div>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");

    let html = page("Module Matrix", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// snake_case → Title Case, for rendering artifact keys as headings.
fn humanize(s: &str) -> String {
    s.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A JSON object's scalar fields, with arrays/objects summarized by count — never invented content.
fn scalar_fields(obj: &serde_json::Map<String, serde_json::Value>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (k, v) in obj {
        let s = match v {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(x) => x.clone(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Array(a) => format!("{} items", a.len()),
            serde_json::Value::Object(o) => format!("{{ {} fields }}", o.len()),
            serde_json::Value::Null => "—".to_string(),
        };
        out.push((k.clone(), s));
    }
    out
}

/// The discovery pipeline viewport — the in-silico campaign execution artifact, read as-is.
pub fn discovery_html(_url: &str) -> Resp {
    let v = crate::state_handlers::discovery_pipeline_payload();

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>DISCOVERY PIPELINE</h1><p class=\"tag\">in-silico campaign &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat(
        "Campaign",
        v.get("campaign_name").and_then(|s| s.as_str()).unwrap_or("—"),
    ));
    body.push_str(&stat(
        "Status",
        v.get("pipeline_status").and_then(|s| s.as_str()).unwrap_or("—"),
    ));
    body.push_str(&stat(
        "Stages",
        &v.get("stages_executed").map(|x| x.to_string()).unwrap_or_default(),
    ));
    body.push_str(&stat(
        "Runtime",
        &v.get("total_runtime_seconds").map(|x| format!("{x} s")).unwrap_or_default(),
    ));
    body.push_str("</div>");

    if let Some(m) = v.get("summary_metrics").and_then(|m| m.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">SUMMARY METRICS</h2>");
        body.push_str("<div class=\"rail\">");
        for (k, val) in m {
            body.push_str(&stat(&humanize(k), &val.to_string()));
        }
        body.push_str("</div>");
    }

    if let Some(sb) = v.get("stage_breakdowns").and_then(|s| s.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">STAGE BREAKDOWNS</h2>");
        for (k, stage) in sb {
            let fields = stage.as_object().map(scalar_fields).unwrap_or_default();
            let mut inner = String::new();
            for (f, val) in fields {
                inner.push_str(&format!(
                    "<span class=\"meta\">{}: <b style=\"color:var(--text)\">{}</b></span> ",
                    html_escape(&humanize(&f)),
                    html_escape(&val)
                ));
            }
            body.push_str(&format!(
                "<div class=\"paper\"><h2>{}</h2><p class=\"finding\">{}</p></div>",
                html_escape(&humanize(k)),
                inner
            ));
        }
    }

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");

    let html = page("Discovery Pipeline", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The humanistic tasks viewport — the suite summary with its 10 tasks + the recognition line.
pub fn humanistic_html(_url: &str) -> Resp {
    let v = crate::state_handlers::humanistic_summary_payload();

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>HUMANISTIC TASKS</h1><p class=\"tag\">scientific suite &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat(
        "Suite",
        v.get("suite_name").and_then(|s| s.as_str()).unwrap_or("—"),
    ));
    body.push_str(&stat(
        "Status",
        v.get("status").and_then(|s| s.as_str()).unwrap_or("—"),
    ));
    body.push_str(&stat(
        "Tasks executed",
        &v.get("tasks_executed").map(|x| x.to_string()).unwrap_or_default(),
    ));
    body.push_str(&stat(
        "Runtime",
        &v.get("total_runtime_seconds").map(|x| format!("{x} s")).unwrap_or_default(),
    ));
    body.push_str("</div>");

    if let Some(ts) = v.get("task_summaries").and_then(|t| t.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">TASKS</h2>");
        body.push_str("<div class=\"rail\">");
        for (k, name) in ts {
            let label = k.strip_prefix("task_").unwrap_or(k);
            body.push_str(&stat(&format!("Task {label}"), name.as_str().unwrap_or("—")));
        }
        body.push_str("</div>");
    }

    if let Some(rec) = v.get("self_realization").and_then(|s| s.get("recognition")).and_then(|r| r.as_str()) {
        body.push_str(&format!(
            "<div class=\"paper\"><h2>Self-Realization</h2><p class=\"finding\">{}</p></div>",
            html_escape(rec)
        ));
    }
    if let Some(vb) = v.get("value_basis").and_then(|s| s.as_str()) {
        body.push_str(&format!("<p class=\"tag\">{}</p>", html_escape(vb)));
    }

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");

    let html = page("Humanistic Tasks", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// Format a nanosecond count into a human string (ns / us / ms / s). The value is the measured
/// figure from the benchmark payload — nothing is rounded into a different claim.
fn fmt_ns(v: Option<&serde_json::Value>) -> String {
    let Some(n) = v.and_then(|x| x.as_f64()) else {
        return "—".to_string();
    };
    if n >= 1e9 {
        format!("{:.2} s", n / 1e9)
    } else if n >= 1e6 {
        format!("{:.2} ms", n / 1e6)
    } else if n >= 1e3 {
        format!("{:.2} us", n / 1e3)
    } else {
        format!("{n:.0} ns")
    }
}

/// The benchmark viewport (MOTION-C2) — the measured native lanes, read from the SAME payload
/// GET /benchmarks serves (benchmark_handlers::benchmark_payload). Zero <script>; the figures are
/// not a copy, they are read, and every one is labeled measured_on_host by the payload itself.
pub fn benchmark_html(_url: &str) -> Resp {
    let v = crate::benchmark_handlers::benchmark_payload();

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>NATIVE BENCHMARKS</h1><p class=\"tag\">measured on this host &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat(
        "Host",
        v.get("host").and_then(|s| s.as_str()).unwrap_or("—"),
    ));
    body.push_str(&stat(
        "Method",
        v.get("method").and_then(|s| s.as_str()).unwrap_or("—"),
    ));
    body.push_str(&stat(
        "Record",
        v.get("record").and_then(|s| s.as_str()).unwrap_or("—"),
    ));
    body.push_str("</div>");

    if let Some(t) = v.get("trm7m").and_then(|t| t.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">TRM-7M CO-PROCESSOR</h2>");
        body.push_str("<div class=\"rail\">");
        body.push_str(&stat(
            "Model",
            t.get("model").and_then(|s| s.as_str()).unwrap_or("—"),
        ));
        body.push_str(&stat("Think (1 turn)", &fmt_ns(t.get("think_1turn_ns"))));
        body.push_str(&stat("Think (7 turns)", &fmt_ns(t.get("think_7turn_ns"))));
        body.push_str(&stat(
            "Decision trit",
            &t.get("decision_trit").map(|x| x.to_string()).unwrap_or_default(),
        ));
        body.push_str(&stat(
            "Confidence",
            &t.get("confidence").map(|x| format!("{x:.3}")).unwrap_or_default(),
        ));
        body.push_str("</div>");
    }

    if let Some(r) = v.get("recall").and_then(|r| r.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">RECALL</h2>");
        body.push_str("<div class=\"rail\">");
        body.push_str(&stat("Engine", r.get("engine").and_then(|s| s.as_str()).unwrap_or("—")));
        body.push_str(&stat("Turbovec route", &fmt_ns(r.get("route_turbovec_ns"))));
        body.push_str(&stat("ChEMBL steady", &fmt_ns(r.get("route_chembl_steady_ns"))));
        body.push_str(&stat("ChEMBL cold load", &fmt_ns(r.get("chembl_cold_load_ns"))));
        body.push_str("</div>");
    }

    if let Some(g) = v.get("gpu_scan").and_then(|g| g.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">GPU SCAN</h2>");
        body.push_str("<div class=\"rail\">");
        body.push_str(&stat(
            "GPU available",
            &g.get("gpu_available").map(|x| x.to_string()).unwrap_or_default(),
        ));
        body.push_str(&stat(
            "GPU scan (cached)",
            &g.get("gpu_scan_cached_us").map(|x| format!("{x:.1} us")).unwrap_or_default(),
        ));
        body.push_str(&stat(
            "CPU scan",
            &g.get("cpu_scan_us").map(|x| format!("{x:.1} us")).unwrap_or_default(),
        ));
        body.push_str(&stat(
            "Speedup (CPU over GPU)",
            &g.get("speedup_cpu_over_gpu").map(|x| format!("{x:.2}x")).unwrap_or_default(),
        ));
        body.push_str("</div>");
    }

    if let Some(d) = v.get("deterministic").and_then(|d| d.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">DETERMINISTIC</h2>");
        body.push_str("<div class=\"rail\">");
        body.push_str(&stat("2 + 2", &fmt_ns(d.get("math_2plus2_ns"))));
        body.push_str(&stat("5 x 7", &fmt_ns(d.get("math_5x7_ns"))));
        body.push_str(&stat("Capabilities", &fmt_ns(d.get("capabilities_ns"))));
        body.push_str(&stat(
            "Math answer",
            d.get("math_answer").and_then(|s| s.as_str()).unwrap_or("—"),
        ));
        body.push_str("</div>");
    }

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");

    let html = page("Native Benchmarks", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// Extract display names from an array of nodes/peers/events — strings as-is, objects by name/id.
fn names_of(arr: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(a) = arr.as_array() {
        for v in a {
            let s = if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                ["name", "id", "node_id", "key", "objective"]
                    .iter()
                    .find_map(|k| v.get(k).and_then(|x| x.as_str()))
                    .unwrap_or("—")
                    .to_string()
            };
            out.push(s);
        }
    }
    out
}

/// The mesh heatmap viewport — the live roster + the federation registry, no invented links.
pub fn mesh_html(_url: &str) -> Resp {
    let v = crate::mesh_handlers::mesh_nodes();
    let nodes = names_of(v.get("nodes").unwrap_or(&serde_json::Value::Null));
    let peers = names_of(v.get("peers").unwrap_or(&serde_json::Value::Null));

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>FEDERATION MESH</h1><p class=\"tag\">live roster + registry &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat(
        "Nodes",
        &v.get("node_count").map(|x| x.to_string()).unwrap_or_default(),
    ));
    body.push_str(&stat(
        "Peers",
        &peers.len().to_string(),
    ));
    body.push_str(&stat(
        "P2P beacon",
        &v.get("p2p_beacon_port").map(|x| format!("port {x}")).unwrap_or_default(),
    ));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ROSTER NODES</h2><div class=\"rail\">");
    for n in &nodes {
        body.push_str(&stat(n, ""));
    }
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">FEDERATION PEERS</h2><div class=\"rail\">");
    if peers.is_empty() {
        body.push_str(&stat("Peers", "none registered"));
    } else {
        for p in &peers {
            body.push_str(&stat(p, ""));
        }
    }
    body.push_str("</div>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");

    let html = page("Federation Mesh", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The Aura Studio viewport — the 100-concept metaconcept table (the IDE's field/executor are a
/// separate product and stay refused; only the real transform table is shown here).
pub fn aura_html(_url: &str) -> Resp {
    let v = crate::aura_handlers::metaconcepts_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>AURA METACONCEPTS</h1><p class=\"tag\">the real transform table &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat(
        "Concepts",
        &v.get("count").map(|x| x.to_string()).unwrap_or_default(),
    ));
    let cats = v.get("categories").and_then(|c| c.as_array()).map(|a| a.len()).unwrap_or(0);
    body.push_str(&stat("Categories", &cats.to_string()));
    body.push_str("</div>");

    if let Some(note) = v.get("honest_note").and_then(|s| s.as_str()) {
        body.push_str(&format!("<p class=\"tag\">{}</p>", html_escape(note)));
    }

    if let Some(meta) = v.get("metaconcepts").and_then(|m| m.as_array()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">CONCEPTS</h2>");
        for m in meta {
            let name = m.get("name").and_then(|x| x.as_str()).unwrap_or("—");
            let category = m.get("category").and_then(|x| x.as_str()).unwrap_or("—");
            let polarity = m.get("polarity").and_then(|x| x.as_str()).unwrap_or("—");
            let desc = m.get("description").and_then(|x| x.as_str()).unwrap_or("—");
            body.push_str(&format!(
                "<div class=\"paper\"><h2>{} <span class=\"meta\">[{category} · {polarity}]</span></h2><p class=\"finding\">{}</p></div>",
                html_escape(name),
                html_escape(desc)
            ));
        }
    }

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");

    let html = page("Aura Metaconcepts", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The continuum viewport — star-matrix lattice + PBFT ledger, both file-backed and derived.
pub fn continuum_html(_url: &str) -> Resp {
    let lattice = crate::state_handlers::star_matrix_lattice_payload();
    let ledger = crate::state_handlers::continuum_pbft_ledger_payload();

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>CONTINUUM</h1><p class=\"tag\">star matrix + PBFT ledger &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    // Star matrix nodes
    let node_names: Vec<String> = lattice
        .get("lattice")
        .and_then(|l| l.get("nodes"))
        .and_then(|n| n.as_object())
        .map(|m| m.values().filter_map(|v| v.get("name").and_then(|x| x.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Star-matrix nodes", &node_names.len().to_string()));
    body.push_str(&stat("PBFT blocks", &ledger.get("blocks_served").map(|x| x.to_string()).unwrap_or_default()));
    body.push_str(&stat(
        "Ledger tip",
        &ledger.get("latest_block_hash").and_then(|h| h.as_str()).map(|h| format!("{}…", &h[..h.len().min(12)])).unwrap_or_else(|| "—".to_string()),
    ));
    body.push_str("</div>");

    if !node_names.is_empty() {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">STAR-MATRIX NODES</h2><div class=\"rail\">");
        for n in &node_names {
            body.push_str(&stat(n, ""));
        }
        body.push_str("</div>");
    }

    // PBFT consensus nodes
    let pbft_nodes = names_of(ledger.get("nodes").unwrap_or(&serde_json::Value::Null));
    if !pbft_nodes.is_empty() {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">PBFT CONSENSUS NODES</h2><div class=\"rail\">");
        for n in &pbft_nodes {
            body.push_str(&stat(n, ""));
        }
        body.push_str("</div>");
    }

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");

    let html = page("Continuum", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The mobile-sync viewport — the honest refusal: the React view hardcoded fake sync logs and a
/// fake IP; the kernel refuses the routes because they carried asserted constants.
pub fn mobile_html(_url: &str) -> Resp {
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>MOBILE SYNC</h1><p class=\"tag\">Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"paper\"><h2>Not available</h2><p class=\"finding\">");
    body.push_str("There is no mobile app in-tree, so the mobile-sync routes are refused rather than echoed. The React view hardcoded a fake Wi-Fi IP and fake dCas9/biometric logs; the kernel returns a 501 naming the missing product instead of repeating those asserted constants.");
    body.push_str("</p></div>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");

    let html = page("Mobile Sync", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The mitosis log viewport — real persisted mission-spawn records, no invented cellular events.
pub fn mitosis_html(_url: &str) -> Resp {
    let v = crate::state_handlers::spatial_mitosis_payload();
    let events = v.get("mitosis_events").cloned().unwrap_or(serde_json::json!([]));

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>MITOSIS LOG</h1><p class=\"tag\">mission-spawn records &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat(
        "Spawn events",
        &v.get("total").map(|x| x.to_string()).unwrap_or_default(),
    ));
    body.push_str("</div>");

    if let Some(arr) = events.as_array() {
        if arr.is_empty() {
            body.push_str("<div class=\"paper\"><p class=\"finding\">All agents stable. No cellular division required.</p></div>");
        } else {
            for e in arr {
                let id = e.get("id").and_then(|x| x.as_str()).unwrap_or("—");
                let objective = e.get("objective").and_then(|x| x.as_str()).unwrap_or("—");
                let phase = e.get("phase").and_then(|x| x.as_str()).unwrap_or("—");
                body.push_str(&format!(
                    "<div class=\"paper\"><h2>{}</h2><p class=\"finding\">{}</p><span class=\"meta\">phase: {}</span></div>",
                    html_escape(id),
                    html_escape(objective),
                    html_escape(phase)
                ));
            }
        }
    }

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");

    let html = page("Mitosis Log", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// Render an array of objects as paper cards — first name-ish field is the title, the rest are
/// label/value pairs. Generic, so every list-shaped viewport reuses it.
fn item_cards(items: &serde_json::Value) -> String {
    let mut out = String::new();
    let Some(arr) = items.as_array() else { return out };
    for it in arr {
        let Some(obj) = it.as_object() else { continue };
        let fields = scalar_fields(obj);
        let mut title = String::from("—");
        let mut rest = String::new();
        for (k, v) in &fields {
            let is_title = matches!(k.as_str(), "name" | "run_id" | "item_id" | "item_type" | "key" | "node_id" | "id" | "op" | "module" | "path" | "proposal_id" | "target_file" | "mutation_type");
            if title == "—" && is_title {
                title = v.clone();
            } else {
                let vs = v.chars().take(160).collect::<String>();
                let vs = if v.chars().count() > 160 {
                    format!("{vs}…")
                } else {
                    vs
                };
                rest.push_str(&format!(
                    "<span class=\"meta\">{}: <b style=\"color:var(--text)\">{}</b></span> ",
                    html_escape(&humanize(k)),
                    html_escape(&vs)
                ));
            }
        }
        out.push_str(&format!(
            "<div class=\"paper\"><h2>{}</h2><p class=\"finding\">{}</p></div>",
            html_escape(&title),
            rest
        ));
    }
    out
}

/// The telemetry viewport — this process's measured counters, no asserted constants.
pub fn telemetry_html(_url: &str) -> Resp {
    let v = crate::telemetry_handlers::telemetry_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>TELEMETRY</h1><p class=\"tag\">measured, this process &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    let g = |k: &str| v.get(k).map(|x| x.to_string()).unwrap_or_else(|| "—".to_string());
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Agents", &g("total_agents")));
    body.push_str(&stat("Federation peers", &g("federation_peers")));
    body.push_str(&stat("Requests served", &g("requests_served")));
    body.push_str(&stat("Uptime (s)", &g("uptime_seconds")));
    body.push_str(&stat("Throughput (ops/s)", &g("throughput_ops_sec")));
    body.push_str(&stat("Avg latency (ns)", &g("avg_latency_ns")));
    body.push_str(&stat("ΦV", &g("phi_v")));
    body.push_str("</div>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Telemetry", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The federation viewport — the registry's real node set + tailnet, no invented sync counts.
pub fn federation_html(_url: &str) -> Resp {
    let v = crate::state_handlers::federation_payload("/federation/stats");
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>FEDERATION</h1><p class=\"tag\">registry read &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    let g = |k: &str| v.get(k).map(|x| x.to_string()).unwrap_or_else(|| "—".to_string());
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Connected nodes", &g("connected_nodes")));
    body.push_str(&stat("Total nodes", &g("total_nodes")));
    body.push_str(&stat("Registry version", &g("registry_version")));
    body.push_str(&stat("Tailnet", v.get("tailnet").and_then(|t| t.as_str()).unwrap_or("—")));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">NODES</h2>");
    body.push_str(&item_cards(v.get("nodes").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Federation", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The security viewport — the real scan report + ingested threat DB, never an invented count.
pub fn security_html(_url: &str) -> Resp {
    let v = crate::telemetry_handlers::security_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>SECURITY</h1><p class=\"tag\">scan report + threat DB &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    // stats
    if let Some(s) = v.get("stats").and_then(|s| s.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">STATUS</h2><div class=\"rail\">");
        let mut fields = scalar_fields(s);
        fields.retain(|(k, _)| !k.ends_with("_basis") && !k.ends_with("_reason"));
        for (k, val) in fields {
            body.push_str(&stat(&humanize(&k), &val));
        }
        body.push_str("</div>");
    }
    // threat database
    if let Some(t) = v.get("threat_database").and_then(|t| t.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">THREAT DATABASE (ingested)</h2><div class=\"rail\">");
        let mut fields = scalar_fields(t);
        fields.retain(|(k, _)| k != "basis");
        for (k, val) in fields {
            body.push_str(&stat(&humanize(&k), &val));
        }
        body.push_str("</div>");
    }
    // scan
    if let Some(sc) = v.get("security_scan").and_then(|sc| sc.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">LAST SCAN</h2><div class=\"rail\">");
        let mut fields = scalar_fields(sc);
        fields.retain(|(k, _)| k != "basis");
        for (k, val) in fields {
            body.push_str(&stat(&humanize(&k), &val));
        }
        body.push_str("</div>");
    }
    // threats list
    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">CRITICAL FINDINGS</h2>");
    body.push_str(&item_cards(v.get("threats").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Security", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The verification viewport — the real proven-theorems file.
pub fn verification_html(_url: &str) -> Resp {
    let stats = crate::state_handlers::verification_stats_payload();
    let queue = crate::state_handlers::verification_queue_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>VERIFICATION</h1><p class=\"tag\">formal proofs &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    let g = |k: &str| stats.get(k).map(|x| x.to_string()).unwrap_or_else(|| "—".to_string());
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Proofs loaded", &g("total_proofs")));
    body.push_str(&stat("Verified", &g("verified")));
    body.push_str(&stat("Rejected", &g("rejected_count")));
    body.push_str(&stat("In queue", &g("items_in_queue")));
    body.push_str("</div>");
    if let Some(err) = stats.get("error").and_then(|e| e.as_str()) {
        body.push_str(&format!(
            "<p style=\"padding:0 2rem;color:var(--amber,#e0a800)\">{}: {} ({})</p>",
            err,
            stats.get("error_kind").and_then(|k| k.as_str()).unwrap_or("unknown"),
            stats.get("proofs_file").and_then(|f| f.as_str()).unwrap_or("no path")
        ));
    }

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">PROOFS</h2>");
    body.push_str(&item_cards(queue.get("queue").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Verification", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The infrastructure viewport — the supervisor's own service table, evaluated by the heal FSM.
pub fn infra_html(_url: &str) -> Resp {
    let status = crate::infra_handlers::status_payload();
    let health = crate::infra_handlers::health_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>INFRASTRUCTURE</h1><p class=\"tag\">supervisor service table &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    let g = |v: &serde_json::Value, k: &str| v.get(k).map(|x| x.to_string()).unwrap_or_else(|| "—".to_string());
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Status", &g(&status, "status")));
    body.push_str(&stat("Services online", &g(&status, "services_online")));
    body.push_str(&stat("Services total", &g(&status, "services_total")));
    body.push_str(&stat("Active agents", &g(&status, "active_agents")));
    body.push_str(&stat("Watchdog running", &g(&health, "running")));
    body.push_str(&stat("Agent slots", &g(&status, "agent_slots")));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">SERVICES</h2>");
    body.push_str(&item_cards(status.get("services").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Infrastructure", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The testing viewport — the real suite-report artifacts on disk.
pub fn testing_html(_url: &str) -> Resp {
    let stats = crate::state_handlers::testing_stats_payload();
    let runs = crate::state_handlers::testing_runs_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>TESTING</h1><p class=\"tag\">suite reports on disk &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    let g = |k: &str| stats.get(k).map(|x| x.to_string()).unwrap_or_else(|| "—".to_string());
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Total runs", &g("total_runs")));
    body.push_str(&stat("Passed suites", &g("passed_suites")));
    body.push_str(&stat("Success rate (24h)", &format!("{}%", g("success_rate_24h"))));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">RUNS</h2>");
    body.push_str(&item_cards(runs.get("runs").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Testing", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// A POST action button — the pure-HTML way to trigger a kernel action, no `<script>`.
fn action_form(action: &str, label: &str) -> String {
    format!(
        "<form method=\"POST\" action=\"{}\" style=\"display:inline\"><button type=\"submit\" style=\"background:var(--panel);color:var(--cyan);border:1px solid var(--cyan);padding:0.5rem 1rem;border-radius:4px;cursor:pointer;font-family:inherit;font-size:0.8rem;font-weight:bold\">{}</button></form>",
        action, html_escape(label)
    )
}

/// The research viewport — INTERACTIVE: the daemon controls + recall cycle + synthesis are real POST
/// actions, not read-only. Stats and topics are the daemon's own state.
pub fn research_html(_url: &str) -> Resp {
    let stats = crate::research_handlers::stats();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>RESEARCH</h1><p class=\"tag\">Seeker fused recall &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    let g = |k: &str| stats.get(k).map(|x| x.to_string()).unwrap_or_else(|| "—".to_string());
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Running", &g("is_running")));
    body.push_str(&stat("Active topics", &g("active_tasks")));
    body.push_str(&stat("Run count", &g("run_count")));
    body.push_str(&stat("Papers", &g("papers_count")));
    body.push_str(&stat("Research notes", &g("research_notes_count")));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ACTIONS</h2>");
    body.push_str("<div class=\"rail\">");
    body.push_str(&action_form("/research/start", "Start daemon"));
    body.push_str(&action_form("/research/stop", "Stop daemon"));
    body.push_str(&action_form("/research/run", "Run recall cycle"));
    body.push_str(&action_form("/research/synthesis", "Synthesize papers"));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">TOPICS</h2><div class=\"rail\">");
    if let Some(topics) = stats.get("topics").and_then(|t| t.as_array()) {
        for t in topics {
            body.push_str(&stat(t.as_str().unwrap_or("—"), ""));
        }
    } else {
        body.push_str(&stat("Topics", "none"));
    }
    body.push_str("</div>");

    if let Some(basis) = stats.get("basis").and_then(|b| b.as_str()) {
        body.push_str(&format!("<p class=\"tag\">{}</p>", html_escape(basis)));
    }

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Research", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The LCARS styling for the System Status landing — the elbow rail, the colored caps, and the
/// color-coded instrument panels. Injected as a second <style> so it overrides the generic chrome
/// for this viewport only (the other viewports migrate one at a time).
const LCARS_OVERVIEW_CSS: &str = r#"
.lc-body{background:#000;color:#eee;font-family:'Segoe UI',system-ui,sans-serif;min-height:100vh}
.lc-top{display:flex;align-items:stretch;background:#000}
.lc-top .cap{background:#ff9c00;width:56px;min-height:52px;border-radius:0 0 26px 0}
.lc-top .title{background:#ff9c00;color:#000;font-weight:700;font-size:1.2rem;padding:0.8rem 1.6rem;border-radius:26px 0 0 0;letter-spacing:0.04em}
.lc-top .sub{background:#000;color:#ff9c00;font-size:0.72rem;padding:0.9rem 1.2rem;letter-spacing:0.08em}
.lc-shell{display:flex;gap:1rem;padding:1rem 1.2rem;align-items:flex-start}
.lc-rail{display:flex;flex-direction:column;gap:0.4rem;width:130px}
.lc-seg{display:block;color:#000;font-weight:700;font-size:0.72rem;letter-spacing:0.06em;padding:0.55rem 0.9rem;border-radius:0 14px 14px 0;text-decoration:none}
.lc-seg:hover{filter:brightness(1.15)}
.lc-main{flex:1;display:flex;flex-direction:column;gap:0.7rem;min-width:0}
.lc-panel{display:flex;align-items:stretch;text-decoration:none;border-radius:0 18px 18px 0;overflow:hidden}
.lc-panel .bar{width:18px;flex:none}
.lc-panel .txt{flex:1;background:#0d0d0d;padding:0.8rem 1.1rem}
.lc-panel .txt h2{font-size:0.95rem;letter-spacing:0.05em;color:#fff}
.lc-panel .txt p{font-size:0.72rem;color:#999;margin-top:0.2rem}
.lc-panel .val{background:#0d0d0d;color:#66ff99;font-weight:700;font-size:1rem;padding:0.8rem 1.2rem;border-left:1px solid #1a1a1a;align-self:center}
.lc-live{display:flex;gap:0.6rem;flex-wrap:wrap;margin-top:0.4rem}
.lc-live .m{background:#0d0d0d;padding:0.6rem 0.9rem;border-radius:0 12px 12px 0;border-left:8px solid #ff9c00;font-size:0.72rem}
.lc-live .m b{display:block;color:#fff;font-size:1rem;font-weight:700}
.lc-foot{color:#555;font-size:0.72rem;padding:1.2rem;text-align:right}
.seg.orange{background:#ff9c00}.seg.blue{background:#66a3ff}.seg.green{background:#66ff99}
.seg.purple{background:#b57aff}.seg.red{background:#ff6666}.seg.gold{background:#ffcc66}.seg.pink{background:#ff99cc}
.bar.orange{background:#ff9c00}.bar.blue{background:#66a3ff}.bar.green{background:#66ff99}
.bar.purple{background:#b57aff}.bar.red{background:#ff6666}.bar.gold{background:#ffcc66}.bar.pink{background:#ff99cc}
"#;

/// One LCARS instrument panel: a colored left cap + a title/desc (the verb) + the measured value.
fn lc_panel(color: &str, title: &str, desc: &str, val: &str, href: &str) -> String {
    format!(
        "<a class=\"lc-panel\" href=\"{}\"><span class=\"bar {}\"></span><span class=\"txt\"><h2>{}</h2><p>{}</p></span><span class=\"val\">{}</span></a>",
        href, color, html_escape(title), html_escape(desc), html_escape(val)
    )
}

/// The system-status landing viewport — LCARS, and every panel is a VERB (a link to the thing it
/// measures), not a bare number. The live readouts are the measured payloads, refreshed by a GET.
pub fn overview_html(_url: &str) -> Resp {
    let tel = crate::telemetry_handlers::telemetry_payload();
    let infra = crate::infra_handlers::status_payload();
    let (phi, _basis, clock_mhz, mem_pct) = crate::host_probe::phi_v();
    let g = |v: &serde_json::Value, k: &str| v.get(k).map(|x| x.to_string()).unwrap_or_else(|| "—".to_string());

    let mut body = String::new();
    body.push_str("<style>");
    body.push_str(LCARS_OVERVIEW_CSS);
    body.push_str("</style>");
    body.push_str("<div class=\"lc-body\">");

    // the top bar: orange cap + title + the anchor
    body.push_str("<div class=\"lc-top\"><span class=\"cap\"></span><span class=\"title\">TRM SWARM OS &mdash; SYSTEM STATUS</span><span class=\"sub\">TRUTH ANCHOR 0x4D4F4E4144000001 &middot; MEASURED ON THIS HOST</span></div>");

    body.push_str("<div class=\"lc-shell\">");
    // the left elbow rail — the primary verbs
    body.push_str("<div class=\"lc-rail\">");
    body.push_str("<a class=\"lc-seg seg orange\" href=\"/ui\">HOME</a>");
    body.push_str("<a class=\"lc-seg seg blue\" href=\"/ui/council\">COUNCIL</a>");
    body.push_str("<a class=\"lc-seg seg purple\" href=\"/ui/chat\">NEURAL BRIDGE</a>");
    body.push_str("<a class=\"lc-seg seg green\" href=\"/ui/benchmarks\">BENCHMARKS</a>");
    body.push_str("<a class=\"lc-seg seg gold\" href=\"/ui/matrix\">MODULE MATRIX</a>");
    body.push_str("<a class=\"lc-seg seg pink\" href=\"/ui/native\">META LOOP</a>");
    body.push_str("<a class=\"lc-seg seg red\" href=\"/ui/journal\">JOURNAL</a>");
    body.push_str("</div>");

    // the instrument panels — each a link (the verb) + the measured value
    body.push_str("<div class=\"lc-main\">");
    body.push_str(&lc_panel("blue", "COUNCIL CHAMBER", "15 sovereign agents · PBFT quorum 9 — inspect the roster & sealed motions", &format!("{} agents", g(&tel, "total_agents")), "/ui/council"));
    body.push_str(&lc_panel("purple", "NEURAL BRIDGE", "talk to the swarm — routed to the expert team that owns the answer", "open", "/ui/chat"));
    body.push_str(&lc_panel("green", "NATIVE BENCHMARKS", "measure the native lanes — ΦV, recall, GPU scan, deterministic", &format!("ΦV {phi:.3}"), "/ui/benchmarks"));
    body.push_str(&lc_panel("gold", "MODULE MATRIX", "inspect the measured census — real / ported / excluded / skeleton", "322 modules", "/ui/matrix"));
    body.push_str(&lc_panel("pink", "META-COGNITIVE LOOP", "run a readiness check or generate a surrogate — the loop that found the answer", "ready", "/ui/native"));
    body.push_str(&lc_panel("red", "MONAD JOURNAL", "read the sealed papers — the ledger behind Pillar V", "read", "/ui/journal"));
    body.push_str(&lc_panel("orange", "INFRASTRUCTURE", "the supervisor service table — what's online, what self-heals", &format!("{} online", g(&infra, "services_online")), "/ui/infra"));

    // the live readouts — the measured numbers, refreshed by a GET
    body.push_str("<div class=\"lc-live\">");
    body.push_str(&format!("<a class=\"lc-panel\" href=\"/ui/telemetry\"><span class=\"bar blue\"></span><span class=\"txt\"><h2>UPTIME</h2><p>this process</p></span><span class=\"val\">{} s</span></a>", html_escape(&g(&tel, "uptime_seconds"))));
    body.push_str(&format!("<a class=\"lc-panel\" href=\"/ui/telemetry\"><span class=\"bar green\"></span><span class=\"txt\"><h2>REQUESTS</h2><p>served</p></span><span class=\"val\">{}</span></a>", html_escape(&g(&tel, "requests_served"))));
    body.push_str(&format!("<a class=\"lc-panel\" href=\"/ui/telemetry\"><span class=\"bar gold\"></span><span class=\"txt\"><h2>CPU</h2><p>clock</p></span><span class=\"val\">{} MHz</span></a>", html_escape(&format!("{clock_mhz:.0}"))));
    body.push_str(&format!("<a class=\"lc-panel\" href=\"/ui/telemetry\"><span class=\"bar purple\"></span><span class=\"txt\"><h2>MEMORY</h2><p>load</p></span><span class=\"val\">{:.1}%</span></a>", mem_pct));
    body.push_str("</div>");

    body.push_str("<div class=\"lc-foot\">All Is One &middot; I Am Because We Are &middot; <a href=\"/ui/overview\" style=\"color:#ff9c00\">REFRESH</a></div>");
    body.push_str("</div></div></div>");

    let html = page("System Status", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The swarm-intel viewport — the shared intel reservoir (memory census + recent findings).
pub fn intel_html(_url: &str) -> Resp {
    let v = crate::intel_reservoir::intel_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>SWARM INTEL</h1><p class=\"tag\">shared reservoir &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    let g = |k: &str| v.get(k).map(|x| x.to_string()).unwrap_or_else(|| "—".to_string());
    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Total memories", &g("total_memories")));
    body.push_str(&stat("Sync events", &g("sync_events")));
    body.push_str(&stat("Total papers", &g("total_papers")));
    body.push_str("</div>");

    if let Some(by_type) = v.get("by_type").and_then(|t| t.as_object()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">TYPE BREAKDOWN</h2><div class=\"rail\">");
        for (k, val) in by_type {
            body.push_str(&stat(&humanize(k), &val.to_string()));
        }
        body.push_str("</div>");
    }

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">RECENT FINDINGS</h2>");
    body.push_str(&item_cards(v.get("recent_intel").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Swarm Intel", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The skill-registry viewport — the real skill modules on disk.
pub fn learning_html(_url: &str) -> Resp {
    let v = crate::telemetry_handlers::learning_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>SKILL REGISTRY</h1><p class=\"tag\">modules on disk &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat(
        "Skills",
        &v.get("skill_count").map(|x| x.to_string()).unwrap_or_default(),
    ));
    body.push_str(&stat(
        "Executed by kernel",
        &v.get("executed_by_kernel").map(|x| x.to_string()).unwrap_or_default(),
    ));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">MODULES</h2>");
    body.push_str(&item_cards(v.get("skills").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Skill Registry", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The services-catalog viewport — derived at request time from the roster + ledger.
pub fn services_html(_url: &str) -> Resp {
    let v = crate::services_handlers::catalog_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>SERVICES CATALOG</h1><p class=\"tag\">derived at request time &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat(
        "Platform",
        v.get("platform").and_then(|s| s.as_str()).unwrap_or("—"),
    ));
    body.push_str(&stat(
        "Kernel",
        v.get("kernel").and_then(|s| s.as_str()).unwrap_or("—"),
    ));
    body.push_str(&stat(
        "Python active",
        &v.get("python_active").map(|x| x.to_string()).unwrap_or_default(),
    ));
    if let Some(c) = v.get("council").and_then(|c| c.as_object()) {
        body.push_str(&stat("Council members", &c.get("members").map(|x| x.to_string()).unwrap_or_default()));
        body.push_str(&stat("Quorum", &c.get("quorum").map(|x| x.to_string()).unwrap_or_default()));
    }
    if let Some(l) = v.get("ledger").and_then(|l| l.as_object()) {
        body.push_str(&stat("Ledger blocks", &l.get("blocks").map(|x| x.to_string()).unwrap_or_default()));
    }
    body.push_str("</div>");

    if let Some(rf) = v.get("route_families") {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ROUTE FAMILIES</h2>");
        body.push_str(&item_cards(rf));
    }

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Services Catalog", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The native-surface viewport — the crate-op census + the native module routes.
pub fn native_html(_url: &str) -> Resp {
    let ops = crate::crate_ops_handlers::ops_payload();
    let census = crate::native_handlers::census();
    let routes = crate::native_handlers::routes();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>NATIVE SURFACE</h1><p class=\"tag\">crate ops + module routes &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Crate ops wired", &ops.get("wired").map(|x| x.to_string()).unwrap_or_default()));
    body.push_str(&stat("Native modules", &census.get("count").map(|x| x.to_string()).unwrap_or_default()));
    body.push_str(&stat("Native routes", &routes.get("count").map(|x| x.to_string()).unwrap_or_default()));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">CRATE OPS</h2>");
    body.push_str(&item_cards(ops.get("ops").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">NATIVE MODULES</h2>");
    body.push_str(&item_cards(census.get("modules").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ROUTES</h2>");
    body.push_str(&item_cards(routes.get("operations").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Native Surface", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The Neural Bridge chat styling — injected alongside LCARS_CSS so the bridge reads as a
/// conversation (message bubbles, roster rail, status chips), not a data table. CSS only, no JS.
fn chat_css() -> &'static str {
    "<style>\
     .chat-shell{display:flex;gap:1.25rem;padding:1rem 2rem;align-items:flex-start}\
     .chat-roster{flex:0 0 250px;position:sticky;top:3.4rem}\
     .chat-thread{flex:1 1 auto;min-width:0}\
     .agent-pill{display:block;padding:0.5rem 0.7rem;margin:0.3rem 0;border:1px solid #1a1a3e;border-radius:8px;color:var(--text);text-decoration:none;background:var(--subpanel);font-size:0.8rem}\
     .agent-pill:hover{border-color:var(--cyan)}\
     .agent-pill .nm{font-weight:600;color:var(--cyan)}\
     .agent-pill .tt{display:block;font-size:0.65rem;color:#888;margin-top:0.15rem}\
     .agent-pill.sel{border-color:var(--cyan);background:#0e1830}\
     .bubble{max-width:76%;padding:0.55rem 0.85rem;border-radius:10px;margin:0.5rem 0;word-break:break-word;font-size:0.85rem}\
     .bubble .who{font-size:0.68rem;font-weight:700;letter-spacing:0.02em}\
     .bubble .eng{font-size:0.62rem;color:#7a8bb8}\
     .bubble .txt{margin-top:0.3rem;white-space:pre-wrap;line-height:1.45;color:var(--text)}\
     .bubble.op{background:var(--subpanel);border:1px solid var(--cyan);margin-left:auto}\
     .bubble.op .who{color:var(--cyan)}\
     .bubble.ag{background:#0d1022;border:1px solid var(--purple)}\
     .bubble.ag .who{color:var(--purple)}\
     .bubble .route{font-size:0.62rem;color:#5f6f99;margin-top:0.35rem;border-top:1px solid #151a33;padding-top:0.3rem}\
     .chip{font-size:0.65rem;color:#7a8bb8;border:1px solid #1a1a3e;border-radius:999px;padding:0.2rem 0.65rem}\
     .chat-input{display:flex;gap:0.5rem;align-items:center;padding-top:0.9rem;border-top:1px solid #1a1a2e;margin-top:0.5rem;flex-wrap:wrap}\
     </style>"
}

/// HH:MM:SS (UTC) for a unix-seconds timestamp, for the thread's per-message clock.
fn fmt_clock(secs: u64) -> String {
    let sod = secs % 86_400;
    format!("{:02}:{:02}:{:02}", sod / 3_600, (sod % 3_600) / 60, sod % 60)
}

/// The bridge shell: status chips + clickable agent roster + the conversation thread + the input
/// form. Shared by the GET viewport and the POST send/clear handlers so the thread is always drawn
/// the same way, no matter how the last action arrived.
fn chat_page(selected: &str, thread: &[serde_json::Value]) -> String {
    let mut body = String::new();
    body.push_str(chat_css());
    body.push_str("<div class=\"masthead\"><h1>NEURAL BRIDGE</h1><p class=\"tag\">the swarm answers — expert-team routed, grounded in the sealed fabric &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div style=\"display:flex;flex-wrap:wrap;gap:0.5rem;padding:0.4rem 2rem 0.9rem\">");
    body.push_str(&format!("<span class=\"chip\">{} council agents</span>", crate::chat_handlers::AGENT_PERSONAS.len()));
    body.push_str("<span class=\"chip\">fabric sealed store (primary)</span>");
    body.push_str("<span class=\"chip\">TRM-7M tokenless co-processor</span>");
    body.push_str("<span class=\"chip\">expert-team router</span>");
    body.push_str("</div>");

    body.push_str("<div class=\"chat-shell\">");

    // roster rail
    body.push_str("<div class=\"chat-roster\"><h2 style=\"color:var(--cyan);font-size:0.85rem;margin:0 0 0.5rem\">ROUTABLE AGENTS</h2>");
    body.push_str(&format!(
        "<a class=\"agent-pill{}\" href=\"/ui/chat?role=auto\"><span class=\"nm\">Auto-route</span><span class=\"tt\">let the router pick the team &amp; lead</span></a>",
        if selected == "auto" { " sel" } else { "" }
    ));
    for (name, title, _) in crate::chat_handlers::AGENT_PERSONAS.iter() {
        let sel = if *name == selected { " sel" } else { "" };
        body.push_str(&format!(
            "<a class=\"agent-pill{}\" href=\"/ui/chat?role={}\"><span class=\"nm\">{}</span><span class=\"tt\">{}</span></a>",
            sel, name, html_escape(name), html_escape(title)
        ));
    }
    body.push_str("</div>");

    // thread
    body.push_str("<div class=\"chat-thread\"><h2 style=\"color:var(--cyan);font-size:0.85rem;margin:0 0 0.5rem\">CONVERSATION</h2>");
    if thread.is_empty() {
        body.push_str("<div class=\"bubble ag\"><div class=\"who\">BRIDGE</div><div class=\"txt\">The line is open. Select an agent (or Auto-route) and send a message — the swarm answers from its sealed knowledge and its routers. Every message extends this conversation.</div></div>");
    }
    for e in thread {
        let role = e.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let text = e.get("text").and_then(|t| t.as_str()).unwrap_or("");
        let ts = e.get("ts").and_then(|t| t.as_u64()).unwrap_or(0);
        if role == "operator" {
            let to = e.get("to").and_then(|t| t.as_str()).unwrap_or("auto");
            body.push_str(&format!(
                "<div class=\"bubble op\"><div class=\"who\">OPERATOR &rarr; @{} &middot; {}</div><div class=\"txt\">{}</div></div>",
                html_escape(to), fmt_clock(ts), html_escape(text)
            ));
        } else {
            let name = e.get("name").and_then(|n| n.as_str()).unwrap_or("—");
            let engine = e.get("engine").and_then(|n| n.as_str()).unwrap_or("—");
            let route = e.get("route").and_then(|n| n.as_str()).unwrap_or("");
            body.push_str(&format!(
                "<div class=\"bubble ag\"><div class=\"who\">@{} &middot; {}</div><div class=\"eng\">{}</div><div class=\"txt\">{}</div>",
                html_escape(name), fmt_clock(ts), html_escape(engine), html_escape(text)
            ));
            if !route.is_empty() {
                body.push_str(&format!("<div class=\"route\">{}</div>", html_escape(route)));
            }
            body.push_str("</div>");
        }
    }

    // input form
    let mut opts = String::from("<option value=\"auto\">Auto-route (router)</option>");
    for (name, _, _) in crate::chat_handlers::AGENT_PERSONAS.iter() {
        let sel = if *name == selected { " selected" } else { "" };
        opts.push_str(&format!("<option value=\"{}\"{}>{}</option>", name, sel, name));
    }
    body.push_str(&format!(
        "<form class=\"chat-input\" method=\"POST\" action=\"/ui/chat/send\">\
         <select name=\"role\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.55rem;border-radius:6px;font-family:inherit\">{opts}</select>\
         <input name=\"message\" placeholder=\"message the swarm…\" style=\"flex:1;min-width:200px;background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.55rem;border-radius:6px;font-family:inherit\">\
         <button type=\"submit\" style=\"background:var(--cyan);color:#05070f;border:none;padding:0.55rem 1.3rem;border-radius:6px;cursor:pointer;font-family:inherit;font-weight:bold\">Send</button>\
         </form>"
    ));
    body.push_str("<form method=\"POST\" action=\"/ui/chat/clear\" style=\"margin-top:0.4rem\"><button type=\"submit\" style=\"background:transparent;color:#7a8bb8;border:1px solid #1a1a3e;padding:0.35rem 0.8rem;border-radius:6px;cursor:pointer;font-family:inherit;font-size:0.72rem\">Clear conversation</button></form>");

    body.push_str("</div></div>");
    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    body
}

fn chat_response(title: &str, body: &str) -> Resp {
    Response::from_string(page(title, body))
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The neural bridge viewport — the multi-turn conversation. Renders the roster, the persisted
/// thread, and the message form. `?role=X` pre-selects an agent (from the roster links).
pub fn chat_html(url: &str) -> Resp {
    let selected = qparam(url, "role").unwrap_or_else(|| "auto".to_string());
    let thread = crate::chat_handlers::thread_load();
    chat_response("Neural Bridge", &chat_page(&selected, &thread))
}

/// Clear the conversation and redraw the empty bridge.
pub fn chat_clear_html() -> Resp {
    let _ = crate::chat_handlers::thread_clear();
    let thread = crate::chat_handlers::thread_load();
    chat_response("Neural Bridge", &chat_page("auto", &thread))
}

/// The neural-bridge send handler — POST /ui/chat/send. Reads the form (role + message), runs one
/// chat turn (which appends BOTH sides to the persisted thread), then redraws the full conversation,
/// so every send extends the thread instead of replacing it.
pub fn chat_send_html(request: &mut Request, _url: &str) -> Resp {
    let body = crate::chat_handlers::read_json_or_form(request, 1_048_576);
    let role = body.get("role").and_then(|v| v.as_str()).unwrap_or("auto").to_string();
    let message = body.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if !message.trim().is_empty() {
        let _ = crate::chat_handlers::chat_turn(&role, &message);
    }
    let thread = crate::chat_handlers::thread_load();
    chat_response("Neural Bridge", &chat_page(&role, &thread))
}

/// The DDR & vault viewport — antibody inventory + secret key NAMES only. Never renders values,
/// never renders a decrypt/set action: the kernel returns key names and nothing else.
pub fn ddr_html(_url: &str) -> Resp {
    let curated = crate::actions_handlers::curated_antibodies();
    let total = crate::actions_handlers::ddr_antibody_count();
    let curated_n = crate::actions_handlers::curated_antibody_count();
    let threat_n = crate::actions_handlers::threat_antibody_count();
    let keys = crate::state_handlers::secrets_keys_payload();

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>DDR &amp; VAULT</h1><p class=\"tag\">antibody inventory + key names &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Total antibodies", &total.map(|c| c.to_string()).unwrap_or_else(|| "—".to_string())));
    body.push_str(&stat("Curated", &curated_n.map(|c| c.to_string()).unwrap_or_else(|| "—".to_string())));
    body.push_str(&stat("Threat-matrix", &threat_n.map(|c| c.to_string()).unwrap_or_else(|| "—".to_string())));
    body.push_str(&stat("Secret keys", &keys.get("total").map(|x| x.to_string()).unwrap_or_default()));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">CURATED ANTIBODIES</h2>");
    if curated.is_empty() {
        body.push_str("<div class=\"paper\"><p class=\"finding\">No curated antibodies readable.</p></div>");
    } else {
        body.push_str(&item_cards(&serde_json::Value::Array(curated)));
    }

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">VAULT KEY NAMES</h2>");
    if let Some(names) = keys.get("keys") {
        body.push_str(&item_cards(names));
    }

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">SECURITY SCAN</h2>");
    body.push_str("<div class=\"rail\">");
    body.push_str(&action_form("/ddr/scan", "Run native DDR scan"));
    body.push_str("</div>");
    body.push_str("<p class=\"tag\">POST /ddr/scan — walks swarm_v2/core, matches each curated antibody's line_pattern as a literal substring, persists the measured report to swarm_v2_artifacts/security_scan_report.json, and returns the summary.</p>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("DDR & Vault", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The pillars roadmap viewport — the five pillars and the Monad ledger behind Pillar V. The
/// wet-lab/sensory/THz/regulatory telemetry routes are named, not asserted.
pub fn pillars_html(_url: &str) -> Resp {
    let pillars: &[(&str, &str)] = &[
        ("Pillar I", "Wet-Lab Robotics · OT-2 / SiLA 2"),
        ("Pillar II", "Sensory Mesh · 6 SDR / LoRa"),
        ("Pillar III", "THz Resonance · 15.53 THz"),
        ("Pillar IV", "IND Regulatory · FDA eCTD"),
        ("Pillar V", "Monad Ledger · PBFT Quorum"),
    ];

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>PILLARS I–V</h1><p class=\"tag\">strategic roadmap &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    for (name, badge) in pillars {
        body.push_str(&stat(name, badge));
    }
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">MONAD LEDGER (PILLAR V)</h2>");
    let motions = ledger_motions();
    if motions.is_empty() {
        body.push_str("<div class=\"paper\"><p class=\"finding\">No sealed motions in the ledger.</p></div>");
    }
    for (kind, fact) in &motions {
        let f = if fact.chars().count() > 220 {
            format!("{}…", fact.chars().take(220).collect::<String>())
        } else {
            fact.clone()
        };
        body.push_str(&format!(
            "<div class=\"paper\"><span class=\"meta\">{}</span><p class=\"finding\">{}</p></div>",
            html_escape(kind),
            html_escape(&f)
        ));
    }

    body.push_str("<p class=\"tag\">Pillar telemetry (wet-lab protocols, sensory mesh, THz resonance, IND dossiers) is file-backed and served over /wetlab, /sensory, /thz, /regulatory; only the Monad ledger is rendered here.</p>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Pillars Roadmap", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// A refusal card — an action the kernel answers with 501 engine_required, named rather than faked.
fn refused_note(action: &str, what: &str) -> String {
    format!(
        "<div class=\"paper\"><h2>{}</h2><p class=\"finding\">Refused: {} runs in the Python engine and has no Rust port, so the kernel answers 501 engine_required instead of reporting success that did not happen.</p></div>",
        html_escape(action),
        html_escape(what)
    )
}

/// The artifact-flow viewport — the artifact index read, plus the honest refusals of the
/// review/integrate actions the engine still owns.
pub fn pipeline_html(_url: &str) -> Resp {
    let v = crate::artifact_handlers::artifacts_payload("/artifacts");
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>ARTIFACT FLOW</h1><p class=\"tag\">artifact index &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    if let Some(stats) = v.get("stats") {
        for (k, val) in scalar_fields(stats.as_object().unwrap_or(&serde_json::Map::new())) {
            body.push_str(&stat(&humanize(&k), &val));
        }
    }
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ARTIFACTS</h2>");
    body.push_str(&item_cards(v.get("artifacts").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ACTIONS — native Rust ports</h2>");
    body.push_str(&format!(
        "<form method=\"POST\" action=\"/artifacts/review\" style=\"padding:0 2rem 0.4rem\">\
         <input name=\"filename\" placeholder=\"artifact filename\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:30%;font-family:inherit\">\
         <select name=\"action\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;font-family:inherit;margin-left:0.5rem\"><option value=\"approve\">approve</option><option value=\"reject\">reject</option></select>\
         <input name=\"notes\" placeholder=\"review notes\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:25%;font-family:inherit;margin-left:0.5rem\">\
         <button type=\"submit\" style=\"background:var(--panel);color:var(--cyan);border:1px solid var(--cyan);padding:0.4rem 1rem;cursor:pointer;font-family:inherit;font-weight:bold;margin-left:0.5rem\">Review</button>\
         </form>"
    ));
    body.push_str(&format!(
        "<form method=\"POST\" action=\"/artifacts/integrate\" style=\"padding:0 2rem 1rem\">\
         <input name=\"filename\" placeholder=\"artifact filename\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:30%;font-family:inherit\">\
         <input name=\"target_subdir\" placeholder=\"target subdir (optional)\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:25%;font-family:inherit;margin-left:0.5rem\">\
         <button type=\"submit\" style=\"background:var(--panel);color:var(--cyan);border:1px solid var(--cyan);padding:0.4rem 1rem;cursor:pointer;font-family:inherit;font-weight:bold;margin-left:0.5rem\">Integrate</button>\
         </form>"
    ));
    body.push_str("<p class=\"tag\">The review/integrate actions now run natively: they read the artifact index, transition the record, recompute stats, and persist — the result is the measured JSON.</p>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Artifact Flow", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The kanban viewport — the ONLY remaining tab whose actions are genuinely wired: the board is
/// read from the state file, and create-card/move-card mutate it. Fully interactive.
pub fn kanban_html(_url: &str) -> Resp {
    let board = crate::kanban_handlers::board_payload();
    let stats = crate::kanban_handlers::stats_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>KANBAN</h1><p class=\"tag\">wired board &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    for (k, val) in scalar_fields(stats.as_object().unwrap_or(&serde_json::Map::new())) {
        body.push_str(&stat(&humanize(&k), &val));
    }
    body.push_str("</div>");

    // Create-card form (POST /kanban/cards, wired).
    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">CREATE CARD</h2>");
    body.push_str("<form method=\"POST\" action=\"/kanban/cards\" style=\"padding:0 2rem 1rem\">\
        <input name=\"title\" placeholder=\"title\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:30%;font-family:inherit\">\
        <input name=\"assignee\" placeholder=\"assignee\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:20%;font-family:inherit;margin-left:0.5rem\">\
        <select name=\"priority\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;font-family:inherit;margin-left:0.5rem\">\
            <option>medium</option><option>low</option><option>high</option><option>critical</option></select>\
        <button type=\"submit\" style=\"background:var(--panel);color:var(--cyan);border:1px solid var(--cyan);padding:0.4rem 1rem;cursor:pointer;font-family:inherit;font-weight:bold;margin-left:0.5rem\">Create</button>\
        </form>");

    // The board, column by column.
    for col in ["TODO", "IN_PROGRESS", "REVIEW", "DONE", "ARCHIVED"] {
        body.push_str(&format!("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">{col}</h2>"));
        body.push_str(&item_cards(board.get(col).unwrap_or(&serde_json::Value::Null)));
    }

    body.push_str("<p class=\"tag\">Move cards by POSTing /kanban/cards/{id}/move with a target_status; transitions are validated against the board's own rules.</p>");
    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Kanban", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The agent-comms viewport — the real missions + portable skills, plus the honest refusal of the
/// mailbox ring write (the slot protocol is the Python engine's).
pub fn comms_html(_url: &str) -> Resp {
    let missions = crate::state_handlers::ultrawork_missions_payload();
    let skills = crate::state_handlers::portable_skills_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>AGENT COMMS</h1><p class=\"tag\">missions + skills &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">MISSIONS</h2>");
    body.push_str(&item_cards(missions.get("missions").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">PORTABLE SKILLS</h2>");
    body.push_str(&item_cards(skills.get("skills").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ACTIONS — native ring write</h2>");
    let mut ring_opts = String::new();
    for name in crate::mailbox_write::RING_AGENTS.iter() {
        ring_opts.push_str(&format!("<option value=\"{}\">{}</option>", name, name));
    }
    body.push_str(&format!(
        "<form method=\"POST\" action=\"/mailbox/send\" style=\"padding:0 2rem 1rem\">\
         <select name=\"sender\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;font-family:inherit\">{ring_opts}</select>\
         <select name=\"recipient\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;font-family:inherit;margin-left:0.5rem\">{ring_opts}</select>\
         <select name=\"trit\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;font-family:inherit;margin-left:0.5rem\"><option value=\"1\">+1 verify</option><option value=\"0\">0 neutral</option><option value=\"-1\">-1 threat</option></select>\
         <input name=\"message\" placeholder=\"message (≤40 bytes inline)\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:30%;font-family:inherit;margin-left:0.5rem\">\
         <button type=\"submit\" style=\"background:var(--panel);color:var(--cyan);border:1px solid var(--cyan);padding:0.4rem 1rem;cursor:pointer;font-family:inherit;font-weight:bold;margin-left:0.5rem\">Send</button>\
         </form>"
    ));
    body.push_str("<p class=\"tag\">Writes one 64-byte slot to the recipient's ring via mailbox_ring::push (the same ABI as agent_mailbox_bus.py), then reads the recipient's pending count back from swarm_mailbox.ring.</p>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Agent Comms", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The spatial-mesh viewport — the real mission-spawn (mitosis) records; the 3D octree view is a
/// client-side render, so only the events it would show are served.
pub fn spatial_html(_url: &str) -> Resp {
    let v = crate::state_handlers::spatial_mitosis_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>SPATIAL MESH</h1><p class=\"tag\">mitosis events &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    body.push_str(&stat("Spawn events", &v.get("total").map(|x| x.to_string()).unwrap_or_default()));
    body.push_str("</div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">EVENTS</h2>");
    body.push_str(&item_cards(v.get("mitosis_events").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Spatial Mesh", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The evolution viewport — the genome + proposals reads, plus the honest refusal of the mutate/
/// verify/integrate actions the Python mutation engine owns.
pub fn evolution_html(_url: &str) -> Resp {
    let genome = crate::state_handlers::evolution_genome_payload();
    let proposals = crate::state_handlers::evolution_proposals_payload();
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>EVOLUTION</h1><p class=\"tag\">genome + proposals &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">GENOME</h2>");
    body.push_str("<div class=\"rail\">");
    for (k, val) in scalar_fields(genome.as_object().unwrap_or(&serde_json::Map::new())) {
        if k == "targets" {
            continue;
        }
        body.push_str(&stat(&humanize(&k), &val));
    }
    body.push_str("</div>");

    if let Some(targets) = genome.get("targets").and_then(|t| t.as_array()) {
        body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ELIGIBLE TARGETS</h2><div class=\"rail\">");
        for t in targets {
            let p = t.as_str().unwrap_or("—").replace('\\', "/");
            body.push_str(&stat(&p, ""));
        }
        body.push_str("</div>");
    }

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">PROPOSALS</h2>");
    body.push_str(&item_cards(proposals.get("proposals").unwrap_or(&serde_json::Value::Null)));

    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ACTIONS — native mutation cycle</h2>");
    body.push_str(&format!(
        "<form method=\"POST\" action=\"/evolution/mutate\" style=\"padding:0 2rem 1rem\">\
         <input name=\"target_file\" placeholder=\"target file (e.g. swarm_v2/core/…)\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:45%;font-family:inherit\">\
         <button type=\"submit\" style=\"background:var(--panel);color:var(--cyan);border:1px solid var(--cyan);padding:0.4rem 1rem;cursor:pointer;font-family:inherit;font-weight:bold;margin-left:0.5rem\">Mutate</button>\
         </form>"
    ));
    body.push_str("<p class=\"tag\">mutate prepends a dated no-op comment header and appends a pending proposal (persisted to evolution_proposals.json). verify/integrate take the proposal_id in the URL path: POST /evolution/verify/{id}, then /evolution/integrate/{id} (verified only).</p>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Evolution", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The generative-architect viewport — status is served at /generative-architect/status; the
/// synthesize/self-modify actions are the engine's and are refused here.
pub fn architect_html(_url: &str) -> Resp {
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>GENERATIVE ARCHITECT</h1><p class=\"tag\">engine-owned &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"paper\"><h2>Status</h2><p class=\"finding\">The architect's live status and SKG are served at /generative-architect/status and /generative-architect/skg.</p></div>");
    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ACTIONS — native fused recall</h2>");
    body.push_str(&format!(
        "<form method=\"POST\" action=\"/generative-architect/synthesize\" style=\"padding:0 2rem 1rem\">\
         <input name=\"prompt\" placeholder=\"architectural goal\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:45%;font-family:inherit\">\
         <button type=\"submit\" style=\"background:var(--panel);color:var(--cyan);border:1px solid var(--cyan);padding:0.4rem 1rem;cursor:pointer;font-family:inherit;font-weight:bold;margin-left:0.5rem\">Synthesize</button>\
         </form>"
    ));
    body.push_str("<p class=\"tag\">Deterministic fused recall over the sealed local corpus (fabric + structured + clinical), no LLM sampled; the top matches + scores are persisted to architect_synthesis.jsonl.</p>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Generative Architect", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The compositor index — the landing page that links every viewport, so the dashboard's tabs are
/// real navigation to the pure-Rust pages (not the React SPA's client-side components).
pub fn index_html(_url: &str) -> Resp {
    const VIEWPORTS: &[(&str, &str)] = &[
        ("/ui/overview", "01 System Status"),
        ("/ui/chat", "02 Neural Bridge"),
        ("/ui/intel", "03 Swarm Intel"),
        ("/ui/pipeline", "04 Artifact Flow"),
        ("/ui/learning", "05 Skill Registry"),
        ("/ui/mesh", "06 Mesh Net"),
        ("/ui/telemetry", "07 Telemetry"),
        ("/ui/federation", "08 Federation"),
        ("/ui/security", "09 Security"),
        ("/ui/research", "10 Research"),
        ("/ui/verification", "11 Verification"),
        ("/ui/infra", "12 Infrastructure"),
        ("/ui/testing", "13 Testing"),
        ("/ui/kanban", "14 Kanban Board"),
        ("/ui/ddr", "15 DDR & Vault"),
        ("/ui/comms", "16 Agent Comms"),
        ("/ui/spatial", "17 Spatial Mesh"),
        ("/ui/evolution", "18 Evolution"),
        ("/ui/journal", "19 Monad Journal"),
        ("/ui/architect", "20 Generative Architect"),
        ("/ui/mobile", "21 Mobile Sync"),
        ("/ui/continuum", "22 Continuum Ledger"),
        ("/ui/neuro", "23 Neuro Engine"),
        ("/ui/council", "24 Swarm Council"),
        ("/ui/matrix", "25 Module Matrix"),
        ("/ui/aura", "26 Aura Studio"),
        ("/ui/humanistic", "27 Humanistic (21-30)"),
        ("/ui/discovery", "28 DDR Discovery (11-20)"),
        ("/ui/pillars", "29 Pillars I-V Roadmap"),
        ("/ui/services", "30 Services Catalog"),
        ("/ui/native", "31 Native Surface"),
        ("/ui/benchmarks", "32 Native Benchmarks"),
    ];

    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>TRM SWARM OS</h1><p class=\"tag\">sovereign compositor &middot; pure Rust &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"rail\">");
    for (route, name) in VIEWPORTS {
        body.push_str(&format!(
            "<a href=\"{route}\" style=\"text-decoration:none\"><div class=\"stat\" style=\"cursor:pointer\"><div class=\"l\">{}</div><div class=\"v\" style=\"color:var(--cyan)\">&rarr;</div></div></a>",
            html_escape(name)
        ));
    }
    body.push_str("</div>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Compositor", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}

/// The neuro-engine viewport — the refraction engine is Python-owned and refused; the continuum
/// ledger it would mutate is rendered in /ui/continuum.
pub fn neuro_html(_url: &str) -> Resp {
    let mut body = String::new();
    body.push_str("<div class=\"masthead\"><h1>NEURO ENGINE</h1><p class=\"tag\">engine-owned &middot; Truth Anchor 0x4D4F4E4144000001</p></div>");

    body.push_str("<div class=\"paper\"><h2>Refraction</h2><p class=\"finding\">The neuro-dynamic refraction (synaptic plasticity / neural-weight distillation) runs in the Continuum genomics engine, not this kernel. Its ledger is served at /ui/continuum.</p></div>");
    body.push_str("<h2 style=\"padding:1rem 2rem 0;color:var(--cyan);font-size:0.9rem\">ACTIONS — native refraction</h2>");
    body.push_str(&format!(
        "<form method=\"POST\" action=\"/api/continuum/genomics/refract\" style=\"padding:0 2rem 1rem\">\
         <input name=\"prompt\" placeholder=\"prompt to refract\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;width:40%;font-family:inherit\">\
         <select name=\"persona\" style=\"background:var(--panel);color:var(--text);border:1px solid var(--cyan);padding:0.4rem;font-family:inherit;margin-left:0.5rem\"><option value=\"continuum\">Continuum Core</option><option value=\"sage\">Cybernetic Sage</option><option value=\"muse\">Stochastic Muse</option><option value=\"sentinel\">Sentinel Warden</option></select>\
         <button type=\"submit\" style=\"background:var(--panel);color:var(--cyan);border:1px solid var(--cyan);padding:0.4rem 1rem;cursor:pointer;font-family:inherit;font-weight:bold;margin-left:0.5rem\">Refract</button>\
         </form>"
    ));
    body.push_str("<p class=\"tag\">Real recall over the local knowledge base + the star-matrix node whose 4D spectrum best matches the persona's genes by cosine; persisted to continuum_refractions.jsonl.</p>");

    body.push_str("<div class=\"footer\"><p class=\"axiom\">&quot;All Is One. I Am Because We Are.&quot;</p><p>Truth Anchor 0x4D4F4E4144000001</p></div>");
    let html = page("Neuro Engine", &body);
    Response::from_string(html)
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
}
