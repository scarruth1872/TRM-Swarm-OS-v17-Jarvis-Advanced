//! TRM Swarm OS — Semantic Index (no_std port).
//! 1:1 core semantics from swarm_v2/core/semantic_index.py — Catalyst.
//! Deterministic semantic query: keyword-overlap scoring against indexed
//! entries. no_std, on-device.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// An indexed semantic entry.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub key: String,
    pub tokens: Vec<String>,
}

/// Semantic index: query by token overlap.
#[derive(Debug, Clone)]
pub struct SemanticIndex {
    pub entries: Vec<IndexEntry>,
    pub query_count: u64,
}

impl SemanticIndex {
    pub fn new() -> Self {
        SemanticIndex {
            entries: Vec::new(),
            query_count: 0,
        }
    }

    /// Index an entry (tokens split on whitespace).
    pub fn add(&mut self, key: &str, text: &str) {
        let tokens = text.split_whitespace().map(|t| t.to_string()).collect();
        self.entries.push(IndexEntry {
            key: key.into(),
            tokens,
        });
    }

    /// Query: returns (best_key, score) — highest token-overlap ratio.
    pub fn query(&mut self, q: &str) -> (String, f64) {
        self.query_count += 1;
        let qt: Vec<String> = q.split_whitespace().map(|t| t.to_string()).collect();
        let mut best = String::from("");
        let mut best_score = 0.0f64;
        for e in &self.entries {
            let mut hits = 0u64;
            for t in &qt {
                if e.tokens.contains(t) {
                    hits += 1;
                }
            }
            let score = if qt.is_empty() {
                0.0
            } else {
                hits as f64 / qt.len() as f64
            };
            if score > best_score {
                best_score = score;
                best = e.key.clone();
            }
        }
        (best, best_score)
    }
}

/// Singleton handle.
pub fn get_semantic_index() -> u64 {
    0x5345_4D41_4E54_4943
}
