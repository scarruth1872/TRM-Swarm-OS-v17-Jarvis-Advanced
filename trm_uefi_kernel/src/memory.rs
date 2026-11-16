//! TRM Swarm OS - dual-layer agent memory.
//!
//! Port of `swarm_v2/core/memory.py` (`AgentMemory`: short-term conversation +
//! long-term persisted knowledge).
//!
//! SWEEP CONTEXT: this file previously held ONLY the `MEMORY_DIR` constant -
//! twelve lines, ZERO functions. The whole AgentMemory API was absent while the
//! module sat in the registry as ported. See tools/surface_diff.py.
//!
//! no_std ADAPTATIONS (each is a real semantic difference, not cosmetic):
//!  * Python persisted to `swarm_v2_memory/<agent>_memory.json`. The kernel has
//!    no filesystem here, so the store is in-memory and `export`/`restore` are
//!    the boundary - the NAND/mmap layer owns durability.
//!  * Python stamped `datetime.now().isoformat()`. The kernel has a monotonic
//!    clock, so entries carry `at_ns`.
//!  * Python's `content[:500]` is BYTE slicing. Rust byte-slicing panics on a
//!    non-UTF8 boundary, so every truncation goes through `trunc_chars`.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub const MEMORY_DIR: &'static str = "swarm_v2_memory";

/// Responses starting with these are summarised, never stored raw.
pub const ACTION_TAG_PREFIXES: &[&str] = &[
    "WRITE_FILE:", "SEARCH_QUERY:", "CREATE_FILES:", "PLAN_FILE:", "MAKE_DIR:",
    "CALL_TOOL:", "DELEGATE_TASK:", "REJECT_ARTIFACT:", "APPROVE_ARTIFACT:",
    "TEST_ARTIFACT:",
];

/// Poisoned/echoed responses - never stored and never injected into context.
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

/// Char-safe truncation. Python's `s[:n]` is byte slicing; doing that in Rust
/// panics when byte `n` lands inside a multi-byte codepoint.
pub fn trunc_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntryKind { Fact, TaskResult, ConversationArchive }

/// One long-term entry. Python stored these as dicts with a "type" key; a
/// tagged struct is the no_std equivalent and cannot drift from its kind.
#[derive(Clone, Debug)]
pub struct MemoryEntry {
    pub kind: EntryKind,
    pub content: String,
    pub source: String,
    pub task: String,
    pub status: String,
    pub at_ns: u64,
}

/// One short-term conversation turn.
#[derive(Clone, Debug)]
pub struct Turn {
    pub sender: String,
    pub content: String,
    pub role: String,
    pub at_ns: u64,
}

/// Result of `compress_history` (Python returned a plain dict).
#[derive(Clone, Copy, Debug)]
pub struct CompressReport {
    pub original: usize,
    pub compressed: usize,
    pub kept_verbatim: usize,
    pub original_chars: usize,
    pub compressed_chars: usize,
    /// compressed_chars / max(1, original_chars), rounded to 3 dp like Python.
    pub ratio_milli: u32,
}

/// Counts from `get_stats`.
#[derive(Clone, Copy, Debug)]
pub struct MemoryStats {
    pub short_term_count: usize,
    pub long_term_count: usize,
    pub facts: usize,
    pub tasks_completed: usize,
    pub archived_conversations: usize,
}

#[derive(Clone, Debug)]
pub struct AgentMemory {
    pub agent_name: String,
    pub short_term_limit: usize,
    pub short_term: Vec<Turn>,
    pub long_term: Vec<MemoryEntry>,
}

/// Round a f64 to 3 decimal places, then scale to milli-units for an integer
/// representation (the kernel avoids float formatting).
fn round3_milli(x: f64) -> u32 {
    let scaled = x * 1000.0;
    let r = crate::float_math::round_f64(scaled);
    if r < 0.0 { 0 } else { r as u32 }
}

impl AgentMemory {
    /// Python read `swarm_v2_memory/<agent>_memory.json` here; the kernel starts
    /// empty and `restore` is the durability boundary.
    pub fn new(agent_name: &str, short_term_limit: usize) -> Self {
        AgentMemory {
            agent_name: agent_name.to_string(),
            short_term_limit,
            short_term: Vec::new(),
            long_term: Vec::new(),
        }
    }

    /// True if the text matches any POISON_PATTERN (Python `any(p in text ...)`).
    pub fn is_poisoned(text: &str) -> bool {
        POISON_PATTERNS.iter().any(|p| text.contains(p))
    }

    /// If content starts with an action tag, replace it with a brief summary so
    /// the model does not echo raw action payloads back into context.
    /// Mirrors `_summarize_if_action`.
    pub fn summarize_if_action(content: &str) -> String {
        let stripped = content.trim();
        let first_line = |tag: &str| -> String {
            stripped
                .split('\n')
                .next()
                .unwrap_or("")
                .replace(tag, "")
                .trim()
                .to_string()
        };
        if stripped.starts_with("WRITE_FILE:") {
            return format!("[wrote file: {}]", first_line("WRITE_FILE:"));
        }
        if stripped.starts_with("SEARCH_QUERY:") {
            return format!("[searched: {}]", first_line("SEARCH_QUERY:"));
        }
        if stripped.starts_with("CREATE_FILES:") {
            return "[created multiple files]".to_string();
        }
        if stripped.starts_with("PLAN_FILE:") {
            return format!("[wrote plan: {}]", first_line("PLAN_FILE:"));
        }
        if stripped.starts_with("DELEGATE_TASK:") {
            // Python: stripped[14:60].strip()
            let body = trunc_chars(&stripped[14.min(stripped.len())..], 46);
            return format!("[delegated task: {}]", body.trim());
        }
        if ACTION_TAG_PREFIXES.iter().any(|t| stripped.starts_with(t)) {
            return "[action completed]".to_string();
        }
        content.to_string()
    }

    /// Add a turn. Action-tag responses from non-user senders are summarised.
    /// Mirrors `add_turn`, including the 5-turn overflow to long-term.
    pub fn add_turn(&mut self, sender: &str, content: &str, role: Option<&str>, now_ns: u64) {
        let stored = if sender != "user" {
            Self::summarize_if_action(content)
        } else {
            content.to_string()
        };
        self.short_term.push(Turn {
            sender: sender.to_string(),
            content: stored,
            role: role.unwrap_or(&self.agent_name).to_string(),
            at_ns: now_ns,
        });
        if self.short_term.len() > self.short_term_limit {
            let overflow: Vec<Turn> = self.short_term.drain(..5).collect();
            self.archive_to_long_term(&overflow, now_ns);
        }
    }

    /// Archive short-term overflow as a conversation_archive entry.
    /// Mirrors `_archive_to_long_term`.
    pub fn archive_to_long_term(&mut self, turns: &[Turn], now_ns: u64) {
        let joined: Vec<String> = turns
            .iter()
            .map(|t| format!("[{}]: {}", t.sender, trunc_chars(&t.content, 200)))
            .collect();
        self.long_term.push(MemoryEntry {
            kind: EntryKind::ConversationArchive,
            content: joined.join("\n"),
            source: String::new(),
            task: String::new(),
            status: format!("{} turns", turns.len()),
            at_ns: now_ns,
        });
    }

    /// Store a learned fact. Mirrors `add_fact`.
    pub fn add_fact(&mut self, fact: &str, source: &str, now_ns: u64) {
        self.long_term.push(MemoryEntry {
            kind: EntryKind::Fact,
            content: fact.to_string(),
            source: source.to_string(),
            task: String::new(),
            status: String::new(),
            at_ns: now_ns,
        });
    }

    /// Store a completed task result. Poisoned task/result text is DROPPED,
    /// never stored - mirrors the early return in `add_task_result`.
    /// Returns false when the write was rejected as poisoned.
    pub fn add_task_result(&mut self, task: &str, result: &str, status: &str, now_ns: u64) -> bool {
        if Self::is_poisoned(result) || Self::is_poisoned(task) {
            return false;
        }
        self.long_term.push(MemoryEntry {
            kind: EntryKind::TaskResult,
            content: trunc_chars(result, 500), // Python: result[:500]
            source: String::new(),
            task: task.to_string(),
            status: status.to_string(),
            at_ns: now_ns,
        });
        true
    }

    /// Counts by kind.
    pub fn get_stats(&self) -> MemoryStats {
        let count = |k: EntryKind| self.long_term.iter().filter(|e| e.kind == k).count();
        MemoryStats {
            short_term_count: self.short_term.len(),
            long_term_count: self.long_term.len(),
            facts: count(EntryKind::Fact),
            tasks_completed: count(EntryKind::TaskResult),
            archived_conversations: count(EntryKind::ConversationArchive),
        }
    }

    pub fn clear_short_term(&mut self) {
        self.short_term.clear();
    }

    /// Build the context string from both layers. Mirrors `get_context_window`,
    /// including the poisoned-entry filter on task results.
    pub fn get_context_window(&self, max_turns: usize) -> String {
        let mut parts: Vec<String> = Vec::new();

        let facts: Vec<&MemoryEntry> = self.long_term.iter()
            .filter(|e| e.kind == EntryKind::Fact).collect();
        if !facts.is_empty() {
            parts.push("=== Long-term Knowledge ===".to_string());
            let start = facts.len().saturating_sub(5); // Python: facts[-5:]
            for f in &facts[start..] {
                parts.push(format!("  • {}", f.content));
            }
        }

        let tasks: Vec<&MemoryEntry> = self.long_term.iter()
            .filter(|e| e.kind == EntryKind::TaskResult)
            .filter(|e| !Self::is_poisoned(&e.content) && !Self::is_poisoned(&e.task))
            .collect();
        if !tasks.is_empty() {
            parts.push("\n=== Recent Task Results ===".to_string());
            let tstart = tasks.len().saturating_sub(3); // Python: tasks[-3:]
            for t in &tasks[tstart..] {
                parts.push(format!("  [{}] {} -> {}",
                    t.status, trunc_chars(&t.task, 60), trunc_chars(&t.content, 80)));
            }
        }

        let recent = if self.short_term.len() > max_turns {
            &self.short_term[self.short_term.len() - max_turns..]
        } else {
            &self.short_term[..]
        };
        if !recent.is_empty() {
            parts.push("\n=== Recent Conversation ===".to_string());
            for turn in recent {
                parts.push(format!("  [{}]: {}", turn.sender, trunc_chars(&turn.content, 120)));
            }
        }
        parts.join("\n")
    }

    /// Recursively compress older turns for near-infinite context.
    /// Mirrors `compress_history` - messages beyond `keep_recent` collapse to a
    /// first-sentence summary; the newest `keep_recent` stay verbatim.
    pub fn compress_history(&mut self, keep_recent: usize) -> CompressReport {
        if self.short_term.len() <= keep_recent {
            return CompressReport {
                original: self.short_term.len(),
                compressed: 0,
                kept_verbatim: self.short_term.len(),
                original_chars: 0,
                compressed_chars: 0,
                ratio_milli: 1000,
            };
        }
        let split = self.short_term.len() - keep_recent;
        let older: Vec<Turn> = self.short_term[..split].to_vec();
        let recent: Vec<Turn> = self.short_term[split..].to_vec();

        let mut compressed: Vec<Turn> = Vec::new();
        for turn in &older {
            // Python: content.split(".")[0][:80].strip(), falling back to [:60]
            let head = turn.content.split('.').next().unwrap_or("");
            let mut first = trunc_chars(head, 80).trim().to_string();
            if first.is_empty() {
                first = trunc_chars(&turn.content, 60).trim().to_string();
            }
            compressed.push(Turn {
                sender: turn.sender.clone(),
                content: format!("[compressed] {}", first),
                role: turn.role.clone(),
                at_ns: turn.at_ns,
            });
        }

        let original_chars: usize = older.iter().map(|t| t.content.chars().count()).sum();
        let compressed_chars: usize = compressed.iter().map(|t| t.content.chars().count()).sum();
        let mut merged = compressed.clone();
        merged.extend(recent.iter().cloned());
        self.short_term = merged;

        let ratio = compressed_chars as f64 / (original_chars.max(1)) as f64;
        CompressReport {
            original: older.len() + recent.len(),
            compressed: compressed.len(),
            kept_verbatim: recent.len(),
            original_chars,
            compressed_chars,
            ratio_milli: round3_milli(ratio),
        }
    }

    /// Context with automatic compression applied. Mirrors `get_compressed_context`.
    pub fn get_compressed_context(&mut self, max_turns: usize) -> String {
        if self.short_term.len() > max_turns {
            let _ = self.compress_history(max_turns / 2);
        }
        self.get_context_window(max_turns)
    }
}
