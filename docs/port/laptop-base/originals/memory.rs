//! TRM Swarm OS — memory: persistent agent memory (short-term + long-term).
//!
//! Ported from `swarm_v2/core/memory.py` (287 lines) under Catalyst MOTION-006.
//!
//! Council conditions honoured:
//!   Shield — no_std + alloc only; no host-OS, filesystem, or network dependency.
//!            The Python original persists long-term memory with `open()`/`json`;
//!            that is a host responsibility and is reached through the `MemoryStore`
//!            trait, so the kernel holds the logic and the host holds the I/O.
//!   Verify — behaviour ported 1:1 from the Python source and covered by tests in
//!            `swarm_v2_artifacts/port_tests/`, driven by vectors taken from the
//!            Python implementation itself.
//!
//! Coverage: the whole pure-logic surface of the original —
//!   `_summarize_if_action`, `_is_poisoned`, `add_turn` (incl. overflow trim),
//!   `_archive_to_long_term`, `add_fact`, `add_task_result`, plus both constant
//!   tables (`ACTION_TAG_PREFIXES`, `POISON_PATTERNS`).
//! NOT ported (host-bound by design): `_load_long_term` / `_save_long_term` file
//! I/O and the Phase-15 `strip_memory_compactor.dll` fast path, which is a host
//! acceleration of the same pure logic.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Directory the host uses for long-term memory files (host-side concern).
pub const MEMORY_DIR: &str = "swarm_v2_memory";

/// Responses beginning with these tags are summarised, never stored raw.
pub const ACTION_TAG_PREFIXES: &[&str] = &[
    "WRITE_FILE:",
    "SEARCH_QUERY:",
    "CREATE_FILES:",
    "PLAN_FILE:",
    "MAKE_DIR:",
    "CALL_TOOL:",
    "DELEGATE_TASK:",
    "REJECT_ARTIFACT:",
    "APPROVE_ARTIFACT:",
    "TEST_ARTIFACT:",
];

/// Tags the platform's native compactor classifies as generic actions — these
/// collapse to `[action completed]`. Measured from the loaded DLL, NOT derived
/// from `ACTION_TAG_PREFIXES`: `MAKE_DIR`, `REJECT_ARTIFACT`, `APPROVE_ARTIFACT`
/// and `TEST_ARTIFACT` are in that table yet pass through unchanged.
pub const NATIVE_ACTION_TAGS: &[&str] = &["CREATE_FILES:", "CALL_TOOL:", "DELEGATE_TASK:"];

/// Echoed/poisoned responses — never stored and never re-injected.
pub const POISON_PATTERNS: &[&str] = &[
    "[DIRECT NEURAL BRIDGE]",
    "[HMI BRIDGE ACTIVE]",
    "[EXECUTION MODE ACTIVE]",
    "[MESH OBSERVABILITY]",
    "[STRICT CONSTRAINT]",
    "I am currently processing this request internally.",
    "My linguistic output is currently stalled",
    "## CRITICAL: Action Execution Rules",
    "WRITE_FILE: architecture_overview.md",
    "SEARCH_QUERY: architecture_overview.md",
    "[Archi:Architect] Task:",
    "[PLAN]\nI will outline the steps",
];

/// Turns moved to long-term when short-term overflows.
pub const OVERFLOW_BATCH: usize = 5;

/// One conversation turn.
#[derive(Debug, Clone, PartialEq)]
pub struct Turn {
    pub sender: String,
    pub content: String,
    pub role: String,
    pub timestamp: String,
}

/// A long-term memory entry — either an archived conversation or a learned fact.
#[derive(Debug, Clone, PartialEq)]
pub enum LongTerm {
    Archive {
        archived_at: String,
        turn_count: usize,
        turns: Vec<Turn>,
    },
    Fact {
        content: String,
        source: String,
        learned_at: String,
    },
    TaskResult {
        task: String,
        result: String,
        status: String,
    },
}

/// Host-side persistence. The kernel owns the logic; the host owns the bytes.
pub trait MemoryStore {
    fn load(&self, agent: &str) -> Vec<LongTerm>;
    fn save(&self, agent: &str, entries: &[LongTerm]);
}

/// A store that keeps everything in memory — correct for a no-persistence node.
#[derive(Debug, Default)]
pub struct VolatileStore;

impl MemoryStore for VolatileStore {
    fn load(&self, _agent: &str) -> Vec<LongTerm> {
        Vec::new()
    }
    fn save(&self, _agent: &str, _entries: &[LongTerm]) {}
}

/// Dual-layer memory: short-term (recent context) + long-term (persisted knowledge).
pub struct AgentMemory {
    pub agent_name: String,
    pub short_term_limit: usize,
    pub short_term: Vec<Turn>,
    pub long_term: Vec<LongTerm>,
    /// Set when a mutation must be flushed by the host.
    pub dirty: bool,
}

impl AgentMemory {
    pub fn new(agent_name: &str, short_term_limit: usize) -> Self {
        AgentMemory {
            agent_name: agent_name.to_string(),
            short_term_limit,
            short_term: Vec::new(),
            long_term: Vec::new(),
            dirty: false,
        }
    }

    /// Restore using a host store (the Python `_load_long_term`).
    pub fn with_store(agent_name: &str, short_term_limit: usize, store: &dyn MemoryStore) -> Self {
        let mut m = AgentMemory::new(agent_name, short_term_limit);
        m.long_term = store.load(agent_name);
        m
    }

    /// Flush to the host store if anything changed (the Python `_save_long_term`).
    pub fn flush(&mut self, store: &dyn MemoryStore) {
        if self.dirty {
            store.save(&self.agent_name, &self.long_term);
            self.dirty = false;
        }
    }

    /// Port of `_summarize_if_action`.
    ///
    /// Behaviour is taken from what the platform ACTUALLY runs. `memory.py` has two
    /// paths — a native compactor (`strip_memory_compactor.dll`, present and loaded)
    /// and a pure-Python fallback — and they do NOT agree. The measurements below
    /// come from the loaded native path, which is the runtime behaviour:
    ///
    /// | prefix | result |
    /// |---|---|
    /// | `WRITE_FILE:` | `[wrote file: <first-line payload>]` |
    /// | `SEARCH_QUERY:` | `[searched: <first-line payload>]` |
    /// | `PLAN_FILE:` | `[wrote plan: <first-line payload>]` |
    /// | `CREATE_FILES:`, `CALL_TOOL:`, `DELEGATE_TASK:` | `[action completed]` |
    /// | `MAKE_DIR:`, `REJECT_ARTIFACT:`, `APPROVE_ARTIFACT:`, `TEST_ARTIFACT:`, unknown | unchanged |
    ///
    /// Note the last row: those four appear in `ACTION_TAG_PREFIXES` yet the native
    /// classifier does not treat them as actions, so they pass through. The
    /// pure-Python fallback would summarise them — porting that would have been wrong.
    pub fn summarize_if_action(content: &str) -> String {
        let stripped = content.trim();
        if stripped.is_empty() {
            return content.to_string();
        }
        let first_line = stripped.split('\n').next().unwrap_or("");

        // Payload is the FIRST LINE with the tag removed, then trimmed.
        if first_line.starts_with("WRITE_FILE:") {
            return format!("[wrote file: {}]", first_line.replace("WRITE_FILE:", "").trim());
        }
        if first_line.starts_with("SEARCH_QUERY:") {
            return format!("[searched: {}]", first_line.replace("SEARCH_QUERY:", "").trim());
        }
        if first_line.starts_with("PLAN_FILE:") {
            return format!("[wrote plan: {}]", first_line.replace("PLAN_FILE:", "").trim());
        }
        for p in NATIVE_ACTION_TAGS {
            if stripped.starts_with(p) {
                return "[action completed]".to_string();
            }
        }
        content.to_string()
    }

    /// Port of `_is_poisoned`.
    pub fn is_poisoned(text: &str) -> bool {
        POISON_PATTERNS.iter().any(|p| text.contains(p))
    }

    /// Port of `add_turn`. Returns true when an overflow archive occurred.
    ///
    /// `sender == "user"` content is stored raw; every other sender is summarised,
    /// exactly as the Python does.
    pub fn add_turn(&mut self, sender: &str, content: &str, role: Option<&str>, now: &str) -> bool {
        let stored = if sender != "user" {
            Self::summarize_if_action(content)
        } else {
            content.to_string()
        };

        self.short_term.push(Turn {
            sender: sender.to_string(),
            content: stored,
            role: role.unwrap_or(&self.agent_name).to_string(),
            timestamp: now.to_string(),
        });

        if self.short_term.len() > self.short_term_limit {
            // Python: overflow = self.short_term[:5]; self.short_term = self.short_term[5:].
            // Both slices are lenient, so with fewer than 5 turns (possible when
            // short_term_limit < 4) everything is archived and short_term empties.
            // Rust's drain panics on an out-of-range end, so clamp to match.
            let n = core::cmp::min(OVERFLOW_BATCH, self.short_term.len());
            let overflow: Vec<Turn> = self.short_term.drain(..n).collect();
            self.archive_to_long_term(overflow, now);
            return true;
        }
        false
    }

    /// Port of `_archive_to_long_term`.
    pub fn archive_to_long_term(&mut self, turns: Vec<Turn>, now: &str) {
        self.long_term.push(LongTerm::Archive {
            archived_at: now.to_string(),
            turn_count: turns.len(),
            turns,
        });
        self.dirty = true;
    }

    /// Port of `add_fact`.
    pub fn add_fact(&mut self, fact: &str, source: &str, now: &str) {
        self.long_term.push(LongTerm::Fact {
            content: fact.to_string(),
            source: source.to_string(),
            learned_at: now.to_string(),
        });
        self.dirty = true;
    }

    /// Port of `add_task_result`.
    pub fn add_task_result(&mut self, task: &str, result: &str, status: &str) {
        self.long_term.push(LongTerm::TaskResult {
            task: task.to_string(),
            result: result.to_string(),
            status: status.to_string(),
        });
        self.dirty = true;
    }

    /// Facts stored so far — the common read path.
    pub fn facts(&self) -> Vec<&str> {
        self.long_term
            .iter()
            .filter_map(|e| match e {
                LongTerm::Fact { content, .. } => Some(content.as_str()),
                _ => None,
            })
            .collect()
    }
}
