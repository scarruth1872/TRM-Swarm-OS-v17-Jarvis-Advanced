//! /ie/* — information extraction (tasks 531-540).
//!
//! Rule-based, span-returning, and explicit about vocabulary: entity linking uses the vocabularies that exist on
//! this machine (ESKAPE organisms and their UniProt proteome IDs) and reports `vocabulary: "local"`. It never says
//! UMLS, because there is no UMLS distribution here and inventing identifiers is the one thing a linker must not do.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use trm_uefi_kernel::information_extraction::{
    criteria_section, detect_negation, detect_uncertainty, extract_all, extract_events, extract_relations,
    extract_temporal, language_gate, link_entities, resolve_coref, EventKind, LanguageVerdict, Span,
};
use trm_uefi_kernel::k3_logic::sha256;
use trm_uefi_kernel::text_reading::split_sentences;

/// Bounded numeric parameter (same denial-of-service class as the reading routes).
fn bounded(body: &Value, key: &str, default: u64, max: u64) -> u64 {
    body.get(key).and_then(|v| v.as_u64()).unwrap_or(default).min(max)
}

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

/// Accept either a repo path or literal text; both carry provenance back to the caller.
fn source(body: &Value) -> Result<(String, String, String), String> {
    if let Some(t) = body.get("text").and_then(|v| v.as_str()) {
        let digest = sha256(t.as_bytes());
        let mut hex = String::with_capacity(64);
        for b in digest.iter() {
            hex.push_str(&format!("{b:02x}"));
        }
        return Ok((t.to_string(), "constructed".to_string(), hex));
    }
    let path = body.get("path").and_then(|v| v.as_str()).ok_or("provide 'path' or 'text'")?;
    read_confined(path)
}

/// Refuse extraction on non-English text rather than tokenising it.
fn gate(text: &str) -> Result<(), String> {
    match language_gate(text) {
        LanguageVerdict::English => Ok(()),
        LanguageVerdict::Refused(why) => Err(format!("REFUSED: {why}")),
    }
}

fn span_json(s: &Span) -> Value {
    json!({"start": s.start, "end": s.end, "text": s.text})
}

/// The local entity vocabulary: ESKAPE organisms and their UniProt proteome IDs.
fn local_vocab() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    if let Ok((text, _, _)) = read_confined("data/eskape_proteomes.json") {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(obj) = v.as_object() {
                for (organism, rec) in obj {
                    let upid = rec.get("upid").and_then(|u| u.as_str()).unwrap_or("");
                    out.push((organism.clone(), upid.to_string(), "organism".to_string()));
                    if let Some(sci) = rec.get("scientific_name").and_then(|s| s.as_str()) {
                        // the strain-qualified name is a longer surface form of the same entity
                        if let Some(base) = sci.split(" (strain").next() {
                            if base != organism {
                                out.push((base.to_string(), upid.to_string(), "organism".to_string()));
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// 531 — relations.
pub fn relations(body: &Value) -> Result<Value, String> {
    let (text, rel, digest) = source(body)?;
    gate(&text)?;
    let rs = extract_relations(&text);
    Ok(json!({
        "file": rel, "sha256": digest,
        "relations": rs.len(),
        "detail": rs.iter().map(|r| json!({
            "subject": span_json(&r.subject),
            "predicate": r.predicate,
            "object": span_json(&r.object),
            "sentence_start": r.sentence_start,
        })).collect::<Vec<_>>(),
        "semantics": "shallow subject-verb-object over a 24-verb lexicon; no dependency parse, so nested clauses are missed"
    }))
}

/// 532 — clinical events.
pub fn events(body: &Value) -> Result<Value, String> {
    let (text, rel, digest) = source(body)?;
    gate(&text)?;
    let ev = extract_events(&text);
    Ok(json!({
        "file": rel, "sha256": digest,
        "events": ev.len(),
        "by_kind": {
            "DIAGNOSIS": ev.iter().filter(|e| e.kind == EventKind::Diagnosis).count(),
            "MEDICATION": ev.iter().filter(|e| e.kind == EventKind::Medication).count(),
            "PROCEDURE": ev.iter().filter(|e| e.kind == EventKind::Procedure).count(),
            "MEASUREMENT": ev.iter().filter(|e| e.kind == EventKind::Measurement).count(),
            "ADVERSE_EVENT": ev.iter().filter(|e| e.kind == EventKind::AdverseEvent).count(),
            "CRITERION_INCLUSION": ev.iter().filter(|e| e.kind == EventKind::CriterionInclusion).count(),
            "CRITERION_EXCLUSION": ev.iter().filter(|e| e.kind == EventKind::CriterionExclusion).count(),
        },
        "detail": ev.iter().map(|e| json!({
            "kind": e.kind.as_str(),
            "trigger": span_json(&e.trigger),
            "arguments": e.arguments.iter().map(span_json).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "semantics": "22 clinical trigger phrases; each event's argument is the clause text it governs"
    }))
}

/// 535 — temporal expressions.
pub fn temporal(body: &Value) -> Result<Value, String> {
    let (text, rel, digest) = source(body)?;
    gate(&text)?;
    let ts = extract_temporal(&text);
    Ok(json!({
        "file": rel, "sha256": digest,
        "temporal": ts.len(),
        "detail": ts.iter().map(|t| json!({
            "span": span_json(&t.span),
            "kind": t.kind.as_str(),
            "normalized": t.normalized,
            "days": t.days,
        })).collect::<Vec<_>>(),
        "semantics": "ISO dates, month-only dates (first of month), durations in days; impossible dates are rejected"
    }))
}

/// 536 — negation, with the criteria-section ground truth when the text has one.
pub fn negation(body: &Value) -> Result<Value, String> {
    let (text, rel, digest) = source(body)?;
    gate(&text)?;
    let mut rows = Vec::new();
    let mut negated = 0usize;
    let mut total = 0usize;
    for s in split_sentences(&text) {
        total += 1;
        let sent = &text[s.start..s.end];
        let n = detect_negation(sent);
        if n.is_some() {
            negated += 1;
        }
        let section = criteria_section(&text, s.start);
        rows.push(json!({
            "sentence": sent,
            "start": s.start,
            "negated": n.is_some(),
            "cue": n.as_ref().map(|c| json!({"text": c.cue.text, "start": s.start + c.cue.start})),
            "scope": n.as_ref().map(|c| c.scope.text.clone()),
            "criteria_section": section.map(|k| k.as_str()),
        }));
    }
    Ok(json!({
        "file": rel, "sha256": digest,
        "sentences": total,
        "negated_sentences": negated,
        "detail": rows,
        "semantics": "16 NegEx-style cues with clause-scoped negation; scope stops at a comma, so a positive clause after one is not negated"
    }))
}

/// 537 — uncertainty.
pub fn uncertainty(body: &Value) -> Result<Value, String> {
    let (text, rel, digest) = source(body)?;
    gate(&text)?;
    let mut rows = Vec::new();
    for s in split_sentences(&text) {
        let sent = &text[s.start..s.end];
        if let Some(u) = detect_uncertainty(sent) {
            rows.push(json!({
                "sentence": sent,
                "start": s.start,
                "cue": u.cue.text,
                "scope": u.scope.text,
            }));
        }
    }
    Ok(json!({
        "file": rel, "sha256": digest,
        "hedged_sentences": rows.len(),
        "detail": rows,
        "semantics": "14 hedge cues with the same clause scoping as negation; a cue list that fires on every sentence would be visible here"
    }))
}

/// 534 — coreference (paragraph-local).
pub fn coref(body: &Value) -> Result<Value, String> {
    let (text, rel, digest) = source(body)?;
    gate(&text)?;
    let chains = resolve_coref(&text);
    Ok(json!({
        "file": rel, "sha256": digest,
        "chains": chains.len(),
        "detail": chains.iter().map(|c| json!({
            "antecedent": span_json(&c.antecedent),
            "mentions": c.mentions.iter().map(span_json).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "semantics": "nearest preceding content-word antecedent within the text; no cross-document resolution and no number/gender agreement"
    }))
}

/// 533 — entity linking against the LOCAL vocabularies. Explicitly not UMLS.
pub fn entities(body: &Value) -> Result<Value, String> {
    let (text, rel, digest) = source(body)?;
    gate(&text)?;
    let vocab = local_vocab();
    let ents = link_entities(&text, &vocab);
    Ok(json!({
        "file": rel, "sha256": digest,
        "vocabulary": "local",
        "vocabularies_used": ["data/eskape_proteomes.json (6 ESKAPE organisms -> UniProt proteome IDs)"],
        "umls": "ABSENT — no UMLS/SNOMED/RxNorm distribution exists on this machine and none was invented",
        "vocab_size": vocab.len(),
        "entities": ents.len(),
        "detail": ents.iter().map(|e| json!({
            "mention": span_json(&e.mention),
            "id": e.id,
            "kind": e.kind,
            "vocabulary": e.vocabulary,
        })).collect::<Vec<_>>(),
    }))
}

/// 539 — the language gate on its own.
pub fn language(body: &Value) -> Result<Value, String> {
    let (text, rel, digest) = source(body)?;
    match language_gate(&text) {
        LanguageVerdict::English => Ok(json!({
            "file": rel, "sha256": digest,
            "verdict": "ENGLISH",
            "extraction_allowed": true,
        })),
        LanguageVerdict::Refused(why) => Ok(json!({
            "file": rel, "sha256": digest,
            "verdict": "REFUSED",
            "extraction_allowed": false,
            "reason": why,
            "note": "no multilingual model is installed; refusing is the honest behaviour, tokenising would produce confident noise",
        })),
    }
}

/// 540 — everything, structured.
pub fn structured(body: &Value) -> Result<Value, String> {
    let (text, rel, digest) = source(body)?;
    gate(&text)?;
    let doc = extract_all(&text, &local_vocab());
    Ok(json!({
        "schema": "trm.ie.structured.v1",
        "file": rel,
        "sha256": digest,
        "sentences": doc.sentences,
        "relations": doc.relations.iter().map(|r| json!({
            "subject": span_json(&r.subject), "predicate": r.predicate, "object": span_json(&r.object)
        })).collect::<Vec<_>>(),
        "events": doc.events.iter().map(|e| json!({
            "kind": e.kind.as_str(), "trigger": span_json(&e.trigger),
            "arguments": e.arguments.iter().map(span_json).collect::<Vec<_>>()
        })).collect::<Vec<_>>(),
        "temporal": doc.temporals.iter().map(|t| json!({
            "span": span_json(&t.span), "kind": t.kind.as_str(), "normalized": t.normalized, "days": t.days
        })).collect::<Vec<_>>(),
        "negations": doc.negations.iter().map(|n| json!({"cue": span_json(&n.cue), "scope": span_json(&n.scope)})).collect::<Vec<_>>(),
        "uncertainties": doc.uncertainties.iter().map(|u| json!({"cue": span_json(&u.cue), "scope": span_json(&u.scope)})).collect::<Vec<_>>(),
        "coreferences": doc.corefs.iter().map(|c| json!({
            "antecedent": span_json(&c.antecedent),
            "mentions": c.mentions.iter().map(span_json).collect::<Vec<_>>()
        })).collect::<Vec<_>>(),
        "entities": doc.entities.iter().map(|e| json!({
            "mention": span_json(&e.mention), "id": e.id, "kind": e.kind, "vocabulary": e.vocabulary
        })).collect::<Vec<_>>(),
    }))
}

/// 538 — suite index.
pub fn suite() -> Value {
    let kernel_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src");
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let tasks: [(u32, &str, &str, &str, &str, &str); 10] = [
        (531, "relation extraction", "information_extraction.rs", "POST /ie/relations", "tools/ie_relation_verify.py", "WORKS (rule-based)"),
        (532, "event extraction", "information_extraction.rs", "POST /ie/events", "tools/ie_event_verify.py", "WORKS (real clinical text)"),
        (533, "entity linking", "information_extraction.rs", "POST /ie/entities", "tools/ie_entity_verify.py", "LOCAL VOCAB ONLY - UMLS ABSENT"),
        (534, "coreference resolution", "information_extraction.rs", "POST /ie/coref", "tools/ie_coref_verify.py", "PARTIAL (paragraph-local)"),
        (535, "temporal expressions", "information_extraction.rs", "POST /ie/temporal", "tools/ie_temporal_verify.py", "WORKS"),
        (536, "negation detection", "information_extraction.rs", "POST /ie/negation", "tools/ie_negation_verify.py", "WORKS (real ground truth)"),
        (537, "uncertainty detection", "information_extraction.rs", "POST /ie/uncertainty", "tools/ie_uncertainty_verify.py", "WORKS (cue-based)"),
        (538, "complete IE suite", "ie_routes.rs", "GET /ie/suite", "tools/ie_suite_verify.py", "WORKS"),
        (539, "multilingual extraction", "information_extraction.rs", "POST /ie/language", "tools/ie_multilingual_verify.py", "BLOCKED - gate refuses"),
        (540, "structured output", "ie_routes.rs", "POST /ie/structured", "tools/ie_suite_verify.py", "WORKS"),
    ];
    let entries: Vec<Value> = tasks.iter().map(|(id, name, module, endpoint, harness, status)| {
        let present = kernel_src.join(module).exists() || here.join(module).exists();
        json!({"id": id, "name": name, "module": module, "module_present": present,
               "endpoint": endpoint, "harness": harness, "capability": status,
               "state": if present { "present" } else { "ABSENT" }})
    }).collect();
    let present = entries.iter().filter(|e| e["module_present"] == json!(true)).count();
    json!({
        "suite": entries,
        "present_modules": present,
        "total_tasks": 10,
        "blocked": [
            {"task": 533, "reason": "UMLS/SNOMED/RxNorm absent from disk; local vocabularies used instead and labelled as such"},
            {"task": 539, "reason": "no multilingual model or corpus; the gate refuses non-English text rather than guessing"}
        ],
        "note": "module_present is a filesystem check; behaviour is proven by the named harness"
    })
}
