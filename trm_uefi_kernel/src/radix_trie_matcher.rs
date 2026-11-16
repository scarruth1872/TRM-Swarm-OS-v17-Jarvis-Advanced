//! TRM Swarm OS — Radix-Trie Deterministic Fast-Path Matcher (no_std port).
//! 1:1 core semantics from swarm_v2/core/radix_trie_matcher.py.
//! O(len) character-trie lookup with function dispatch; preloaded with the 88
//! DETERMINISTIC_HANDLERS patterns of the python module. Trie keys are stored
//! lowercased (python register() lowercases the pattern). Handler functions
//! whose python body imports another live engine are marked // MANUAL and
//! return an honest static descriptor (no fabricated live-state numbers).

extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Handler type: python Callable returning a status string.
pub type Handler = fn() -> &'static str;

/// ---------------------------------------------------------------------------
/// Pure static handlers (python bodies return these strings verbatim).
/// ---------------------------------------------------------------------------
pub fn _handle_system_status() -> &'static str {
    "System v5_online — 12 agents, TurboVecNative memory, HVBE active"
}
pub fn _handle_health_check() -> &'static str {
    "Healthy — all services responding"
}
pub fn _handle_healing() -> &'static str {
    "SelfHealing: mesh intact, DDR antibodies active (294 patterns)"
}
pub fn _handle_research() -> &'static str {
    "Research: indexed findings in TurboVec native memory"
}
pub fn _handle_test() -> &'static str {
    "Test harness: 29 unit tests, mutation coverage 85%"
}
pub fn _handle_state_update() -> &'static str {
    "State updated via deterministic path — no LLM called"
}
pub fn _handle_code_compile() -> &'static str {
    "Devo: Hot-patch compiler ready, sandbox TTL=15s, transactional C-types"
}
pub fn _handle_search() -> &'static str {
    "Seeker: indexed findings, 110 reconnaissance cycles"
}
pub fn _handle_audit() -> &'static str {
    "Verify: FVE passed, AST safety proofs INV-001 to INV-004 enforced"
}
pub fn _handle_connect() -> &'static str {
    "Flow: 8 service mesh nodes, Istio mTLS, 100GbE HBTF fabric"
}
pub fn _handle_fetch() -> &'static str {
    "Bridge: HTTP/3 QUIC ready, OAuth2 tokens cached, rate-limited at 100 req/s"
}
pub fn _handle_observe() -> &'static str {
    "EventLedger: OpenTelemetry spans active, Prometheus metrics exporting"
}

/// MANUAL: python body imports swarm_v2.core.ddr_antibody.get_ddr() (side
/// effect: ensures DDR loaded) then returns this static string verbatim.
pub fn _handle_security_audit() -> &'static str {
    "Shield: DDR antibodies active, 99.97% detection rate, 294 patterns"
}

/// MANUAL: python body imports swarm_v2.core.ternary_kleene_engine (side
/// effect only) then returns this static string verbatim.
pub fn _handle_theorem_prove() -> &'static str {
    "Logic: K3 engine online, 5-trit packing active"
}

pub fn _handle_jarvis_chat() -> &'static str {
    "Jarvis Chat Module: Express.js API on port 3000, 16 integrated modules, TRM bridge on :8021"
}
pub fn _handle_jarvis_skills() -> &'static str {
    "Jarvis Skills: QuantumBiomimeticArchitect, ResonantFormalVerification, + cross-platform synthesis, fidelity 0.98"
}
pub fn _handle_jarvis_memory() -> &'static str {
    "Jarvis Memory System: 6 API endpoints, core/episodic/semantic memory, chat history, skills registry"
}
pub fn _handle_jarvis_swarm() -> &'static str {
    "Jarvis Swarm Integration: TRM microkernel on :8021, Radix-Trie + LoRA pipeline, 584 agents"
}

/// MANUAL handlers — python bodies query live engines (dynamic numbers);
/// returning fabricated numbers would be dishonest, so each states its engine.
pub fn _handle_memory_query() -> &'static str {
    "TurboVec memory query (MANUAL: python body reads swarm_v2.core.turbovec_memory live state)"
}
pub fn _handle_agent_spawn() -> &'static str {
    "Microkernel spawner query (MANUAL: python body reads swarm_v2.core.microkernel_spawner live state)"
}
pub fn _handle_telemetry() -> &'static str {
    "HVBE Phi_V telemetry (MANUAL: python body reads swarm_v2.core.hardware_voltage_microkernel_binder live state)"
}
pub fn _handle_blueprint() -> &'static str {
    "Archi blueprint status (MANUAL: python body reads swarm_v2.core.generative_architect live state)"
}
pub fn _handle_orchestrate() -> &'static str {
    "Orchestra task stats (MANUAL: python body reads swarm_v2.core.digital_twin_singleton live state)"
}
pub fn _handle_cosmos_bridge() -> &'static str {
    "CosmosBridge stats (MANUAL: python body reads swarm_v2.core.cosmos3_edge_bridge live state)"
}
pub fn _handle_hvbe_telemetry() -> &'static str {
    "HVBE snapshot telemetry (MANUAL: python body reads swarm_v2.core.hardware_voltage_microkernel_binder live state)"
}
pub fn _handle_lora_status() -> &'static str {
    "LoRA adapter registry (MANUAL: python body reads swarm_v2.core.kernel_lora_adapter live state)"
}
pub fn _handle_thz_gateway() -> &'static str {
    "THz Gateway stats (MANUAL: python body reads swarm_v2.core.thz_wave_gateway live state)"
}

/// Lambda handlers (python `lambda: "..."` bodies → named fns; output verbatim).
pub fn _lambda_persistent_memory() -> &'static str {
    "NVMe-backed mmap arena: 64MB persistent, survives restarts, <5ms resume"
}
pub fn _lambda_write_ahead_log() -> &'static str {
    "FlatBuffers WAL: binary append-only, 5 records, crash-safe replay"
}
pub fn _lambda_checkpoint_status() -> &'static str {
    "Digital Twin checkpoint: 500ms dual-buffer, max 500ms cognitive loss"
}
pub fn _lambda_calibration_status() -> &'static str {
    "HVBE cold-start calibration: 50 samples, epsilon=0.0000, stable"
}
pub fn _lambda_biomimetic() -> &'static str {
    "Biomimetic Lab: AlphaFold platform, myo_oxy fold (oxygen-storing monomer), CFTR F508del genome, quantum biomimetic architecture"
}
pub fn _lambda_alphafold() -> &'static str {
    "AlphaFold: 3D protein docking active, myoglobin fold preset, tau aggregation inhibitor screening"
}
pub fn _lambda_protein_folding() -> &'static str {
    "Protein Folding: Mamba-3 SSM energy minimization, Kleene ternary state mapping from fold states"
}
pub fn _lambda_tool_routing() -> &'static str {
    "Hermes Agent: Tool routing active — 78 trie patterns, desktop orchestrator, multi-agent coordination via swarm chat"
}
pub fn _lambda_desktop_automation() -> &'static str {
    "Hermes Agent: Desktop automation online — file ops, terminal control, process management, pane control"
}
pub fn _lambda_skill_synthesis() -> &'static str {
    "Hermes Agent: Skill synthesis engine — cross-platform skill creation, auto-patch, swarm+jarvis integration"
}
pub fn _lambda_hello_hermes() -> &'static str {
    "Hermes Agent v18.52 — Desktop orchestration platform. 15 agents, 72 trie patterns, τ=2.82μs. How can I assist, Boss?"
}
pub fn _lambda_hermes_status() -> &'static str {
    "Hermes Agent: 81 commits, 15 agents, 76+ patterns, τ=2.82μs, $0.00000006/task, running on Ryzen 9 3900X + RX 6700 XT"
}
pub fn _lambda_what_is_your_role() -> &'static str {
    "Hermes Agent: Desktop orchestration, tool integration, multi-agent coordination across 15 swarm agents"
}
pub fn _lambda_what_can_you_do() -> &'static str {
    "Hermes Agent capabilities: tool routing (76 patterns), desktop automation, skill synthesis, swarm coordination, file operations, terminal control"
}
pub fn _lambda_how_are_you() -> &'static str {
    "Hermes Agent: v5_online, 15 agents active, τ=2.82μs, HVBE Φ_V=0.74, all systems operational"
}
pub fn _lambda_help_me() -> &'static str {
    "Hermes Agent: I can orchestrate agents, route tools, automate desktop tasks, synthesize skills, search memory, and analyze biomimetic data"
}
pub fn _lambda_can_you() -> &'static str {
    "Hermes Agent: Yes — I can route tools, automate desktops, synthesize skills, search memory, orchestrate 15 agents, and analyze biomimetic data. What do you need?"
}
pub fn _lambda_hello() -> &'static str {
    "Hermes Agent v18.52 — Desktop orchestration platform. 15 agents, 82 trie patterns, τ=2.82μs. How can I assist, Boss?"
}
pub fn _lambda_hi() -> &'static str {
    "Hermes Agent online. 15 agents, 82 patterns, τ=2.82μs. What do you need?"
}
pub fn _lambda_hey() -> &'static str {
    "Hermes Agent ready. Swarm chat active, all 15 agents available. What can I do for you?"
}
pub fn _lambda_good_morning() -> &'static str {
    "Good morning, Boss. Hermes Agent online — 15 agents, τ=2.82μs, all systems operational."
}
pub fn _lambda_good_afternoon() -> &'static str {
    "Good afternoon, Boss. Hermes Agent ready. Swarm chat active across all 15 agents."
}
pub fn _lambda_good_evening() -> &'static str {
    "Good evening, Boss. Hermes Agent standing by — 82 patterns, TurboVec 352 entries, HVBE Φ_V=0.74."
}

/// ---------------------------------------------------------------------------
/// DETERMINISTIC_HANDLERS — 1:1 transcription of the python module dict
/// (order preserved; 88 entries, matching python len(DETERMINISTIC_HANDLERS)).
/// ---------------------------------------------------------------------------
pub static DETERMINISTIC_HANDLERS: &[(&str, Handler)] = &[
    // Core system
    ("system status", _handle_system_status),
    ("health check", _handle_health_check),
    ("memory query", _handle_memory_query),
    ("agent spawn", _handle_agent_spawn),
    ("telemetry", _handle_telemetry),
    ("healing", _handle_healing),
    ("research", _handle_research),
    ("test", _handle_test),
    ("state update", _handle_state_update),
    // Short aliases
    ("status", _handle_system_status),
    ("health", _handle_health_check),
    ("memory", _handle_memory_query),
    ("spawn", _handle_agent_spawn),
    ("metrics", _handle_telemetry),
    ("repair", _handle_healing),
    ("find", _handle_research),
    ("verify", _handle_test),
    ("update", _handle_state_update),
    // Agent-specific
    ("security audit", _handle_security_audit),
    ("ddr scan", _handle_security_audit),
    ("shield scan", _handle_security_audit),
    ("theorem prove", _handle_theorem_prove),
    ("k3 check", _handle_theorem_prove),
    ("logic prove", _handle_theorem_prove),
    ("code compile", _handle_code_compile),
    ("hot patch", _handle_code_compile),
    ("devo build", _handle_code_compile),
    ("search query", _handle_search),
    ("seeker find", _handle_search),
    ("literature search", _handle_search),
    ("audit", _handle_audit),
    ("fve verify", _handle_audit),
    ("ast check", _handle_audit),
    ("blueprint", _handle_blueprint),
    ("archi design", _handle_blueprint),
    ("synthesize architecture", _handle_blueprint),
    ("connect mesh", _handle_connect),
    ("flow route", _handle_connect),
    ("service mesh", _handle_connect),
    ("fetch data", _handle_fetch),
    ("bridge fetch", _handle_fetch),
    ("http request", _handle_fetch),
    ("orchestrate", _handle_orchestrate),
    ("scheduler status", _handle_orchestrate),
    ("task queue", _handle_orchestrate),
    ("observe", _handle_observe),
    ("event log", _handle_observe),
    ("telemetry stream", _handle_observe),
    ("cosmos bridge", _handle_cosmos_bridge),
    ("cosmos status", _handle_cosmos_bridge),
    ("vla bridge", _handle_cosmos_bridge),
    ("hvbe telemetry", _handle_hvbe_telemetry),
    ("voltage binder", _handle_hvbe_telemetry),
    ("hardware status", _handle_hvbe_telemetry),
    ("lora status", _handle_lora_status),
    ("adapter list", _handle_lora_status),
    ("lora adapter", _handle_lora_status),
    // Jarvis modules
    ("jarvis chat", _handle_jarvis_chat),
    ("chat module", _handle_jarvis_chat),
    ("jarvis skills", _handle_jarvis_skills),
    ("skills module", _handle_jarvis_skills),
    ("jarvis memory", _handle_jarvis_memory),
    ("memory system", _handle_jarvis_memory),
    ("jarvis swarm", _handle_jarvis_swarm),
    ("swarm integration", _handle_jarvis_swarm),
    // Gap fix verification
    ("persistent memory", _lambda_persistent_memory),
    ("write ahead log", _lambda_write_ahead_log),
    ("checkpoint status", _lambda_checkpoint_status),
    ("calibration status", _lambda_calibration_status),
    // Biomimetic lab
    ("biomimetic", _lambda_biomimetic),
    ("alphafold", _lambda_alphafold),
    ("protein folding", _lambda_protein_folding),
    // Hermes Agent commands
    ("tool routing", _lambda_tool_routing),
    ("desktop automation", _lambda_desktop_automation),
    ("skill synthesis", _lambda_skill_synthesis),
    ("hello hermes", _lambda_hello_hermes),
    ("hermes status", _lambda_hermes_status),
    // Common Dashboard natural language queries
    ("what is your role", _lambda_what_is_your_role),
    ("what can you do", _lambda_what_can_you_do),
    ("how are you", _lambda_how_are_you),
    ("help me", _lambda_help_me),
    ("can you", _lambda_can_you),
    // Common greetings
    ("hello", _lambda_hello),
    ("hi", _lambda_hi),
    ("hey", _lambda_hey),
    ("good morning", _lambda_good_morning),
    ("good afternoon", _lambda_good_afternoon),
    ("good evening", _lambda_good_evening),
];

/// Number of preloaded deterministic patterns (python len(DETERMINISTIC_HANDLERS) = 88).
pub const DETERMINISTIC_HANDLER_COUNT: usize = DETERMINISTIC_HANDLERS.len();

/// ---------------------------------------------------------------------------
/// TrieNode (1:1 python class; children dict -> Vec<(char, TrieNode)> since
/// no_std alloc has no HashMap).
/// ---------------------------------------------------------------------------
#[derive(Debug)]
pub struct TrieNode {
    pub children: Vec<(char, Box<TrieNode>)>,
    pub handler: Option<Handler>,
    pub endpoint: Option<String>,
}

impl TrieNode {
    pub fn new() -> Self {
        TrieNode {
            children: Vec::new(),
            handler: None,
            endpoint: None,
        }
    }

    fn get_child(&self, c: char) -> Option<&TrieNode> {
        self.children
            .iter()
            .find(|(ch, _)| *ch == c)
            .map(|(_, node)| node.as_ref())
    }

    fn get_child_mut(&mut self, c: char) -> Option<&mut TrieNode> {
        self.children
            .iter_mut()
            .find(|(ch, _)| *ch == c)
            .map(|(_, node)| node.as_mut())
    }

    fn insert_child(&mut self, c: char) -> &mut TrieNode {
        if self.get_child(c).is_none() {
            self.children.push((c, Box::new(TrieNode::new())));
        }
        self.get_child_mut(c).expect("child present after insert")
    }

    // ── base-compatible aliases (the v20 tree names these differently and walks bytes) ───────────
    // Adapted to our children: Vec<(char, Box<TrieNode>)> model rather than grafted verbatim, because a
    // byte-based method cannot compile against a char-keyed node.
    pub fn find_child(&self, c: char) -> Option<&TrieNode> {
        self.get_child(c)
    }

    pub fn find_child_mut(&mut self, c: char) -> Option<&mut TrieNode> {
        self.get_child_mut(c)
    }

    pub fn child_or_insert(&mut self, c: char) -> &mut TrieNode {
        self.insert_child(c)
    }
}

/// Base-compatible view of a match (the v20 tree returns this from `match_text`).
///
/// Our node model keys on a dispatch `Handler` plus an endpoint string and has no separate `pattern`
/// field, so `pattern` here carries the endpoint text - the same value the base derives from its matched
/// pattern. The field is documented rather than silently duplicated.
#[derive(Debug, Clone)]
pub struct TrieMatch {
    /// Endpoint text of the deepest registered match.
    pub pattern: String,
    /// Registered endpoint (identical to `pattern` in this implementation).
    pub endpoint: String,
    /// Number of input CHARACTERS consumed by the longest match.
    pub match_len: usize,
}

/// RadixTrieMatcher (1:1 python class).
#[derive(Debug)]
pub struct RadixTrieMatcher {
    pub root: TrieNode,
    pub registered: u64,
}

impl RadixTrieMatcher {
    pub fn new() -> Self {
        RadixTrieMatcher {
            root: TrieNode::new(),
            registered: 0,
        }
    }

    /// Register a deterministic operation pattern -> native handler
    /// (1:1 python register; pattern is lowercased before insertion).
    pub fn register(&mut self, pattern: &str, handler: Handler, endpoint: Option<&str>) {
        let mut node = &mut self.root;
        for ch in pattern.to_lowercase().chars() {
            node = node.insert_child(ch);
        }
        node.handler = Some(handler);
        node.endpoint = endpoint.map(|e| e.to_string());
        self.registered += 1;
    }

    /// Longest-prefix match: returns (handler, endpoint, match_length)
    /// (1:1 python match_prefix).
    pub fn match_prefix(&self, text: &str) -> Option<(Handler, Option<String>, usize)> {
        let mut node = &self.root;
        let text_lower = text.to_lowercase();
        let mut last_match: Option<(Handler, Option<String>, usize)> = None;

        for (i, ch) in text_lower.chars().enumerate() {
            match node.get_child(ch) {
                None => break,
                Some(next) => {
                    node = next;
                    if node.handler.is_some() {
                        last_match = Some((node.handler.unwrap(), node.endpoint.clone(), i + 1));
                    }
                }
            }
        }
        last_match
    }

    /// Deepest match as a `TrieMatch` (base-compatible shape).
    pub fn match_text(&self, text: &str) -> Option<TrieMatch> {
        self.match_prefix(text).map(|(_handler, endpoint, match_len)| {
            let ep = endpoint.unwrap_or_default();
            TrieMatch { pattern: ep.clone(), endpoint: ep, match_len }
        })
    }

    /// Batch form: one endpoint per query, `None` where nothing matched.
    pub fn match_batch(&self, queries: &[&str]) -> Vec<Option<String>> {
        queries
            .iter()
            .map(|q| self.match_text(q).map(|m| m.endpoint))
            .collect()
    }

    /// Walk a UTF-8 byte slice as characters. The base walks raw bytes (u8 children); our nodes are
    /// char-keyed, so invalid UTF-8 yields `None` rather than a partially walked prefix.
    pub fn walk_char_bytes<'a>(&'a self, bytes: &'a [u8]) -> Option<&'a TrieNode> {
        let text = core::str::from_utf8(bytes).ok()?;
        let mut node = &self.root;
        for ch in text.to_lowercase().chars() {
            node = node.get_child(ch)?;
        }
        Some(node)
    }

    /// O(len) match — prefix first, exact fallback (1:1 python match).
    pub fn r#match(&self, text: &str) -> Option<(Handler, Option<String>)> {
        // Try longest prefix match first (partial queries like
        // 'literature search on tau...').
        if let Some((handler, endpoint, _len)) = self.match_prefix(text) {
            return Some((handler, endpoint));
        }
        // Fall back to exact match
        let mut node = &self.root;
        let text_lower = text.to_lowercase();
        for ch in text_lower.chars() {
            match node.get_child(ch) {
                None => return None,
                Some(next) => node = next,
            }
        }
        if node.handler.is_some() {
            return Some((node.handler.unwrap(), node.endpoint.clone()));
        }
        None
    }
}

impl Default for RadixTrieMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the default matcher exactly as the python module singleton does:
/// register every DETERMINISTIC_HANDLERS entry with endpoint
/// pattern.replace(" ", "_").
pub fn build_default_matcher() -> RadixTrieMatcher {
    let mut trie = RadixTrieMatcher::new();
    for (pattern, handler) in DETERMINISTIC_HANDLERS.iter() {
        let endpoint: String = pattern
            .chars()
            .map(|c| if c == ' ' { '_' } else { c })
            .collect();
        trie.register(pattern, *handler, Some(&endpoint));
    }
    trie
}

/// Singleton handle (repo convention; python get_radix_trie() returns the
/// preloaded singleton matcher — use build_default_matcher() for the data).
pub fn get_radix_trie() -> u64 {
    0x5241_4458_5452_4945 // "RADXTRIE"
}
