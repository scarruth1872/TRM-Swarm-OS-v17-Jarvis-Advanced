//! TRM Swarm OS — Radix-Trie Deterministic Fast-Path Matcher (real no_std port).
//! Ported from swarm_v2/core/radix_trie_matcher.py (Stage 1 Fast-Path Router).
//!
//! The python trie maps registered patterns to native handler callbacks; this
//! no_std port has no function callbacks, so terminal nodes store the matched
//! PATTERN STRING (and its derived endpoint, python `pattern.replace(" ", "_")`)
//! and matching returns those strings instead of `(handler, endpoint)`.
//!
//! Children are stored as `Vec<(u8, TrieNode)>` with a linear scan (alphabet is
//! ~27 ascii chars — matches the crate's no-HashMap constraint). Register /
//! match / match_prefix semantics are 1:1 with python, including `.lower()`
//! on both register and match paths.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// One trie node — python `TrieNode(children, handler, endpoint)` with the
/// handler slot replaced by the registered pattern string.
#[derive(Default)]
pub struct TrieNode {
    children: Vec<(u8, TrieNode)>,
    pattern: Option<String>,
    endpoint: Option<String>,
}

impl TrieNode {
    pub fn new() -> Self {
        TrieNode::default()
    }

    fn find_child(&self, c: u8) -> Option<&TrieNode> {
        self.children.iter().find(|(k, _)| *k == c).map(|(_, n)| n)
    }

    fn find_child_mut(&mut self, c: u8) -> Option<&mut TrieNode> {
        self.children
            .iter_mut()
            .find(|(k, _)| *k == c)
            .map(|(_, n)| n)
    }

    fn child_or_insert(&mut self, c: u8) -> &mut TrieNode {
        if !self.children.iter().any(|(k, _)| *k == c) {
            self.children.push((c, TrieNode::new()));
        }
        self.find_child_mut(c).expect("child was just inserted")
    }
}

/// Result of a successful match — replaces python's `(handler, endpoint)`
/// tuple: `pattern` stands in for the handler, `endpoint` is 1:1, and
/// `match_len` mirrors python's `match_prefix` third element.
#[derive(Clone, Debug, PartialEq)]
pub struct TrieMatch {
    /// The registered pattern that matched (deepest match).
    pub pattern: String,
    /// Registered endpoint (python default: pattern with spaces -> '_').
    pub endpoint: String,
    /// Number of input characters consumed by the longest match.
    pub match_len: usize,
}

/// RadixTrieMatcher — python class of the same name.
pub struct RadixTrieMatcher {
    root: TrieNode,
    /// python `self.registered`.
    pub registered: usize,
}

impl Default for RadixTrieMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RadixTrieMatcher {
    pub fn new() -> Self {
        RadixTrieMatcher {
            root: TrieNode::new(),
            registered: 0,
        }
    }

    /// python `register(pattern, handler, endpoint=None)`: walks (or creates)
    /// the lowercased character chain and stores pattern + endpoint at the
    /// terminal node. No handler callback exists in the no_std port — callers
    /// pass only the pattern and optional endpoint.
    pub fn register(&mut self, pattern: &str, endpoint: Option<&str>) {
        let mut node = &mut self.root;
        for c in pattern.bytes() {
            node = node.child_or_insert(c.to_ascii_lowercase());
        }
        node.pattern = Some(pattern.to_string());
        node.endpoint = Some(match endpoint {
            Some(e) => e.to_string(),
            None => pattern.replace(' ', "_"),
        });
        self.registered += 1;
    }

    /// python `match_prefix(text)`: walk the lowercased input, remembering the
    /// deepest node holding a registered pattern. Returns
    /// (pattern, endpoint, match_len) or None.
    pub fn match_prefix(&self, text: &str) -> Option<TrieMatch> {
        let mut node = &self.root;
        let mut last: Option<TrieMatch> = None;
        for (i, c) in text.bytes().enumerate() {
            node = match node.find_child(c.to_ascii_lowercase()) {
                Some(n) => n,
                None => break,
            };
            if let (Some(pattern), Some(endpoint)) = (&node.pattern, &node.endpoint) {
                last = Some(TrieMatch {
                    pattern: pattern.clone(),
                    endpoint: endpoint.clone(),
                    match_len: i + 1,
                });
            }
        }
        last
    }

    /// python `match(text)`: longest-prefix match first (partial queries such
    /// as 'literature search on tau…'); exact-match fallback mirrors python's
    /// second walk over the fully-consumed lowercased text.
    pub fn match_text(&self, text: &str) -> Option<TrieMatch> {
        if let Some(m) = self.match_prefix(text) {
            return Some(m);
        }
        // Exact fallback (python `match` lines 96-104): only succeeds when the
        // entire text is a registered terminal pattern.
        let mut node = &self.root;
        for c in text.bytes() {
            node = match node.find_child(c.to_ascii_lowercase()) {
                Some(n) => n,
                None => return None,
            };
        }
        match (&node.pattern, &node.endpoint) {
            (Some(pattern), Some(endpoint)) => Some(TrieMatch {
                pattern: pattern.clone(),
                endpoint: endpoint.clone(),
                match_len: text.len(),
            }),
            _ => None,
        }
    }
}

/// Preloaded pattern keys, 1:1 in order from `DETERMINISTIC_HANDLERS` in
/// radix_trie_matcher.py. The python dict values were handler lambdas; this
/// port registers only the keys (endpoints derived as `key.replace(' ', '_')`,
/// exactly as python's `get_radix_trie` did).
pub const DETERMINISTIC_PATTERNS: &[&str] = &[
    // Core system
    "system status",
    "health check",
    "memory query",
    "agent spawn",
    "telemetry",
    "healing",
    "research",
    "test",
    "state update",
    // Short aliases
    "status",
    "health",
    "memory",
    "spawn",
    "metrics",
    "repair",
    "find",
    "verify",
    "update",
    // Agent-specific
    "security audit",
    "ddr scan",
    "shield scan",
    "theorem prove",
    "k3 check",
    "logic prove",
    "code compile",
    "hot patch",
    "devo build",
    "search query",
    "seeker find",
    "literature search",
    "audit",
    "fve verify",
    "ast check",
    "blueprint",
    "archi design",
    "synthesize architecture",
    "connect mesh",
    "flow route",
    "service mesh",
    "fetch data",
    "bridge fetch",
    "http request",
    "orchestrate",
    "scheduler status",
    "task queue",
    "observe",
    "event log",
    "telemetry stream",
    "cosmos bridge",
    "cosmos status",
    "vla bridge",
    "hvbe telemetry",
    "voltage binder",
    "hardware status",
    "lora status",
    "adapter list",
    "lora adapter",
    // Jarvis modules
    "jarvis chat",
    "chat module",
    "jarvis skills",
    "skills module",
    "jarvis memory",
    "memory system",
    "jarvis swarm",
    "swarm integration",
    // Gap-fix verification
    "persistent memory",
    "write ahead log",
    "checkpoint status",
    "calibration status",
    // Biomimetic lab
    "biomimetic",
    "alphafold",
    "protein folding",
    // Hermes Agent commands
    "tool routing",
    "desktop automation",
    "skill synthesis",
    "hello hermes",
    "hermes status",
    // Dashboard natural-language queries
    "what is your role",
    "what can you do",
    "how are you",
    "help me",
    "can you",
    // Greetings
    "hello",
    "hi",
    "hey",
    "good morning",
    "good afternoon",
    "good evening",
];

/// Count of preloaded deterministic patterns (python dict length).
pub const DETERMINISTIC_PATTERN_COUNT: usize = DETERMINISTIC_PATTERNS.len();

/// Singleton — python `get_radix_trie()` builds the matcher once and registers
/// every DETERMINISTIC_HANDLERS key with its derived endpoint. Uses the
/// crate's established `static mut Option` + unsafe getter idiom (see
/// agent_runtime::get_runtime): single-threaded UEFI payload, initialized
/// once before any use.
static mut RADIX_TRIE: Option<RadixTrieMatcher> = None;

pub fn get_radix_trie() -> &'static RadixTrieMatcher {
    // SAFETY: single-threaded UEFI payload; initialized once before any use
    // (crate convention, identical to agent_runtime::get_runtime).
    unsafe {
        if RADIX_TRIE.is_none() {
            let mut trie = RadixTrieMatcher::new();
            for pattern in DETERMINISTIC_PATTERNS {
                trie.register(pattern, None);
            }
            RADIX_TRIE = Some(trie);
        }
        RADIX_TRIE.as_ref().unwrap()
    }
}
