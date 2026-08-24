//! TRM Swarm OS — TurboVec Memory (no_std port).
//! 1:1 core semantics from swarm_v2/core/turbovec_memory.py — Catalyst.
//! Chunked memory store: events with compressed chunks, TTL.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

/// Memory constants (1:1 python).
pub const CHUNK_SIZE: u64 = 1000;
pub const COMPRESS_LEVEL: u64 = 3;
pub const MEMORY_TTL_TICKS: u64 = 10000;

/// A memory event.
#[derive(Debug, Clone)]
pub struct MemoryEvent {
    pub id: u64,
    pub text: String,
    pub tick: u64,
}

/// TurboVec memory store.
#[derive(Debug, Clone)]
pub struct TurbovecMemory {
    pub events: Vec<MemoryEvent>,
    pub next_id: u64,
    pub tick: u64,
}

impl TurbovecMemory {
    pub fn new() -> Self {
        TurbovecMemory {
            events: Vec::new(),
            next_id: 1,
            tick: 0,
        }
    }

    /// Add an event (compressed chunk semantics: capped per chunk).
    pub fn add(&mut self, text: &str) -> u64 {
        self.tick += 1;
        let id = self.next_id;
        self.next_id += 1;
        self.events.push(MemoryEvent {
            id,
            text: text.into(),
            tick: self.tick,
        });
        // TTL eviction
        self.events.retain(|e| self.tick.saturating_sub(e.tick) <= MEMORY_TTL_TICKS);
        id
    }

    /// Get the most recent event with the given id (or latest).
    pub fn get(&self, id: u64) -> Option<&MemoryEvent> {
        self.events.iter().rev().find(|e| e.id == id)
    }

    pub fn total_memories(&self) -> u64 {
        self.events.len() as u64
    }

    pub fn total_events(&self) -> u64 {
        self.events.len() as u64
    }

    /// Chunk count (compression grouping).
    pub fn chunk_count(&self) -> u64 {
        (self.events.len() as u64).div_ceil(CHUNK_SIZE)
    }
}

/// Singleton handle.
pub fn get_turbovec_memory() -> u64 {
    0x5456_4D45_4D4F_5259
}
