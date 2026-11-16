//! TRM Swarm OS - kanban_board (no_std port, TRM platform engine).
//! Pure portable logic extracted 1:1 from swarm_v2/core/kanban_board.py.
//!
//! Ported:
//!   * `auto_assign_agent`          — council domain-ontology assignment (substring scan).
//!   * `kanban_tally`               — `native_kanban_tally`'s Python fallback (priority/status
//!                                    tally + board-health formula), the path the oracle takes
//!                                    when the strip DLL is absent.
//!   * `is_valid_transition` / `allowed_transitions` — the TRANSITIONS table rule.
//!   * `status_index` / `priority_index` — the get_stats() index literals + defaults.
//!   * `is_council_agent`           — the COUNCIL_AGENTS membership used by `_load`.
//!   * `COUNCIL_AGENTS` / `AGENT_DOMAINS` / `TRANSITIONS` / `PRIORITY_ORDER` tables.
//!
//! Host-side (no no_std spelling; deliberately absent — see the generator's exclusions):
//!   * file I/O (`_load`/`_save`, `KANBAN_STATE_PATH`, `os.makedirs`),
//!   * `datetime.now()` (created_at/updated_at/completed_at), `uuid.uuid4()` (card ids),
//!   * the worktree/port manager imports (`_on_start`/`_on_complete`), `logging`,
//!   * the file-backed `KanbanBoard` singleton (`get_kanban_board`).

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

// ── CONSTANTS ─────────────────────────────────────────────────────────────────

/// Council agents (the oracle's COUNCIL_AGENTS, in the oracle's order).
pub const COUNCIL_AGENTS: [&str; 14] = [
    "Hermes", "Jarvis", "Bridge", "Shield", "Logic", "Devo", "Pulse",
    "Flow", "EventLedger", "Orchestra", "THzGateway", "Verify", "Seeker", "Archi",
];

/// Column transition table (the oracle's TRANSITIONS: CardStatus value -> allowed targets,
/// in the oracle's order). `move_card` validates `target in TRANSITIONS.get(current, [])`.
pub const TRANSITIONS: &[(&str, &[&str])] = &[
    ("TODO", &["IN_PROGRESS", "DONE"]),
    ("IN_PROGRESS", &["REVIEW", "DONE", "TODO"]),
    ("REVIEW", &["DONE", "IN_PROGRESS", "TODO"]),
    ("DONE", &["ARCHIVED", "TODO", "IN_PROGRESS"]),
    ("ARCHIVED", &["TODO"]),
];

/// Priority sort order (the oracle's PRIORITY_ORDER: lower index = higher priority).
/// get_stats() builds the same mapping inline as `prio_index` and looks it up with
/// `.lower()` + default 2, which is what `priority_index` below implements.
pub const PRIORITY_ORDER: &[(&str, i32)] = &[
    ("critical", 0),
    ("high", 1),
    ("medium", 2),
    ("low", 3),
];

/// The council's domain ontology, in the oracle's order: fourteen agents, keyword counts as
/// the oracle has them (a fixed-size array per agent would only invite a miscount).
pub const AGENT_DOMAINS: [(&str, &[&str]); 14] = [
    ("Verify", &["test", "fuzz", "qa", "assert", "mock", "chaos", "check"]),
    ("Shield", &["security", "vuln", "qkd", "ddr", "antibody", "auth", "waf", "crypto", "protect", "scan"]),
    ("Pulse", &["gpu", "vulkan", "telemetry", "vital", "hardware", "metric", "observab", "power", "thermal", "voltage", "hvbe"]),
    ("Logic", &["logic", "prover", "math", "theorem", "formal", "k3", "kleene", "proof", "inference"]),
    ("Devo", &["ci", "pipeline", "refactor", "build", "compile", "git", "syntax", "format", "lint", "docker"]),
    ("Archi", &["architecture", "spec", "blueprint", "design", "overview", "continuum", "scaffold", "layout"]),
    ("Flow", &["mesh", "k8s", "infra", "container", "network", "route", "p2p", "worktree", "density"]),
    ("Seeker", &["research", "paper", "discovery", "science", "drug", "bio", "exome", "molecule", "reconnaissance"]),
    ("EventLedger", &["ledger", "journal", "consensus", "audit", "record", "pbft", "monad"]),
    ("Orchestra", &["orchestrat", "schedule", "dispatch", "multi-agent", "batch", "monitoring"]),
    ("THzGateway", &["thz", "rf", "frequency", "spectrum", "antenna", "carrier"]),
    ("Jarvis", &["memory", "turbovec", "vector", "persona", "embedding"]),
    ("Hermes", &["chat", "comm", "dialogue", "message", "external", "expert"]),
    ("Bridge", &["ffi", "ipc", "socket", "bridge", "adapter", "connector"]),
];

// ── KanbanCard ────────────────────────────────────────────────────────────────

/// One history entry: the oracle's `List[Dict[str, str]]` with the fixed keys
/// `action`, `timestamp`, `details`.
#[derive(Debug, Clone)]
pub struct KanbanHistoryEntry {
    pub action: String,
    pub timestamp: String,
    pub details: String,
}

/// A task card on the Kanban board (the oracle's `KanbanCard` dataclass).
/// `created_at`/`updated_at` carry `datetime.now().isoformat()` host-side; here they
/// are plain strings so the pure data model stays portable.
#[derive(Debug, Clone)]
pub struct KanbanCard {
    pub card_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub assignee: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub worktree_path: String,
    pub allocated_port: i64,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: String,
    pub history: Vec<KanbanHistoryEntry>,
}

impl KanbanCard {
    /// Dataclass defaults: status = "TODO", priority = "medium", everything else empty/zero.
    pub fn new() -> Self {
        KanbanCard {
            card_id: String::new(),
            title: String::new(),
            description: String::new(),
            status: String::from("TODO"),
            assignee: String::new(),
            priority: String::from("medium"),
            tags: Vec::new(),
            worktree_path: String::new(),
            allocated_port: 0,
            created_at: String::new(),
            updated_at: String::new(),
            completed_at: String::new(),
            history: Vec::new(),
        }
    }
}

// ── auto_assign_agent ─────────────────────────────────────────────────────────
//
// `auto_assign_agent` lowercases `f"{title} {description} {' '.join(tags)}"`, then walks
// AGENT_DOMAINS in ORDER -- agents first, then their keywords -- and returns the first agent
// with any keyword occurring as a SUBSTRING. No keywords match anywhere -> "Devo".
//
// The table is the council's own domain ontology, and the substring semantics are the oracle's,
// not an accident to be tidied: "ci" in the Devo list matches inside words like "efficient"
// (effi-ci-ent), so a card titled "make the parser efficient" resolves to Devo before Archi ever
// sees "design". The only textual divergence from the oracle is trailing whitespace when `tags`
// is empty (the oracle's f-string keeps one literal space; this port emits none) -- it can never
// change the result because no keyword contains a space, so substring containment is unaffected.

/// The oracle's assignment: first agent in table order with a matching keyword, else "Devo".
pub fn auto_assign_agent(title: &str, description: &str, tags: &[&str]) -> &'static str {
    let mut text = String::from(title);
    text.push(' ');
    text.push_str(description);
    for t in tags.iter() {
        text.push(' ');
        text.push_str(t);
    }
    let low = text.to_lowercase();
    for (agent, keywords) in AGENT_DOMAINS.iter() {
        for kw in keywords.iter() {
            if low.contains(*kw) {
                return agent;
            }
        }
    }
    "Devo"
}

// ── kanban_tally (native_kanban_tally's Python fallback) ──────────────────────
//
// The oracle's `native_kanban_tally` has two paths. With the strip DLL loaded it dispatches to
// `strip_kanban_tally` (float32 health); without it (the fallback, and the path this port
// targets) it counts priorities/statuses itself and computes health in float64. The fallback:
//   * `zip(prios, stats)` truncates to the SHORTER list;
//   * a priority is counted only when `0 <= p < 4`, a status only when `0 <= s < 5`;
//   * `crit_todo` = pairs with `p == 0 and s == 0`;
//   * `raw_score = done*2.0 + review*1.5 - in_progress*0.5 - crit_todo*3.0`;
//   * `health = clamp(50.0 + raw_score * 0.05, 0.0, 100.0)` (float64).

pub struct KanbanTally {
    pub prio_counts: [i32; 4],
    pub status_counts: [i32; 5],
    pub crit_todo: i32,
    pub health: f64,
}

/// `native_kanban_tally(prios, stats)` fallback. Health is float64, computed in the oracle's
/// exact operation order so the result is bit-identical to the Python fallback.
pub fn kanban_tally(prios: &[i32], stats: &[i32]) -> KanbanTally {
    let mut prio_counts = [0i32; 4];
    let mut status_counts = [0i32; 5];
    let n = if prios.len() < stats.len() { prios.len() } else { stats.len() };
    let mut crit_todo = 0i32;
    for i in 0..n {
        let p = prios[i];
        let s = stats[i];
        if p >= 0 && p < 4 {
            prio_counts[p as usize] += 1;
        }
        if s >= 0 && s < 5 {
            status_counts[s as usize] += 1;
        }
        if p == 0 && s == 0 {
            crit_todo += 1;
        }
    }
    let raw_score = status_counts[3] as f64 * 2.0
        + status_counts[2] as f64 * 1.5
        - status_counts[1] as f64 * 0.5
        - crit_todo as f64 * 3.0;
    let health = (50.0 + raw_score * 0.05).clamp(0.0, 100.0);
    KanbanTally { prio_counts, status_counts, crit_todo, health }
}

// ── transition rule ───────────────────────────────────────────────────────────

/// The oracle's `target in TRANSITIONS.get(current, [])`: an unknown `current` has an empty
/// allowed set, so every move out of it is invalid.
pub fn is_valid_transition(current: &str, target: &str) -> bool {
    TRANSITIONS
        .iter()
        .find(|(from, _)| *from == current)
        .map(|(_, to)| to.contains(&target))
        .unwrap_or(false)
}

/// The targets allowed from `current`, in the oracle's order (empty for an unknown status).
pub fn allowed_transitions(current: &str) -> Vec<&'static str> {
    TRANSITIONS
        .iter()
        .find(|(from, _)| *from == current)
        .map(|(_, to)| to.to_vec())
        .unwrap_or_default()
}

// ── get_stats() index literals ────────────────────────────────────────────────

/// get_stats()'s `status_index` literal: {TODO:0, IN_PROGRESS:1, REVIEW:2, DONE:3, ARCHIVED:4},
/// looked up with `.get(status, 0)` so an unknown status reads as 0 (TODO).
pub fn status_index(status: &str) -> i32 {
    match status {
        "TODO" => 0,
        "IN_PROGRESS" => 1,
        "REVIEW" => 2,
        "DONE" => 3,
        "ARCHIVED" => 4,
        _ => 0,
    }
}

/// get_stats()'s `prio_index` lookup: `prio_index.get((priority or "medium").lower(), 2)`.
/// Empty/unknown priorities fold to medium (2).
pub fn priority_index(priority: &str) -> i32 {
    match priority.to_lowercase().as_str() {
        "critical" => 0,
        "high" => 1,
        "medium" => 2,
        "low" => 3,
        _ => 2,
    }
}

// ── COUNCIL_AGENTS membership ─────────────────────────────────────────────────

/// The oracle's `_load` sanitization: `assignee not in COUNCIL_AGENTS` (and the empty/legacy
/// "Lead Developer" values) triggers re-assignment. Empty/legacy folding lives host-side;
/// this is the membership test itself.
pub fn is_council_agent(name: &str) -> bool {
    COUNCIL_AGENTS.contains(&name)
}
