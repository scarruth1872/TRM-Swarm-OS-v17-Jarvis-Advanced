//! evolution_actions.rs — the REAL deterministic mutation cycle for /evolution.
//!
//! Replaces the 501 "engine_required" refusal for the POST mutation actions with an honest,
//! file-backed cycle that runs entirely in the kernel:
//!
//!   POST /evolution/mutate?target_file=X  -> read X, prepend a dated no-op comment header (a
//!                                             real, readable edit), append a new proposal
//!                                             (status "pending") to the real state file
//!                                             swarm_v2_artifacts/evolution_proposals.json, and
//!                                             return the proposal.
//!   POST /evolution/verify/{id}           -> set the proposal status to "verified" when its
//!                                             mutated_code is non-empty and distinct from its
//!                                             original_code (a real, measurable check), else
//!                                             "pending"; persist the change.
//!   POST /evolution/integrate/{id}        -> write the verified proposal's mutated_code to its
//!                                             target_file (a real edit on disk) and mark it
//!                                             "integrated"; persist the change.
//!
//! VOLTAGE DOCTRINE: derive/measure, never fabricate. The mutation is a real, readable edit; the
//! score is null until a verify measures it; every status change is written back to the real state
//! file. GET routes are NOT claimed (handle returns None) so the existing read endpoints in
//! state_handlers keep serving /evolution/genome, /evolution/proposals and /evolution/targets.

use serde_json::{json, Value};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response};

type Resp = Response<std::io::Cursor<Vec<u8>>>;

/// JSON 200 with the server-wide CORS header, matching the other handler modules.
fn ok(v: Value) -> Resp {
    Response::from_string(v.to_string())
        .with_status_code(200)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// The proposals state file, under swarm_v2_artifacts/ — the same file the Python mutation engine
/// and state_handlers read and write.
fn proposals_path() -> Option<PathBuf> {
    crate::k3_retrieval::repo_root().map(|r| {
        r.join("swarm_v2_artifacts")
            .join("evolution_proposals.json")
    })
}

/// Load the proposals file. A missing file is an empty proposals list (first-ever mutation creates
/// it); a present-but-unreadable file is an error, never silently clobbered.
fn load_proposals() -> Result<Value, String> {
    let path = proposals_path().ok_or("repo_root_unresolved")?;
    match std::fs::read_to_string(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({ "proposals": [] })),
        Err(e) => Err(format!("proposals_unreadable: {e}")),
        Ok(raw) => {
            serde_json::from_str::<Value>(&raw).map_err(|e| format!("proposals_parse_failed: {e}"))
        }
    }
}

/// Persist the proposals file after a status/contents change.
fn save_proposals(evo: &Value) -> bool {
    let Some(path) = proposals_path() else {
        return false;
    };
    match serde_json::to_string(evo) {
        Ok(s) => std::fs::write(&path, s).is_ok(),
        Err(_) => false,
    }
}

/// Index of the proposal with the given id, if present.
fn find_proposal_index(evo: &Value, id: &str) -> Option<usize> {
    evo.get("proposals")
        .and_then(|a| a.as_array())
        .and_then(|a| {
            a.iter()
                .position(|p| p.get("proposal_id").and_then(|v| v.as_str()) == Some(id))
        })
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Resolve a repo-relative target path (the shape analyze_codebase stores) against the repo root.
/// Path traversal ("..") is rejected by the caller before this runs.
fn resolve_target(target: &str) -> Option<PathBuf> {
    crate::k3_retrieval::repo_root().map(|r| r.join(target.replace('\\', "/")))
}

fn read_target(target: &str) -> Result<String, String> {
    let path = resolve_target(target).ok_or("repo_root_unresolved")?;
    std::fs::read_to_string(&path).map_err(|e| format!("target_read_failed: {e}"))
}

/// A line-comment prefix that is a true no-op for the target's language, so the mutated file still
/// parses. None = the language has no safe line comment (e.g. JSON), so no mutation is attempted.
fn line_comment_prefix(target: &str) -> Option<&'static str> {
    let ext = target.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "py" | "sh" | "bash" | "rb" | "pl" | "toml" | "yaml" | "yml" | "cfg" | "ini" | "conf"
        | "mk" | "r" | "ps1" | "txt" | "md" => Some("#"),
        "rs" | "c" | "h" | "cpp" | "cc" | "hpp" | "js" | "mjs" | "ts" | "tsx" | "jsx" | "go"
        | "java" | "cs" | "kt" | "swift" | "scala" | "zig" | "sql" => Some("//"),
        "lua" => Some("--"),
        _ => None,
    }
}

/// The deterministic safe mutation: a dated, id-tagged no-op header prepended to the file's
/// existing content. The original body is preserved byte-for-byte below it.
fn build_mutation(original: &str, target: &str, id: &str, secs: u64, prefix: &str) -> String {
    format!(
        "{prefix} ── TRM evolution {id} · target {target} · applied {secs} (unix) · no-op header ──\n{original}"
    )
}

/// Port of analyze_codebase's target heuristic (mirrors state_handlers::collect_evolution_targets):
/// the first swarm_v2/core .py file > 15 KB with no "__" in its name.
fn scan_dir(dir: &Path, root: &Path) -> Option<String> {
    let entries = std::fs::read_dir(dir).ok()?;
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if let Some(f) = scan_dir(&p, root) {
                return Some(f);
            }
        } else if p.extension().and_then(|x| x.to_str()) == Some("py") {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.contains("__") {
                if let Ok(meta) = std::fs::metadata(&p) {
                    if meta.len() > 15_000 {
                        let rel = p
                            .strip_prefix(root)
                            .map(|q| q.to_string_lossy().to_string())
                            .unwrap_or_else(|_| name.to_string());
                        return Some(rel);
                    }
                }
            }
        }
    }
    None
}

fn scan_first_target() -> Option<String> {
    let root = crate::k3_retrieval::repo_root()?;
    scan_dir(&root.join("swarm_v2").join("core"), &root)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex_val(b[i + 1]), hex_val(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

/// Extract a key from a `a=b&c=d` query string, percent-decoded.
fn query_param(query: Option<&str>, key: &str) -> Option<String> {
    let q = query?;
    for pair in q.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(percent_decode(v));
            }
        }
    }
    None
}

/// Fallback target_file source: a JSON body `{"target_file": "..."}` when the query string omits it.
fn body_target(request: &mut Request) -> Option<String> {
    let mut body = String::new();
    let _ = request.as_reader().take(65_536).read_to_string(&mut body);
    serde_json::from_str::<Value>(&body)
        .ok()?
        .get("target_file")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// POST /evolution/mutate?target_file=X — read the target, apply the deterministic safe mutation,
/// append a new "pending" proposal, persist, and return the proposal.
fn evolution_mutate(target: Option<String>) -> Value {
    let target = match target {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => match scan_first_target() {
            Some(t) => t,
            None => {
                return json!({
                    "status": "error",
                    "error": "no_target",
                    "detail": "target_file query/body param required, and no scannable swarm_v2/core target found",
                })
            }
        },
    };
    if target.split(['/', '\\']).any(|seg| seg == "..") {
        return json!({ "status": "error", "error": "unsafe_target", "target_file": target });
    }
    let Some(prefix) = line_comment_prefix(&target) else {
        return json!({
            "status": "error",
            "error": "unsupported_target_extension",
            "target_file": target,
            "detail": "no safe line-comment syntax for this file type; no mutation attempted",
        });
    };
    let original = match read_target(&target) {
        Ok(s) => s,
        Err(e) => {
            return json!({
                "status": "error",
                "error": "target_read_failed",
                "target_file": target,
                "detail": e,
            })
        }
    };
    let proposal_id = format!("ev-{}", unix_nanos());
    let secs = unix_secs();
    let mutated = build_mutation(&original, &target, &proposal_id, secs, prefix);
    let description =
        format!("Deterministic safe mutation: prepended a dated no-op comment header to {target}");
    let proposal = json!({
        "proposal_id": proposal_id,
        "target_file": target,
        "mutation_type": "comment_header",
        "status": "pending",
        "score": Value::Null,
        "score_basis": "unmeasured until verify",
        "timestamp": secs,
        "description": description,
        "original_code": original,
        "mutated_code": mutated,
    });
    let mut evo = match load_proposals() {
        Ok(e) => e,
        Err(e) => return json!({ "status": "error", "error": e }),
    };
    if let Some(arr) = evo.get_mut("proposals").and_then(|a| a.as_array_mut()) {
        arr.push(proposal.clone());
    } else {
        evo["proposals"] = json!([proposal.clone()]);
    }
    let persisted = save_proposals(&evo);
    json!({
        "status": "mutation_proposed",
        "proposal": proposal,
        "persisted": persisted,
        "persisted_to": "swarm_v2_artifacts/evolution_proposals.json",
    })
}

/// POST /evolution/verify/{id} — a real, measurable check: mutated_code is non-empty and distinct
/// from original_code. Pass -> "verified", fail -> "pending"; score is derived from that check and
/// the change is persisted.
fn evolution_verify(id: &str) -> Value {
    let mut evo = match load_proposals() {
        Ok(e) => e,
        Err(e) => return json!({ "status": "error", "error": e, "proposal_id": id }),
    };
    let Some(idx) = find_proposal_index(&evo, id) else {
        return json!({ "status": "error", "error": "proposal_not_found", "proposal_id": id });
    };
    let original = evo["proposals"][idx]
        .get("original_code")
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();
    let mutated = evo["proposals"][idx]
        .get("mutated_code")
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();
    let non_empty = !mutated.trim().is_empty();
    let distinct = mutated != original;
    let passed = non_empty && distinct;
    evo["proposals"][idx]["status"] = json!(if passed { "verified" } else { "pending" });
    evo["proposals"][idx]["score"] = json!(if passed { 1.0 } else { 0.0 });
    evo["proposals"][idx]["verify_basis"] = json!({
        "mutated_non_empty": non_empty,
        "distinct_from_original": distinct,
    });
    let persisted = save_proposals(&evo);
    json!({
        "status": if passed { "verified" } else { "pending" },
        "proposal_id": id,
        "checks": { "mutated_non_empty": non_empty, "distinct_from_original": distinct },
        "score": if passed { 1.0 } else { 0.0 },
        "persisted": persisted,
        "persisted_to": "swarm_v2_artifacts/evolution_proposals.json",
    })
}

/// POST /evolution/integrate/{id} — write the verified proposal's mutated_code to its target_file
/// (a real, persisted edit), then mark it "integrated". Gated on status "verified" (or the Python
/// engine's equivalent "passed").
fn evolution_integrate(id: &str) -> Value {
    let mut evo = match load_proposals() {
        Ok(e) => e,
        Err(e) => return json!({ "status": "error", "error": e, "proposal_id": id }),
    };
    let Some(idx) = find_proposal_index(&evo, id) else {
        return json!({ "status": "error", "error": "proposal_not_found", "proposal_id": id });
    };
    let status = evo["proposals"][idx]
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("");
    if status != "verified" && status != "passed" {
        return json!({
            "status": "error",
            "error": "not_verified",
            "proposal_id": id,
            "current_status": status,
            "detail": "only a proposal whose status is 'verified' (or Python 'passed') can be integrated; run /evolution/verify/{id} first",
        });
    }
    let target = evo["proposals"][idx]
        .get("target_file")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    let mutated = evo["proposals"][idx]
        .get("mutated_code")
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();
    if target.is_empty() {
        return json!({ "status": "error", "error": "no_target_file", "proposal_id": id });
    }
    if target.split(['/', '\\']).any(|seg| seg == "..") {
        return json!({
            "status": "error",
            "error": "unsafe_target",
            "proposal_id": id,
            "target_file": target,
        });
    }
    let Some(path) = resolve_target(&target) else {
        return json!({ "status": "error", "error": "repo_root_unresolved", "proposal_id": id });
    };
    if std::fs::write(&path, &mutated).is_err() {
        return json!({
            "status": "error",
            "error": "write_failed",
            "proposal_id": id,
            "target_file": target,
        });
    }
    evo["proposals"][idx]["status"] = json!("integrated");
    let persisted = save_proposals(&evo);
    json!({
        "status": "integrated",
        "proposal_id": id,
        "target_file": target,
        "bytes_written": mutated.len(),
        "persisted": persisted,
    })
}

/// Dispatch /evolution POST mutation actions. Returns None for anything else — including all GETs —
/// so the read endpoints in state_handlers (genome/proposals/targets) keep answering.
pub fn handle(request: &mut Request, url: &str) -> Option<Resp> {
    let method = request.method().clone();
    if method != Method::Post {
        return None;
    }
    let (path, query) = match url.split_once('?') {
        Some((p, q)) => (p.trim_end_matches('/'), Some(q)),
        None => (url.trim_end_matches('/'), None),
    };
    if path == "/evolution/mutate" {
        let target = query_param(query, "target_file").or_else(|| body_target(request));
        return Some(ok(evolution_mutate(target)));
    }
    if let Some(rest) = path.strip_prefix("/evolution/verify/") {
        let id = rest.trim_end_matches('/');
        if !id.is_empty() {
            return Some(ok(evolution_verify(id)));
        }
    }
    if let Some(rest) = path.strip_prefix("/evolution/integrate/") {
        let id = rest.trim_end_matches('/');
        if !id.is_empty() {
            return Some(ok(evolution_integrate(id)));
        }
    }
    None
}
