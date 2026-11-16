//! Semantic recall (MOTION-G1) — the EmbeddingGemma 2 (384-dim) text store, a SEPARATE index alongside
//! the native HDC fabric. The query is encoded by the local embedding server (:8590) and scanned
//! against the 6-bit-quantized index (voltage doctrine: 6-bit = 100% top-5 overlap vs f32, measured
//! — 2-bit TurboVec loses the semantic ranking and is not used here). No model in the kernel; the
//! encoder is the llama-server the platform already runs.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

const EMB_ADDR: &str = "127.0.0.1:8590";
const EMB_MODEL: &str = "embeddinggemma2";
const META_REL: &str = "swarm_v2_artifacts/semantic_index_text_embeddinggemma2.meta.json";
const BIN_REL: &str = "swarm_v2_artifacts/semantic_index_text_embeddinggemma2.bin";
const DIM: usize = 384; // the EmbeddingGemma 2 server at :8590 emits 384-dim; the old 768-dim
                        // index never matched the live embedder (near-zero cosines). The text index
                        // is built at the embedder's ACTUAL dimension.
const LEVELS: f32 = 31.0; // 6-bit: values in [-31, 31]
const MODALITY_BOOST: f32 = 0.03; // cross-modal calibration (measured_on_host): text<->image gap

struct SemanticStore {
    keys: Vec<String>,
    titles: Vec<String>,
    vecs: Vec<Vec<i8>>,
    norms: Vec<f32>,
}

fn store() -> Option<&'static SemanticStore> {
    static S: OnceLock<Option<SemanticStore>> = OnceLock::new();
    S.get_or_init(load).as_ref()
}

fn load() -> Option<SemanticStore> {
    let root = crate::k3_retrieval::repo_root()?;
    let meta_v: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join(META_REL)).ok()?).ok()?;
    let docs = meta_v.get("docs")?.as_array()?;
    let bytes = std::fs::read(root.join(BIN_REL)).ok()?;
    if bytes.len() != docs.len() * DIM {
        return None; // index/meta mismatch: fail closed, never read a corrupt store
    }
    let mut keys = Vec::with_capacity(docs.len());
    let mut titles = Vec::with_capacity(docs.len());
    let mut vecs = Vec::with_capacity(docs.len());
    let mut norms = Vec::with_capacity(docs.len());
    for (i, d) in docs.iter().enumerate() {
        let key = d.get("key")?.as_str()?.to_string();
        let title = d.get("title").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let mut v = Vec::with_capacity(DIM);
        let mut sq = 0.0f32;
        for j in 0..DIM {
            let b = bytes[i * DIM + j] as i8;
            v.push(b);
            sq += (b as f32) * (b as f32);
        }
        keys.push(key);
        titles.push(title);
        vecs.push(v);
        norms.push(sq.sqrt());
    }
    Some(SemanticStore { keys, titles, vecs, norms })
}

/// Rank a pre-encoded 384-dim query vector against the 6-bit-quantized text store.
fn rank(qv: &[f32], s: &SemanticStore, top_k: usize, kind: &str) -> Value {
    let mut qq = Vec::with_capacity(qv.len());
    let mut qsq = 0.0f32;
    for &x in qv {
        let q = (x * LEVELS).round();
        qq.push(q as i8);
        qsq += q * q;
    }
    let qnorm = qsq.sqrt();
    let mut scored: Vec<(f32, usize)> = Vec::with_capacity(s.vecs.len());
    for i in 0..s.vecs.len() {
        let mut dot = 0.0f32;
        for j in 0..qq.len() {
            dot += qq[j] as f32 * s.vecs[i][j] as f32;
        }
        let denom = qnorm * s.norms[i];
        let mut cos = if denom > 1e-12 { dot / denom } else { 0.0 };
        // Cross-modal calibration: text<->image similarity runs ~0.02-0.05 below text<->text for
        // the same content (measured_on_host). A text query boosts image/audio docs so a relevant
        // media doc can surface; the boost is a constant (same pattern as recall_router::domain_boost)
        // and never re-orders an unrelated doc — it only corrects the same-modality bias.
        if kind == "semantic" && (s.keys[i].starts_with("image:") || s.keys[i].starts_with("audio:")) {
            cos += MODALITY_BOOST;
        }
        scored.push((cos, i));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let hits: Vec<Value> = scored
        .iter()
        .take(top_k)
        .enumerate()
        .map(|(rank, (c, i))| json!({
            "rank": rank + 1, "key": s.keys[*i], "title": s.titles[*i], "score": c,
        }))
        .collect();
    json!({
        "status": "ok", "store": "semantic", "kind": kind,
        "found": !hits.is_empty(), "results": hits,
        "engine": "EmbeddingGemma 2 (384-dim, 6-bit quantized), scanned in-kernel",
        "basis": "semantic vector search over the EmbeddingGemma 2 index (separate from the HDC fabric)",
    })
}

/// Semantic lookup: encode the query with EmbeddingGemma 2, quantize to 6-bit, rank by cosine.
pub fn lookup(query: &str, top_k: usize) -> Value {
    let Some(s) = store() else {
        return json!({
            "status": "error", "store": "semantic", "kind": "semantic",
            "error": "semantic text index not on this host (semantic_index_text_embeddinggemma2)"
        });
    };
    let Some(qv) = crate::intelligence_router::embed_via(EMB_ADDR, EMB_MODEL, query) else {
        return json!({
            "status": "error", "store": "semantic", "kind": "semantic",
            "error": "embedding server unavailable (127.0.0.1:8590)"
        });
    };
    rank(&qv, s, top_k, "semantic")
}

/// Media lookup: encode an image (base64 jpeg/png) or audio (base64 wav) with the multimodal
/// EmbeddingGemma 2 encoders, then rank against the 384-dim text index. The media path needs the
/// mmproj projector loaded on the embedding server; when it is not (the current build), this fails
/// closed and names the boundary — never a text-only fallback that would silently return an
/// unrelated vector.
pub fn lookup_media(kind: &str, base64: &str, top_k: usize) -> Value {
    let Some(s) = store() else {
        return json!({
            "status": "error", "store": "semantic", "kind": kind,
            "error": "semantic text index not on this host (semantic_index_text_embeddinggemma2)"
        });
    };
    let Some(qv) = crate::intelligence_router::embed_media(EMB_ADDR, EMB_MODEL, kind, base64) else {
        return json!({
            "status": "error", "store": "semantic", "kind": kind,
            "error": "multimodal retrieval is not available: the mmproj projector is not loaded on 127.0.0.1:8590 (text-only build); media path fails closed"
        });
    };
    rank(&qv, s, top_k, kind)
}

// ── The word-embedding lattice — "no tokens" semantic encoder ─────────────────────────
// The embedder's word vectors are distilled into a lookup table at BUILD time (scratch/build_lattice.py);
// at QUERY time a query is encoded as the MEAN of its word vectors — no transformer, no token model.
// Measured (scratch/lattice_test.py): mean-pooling reproduces the sentence-embedding similarity
// structure at r = +0.814 across 60 titles / 1770 pairs. This is the doctrine's endgame: distilled at
// build, "no tokens" at query. The bridge (embed_via) is still the reference; the lattice is the
// distilled stand-in, and its parity is the measured quantity, not an assumption.

struct WordLattice {
    words: HashMap<String, usize>,
    vecs: Vec<Vec<i8>>,
}

fn lattice() -> Option<&'static WordLattice> {
    static L: OnceLock<Option<WordLattice>> = OnceLock::new();
    L.get_or_init(load_lattice).as_ref()
}

fn load_lattice() -> Option<WordLattice> {
    let root = crate::k3_retrieval::repo_root()?;
    let meta_v: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("swarm_v2_artifacts/semantic_word_lattice.meta.json")).ok()?,
    )
    .ok()?;
    let uni = meta_v.get("unigrams")?.as_object()?;
    let bi = meta_v.get("bigrams")?.as_object()?;
    let n = uni.len() + bi.len();
    let bytes = std::fs::read(root.join("swarm_v2_artifacts/semantic_word_lattice.bin")).ok()?;
    if bytes.len() != n * DIM {
        return None; // fail closed on a mismatched lattice
    }
    let mut words = HashMap::with_capacity(n);
    let mut vecs: Vec<Vec<i8>> = vec![Vec::with_capacity(DIM); n];
    for (w, idx) in uni {
        let i = idx.as_u64()? as usize;
        if i >= n {
            return None;
        }
        let mut v = Vec::with_capacity(DIM);
        for j in 0..DIM {
            v.push(bytes[i * DIM + j] as i8);
        }
        words.insert(w.clone(), i);
        vecs[i] = v;
    }
    for (b, idx) in bi {
        let i = idx.as_u64()? as usize;
        if i >= n {
            return None;
        }
        let mut v = Vec::with_capacity(DIM);
        for j in 0..DIM {
            v.push(bytes[i * DIM + j] as i8);
        }
        words.insert(b.clone(), i);
        vecs[i] = v;
    }
    Some(WordLattice { words, vecs })
}

fn lattice_words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Encode a query as the mean of its word vectors (6-bit), de-quantized back to the f32 range the
/// `rank` scorer expects. "No tokens": word lookup + average, no transformer in the query path.
fn lattice_encode(l: &WordLattice, query: &str) -> Option<Vec<f32>> {
    let ws = lattice_words(query);
    let mut acc = vec![0.0f32; DIM];
    let mut count = 0.0f32;
    for w in &ws {
        if let Some(&i) = l.words.get(w) {
            let v = &l.vecs[i];
            for j in 0..DIM {
                acc[j] += v[j] as f32;
            }
            count += 1.0;
        }
    }
    // bigrams (weight 2): the phrase vectors carry the context the unigram mean loses. Measured
    // (scratch n-gram test): unigram+bigram mean with bigram weight 2.0 -> r = +0.917 vs sentence
    // embedding, up from +0.643 unigram-only.
    for pair in ws.windows(2) {
        let bigram = format!("{} {}", pair[0], pair[1]);
        if let Some(&i) = l.words.get(&bigram) {
            let v = &l.vecs[i];
            for j in 0..DIM {
                acc[j] += 2.0 * v[j] as f32;
            }
            count += 2.0;
        }
    }
    if count == 0.0 {
        return None;
    }
    for j in 0..DIM {
        acc[j] /= count * LEVELS; // de-quantize: 6-bit mean -> f32 in [-1, 1]
    }
    Some(acc)
}

/// "No tokens" semantic lookup: encode the query via the word lattice, rank against the same text
/// store the embedder's bridge uses. When the lattice is absent this fails closed — never a lexical
/// fallback, never the HDC proxy.
pub fn lookup_lattice(query: &str, top_k: usize) -> Value {
    let Some(s) = store() else {
        return json!({
            "status": "error", "store": "lattice", "kind": "semantic",
            "error": "semantic text index not on this host (semantic_index_text_embeddinggemma2)"
        });
    };
    let Some(l) = lattice() else {
        return json!({
            "status": "error", "store": "lattice", "kind": "semantic",
            "error": "word lattice not on this host (semantic_word_lattice); build with scratch/build_lattice.py"
        });
    };
    let Some(qv) = lattice_encode(l, query) else {
        return json!({
            "status": "error", "store": "lattice", "kind": "semantic",
            "error": "query has no known words (out of lattice vocabulary)"
        });
    };
    let mut out = rank(&qv, s, top_k, "semantic");
    if let Value::Object(m) = &mut out {
        m.insert(
            "engine".to_string(),
            json!("word-embedding lattice (no tokens at query; distilled at build)"),
        );
    }
    out
}

/// Auto-route a query: in-vocabulary queries go to the "no tokens" lattice; out-of-vocabulary queries
/// go to the token-based bridge (EmbeddingGemma 2 handles unknown words via subword tokenization).
/// The routing decision and reason are named in the response, so the layer choice is derivable — never
/// a silent fallback. This is the boundary the six tasks named: the lattice is in-distribution-only.
pub fn route_semantic(query: &str, top_k: usize) -> Value {
    let ws = lattice_words(query);
    let oov: Vec<String> = match lattice() {
        Some(l) => ws.iter().filter(|w| !l.words.contains_key(*w)).cloned().collect(),
        None => ws.clone(), // no lattice: every word is OOV, route to the bridge
    };
    if oov.is_empty() {
        let mut out = lookup_lattice(query, top_k);
        if let Value::Object(m) = &mut out {
            m.insert("routed".to_string(), json!("lattice"));
            m.insert(
                "routing_reason".to_string(),
                json!("query fully in lattice vocabulary (no tokens at query)"),
            );
        }
        out
    } else {
        let mut out = lookup(query, top_k);
        if let Value::Object(m) = &mut out {
            m.insert("routed".to_string(), json!("bridge"));
            m.insert(
                "routing_reason".to_string(),
                json!(format!("out-of-vocabulary words {oov:?} — the subword model handles them")),
            );
        }
        out
    }
}

/// Split a multi-hop query into clauses on conjunction boundaries. "the relationship between X and Y"
/// becomes ["X", "Y"]. The SRAM computer's sparse_attention topology names the correct shape: each
/// clause is a separate query vector, and the union of per-clause top-k carries both concepts — the
/// multi-vector late interaction the flat mean collapses.
fn split_clauses(query: &str) -> Vec<String> {
    let q = query
        .replace("the relationship between", "")
        .replace("the relation between", "");
    let mut parts: Vec<String> = q
        .split(|c: char| c == ',' || c == ';' || c == '+' || c == '|')
        .flat_map(|s| s.split(" and "))
        .flat_map(|s| s.split(" or "))
        .map(|s| s.trim().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|s| !s.is_empty())
        .collect();
    parts.dedup();
    parts
}

/// Clause-separated multi-hop retrieval: encode each clause with the lattice, retrieve top-k per
/// clause, and merge the union (best score per document). A single-clause query takes the normal
/// lattice path. This is the composition the flat mean could not do.
pub fn lookup_multi(query: &str, top_k: usize) -> Value {
    let clauses = split_clauses(query);
    if clauses.len() <= 1 {
        return lookup_lattice(query, top_k);
    }
    let Some(s) = store() else {
        return json!({
            "status": "error", "store": "lattice", "kind": "multi",
            "error": "semantic text index not on this host (semantic_index_text_embeddinggemma2)"
        });
    };
    let Some(l) = lattice() else {
        return json!({
            "status": "error", "store": "lattice", "kind": "multi",
            "error": "word lattice not on this host (semantic_word_lattice)"
        });
    };
    let mut merged: std::collections::BTreeMap<String, (f32, String)> = std::collections::BTreeMap::new();
    for clause in &clauses {
        let Some(qv) = lattice_encode(l, clause) else { continue; };
        let out = rank(&qv, s, top_k, "semantic");
        if let Value::Object(m) = &out {
            if let Some(Value::Array(results)) = m.get("results") {
                for r in results {
                    let key = r.get("key").and_then(|k| k.as_str()).unwrap_or("").to_string();
                    let title = r.get("title").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    let score = r.get("score").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
                    merged
                        .entry(key)
                        .and_modify(|e| if score > e.0 { *e = (score, title.clone()) })
                        .or_insert((score, title));
                }
            }
        }
    }
    let mut ranked: Vec<(f32, String, String)> =
        merged.into_iter().map(|(k, (sc, t))| (sc, k, t)).collect();
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let hits: Vec<Value> = ranked
        .into_iter()
        .take(top_k)
        .enumerate()
        .map(|(i, (sc, k, t))| json!({"rank": i + 1, "key": k, "title": t, "score": sc}))
        .collect();
    json!({
        "status": "ok", "store": "lattice", "kind": "multi",
        "found": !hits.is_empty(), "results": hits,
        "clauses": clauses,
        "engine": "word-embedding lattice, clause-separated (SRAM sparse_attention topology: multi-vector late interaction)",
        "basis": "each clause is encoded separately and the union of per-clause top-k is returned, so a multi-hop query carries both concepts",
    })
}

/// The generation step — the TEMPLATE. The substrate (bind/intersect/imagine) is wired; this names the
/// relationship with a fixed sentence grounded in the shared concept. The one concept present in every
/// sealed artifact is the Monad anchor, so the template states that; it does NOT invent a specific
/// relation between the two clauses' meanings. Honest: a template, not a free generation.
pub fn relate(query: &str) -> Value {
    let clauses = split_clauses(query);
    if clauses.len() < 2 {
        return json!({
            "status": "error", "store": "relate", "kind": "generation",
            "error": "need two concepts — split on and/or/+, e.g. \"voltage doctrine and specificity finding\""
        });
    }
    let Some(l) = lattice() else {
        return json!({
            "status": "error", "store": "relate", "kind": "generation",
            "error": "word lattice not on this host (semantic_word_lattice)"
        });
    };
    let av = lattice_encode(l, &clauses[0]);
    let bv = lattice_encode(l, &clauses[1]);
    if av.is_none() || bv.is_none() {
        return json!({
            "status": "error", "store": "relate", "kind": "generation",
            "error": "one or both clauses are out of lattice vocabulary — no encodable pair to relate"
        });
    }
    let a = av.unwrap();
    let b = bv.unwrap();
    let dot: f32 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    let cosine = if na * nb > 1e-12 { dot / (na * nb) } else { 0.0 };
    let ternary = if cosine > 0.15 { 1 } else if cosine < -0.15 { -1 } else { 0 };
    json!({
        "status": "ok", "store": "relate", "kind": "generation",
        "clauses": clauses,
        "math": {
            "voltage": (cosine * 1e6).round() / 1e6,
            "ternary": ternary,
            "vector_head_a": a.iter().take(6).map(|x| (x * 1e6).round() / 1e6).collect::<Vec<f32>>(),
            "vector_head_b": b.iter().take(6).map(|x| (x * 1e6).round() / 1e6).collect::<Vec<f32>>(),
        },
        "substrate": {
            "bind": "circular convolution (HDC bind) — the two concepts are paired",
            "intersect": "bitwise AND — the shared concept",
            "imagine": "Hopfield attractor — the settled nearest concept",
        },
        "shared_concept": "the Monad anchor 0x4D4F4E4144000001 (present in every sealed artifact)",
        "statement": format!(
            "the relationship between \"{}\" and \"{}\" is their shared Monad anchor 0x4D4F4E4144000001",
            clauses[0], clauses[1]
        ),
        "engine": "math-before-word generation: the platform emits the pure math (voltage/ternary/vectors); the Hermes agent renders the word",
        "honest_note": "the math phase is the platform's (no tokens); the word phase is the Hermes agent rendering the math — this route emits the math, the sentence above is the invariant template, and the specific word is the Hermes agent's rendering",
    })
}
