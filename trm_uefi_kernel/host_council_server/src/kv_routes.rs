//! /kv/* — knowledge verification (tasks 591-600).
//!
//! These endpoints check the platform's own outputs. The posture throughout: a verdict is only useful if the
//! evidence for it is returned alongside, and every one of them has a case it refuses.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use trm_uefi_kernel::claims::{contradicts, verify};
use trm_uefi_kernel::k3_logic::sha256;
use trm_uefi_kernel::knowledge_verification::{
    classify, find_retractions, quantify, resolve, retraction_targets, review_document, review_verdicts,
    verify_source, Position, Signals, SourceKind,
};
use trm_uefi_kernel::text_reading::{quantities, split_sentences};

use crate::evidence_index;

fn repo_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..6 {
        if dir.join("data").is_dir() || dir.join(".git").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn read_confined(path: &str) -> Result<(String, String, String), String> {
    let root = repo_root().ok_or("cannot locate the repository root")?;
    let raw = Path::new(path);
    let joined = if raw.is_absolute() { raw.to_path_buf() } else { root.join(raw) };
    let canon = joined.canonicalize().map_err(|e| format!("cannot open '{path}': {e}"))?;
    let root_canon = root.canonicalize().unwrap_or(root);
    if !canon.starts_with(&root_canon) {
        return Err(format!("'{path}' resolves outside the repository and will not be read"));
    }
    let text = std::fs::read_to_string(&canon).map_err(|e| format!("cannot read '{}': {e}", canon.display()))?;
    let rel = canon
        .strip_prefix(&root_canon)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string());
    let digest = sha256(text.as_bytes());
    let mut hex = String::with_capacity(64);
    for b in digest.iter() {
        hex.push_str(&format!("{b:02x}"));
    }
    Ok((text, rel, hex))
}

/// Scores every sentence of the retrieved chunks and returns the best evidence with its signals.
fn best_evidence(claim: &str, k: usize) -> (Vec<String>, Option<(String, String, f64)>) {
    let hits = evidence_index::search(claim, k);
    let mut sentences: Vec<String> = Vec::new();
    let mut best: Option<(String, String, f64)> = None; // (source, sentence, score)
    for h in &hits {
        for s in split_sentences(&h.text) {
            let c = verify(claim, &[s.text.clone()]);
            if c.coverage > 0.0 && best.as_ref().map(|(_, _, b)| c.coverage > *b).unwrap_or(true) {
                best = Some((h.source.clone(), s.text.clone(), c.coverage));
            }
            sentences.push(s.text.clone());
        }
    }
    (sentences, best)
}

/// 591 — fact check a claim: verdict, evidence, confidence, and a ledger cross-check when the claim is about a
/// shard whose measured count is recorded.
pub fn factcheck(body: &Value) -> Result<Value, String> {
    let claim = body.get("claim").and_then(|v| v.as_str()).ok_or("'claim' must be a string")?;
    let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let (sentences, best) = best_evidence(claim, k);
    let check = verify(claim, &sentences);

    let claim_nums: Vec<f64> = quantities(claim).iter().map(|q| q.value).collect();
    let (ev_kind, numbers_agree, numbers_conflict) = match &best {
        Some((src, sentence, _)) => {
            let ev_nums: Vec<f64> = quantities(sentence).iter().map(|q| q.value).collect();
            let agree = claim_nums.iter().all(|c| ev_nums.iter().any(|e| (c - e).abs() < 1e-9));
            let conflict = !claim_nums.is_empty() && !ev_nums.is_empty() && !agree;
            (classify(src), agree, conflict)
        }
        None => (SourceKind::Secondary, false, false),
    };
    let conf = quantify(&Signals {
        coverage: check.coverage,
        numbers_agree,
        numbers_conflict,
        evidence_kind: ev_kind,
        no_evidence: check.evidence_index.is_none(),
    });

    Ok(json!({
        "claim": claim,
        "verdict": check.verdict.as_str(),
        "reason": check.reason,
        "coverage": check.coverage,
        "confidence": conf.p_support.map(|p| json!(p)).unwrap_or(json!(null)),
        "confidence_parts": conf.parts.iter().map(|(n, v)| json!({"signal": n, "contribution": v})).collect::<Vec<_>>(),
        "abstention": conf.abstention,
        "evidence": best.map(|(src, s, _)| json!({"source": src, "sentence": s, "source_kind": classify(&src).as_str()})),
        "evidence_considered": sentences.len(),
        "semantics": "SUPPORTED/REFUTED/UNKNOWN from task 585; confidence is a stated policy model whose calibration is measured by tools/kv_factcheck_verify.py, not an authority"
    }))
}

/// 592 — knowledge consistency: cross-document contradictions plus the ledger's claimed-vs-measured gaps.
pub fn consistency(body: &Value) -> Result<Value, String> {
    let paths: Vec<String> = body
        .get("paths")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .ok_or("'paths' must be an array of file paths")?;
    let max_pairs = body.get("max_pairs").and_then(|v| v.as_u64()).unwrap_or(50_000) as usize;

    struct Item { file: String, start: usize, text: String }
    let mut items: Vec<Item> = Vec::new();
    for p in &paths {
        let (text, rel, _) = read_confined(p)?;
        for s in split_sentences(&text) {
            if s.text.len() >= 50 {
                items.push(Item { file: rel.clone(), start: s.start, text: s.text });
            }
        }
    }
    let mut conflicts = Vec::new();
    let mut compared = 0usize;
    'outer: for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            if items[i].file == items[j].file {
                continue; // internal consistency within one document is checked by 597, not here
            }
            compared += 1;
            if compared > max_pairs {
                break 'outer;
            }
            if let Some(c) = contradicts(&items[i].text, &items[j].text) {
                conflicts.push(json!({
                    "kind": c.kind.as_str(), "detail": c.detail,
                    "a": {"file": items[i].file, "start": items[i].start, "text": items[i].text},
                    "b": {"file": items[j].file, "start": items[j].start, "text": items[j].text},
                }));
            }
        }
    }

    // the corpus ledger records claimed vs measured per shard; a gap is a known inconsistency, not a contradiction
    let mut ledger_gaps = Vec::new();
    if let Ok((text, _, _)) = read_confined("data/corpus_ledger_measured.json") {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(shards) = v.get("shards").and_then(|s| s.as_object()) {
                for (name, rec) in shards {
                    let claimed = rec.get("claimed").and_then(|x| x.as_u64());
                    let measured = rec.get("measured").and_then(|x| x.as_u64());
                    if let (Some(c), Some(m)) = (claimed, measured) {
                        if c != m {
                            ledger_gaps.push(json!({
                                "shard": name, "claimed": c, "measured": m,
                                "delta": m as i64 - c as i64,
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(json!({
        "files": paths.len(),
        "sentences": items.len(),
        "pairs_compared": compared.min(max_pairs),
        "contradictions": conflicts.len(),
        "detail": conflicts,
        "ledger_gaps": ledger_gaps,
        "semantics": "contradictions require shared subject coverage both ways plus a numeric or polarity conflict; ledger gaps are claimed-vs-measured differences already recorded in data/corpus_ledger_measured.json"
    }))
}

/// 593 — source verification: hash against a recorded value, and the source kind it is treated as.
pub fn source(body: &Value) -> Result<Value, String> {
    let path = body.get("path").and_then(|v| v.as_str()).ok_or("'path' must be a string")?;
    let recorded = body.get("recorded_sha").and_then(|v| v.as_str());
    let (text, rel, actual) = read_confined(path)?;
    let v = verify_source(&rel, recorded, &actual);
    Ok(json!({
        "file": rel,
        "bytes": text.len(),
        "source_kind": v.kind.as_str(),
        "kind_weight": v.kind.weight(),
        "hash_matches": v.hash_matches,
        "recorded_sha": v.recorded_sha,
        "actual_sha": v.actual_sha,
        "detail": v.detail,
        "semantics": "an unrecorded source is NOT verified: a hash nobody published attests to nothing"
    }))
}

/// 594 — conflict resolution between two positions.
pub fn conflict(body: &Value) -> Result<Value, String> {
    let parse = |key: &str| -> Result<Position, String> {
        let o = body.get(key).ok_or_else(|| format!("'{key}' must be an object"))?;
        let label = o.get("label").and_then(|v| v.as_str()).ok_or_else(|| format!("{key}.label required"))?;
        let kind_s = o.get("kind").and_then(|v| v.as_str()).unwrap_or("secondary");
        let kind = match kind_s.to_uppercase().as_str() {
            "PRIMARY" => SourceKind::Primary,
            "DERIVED" => SourceKind::Derived,
            _ => SourceKind::Secondary,
        };
        Ok(Position {
            label: label.to_string(),
            kind,
            measured: o.get("measured").and_then(|v| v.as_bool()).unwrap_or(false),
            date: o.get("date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            value: o.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
    };
    let a = parse("a")?;
    let b = parse("b")?;
    let r = resolve(&a, &b);
    Ok(json!({
        "winner": r.winner,
        "rule": r.rule.as_str(),
        "losers": r.losers,
        "rationale": r.rationale,
        "semantics": "precedence: measured over claimed, then primary over secondary, then newer over older; a true tie returns UNRESOLVED rather than an arbitrary winner"
    }))
}

/// 595 — uncertainty quantification for a claim, with the components that make up the number.
pub fn uncertainty(body: &Value) -> Result<Value, String> {
    let claim = body.get("claim").and_then(|v| v.as_str()).ok_or("'claim' must be a string")?;
    let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let (sentences, best) = best_evidence(claim, k);
    let check = verify(claim, &sentences);
    let claim_nums: Vec<f64> = quantities(claim).iter().map(|q| q.value).collect();
    let (kind, agree, conflict) = match &best {
        Some((src, sentence, _)) => {
            let ev: Vec<f64> = quantities(sentence).iter().map(|q| q.value).collect();
            let a = claim_nums.iter().all(|c| ev.iter().any(|e| (c - e).abs() < 1e-9));
            (classify(src), a, !claim_nums.is_empty() && !ev.is_empty() && !a)
        }
        None => (SourceKind::Secondary, false, false),
    };
    let c = quantify(&Signals {
        coverage: check.coverage,
        numbers_agree: agree,
        numbers_conflict: conflict,
        evidence_kind: kind,
        no_evidence: check.evidence_index.is_none(),
    });
    Ok(json!({
        "claim": claim,
        "p_support": c.p_support.map(|p| json!(p)).unwrap_or(json!(null)),
        "abstention": c.abstention,
        "parts": c.parts.iter().map(|(n, v)| json!({"signal": n, "logodds": v})).collect::<Vec<_>>(),
        "verdict": check.verdict.as_str(),
        "semantics": "additive log-odds model with stated weights, squashed by a no_std exp approximation; calibration is measured externally (Brier + reliability) and published by the harness"
    }))
}

/// 597 — peer review: mechanical checklist over a document.
pub fn peerreview(body: &Value) -> Result<Value, String> {
    // `text` reviews constructed prose so the harness can prove the check FIRES on a line that should fail; a
    // check that is only ever run on documents written to pass it is not being tested.
    if let Some(t) = body.get("text").and_then(|v| v.as_str()) {
        let findings = review_document(t);
        let (blocking, warnings, notes) = review_verdicts(&findings);
        return Ok(json!({
            "file": "constructed",
            "blocking": blocking, "warnings": warnings, "notes": notes,
            "passes": blocking == 0,
            "findings": findings.iter().map(|f| json!({
                "severity": f.severity.as_str(), "rule": f.rule, "line": f.line, "detail": f.detail
            })).collect::<Vec<_>>(),
            "semantics": "constructed text: reviewed by the same four rules as a document"
        }));
    }
    let path = body.get("path").and_then(|v| v.as_str()).ok_or("provide 'path' or 'text'")?;
    let (text, rel, digest) = read_confined(path)?;
    let findings = review_document(&text);
    let (blocking, warnings, notes) = review_verdicts(&findings);
    Ok(json!({
        "file": rel,
        "sha256": digest,
        "blocking": blocking,
        "warnings": warnings,
        "notes": notes,
        "passes": blocking == 0,
        "findings": findings.iter().map(|f| json!({
            "severity": f.severity.as_str(), "rule": f.rule, "line": f.line, "detail": f.detail
        })).collect::<Vec<_>>(),
        "semantics": "four textual rules; BLOCKING means a result is asserted with no path or offset on the line. These are stated limits, not understanding"
    }))
}

/// 598 — reproducibility: do the artifacts a document points at still exist, and do its numbers still resolve?
pub fn reproducibility(body: &Value) -> Result<Value, String> {
    let path = body.get("path").and_then(|v| v.as_str()).ok_or("'path' must be a string")?;
    let (text, rel, digest) = read_confined(path)?;
    let root = repo_root().ok_or("no repo root")?;

    let mut referenced: Vec<String> = Vec::new();
    for tok in text.split(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '`' || c == ',' || c == ':') {
        let t = tok.trim_matches(|c: char| c.is_ascii_punctuation() && c != '_' && c != '-');
        if t.contains('/')
            && (t.ends_with(".py") || t.ends_with(".rs") || t.ends_with(".json") || t.ends_with(".md")
                || t.ends_with(".txt"))
        {
            referenced.push(t.to_string());
        }
    }
    referenced.sort();
    referenced.dedup();

    let mut missing = Vec::new();
    let mut present = 0usize;
    for r in &referenced {
        if root.join(r).exists() {
            present += 1;
        } else {
            missing.push(r.clone());
        }
    }

    Ok(json!({
        "file": rel,
        "sha256": digest,
        "referenced_artifacts": referenced.len(),
        "present": present,
        "missing": missing,
        "reproducible": missing.is_empty(),
        "semantics": "checks that every file path the document names still exists; it does NOT re-run the measurements, and that limit is the honest boundary of this check"
    }))
}

/// 599 — retraction detection: markers in a document, and which claims they cover.
pub fn retraction(body: &Value) -> Result<Value, String> {
    let claim = body.get("claim").and_then(|v| v.as_str());
    if let Some(t) = body.get("text").and_then(|v| v.as_str()) {
        let found = find_retractions(t);
        let covered = claim.map(|c| found.iter().any(|r| retraction_targets(r, c)));
        return Ok(json!({
            "file": "constructed",
            "retractions": found.len(),
            "detail": found.iter().map(|r| json!({
                "line": r.marker_line, "marker": r.marker,
                "body": r.body.chars().take(300).collect::<String>()
            })).collect::<Vec<_>>(),
            "claim_tested": claim,
            "claim_retracted": covered,
            "semantics": "constructed text: same marker detection as a document"
        }));
    }
    let path = body.get("path").and_then(|v| v.as_str()).ok_or("provide 'path' or 'text'")?;
    let (text, rel, digest) = read_confined(path)?;
    let found = find_retractions(&text);
    let covered = claim.map(|c| found.iter().any(|r| retraction_targets(r, c)));
    Ok(json!({
        "file": rel,
        "sha256": digest,
        "retractions": found.len(),
        "detail": found.iter().map(|r| json!({
            "line": r.marker_line, "marker": r.marker,
            "body": r.body.chars().take(300).collect::<String>()
        })).collect::<Vec<_>>(),
        "claim_tested": claim,
        "claim_retracted": covered,
        "semantics": "a retraction covers a claim only when its body names that claim; a document-level correction does not clear every statement in the document"
    }))
}

/// 600 — the comprehensive report: every check above, run over a document set.
pub fn report(body: &Value) -> Result<Value, String> {
    let paths: Vec<String> = body
        .get("paths")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec!["docs/MACHINE-READING-RESEARCH.md".to_string()]);

    let mut rows = Vec::new();
    let mut total_blocking = 0usize;
    let mut total_retractions = 0usize;
    let mut missing_artifacts = 0usize;
    for p in &paths {
        let Ok((text, rel, digest)) = read_confined(p) else {
            rows.push(json!({"file": p, "error": "unreadable"}));
            continue;
        };
        let findings = review_document(&text);
        let (b, w, n) = review_verdicts(&findings);
        let retr = find_retractions(&text);
        let root = repo_root().unwrap_or_default();
        let mut referenced = 0usize;
        let mut missing = 0usize;
        for tok in text.split(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '`' || c == ',' || c == ':') {
            let t = tok.trim_matches(|c: char| c.is_ascii_punctuation() && c != '_' && c != '-');
            if t.contains('/') && (t.ends_with(".py") || t.ends_with(".rs") || t.ends_with(".json") || t.ends_with(".md") || t.ends_with(".txt")) {
                referenced += 1;
                if !root.join(t).exists() {
                    missing += 1;
                }
            }
        }
        total_blocking += b;
        total_retractions += retr.len();
        missing_artifacts += missing;
        rows.push(json!({
            "file": rel, "sha256": digest,
            "blocking": b, "warnings": w, "notes": n,
            "retractions": retr.len(),
            "referenced_artifacts": referenced, "missing_artifacts": missing,
            "passes_review": b == 0,
        }));
    }
    Ok(json!({
        "documents": rows.len(),
        "rows": rows,
        "totals": {
            "blocking": total_blocking,
            "retractions": total_retractions,
            "missing_artifacts": missing_artifacts,
        },
        "verdict": if total_blocking == 0 && missing_artifacts == 0 { "CLEAN" } else { "FINDINGS" },
        "semantics": "aggregate of the review and reproducibility checks; a CLEAN verdict means no blocking findings and no missing artifacts, not that the claims are true"
    }))
}

/// 596 — the suite index, with module presence checked on disk.
pub fn suite() -> Value {
    let kernel_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src");
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let tasks: [(u32, &str, &str, &str, &str); 10] = [
        (591, "fact checking", "claims.rs", "POST /kv/factcheck", "tools/kv_factcheck_verify.py"),
        (592, "knowledge consistency", "claims.rs", "POST /kv/consistency", "tools/kv_consistency_verify.py"),
        (593, "source verification", "knowledge_verification.rs", "POST /kv/source", "tools/kv_source_verify.py"),
        (594, "conflict resolution", "knowledge_verification.rs", "POST /kv/conflict", "tools/kv_conflict_verify.py"),
        (595, "uncertainty quantification", "knowledge_verification.rs", "POST /kv/uncertainty", "tools/kv_factcheck_verify.py"),
        (596, "complete verification suite", "kv_routes.rs", "GET /kv/suite", "tools/kv_suite_verify.py"),
        (597, "peer review automation", "knowledge_verification.rs", "POST /kv/peerreview", "tools/kv_peerreview_verify.py"),
        (598, "reproducibility checking", "kv_routes.rs", "POST /kv/reproducibility", "tools/kv_reproducibility_verify.py"),
        (599, "retraction detection", "knowledge_verification.rs", "POST /kv/retraction", "tools/kv_retraction_verify.py"),
        (600, "comprehensive verification report", "kv_routes.rs", "POST /kv/report", "tools/kv_suite_verify.py"),
    ];
    let entries: Vec<Value> = tasks.iter().map(|(id, name, module, endpoint, harness)| {
        let present = kernel_src.join(module).exists() || here.join(module).exists();
        json!({"id": id, "name": name, "module": module, "module_present": present,
               "endpoint": endpoint, "harness": harness,
               "status": if present { "present" } else { "ABSENT" }})
    }).collect();
    let present = entries.iter().filter(|e| e["module_present"] == json!(true)).count();
    json!({
        "suite": entries,
        "present_modules": present,
        "total_tasks": 10,
        "note": "module_present is a filesystem check at request time; behaviour is proven by the named harness"
    })
}
