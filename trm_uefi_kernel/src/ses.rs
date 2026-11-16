//! Semantic Enrichment Service (SES) - entity + triple extraction.
//!
//! Port of `swarm_v2/core/ses.py`. Populates the Dynamic Knowledge Graph by
//! pulling (subject, predicate, object) triples out of raw text.
//!
//! SWEEP CONTEXT: only a singleton stub accessor had been ported; the whole
//! SESEngine (entity_patterns, extract_triples, _identify_entities) was absent.
//!
//! REGEX HANDLING: the Python uses four regexes with `\b` word boundaries:
//!   SERVICE  \b(?:api|db|gateway|auth|worker|swarm)\b
//!   ERROR    \b(?:404|500|ConnectionRefused|Timeout|Crash)\b
//!   USER     \buser_[a-f0-9]{8}\b
//!   ACTION   \b(?:GET|POST|PUT|DELETE|FETCH|STORE)\b
//! The kernel has no regex engine, so these are matched directly. NOTE: a naive
//! substring port of this file would repeat the self_healing defect, so the
//! word boundaries are enforced explicitly here (see `find_keyword`).
//!
//! no_std ADAPTATIONS: `datetime.now().isoformat()` -> `at_ns`; positions are
//! CHAR indices (Python str indices are char-based, Rust byte offsets are not).

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const SERVICE_WORDS: &[&str] = &["api", "db", "gateway", "auth", "worker", "swarm"];
pub const ERROR_WORDS: &[&str] = &["404", "500", "ConnectionRefused", "Timeout", "Crash"];
pub const ACTION_WORDS: &[&str] = &["GET", "POST", "PUT", "DELETE", "FETCH", "STORE"];

#[derive(Clone, Debug)]
pub struct Entity {
    pub name: String,
    pub entity_type: String, // "SERVICE" | "ERROR" | "USER" | "ACTION"
    pub start: usize,        // char index
    pub end: usize,          // char index, exclusive
}

#[derive(Clone, Debug)]
pub struct Triple {
    pub subject: String,
    pub subject_type: String,
    pub predicate: String,
    pub object: String,
    pub object_type: String,
    pub at_ns: u64,
}

/// Regex `\w` equivalent for the boundary test.
fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// All case-insensitive occurrences of `kw` sitting on word boundaries -
/// mirrors `\b(?:kw)\b`. Returns CHAR indices.
fn find_keyword(text: &str, kw: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    if kw.is_empty() { return out; }
    let chars: Vec<char> = text.chars().collect();
    let k: Vec<char> = kw.chars().collect();
    if chars.len() < k.len() { return out; }
    for i in 0..=(chars.len() - k.len()) {
        let seg = &chars[i..i + k.len()];
        let hit = seg.iter().zip(k.iter())
            .all(|(a, b)| a.eq_ignore_ascii_case(b));
        if !hit { continue; }
        let before_ok = i == 0 || !is_word_char(chars[i - 1]);
        let j = i + k.len();
        let after_ok = j >= chars.len() || !is_word_char(chars[j]);
        if before_ok && after_ok { out.push((i, j)); }
    }
    out
}

/// `\buser_[a-f0-9]{8}\b` - the prefix, exactly 8 hex digits, then a boundary.
fn find_user_ids(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let pre: Vec<char> = "user_".chars().collect();
    if chars.len() < pre.len() + 8 { return out; }
    for i in 0..=(chars.len() - pre.len() - 8) {
        if !chars[i..i + pre.len()].iter().zip(pre.iter())
            .all(|(a, b)| a.eq_ignore_ascii_case(b)) { continue; }
        let hex_start = i + pre.len();
        let hex_ok = chars[hex_start..hex_start + 8].iter()
            .all(|c| c.is_ascii_hexdigit()); // IGNORECASE: A-F count too
        if !hex_ok { continue; }
        let j = hex_start + 8;
        let before_ok = i == 0 || !is_word_char(chars[i - 1]);
        let after_ok = j >= chars.len() || !is_word_char(chars[j]);
        if before_ok && after_ok { out.push((i, j)); }
    }
    out
}

pub struct SesEngine;

impl SesEngine {
    pub fn new() -> Self { SesEngine }

    /// All entities, sorted by appearance. Python iterates its pattern dict in
    /// insertion order (SERVICE, ERROR, USER, ACTION) and sorts stably by start,
    /// so ties resolve in that same order - reproduced here.
    pub fn identify_entities(text: &str) -> Vec<Entity> {
        let mut entities: Vec<Entity> = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut push = |spans: Vec<(usize, usize)>, label: &str, out: &mut Vec<Entity>| {
            for (s, e) in spans {
                out.push(Entity {
                    name: chars[s..e].iter().collect(),
                    entity_type: label.to_string(),
                    start: s,
                    end: e,
                });
            }
        };
        for kw in SERVICE_WORDS { push(find_keyword(text, kw), "SERVICE", &mut entities); }
        for kw in ERROR_WORDS   { push(find_keyword(text, kw), "ERROR",   &mut entities); }
        push(find_user_ids(text), "USER", &mut entities);
        for kw in ACTION_WORDS  { push(find_keyword(text, kw), "ACTION",  &mut entities); }
        entities.sort_by_key(|e| e.start);   // stable, like Python's sorted()
        entities
    }

    /// (subject, predicate, object) triples over CONSECUTIVE entity pairs.
    /// Predicate priority mirrors the Python if/elif chain:
    /// triggered > failed > depends on > RELATES_TO.
    pub fn extract_triples(text: &str, now_ns: u64) -> Vec<Triple> {
        let entities = Self::identify_entities(text);
        let mut triples = Vec::new();
        if entities.len() < 2 { return triples; }
        let lower = text.to_lowercase();
        let predicate = if lower.contains("triggered") { "TRIGGERED" }
            else if lower.contains("failed") { "FAILED_ON" }
            else if lower.contains("depends on") { "DEPENDS_ON" }
            else { "RELATES_TO" };
        for w in entities.windows(2) {
            triples.push(Triple {
                subject: w[0].name.clone(),
                subject_type: w[0].entity_type.clone(),
                predicate: predicate.to_string(),
                object: w[1].name.clone(),
                object_type: w[1].entity_type.clone(),
                at_ns: now_ns,
            });
        }
        triples
    }
}
