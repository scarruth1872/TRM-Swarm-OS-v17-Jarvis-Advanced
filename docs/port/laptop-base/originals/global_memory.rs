//! TRM Swarm OS — Global Memory Sync (no_std port).
//! Production-grade distributed vector database for shared long-term memory.
//! 1:1 deterministic core from swarm_v2/core/global_memory.py:
//!   - TurboVec 16x vector memory integration
//!   - Shared semantic embeddings and metadata tagging
//!   - Sub-millisecond associative recall across 14 council agents

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const MAX_GLOBAL_MEMORIES: usize = 2048;
pub const EMBEDDING_DIM: usize = 128;

#[derive(Debug, Clone)]
pub struct GlobalMemoryEntry {
    pub id: u64,
    pub agent: &'static str,
    pub category: &'static str,
    pub content: String,
    pub embedding: [f32; EMBEDDING_DIM],
    pub timestamp_ns: u64,
}

pub struct GlobalMemorySync {
    pub entries: Vec<GlobalMemoryEntry>,
    pub next_id: u64,
}

impl GlobalMemorySync {
    pub fn new() -> Self {
        GlobalMemorySync {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    pub fn store(
        &mut self,
        agent: &'static str,
        category: &'static str,
        content: &str,
        embedding: Option<[f32; EMBEDDING_DIM]>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let emb = embedding.unwrap_or([0.0f32; EMBEDDING_DIM]);

        if self.entries.len() >= MAX_GLOBAL_MEMORIES {
            self.entries.remove(0); // Evict oldest
        }

        self.entries.push(GlobalMemoryEntry {
            id,
            agent,
            category,
            content: content.to_string(),
            embedding: emb,
            timestamp_ns: crate::time::now_ns(),
        });

        id
    }

    pub fn recall_by_agent(&self, agent: &str) -> Vec<&GlobalMemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.agent.eq_ignore_ascii_case(agent))
            .collect()
    }

    pub fn recall_by_category(&self, category: &str) -> Vec<&GlobalMemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.category.eq_ignore_ascii_case(category))
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }
}

static SINGLETON_GLOBAL_MEMORY: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub fn get_global_memory() -> u64 {
    let v = SINGLETON_GLOBAL_MEMORY.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_GLOBAL_MEMORY.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ── The oracle's model (swarm_v2/core/global_memory.py) ──────────────────────────────────────────
//
// `_is_poisoned` is a plain substring scan over the eleven POISON_PATTERNS (extracted from the source).
// `get_context_for_agent` assembles prose from two already-ranked retrievals:
//     anchors with score > 0.4      -> "Core System Philosophy: {content}"
//     memories with score > 0.35    -> "Past knowledge from {author}: {content[:300]}"
//     memories of type "source_anchor" are SKIPPED in that second pass (the anchors already covered
//     them), and poisoned content is skipped
//     nothing survives -> "" ; otherwise "Global Intelligence Feed:\n" + the lines
// The retrieval itself (ChromaDB, embeddings, `top_k`) stays host-side: the lists arrive as arguments.

/// `POISON_PATTERNS`, extracted from the oracle's tuple.
pub const POISON_PATTERNS: [&str; 11] = ["[DIRECT NEURAL BRIDGE]", "[HMI BRIDGE ACTIVE]", "[EXECUTION MODE ACTIVE]", "[MESH OBSERVABILITY]", "[STRICT CONSTRAINT]", "I am currently processing this request internally.", "My linguistic output is currently stalled", "## CRITICAL: Action Execution Rules", "WRITE_FILE: architecture_overview.md", "SEARCH_QUERY:", "[Archi:Architect] Task:"];

/// `_is_poisoned`: any pattern occurring as a substring.
pub fn is_poisoned(text: &str) -> bool {
    POISON_PATTERNS.iter().any(|p| text.contains(*p))
}

/// Anchor threshold: a source anchor is quoted above this score.
pub const ANCHOR_MIN_SCORE: f64 = 0.4;
/// Memory threshold: a past memory is quoted above this score.
pub const MEMORY_MIN_SCORE: f64 = 0.35;
/// The oracle truncates a quoted memory to this many characters.
pub const MEMORY_TRUNCATE: usize = 300;

/// One retrieved memory: (score, author, memory_type, content).
pub type MemoryHit<'a> = (f64, &'a str, &'a str, &'a str);

/// `get_context_for_agent` over already-retrieved hits: (anchors, memories) -> assembled string.
pub fn get_context_for_agent(anchors: &[(f64, &str)], memories: &[MemoryHit]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for (score, content) in anchors.iter() {
        if *score > ANCHOR_MIN_SCORE {
            let mut s = String::from("Core System Philosophy: ");
            s.push_str(content);
            lines.push(s);
        }
    }
    for (score, author, memory_type, content) in memories.iter() {
        if *memory_type == "source_anchor" {
            continue;
        }
        if is_poisoned(content) {
            continue;
        }
        if *score > MEMORY_MIN_SCORE {
            let cut: String = content.chars().take(MEMORY_TRUNCATE).collect();
            let mut s = String::from("Past knowledge from ");
            s.push_str(author);
            s.push_str(": ");
            s.push_str(&cut);
            lines.push(s);
        }
    }
    if lines.is_empty() {
        return String::new();
    }
    let mut out = String::from("Global Intelligence Feed:\n");
    out.push_str(&lines.join("\n"));
    out
}

// ── Ports of the remaining pure functions in the oracle ─────────────────────────────────────────
//
// `_native_memory_token_overlap` (Python fallback)  -> `memory_token_overlap`
// `_sync_to_shared_jsonld` relationship regex       -> `parse_relationship`
// `query` keyword-overlap fallback (CHROMA off)     -> `keyword_query`
// `get_stats` Counter aggregation                   -> `compute_stats`
// `get_memory_health` arithmetic                    -> `memory_health`
//
// Host-side (NOT ported — the kernel is deterministic and has no filesystem):
//   ChromaDB / TurboVec retrieval, DGE masking/audit (`mask_pii`, `log_access`,
//   `validate_memory_contribution`), metadata file I/O (`_load_metadata` /
//   `_save_metadata`), `datetime.now()`, Python's salted `hash()` (entry ids and
//   the dedup signatures in `optimize_collective_memory` are NOT deterministic
//   across processes), and the `strip_global_memory.dll` fast path.

/// `_native_memory_token_overlap` Python fallback: per-entry overlap score.
///
/// `q_set = set(query_tokens)`, `q_len = max(len(q_set), 1)`, and each entry's
/// score is `(count of entry tokens present in q_set) / q_len` — computed in f64
/// then narrowed to f32, because the oracle returns `numpy.float32`.
pub fn memory_token_overlap(query_tokens: &[i32], entry_tokens_matrix: &[Vec<i32>]) -> Vec<f32> {
    let mut q_set: Vec<i32> = Vec::new();
    for &t in query_tokens {
        if !q_set.contains(&t) {
            q_set.push(t);
        }
    }
    let q_len = core::cmp::max(q_set.len(), 1) as f64;
    entry_tokens_matrix
        .iter()
        .map(|entry| {
            let count = entry.iter().filter(|t| q_set.contains(t)).count();
            ((count as f64) / q_len) as f32
        })
        .collect()
}

/// One `source --[label]--> target` (or `source -> label -> target`) triple,
/// the parsed form the JSON-LD graph builder consumes.
#[derive(Debug, Clone, PartialEq)]
pub struct Relationship {
    pub source: String,
    pub label: String,
    pub target: String,
}

/// The relationship regex of `_sync_to_shared_jsonld`, extracted from the JSON-LD
/// file I/O it is entangled with. Tries the `--[label]-->` form first, then the
/// `-> label ->` form, exactly as the oracle does. Returns `None` for plain text.
pub fn parse_relationship(content: &str) -> Option<Relationship> {
    let s = content.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(rel) = relationship_double_arrow(s) {
        return Some(rel);
    }
    relationship_plain_arrow(s)
}

/// Pattern 1: `^(.+?)\s+--\[(.+?)\]-->\s+(.+)$` (source --[label]--> target).
fn relationship_double_arrow(s: &str) -> Option<Relationship> {
    let m = s.find("--[")?;
    let before = &s[..m];
    let source = before.trim_end();
    // `(.+?)` needs >=1 char and `\s+` needs >=1 whitespace char before `--[`.
    if source.is_empty() || source.len() == before.len() {
        return None;
    }
    let rest = &s[m + 3..];
    let k = rest.find("]-->")?;
    if k == 0 {
        return None; // label needs >=1 char
    }
    let label = &rest[..k];
    let after = &rest[k + 4..];
    let target = after.trim_start();
    // `\s+(.+)` needs >=1 whitespace char then >=1 target char.
    if target.is_empty() || target.len() == after.len() {
        return None;
    }
    Some(Relationship {
        source: source.trim().to_string(),
        label: label.trim().to_string(),
        target: target.trim().to_string(),
    })
}

/// Pattern 2: `^(.+?)\s+->\s+(.+?)\s+->\s+(.+)$` (source -> label -> target).
fn relationship_plain_arrow(s: &str) -> Option<Relationship> {
    let (ws1_start, ws1_end) = find_surrounded_arrow(s, 0)?;
    if ws1_start == 0 {
        return None; // source needs >=1 char
    }
    let (ws2_start, ws2_end) = find_surrounded_arrow(s, ws1_end)?;
    let label = &s[ws1_end..ws2_start];
    if label.is_empty() {
        return None; // label needs >=1 char
    }
    let target = &s[ws2_end..];
    if target.is_empty() {
        return None; // target needs >=1 char
    }
    Some(Relationship {
        source: s[..ws1_start].trim().to_string(),
        label: label.trim().to_string(),
        target: target.trim().to_string(),
    })
}

/// Find the first `\s+->\s+` at or after `from`. Returns `(ws_before_start, ws_after_end)`:
/// the whitespace run before `->` starts at `ws_before_start`, and the whitespace run
/// after `->` ends at `ws_after_end`.
fn find_surrounded_arrow(s: &str, from: usize) -> Option<(usize, usize)> {
    let mut search_from = from;
    loop {
        let idx = search_from + s[search_from..].find("->")?;
        let before = &s[..idx];
        let ws_before_start = before.trim_end().len();
        if ws_before_start == idx {
            search_from = idx + 2; // no whitespace before this arrow
            continue;
        }
        let after = &s[idx + 2..];
        let trimmed = after.trim_start();
        if trimmed.len() == after.len() {
            search_from = idx + 2; // no whitespace after this arrow
            continue;
        }
        let ws_after_end = idx + 2 + (after.len() - trimmed.len());
        return Some((ws_before_start, ws_after_end));
    }
}

/// One memory entry for the keyword fallback / stats aggregation.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryEntry {
    pub id: String,
    pub author: String,
    pub memory_type: String,
    pub content: String,
    pub access_count: u64,
}

/// `query` keyword-overlap fallback (CHROMA_AVAILABLE=False): score every entry by
/// the fraction of unique query words that occur as substrings of its lowercased
/// content, keep an entry when it has any overlap (or the query is blank), sort by
/// score descending (stable, so ties keep insertion order) and truncate to `top_k`.
///
/// Returns `(score, index)` pairs — `index` is the position in `entries`, which the
/// host maps back to the entry id. The access-count mutation, DGE mask/audit and the
/// metadata save that the oracle performs after the sort are host-side and excluded.
pub fn keyword_query(
    query_text: &str,
    entries: &[MemoryEntry],
    author_filter: Option<&str>,
    type_filter: Option<&str>,
    top_k: usize,
) -> Vec<(f64, usize)> {
    // query_words = set(query_text.lower().split())
    let lower = query_text.to_lowercase();
    let mut query_words: Vec<&str> = Vec::new();
    for w in lower.split_whitespace() {
        if !query_words.contains(&w) {
            query_words.push(w);
        }
    }
    let q_len = core::cmp::max(query_words.len(), 1);
    let blank = query_text.trim().is_empty();

    let mut results: Vec<(f64, usize)> = Vec::new();
    for (idx, entry) in entries.iter().enumerate() {
        if let Some(af) = author_filter {
            if !af.is_empty() && entry.author.as_str() != af {
                continue;
            }
        }
        if let Some(tf) = type_filter {
            if !tf.is_empty() && entry.memory_type.as_str() != tf {
                continue;
            }
        }
        let content_lower = entry.content.to_lowercase();
        let mut overlap = 0usize;
        for w in &query_words {
            if content_lower.contains(*w) {
                overlap += 1;
            }
        }
        let score = (overlap as f64) / (q_len as f64);
        if overlap > 0 || blank {
            results.push((score, idx));
        }
    }
    results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));
    results.truncate(top_k);
    results
}

/// `get_stats` Counter aggregation (legacy branch): type/author tallies in
/// first-seen order plus the summed access count.
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    pub total_memories: usize,
    pub type_counts: Vec<(String, usize)>,
    pub author_counts: Vec<(String, usize)>,
    pub total_accesses: u64,
}

/// Port of the `Counter` aggregation in `get_stats`.
pub fn compute_stats(entries: &[MemoryEntry]) -> Stats {
    let mut type_counts: Vec<(String, usize)> = Vec::new();
    let mut author_counts: Vec<(String, usize)> = Vec::new();
    let mut total_accesses: u64 = 0;
    for e in entries {
        total_accesses += e.access_count;
        match type_counts.iter_mut().find(|(k, _)| *k == e.memory_type) {
            Some((_, c)) => *c += 1,
            None => type_counts.push((e.memory_type.clone(), 1)),
        }
        match author_counts.iter_mut().find(|(k, _)| *k == e.author) {
            Some((_, c)) => *c += 1,
            None => author_counts.push((e.author.clone(), 1)),
        }
    }
    Stats {
        total_memories: entries.len(),
        type_counts,
        author_counts,
        total_accesses,
    }
}

/// `get_memory_health` arithmetic over an already-aggregated stats dict.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryHealth {
    pub total_memories: usize,
    pub total_accesses: u64,
    pub avg_access_rate: f64,
    pub type_diversity: usize,
    pub author_diversity: usize,
    pub backend: String,
    pub health_score: f64,
}

/// Port of `get_memory_health` (input is the stats the oracle's `get_stats` returns).
pub fn memory_health(
    total_memories: usize,
    total_accesses: u64,
    type_count: usize,
    author_count: usize,
    backend: &str,
) -> MemoryHealth {
    let avg_access = if total_memories > 0 {
        (total_accesses as f64) / (total_memories as f64)
    } else {
        0.0
    };
    let avg_access_rate = round_half_even(avg_access, 2);
    let raw = avg_access * 0.3 + (type_count as f64) * 0.1 + (author_count as f64) * 0.05;
    let capped = if raw < 1.0 { raw } else { 1.0 };
    let health_score = round_half_even(capped, 3);
    MemoryHealth {
        total_memories,
        total_accesses,
        avg_access_rate,
        type_diversity: type_count,
        author_diversity: author_count,
        backend: backend.to_string(),
        health_score,
    }
}

/// Python's `round(x, ndigits)`: round-half-to-even (Rust's `f64::round` is
/// half-away-from-zero, so it cannot be used to match the oracle). `powi`/`floor`
/// are unavailable on this no_std target, so the floor is taken via a truncating
/// integer cast (valid for `x >= 0`, and negatives are folded by symmetry).
fn round_half_even(x: f64, ndigits: i32) -> f64 {
    if !x.is_finite() {
        return x;
    }
    if x < 0.0 {
        return -round_half_even(-x, ndigits);
    }
    let mut factor = 1.0f64;
    for _ in 0..ndigits {
        factor *= 10.0;
    }
    let scaled = x * factor;
    let floored = (scaled as i64) as f64; // x >= 0, so truncation == floor
    let frac = scaled - floored;
    let rounded = if frac > 0.5 {
        floored + 1.0
    } else if frac < 0.5 {
        floored
    } else if (floored as i64) % 2 == 0 {
        floored
    } else {
        floored + 1.0
    };
    rounded / factor
}
