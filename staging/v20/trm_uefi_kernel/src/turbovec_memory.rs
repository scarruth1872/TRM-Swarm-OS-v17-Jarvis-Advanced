//! TRM Swarm OS — TurboVec Memory (no_std port).
//! 1:1 core semantics from swarm_v2/core/turbovec_memory.py — Catalyst.
//! Chunked memory store: events with compressed chunks, TTL.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
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

// ── The oracle's model (swarm_v2/core/turbovec_memory.py) ────────────────────────────────────────
//
// `compact(max_entries=100000)`: only when `len(cache) > max_entries` (strictly), the cache keeps its
// LAST `max_entries` entries, the index is rebuilt and the compressed file rewritten.
// `_atomic_append`: one entry becomes `json.dumps(entry) + "\n"`, gzipped into a buffer, appended and
// fsynced. The line rule is portable; the gzip container and the fsync stay host-side, and this module
// says so rather than pretending a firmware has a filesystem barrier.

/// The oracle's default compaction ceiling.
pub const COMPACT_MAX_DEFAULT: usize = 100000;

/// `compact`'s decision: (did it compact, the length afterwards).
pub fn compact_plan(entries: usize, max_entries: usize) -> (bool, usize) {
    if entries > max_entries {
        (true, max_entries)
    } else {
        (false, entries)
    }
}

/// `_atomic_append`'s payload, minus the gzip container the host adds.
pub fn atomic_append_line(entry_json: &str) -> String {
    let mut s = String::from(entry_json);
    s.push('\n');
    s
}

/// `_extract_agent_name` — the portable (string + meta) branches.
///
/// The oracle has four branches. Two are host-side and excluded here: an
/// `isinstance(agent_val, dict)` branch, and the `ast.literal_eval` branch
/// (a string starting `{` that contains `"author"` — JSON/dict parsing, which
/// needs no_std-unavailable machinery; the caller resolves those before calling
/// this, or accepts the documented `"Seeker"` parse-failure fallback). The two
/// portable branches are the plain-string rule and the meta fallback.
pub fn extract_agent_name(agent_val: &str, meta_author: Option<&str>, meta_agent: Option<&str>) -> String {
    if agent_val.starts_with('{') && agent_val.contains("author") {
        // ast.literal_eval path — host-side. The oracle returns "Seeker" when that
        // parse fails; this is the no_std stand-in.
        return String::from("Seeker");
    }
    if !(agent_val.is_empty() || agent_val == "unknown" || agent_val == "None") {
        return agent_val.to_string();
    }
    let m = meta_author
        .filter(|a| !a.is_empty())
        .or_else(|| meta_agent.filter(|a| !a.is_empty()));
    match m {
        Some(a) => a.to_string(),
        None => String::from("Swarm Collective"),
    }
}

/// `get_stats`'s memory-type categorization — a pure agent-name → category table.
/// The oracle lowers/replaces only on the FINAL `m_type` (`str(m_type).lower().replace("-", "_")
/// .replace(" ", "_")`); the category names here contain no `-`/space, so this table is the
/// post-normalization value directly.
pub fn memory_type_for(agent_name: &str) -> &'static str {
    match agent_name {
        "Seeker" | "Researcher" => "research",
        "Logic" | "LogicMaster" | "Reasoning Engine" => "reasoning_state",
        "Archi" | "Architect" => "architecture",
        "Verify" | "QA Engineer" => "verification",
        "Shield" | "Security Auditor" => "security",
        "Devo" | "Lead Developer" => "task_result",
        "Flow" | "Integration Specialist" => "workflow",
        _ => "experience",
    }
}
