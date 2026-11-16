//! TRM Swarm OS — Semantic Cache (no_std port).
//! 1:1 core semantics from swarm_v2/core/semantic_cache.py — Catalyst.
//! TTL cache with expiry check + LRU-ish eviction at capacity.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Cache constants (1:1 python).
pub const DEFAULT_TTL_SECONDS: u64 = 300;
pub const DEFAULT_MAX_ENTRIES: u64 = 256;

/// A cache entry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub ttl_seconds: u64,
    pub stored_at: u64, // monotonic tick
}

/// Semantic cache.
#[derive(Debug, Clone)]
pub struct SemanticCache {
    pub entries: Vec<CacheEntry>,
    pub max_entries: u64,
    pub tick: u64,
    pub hits: u64,
    pub misses: u64,
}

impl SemanticCache {
    pub fn new(max_entries: u64) -> Self {
        SemanticCache {
            entries: Vec::new(),
            max_entries: if max_entries == 0 { DEFAULT_MAX_ENTRIES } else { max_entries },
            tick: 0,
            hits: 0,
            misses: 0,
        }
    }

    fn is_expired(&self, e: &CacheEntry) -> bool {
        self.tick.saturating_sub(e.stored_at) > e.ttl_seconds
    }

    /// Get a value; returns Option (expired entries are evicted).
    pub fn get(&mut self, key: &str) -> Option<String> {
        self.tick += 1;
        let mut found = None;
        let tick = self.tick;
        let key_owned = key.to_string();
        self.entries.retain(|e| {
            if e.key == key_owned {
                let expired = tick.saturating_sub(e.stored_at) > e.ttl_seconds;
                if expired {
                    false
                } else {
                    found = Some(e.value.clone());
                    true
                }
            } else {
                true
            }
        });
        if found.is_some() {
            self.hits += 1;
        } else {
            self.misses += 1;
        }
        found
    }

    /// Store a value; evicts oldest when over capacity.
    pub fn set(&mut self, key: &str, value: &str, ttl_seconds: u64) {
        self.tick += 1;
        self.entries.retain(|e| e.key != key);
        self.entries.push(CacheEntry {
            key: key.into(),
            value: value.into(),
            ttl_seconds: if ttl_seconds == 0 { DEFAULT_TTL_SECONDS } else { ttl_seconds },
            stored_at: self.tick,
        });
        while self.entries.len() as u64 > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Stats: (entries, hits, misses).
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (self.entries.len() as u64, self.hits, self.misses)
    }
}

/// Singleton handle.
pub fn get_cache() -> u64 {
    0x5345_4D43_4143_4845
}
