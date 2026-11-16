//! The SHARED MEMORY CORPUS, loaded from the gzip JSONL store this host actually has.
//!
//! `/memory/stats` used to answer `total_memories: null`, `by_type: {}` and a `corpus` block of
//! nulls. The reason was not a broken route: `intel_reservoir::stats_payload` derived its census
//! from `packed_ternary::store()` (`.turbovec` packed store) and its meta block from
//! `swarm_v2_artifacts/kernel_knowledge_index.json`. Neither artifact exists on this machine. The
//! corpus that DOES exist is the agent memory fabric written by
//! `swarm_v2/core/turbovec_memory.py` and deduplicated into `fabric/fabric_distinct.jsonl.gz` by
//! the fabric builder -- 42,805 deduped rows behind 6.2 MB of gzip. Nothing was reading it.
//!
//! This module streams that gzip, parses the JSONL rows into an in-memory corpus, and measures:
//!   * `documents` -- rows successfully parsed (and the rows that were not, counted separately),
//!   * `by_type` -- the corpus's own filing: the leading path segment of each key where the key is
//!     a path, otherwise the row's own `memory_type`/`class` field (see `type_of_key`),
//!   * `by_author` -- the row's `agent.author` (or `agent` when the legacy shape put a string there),
//!   * the newest/oldest `timestamp` the rows carry,
//!   * retrieval over the stored text, keyword/exact, scored as the Python reader scores it
//!     (`turbovec_memory.query_scored`: |matched content terms| / |content terms|).
//!
//! Every number a caller reports comes from here as a measurement. A corpus that cannot be read is
//! an `Err`, which the routes turn into a null WITH its reason -- never a 0, and never a constant.

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

/// The corpora this host keeps, most-authoritative first. `fabric_distinct` is the deduped fabric
/// (one row per distinct memory); `turbo_memory` is the raw append log it was derived from.
pub const CORPUS_RELS: [&str; 2] = [
    "swarm_v2_global_memory/fabric/fabric_distinct.jsonl.gz",
    "swarm_v2_global_memory/turbo_memory.jsonl.gz",
];

/// The embedding artifact that sits beside the deduped fabric: `fabric_ids.json` names the rows in
/// order, `fabric_vecs.npy` holds their vectors. Kept as a separate measurement so `dim` is only
/// ever reported from a file that was actually read.
pub const IDS_REL: &str = "swarm_v2_global_memory/fabric/fabric_ids.json";
pub const VECS_REL: &str = "swarm_v2_global_memory/fabric/fabric_vecs.npy";

/// Per-document text kept in memory. The fabric builder embeds at most 1,200 chars
/// (`fabric_manifest.json: max_chars`) and the longest rows in the live file are sub-10 KiB
/// continuations; 8 KiB covers essentially all of them while bounding RSS. Rows over the cap are
/// counted in `truncated_rows` so the number is visible rather than silently applied.
pub const TEXT_CAP: usize = 8 * 1024;

/// Function words excluded from a query, matching `turbovec_memory.STOPWORDS` so the Rust score and
/// the Python score mean the same thing. A query of only these words matches nothing.
const STOPWORDS: [&str; 37] = [
    "the", "and", "for", "are", "but", "not", "you", "all", "any", "can", "had", "her", "was",
    "one", "our", "out", "day", "get", "has", "him", "his", "how", "its", "may", "new", "now",
    "old", "see", "two", "way", "who", "did", "yes", "with", "that", "this", "from",
];

// ── Row model ──────────────────────────────────────────────────────────────────────────

/// One corpus document, parsed from one JSONL row.
#[derive(Clone, Debug)]
pub struct Doc {
    /// The row's key. `id`, else `norm_key`, else the first entry of `ids`.
    pub id: String,
    /// The row's text: `content`, else `text`/`document`. Original case, capped at `TEXT_CAP`.
    pub text: String,
    /// `agent.author` when `agent` is the object shape, the string itself for the legacy shape.
    pub agent: String,
    /// The row's own filing label: `metadata.memory_type`, else `class`, else `agent.memory_type`.
    pub mtype: String,
    /// `timestamp`, else `metadata.created_at`.
    pub ts: String,
    /// Length of the text BEFORE the cap, so a truncated document is still counted honestly.
    pub text_chars: usize,
}

/// A loaded corpus plus everything measured while loading it.
#[derive(Debug)]
pub struct Corpus {
    pub source: String,
    pub gz_bytes: u64,
    pub docs: Vec<Doc>,
    pub by_type: BTreeMap<String, u64>,
    pub by_author: BTreeMap<String, u64>,
    pub rows_read: u64,
    pub malformed_rows: u64,
    pub empty_text_rows: u64,
    pub truncated_rows: u64,
    pub duplicate_ids: u64,
    pub newest_ts: Option<String>,
    pub oldest_ts: Option<String>,
    pub loaded_at_unix: u64,
}

impl Corpus {
    pub fn documents(&self) -> usize {
        self.docs.len()
    }

    /// Keyword/exact retrieval: the query's content terms (lowercased, stopwords and <=2-char words
    /// dropped, exactly as `turbovec_memory.query_scored` builds them) are searched for as substrings
    /// of each document's text, case-insensitively. The score is matched/|terms|, so a document that
    /// matches every term scores 1.0. Ties break on the newer timestamp, then on id, so the ranking is
    /// reproducible rather than dependent on load order.
    pub fn search(&self, query: &str, k: usize) -> (Vec<Hit>, Vec<String>) {
        let terms = query_terms(query);
        if terms.is_empty() || k == 0 {
            return (Vec::new(), terms);
        }
        let mut hits: Vec<Hit> = Vec::new();
        for d in &self.docs {
            let mut matched: Vec<&str> = Vec::new();
            for t in &terms {
                if contains_ci(&d.text, t) {
                    matched.push(t.as_str());
                }
            }
            if matched.is_empty() {
                continue;
            }
            hits.push(Hit {
                score: matched.len() as f64 / terms.len() as f64,
                matched: matched.iter().map(|s| s.to_string()).collect(),
                doc: d.clone(),
            });
        }
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.doc.ts.cmp(&a.doc.ts))
                .then_with(|| a.doc.id.cmp(&b.doc.id))
        });
        hits.truncate(k);
        (hits, terms)
    }
}

/// One scored document.
#[derive(Clone, Debug)]
pub struct Hit {
    pub score: f64,
    pub matched: Vec<String>,
    pub doc: Doc,
}

/// The query's content terms, in the order they were first seen (deduped).
pub fn query_terms(query: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for w in query.to_lowercase().split_whitespace() {
        let w = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
        if w.chars().count() <= 2 || STOPWORDS.contains(&w) {
            continue;
        }
        if !out.iter().any(|x| x == w) {
            out.push(w.to_string());
        }
    }
    out
}

/// Case-insensitive substring search without allocating a lowercased copy of the document. ASCII
/// folding only, which is what the Python `w in content.lower()` comparison does for the ASCII terms
/// this route is given; non-ASCII bytes compare exactly.
fn contains_ci(haystack: &str, needle: &str) -> bool {
    let (h, n) = (haystack.as_bytes(), needle.as_bytes());
    if n.is_empty() || n.len() > h.len() {
        return false;
    }
    h.windows(n.len()).any(|w| {
        w.iter()
            .zip(n.iter())
            .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
    })
}

// ── Parsing ───────────────────────────────────────────────────────────────────────────

/// The type label for a document.
///
/// Primary rule (the corpus's own filing scheme): the leading path segment of the key --
/// `papers/2024/x` is filed under `papers`. This host's live keys are content hashes
/// (`36ed6a1b6200533b`) with no path (verified: 0 of 42,805 keys contain a `/`), so the rule falls
/// through to the row's own type field (`metadata.memory_type`, else `class`, else the agent
/// object's `memory_type` -- see `parse_row`) and finally to `root` for a row that files itself
/// nowhere. `root` is a real bucket -- a document that carries no type -- not a bucket named after
/// its own filename.
pub fn type_of_key(key: &str, mtype: &str) -> String {
    if let Some((dir, _)) = key.split_once('/') {
        if !dir.is_empty() {
            return dir.to_string();
        }
    }
    if !mtype.is_empty() {
        return mtype.to_string();
    }
    "root".to_string()
}

fn str_field<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(|x| x.as_str())
}

/// Parse one JSONL row into a `Doc`. `Err` means the row is not a usable document (bad JSON or no
/// text at all); the caller counts those instead of dropping them silently.
pub fn parse_row(line: &str) -> Result<Doc, String> {
    let v: Value = serde_json::from_str(line).map_err(|e| format!("json: {e}"))?;
    let id = str_field(&v, "id")
        .or_else(|| str_field(&v, "norm_key"))
        .or_else(|| {
            v.get("ids")
                .and_then(|x| x.as_array())
                .and_then(|a| a.first())
                .and_then(|x| x.as_str())
        })
        .unwrap_or("")
        .to_string();
    let raw_text = str_field(&v, "content")
        .or_else(|| str_field(&v, "text"))
        .or_else(|| str_field(&v, "document"))
        .unwrap_or("");
    let text_chars = raw_text.chars().count();
    if text_chars == 0 {
        return Err("row carries no text (content/text/document)".to_string());
    }
    // `agent` is an object in the newer rows (`{"author": ...}`) and a bare string in the legacy
    // ones. Both shapes are read; neither is guessed.
    let agent = match v.get("agent") {
        Some(Value::Object(_)) => v
            .get("agent")
            .and_then(|a| a.get("author"))
            .and_then(|a| a.as_str())
            .unwrap_or("unknown")
            .to_string(),
        Some(Value::String(s)) => s.clone(),
        _ => "unknown".to_string(),
    };
    let mtype = v
        .get("metadata")
        .and_then(|m| m.as_object())
        .and_then(|m| m.get("memory_type"))
        .and_then(|m| m.as_str())
        .or_else(|| str_field(&v, "class"))
        .or_else(|| {
            v.get("agent")
                .and_then(|a| a.as_object())
                .and_then(|a| a.get("memory_type"))
                .and_then(|m| m.as_str())
        })
        .unwrap_or("")
        .to_string();
    let ts = str_field(&v, "timestamp")
        .or_else(|| {
            v.get("metadata")
                .and_then(|m| m.as_object())
                .and_then(|m| m.get("created_at"))
                .and_then(|m| m.as_str())
        })
        .unwrap_or("")
        .to_string();
    let text = cap_text(raw_text, TEXT_CAP);
    Ok(Doc { id, text, agent, mtype, ts, text_chars })
}

/// Truncate to `cap` chars on a char boundary (never inside a UTF-8 sequence).
fn cap_text(s: &str, cap: usize) -> String {
    match s.char_indices().nth(cap) {
        Some((i, _)) => s[..i].to_string(),
        None => s.to_string(),
    }
}

/// Stream a gzip JSONL corpus into memory.
///
/// The gzip is read through `flate2`'s decoder in streaming mode, decompressed with `BufReader`, so a
/// 6 MB compressed / ~200 MB expanded corpus never exists as one buffer. Blank lines are skipped
/// (they are separators, not documents); unparsable rows are COUNTED, not silently dropped.
pub fn load_path(path: &Path) -> Result<Corpus, String> {
    let gz_bytes = std::fs::metadata(path).map_err(|e| format!("stat {}: {e}", path.display()))?.len();
    let f = std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut r = BufReader::with_capacity(1 << 20, flate2::read::GzDecoder::new(f));
    let mut corpus = Corpus {
        source: path.display().to_string(),
        gz_bytes,
        docs: Vec::new(),
        by_type: BTreeMap::new(),
        by_author: BTreeMap::new(),
        rows_read: 0,
        malformed_rows: 0,
        empty_text_rows: 0,
        truncated_rows: 0,
        duplicate_ids: 0,
        newest_ts: None,
        oldest_ts: None,
        loaded_at_unix: now_unix(),
    };
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut line = String::new();
    loop {
        line.clear();
        let n = r
            .read_line(&mut line)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed.trim().is_empty() {
            continue;
        }
        corpus.rows_read += 1;
        match parse_row(trimmed) {
            Ok(d) => {
                if d.text_chars > TEXT_CAP {
                    corpus.truncated_rows += 1;
                }
                *corpus
                    .by_type
                    .entry(type_of_key(&d.id, &d.mtype))
                    .or_insert(0) += 1;
                *corpus.by_author.entry(d.agent.clone()).or_insert(0) += 1;
                if !d.ts.is_empty() {
                    if corpus.newest_ts.as_deref().map_or(true, |t| d.ts.as_str() > t) {
                        corpus.newest_ts = Some(d.ts.clone());
                    }
                    if corpus.oldest_ts.as_deref().map_or(true, |t| d.ts.as_str() < t) {
                        corpus.oldest_ts = Some(d.ts.clone());
                    }
                }
                if !d.id.is_empty() && !seen.insert(fnv1a64(d.id.as_bytes())) {
                    corpus.duplicate_ids += 1;
                }
                corpus.docs.push(d);
            }
            Err(e) => {
                corpus.malformed_rows += 1;
                if corpus.malformed_rows <= 5 {
                    eprintln!("[memory_corpus] skipped row {}: {e}", corpus.rows_read);
                }
                if e.contains("no text") {
                    corpus.empty_text_rows += 1;
                }
            }
        }
    }
    if corpus.docs.is_empty() {
        return Err(format!(
            "{} expanded to {} rows but none parsed as a document ({} rejected)",
            path.display(),
            corpus.rows_read,
            corpus.malformed_rows
        ));
    }
    Ok(corpus)
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn fnv1a64(b: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for x in b {
        h ^= *x as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

// ── Vector artifact: the only source of `dim` that is not a guess ──────────────────────

/// What the embedding artifact says, measured from the bytes on disk.
pub struct VectorFacts {
    pub vecs_path: String,
    pub ids_path: Option<String>,
    pub ids_count: Option<u64>,
    /// Rows in the .npy array (from its header AND cross-checked against the payload length).
    pub rows: usize,
    /// Columns per row: payload_bytes / (rows * dtype_bytes). This is `dim`.
    pub dim: usize,
    pub dtype: String,
    pub payload_bytes: u64,
    /// Mean L2 norm over every row, computed by decoding the whole payload: the manifest claims
    /// "float16 (L2-normalised)", and this is the measurement that checks it.
    pub mean_norm: f64,
}

/// Decode IEEE-754 binary16 to f32 (no `half` crate; 12 lines, exact for all finite values).
fn f16_to_f32(h: u16) -> f32 {
    let sign = ((h >> 15) & 1) as u32;
    let exp = ((h >> 10) & 0x1f) as u32;
    let frac = (h & 0x3ff) as u32;
    let out = if exp == 0 {
        if frac == 0 {
            sign << 31
        } else {
            // subnormal: renormalise
            let mut e = -1i32;
            let mut f = frac;
            while f & 0x400 == 0 {
                f <<= 1;
                e -= 1;
            }
            f &= 0x3ff;
            let e32 = (127 - 15 + e + 1) as u32;
            (sign << 31) | (e32 << 23) | (f << 13)
        }
    } else if exp == 0x1f {
        (sign << 31) | (0xff << 23) | (frac << 13)
    } else {
        (sign << 31) | ((exp + 127 - 15) << 23) | (frac << 13)
    };
    f32::from_bits(out)
}

/// Parse the `.npy` header: `{'descr': '<f2', 'fortran_order': False, 'shape': (42805, 768), }`.
fn npy_header(b: &[u8]) -> Result<(String, Vec<usize>, usize), String> {
    if b.len() < 12 || &b[0..6] != b"\x93NUMPY" {
        return Err("not a .npy file (bad magic)".to_string());
    }
    let (major, hlen, off) = match b[6] {
        1 => (b[6], u16::from_le_bytes([b[8], b[9]]) as usize, 10usize),
        2 | 3 => (
            b[6],
            u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize,
            if b[6] == 2 { 12 } else { 12 },
        ),
        v => return Err(format!("unsupported .npy version {v}")),
    };
    let _ = major;
    let end = off
        .checked_add(hlen)
        .ok_or("npy header length overflow")?;
    if end > b.len() {
        return Err("npy header runs past end of file".to_string());
    }
    let hdr = String::from_utf8_lossy(&b[off..end]).to_string();
    let descr = hdr
        .split("'descr':")
        .nth(1)
        .and_then(|s| s.split('\'').nth(1))
        .ok_or_else(|| format!("npy header has no descr: {hdr}"))?
        .to_string();
    let shape_txt = hdr
        .split("'shape':")
        .nth(1)
        .and_then(|s| s.split('(').nth(1))
        .and_then(|s| s.split(')').next())
        .ok_or_else(|| format!("npy header has no shape: {hdr}"))?;
    let shape: Vec<usize> = shape_txt
        .split(',')
        .filter_map(|x| x.trim().parse::<usize>().ok())
        .collect();
    if shape.len() != 2 {
        return Err(format!("npy shape is not 2-D: {shape:?}"));
    }
    Ok((descr, shape, end))
}

/// Locate an artifact that sits beside the corpus: `fabric_vecs.npy` / `fabric_ids.json`.
///
/// When `TRM_MEMORY_CORPUS` pins a corpus, the artifacts are looked for in that corpus's own
/// directory first (they are written next to it), then the repo walk is used as before. Without the
/// pin, this is exactly `repo_relative`.
fn artifact_path(rel: &str) -> Option<PathBuf> {
    if let Ok(c) = std::env::var(CORPUS_ENV) {
        let p = PathBuf::from(c);
        if let (Some(dir), Some(name)) = (p.parent(), Path::new(rel).file_name()) {
            let cand = dir.join(name);
            if cand.exists() {
                return Some(cand);
            }
        }
    }
    crate::k3_retrieval::repo_relative(rel)
}

/// Read the vector artifact and measure it. `Err` carries the reason `dim` is not reported.
pub fn vector_facts() -> Result<VectorFacts, String> {
    let path = artifact_path(VECS_REL)
        .ok_or_else(|| format!("vector artifact not found: {VECS_REL}"))?;
    let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let (descr, shape, off) = npy_header(&bytes)?;
    let itemsize = match descr.as_str() {
        "<f2" | "|f2" | "=f2" => 2usize,
        "<f4" | "|f4" | "=f4" => 4usize,
        other => return Err(format!("unsupported npy dtype {other}")),
    };
    let payload = (bytes.len() - off) as u64;
    let rows = shape[0];
    let cols_from_payload = payload as usize / itemsize / rows.max(1);
    if cols_from_payload != shape[1] {
        return Err(format!(
            "npy payload is {payload} bytes = {cols_from_payload} columns/row, header says {}; refusing to report either as dim",
            shape[1]
        ));
    }
    // Measure the norms: decode every element and accumulate per-row sums of squares.
    let mut norm_sum = 0.0f64;
    let mut p = off;
    for _ in 0..rows {
        let mut ss = 0.0f64;
        for _ in 0..cols_from_payload {
            let v = if itemsize == 2 {
                f16_to_f32(u16::from_le_bytes([bytes[p], bytes[p + 1]])) as f64
            } else {
                f32::from_le_bytes([bytes[p], bytes[p + 1], bytes[p + 2], bytes[p + 3]]) as f64
            };
            p += itemsize;
            ss += v * v;
        }
        norm_sum += ss.sqrt();
    }
    let ids_path = artifact_path(IDS_REL);
    let ids_count = ids_path.as_ref().and_then(|p| {
        std::fs::read(p)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .and_then(|v| v.as_array().map(|a| a.len() as u64))
    });
    Ok(VectorFacts {
        vecs_path: path.display().to_string(),
        ids_path: ids_path.map(|p| p.display().to_string()),
        ids_count,
        rows,
        dim: cols_from_payload,
        dtype: descr,
        payload_bytes: payload,
        mean_norm: if rows > 0 { norm_sum / rows as f64 } else { 0.0 },
    })
}

// ── Cached access ─────────────────────────────────────────────────────────────────────

static CORPUS: crate::file_cache::Slot<Corpus> = std::sync::Mutex::new(None);
static VECTORS: crate::file_cache::Slot<VectorFacts> = std::sync::Mutex::new(None);

/// Deployment/test hook: `TRM_MEMORY_CORPUS=/abs/path/file.jsonl.gz` pins the corpus instead of
/// discovering it. Needed because discovery walks up from the running executable and this host has
/// more than one `swarm_v2_global_memory/` directory (a 1-row stub under the user profile, the real
/// 42,805-row fabric under the repo). With this set, the route reads exactly the file named.
pub const CORPUS_ENV: &str = "TRM_MEMORY_CORPUS";

/// The first corpus that resolves from this process's own location, loaded once and reloaded when
/// the gzip changes (the fabric builder rewrites it). `Err` names every path tried.
pub fn corpus() -> Result<std::sync::Arc<Corpus>, String> {
    let p = corpus_path()?;
    crate::file_cache::get(&CORPUS, &p, load_path)
}

/// Resolve the corpus path: `TRM_MEMORY_CORPUS` if set (and it must exist -- a pinned path that is
/// missing is an error, not a licence to fall back to a different corpus), else the first of
/// [`CORPUS_RELS`] found by walking up from the executable.
pub fn corpus_path() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var(CORPUS_ENV) {
        if p.trim().is_empty() {
            return Err(format!("{CORPUS_ENV} is set but empty"));
        }
        let p = PathBuf::from(p);
        if !p.is_file() {
            return Err(format!(
                "{CORPUS_ENV}={} is not a readable file",
                p.display()
            ));
        }
        return Ok(p);
    }
    let mut tried: Vec<String> = Vec::new();
    for rel in CORPUS_RELS {
        match crate::k3_retrieval::repo_relative(rel) {
            Some(p) => return Ok(p),
            None => tried.push(rel.to_string()),
        }
    }
    Err(format!("no memory corpus found on this host; tried: {}", tried.join(", ")))
}

/// The vector artifact's measured facts, cached the same way.
pub fn vectors() -> Result<std::sync::Arc<VectorFacts>, String> {
    let path = artifact_path(VECS_REL)
        .ok_or_else(|| format!("vector artifact not found: {VECS_REL}"))?;
    crate::file_cache::get(&VECTORS, &path, |_| vector_facts())
}

/// `/memory/query?q=...&k=N` -- the query string as (terms, k). `+` and `%XX` are decoded, because a
/// query arrives from a dashboard's `encodeURIComponent`, not from a shell.
pub fn query_from_url(url: &str) -> (String, usize) {
    let qs = url.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut q = String::new();
    let mut k = 5usize;
    for pair in qs.split('&') {
        let (key, val) = match pair.split_once('=') {
            Some(x) => x,
            None => continue,
        };
        match key {
            "q" | "query" => q = percent_decode(val),
            "k" | "limit" => k = val.parse::<usize>().unwrap_or(5),
            _ => {}
        }
    }
    (q, k)
}

/// `%XX` and `+` decoding for a query-string value. Invalid escapes are kept as written rather than
/// dropped, so the caller sees the term that was actually sent.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => {
                let hex = |c: u8| match c {
                    b'0'..=b'9' => Some(c - b'0'),
                    b'a'..=b'f' => Some(c - b'a' + 10),
                    b'A'..=b'F' => Some(c - b'A' + 10),
                    _ => None,
                };
                match (hex(b[i + 1]), hex(b[i + 2])) {
                    (Some(h), Some(l)) => {
                        out.push(h * 16 + l);
                        i += 3;
                    }
                    _ => {
                        out.push(b[i]);
                        i += 1;
                    }
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

/// `/memory/query?q=...&k=N` -- top-k keyword matches with scores and the matched text.
pub fn query_payload(q: &str, k: usize) -> Value {
    let k = k.clamp(1, 50);
    let c = match corpus() {
        Ok(c) => c,
        Err(e) => {
            return json!({
                "status": "unavailable",
                "query": q,
                "results": [],
                "count": 0,
                "error": e,
                "basis": "the memory corpus could not be loaded, so no query was scored. An unreadable corpus is not an empty one.",
            })
        }
    };
    let t0 = std::time::Instant::now();
    let (hits, terms) = c.search(q, k);
    let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let results: Vec<Value> = hits
        .iter()
        .map(|h| {
            json!({
                "id": h.doc.id,
                "score": h.score,
                "matched_terms": h.matched,
                "text": h.doc.text,
                "text_chars": h.doc.text_chars,
                "agent": h.doc.agent,
                "type": type_of_key(&h.doc.id, &h.doc.mtype),
                "memory_type": h.doc.mtype,
                "timestamp": h.doc.ts,
            })
        })
        .collect();
    json!({
        "status": "online",
        "query": q,
        "k": k,
        "terms": terms,
        "count": results.len(),
        "results": results,
        "documents_scanned": c.documents(),
        "scanned_ms": (elapsed_ms * 1000.0).round() / 1000.0,
        "corpus": c.source,
        "backend": "keyword_exact",
        "score_basis": "matched content terms / total content terms, the same ratio turbovec_memory.query_scored uses; terms are lowercased, stopwords and words of <=2 chars dropped. Ties break on newer timestamp then id.",
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// The synthetic fixture committed beside the crate: 6 rows in the shapes the live corpus uses
    /// (object `agent`, legacy string `agent`, a path-style key, a text-less row, a malformed line).
    /// Never the live 123k-row corpus -- a test that needs the live file is not a unit test.
    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/memory_corpus_sample.jsonl.gz")
    }

    fn loaded() -> Corpus {
        load_path(&fixture()).expect("fixture corpus must load")
    }

    #[test]
    fn parses_the_gzip_jsonl_fixture_into_documents() {
        let c = loaded();
        assert_eq!(c.documents(), 5, "5 of 7 rows are documents");
        assert_eq!(c.rows_read, 7);
        assert_eq!(c.malformed_rows, 2, "the broken line and the text-less row are both counted");
        assert_eq!(c.empty_text_rows, 1, "of those, one parsed as JSON but carried no text");
        let first = &c.docs[0];
        assert_eq!(first.id, "aaaaaaaaaaaaaaaa");
        assert_eq!(first.agent, "Archi", "object-shaped agent.author is read");
        assert_eq!(first.mtype, "knowledge");
        assert!(first.text.contains("ternary microkernel"));
    }

    #[test]
    fn legacy_string_agent_shape_is_read() {
        let c = loaded();
        let legacy = c.docs.iter().find(|d| d.id == "cccccccccccccccc").unwrap();
        assert_eq!(legacy.agent, "Logic", "agent as a bare string is read, not dropped");
        assert_eq!(legacy.mtype, "experience");
    }

    #[test]
    fn by_type_sums_to_the_document_count() {
        let c = loaded();
        let total: u64 = c.by_type.values().sum();
        assert_eq!(
            total as usize,
            c.documents(),
            "by_type must account for every document: {by_type:?} vs {} documents",
            c.documents(),
            by_type = c.by_type
        );
        // The path-style key files under its leading segment; hash keys fall back to memory_type.
        assert_eq!(c.by_type.get("papers").copied(), Some(1));
        assert_eq!(c.by_type.get("knowledge").copied(), Some(3));
        assert_eq!(c.by_type.get("experience").copied(), Some(1));
    }

    #[test]
    fn by_author_sums_to_the_document_count_too() {
        let c = loaded();
        let total: u64 = c.by_author.values().sum();
        assert_eq!(total as usize, c.documents());
    }

    #[test]
    fn query_returns_hits_for_a_present_term_and_nothing_for_a_nonsense_one() {
        let c = loaded();
        let (hits, terms) = c.search("ternary microkernel", 3);
        assert_eq!(terms, vec!["ternary".to_string(), "microkernel".to_string()]);
        assert!(!hits.is_empty(), "a term present in the fixture must match");
        assert_eq!(hits[0].doc.id, "aaaaaaaaaaaaaaaa");
        assert_eq!(hits[0].score, 1.0, "both terms matched in one document");
        assert_eq!(hits[0].matched, vec!["ternary".to_string(), "microkernel".to_string()]);

        let (none, _) = c.search("zyzzogeton-nonexistent-token", 5);
        assert!(none.is_empty(), "a nonsense term must not match anything");
    }

    #[test]
    fn a_query_of_only_stopwords_returns_nothing() {
        let c = loaded();
        let (hits, terms) = c.search("the and of it", 5);
        assert!(terms.is_empty());
        assert!(hits.is_empty());
    }

    #[test]
    fn scores_are_bounded_and_ordered() {
        let c = loaded();
        let (hits, _) = c.search("consensus refute", 5);
        assert!(hits.len() >= 2, "two rows mention these terms");
        let mut prev = f64::INFINITY;
        for h in &hits {
            assert!(h.score > 0.0 && h.score <= 1.0, "score {} out of range", h.score);
            assert!(h.score <= prev, "hits must be sorted by descending score");
            prev = h.score;
        }
    }

    #[test]
    fn the_newest_and_oldest_timestamps_are_measured_not_assumed() {
        let c = loaded();
        assert_eq!(c.oldest_ts.as_deref(), Some("2025-01-02T03:04:05"));
        assert_eq!(c.newest_ts.as_deref(), Some("2026-02-03T04:05:06"));
    }

    #[test]
    fn a_missing_file_is_an_error_not_an_empty_corpus() {
        let err = load_path(Path::new("does/not/exist.jsonl.gz")).unwrap_err();
        assert!(err.starts_with("stat"), "got: {err}");
    }

    #[test]
    fn query_string_is_decoded_from_the_url() {
        let (q, k) = query_from_url("/memory/query?q=ternary%20microkernel&k=7");
        assert_eq!(q, "ternary microkernel");
        assert_eq!(k, 7);
        let (q2, k2) = query_from_url("/memory/query?q=shard+journal%2Bledger");
        assert_eq!(q2, "shard journal+ledger");
        assert_eq!(k2, 5, "k defaults to 5 when the caller omits it");
        let (q3, _) = query_from_url("/memory/query");
        assert_eq!(q3, "");
    }

    #[test]
    fn type_of_key_prefers_the_leading_segment() {
        assert_eq!(type_of_key("papers/2024/x.md", "knowledge"), "papers");
        assert_eq!(type_of_key("36ed6a1b6200533b", "experience"), "experience");
        assert_eq!(type_of_key("36ed6a1b6200533b", ""), "root");
    }

    // ── Live-corpus measurement (opt-in) ──────────────────────────────────────────────
    //
    // `#[ignore]` so `cargo test` stays off the 6 MB live gzip: the unit tests above run on the
    // fixture. Run it explicitly to print the measured census of whatever corpus this host holds:
    //     cargo test --release live_corpus_measurement -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_corpus_measurement() {
        let c = match corpus() {
            Ok(c) => c,
            Err(e) => panic!("no live corpus: {e}"),
        };
        println!("LIVE source          = {}", c.source);
        println!("LIVE documents       = {}", c.documents());
        println!("LIVE rows_read       = {}", c.rows_read);
        println!("LIVE malformed_rows  = {}", c.malformed_rows);
        println!("LIVE empty_text_rows = {}", c.empty_text_rows);
        println!("LIVE truncated_rows  = {}", c.truncated_rows);
        println!("LIVE by_type         = {:?}", c.by_type);
        let author_total: u64 = c.by_author.values().sum();
        println!("LIVE by_author total = {author_total} (authors: {})", c.by_author.len());
        println!("LIVE oldest_ts       = {:?}", c.oldest_ts);
        println!("LIVE newest_ts       = {:?}", c.newest_ts);
        assert_eq!(
            c.by_type.values().sum::<u64>() as usize,
            c.documents(),
            "by_type must sum to the document count on the LIVE corpus too"
        );
        let (hits, terms) = c.search("ternary quantization", 3);
        println!("LIVE query terms     = {terms:?}");
        println!("LIVE hits            = {}", hits.len());
        for h in &hits {
            println!(
                "LIVE hit score={:.3} id={} matched={:?} chars={} text={:.120}",
                h.score,
                h.doc.id,
                h.matched,
                h.doc.text_chars,
                h.doc.text
            );
        }
        let (none, _) = c.search("zzqqxx-nonexistent-nonsense-token", 3);
        println!("LIVE nonsense hits   = {}", none.len());
        assert!(none.is_empty(), "a nonsense term must match nothing");
        // The /memory/query payload itself, as the route serializes it.
        let p = query_payload("kleene ternary", 2);
        println!("LIVE query_payload   = {}", serde_json::to_string(&p).unwrap());
        assert_eq!(p["count"], 2, "the route payload must carry the hits");
        assert_eq!(p["backend"], "keyword_exact");
        assert!(p["results"][0]["text"].as_str().map_or(false, |t| !t.is_empty()));
        let pn = query_payload("zzqqxx-nonexistent-nonsense-token", 3);
        assert_eq!(pn["count"], 0, "a nonsense term must return an empty result set");
        if let Ok(v) = vectors() {
            println!(
                "LIVE vectors         = rows={} dim={} dtype={} payload={}B mean_norm={:.4} ids={:?}",
                v.rows, v.dim, v.dtype, v.payload_bytes, v.mean_norm, v.ids_count
            );
        }
    }
}
