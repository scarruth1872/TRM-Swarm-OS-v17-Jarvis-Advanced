//! Multi-store recall router — routes a query BY KIND across the platform's three knowledge tiers.
//!
//! The council (MOTION-049/051) ratified: main fabric (2,180 docs) is the general recall surface,
//! the ChEMBL co-processor (2,897,805 compounds) is the molecule surface, and the structured SQL
//! store (~3.9M rows) is host-side. This module classifies a query and dispatches to the right
//! store instead of dumping everything into one index:
//!
//!   * `molecule`   -> `chembl_lookup` (by ChEMBL ID, or SMILES/term match) over the co-processor
//!   * `structured` -> an honest pointer to the host-side SQL store (this kernel has no SQLite)
//!   * `general`    -> the existing `/k3/retrieve` engine (fabric + distilled model stores)
//!
//! The molecule route is lexical (ID / term), not a full 768-dim vector scan: the ID path stops
//! early because the sidecar is sorted by ChEMBL ID, and the term path is capped. A packed-domain
//! vector scan over the 2.9M co-processor is a separate follow-up, stated here rather than faked.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Classify a query into a recall store. Deterministic keyword/substring routing, no model call.
pub fn classify(query: &str) -> &'static str {
    let q = query.to_ascii_lowercase();
    // ChEMBL ID or molecular vocabulary -> the molecule store.
    if q.contains("chembl")
        || q.contains("smiles")
        || q.contains("molecule")
        || q.contains("compound")
        || q.contains("ligand")
        || q.contains("ic50")
        || q.contains("kinase")
        || q.contains("inhibitor")
        || q.contains("molecular weight")
    {
        return "molecule";
    }
    // Chemical-biology vocabulary -> the bioassay store (tox21 + binding). "toxicity"/"toxic" are
    // the natural-query forms of the tox21 store (MOTION-C1) — a query like "toxicity of aspirin"
    // is a bioassay question, not a general-prose one.
    for kw in ["tox21", "toxicity", "toxic", "binding affinity", "assay", "target", "protein", "gene", "activity class"] {
        if q.contains(kw) {
            return "bioassay";
        }
    }
    // Clinical vocabulary -> the clinical catalog lookup (lexical, not vector).
    if q.contains("clinical") {
        return "clinical";
    }
    // Structured/medical vocabulary -> the structured text store (ethics + prose).
    for kw in ["ethics", "side effect", "indication"] {
        if q.contains(kw) {
            return "structured";
        }
    }
    // Clinical measurement exact match (case-insensitive alias or code) -> the catalog lookup.
    if is_clinical_measurement(&q) {
        return "clinical";
    }
    "general"
}

/// Extract the JSON string value for `field` out of one sidecar line, resolving `\n` etc.
fn field(line: &str, field_name: &str) -> Option<String> {
    let pat = format!("\"{}\"", field_name);
    let pos = line.find(&pat)?;
    let mut i = pos + pat.len();
    while i < line.len() && (line.as_bytes()[i] == b' ' || line.as_bytes()[i] == b':') {
        i += 1;
    }
    if i >= line.len() || line.as_bytes()[i] != b'"' {
        return None;
    }
    i += 1;
    let mut out = String::new();
    let bytes = line.as_bytes();
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'"' {
            return Some(out);
        }
        if c == b'\\' && i + 1 < bytes.len() {
            let e = bytes[i + 1];
            out.push(match e {
                b'n' => '\n',
                b't' => '\t',
                b'r' => ' ',
                b'"' => '"',
                b'\\' => '\\',
                b'/' => '/',
                other => other as char,
            });
            i += 2;
            continue;
        }
        if (0x20..0x7f).contains(&c) {
            out.push(c as char);
        }
        i += 1;
    }
    Some(out)
}

/// Extract a `CHEMBL<digits>` id from the query, if one is present.
fn chembl_id(query: &str) -> Option<String> {
    let q = query.to_ascii_uppercase();
    let pos = q.find("CHEMBL")?;
    let mut end = pos + 6;
    while end < q.len() && q.as_bytes()[end].is_ascii_digit() {
        end += 1;
    }
    if end == pos + 6 {
        return None; // "CHEMBL" with no digits is not an id
    }
    Some(q[pos..end].to_string())
}

fn sidecar_path() -> Option<std::path::PathBuf> {
    crate::k3_retrieval::repo_root().map(|r| r.join("swarm_v2_chembl_fabric.text.jsonl"))
}

/// ChEMBL-id -> byte-offset index for the chembl text sidecar (exact-ID fast path). Derived once by
/// scratch/build_chembl_id_offset_index.py from the existing file. Key is the NUMERIC chembl id
/// (u32), value is the record's start byte offset in swarm_v2_chembl_fabric.text.jsonl. Loaded
/// lazily over file IPC — the index is what makes the ID path O(1) instead of a sorted scan.
fn chembl_id_offset_index() -> &'static HashMap<u32, u64> {
    static INDEX: OnceLock<HashMap<u32, u64>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut map = HashMap::new();
        let p = crate::k3_retrieval::repo_root()
            .map(|r| r.join("swarm_v2_artifacts/chembl_id_offset_index.json"))
            .filter(|p| p.is_file());
        if let Some(p) = p {
            if let Ok(raw) = std::fs::read_to_string(p) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    if let Some(obj) = v.as_object() {
                        for (digits, off) in obj {
                            if let (Ok(d), Some(o)) = (digits.parse::<u32>(), off.as_u64()) {
                                map.insert(d, o);
                            }
                        }
                    }
                }
            }
        }
        map
    })
}

/// Molecule recall: by ChEMBL ID (sorted, early exit) or by SMILES/term match (capped scan).
fn chembl_lookup(query: &str) -> Value {
    let path = match sidecar_path() {
        Some(p) if p.is_file() => p,
        _ => {
            return json!({
                "status": "ok", "store": "chembl", "kind": "molecule",
                "found": false,
                "basis": "chembl co-processor sidecar not on this host (swarm_v2_chembl_fabric.text.jsonl)"
            })
        }
    };
    use std::io::{BufRead, BufReader};
    let f = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => return json!({"status":"error","store":"chembl","error":format!("open: {e}")}),
    };

    // ID path (voltage-doctrine fast path): O(1) byte-offset index -> seek -> read one record. The
    // sidecar is sorted by molregno/ingestion order, NOT chembl_id string, so the old sorted-scan
    // early-exit returned false before reaching non-top-sorted ids (CHEMBL25 sits at line 4247 while
    // CHEMBL440060 is line 2). The index is derived once from this same file, no external data.
    if let Some(id) = chembl_id(query) {
        let digits = match id.strip_prefix("CHEMBL").and_then(|d| d.parse::<u32>().ok()) {
            Some(d) => d,
            None => return json!({
                "status": "ok", "store": "chembl", "kind": "molecule",
                "found": false, "query_id": id,
                "basis": "chembl co-processor sidecar: invalid ChEMBL id (non-numeric)"
            }),
        };
        if let Some(offset) = chembl_id_offset_index().get(&digits).copied() {
            if let Ok(mut f) = std::fs::File::open(&path) {
                use std::io::{Seek, SeekFrom};
                if f.seek(SeekFrom::Start(offset)).is_ok() {
                    let mut line = String::new();
                    if std::io::BufReader::new(f).read_line(&mut line).unwrap_or(0) > 0 {
                        let key = field(&line, "key").unwrap_or_default();
                        let text = field(&line, "text").unwrap_or_default();
                        return json!({
                            "status": "ok", "store": "chembl", "kind": "molecule",
                            "found": true, "key": key, "text": text, "records_scanned": 1,
                            "basis": "chembl co-processor sidecar, exact ChEMBL-id match via byte-offset index (O(1) seek)"
                        });
                    }
                }
            }
        }
        return json!({
            "status": "ok", "store": "chembl", "kind": "molecule",
            "found": false, "query_id": id,
            "basis": "chembl co-processor sidecar: no such ChEMBL id (byte-offset index miss)"
        });
    }

    // Term path: lowercase query terms of 3+ chars, matched in the record text, capped at 4M lines.
    let terms: Vec<String> = query
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() >= 3)
        .map(|t| t.to_string())
        .collect();
    if terms.is_empty() {
        return json!({"status":"error","store":"chembl","kind":"molecule","error":"no_query_terms",
                      "detail":"a molecule query needs a ChEMBL id or SMILES/terms"});
    }
    let cap = 4_000_000usize;
    let mut reader = BufReader::new(f);
    let mut line = String::new();
    let mut best: Option<(usize, String, String)> = None;
    let mut scanned = 0usize;
    while scanned < cap {
        line.clear();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        scanned += 1;
        if !line.contains('{') {
            continue;
        }
        let text = field(&line, "text").unwrap_or_default();
        let lower = text.to_ascii_lowercase();
        let mut hits = 0usize;
        for t in &terms {
            if lower.contains(t.as_str()) {
                hits += 1;
            }
        }
        if hits > 0 && best.as_ref().map_or(true, |b| hits > b.0) {
            let key = field(&line, "key").unwrap_or_default();
            best = Some((hits, key, text));
        }
    }
    match best {
        Some((hits, key, text)) => json!({
            "status": "ok", "store": "chembl", "kind": "molecule",
            "found": true, "key": key, "text": text, "matched_terms": hits,
            "records_scanned": scanned, "capped": scanned >= cap,
            "basis": "chembl co-processor sidecar, lexical term match (capped scan; a packed vector scan is the follow-up)"
        }),
        None => json!({
            "status": "ok", "store": "chembl", "kind": "molecule", "found": false,
            "records_scanned": scanned, "capped": scanned >= cap,
            "basis": "chembl co-processor sidecar: no term match in the scanned records"
        }),
    }
}

/// Molecule vector recall (MOTION-020): encode the query with the store's own encoder
/// (native-turbovec-hdc, 768-dim) and scan the packed 2.9M-compound store on the GPU (CPU fallback).
/// A real vector search, not a lexical grep. Called only after chembl_lookup finds nothing, so the
/// lexical ID fast path is preserved (LogicMaster's condition). Reads the store over file IPC.
fn chembl_vector_lookup(query: &str) -> Value {
    let store = match crate::packed_ternary::chembl_store() {
        Some(s) => s,
        None => return json!({
            "status": "error", "store": "chembl", "kind": "molecule",
            "error": "chembl packed store not on this host (swarm_v2_chembl_fabric.turbovec)"
        }),
    };
    let qv = crate::native_encoder::encode_f32(query, store.dim());
    match crate::packed_ternary::chembl_search(&qv, 5) {
        Some(out) => {
            let hits: Vec<Value> = out.hits.iter().map(|h| json!({
                "rank": h.rank, "key": h.key, "score": h.score,
            })).collect();
            json!({
                "status": "ok", "store": "chembl", "kind": "molecule",
                "found": !hits.is_empty(), "results": hits,
                "verdict": out.verdict, "scanned": out.scanned, "elapsed_us": out.us,
                "basis": out.basis,
                "engine": "packed-domain GPU scan over swarm_v2_chembl_fabric.turbovec (2,897,805 compounds, native-turbovec-hdc)",
            })
        }
        None => json!({
            "status": "error", "store": "chembl", "kind": "molecule",
            "error": "vector scan failed (store rejected or empty query)"
        }),
    }
}

/// Structured vector recall (MOTION-021): encode the query and scan the structured text store
/// (ethics + prose) on the GPU (CPU fallback). The SMILES domains (tox21/binding) + clinical are a
/// stated follow-up, not faked — this scan covers the text domains only.
fn structured_vector_lookup(query: &str) -> Value {
    let store = match crate::packed_ternary::structured_store() {
        Some(s) => s,
        None => return json!({
            "status": "error", "store": "structured", "kind": "structured",
            "error": "structured text store not on this host (swarm_v2_structured_text.turbovec)"
        }),
    };
    let qv = crate::native_encoder::encode_f32(query, store.dim());
    match crate::packed_ternary::structured_search(&qv, 5) {
        Some(out) => {
            let hits: Vec<Value> = out.hits.iter().map(|h| json!({
                "rank": h.rank, "key": h.key, "score": h.score,
            })).collect();
            json!({
                "status": "ok", "store": "structured", "kind": "structured",
                "found": !hits.is_empty(), "results": hits,
                "verdict": out.verdict, "scanned": out.scanned, "elapsed_us": out.us,
                "basis": out.basis,
                "engine": "packed-domain GPU scan over swarm_v2_structured_text.turbovec (ethics+prose, 97,480 docs, native-turbovec-hdc)",
                "note": "text domains only; SMILES (tox21/binding) + clinical are a stated follow-up, not served here",
            })
        }
        None => json!({
            "status": "error", "store": "structured", "kind": "structured",
            "error": "vector scan failed (store rejected or empty query)"
        }),
    }
}

/// Bioassay vector recall (MOTION-021 follow-up): encode the query and scan the bioassay store
/// (tox21 SMILES+assay + binding family+SMILES) on the GPU (CPU fallback).
fn bioassay_vector_lookup(query: &str) -> Value {
    let store = match crate::packed_ternary::bioassay_store() {
        Some(s) => s,
        None => return json!({
            "status": "error", "store": "bioassay", "kind": "bioassay",
            "error": "bioassay store not on this host (swarm_v2_bioassay.turbovec)"
        }),
    };
    let qv = crate::native_encoder::encode_f32(query, store.dim());
    match crate::packed_ternary::bioassay_search(&qv, 5) {
        Some(out) => {
            let hits: Vec<Value> = out.hits.iter().map(|h| json!({
                "rank": h.rank, "key": h.key, "score": h.score,
            })).collect();
            json!({
                "status": "ok", "store": "bioassay", "kind": "bioassay",
                "found": !hits.is_empty(), "results": hits,
                "verdict": out.verdict, "scanned": out.scanned, "elapsed_us": out.us,
                "basis": out.basis,
                "engine": "packed-domain GPU scan over swarm_v2_bioassay.turbovec (tox21+binding, 597,430 records, native-turbovec-hdc)",
            })
        }
        None => json!({
            "status": "error", "store": "bioassay", "kind": "bioassay",
            "error": "vector scan failed (store rejected or empty query)"
        }),
    }
}

/// The clinical catalog sidecar (40 measurements + summary + sum check), loaded once over file IPC.
fn clinical_catalog() -> Option<&'static serde_json::Value> {
    static CATALOG: OnceLock<Option<Value>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let p = crate::k3_retrieval::repo_root()?.join("swarm_v2_artifacts/clinical_catalog.json");
        let raw = std::fs::read_to_string(p).ok()?;
        serde_json::from_str(&raw).ok()
    }).as_ref()
}

/// Exact (case-insensitive) match of a query against the clinical catalog's aliases + codes, with a
/// word-level fallback so a sentence like "query the clinical value for calcium" still resolves the
/// bare measurement token. No fuzzy — a word either matches an alias/code or it does not.
fn is_clinical_measurement(q: &str) -> bool {
    let Some(catalog) = clinical_catalog() else { return false; };
    let aliases = catalog.get("aliases").and_then(Value::as_object);
    let ql = q.to_ascii_lowercase();
    if aliases.and_then(|a| a.get(&ql)).is_some() {
        return true;
    }
    // Word-level: check each alphanumeric token against the alias map + the code list.
    let words: Vec<String> = q
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_ascii_lowercase())
        .collect();
    if let Some(a) = aliases {
        if words.iter().any(|w| a.get(w).is_some()) {
            return true;
        }
    }
    catalog.get("measurements").and_then(Value::as_array).map(|arr| {
        arr.iter().any(|m| {
            let code = m.get("code").and_then(Value::as_str).map(|c| c.to_ascii_lowercase()).unwrap_or_default();
            code == ql || words.iter().any(|w| *w == code)
        })
    }).unwrap_or(false)
}

/// Clinical lexical lookup (MOTION-022): case-insensitive EXACT match — the alias map first (explicit,
/// build-tested), then the 40 codes. No fuzzy matching. No measurement named -> the 40-code catalog.
fn clinical_lookup(query: &str) -> Value {
    let Some(catalog) = clinical_catalog() else {
        return json!({"status":"error","store":"clinical","kind":"clinical",
                      "error":"clinical catalog not on this host (swarm_v2_artifacts/clinical_catalog.json)"});
    };
    let q = query.trim().to_ascii_lowercase();
    let ms = catalog.get("measurements").and_then(Value::as_array);
    let aliases = catalog.get("aliases").and_then(Value::as_object);
    // Word-level fallback: a sentence like "query the clinical value for calcium" carries the
    // measurement as one token; resolve that token against the alias map.
    let words: Vec<String> = query
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_ascii_lowercase())
        .collect();
    let canonical: Option<String> = aliases
        .and_then(|a| a.get(&q))
        .and_then(Value::as_str)
        .map(String::from)
        .or_else(|| {
            aliases.and_then(|a| {
                words.iter().find_map(|w| a.get(w)).and_then(Value::as_str).map(String::from)
            })
        })
        .or_else(|| {
            ms.and_then(|arr| {
                arr.iter()
                    .find(|m| {
                        let code = m.get("code").and_then(Value::as_str).map(|c| c.to_ascii_lowercase()).unwrap_or_default();
                        code == q || words.iter().any(|w| *w == code)
                    })
                    .and_then(|m| m.get("code").and_then(Value::as_str).map(String::from))
            })
        });

    let sum_check = catalog.get("sum_check").cloned().unwrap_or(Value::Null);
    match canonical {
        Some(code) => {
            let rec = ms.and_then(|arr| arr.iter().find(|m| m.get("code").and_then(Value::as_str) == Some(code.as_str()))).cloned();
            match rec {
                Some(r) => json!({
                    "status": "ok", "store": "clinical", "kind": "clinical", "found": true,
                    "measurement": r["code"], "count": r["count"],
                    "min": r["min"], "max": r["max"], "mean": r["mean"],
                    "basis": "clinical_catalog.json lexical measurement lookup (case-insensitive exact match)",
                    "sum_check": sum_check,
                }),
                None => json!({"status":"error","store":"clinical","error":"catalog inconsistency: canonical code not in measurements"}),
            }
        }
        None => {
            let codes: Vec<&str> = ms.map(|arr| arr.iter().filter_map(|m| m.get("code").and_then(Value::as_str)).collect()).unwrap_or_default();
            json!({
                "status": "ok", "store": "clinical", "kind": "clinical", "found": false,
                "distinct_measurements": codes.len(), "measurements": codes,
                "basis": "clinical catalog: no measurement named; returning the code catalog (exact match only, no fuzzy)",
                "sum_check": sum_check,
            })
        }
    }
}

/// The sealed trm7m corpus scores (block 352): doc key -> (trit, confidence). Loaded lazily from the
/// local index over file I/O — never HTTP, no secret material, no network egress (Shield's condition).
/// Fail-closed: a key absent from the index has no entry, and the annotator reports "unscored".
static TRM7M_SCORES: OnceLock<HashMap<String, (i8, f32)>> = OnceLock::new();

fn trm7m_scores() -> &'static HashMap<String, (i8, f32)> {
    TRM7M_SCORES.get_or_init(|| {
        let mut map = HashMap::new();
        let path = crate::k3_retrieval::repo_root()
            .map(|r| r.join("swarm_v2_artifacts/corpus_trm7m_scores.jsonl"))
            .filter(|p| p.is_file());
        if let Some(p) = path {
            if let Ok(f) = std::fs::File::open(&p) {
                use std::io::{BufRead, BufReader};
                for line in BufReader::new(f).lines().flatten() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                        let key = v.get("key").and_then(serde_json::Value::as_str).unwrap_or("").to_string();
                        let trit = v.get("trit").and_then(serde_json::Value::as_i64).unwrap_or(0) as i8;
                        let conf = v.get("confidence").and_then(serde_json::Value::as_f64).unwrap_or(0.0) as f32;
                        map.insert(key, (trit, conf));
                    }
                }
            }
        }
        map
    })
}

/// Annotate a k3_retrieve payload's results with their sealed trm7m scores AND re-rank them by the
/// model tier (MOTION-023). Deterministic key -> (trit, confidence) lookup (LogicMaster's condition),
/// fail-closed to "unscored" for keys absent from the index. The re-rank is a STABLE sort: results
/// ordered by (trm7m_trit desc, trm7m_confidence desc), ties keep the vector rank; unscored docs sort
/// after scored docs (no model verdict = least model-tier support). This is a re-order, not a filter —
/// no result is dropped (a filter is a later, separately-ratified step). Runs in-kernel; no Python.
fn annotate_and_rerank_trm7m(mut payload: Value, filter: bool) -> Value {
    if let Some(obj) = payload.as_object_mut() {
        if let Some(results) = obj.get_mut("results").and_then(Value::as_array_mut) {
            let scores = trm7m_scores();
            let mut scored = 0usize;
            let mut unscored = 0usize;
            for (i, r) in results.iter_mut().enumerate() {
                r["vector_rank"] = json!(i + 1);
                let doc = r.get("doc").and_then(Value::as_str).unwrap_or("");
                match scores.get(doc) {
                    Some(&(trit, conf)) => {
                        r["trm7m_trit"] = json!(trit);
                        r["trm7m_trit_label"] = json!(crate::trm7m::trit_label(trit));
                        r["trm7m_confidence"] = json!(conf);
                        scored += 1;
                    }
                    None => {
                        r["trm7m"] = json!("unscored");
                        unscored += 1;
                    }
                }
            }
            // Filter (CAP-004): drop results the model marks FALSE (trit -1). Unscored stays (no
            // verdict, fail-closed keep). Re-rank re-orders and keeps everything; filter removes.
            let mut dropped = 0usize;
            if filter {
                let before = results.len();
                results.retain(|r| r.get("trm7m_trit").and_then(Value::as_i64).map(|t| t != -1).unwrap_or(true));
                dropped = before - results.len();
            }
            // Stable re-rank by (trit desc, confidence desc); unscored sorts last.
            results.sort_by(|a, b| {
                let (ta, ca) = trm7m_sort_key(a);
                let (tb, cb) = trm7m_sort_key(b);
                tb.cmp(&ta).then(cb.total_cmp(&ca))
            });
            if filter {
                obj.insert("trm7m_filtered".into(), json!(true));
                obj.insert("trm7m_dropped_false".into(), json!(dropped));
            }
            obj.insert("re_ranked_by".into(), json!("trm7m model tier (trit desc, confidence desc, stable on vector_rank; unscored last; re-order, no drop)"));
            obj.insert("trm7m_scored".into(), json!(scored));
            obj.insert("trm7m_unscored".into(), json!(unscored));
        }
    }
    payload
}

/// Sort key for the trm7m re-rank: (trit, confidence); unscored = (-2, 0.0) so it sorts below FALSE (-1).
fn trm7m_sort_key(r: &Value) -> (i8, f32) {
    if r.get("trm7m").and_then(Value::as_str) == Some("unscored") {
        return (-2, 0.0);
    }
    let trit = r.get("trm7m_trit").and_then(Value::as_i64).unwrap_or(0) as i8;
    let conf = r.get("trm7m_confidence").and_then(Value::as_f64).unwrap_or(0.0) as f32;
    (trit, conf)
}

/// The router entry point. Returns (json, http_status).
pub fn route(query: &str) -> (Value, u16) {
    let kind = classify(query);
    match kind {
        "molecule" => {
            // ChEMBL ID -> exact lexical match (fast path, preserved). Everything else -> the
            // packed-domain vector scan (GPU, seconds), replacing the slow capped term grep.
            // A query that NAMES a ChEMBL ID must NOT fall through to the vector scan: an ID
            // absent from the sidecar is a NOT-FOUND fact, not "compounds similar to the ID
            // string". Only a term query (no ID) falls through.
            let has_id = chembl_id(query).is_some();
            let mut v = if has_id {
                chembl_lookup(query)
            } else {
                chembl_vector_lookup(query)
            };
            if !has_id && !v.get("found").and_then(Value::as_bool).unwrap_or(false) {
                v = chembl_vector_lookup(query);
            }
            if let Some(obj) = v.as_object_mut() {
                obj.insert("kind".into(), json!("molecule"));
                obj.insert("router".into(), json!("multi-store (fabric + chembl + sql)"));
            }
            (v, 200)
        }
        "structured" => {
            // MOTION-021: the structured tier's text domains (ethics + prose) are now a real vector
            // scan; SMILES/clinical stay a stated follow-up, not faked.
            let mut v = structured_vector_lookup(query);
            if let Some(obj) = v.as_object_mut() {
                obj.insert("router".into(), json!("multi-store (fabric + chembl + structured-text)"));
            }
            (v, 200)
        },
        "bioassay" => {
            // MOTION-021 follow-up: tox21 + binding are now a real vector scan.
            let mut v = bioassay_vector_lookup(query);
            if let Some(obj) = v.as_object_mut() {
                obj.insert("router".into(), json!("multi-store (fabric + chembl + bioassay + structured-text)"));
            }
            (v, 200)
        },
        "clinical" => {
            // MOTION-022: clinical is a lexical measurement lookup (tabular), not a vector scan.
            let mut v = clinical_lookup(query);
            if let Some(obj) = v.as_object_mut() {
                obj.insert("router".into(), json!("multi-store (fabric + chembl + bioassay + structured-text + clinical)"));
            }
            (v, 200)
        },
        _ => {
            // General recall is the existing K3 retrieval engine (fabric + distilled model stores).
            // Annotated with the sealed trm7m scores (MOTION-019): each result carries its model-tier
            // trit+confidence alongside the packed-domain cosine rank.
            let q = query.replace('&', "%26").replace('?', "%3F").replace(' ', "%20");
            let (payload, code) = crate::k3_retrieval::handle(&format!("/k3/retrieve?q={q}"));
            (annotate_and_rerank_trm7m(payload, false), code)
        }
    }
}

/// Cross-domain synthesis (CAP-001/CAP-102): fuse a query across every knowledge domain into one
/// ranked answer. Fusion method (MOTION-026): Reciprocal Rank Fusion (k=60) over the four vector
/// stores — each store contributes its top-K in its OWN ranking (fabric trm7m-re-ranked; the others
/// cosine-ranked), and a result's fused score is 1/(k + rank). Rank-based fusion combines ORDERS, not
/// magnitudes, so no cross-store score-comparability assumption. Since the stores are disjoint, the
/// cross-store rank-1s tie on RRF; the tie is broken by the raw cosine (same query, same encoder and
/// 2-bit metric — a legitimate secondary signal). Clinical is tabular, so it is reported as a fact.
/// Domain preference for the fused rank-1 tie-break (MOTION-C3): when a query classifies as
/// molecule / bioassay / structured, that domain's store gains a small rrf preference so its rank-1
/// outranks the other stores' rank-1s — but the boost is less than one rank band (1/61 - 1/62 ≈
/// 0.00026), so a domain store's rank-2 never outranks another store's rank-1. This re-orders tied
/// rank-1s; it never drops a result.
fn domain_boost(kind: &str, store: &str) -> f64 {
    match (kind, store) {
        ("molecule", "chembl") | ("bioassay", "bioassay") | ("structured", "structured") => 0.0001,
        _ => 0.0,
    }
}

pub fn fused_lookup(query: &str) -> Value {
    const K: f64 = 60.0;
    let mut fused: Vec<Value> = Vec::new();
    let mut searched: Vec<&str> = Vec::new();
    let kind = classify(query);

    // 0. Drug-name resolution (MOTION-043): a bare drug name resolves to its ChEMBL id + SMILES via
    // the existing name index + SMILES sidecar (voltage doctrine — derived, not fetched). The
    // resolved record is ranked first (rrf 1.0), like clinical, so a bare drug name answers the
    // chembl store it belongs to instead of prose that merely mentions it. Clinical measurements
    // (Calcium, Glucose, Potassium, ...) also occur as drug/mineral synonyms, so the clinical
    // classification wins those — a lab value is a clinical fact, not a drug query.
    if kind != "clinical" {
        if let Some((cid, smiles)) = chembl_name_resolve(query) {
            fused.push(json!({
                "store": "chembl",
                "key": format!("chembl/{}", cid),
                "smiles": smiles,
                "score": 1.0,
                "rrf": 1.0,
                "name_resolved_from": query.trim().to_ascii_lowercase(),
                "basis": "bare drug name -> ChEMBL id + SMILES via chembl_name_index + chembl_smiles sidecars (derived, not fetched); ranked first",
            }));
        }
    }

    // 1. fabric (general path) — trm7m-annotated + re-ranked; its rank is the re-ranked position.
    let q = query.replace('&', "%26").replace('?', "%3F").replace(' ', "%20");
    let (fabric_payload, _) = crate::k3_retrieval::handle(&format!("/k3/retrieve?q={q}"));
    let fabric = annotate_and_rerank_trm7m(fabric_payload, false);
    if let Some(results) = fabric.get("results").and_then(Value::as_array) {
        searched.push("fabric");
        for (i, r) in results.iter().enumerate() {
            let mut x = r.clone();
            // Provenance (MOTION-042): the fabric's relational/* segments are ethics/prose — the
            // structured store's domain — so they are relabeled structured; only papers + research
            // notes stay "fabric". This resolves the structured/fabric overlap in the fusion.
            let doc = x.get("doc").and_then(Value::as_str).unwrap_or("");
            let structured_domain = doc.starts_with("relational/");
            let own_research = doc.starts_with("papers/");
            x["store"] = json!(if structured_domain { "structured" } else { "fabric" });
            let base_rrf = 1.0 / (K + (i as f64) + 1.0);
            // Provenance boost (MOTION-042): the platform's curated papers rank above the external
            // relational content, so a research query prefers its own papers over ethics/enwikiquote
            // quotations. The Seeker's research notes (swarm_v2_artifacts/research/*) are NOT boosted
            // — they are noisy recall logs and would outrank ethics content for ethics queries.
            x["rrf"] = json!(if own_research { base_rrf + 0.02 } else { base_rrf });
            fused.push(x);
        }
    }

    // 2–4. molecule / structured / bioassay vector scans (cosine-ranked).
    for (name, payload) in [
        ("chembl", chembl_vector_lookup(query)),
        ("structured", structured_vector_lookup(query)),
        ("bioassay", bioassay_vector_lookup(query)),
    ] {
        if let Some(results) = payload.get("results").and_then(Value::as_array) {
            searched.push(name);
            for (i, r) in results.iter().enumerate() {
                let mut x = r.clone();
                x["store"] = json!(name);
                x["rrf"] = json!(1.0 / (K + (i as f64) + 1.0) + domain_boost(kind, name));
                fused.push(x);
            }
        }
    }

    // 4b. semantic store (MOTION-G1): EmbeddingGemma 2 index, separate from the HDC fabric. The
    // query is encoded by the local embedding server and ranked by cosine over the 6-bit index —
    // natural-language retrieval without keyword overlap.
    let semantic = crate::semantic_recall::lookup(query, 5);
    if let Some(results) = semantic.get("results").and_then(Value::as_array) {
        searched.push("semantic");
        for (i, r) in results.iter().enumerate() {
            let mut x = r.clone();
            x["store"] = json!("semantic");
            x["rrf"] = json!(1.0 / (K + (i as f64) + 1.0));
            fused.push(x);
        }
    }

    // 5. clinical (lexical) — an exact match is a RANKED candidate, not just a side fact. The
    // definitive exact match must be able to rank first (top RRF slot); carrying it only as a
    // side fact is why the clinical P@1 was 0/40 (the fact could never be top-1).
    let clinical = clinical_lookup(query);
    searched.push("clinical");
    if clinical.get("found").and_then(Value::as_bool) == Some(true) {
        fused.push(json!({
            "store": "clinical",
            "doc": clinical.get("measurement").cloned().unwrap_or(Value::Null),
            "score": 1.0,
            "rrf": 1.0,
            "measurement": clinical.get("measurement").cloned().unwrap_or(Value::Null),
            "count": clinical.get("count").cloned().unwrap_or(Value::Null),
            "mean": clinical.get("mean").cloned().unwrap_or(Value::Null),
            "min": clinical.get("min").cloned().unwrap_or(Value::Null),
            "max": clinical.get("max").cloned().unwrap_or(Value::Null),
            "basis": "clinical_catalog.json exact lexical match (ranked first — definitive answer)",
        }));
    }

    // Rank-based fusion: RRF desc, tie-break by raw cosine desc.
    fused.sort_by(|a, b| {
        let ra = a.get("rrf").and_then(Value::as_f64).unwrap_or(0.0);
        let rb = b.get("rrf").and_then(Value::as_f64).unwrap_or(0.0);
        let ca = a.get("score").and_then(Value::as_f64).unwrap_or(0.0);
        let cb = b.get("score").and_then(Value::as_f64).unwrap_or(0.0);
        rb.total_cmp(&ra).then(cb.total_cmp(&ca))
    });

    json!({
        "status": "ok",
        "kind": "fused",
        "store": "multi-store",
        "fusion": "reciprocal-rank-fusion (k=60) with cosine tie-break, over fabric + chembl + structured + bioassay",
        "stores_searched": searched,
        "result_count": fused.len(),
        "merged": fused,
        "clinical_fact": clinical,
        "basis": "cross-domain synthesis: rank-based fusion (RRF k=60, cosine tie-break); fabric trm7m-re-ranked; clinical reported as a fact",
    })
}

/// Router entry point for the fused (cross-domain) mode.
pub fn route_fused(query: &str) -> (Value, u16) {
    (fused_lookup(query), 200)
}

/// Text-domain fused recall — fabric + structured + clinical, skipping the molecule/bioassay
/// vector scans. For a query that does not classify as molecule/bioassay, the chembl (2.9M) and
/// bioassay (597K) packed scans are irrelevant and cost the bulk of the wall-clock, so this path
/// is the fast one for general research topics. Same RRF fusion, same tie-break.
pub fn fused_lookup_text(query: &str) -> Value {
    const K: f64 = 60.0;
    let mut fused: Vec<Value> = Vec::new();
    let mut searched: Vec<&str> = Vec::new();
    let kind = classify(query);

    // 0. Drug-name resolution (MOTION-043): same as fused_lookup — a bare drug name resolves to its
    // ChEMBL id + SMILES and ranks first, so the text path also answers drug names (not prose).
    // Clinical measurements win over drug-name resolution (Calcium/Glucose/... are lab values).
    if kind != "clinical" {
        if let Some((cid, smiles)) = chembl_name_resolve(query) {
            fused.push(json!({
                "store": "chembl",
                "key": format!("chembl/{}", cid),
                "smiles": smiles,
                "score": 1.0,
                "rrf": 1.0,
                "name_resolved_from": query.trim().to_ascii_lowercase(),
                "basis": "bare drug name -> ChEMBL id + SMILES via chembl_name_index + chembl_smiles sidecars (derived, not fetched); ranked first",
            }));
        }
    }

    // 1. fabric (general path) — trm7m-annotated + re-ranked.
    let q = query.replace('&', "%26").replace('?', "%3F").replace(' ', "%20");
    let (fabric_payload, _) = crate::k3_retrieval::handle(&format!("/k3/retrieve?q={q}"));
    let fabric = annotate_and_rerank_trm7m(fabric_payload, false);
    if let Some(results) = fabric.get("results").and_then(Value::as_array) {
        searched.push("fabric");
        for (i, r) in results.iter().enumerate() {
            let mut x = r.clone();
            // Provenance (MOTION-042): the fabric's relational/* segments are ethics/prose — the
            // structured store's domain — so they are relabeled structured; only papers + research
            // notes stay "fabric". This resolves the structured/fabric overlap in the fusion.
            let doc = x.get("doc").and_then(Value::as_str).unwrap_or("");
            let structured_domain = doc.starts_with("relational/");
            let own_research = doc.starts_with("papers/");
            x["store"] = json!(if structured_domain { "structured" } else { "fabric" });
            let base_rrf = 1.0 / (K + (i as f64) + 1.0);
            // Provenance boost (MOTION-042): the platform's curated papers rank above the external
            // relational content, so a research query prefers its own papers over ethics/enwikiquote
            // quotations. The Seeker's research notes (swarm_v2_artifacts/research/*) are NOT boosted
            // — they are noisy recall logs and would outrank ethics content for ethics queries.
            x["rrf"] = json!(if own_research { base_rrf + 0.02 } else { base_rrf });
            fused.push(x);
        }
    }

    // 2. structured text store (97,480 ethics+prose docs; far cheaper than chembl/bioassay).
    let structured = structured_vector_lookup(query);
    if let Some(results) = structured.get("results").and_then(Value::as_array) {
        searched.push("structured");
        for (i, r) in results.iter().enumerate() {
            let mut x = r.clone();
            x["store"] = json!("structured");
            x["rrf"] = json!(1.0 / (K + (i as f64) + 1.0) + domain_boost(kind, "structured"));
            fused.push(x);
        }
    }

    // 2b. semantic store (MOTION-G1): EmbeddingGemma 2 index, separate from the HDC fabric. The
    // query is encoded by the local embedding server and ranked by cosine over the 6-bit index.
    // This is what lets a natural-language query find the semantically-right paper without keyword
    // overlap — a capability the lexical HDC fabric does not have.
    let semantic = crate::semantic_recall::lookup(query, 5);
    if let Some(results) = semantic.get("results").and_then(Value::as_array) {
        searched.push("semantic");
        for (i, r) in results.iter().enumerate() {
            let mut x = r.clone();
            x["store"] = json!("semantic");
            x["rrf"] = json!(1.0 / (K + (i as f64) + 1.0));
            fused.push(x);
        }
    }

    // 3. clinical (lexical) — an exact match is a RANKED candidate (top RRF slot), not just a
    // side fact; this is what lets a clinical measurement code rank first.
    let clinical = clinical_lookup(query);
    searched.push("clinical");
    if clinical.get("found").and_then(Value::as_bool) == Some(true) {
        fused.push(json!({
            "store": "clinical",
            "doc": clinical.get("measurement").cloned().unwrap_or(Value::Null),
            "score": 1.0,
            "rrf": 1.0,
            "measurement": clinical.get("measurement").cloned().unwrap_or(Value::Null),
            "count": clinical.get("count").cloned().unwrap_or(Value::Null),
            "mean": clinical.get("mean").cloned().unwrap_or(Value::Null),
            "min": clinical.get("min").cloned().unwrap_or(Value::Null),
            "max": clinical.get("max").cloned().unwrap_or(Value::Null),
            "basis": "clinical_catalog.json exact lexical match (ranked first — definitive answer)",
        }));
    }

    fused.sort_by(|a, b| {
        let ra = a.get("rrf").and_then(Value::as_f64).unwrap_or(0.0);
        let rb = b.get("rrf").and_then(Value::as_f64).unwrap_or(0.0);
        let ca = a.get("score").and_then(Value::as_f64).unwrap_or(0.0);
        let cb = b.get("score").and_then(Value::as_f64).unwrap_or(0.0);
        rb.total_cmp(&ra).then(cb.total_cmp(&ca))
    });

    json!({
        "status": "ok",
        "kind": "fused",
        "store": "multi-store",
        "fusion": "reciprocal-rank-fusion (k=60) with cosine tie-break, over fabric + structured",
        "stores_searched": searched,
        "result_count": fused.len(),
        "merged": fused,
        "clinical_fact": clinical,
        "basis": "text-domain fusion (fabric + structured + clinical); chembl/bioassay skipped because the query is not a molecule/bioassay query",
    })
}

/// SMILES cross-reference sidecar (CAP-101): chembl_id -> [bioassay keys] matched by exact SMILES
/// string equality (whitespace-stripped), not chemical canonicalization. Loaded lazily over file IPC.
fn smiles_crossref() -> &'static HashMap<String, Vec<String>> {
    static CROSSREF: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    CROSSREF.get_or_init(|| {
        let mut map = HashMap::new();
        let p = crate::k3_retrieval::repo_root()
            .map(|r| r.join("swarm_v2_artifacts/smiles_crossref.json"))
            .filter(|p| p.is_file());
        if let Some(p) = p {
            if let Ok(raw) = std::fs::read_to_string(p) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    if let Some(obj) = v.as_object() {
                        for (cid, matches) in obj {
                            if let Some(arr) = matches.as_array() {
                                map.insert(cid.clone(), arr.iter().filter_map(|x| x.as_str().map(String::from)).collect());
                            }
                        }
                    }
                }
            }
        }
        map
    })
}

/// Drug-name -> ChEMBL-id index (MOTION-043): built by scratch/build_chembl_name_index.py from
/// scratch/chembl_names/chembl_names_map.json (51,220 chembl_ids, 130,053 pref-names + synonyms).
/// This is the voltage-doctrine module for name resolution — derived from the existing map, not
/// fetched over any network. Loaded lazily over file IPC.
fn chembl_name_index() -> &'static HashMap<String, String> {
    static INDEX: OnceLock<HashMap<String, String>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut map = HashMap::new();
        let p = crate::k3_retrieval::repo_root()
            .map(|r| r.join("swarm_v2_artifacts/chembl_name_index.json"))
            .filter(|p| p.is_file());
        if let Some(p) = p {
            if let Ok(raw) = std::fs::read_to_string(p) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    if let Some(obj) = v.as_object() {
                        for (name, cid) in obj {
                            if let Some(c) = cid.as_str() {
                                map.insert(name.clone(), c.to_string());
                            }
                        }
                    }
                }
            }
        }
        map
    })
}

/// ChEMBL-id -> SMILES for the named compounds (MOTION-043): derived by scratch/build_chembl_smiles.py
/// from the existing chembl text sidecar's own `smiles:` field. Loaded lazily over file IPC.
fn chembl_smiles() -> &'static HashMap<String, String> {
    static SMILES: OnceLock<HashMap<String, String>> = OnceLock::new();
    SMILES.get_or_init(|| {
        let mut map = HashMap::new();
        let p = crate::k3_retrieval::repo_root()
            .map(|r| r.join("swarm_v2_artifacts/chembl_smiles.json"))
            .filter(|p| p.is_file());
        if let Some(p) = p {
            if let Ok(raw) = std::fs::read_to_string(p) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    if let Some(obj) = v.as_object() {
                        for (cid, smi) in obj {
                            if let Some(s) = smi.as_str() {
                                map.insert(cid.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }
        map
    })
}

/// Resolve a bare drug name to its (ChEMBL id, SMILES) via the name index + SMILES sidecar. Exact
/// name match first (case-insensitive, trimmed), then a word-level fallback so a task like
/// "submit the bare drug name imatinib" still resolves the name token. None when no word is a known name.
fn chembl_name_resolve(query: &str) -> Option<(String, String)> {
    let name = query.trim().to_ascii_lowercase();
    if let Some(cid) = chembl_name_index().get(&name) {
        let smiles = chembl_smiles().get(cid).cloned().unwrap_or_default();
        return Some((cid.clone(), smiles));
    }
    // Word-level fallback: check each alphanumeric token against the name index.
    for word in query.split(|c: char| !c.is_ascii_alphanumeric()) {
        if word.len() < 3 {
            continue;
        }
        let w = word.to_ascii_lowercase();
        if let Some(cid) = chembl_name_index().get(&w) {
            let smiles = chembl_smiles().get(cid).cloned().unwrap_or_default();
            return Some((cid.clone(), smiles));
        }
    }
    None
}

/// Drug-repurposing pipeline (CAP-002/CAP-101): a staged discovery answer — ChEMBL candidates (now
/// SMILES-cross-referenced to tox21/binding) -> bioassay profile -> clinical fact. The cross-ref is
/// exact-SMILES-string equality (whitespace-stripped), not chemical canonicalization: a candidate is
/// annotated only when its SMILES string is byte-identical to a tox21/binding record's. tox21 is the
/// toxicity source; the clinical store is ICU patient vitals (a fact context, not drug toxicity).
pub fn pipeline_lookup(query: &str) -> Value {
    let chembl = chembl_vector_lookup(query);
    let mut candidates = chembl.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
    let bioassay = bioassay_vector_lookup(query);
    let profile = bioassay.get("results").and_then(Value::as_array).cloned().unwrap_or_default();
    let clinical = clinical_lookup(query);

    // CAP-101: annotate each candidate with its exact-SMILES tox21/binding matches.
    let crossref = smiles_crossref();
    let mut smi_matched = 0usize;
    for c in candidates.iter_mut() {
        let cid = c.get("key").and_then(Value::as_str).unwrap_or("")
            .strip_prefix("chembl/").unwrap_or("").to_string();
        if let Some(matches) = crossref.get(&cid) {
            c["smi_matches"] = json!(matches);
            c["smi_match_count"] = json!(matches.len());
            smi_matched += 1;
        }
    }

    json!({
        "status": "ok",
        "kind": "pipeline",
        "store": "discovery",
        "pipeline": ["chembl_candidates", "bioassay_profile", "clinical_fact"],
        "candidates": candidates,
        "candidate_count": candidates.len(),
        "smi_matched_candidates": smi_matched,
        "bioassay_profile": profile,
        "bioassay_profile_count": profile.len(),
        "clinical_fact": clinical,
        "honest_note": "SMILES cross-ref by exact string equality (whitespace-stripped), not chemical canonicalization. tox21 is the toxicity data; clinical is ICU vitals.",
        "basis": "drug-repurposing pipeline: chembl candidates (SMILES-cross-referenced to tox21/binding) -> bioassay profile -> clinical fact",
    })
}

/// Router entry point for the discovery-pipeline mode.
pub fn route_pipeline(query: &str) -> (Value, u16) {
    (pipeline_lookup(query), 200)
}

/// Router entry point for the filtered (truth-filter) mode: general recall + trm7m re-rank + drop
/// of results the model marks FALSE (trit -1). CAP-004.
pub fn route_filtered(query: &str) -> (Value, u16) {
    let q = query.replace('&', "%26").replace('?', "%3F").replace(' ', "%20");
    let (payload, code) = crate::k3_retrieval::handle(&format!("/k3/retrieve?q={q}"));
    (annotate_and_rerank_trm7m(payload, true), code)
}
