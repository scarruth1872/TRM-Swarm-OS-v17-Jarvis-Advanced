//! TRM Swarm OS — microkernel_spawner (faithful no_std port).
//! Deterministic core ported from swarm_v2/core/microkernel_spawner.py:
//!   - `DETERMINISTIC_OPS` keyword table + `classify_deterministic()`: the
//!     Python function first tries an external radix-trie singleton
//!     (`swarm_v2.core.radix_trie_matcher`, not linkable in the kernel) and
//!     falls back to this exact ordered keyword matcher. That fallback IS the
//!     deterministic core ported here 1:1 (op order, keyword order,
//!     case-insensitive substring semantics).
//!   - `intercept_deterministic()`: canned deterministic-route responses.
//!     Python's `latency_us` field is wall-clock based → honest 0 here (see
//!     `// MANUAL` markers). Memory-query hits come from the out-of-kernel
//!     global-memory store → passed in as parameters.
//!   - `MicrokernelSubAgent.execute()` deterministic dispatch: continuum
//!     function → memory/latency budget table, then the Specialist sandbox
//!     fallback chain (blueprint|archi → formal|shield|verify → hotpatch|devo
//!     → targeted_reasoning) in the exact Python if/elif order; memory usage
//!     and latency constants match 1:1. Deep async bodies (DDR antibody
//!     injection, Mamba-3 SSM routing, PBFT cycles, IPC mailbox sends,
//!     global-memory contribution) are OS/host-bound → `// MANUAL`.
//!   - Sub-agent id scheme `sub_<clean_parent>_<6-hex>` replicates
//!     `uuid4().hex[:6]` using a deterministic caller-supplied nonce — uuid
//!     is non-deterministic by design, so it is *replaced*, never faked.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────
// DETERMINISTIC OPERATION INTERCEPTOR (Cognitive Microkernel Bypass)
// ─────────────────────────────────────────────────────────────

/// Keyword table mirroring the Python `DETERMINISTIC_OPS` dict verbatim
/// (insertion order preserved; both orders matter for the first-hit scan).
pub const DETERMINISTIC_OPS: [(&str, &[&str]); 10] = [
    (
        "memory_query",
        &["memory", "remember", "recall", "search memory", "find in memory", "lookup"],
    ),
    (
        "system_status",
        &["status", "health", "stats", "uptime", "system status", "state", "how are you"],
    ),
    (
        "agent_spawn",
        &["spawn", "create agent", "deploy", "new agent", "start agent"],
    ),
    ("tool_execute", &["run", "execute", "exec", "command"]),
    (
        "state_update",
        &["set", "update", "change", "modify", "configure", "write", "save"],
    ),
    ("health_check", &["health check", "ping", "alive", "online", "check"]),
    (
        "telemetry",
        &["telemetry", "metrics", "benchmark", "speed", "latency", "throughput"],
    ),
    (
        "healing",
        &["heal", "fix", "repair", "recover", "restart", "self-heal"],
    ),
    (
        "research",
        &["research", "analyze", "study", "investigate", "discover"],
    ),
    ("test", &["test", "verify", "validate", "audit", "check"]),
];

/// Classify a task as a deterministic microkernel op type.
/// Returns `None` when no keyword matches → the task "needs an LLM"
/// (semantic ambiguity), exactly like the Python fallback matcher.
///
/// // MANUAL: the Python entry point first asks the external radix-trie
/// singleton (`radix_trie_matcher.get_radix_trie().match(task)`) and only on
/// ImportError falls through to this keyword scan. The trie lives outside
/// the kernel; the fallback below is ported faithfully.
pub fn classify_deterministic(task: &str) -> Option<&'static str> {
    let task_lower = task.to_lowercase();
    for (op_type, keywords) in DETERMINISTIC_OPS.iter() {
        for kw in keywords.iter() {
            if task_lower.contains(kw) {
                return Some(*op_type);
            }
        }
    }
    None
}

/// Continuum function an intercepted op type routes to
/// (Python `intercept_deterministic` tail mapping; default
/// `instella_thinking_matrix` for unknown op types).
pub fn continuum_function_for(op_type: &str) -> &'static str {
    match op_type {
        "agent_spawn" | "tool_execute" => "hermes_agent",
        "state_update" => "event_ledger_append",
        "research" => "genomics_refraction",
        "test" => "pbft_consensus_vote",
        _ => "instella_thinking_matrix",
    }
}

/// A deterministic intercept result (`intercept_deterministic` in Python).
#[derive(Debug, Clone)]
pub struct DeterministicIntercept {
    pub bypass: &'static str,
    pub op_type: &'static str,
    pub result: String,
    /// `Some("route_to_continuum")` for ops that need real execution
    /// (agent_spawn, tool_execute, state_update, research, test).
    pub action: Option<&'static str>,
    pub continuum_function: Option<&'static str>,
    pub llm_avoided: bool,
    /// Python records wall-clock microseconds here; no clock in the kernel →
    /// honest 0. // MANUAL
    pub latency_us: u64,
}

impl DeterministicIntercept {
    fn new(op_type: &'static str, result: String) -> Self {
        DeterministicIntercept {
            bypass: "microkernel_deterministic",
            op_type,
            result,
            action: None,
            continuum_function: None,
            llm_avoided: true,
            latency_us: 0,
        }
    }
}

/// Canned `[MEMORY_QUERY]` response. `memory_hits` is the number of
/// relevant memories found in the global memory store (out-of-kernel → the
/// caller supplies it; the kernel's default path uses 0).
/// // MANUAL: `results[0][:200]` renders live memory content in Python.
pub fn intercept_memory_query(memory_hits: u64, top_result_head: Option<&str>) -> DeterministicIntercept {
    let head = match top_result_head {
        Some(t) => t.chars().take(200).collect::<String>(),
        None => "No matches.".to_string(),
    };
    let result = format!(
        "[MEMORY_QUERY] Found {} relevant memories. {}",
        memory_hits, head
    );
    DeterministicIntercept::new("memory_query", result)
}

/// Canned `[SYSTEM_STATUS]` response. `mem_entries` is the runtime
/// `gm.total_memories` value (out-of-kernel → caller-supplied).
pub fn intercept_system_status(mem_entries: u64) -> DeterministicIntercept {
    let result = format!(
        "[SYSTEM_STATUS] TRM Swarm OS v18 | v5_online | Mamba-3 SSM 0.0774μs | \
         Instella MoE 64-expert | 524 sub-agents | DDR 99.97% | Memory: {} entries",
        mem_entries
    );
    DeterministicIntercept::new("system_status", result)
}

/// Canned `[HEALTH]` response (healing / health_check ops).
pub fn intercept_health() -> DeterministicIntercept {
    let result = "[HEALTH] All systems nominal. DDR Antibody shields active \
                  (294 patterns, 99.97% detection). PHOENIX-Protocol tolerance \
                  96.8%. PBFT consensus: healthy. 509 sub-agents operational."
        .to_string();
    DeterministicIntercept::new("healing", result)
}

/// Canned `[TELEMETRY]` response.
pub fn intercept_telemetry() -> DeterministicIntercept {
    let result = "[TELEMETRY] Mamba-3: 0.0774μs/step | Instella: 0.14ms route | \
                  TurboVec: 32.6B vectors/sec | DDR: 0.183ms self-heal | \
                  Cost: $0.00018/100 tasks"
        .to_string();
    DeterministicIntercept::new("telemetry", result)
}

/// Routing directive for ops that need actual execution.
pub fn intercept_routed(op_type: &'static str) -> DeterministicIntercept {
    let result = format!(
        "[DETERMINISTIC_ROUTED] Operation '{}' intercepted at microkernel. \
         Routing to continuum function.",
        op_type
    );
    DeterministicIntercept {
        bypass: "microkernel_deterministic",
        op_type,
        result,
        action: Some("route_to_continuum"),
        continuum_function: Some(continuum_function_for(op_type)),
        llm_avoided: true,
        latency_us: 0,
    }
}

/// Intercept a task at the microkernel level. `None` → LLM fallthrough.
/// // MANUAL: Python interpolates live global-memory counts into the
/// memory_query/system_status canned strings; the kernel passes 0 counts
/// (host memory store is not linkable here).
pub fn intercept_deterministic(task: &str) -> Option<DeterministicIntercept> {
    match classify_deterministic(task)? {
        "memory_query" => Some(intercept_memory_query(0, None)),
        "system_status" => Some(intercept_system_status(0)),
        "healing" | "health_check" => Some(intercept_health()),
        "telemetry" => Some(intercept_telemetry()),
        other => Some(intercept_routed(other)),
    }
}

// ─────────────────────────────────────────────────────────────
// MicrokernelSubAgent.execute() deterministic dispatch
// ─────────────────────────────────────────────────────────────

/// Known continuum execution functions (execute() if/elif ladder).
pub const CONTINUUM_FUNCTIONS: [&str; 10] = [
    "instella_thinking_matrix",
    "genomics_refraction",
    "star_matrix_lattice",
    "pbft_consensus_vote",
    "jarvis_cognitive_ingest",
    "cognitive_synthesis",
    "trm_reasoning",
    "skill_matrix_sync",
    "event_ledger_append",
    "hermes_agent",
];

/// Deterministic outcome of the Python `execute()` dispatch ladder:
/// which sandbox path ran, the memory budget it set, and its latency
/// constant. Status is always COMPLETED in the kernel — the Python FAILED
/// branch only fires on async/host exceptions (DDR/Mamba-3/mailbox), all of
/// which are // MANUAL here.
#[derive(Debug, Clone)]
pub struct ExecutionOutcome {
    /// Branch label: the continuum_function name or the specialist
    /// sub_task value (blueprinting_validation / ast_invariant_proof /
    /// hotpatch_sandbox_test / targeted_reasoning).
    pub label: &'static str,
    pub memory_usage_mb: f64,
    pub latency_ms: f64,
}

fn outcome(label: &'static str, memory_usage_mb: f64, latency_ms: f64) -> ExecutionOutcome {
    ExecutionOutcome {
        label,
        memory_usage_mb,
        latency_ms,
    }
}

/// Ports the MicrokernelSubAgent.execute() if/elif ladder deterministically.
/// Python evaluates the nine continuum_function branches first; any other
/// value (or None) falls through to the Specialist keyword sandbox checks,
/// then to targeted_reasoning. Order and substring semantics match 1:1.
pub fn execute_dispatch(
    parent_role: &str,
    task_spec: &str,
    continuum_function: Option<&str>,
) -> ExecutionOutcome {
    if let Some(f) = continuum_function {
        match f {
            "instella_thinking_matrix" => return outcome("instella_thinking_matrix", 24.2, 0.18),
            "genomics_refraction" => return outcome("genomics_refraction", 16.4, 0.24),
            "star_matrix_lattice" => return outcome("star_matrix_lattice", 14.8, 0.31),
            "pbft_consensus_vote" => return outcome("pbft_consensus_vote", 22.5, 0.39),
            "jarvis_cognitive_ingest" => return outcome("jarvis_cognitive_ingest", 32.5, 0.12),
            "cognitive_synthesis" => return outcome("cognitive_synthesis", 45.0, 0.85),
            "trm_reasoning" => return outcome("trm_reasoning", 28.0, 0.45),
            "skill_matrix_sync" => return outcome("skill_matrix_sync", 8.0, 0.05),
            "event_ledger_append" => return outcome("event_ledger_append", 6.5, 0.02),
            "hermes_agent" => return outcome("hermes_agent", 16.0, 0.12),
            _ => {} // unknown continuum value → Specialist fallback chain
        }
    }

    // Specialist Agent Fallback Sandbox Execution Paths
    let parent_l = parent_role.to_lowercase();
    let task_l = task_spec.to_lowercase();

    if task_l.contains("blueprint") || parent_l.contains("archi") {
        // Blueprint GAN validation path.
        outcome("blueprinting_validation", 18.2, 0.35)
    } else if task_l.contains("formal") || parent_l.contains("shield") || parent_l.contains("verify") {
        // AST formal verification path.
        outcome("ast_invariant_proof", 12.4, 0.28)
    } else if task_l.contains("hotpatch") || parent_l.contains("devo") {
        // Hotpatch sandbox test path.
        outcome("hotpatch_sandbox_test", 24.1, 0.41)
    } else {
        // Python re-checks `skill_matrix_sync`/`event_ledger_append` here but
        // those were already handled above → unreachable dead code, skipped.
        outcome("targeted_reasoning", 14.0, 0.15)
    }
}

// ─────────────────────────────────────────────────────────────
// Sub-agent id scheme
// ─────────────────────────────────────────────────────────────

/// Deterministic replacement for the Python
/// `f"sub_{clean_parent}_{uuid.uuid4().hex[:6]}"`. `uuid4().hex[:6]` is six
/// random lowercase hex digits; here `nonce & 0xFFFFFF` is zero-padded to six
/// hex digits so the format is identical while staying deterministic.
pub fn make_subagent_id(parent_role: &str, nonce: u64) -> String {
    let clean: String = parent_role
        .to_lowercase()
        .chars()
        .map(|c| if c == ' ' { '_' } else { c })
        .collect();
    format!("sub_{}_{:06x}", clean, nonce & 0xFF_FFFF)
}

// ─────────────────────────────────────────────────────────────
// FractalTelemetryBus
// ─────────────────────────────────────────────────────────────

pub const TELEMETRY_BUS_MAX_EVENTS: usize = 1000;

/// High-frequency telemetry event (Python dict fields; `timestamp` was an
/// ISO-8601 wall-clock string → // MANUAL, dropped in the no_std kernel).
#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    pub subagent_id: String,
    pub function: String,
    pub latency_ms: f64,
    pub memory_mb: f64,
    pub resonance: f64,
}

/// Bounded high-speed telemetry buffer: append, and drop the oldest event
/// once the buffer exceeds 1000 entries (Python `events.pop(0)`).
#[derive(Debug, Clone, Default)]
pub struct FractalTelemetryBus {
    pub events: Vec<TelemetryEvent>,
}

impl FractalTelemetryBus {
    pub fn new() -> Self {
        FractalTelemetryBus { events: Vec::new() }
    }

    pub fn emit(
        &mut self,
        subagent_id: &str,
        function: &str,
        latency_ms: f64,
        memory_mb: f64,
        resonance: f64,
    ) {
        self.events.push(TelemetryEvent {
            subagent_id: subagent_id.to_string(),
            function: function.to_string(),
            latency_ms,
            memory_mb,
            resonance,
        });
        if self.events.len() > TELEMETRY_BUS_MAX_EVENTS {
            self.events.remove(0);
        }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

// ─────────────────────────────────────────────────────────────
// MicrokernelSpawner registry
// ─────────────────────────────────────────────────────────────

/// Registry entry mirroring `MicrokernelSubAgent` public state.
#[derive(Debug, Clone)]
pub struct SubagentRecord {
    pub subagent_id: String,
    pub parent_role: String,
    pub subagent_name: String,
    pub task_spec: String,
    pub persona: String,
    pub continuum_function: Option<String>,
    pub status: String,
    pub memory_usage_mb: f64,
    pub memory_limit_mb: u64,
    pub ttl_seconds: u64,
}

/// Process manager port (`MicrokernelSpawner`). Python keys a dict by
/// subagent_id; the no_std port keeps insertion-ordered `Vec` records with
/// the same lookup/terminate semantics.
#[derive(Debug, Clone, Default)]
pub struct MicrokernelSpawner {
    pub subagents: Vec<SubagentRecord>,
}

impl MicrokernelSpawner {
    pub fn new() -> Self {
        MicrokernelSpawner {
            subagents: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.subagents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.subagents.is_empty()
    }

    /// Registers a sub-agent and runs the deterministic execute() dispatch.
    /// Returns the generated subagent_id. Mirrors
    /// `spawn_subagent()` + `MicrokernelSubAgent.execute()`.
    ///
    /// // MANUAL: Python then contributes to global memory and the parent
    /// mailbox (async, host-side) — out of kernel scope.
    pub fn spawn_subagent(
        &mut self,
        parent_role: &str,
        subagent_name: &str,
        task_spec: &str,
        persona: Option<&str>,
        continuum_function: Option<&str>,
        memory_limit_mb: u64,
        ttl_seconds: u64,
        id_nonce: u64,
    ) -> String {
        let persona = persona.unwrap_or("continuum");
        let subagent_id = make_subagent_id(parent_role, id_nonce);
        let exec = execute_dispatch(parent_role, task_spec, continuum_function);
        self.subagents.push(SubagentRecord {
            subagent_id: subagent_id.clone(),
            parent_role: parent_role.to_string(),
            subagent_name: subagent_name.to_string(),
            task_spec: task_spec.to_string(),
            persona: persona.to_string(),
            continuum_function: continuum_function.map(|s| s.to_string()),
            status: "COMPLETED".to_string(),
            memory_usage_mb: exec.memory_usage_mb,
            memory_limit_mb,
            ttl_seconds,
        });
        subagent_id
    }

    /// Syscall SYSCALL_TERMINATE_SUBAGENT: flips status to TERMINATED
    /// (Python keeps the record in the dict; it is not removed).
    pub fn terminate_subagent(&mut self, subagent_id: &str) -> &'static str {
        for sa in self.subagents.iter_mut() {
            if sa.subagent_id == subagent_id {
                sa.status = "TERMINATED".to_string();
                return "TERMINATED";
            }
        }
        "NOT_FOUND"
    }

    pub fn get_record(&self, index: usize) -> Option<&SubagentRecord> {
        self.subagents.get(index)
    }

    pub fn find(&self, subagent_id: &str) -> Option<&SubagentRecord> {
        self.subagents.iter().find(|sa| sa.subagent_id == subagent_id)
    }

    /// Syscall SYSCALL_GET_SUBAGENT_METRICS: number of live records.
    pub fn process_table_len(&self) -> usize {
        self.subagents.len()
    }
}
