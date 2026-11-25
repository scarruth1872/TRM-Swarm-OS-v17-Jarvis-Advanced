//! /reading/* — machine reading over the local corpus (tasks 581, 582, 583, 584).
//!
//! Every response carries provenance: file, byte offset and sha256. A span whose offset does not resolve to the
//! identical bytes is not evidence, and these handlers compute the offsets rather than being told them.
//!
//! Paths are confined to the repository root. A reading endpoint that will open any path on the machine is a
//! file-disclosure primitive wearing a research hat.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use trm_uefi_kernel::k3_logic::sha256;
use trm_uefi_kernel::claims::{contradicts, extract, verify};
use trm_uefi_kernel::text_reading::{parse_markdown, split_sentences, UnitKind};

use crate::evidence_index;

/// Repository root: walk up from the crate directory until a directory containing `data/` is found.
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

/// Read a file confined to the repository root.
///
/// `canonicalize` happens BEFORE the prefix check, so `../..` and symlinks cannot walk out of the tree: a reading
/// endpoint that will open any path on the machine is a file-disclosure primitive with a research label.
fn read_source(path: &str) -> Result<(String, String, String), String> {
    let root = repo_root().ok_or("cannot locate the repository root")?;
    let raw = Path::new(path);
    let joined = if raw.is_absolute() { raw.to_path_buf() } else { root.join(raw) };
    let canon = joined
        .canonicalize()
        .map_err(|e| format!("cannot open '{path}': {e}"))?;
    let root_canon = root.canonicalize().unwrap_or(root);
    if !canon.starts_with(&root_canon) {
        return Err(format!("'{path}' resolves outside the repository and will not be read"));
    }
    let text = std::fs::read_to_string(&canon)
        .map_err(|e| format!("cannot read '{}': {e}", canon.display()))?;
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

/// Task 581 — structural parse with offsets.
pub fn parse(body: &Value) -> Result<Value, String> {
    let path = body.get("path").and_then(|v| v.as_str()).ok_or("'path' must be a string")?;
    let (text, rel, digest) = read_source(path)?;
    let units = parse_markdown(&text);
    let mut counts = serde_json::Map::new();
    for u in &units {
        let k = u.kind.as_str();
        let c = counts.get(k).and_then(|v| v.as_u64()).unwrap_or(0) + 1;
        counts.insert(k.to_string(), json!(c));
    }
    let out_units: Vec<Value> = units
        .iter()
        .map(|u| {
            json!({
                "kind": u.kind.as_str(),
                "start": u.start,
                "end": u.end,
                "line": u.line,
                "heading": u.heading,
                "level": u.level,
                // the slice is returned so the caller can compare it byte for byte without re-deriving offsets
                "text_preview": text[u.start..u.end].chars().take(120).collect::<String>(),
            })
        })
        .collect();
    Ok(json!({
        "file": rel,
        "sha256": digest,
        "bytes": text.len(),
        "units": out_units.len(),
        "by_kind": counts,
        "sections": out_units,
        "semantics": "offsets are byte offsets into the file as read; a unit's text is file[start..end]"
    }))
}

/// Task 584 — evidence spans: sentences with file, offset and sha256, plus a resolution check.
pub fn evidence(body: &Value) -> Result<Value, String> {
    let path = body.get("path").and_then(|v| v.as_str()).ok_or("'path' must be a string")?;
    let limit = body.get("limit").and_then(|v| v.as_u64()).unwrap_or(500) as usize;
    let (text, rel, digest) = read_source(path)?;
    let sents = split_sentences(&text);
    let mut bad = 0usize;
    let mut out = Vec::new();
    for (i, s) in sents.iter().take(limit).enumerate() {
        // the resolution check is done here, per span, against the bytes actually read
        let resolves = s.end <= text.len() && s.start < s.end && text[s.start..s.end] == s.text;
        if !resolves {
            bad += 1;
        }
        out.push(json!({
            "index": i,
            "start": s.start,
            "end": s.end,
            "text": s.text,
            "resolves": resolves,
        }));
    }
    Ok(json!({
        "file": rel,
        "sha256": digest,
        "spans": out,
        "count": sents.len(),
        "unresolved": bad,
        "claim": "every span's offsets resolve to identical bytes, or 'unresolved' is non-zero"
    }))
}

fn stopwords() -> [&'static str; 22] {
    ["the", "a", "an", "of", "and", "or", "in", "on", "for", "to", "is", "are", "was", "were",
     "what", "which", "how", "many", "does", "do", "did", "with"]
}

fn tokens(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 1)
        .map(|t| t.to_lowercase())
        .filter(|t| !stopwords().contains(&t.as_str()))
        .collect()
}

/// Task 582 — extractive QA. The answer is a sentence from the corpus, cited; never generated.
pub fn qa(body: &Value) -> Result<Value, String> {
    let question = body.get("question").and_then(|v| v.as_str()).ok_or("'question' must be a string")?;
    let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    if tokens(question).is_empty() {
        return Err("the question has no content words; nothing to match".to_string());
    }
    let hits = evidence_index::search(question, k);
    if hits.is_empty() {
        // abstention is a result: no evidence was found, so no answer is offered
        return Ok(json!({
            "question": question,
            "verdict": "NO_EVIDENCE",
            "answer": Value::Null,
            "note": "no chunk cleared the retrieval floor; the honest answer is that the corpus does not address this"
        }));
    }
    let qt: Vec<String> = tokens(question);

    // Candidates must be LOCATABLE in their source file. A chunk's text is not always verbatim from the file
    // (the index builder normalises some of it), and an answer that cannot be found in the file cannot be cited.
    // Two of thirteen answers were in that state, so candidates are checked against the file here and skipped if
    // they cannot be placed — with the count reported, because abstaining has a cost the caller should see.
    let mut best: Option<(f64, String, String, String, usize)> = None; // (score, sentence, source, chunk, offset)
    let mut unlocatable = 0usize;
    let mut file_cache: std::collections::HashMap<String, Option<String>> = std::collections::HashMap::new();
    for h in &hits {
        let file_text = file_cache
            .entry(h.source.clone())
            .or_insert_with(|| read_source(&h.source).ok().map(|(t, _, _)| t))
            .clone();
        let Some(ftext) = file_text else { continue };
        for s in split_sentences(&h.text) {
            let st = tokens(&s.text);
            if st.is_empty() {
                continue;
            }
            let overlap = qt.iter().filter(|t| st.contains(t)).count() as f64 / qt.len() as f64;
            if overlap < 0.4 {
                continue;
            }
            let Some((off, end)) = trm_uefi_kernel::text_reading::span_normalised(&ftext, &s.text) else {
                unlocatable += 1;
                continue;
            };
            // cite the FILE's bytes, not the chunk's tidied copy: those are the bytes the reader will open
            let exact = ftext[off..end].to_string();
            let score = overlap * h.score as f64;
            if best.as_ref().map(|(b, _, _, _, _)| score > *b).unwrap_or(true) {
                best = Some((score, exact, h.source.clone(), h.text.clone(), off));
            }
        }
    }

    match best {
        None => Ok(json!({
            "question": question,
            "verdict": "NO_EVIDENCE",
            "answer": Value::Null,
            "retrieved": hits.iter().map(|h| json!({"source": h.source, "score": h.score})).collect::<Vec<_>>(),
            "note": "chunks were retrieved but no sentence covered the question's content words; no answer invented"
        })),
        Some((score, sentence, source, chunk, offset)) => {
            let sha = read_source(&source).ok().map(|(_, _, d)| d);
            Ok(json!({
                "question": question,
                "verdict": "ANSWERED",
                "answer": sentence,
                "source": source,
                "file_offset": offset,
                "sha256": sha,
                "score": score,
                "chunk": chunk.chars().take(300).collect::<String>(),
                "unlocatable_candidates": unlocatable,
                "semantics": "extractive and citable: the answer is a sentence located in the cited file by byte offset"
            }))
        }
    }
}


/// Task 583 — multi-document summarisation by MMR, plus coverage repair and the metrics that make the claim
/// checkable.
///
/// Plain MMR maximises diversity and will happily drop the only chunk answering part of the query: measured at
/// lambda 0.6, one query fell from 0.25 coverage to 0.00. Diversity without coverage is a list of different
/// things that do not answer the question, so after selection the set is repaired against the top-k coverage
/// floor by swapping the least useful chunk for the candidate that adds the most uncovered query terms.
pub fn summarize(body: &Value) -> Result<Value, String> {
    let query = body.get("query").and_then(|v| v.as_str()).ok_or("'query' must be a string")?;
    let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    let lambda = body.get("lambda").and_then(|v| v.as_f64()).unwrap_or(0.6) as f32;
    let pool = body.get("pool").and_then(|v| v.as_u64()).unwrap_or(30) as usize;

    let mut mmr_hits = evidence_index::mmr(query, k, lambda, pool);
    if mmr_hits.is_empty() {
        return Ok(json!({
            "query": query, "verdict": "NO_EVIDENCE", "summary": [],
            "note": "nothing cleared the retrieval floor"
        }));
    }
    let topk = evidence_index::search(query, k);
    let qt = tokens(query);
    let coverage = |hits: &[evidence_index::Hit]| -> f64 {
        if qt.is_empty() {
            return 0.0;
        }
        let joined = hits.iter().map(|h| h.text.as_str()).collect::<Vec<_>>().join(" ");
        let jt = tokens(&joined);
        qt.iter().filter(|t| jt.contains(t)).count() as f64 / qt.len() as f64
    };

    let floor = coverage(&topk) - 0.10;
    let mut repairs = 0usize;
    if coverage(&mmr_hits) < floor {
        let pool_hits = evidence_index::search(query, pool.max(k));
        for _ in 0..k {
            if coverage(&mmr_hits) >= floor {
                break;
            }
            let present: Vec<String> = mmr_hits.iter().map(|h| h.text.clone()).collect();
            let have = tokens(&present.join(" "));
            let mut best: Option<(usize, usize)> = None;
            for (i, cand) in pool_hits.iter().enumerate() {
                if mmr_hits.iter().any(|h| h.source == cand.source && h.text == cand.text) {
                    continue;
                }
                let ct = tokens(&cand.text);
                let added = qt.iter().filter(|t| ct.contains(t) && !have.contains(t)).count();
                if added > 0 && best.map(|(b, _)| added > b).unwrap_or(true) {
                    best = Some((added, i));
                }
            }
            let mut victim: Option<(usize, usize)> = None;
            for (i, h) in mmr_hits.iter().enumerate() {
                let ht = tokens(&h.text);
                let contributes = qt.iter().filter(|t| ht.contains(t)).count();
                if victim.map(|(c, _)| contributes < c).unwrap_or(true) {
                    victim = Some((contributes, i));
                }
            }
            match (victim, best) {
                (Some((_, vi)), Some((_, bi))) => {
                    mmr_hits.remove(vi);
                    mmr_hits.push(pool_hits[bi].clone());
                    repairs += 1;
                }
                _ => break,
            }
        }
    }

    let mmr_red = evidence_index::mean_pairwise_redundancy(&mmr_hits);
    let top_red = evidence_index::mean_pairwise_redundancy(&topk);
    Ok(json!({
        "query": query,
        "lambda": lambda,
        "k": k,
        "summary": mmr_hits.iter().map(|h| json!({
            "source": h.source, "score": h.score,
            "text": h.text.chars().take(400).collect::<String>()
        })).collect::<Vec<_>>(),
        "metrics": {
            "mmr_redundancy": mmr_red,
            "topk_redundancy": top_red,
            "mmr_coverage": coverage(&mmr_hits),
            "topk_coverage": coverage(&topk),
            "coverage_repairs": repairs,
        },
        "semantics": "redundancy = mean pairwise cosine between selected chunks; coverage = fraction of query content words present; repairs = chunks swapped in to hold the coverage floor"
    }))
}


/// Task 585 — verify a claim against retrieved evidence. UNKNOWN is a verdict, not a failure.
pub fn claim(body: &Value) -> Result<Value, String> {
    let text = body.get("claim").and_then(|v| v.as_str()).ok_or("'claim' must be a string")?;
    let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(8) as usize;
    let hits = evidence_index::search(text, k);
    // evidence is sentences, not chunks: a chunk that supports half a claim should not be scored as if it
    // supported the whole thing
    let mut sentences: Vec<String> = Vec::new();
    let mut provenance: Vec<(String, usize)> = Vec::new();
    for h in &hits {
        for s in split_sentences(&h.text) {
            sentences.push(s.text.clone());
            provenance.push((h.source.clone(), 0));
        }
    }
    let r = verify(text, &sentences);
    let evidence = r.evidence_index.map(|i| {
        let (src, _) = provenance.get(i).cloned().unwrap_or_default();
        let sentence = sentences.get(i).cloned().unwrap_or_default();
        let offset = read_source(&src)
            .ok()
            .and_then(|(ft, _, _)| trm_uefi_kernel::text_reading::span_normalised(&ft, &sentence))
            .map(|(a, b)| json!({"start": a, "end": b}));
        json!({"source": src, "sentence": sentence, "offset": offset})
    });
    Ok(json!({
        "claim": text,
        "verdict": r.verdict.as_str(),
        "reason": r.reason,
        "coverage": r.coverage,
        "evidence": evidence,
        "evidence_considered": sentences.len(),
        "semantics": "SUPPORTED requires shared content words with no numeric or polarity conflict; silence returns UNKNOWN, never REFUTED"
    }))
}

/// Task 587 — extract scientific claims (quantity, unit, polarity, hedging) with verbatim quantity checks.
pub fn scientific(body: &Value) -> Result<Value, String> {
    let path = body.get("path").and_then(|v| v.as_str()).ok_or("'path' must be a string")?;
    let limit = body.get("limit").and_then(|v| v.as_u64()).unwrap_or(300) as usize;
    let (text, rel, digest) = read_source(path)?;
    let mut claims = Vec::new();
    let mut verbatim_failures = 0usize;
    for s in split_sentences(&text).into_iter().take(limit) {
        let Some(c) = extract(&s.text) else { continue };
        if c.numbers.is_empty() {
            continue; // a claim without a quantity is out of scope for task 587
        }
        let mut nums = Vec::new();
        for q in trm_uefi_kernel::text_reading::quantities(&s.text) {
            // the literal must exist in the sentence at the exact bytes it claims
            let literal_ok = s.text[q.start..q.end] == q.text;
            if !literal_ok {
                verbatim_failures += 1;
            }
            nums.push(json!({
                "text": q.text, "value": q.value, "unit": q.unit,
                "start": s.start + q.start, "end": s.start + q.end,
                "literal_ok": literal_ok,
            }));
        }
        claims.push(json!({
            "sentence": s.text,
            "sentence_start": s.start,
            "negated": c.negated,
            "hedged": c.hedged,
            "quantities": nums,
        }));
    }
    Ok(json!({
        "file": rel, "sha256": digest,
        "claims": claims.len(),
        "detail": claims,
        "verbatim_failures": verbatim_failures,
        "semantics": "quantities are reported with their exact bytes; a quantity is only valid if text[start..end] == literal"
    }))
}

/// Task 589 — contradiction detection across one or more files.
pub fn contradictions(body: &Value) -> Result<Value, String> {
    // Explicit sentences are accepted so a harness can test constructed pairs (one number flipped) without
    // writing fixture files into the corpus. `source` labels them as constructed, never as corpus evidence.
    if let Some(arr) = body.get("sentences").and_then(|v| v.as_array()) {
        let mut items: Vec<(String, String)> = Vec::new();
        for (i, v) in arr.iter().enumerate() {
            let text = v.as_str().ok_or_else(|| format!("sentences[{i}] must be a string"))?;
            items.push((text.to_string(), format!("constructed:{i}")));
        }
        let mut found = Vec::new();
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                if let Some(c) = contradicts(&items[i].0, &items[j].0) {
                    found.push(json!({
                        "kind": c.kind.as_str(), "detail": c.detail, "coverage": c.coverage,
                        "a": {"index": i, "text": items[i].0},
                        "b": {"index": j, "text": items[j].0},
                    }));
                }
            }
        }
        return Ok(json!({
            "mode": "constructed_sentences",
            "sentences": items.len(),
            "pairs_compared": items.len() * items.len().saturating_sub(1) / 2,
            "contradictions": found.len(),
            "detail": found,
            "semantics": "same subject coverage required both ways, then a numeric or polarity conflict"
        }));
    }

    let paths: Vec<String> = body
        .get("paths")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .or_else(|| body.get("path").and_then(|v| v.as_str()).map(|s| vec![s.to_string()]))
        .ok_or("provide 'path', 'paths' or 'sentences'")?;
    let max_pairs = body.get("max_pairs").and_then(|v| v.as_u64()).unwrap_or(200_000) as usize;

    struct Item { file: String, start: usize, text: String }
    let mut items: Vec<Item> = Vec::new();
    for path in &paths {
        let (text, rel, _) = read_source(path)?;
        for s in split_sentences(&text) {
            if s.text.len() < 40 {
                continue; // too short to carry a claim
            }
            items.push(Item { file: rel.clone(), start: s.start, text: s.text });
        }
    }
    let mut found = Vec::new();
    let mut compared = 0usize;
    'outer: for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            compared += 1;
            if compared > max_pairs {
                break 'outer;
            }
            if let Some(c) = contradicts(&items[i].text, &items[j].text) {
                found.push(json!({
                    "kind": c.kind.as_str(),
                    "detail": c.detail,
                    "coverage": c.coverage,
                    "a": {"file": items[i].file, "start": items[i].start, "text": items[i].text},
                    "b": {"file": items[j].file, "start": items[j].start, "text": items[j].text},
                }));
            }
        }
    }
    Ok(json!({
        "files": paths.len(),
        "sentences": items.len(),
        "pairs_compared": compared.min(max_pairs),
        "contradictions": found.len(),
        "detail": found,
        "semantics": "a contradiction requires shared subject coverage in both directions, then a numeric or polarity conflict"
    }))
}


/// Task 588 — evidence-based recommendation: rank options by the evidence that supports them, and refuse when
/// there is none.
///
/// The failure mode this guards against is a recommender that launders a preference as evidence. It refuses
/// (verdict INSUFFICIENT_EVIDENCE) when an option has no retrievable support, and it returns the citations for
/// every option it ranks so a reader can check whether the ranking follows the evidence or the phrasing.
pub fn recommend(body: &Value) -> Result<Value, String> {
    let options = body.get("options").and_then(|v| v.as_array()).ok_or("'options' must be an array of {id, query}")?;
    if options.len() < 2 {
        return Err("a recommendation needs at least two options".to_string());
    }
    let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    // SUPPORT IS LEXICAL, NOT COSINE. A nonsense option scored 0.50 mean cosine — the embedder places unrelated
    // text close enough that a similarity floor either admits nonsense or refuses real queries. So similarity
    // decides the ORDER, and whether the best citation names the option decides whether there is support at all.
    // REFUSAL IS FOR HAVING NO CITATIONS, NOT FOR THIN ONES. The pre-registered test was "citation-free input must
    // be refused", and that is what this does: an option with no retrievable evidence is refused rather than
    // ranked. Lexical support is computed and REPORTED per option as a metric — measuring it showed it is
    // unreliable in both directions on this corpus (a nonsense query scored 0.68 cosine against a grounding
    // document that mentions 'maintenance'; a genuine Lipinski query retrieved a chunk containing none of its
    // words). A caller can pass `min_support` to gate on it, but the default reports rather than discards.
    // Measured: the embedder returns *something* for any input (gibberish scored 0.53 cosine), so "no citations"
    // never happens by retrieval returning empty. What does separate them is whether the best citation NAMES the
    // option: gibberish 0.00, a documented topic 0.75. So zero support is the refusal criterion, and any support
    // above zero is ranked with the exact figure reported.
    let min_support = body.get("min_support").and_then(|v| v.as_f64()).unwrap_or(0.01);

    let mut scored: Vec<(String, String, f64, f64, Vec<Value>)> = Vec::new(); // id, query, weight, support, citations
    let mut unsupported: Vec<String> = Vec::new();
    let mut weak: Vec<Value> = Vec::new();
    for (i, o) in options.iter().enumerate() {
        let id = o.get("id").and_then(|v| v.as_str()).ok_or_else(|| format!("option[{i}] has no 'id'"))?;
        let q = o.get("query").and_then(|v| v.as_str()).ok_or_else(|| format!("option[{i}] has no 'query'"))?;
        let hits = evidence_index::search(q, k);
        if hits.is_empty() {
            unsupported.push(id.to_string());
            continue;
        }
        let weight = hits.iter().map(|h| h.score as f64).sum::<f64>() / hits.len() as f64;
        let cites: Vec<Value> = hits.iter().map(|h| json!({"source": h.source, "score": h.score})).collect();
        let query_tokens: Vec<String> = q
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() > 2)
            .map(|t| t.to_lowercase())
            .collect();
        // The BEST citation must name the option. Unioning every retrieved chunk admitted nonsense: "unicycle
        // maintenance schedule for winter storage" found "maintenance" and "schedule" somewhere in the corpus.
        // One document has to actually talk about the thing being recommended.
        let joined = hits[0].text.to_lowercase();
        let support = if query_tokens.is_empty() {
            0.0
        } else {
            query_tokens.iter().filter(|t| joined.contains(t.as_str())).count() as f64 / query_tokens.len() as f64
        };
        if support < min_support {
            weak.push(json!({"id": id, "weight": weight, "lexical_support": support, "min_support": min_support}));
            continue;
        }
        // the support figure travels with the citations so a reader can see how much of the option the evidence names
        scored.push((id.to_string(), q.to_string(), weight, support, cites));
    }

    if !unsupported.is_empty() || !weak.is_empty() {
        return Ok(json!({
            "verdict": "INSUFFICIENT_EVIDENCE",
            "unsupported": unsupported,
            "below_support": weak,
            "min_support": min_support,
            "note": "an option with no retrievable evidence, or with evidence below the strength floor, is refused rather than ranked - ranking it would be a preference dressed as a finding"
        }));
    }

    scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    Ok(json!({
        "verdict": "RANKED",
        "min_support": min_support,
        "ranking": scored.iter().enumerate().map(|(rank, (id, q, w, support, cites))| json!({
            "rank": rank + 1, "id": id, "query": q, "evidence_weight": w,
            "lexical_support": support,
            "citations": cites
        })).collect::<Vec<_>>(),
        "semantics": "weight = mean cosine of retrieved evidence; every ranked option carries its citations, and an option with no evidence is refused rather than ranked"
    }))
}

/// Task 590 — knowledge synthesis: sections of evidence sentences, each carrying an id that resolves.
///
/// Every sentence emitted is an extractive span with file, offset and sha256 — there is no generated prose in this
/// endpoint, so a sentence cannot imply support it does not have. Anything that is not a corpus span would be
/// tagged INFERENCE; nothing here is, and a caller that adds inference must tag it or fail the check.
pub fn synthesis(body: &Value) -> Result<Value, String> {
    let topic = body.get("topic").and_then(|v| v.as_str()).ok_or("'topic' must be a string")?;
    let k = body.get("k").and_then(|v| v.as_u64()).unwrap_or(6) as usize;
    let per_source = body.get("per_source").and_then(|v| v.as_u64()).unwrap_or(2) as usize;

    let hits = evidence_index::mmr(topic, k, 0.6, 30);
    if hits.is_empty() {
        return Ok(json!({"topic": topic, "verdict": "NO_EVIDENCE", "sections": []}));
    }

    let mut by_source: std::collections::BTreeMap<String, Vec<Value>> = Default::default();
    let mut emitted = 0usize;
    let mut unresolved = 0usize;
    for h in &hits {
        let Ok((ftext, rel, digest)) = read_source(&h.source) else { continue };
        let mut taken = 0usize;
        for s in split_sentences(&h.text) {
            if taken >= per_source {
                break;
            }
            let Some((a, b)) = trm_uefi_kernel::text_reading::span_normalised(&ftext, &s.text) else {
                unresolved += 1;
                continue;
            };
            let exact = ftext[a..b].to_string();
            by_source.entry(rel.clone()).or_default().push(json!({
                "claim": exact,
                "evidence_id": format!("{}@{}", rel, a),
                "file": rel,
                "offset": a,
                "end": b,
                "sha256": digest,
                "tag": "EVIDENCE",
            }));
            taken += 1;
            emitted += 1;
        }
    }

    let sections: Vec<Value> = by_source.into_iter().map(|(file, claims)| json!({
        "source": file,
        "claims": claims,
    })).collect();

    Ok(json!({
        "topic": topic,
        "verdict": "SYNTHESISED",
        "sections": sections,
        "sentences": emitted,
        "unresolved_spans": unresolved,
        "semantics": "every sentence is an extractive span with a resolving offset; an INFERENCE tag would be required for anything not taken from the corpus, and none is emitted here"
    }))
}

/// Task 586 — the machine reading suite index, with module presence checked on disk at request time.
pub fn suite() -> Value {
    let kernel_src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src");
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let tasks: [(u32, &str, &str, &str, &str); 10] = [
        (581, "document understanding", "text_reading.rs", "POST /reading/parse", "tools/reading_document_verify.py"),
        (582, "reading comprehension", "reading_routes.rs", "POST /reading/qa", "tools/reading_qa_verify.py"),
        (583, "multi-document summarisation", "evidence_index.rs", "POST /reading/summarize", "tools/reading_summarize_verify.py"),
        (584, "evidence extraction", "text_reading.rs", "POST /reading/evidence", "tools/reading_evidence_verify.py"),
        (585, "claim verification", "claims.rs", "POST /reading/claim", "tools/reading_claim_verify.py"),
        (586, "complete reading suite", "reading_routes.rs", "GET /reading/suite", "tools/reading_suite_verify.py"),
        (587, "scientific claim extraction", "claims.rs", "POST /reading/scientific", "tools/reading_scientific_verify.py"),
        (588, "evidence-based recommendation", "reading_routes.rs", "POST /reading/recommend", "tools/reading_recommend_verify.py"),
        (589, "contradiction detection", "claims.rs", "POST /reading/contradictions", "tools/reading_contradiction_verify.py"),
        (590, "knowledge synthesis report", "reading_routes.rs", "POST /reading/synthesis", "tools/reading_synthesis_verify.py"),
    ];
    let entries: Vec<Value> = tasks.iter().map(|(id, name, module, endpoint, harness)| {
        let present = kernel_src.join(module).exists() || here.join(module).exists();
        json!({
            "id": id, "name": name, "module": module,
            "module_present": present, "endpoint": endpoint, "harness": harness,
            "status": if present { "present" } else { "ABSENT" },
        })
    }).collect();
    let present = entries.iter().filter(|e| e["module_present"] == json!(true)).count();
    json!({
        "suite": entries,
        "present_modules": present,
        "total_tasks": 10,
        "note": "module_present is a filesystem check at request time; behaviour is proven by the named harness, not by the file existing"
    })
}
