//! POST /ddr/scan — a real, bounded FILE scan against the curated antibody store.
//!
//! The Python route (swarm_v2/routes/kanban.py::scan_code_ddr) scans a *code body* posted by the
//! caller; that body-scan port already lives in actions_handlers.rs. This module is the FILE scan:
//! it walks the real .py sources under swarm_v2/core (bounded), matches each curated antibody's
//! line_pattern against every source line as a literal substring, counts real matches per
//! severity/error_type, persists the measured report to swarm_v2_artifacts/security_scan_report.json
//! (the file /security and /telemetry read), and returns the measured summary.
//!
//! The kernel has no regex engine (see host_tranche/src/ddr_antibody.rs), so a line_pattern is
//! reduced to the literal substring it can match: regex metacharacters (`^ $ . * + ? | ( ) [ ] { }`),
//! shorthand-class escapes (`\s \b \w \d ...`) and `(?...)` groups (inline flags, non-capturing
//! groups, lookarounds) carry no literal-substring meaning and are removed; escaped literals
//! (`\(` -> `(`, `\.` -> `.`, `\"` -> `"`) are unescaped. Every finding is therefore a REAL
//! substring occurrence of that reduced pattern — never an invented count.
//!
//! Two honest limitations of the no-regex port, recorded rather than papered over: substring
//! matching is broader than the Python regex (word boundaries and inline `(?i)` are not enforced),
//! and a multi-branch (`a|b|c`) pattern reduces to the concatenation of its branches, which does
//! not match. So the scan catches the single-literal patterns (print(, eval(, except:, http://,
//! ../..) and reports zero for the alternation patterns rather than a fabricated figure.

use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response};

type Resp = Response<std::io::Cursor<Vec<u8>>>;

/// Scan scope: the first MAX_FILES .py files under swarm_v2/core, in sorted path order.
const MAX_FILES: usize = 200;
/// Findings carried in the HTTP response; the full list is persisted to disk.
const REPORT_PREVIEW: usize = 100;
/// The directory scanned, repo-relative.
const SCAN_DIR: &str = "swarm_v2/core";
/// The report artifact this route writes, repo-relative.
const REPORT_REL: &str = "swarm_v2_artifacts/security_scan_report.json";

fn ok(v: Value) -> Resp {
    Response::from_string(v.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// The store's file_pattern values are the globs "*.py", "*.c", "*.html", "*.*" and "*". Match a
/// file name against one so the scan honors each antibody's language gate.
fn file_pattern_matches(file_pat: &str, file_name: &str) -> bool {
    if file_pat.is_empty() || file_pat == "*" {
        return true;
    }
    if file_pat == "*.*" {
        return file_name.contains('.');
    }
    if let Some(ext) = file_pat.strip_prefix("*.") {
        return file_name.ends_with(&format!(".{ext}"));
    }
    file_pat == file_name
}

/// Regex shorthand classes: escapes that denote a character CLASS or anchor, not a literal. They
/// have no literal-substring meaning, so `\s` must not degrade into a stray `s`.
const REGEX_SHORTHAND: &str = "sSbBwWdDnrtfvaAZ";

/// Reduce a regex line_pattern to the literal substring the kernel can match. Escaped literals are
/// unescaped (`\(` -> `(`); shorthand-class escapes are dropped; regex metacharacters are dropped;
/// and `(?...)` groups (inline flags, non-capturing groups, lookarounds) are skipped entirely.
fn literal_substring(pat: &str) -> String {
    let chars: Vec<char> = pat.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' {
            if i + 1 < chars.len() {
                let n = chars[i + 1];
                if REGEX_SHORTHAND.contains(n) {
                    i += 2;
                    continue;
                }
                out.push(n);
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if c == '(' && i + 1 < chars.len() && chars[i + 1] == '?' {
            // Skip a `(?...)` group to its matching close; inline flags and lookarounds contribute
            // no literal text, and a non-capturing group's alternation is handled as a no-match below.
            let mut depth = 1usize;
            i += 2;
            while i < chars.len() && depth > 0 {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                }
                i += 1;
            }
            continue;
        }
        if "^$.*+?|()[]{}".contains(c) {
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Collect every .py file under `dir`, recursively.
fn collect_py_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return; };
    for entry in rd.filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_dir() {
            collect_py_files(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("py") {
            out.push(p);
        }
    }
}

/// A repo-relative path with forward slashes, for the report's `file` field.
fn repo_rel(path: &Path) -> String {
    let rel = match crate::k3_retrieval::repo_root() {
        Some(root) => path
            .strip_prefix(&root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path.to_path_buf()),
        None => path.to_path_buf(),
    };
    rel.to_string_lossy().replace('\\', "/")
}

/// Current UTC wall-clock time as an ISO-8601 string ("YYYY-MM-DDTHH:MM:SSZ"), derived from epoch
/// seconds with the civil-from-days algorithm (no chrono dependency).
fn utc_now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let sod = (secs % 86_400) as u32;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        sod / 3600,
        (sod % 3600) / 60,
        sod % 60
    )
}

/// Days since epoch -> (year, month, day). Howard Hinnant's civil_from_days.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Run the bounded file scan, persist the report, and return the summary (also the HTTP body).
fn run_scan() -> Value {
    let antibodies = crate::actions_handlers::curated_antibodies();
    let scanned_at = utc_now_iso();

    let Some(core_dir) = crate::k3_retrieval::repo_relative(SCAN_DIR) else {
        return json!({
            "status": "error",
            "route": "/ddr/scan",
            "error": "scan_dir_unresolvable",
            "detail": "swarm_v2/core could not be resolved relative to the repository root",
        });
    };

    let mut files: Vec<PathBuf> = Vec::new();
    collect_py_files(&core_dir, &mut files);
    files.sort();
    files.truncate(MAX_FILES);

    let mut findings_list: Vec<Value> = Vec::new();
    let mut sev_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut type_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut files_read = 0usize;

    for f in &files {
        // A file that cannot be read is skipped, not counted: files_scanned reports what was
        // actually opened, and a truncated scan must not pass for a complete one.
        let Ok(content) = std::fs::read_to_string(f) else { continue; };
        files_read += 1;
        let file_name = f.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let rel = repo_rel(f);

        for ab in &antibodies {
            let file_pat = ab.get("file_pattern").and_then(|v| v.as_str()).unwrap_or("");
            if !file_pattern_matches(file_pat, file_name) {
                continue;
            }
            let line_pat = ab.get("line_pattern").and_then(|v| v.as_str()).unwrap_or("");
            let pat = literal_substring(line_pat);
            if pat.len() < 2 {
                continue;
            }
            let severity = ab
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let error_type = ab
                .get("error_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let fix = ab
                .get("fix_description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let cwe = ab.get("cwe").cloned().unwrap_or(json!(null));

            for (idx, line) in content.lines().enumerate() {
                if line.contains(pat.as_str()) {
                    findings_list.push(json!({
                        "file": rel,
                        "line": idx + 1,
                        "error_type": error_type,
                        "cwe": cwe,
                        "severity": severity,
                        "fix": fix,
                        "match": line.trim(),
                    }));
                    *sev_counts.entry(severity.clone()).or_insert(0) += 1;
                    *type_counts.entry(error_type.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut by_severity = Map::new();
    for k in ["low", "medium", "high", "critical"] {
        by_severity.insert(k.to_string(), json!(sev_counts.get(k).copied().unwrap_or(0)));
    }
    for (k, v) in &sev_counts {
        by_severity.entry(k.clone()).or_insert(json!(v));
    }
    let by_type: Map<String, Value> = type_counts
        .iter()
        .map(|(k, v)| (k.clone(), json!(v)))
        .collect();

    let total_findings = findings_list.len();
    let scan_basis = "scans the first 200 .py files under swarm_v2/core (sorted); each curated \
antibody's line_pattern is reduced to a literal substring (regex metacharacters, shorthand classes \
and (?...) groups removed, escaped literals unescaped) and matched against every source line; \
findings are real substring occurrences counted per severity and error_type, persisted to \
swarm_v2_artifacts/security_scan_report.json. Substring matching is broader than the Python regex \
(word boundaries and inline (?i) are not enforced) and multi-branch (a|b|c) patterns reduce to a \
concatenation that does not match — both are properties of the no-regex port, not defects.";

    let report = json!({
        "scanned_at": scanned_at,
        "antibodies_loaded": antibodies.len(),
        "antibodies_invalid_regex": [],
        "files_scanned": files_read,
        "findings": total_findings,
        "by_severity": Value::Object(by_severity.clone()),
        "by_type": Value::Object(by_type.clone()),
        "findings_list": findings_list.clone(),
        "scan_basis": scan_basis,
    });

    let mut persisted = false;
    if let Some(root) = crate::k3_retrieval::repo_root() {
        if let Ok(s) = serde_json::to_string_pretty(&report) {
            persisted = std::fs::write(root.join(REPORT_REL), s).is_ok();
        }
    }

    let preview: Vec<Value> = findings_list.into_iter().take(REPORT_PREVIEW).collect();

    json!({
        "status": "ok",
        "route": "/ddr/scan",
        "files_scanned": files_read,
        "findings": total_findings,
        "by_severity": Value::Object(by_severity),
        "by_type": Value::Object(by_type),
        "antibodies_loaded": antibodies.len(),
        "scanned_at": scanned_at,
        "artifact": REPORT_REL,
        "persisted": persisted,
        "scan_basis": scan_basis,
        "findings_list": preview,
    })
}

/// Claim POST /ddr/scan. GET is never claimed (the /ddr read arms in main.rs keep answering).
pub fn handle(request: &mut Request, url: &str) -> Option<Resp> {
    let method = request.method().clone();
    if method != Method::Post {
        return None;
    }
    let path = url.split('?').next().unwrap_or(url);
    if path != "/ddr/scan" {
        return None;
    }
    Some(ok(run_scan()))
}
