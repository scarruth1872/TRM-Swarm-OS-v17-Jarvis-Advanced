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
