//! The artifact pipeline's two WRITE actions the kernel can perform without the Python engine:
//! review (approve/reject) and integrate.
//!
//! The Python routes (swarm_v2/routes/artifacts.py -> swarm_v2/core/artifact_pipeline.py) own the
//! full pending->approved->tested->integrated state machine and persist to
//! swarm_v2_memory/artifact_pipeline.json. The kernel does NOT touch that DB: its state file is the
//! compact index it already serves (swarm_v2_artifacts/artifact_pipeline_index.json, the export of
//! that DB with the heavy `content`/`security_result` fields stripped). This module reads the index,
//! mutates the one named record, recomputes `stats` from the mutated `artifacts[]`, and writes the
//! whole document back to the SAME file. The GET /artifacts read (artifact_handlers.rs) caches keyed
//! on (mtime, length), so the next read after a write re-parses and reflects the transition.
//!
//! Voltage doctrine: every field this module writes is derived, never fabricated. The status
//! transition is a real write to a real file; the `reviewed_at`/`integrated_at` timestamps are the
//! host clock read at request time (UTC); `integrated` is a real boolean flag set only after the
//! record actually transitions. The physical copy that integration implies in the Python pipeline is
//! ATTEMPTED (swarm_v2_artifacts/<filename> -> swarm_v2_integrated/<target_subdir>/<filename>) and its
//! outcome is reported — `copied: true/false` and a `integrated_path` that is null with a named
//! reason when the copy could not happen, never a path asserted for a file that does not exist.

use serde_json::{json, Value};
use std::io::Read;
use tiny_http::{Header, Method, Request, Response};

type Resp = Response<std::io::Cursor<Vec<u8>>>;

/// The kernel's own artifact state file — the compact index the /artifacts read already serves.
const INDEX_REL: &str = "swarm_v2_artifacts/artifact_pipeline_index.json";

fn ok(v: Value) -> Resp {
    Response::from_string(v.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn err(code: u16, v: Value) -> Resp {
    Response::from_string(v.to_string())
        .with_status_code(code)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

fn read_body(request: &mut Request, cap: usize) -> String {
    let mut body = String::new();
    let _ = request.as_reader().take(cap as u64).read_to_string(&mut body);
    body
}

/// UTC ISO-8601 timestamp (`YYYY-MM-DDTHH:MM:SS.sssZ`) derived from the host clock. The Python DB
/// stores `datetime.now().isoformat()` (local, no zone); this module writes UTC with an explicit `Z`
/// so the instant is unambiguous without a chrono dependency. The day arithmetic is Howard Hinnant's
/// `civil_from_days`, correct for every date at/after the epoch (all divisions are non-negative).
fn now_iso8601_utc() -> String {
    let dur = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    let days = (secs / 86_400) as i64;
    let sod = secs % 86_400;
    let (hh, mm, ss) = (sod / 3_600, (sod % 3_600) / 60, sod % 60);
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}.{millis:03}Z")
}

/// Recompute the `stats` object from the mutated `artifacts[]`, the same six counters the export
/// (swarm_v2_artifacts/export_artifact_index.py) derives. Derived from the array itself, never typed,
/// so a transition cannot leave a stale count.
fn recompute_stats(artifacts: &[Value]) -> Value {
    use std::collections::HashMap;
    let mut by_status: HashMap<String, u64> = HashMap::new();
    for a in artifacts {
        let s = a.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
        *by_status.entry(s).or_insert(0) += 1;
    }
    let n = artifacts.len() as u64;
    json!({
        "total": n,
        "pending": by_status.get("pending").copied().unwrap_or(0),
        "approved": by_status.get("approved").copied().unwrap_or(0),
        "rejected": by_status.get("rejected").copied().unwrap_or(0),
        "tested": by_status.get("tested").copied().unwrap_or(0),
        "test_failed": by_status.get("test_failed").copied().unwrap_or(0),
        "integrated": by_status.get("integrated").copied().unwrap_or(0),
    })
}

/// Read and parse the index. Returns (resolved path, document); a failure names what failed so the
/// caller can report it instead of inventing an empty index.
fn load_index() -> Result<(std::path::PathBuf, Value), String> {
    let path = crate::k3_retrieval::repo_relative(INDEX_REL)
        .ok_or_else(|| format!("{INDEX_REL} not found; run swarm_v2_artifacts/export_artifact_index.py"))?;
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("cannot read index: {e}"))?;
    let v: Value = serde_json::from_str(&raw).map_err(|e| format!("index is not JSON: {e}"))?;
    Ok((path, v))
}

/// Persist the index atomically: write a sibling `.tmp`, then rename over the original (the Python
/// `_save_sync` does the same, and a torn 12 MB write would corrupt the platform's state file).
fn persist_index(path: &std::path::Path, v: &Value) -> Result<(), String> {
    let s = serde_json::to_string(v).map_err(|e| format!("serialize failed: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &s).map_err(|e| format!("write failed: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename failed: {e}"))?;
    Ok(())
}

/// The physical copy Python's integrate performs, executed here so `integrated_path` names a file
/// that actually exists rather than one asserted into place. Returns
/// (repo-relative integrated_path, copied, note). The copy is best-effort and REPORTED, never
/// claimed: the index flag is the real, persisted write either way, and a source that is absent (or
/// a copy that fails) yields a null path with the reason in the note.
fn copy_artifact(filename: &str, target_subdir: &str) -> (Option<String>, bool, String) {
    let root = match crate::k3_retrieval::repo_root() {
        Some(r) => r,
        None => {
            return (
                None,
                false,
                "repo root unresolved, so no physical copy was attempted; the index flag is the real write"
                    .to_string(),
            )
        }
    };
    let src = root.join("swarm_v2_artifacts").join(filename);
    if !src.is_file() {
        return (
            None,
            false,
            format!("source \"{filename}\" is not present in swarm_v2_artifacts, so no physical copy was made; integrated_path left null with a reason"),
        );
    }
    let dest_dir = if target_subdir.is_empty() {
        root.join("swarm_v2_integrated")
    } else {
        root.join("swarm_v2_integrated").join(target_subdir)
    };
    let dest = dest_dir.join(filename);
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::copy(&src, &dest) {
        Ok(bytes) => {
            let rel = if target_subdir.is_empty() {
                format!("swarm_v2_integrated/{filename}")
            } else {
                format!("swarm_v2_integrated/{target_subdir}/{filename}")
            };
            let msg = format!("copied \"{filename}\" -> \"{rel}\" ({bytes} bytes)");
            (Some(rel), true, msg)
        }
        Err(e) => (
            None,
            false,
            format!("copy failed: {e}; the index flag is still the real, persisted write"),
        ),
    }
}

/// POST /artifacts/review — the review state machine, two legal transitions only.
/// Body: { "filename", "action": "approve"|"reject", "notes"?, "reviewer"? }.
/// Sets status to approved/rejected, records reviewer/timestamp/notes, recomputes stats, persists.
fn review(request: &mut Request) -> Resp {
    let parsed: Value = crate::chat_handlers::read_json_or_form(request, 65_536);
    let filename = parsed.get("filename").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let action = parsed.get("action").and_then(|v| v.as_str()).unwrap_or("").trim().to_ascii_lowercase();
    let notes = parsed.get("notes").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let reviewer = parsed.get("reviewer").and_then(|v| v.as_str()).unwrap_or("user").to_string();

    if filename.is_empty() {
        return err(400, json!({
            "status": "error", "route": "/artifacts/review", "error": "missing_filename",
            "detail": "POST {\"filename\": \"...\", \"action\": \"approve\"|\"reject\", \"notes\": \"...\", \"reviewer\": \"...\"}",
            "basis": "no filename was supplied, so no index record could be located",
        }));
    }
    if action != "approve" && action != "reject" {
        return err(400, json!({
            "status": "error", "route": "/artifacts/review", "error": "invalid_action",
            "detail": "action must be 'approve' or 'reject'",
            "basis": "the review state machine has exactly two legal transitions: approve -> approved, reject -> rejected",
        }));
    }

    let (path, mut idx) = match load_index() {
        Ok(v) => v,
        Err(e) => return err(500, json!({"status":"error","route":"/artifacts/review","error":"index_unavailable","detail":e})),
    };

    let pos = match idx.get("artifacts").and_then(|a| a.as_array()) {
        Some(arr) => match arr.iter().position(|r| {
            r.get("filename").and_then(|f| f.as_str()) == Some(filename.as_str())
        }) {
            Some(p) => p,
            None => return err(404, json!({
                "status": "error", "route": "/artifacts/review", "error": "not_found",
                "filename": filename, "detail": "no artifact with this filename in the index",
            })),
        },
        None => return err(500, json!({"status":"error","route":"/artifacts/review","error":"index_malformed","detail":"the index has no artifacts[] array"})),
    };

    let new_status = if action == "approve" { "approved" } else { "rejected" };
    let now = now_iso8601_utc();
    {
        let arr = match idx.get_mut("artifacts").and_then(|a| a.as_array_mut()) {
            Some(a) => a,
            None => return err(500, json!({"status":"error","route":"/artifacts/review","error":"index_malformed","detail":"artifacts is not an array"})),
        };
        let rec = &mut arr[pos];
        rec["status"] = json!(new_status);
        rec["reviewed_by"] = json!(reviewer);
        rec["reviewed_at"] = json!(now);
        rec["review_notes"] = json!(notes);
    }

    let stats = match idx.get("artifacts").and_then(|a| a.as_array()) {
        Some(arr) => recompute_stats(arr),
        None => recompute_stats(&[]),
    };
    idx["stats"] = stats;
    idx["generated_at"] = json!(now);

    if let Err(e) = persist_index(&path, &idx) {
        return err(500, json!({"status":"error","route":"/artifacts/review","error":"persist_failed","detail":e}));
    }

    let updated = idx
        .get("artifacts")
        .and_then(|a| a.as_array())
        .and_then(|a| a.get(pos))
        .cloned()
        .unwrap_or_else(|| json!({}));
    ok(json!({
        "status": "ok",
        "route": "/artifacts/review",
        "action": new_status,
        "filename": filename,
        "artifact": updated,
        "basis": format!("status transitioned to \"{new_status}\" by reviewer \"{reviewer}\" at {now} UTC; the record and recomputed stats were written back to {INDEX_REL} (the kernel's own state file)"),
    }))
}

/// POST /artifacts/integrate — mark an approved (or tested/already-integrated) artifact integrated.
/// Body: { "filename", "target_subdir"? }. Sets status=integrated, a real `integrated: true` flag,
/// `integrated_at`, and `integrated_path` (a real copy, or null + reason), recomputes stats, persists.
fn integrate(request: &mut Request) -> Resp {
    let parsed: Value = crate::chat_handlers::read_json_or_form(request, 65_536);
    let filename = parsed.get("filename").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let target_subdir = parsed.get("target_subdir").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();

    if filename.is_empty() {
        return err(400, json!({
            "status": "error", "route": "/artifacts/integrate", "error": "missing_filename",
            "detail": "POST {\"filename\": \"...\", \"target_subdir\": \"...\"}",
            "basis": "no filename was supplied, so no index record could be located",
        }));
    }
    // A write that creates files must not be handed a path that escapes the repo: ".." is never a
    // legal artifact key. Review has no filesystem side effect, so it needs no such guard.
    if filename.contains("..") || filename.starts_with('/') || filename.starts_with('\\') {
        return err(400, json!({
            "status": "error", "route": "/artifacts/integrate", "error": "invalid_filename",
            "filename": filename,
            "detail": "filename must be a repo-relative artifact key (no '..' or absolute path)",
            "basis": "integration copies the artifact into the repo, so an escaping path is refused",
        }));
    }

    let (path, mut idx) = match load_index() {
        Ok(v) => v,
        Err(e) => return err(500, json!({"status":"error","route":"/artifacts/integrate","error":"index_unavailable","detail":e})),
    };

    let (pos, cur_status) = match idx.get("artifacts").and_then(|a| a.as_array()) {
        Some(arr) => match arr.iter().position(|r| {
            r.get("filename").and_then(|f| f.as_str()) == Some(filename.as_str())
        }) {
            Some(p) => (
                p,
                arr[p].get("status").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            ),
            None => return err(404, json!({
                "status": "error", "route": "/artifacts/integrate", "error": "not_found",
                "filename": filename, "detail": "no artifact with this filename in the index",
            })),
        },
        None => return err(500, json!({"status":"error","route":"/artifacts/integrate","error":"index_malformed","detail":"the index has no artifacts[] array"})),
    };

    // The pipeline's own gate (artifact_pipeline.py::integrate): only tested/approved/integrated may
    // integrate. A pending or rejected artifact is refused rather than silently promoted.
    if !matches!(cur_status.as_str(), "approved" | "tested" | "integrated") {
        return err(400, json!({
            "status": "error", "route": "/artifacts/integrate", "error": "wrong_status",
            "filename": filename, "current_status": cur_status,
            "detail": "integration requires status approved|tested|integrated (the pipeline's own gate)",
            "basis": "the record's current status was read from the index; nothing was written",
        }));
    }

    let now = now_iso8601_utc();
    let (integrated_path, copied, copy_note) = copy_artifact(&filename, &target_subdir);
    {
        let arr = match idx.get_mut("artifacts").and_then(|a| a.as_array_mut()) {
            Some(a) => a,
            None => return err(500, json!({"status":"error","route":"/artifacts/integrate","error":"index_malformed","detail":"artifacts is not an array"})),
        };
        let rec = &mut arr[pos];
        rec["status"] = json!("integrated");
        rec["integrated"] = json!(true);
        rec["integrated_at"] = json!(now);
        match integrated_path {
            Some(p) => {
                rec["integrated_path"] = json!(p);
                rec["integrated_path_reason"] = Value::Null;
            }
            None => {
                rec["integrated_path"] = Value::Null;
                rec["integrated_path_reason"] = json!(copy_note);
            }
        }
    }

    let stats = match idx.get("artifacts").and_then(|a| a.as_array()) {
        Some(arr) => recompute_stats(arr),
        None => recompute_stats(&[]),
    };
    idx["stats"] = stats;
    idx["generated_at"] = json!(now);

    if let Err(e) = persist_index(&path, &idx) {
        return err(500, json!({"status":"error","route":"/artifacts/integrate","error":"persist_failed","detail":e}));
    }

    let updated = idx
        .get("artifacts")
        .and_then(|a| a.as_array())
        .and_then(|a| a.get(pos))
        .cloned()
        .unwrap_or_else(|| json!({}));
    ok(json!({
        "status": "ok",
        "route": "/artifacts/integrate",
        "filename": filename,
        "artifact": updated,
        "copied": copied,
        "basis": format!("record transitioned to \"integrated\" with a real integrated:true flag at {now} UTC; {copy_note}; the record and recomputed stats were written back to {INDEX_REL} (the kernel's own state file)"),
    }))
}

/// Entry point for the dispatch ladder. Claims exactly the two write routes this module owns and
/// nothing else: GET is never claimed (the /artifacts read lives in artifact_handlers.rs and a
/// prefix-blind guard would bury it), and the remaining 501'd actions (batch-review, batch-integrate,
/// test) still fall through to actions_handlers.
pub fn handle(request: &mut Request, url: &str) -> Option<Resp> {
    let method = request.method().clone();
    if method != Method::Post {
        return None;
    }
    let path = url.split('?').next().unwrap_or(url);
    match path {
        "/artifacts/review" => Some(review(request)),
        "/artifacts/integrate" => Some(integrate(request)),
        _ => None,
    }
}
