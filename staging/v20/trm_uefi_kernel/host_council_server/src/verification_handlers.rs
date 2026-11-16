// P1 — artifact provenance: the platform's answer to "is this artifact a computation's output?"
//
// The census is produced by scratch/prove_artifacts.py (provenance-by-producer: it traces each
// artifact to the script that writes it and reads that producer for an engine call or an RNG draw).
// The census on disk is a CLAIM, so this handler re-derives what it can at request time:
//   * the tallies are counted from the entries, never read from the census's own tally block;
//   * every receipt (producer file + line + token) is re-read from disk here, so a census that has
//     drifted from the producers reports reproducible < checked instead of a clean number;
//   * and with no census on disk the route refuses with the reason rather than reporting zero
//     artifacts as a pass.
//
// The verdict vocabulary is the tool's: COMPUTED (engine reached, fields not drawn), GENERATED
// (fields drawn from random/seeds), STATIC (producer shows neither -- not proven by that rule),
// UNTRACED (no producer on disk -- a finding, not a pass).

use serde_json::{json, Value};
use tiny_http::{Header, Method, Request, Response};

const CENSUS: &str = "swarm_v2_artifacts/artifact_provenance.json";
const CONTRACTS: &str = "swarm_v2_artifacts/artifact_contracts.json";

/// The contracts census, for the artifacts the provenance rule could not vouch for. The kernel does
/// not replay its verdicts either: it re-runs the range contract itself (see `range_violations`).
fn read_json_rel(rel: &str) -> Option<Value> {
    let p = crate::k3_retrieval::repo_relative(rel)?;
    serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()
}

fn age_of(rel: &str) -> Option<u64> {
    std::fs::metadata(crate::k3_retrieval::repo_relative(rel)?)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs())
}

/// Is this field a frequency (unit interval) or a percentage (0..100)? The same rule the tool states.
fn unit_bounds(key: &str) -> Option<(f64, f64)> {
    if key.starts_with("af_")
        || key.ends_with("_freq")
        || key.ends_with("_frequency")
        || key.ends_with("_rate")
        || key.ends_with("_fraction")
        || key == "freq"
        || key == "frequency"
        || key == "rate"
    {
        return Some((0.0, 1.0));
    }
    if key.ends_with("_pct") || key.ends_with("_percent") || key.ends_with("_percentage") || key == "pct" {
        return Some((0.0, 100.0));
    }
    None
}

/// Recompute the range contract from the artifact on disk: (values checked, values out of range).
/// Returns None when the artifact cannot be read -- which is reported as such, never as a pass.
fn range_violations(path: &str) -> Option<(u64, u64)> {
    let p = if path.len() > 1 && path.as_bytes()[1] == b':' {
        std::path::PathBuf::from(path)
    } else {
        crate::k3_retrieval::repo_relative(path)?
    };
    let data: Value = serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()?;
    let recs = data
        .as_array()
        .cloned()
        .or_else(|| {
            data.get("records")
                .or_else(|| data.get("results"))
                .or_else(|| data.get("rows"))
                .or_else(|| data.get("runs"))
                .or_else(|| data.get("items"))
                .and_then(|v| v.as_array())
                .cloned()
        })?;
    let (mut checked, mut bad) = (0u64, 0u64);
    for r in recs.iter().take(20_000) {
        if let Some(o) = r.as_object() {
            for (k, v) in o {
                if let (Some((lo, hi)), Some(x)) = (unit_bounds(k), v.as_f64()) {
                    checked += 1;
                    if x < lo || x > hi {
                        bad += 1;
                    }
                }
            }
        }
    }
    Some((checked, bad))
}

fn answer(body: Value, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body.to_string())
        .with_status_code(status)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
}

/// Re-read the producer line a verdict rests on. If the token the tool quoted is no longer on that
/// line, the verdict does not reproduce and is reported as such instead of being kept on trust.
fn receipt_holds(file: &str, line: u64, token: &str) -> bool {
    let p = match crate::k3_retrieval::repo_relative(file) {
        Some(p) => p,
        None => return false,
    };
    match std::fs::read_to_string(p) {
        Ok(txt) => txt
            .lines()
            .nth(line.saturating_sub(1) as usize)
            .map(|l| l.contains(token))
            .unwrap_or(false),
        Err(_) => false,
    }
}

fn q(url: &str, key: &str) -> Option<String> {
    url.split('?')
        .nth(1)?
        .split('&')
        .find_map(|kv| kv.strip_prefix(&format!("{key}=")).map(|v| v.to_string()))
}

/// `POST /verification/proof` — the WSL tree's proof-tree checker (`swarm-proof`), wired into the
/// kernel rather than re-implemented: the crate's own `ProofNode::verify` decides. Body:
/// `{"proof": {"conclusion": "Q", "rule": "ModusPonens", "premises": [...]}}`. A rejected proof is a
/// 200 with `sound: false` and the checker's own reason -- a refusal of the PROOF, not of the route.
fn proof_response(request: &mut Request, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    use std::io::Read;
    if request.method() != &Method::Post {
        return answer(
            json!({
                "status": "method_not_allowed", "error": "method_not_allowed", "route": path,
                "allowed": ["POST"],
                "message": "submitting a proof tree is a POST; this route implements POST only"
            }),
            405,
        );
    }
    let mut body = String::new();
    let _ = request.as_reader().take(262_144).read_to_string(&mut body);
    let parsed: Value = serde_json::from_str(&body).unwrap_or(json!({}));
    let node = match parsed.get("proof") {
        Some(v) => v,
        None => {
            return answer(
                json!({ "status": "error", "route": path, "error": "no_proof",
                        "detail": "POST {\"proof\": {\"conclusion\": \"Q\", \"rule\": \"ModusPonens\", \
                                   \"premises\": [{\"conclusion\": \"P\", \"rule\": \"Axiom\", \"premises\": []}]}}",
                        "rules": ["Axiom", "ModusPonens", "OrIntro", "AndElim"] }),
                400,
            )
        }
    };

    fn build(v: &Value, depth: usize) -> Result<swarm_proof::ProofNode, String> {
        if depth > 32 {
            return Err("proof tree deeper than 32 levels".to_string());
        }
        let conclusion = v
            .get("conclusion")
            .and_then(|c| c.as_str())
            .ok_or("every node needs a string `conclusion`")?;
        let rule = match v.get("rule").and_then(|r| r.as_str()).unwrap_or("Axiom") {
            "Axiom" => swarm_proof::Rule::Axiom,
            "ModusPonens" => swarm_proof::Rule::ModusPonens,
            "OrIntro" => swarm_proof::Rule::OrIntro,
            "AndElim" => swarm_proof::Rule::AndElim,
            other => return Err(format!("unknown rule `{other}`")),
        };
        let mut premises = Vec::new();
        if let Some(ps) = v.get("premises").and_then(|p| p.as_array()) {
            for p in ps {
                premises.push(build(p, depth + 1)?);
            }
        }
        Ok(swarm_proof::ProofNode { conclusion: conclusion.to_string(), rule, premises })
    }

    match build(node, 0) {
        Ok(tree) => {
            let verdict = swarm_proof::verify_proof(&tree);
            let sound = verdict.starts_with("PROOF VERIFIED");
            answer(
                json!({
                    "status": "ONLINE",
                    "route": path,
                    "sound": sound,
                    "verdict": verdict,
                    "engine": "swarm-proof (WSL crate trm-rust/swarm-proof, built in this workspace)",
                    "engine_note": "the crate's own ProofNode::verify decides; the kernel adds no rule and \
                                    rewrites no conclusion",
                    "node_count": count_nodes(&tree),
                    "rules": ["Axiom", "ModusPonens", "OrIntro", "AndElim"]
                }),
                200,
            )
        }
        Err(e) => answer(
            json!({ "status": "error", "route": path, "error": "malformed_proof", "detail": e,
                    "rules": ["Axiom", "ModusPonens", "OrIntro", "AndElim"] }),
            400,
        ),
    }
}

fn count_nodes(n: &swarm_proof::ProofNode) -> u64 {
    1 + n.premises.iter().map(count_nodes).sum::<u64>()
}

const QUARANTINE: &str = "swarm_v2_artifacts/fabrication_quarantine.json";

/// `GET /verification/fabrication` -- which artifacts claim results they cannot have.
///
/// The tally is DERIVED from the rows on every request instead of being read from the file's own
/// summary: a stored total can drift from the data it summarises, and this route exists to be the
/// surface that does not drift. `?verdict=X` filters the listed rows; counts always cover every row.
fn fabrication_response(url: &str, path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let file = match crate::k3_retrieval::repo_relative(QUARANTINE) {
        Some(p) => p,
        None => {
            return answer(
                json!({
                    "status": "absent",
                    "error": "quarantine_absent",
                    "message": format!(
                        "no fabrication quarantine on disk ({QUARANTINE}). Produce it with: py -3 \
                         scratch/p6_quarantine_fabrications.py. An absent quarantine is not a clean \
                         bill of health."
                    )
                }),
                501,
            )
        }
    };
    let doc: Value = match std::fs::read_to_string(&file)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
    {
        Some(v) => v,
        None => {
            return answer(
                json!({
                    "status": "unreadable",
                    "error": "quarantine_unreadable",
                    "message": format!("{QUARANTINE} exists but could not be parsed as JSON; re-run the prover")
                }),
                501,
            )
        }
    };

    let empty = vec![];
    let rows = doc.get("rows").and_then(|v| v.as_array()).unwrap_or(&empty);
    let want = q(url, "verdict");
    let limit: usize = q(url, "limit").and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut tally = serde_json::Map::new();
    let mut fabricated: Vec<Value> = Vec::new();
    let mut with_artifacts = 0u64;

    for r in rows {
        let verdict = r.get("verdict").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
        let n = tally.get(verdict).and_then(|v| v.as_u64()).unwrap_or(0);
        tally.insert(verdict.to_string(), json!(n + 1));

        let arts = r.get("artifacts").and_then(|v| v.as_array());
        if arts.map(|a| !a.is_empty()).unwrap_or(false) {
            with_artifacts += 1;
        }
        if verdict.starts_with("FABRICATED") && want.as_deref().map(|w| w == verdict).unwrap_or(true) {
            fabricated.push(json!({
                "n": r.get("n"),
                "domain": r.get("domain"),
                "gate": r.get("gate"),
                "verdict": verdict,
                "artifacts": arts.cloned().unwrap_or_default(),
                "producer": r.get("producer"),
                "evidence": r.get("evidence"),
            }));
        }
    }
    let count = fabricated.len();
    let shown: Vec<Value> = match limit {
        0 => fabricated,
        l if l < count => fabricated[..l].to_vec(),
        _ => fabricated,
    };

    answer(
        json!({
            "status": "ONLINE",
            "route": path,
            "source": QUARANTINE,
            "checked": rows.len(),
            "tasks_with_artifacts": with_artifacts,
            "verdicts": Value::Object(tally),
            "fabricated_count": count,
            "fabricated_shown": shown.len(),
            "fabricated": shown,
            "note": "counts are derived from the rows on every request; a stored total is not trusted"
        }),
        200,
    )
}

pub fn handle(request: &mut Request, url: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {
    let path = url.split('?').next().unwrap_or(url);
    if path == "/verification/proof" {
        return Some(proof_response(request, path));
    }
    if path == "/verification/fabrication" {
        // The reads own GET, answered here rather than by the catch-all (same rule as the census).
        if request.method() != &Method::Get {
            return Some(answer(
                json!({
                    "status": "method_not_allowed",
                    "error": "method_not_allowed",
                    "route": path,
                    "allowed": ["GET"],
                    "message": "the fabrication quarantine is a read; this route implements GET only"
                }),
                405,
            ));
        }
        return Some(fabrication_response(url, path));
    }
    if path != "/verification/artifacts" {
        return None;
    }
    // The reads own GET. Any other method is answered honestly here instead of falling through to the
    // catch-all, which would report 200 for a method this route does not implement.
    if request.method() != &Method::Get {
        return Some(answer(
            json!({
                "status": "method_not_allowed",
                "error": "method_not_allowed",
                "route": path,
                "allowed": ["GET"],
                "message": "artifact provenance is a read; this route implements GET only"
            }),
            405,
        ));
    }

    let file = match crate::k3_retrieval::repo_relative(CENSUS) {
        Some(p) => p,
        None => {
            return Some(answer(
                json!({
                    "status": "absent",
                    "error": "census_absent",
                    "message": format!(
                        "no artifact census on disk ({CENSUS}). Produce it with: py -3 \
                         scratch/prove_artifacts.py --limit N. The kernel will not report zero \
                         artifacts as a pass."
                    )
                }),
                501,
            ))
        }
    };
    let census: Value = match std::fs::read_to_string(&file)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
    {
        Some(v) => v,
        None => {
            return Some(answer(
                json!({
                    "status": "unreadable",
                    "error": "census_unreadable",
                    "message": format!("{CENSUS} exists but could not be parsed as JSON; re-run the prover")
                }),
                501,
            ))
        }
    };

    let empty = vec![];
    let entries = census.get("entries").and_then(|v| v.as_array()).unwrap_or(&empty);
    let mut tally = serde_json::Map::new();
    let mut checked = 0u64;
    let mut reproducible = 0u64;
    let mut broken: Vec<Value> = Vec::new();
    let want_class = q(url, "class");
    let limit: usize = q(url, "limit").and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut kept: Vec<Value> = Vec::new();

    for e in entries {
        let verdict = e.get("verdict").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
        let n = tally.get(verdict).and_then(|v| v.as_u64()).unwrap_or(0);
        tally.insert(verdict.to_string(), json!(n + 1));

        let rfile = e.get("receipt_file").and_then(|v| v.as_str());
        let rline = e.get("receipt_line").and_then(|v| v.as_u64());
        let rtoken = e.get("receipt_token").and_then(|v| v.as_str());
        let holds = match (rfile, rline, rtoken) {
            (Some(f), Some(l), Some(t)) if !t.is_empty() => {
                checked += 1;
                let ok = receipt_holds(f, l, t);
                if ok {
                    reproducible += 1;
                } else {
                    broken.push(json!({ "artifact": e.get("artifact"), "producer": f, "line": l }));
                }
                ok
            }
            _ => false,
        };

        if want_class.as_deref().map(|c| c == verdict).unwrap_or(true) {
            let mut row = e.clone();
            if let Some(o) = row.as_object_mut() {
                o.insert("receipt_reproducible".into(), json!(holds));
            }
            kept.push(row);
        }
    }
    // Every class is named even at zero: a tally that simply omits a class reads as "not measured"
    // when it means "measured, none found".
    for k in ["COMPUTED", "GENERATED", "STATIC", "UNTRACED"] {
        tally.entry(k.to_string()).or_insert(json!(0));
    }
    if limit > 0 && kept.len() > limit {
        kept.truncate(limit);
    }

    let age = std::fs::metadata(&file)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs());

    // ── Contracts: the second tier, for the artifacts provenance could not vouch for ──────────
    let contracts_block = match read_json_rel(CONTRACTS) {
        Some(c) => {
            let cempty = vec![];
            let centries = c.get("entries").and_then(|v| v.as_array()).unwrap_or(&cempty);
            let mut ctally = serde_json::Map::new();
            let (mut rechecked, mut agreeing, mut unreadable) = (0u64, 0u64, 0u64);
            let mut disagreed: Vec<Value> = Vec::new();
            for ce in centries {
                let v = ce.get("verdict").and_then(|x| x.as_str()).unwrap_or("UNKNOWN");
                let n = ctally.get(v).and_then(|x| x.as_u64()).unwrap_or(0);
                ctally.insert(v.to_string(), json!(n + 1));
                let range = ce
                    .get("contracts")
                    .and_then(|x| x.as_array())
                    .and_then(|a| {
                        a.iter().find(|c| {
                            c.get("name").and_then(|n| n.as_str()) == Some("frequency_range")
                        })
                    });
                if let Some(rc) = range {
                    let art = ce.get("artifact").and_then(|x| x.as_str()).unwrap_or("");
                    let claimed = rc.get("n_failed").and_then(|v| v.as_u64()).unwrap_or(0);
                    match range_violations(art) {
                        Some((_checked, bad)) => {
                            rechecked += 1;
                            if bad == claimed {
                                agreeing += 1;
                            } else {
                                disagreed.push(json!({ "artifact": art, "census_failed": claimed,
                                                       "kernel_failed": bad }));
                            }
                        }
                        None => unreadable += 1,
                    }
                }
            }
            for k in ["CONTRACTS_PASS", "CONTRACTS_FAIL", "NO_CONTRACT"] {
                ctally.entry(k.to_string()).or_insert(json!(0));
            }
            json!({
                "status": "ONLINE",
                "file": CONTRACTS,
                "age_seconds": age_of(CONTRACTS),
                "artifacts_examined": centries.len(),
                "verdicts": Value::Object(ctally),
                "verdicts_basis": "counted from the contracts census's entries at request time",
                "range_rechecked": rechecked,
                "range_agreeing": agreeing,
                "range_unreadable": unreadable,
                "range_disagreed": disagreed,
                "recheck_basis": "the kernel re-ran the range contract itself over each artifact's own \
                                  fields; a disagreement is reported, never averaged away",
                "vocabulary": {
                    "CONTRACTS_PASS": "every contract that applied to this artifact's shape passed -- \
                                       range consistency only, which is WEAK evidence and is not \
                                       evidence that a computation ran",
                    "CONTRACTS_FAIL": "a relation the artifact's own domain defines does not hold over \
                                       its own fields (e.g. mis_z independent of obs_mis/exp_mis)",
                    "NO_CONTRACT": "no contract in the library applies to this record shape -- still \
                                    unproven"
                }
            })
        }
        None => json!({
            "status": "absent",
            "file": CONTRACTS,
            "message": "no contracts census on disk; produce it with: py -3 \
                        scratch/verify_artifact_contracts.py"
        }),
    };

    Some(answer(
        json!({
            "status": "ONLINE",
            "census_file": CENSUS,
            "census_generated_at": census.get("generated_at"),
            "census_age_seconds": age,
            "producer": census.get("producer"),
            "scope": census.get("scope"),
            "artifacts_examined": entries.len(),
            "verdicts": Value::Object(tally),
            "verdicts_basis": "counted from the census's entries at request time, never read from the \
                               census's own tally block",
            "receipts_checked": checked,
            "receipts_reproducible": reproducible,
            "receipts_broken": broken,
            "receipts_basis": "each receipt (producer file + line + token) is re-read from disk here; \
                               reproducible < checked means the census has drifted from the producers \
                               and the affected verdicts need re-proving",
            "verdict_vocabulary": {
                "COMPUTED": "producer reaches a real engine and the record fields are not drawn",
                "GENERATED": "fields come from random/seeds/uuid4 with no engine call",
                "STATIC": "a producer exists but shows neither engine nor RNG -- not proven by this rule",
                "UNTRACED": "no producer on disk -- nothing vouches for the artifact"
            },
            "filter": { "class": want_class, "limit": if limit == 0 { json!(null) } else { json!(limit) } },
            "contracts": contracts_block,
            "entries": kept
        }),
        200,
    ))
}
