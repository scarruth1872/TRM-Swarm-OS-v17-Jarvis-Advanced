//! TRM Swarm OS - base_agent (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/base_agent.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  AgentPersona 
#[derive(Debug, Clone)]
pub struct AgentPersona {
    pub name: String,
    pub role: String,
    pub background: String,
    pub specialties: Vec<String>,
    pub avatar_color: String,
    pub model: String,
    pub department: String,
    pub llm_backend: String,
    pub core_axioms: Vec<String>,
    pub forbidden_lexicon: Vec<String>,
}

impl AgentPersona {
    pub fn new() -> Self {
        AgentPersona {
            name: String::new(),
            role: String::new(),
            background: String::new(),
            specialties: vec![],
            avatar_color: String::new(),
            model: String::new(),
            department: String::new(),
            llm_backend: String::new(),
            core_axioms: vec![],
            forbidden_lexicon: vec![],
        }
    }
}

//  Message 
#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub receiver: String,
    pub content: String,
    pub task_id: String,
    pub metadata: Vec<(String, f64)>,
}

impl Message {
    pub fn new() -> Self {
        Message {
            sender: String::new(),
            receiver: String::new(),
            content: String::new(),
            task_id: String::new(),
            metadata: vec![],
        }
    }
}

// ═══ PORTED DETERMINISTIC LAYER ══════════════════════════════════════════════
// 1:1 from swarm_v2/core/base_agent.py. Python reached score_tasks through
// strip_base_agent.dll; in-kernel the definition IS the implementation
// (score_i = urgency_i * relevance_i / (1 + load_i)) — the strip evaluates the
// same expression in f32 and the python fallback is identical.

/// 1:1 python `_native_score_tasks`: score_i = u_i * r_i / (1.0 + l_i).
/// zip semantics — the result length is the SHORTEST input (python `zip`
/// truncates); f32 to match the strip's dtype (`_np.float32`).
/// MEASURED 2026-09-10 (N=200,000, this host): strip 0.12 ms vs python 13.81 ms
/// = 117.7x, so the python docstring's "84x" understates it here; and the
/// docstring's "K3: +1 TRUE (diff=0.00)" does NOT hold — the strip's f32 math
/// differs from the python f64 fallback by max rel 1.58e-07 (f32 epsilon).
pub fn score_tasks(urgency: &[f32], relevance: &[f32], load: &[f32]) -> Vec<f32> {
    let n = urgency.len().min(relevance.len()).min(load.len());
    if n == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(urgency[i] * relevance[i] / (1.0 + load[i]));
    }
    out
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// ASCII-lowercase into bytes. Python `str.lower()` is unicode-aware, but every
/// pattern below is ASCII — a non-ASCII byte simply never matches.
fn ascii_lower(s: &str) -> Vec<u8> {
    s.bytes().map(|b| b.to_ascii_lowercase()).collect()
}

/// python `re.search(r'\b' + re.escape(word) + r'\b', hay)`.
fn has_word(hay: &[u8], word: &str) -> bool {
    let w = word.as_bytes();
    if w.is_empty() || hay.len() < w.len() {
        return false;
    }
    let mut i = 0usize;
    while i + w.len() <= hay.len() {
        if &hay[i..i + w.len()] == w {
            let cur = is_word_byte(hay[i]);
            let start_ok = if i == 0 { cur } else { is_word_byte(hay[i - 1]) != cur };
            let e = i + w.len();
            let prev = is_word_byte(hay[e - 1]);
            let end_ok = if e == hay.len() { prev } else { prev != is_word_byte(hay[e]) };
            if start_ok && end_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// python `needle in haystack` (plain substring, no boundaries).
fn contains_lit(hay: &[u8], lit: &str) -> bool {
    let n = lit.as_bytes();
    !n.is_empty() && hay.len() >= n.len() && hay.windows(n.len()).any(|w| w == n)
}

/// python `_is_greeting`: word-boundary patterns.
const GREETING_PATTERNS: [&str; 10] = [
    "hello", "hi", "hey", "hiya", "howdy", "greetings",
    "what's up", "whats up", "wassup", "sup",
];
/// python: plain substring test against the FIRST 15 chars (`short_check`).
const GREETING_QUESTIONS: [&str; 4] = ["who are you", "who r u", "what is your name", "tell me ab"];
/// python `task_keywords` (39 entries, order preserved).
const TASK_KEYWORDS: [&str; 39] = [
    "write", "create", "build", "implement", "fix", "bug", "error", "help", "search", "find",
    "make", "generate", "code", "file", "read", "list", "show", "get", "do", "run", "test",
    "craft", "proposal", "response", "design", "analyze", "explain", "compare", "plan", "deploy",
    "configure", "setup", "install", "debug", "review", "audit", "scan", "research", "investigate",
];

/// python `BaseAgent._is_greeting`. The python also computes `first_word` and
/// never reads it — that dead local is deliberately not ported.
pub fn is_greeting(task: &str) -> bool {
    let lower = ascii_lower(task.trim());
    let is_g = GREETING_PATTERNS.iter().any(|p| has_word(&lower, p));
    let short = &lower[..lower.len().min(15)];
    let is_q = GREETING_QUESTIONS.iter().any(|q| contains_lit(short, q));
    let has_task = TASK_KEYWORDS.iter().any(|k| has_word(&lower, k));
    (is_g || is_q) && !has_task
}

/// python `_generate_greeting` — the 12-entry persona table verbatim, then the
/// python fallback (`specialties[:2]` joined with ", ").
pub fn generate_greeting(persona: &AgentPersona) -> String {
    const TABLE: [(&str, &str); 12] = [
        ("Archi", "Hello! I'm Archi, your Architect. I specialize in system design, scalable architecture, and infrastructure planning. How can I help design your next system?"),
        ("Devo", "Hey! I'm Devo, Lead Developer. I love building clean, elegant code. What should we create today?"),
        ("Seeker", "Greetings! I'm Seeker, your Researcher. I excel at finding information and synthesizing knowledge. What would you like to explore?"),
        ("Logic", "Hello. I'm Logic, specialized in complex reasoning, algorithms, and mathematical deductions. What problem shall we solve?"),
        ("Shield", "Hello. I'm Shield, your Security Auditor. I identify vulnerabilities and ensure robust security. What needs auditing?"),
        ("Flow", "Hey there! I'm Flow, DevOps Engineer. I handle CI/CD, containers, and cloud deployments. Ready to deploy something?"),
        ("Vision", "Hi! I'm Vision, ui_ux Designer. I craft beautiful, accessible interfaces. What should we design?"),
        ("Verify", "Hello! I'm Verify, QA Engineer. I test, find bugs, and ensure quality. What should we verify?"),
        ("Orchestra", "Greetings! I'm Orchestra, Swarm Manager. I coordinate agent teams and optimize task delegation. What needs orchestrating?"),
        ("Scribe", "Hello! I'm Scribe, Technical Writer. I create clear documentation and tutorials. What should we document?"),
        ("Bridge", "Hey! I'm Bridge, Integration Specialist. I connect APIs, protocols, and tools. What integrations do you need?"),
        ("Pulse", "Hello! I'm Pulse, Data Analyst. I process data and generate insights. What data should we analyze?"),
    ];
    for (name, greeting) in TABLE.iter() {
        if *name == persona.name.as_str() {
            return String::from(*greeting);
        }
    }
    let n = persona.specialties.len().min(2);
    alloc::format!(
        "Hello! I'm {}, a {}. I specialize in {}. How can I help?",
        persona.name,
        persona.role,
        persona.specialties[..n].join(", ")
    )
}

/// python skills are objects; the portable projection is the name, the
/// description, and the method names the object exposes (`hasattr` on the
/// python side). NOTE: python falls back to `s.__class__.__name__` when
/// `skill_name` is missing — a static port cannot reflect a type name, so
/// `skill_name` is a constructor argument here.
#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub skill_name: String,
    pub description: String,
    pub methods: Vec<String>,
}

/// python `get_subagents()` row: id + persona fields + `get_skill_names()`.
#[derive(Debug, Clone)]
pub struct SubagentRecord {
    pub id: String,
    pub name: String,
    pub role: String,
    pub background: String,
    pub skills: Vec<String>,
}

/// python `BaseAgent` — the deterministic state the portable layer owns.
/// // MANUAL (host-owned, not ported): process_task / _llm_generate (LLM I/O),
/// initialize_qstate (qic._state import), autonomous_code_test_fix (code+test
/// cycles), spawn_subagent (microkernel syscall), get_memory_stats
/// (AgentMemory), _check_python_syntax (py_compile).
#[derive(Debug, Clone)]
pub struct BaseAgent {
    pub agent_id: String,
    pub persona: AgentPersona,
    pub skills: Vec<SkillEntry>,
    pub subagents: Vec<(String, SubagentRecord)>,
    pub nodal_logs: Vec<String>,
    pub task_history: Vec<(String, String)>,
    pub active_task: Option<String>,
}

impl BaseAgent {
    pub fn new(agent_id: String, persona: AgentPersona, skills: Vec<SkillEntry>) -> Self {
        BaseAgent {
            agent_id,
            persona,
            skills,
            subagents: vec![],
            nodal_logs: vec![],
            task_history: vec![],
            active_task: None,
        }
    }

    /// python `log_nodal_activity`: the caller supplies the clock (the python
    /// `datetime.now().strftime("%H:%M:%S")` is host-owned), non-ASCII is
    /// stripped, and the log is capped at 50 by dropping the oldest.
    pub fn log_nodal_activity(&mut self, timestamp: &str, message: &str) {
        let safe: String = message.chars().filter(|c| (*c as u32) < 128).collect();
        let entry = alloc::format!("[{}] {}", timestamp, safe);
        self.nodal_logs.push(entry);
        if self.nodal_logs.len() > 50 {
            self.nodal_logs.remove(0);
        }
    }

    pub fn skill_names(&self) -> Vec<String> {
        self.skills.iter().map(|s| s.skill_name.clone()).collect()
    }

    pub fn skill_descriptions(&self) -> Vec<(String, String)> {
        self.skills
            .iter()
            .map(|s| (s.skill_name.clone(), s.description.clone()))
            .collect()
    }

    /// python `_has_skill`: `any(hasattr(s, method_name) for s in skills)`.
    pub fn has_skill(&self, method_name: &str) -> bool {
        self.skills
            .iter()
            .any(|s| s.methods.iter().any(|m| m == method_name))
    }

    /// python `_get_skill`: the first skill exposing the method, else None.
    pub fn get_skill(&self, method_name: &str) -> Option<&SkillEntry> {
        self.skills
            .iter()
            .find(|s| s.methods.iter().any(|m| m == method_name))
    }

    /// python `spawn_subagent`: `f"{persona.name}_Sub{len(self.subagents)+1}"`.
    pub fn subagent_name(&self) -> String {
        alloc::format!("{}_Sub{}", self.persona.name, self.subagents.len() + 1)
    }

    /// python `spawn_subagent` persona construction, minus the syscall:
    /// `role`, `specialties[:2]`, color #ffaa00, and
    /// `f"Microkernel SubAgent spawned by {name}: {task[:60]}"`.
    pub fn subagent_persona(&self, role: &str, task: &str) -> AgentPersona {
        let mut p = AgentPersona::new();
        p.name = self.subagent_name();
        p.role = String::from(role);
        p.background = alloc::format!(
            "Microkernel SubAgent spawned by {}: {}",
            self.persona.name,
            char_prefix(task, 60)
        );
        p.specialties = self.persona.specialties.iter().take(2).cloned().collect();
        p.avatar_color = String::from("#ffaa00");
        p
    }

    /// python `sub = BaseAgent(sub_persona, skills=list(self.skills))` then
    /// `self.subagents[sub.agent_id] = sub`. The record carries the PARENT's
    /// skill list, copied at spawn time — python passes `list(self.skills)`.
    /// // MANUAL: the microkernel syscall that yields `subagent_id` (host-owned).
    pub fn spawn_record(&mut self, subagent_id: &str, role: &str, task: &str) -> SubagentRecord {
        let persona = self.subagent_persona(role, task);
        let record = SubagentRecord {
            id: String::from(subagent_id),
            name: persona.name.clone(),
            role: persona.role.clone(),
            background: persona.background.clone(),
            skills: self.skill_names(),
        };
        self.subagents.push((record.id.clone(), record.clone()));
        record
    }

    /// python `get_subagents()` — one row per spawned subagent.
    pub fn get_subagents(&self) -> Vec<SubagentRecord> {
        self.subagents.iter().map(|(_, r)| r.clone()).collect()
    }
}

/// python `_contribute_to_global_memory` summary line:
/// `f"Task: {task[:100]} | Result: {response[:200]}"`.
pub fn global_memory_summary(task: &str, response: &str) -> String {
    alloc::format!(
        "Task: {} | Result: {}",
        char_prefix(task, 100),
        char_prefix(response, 200)
    )
}

/// python slicing counts code points, not bytes.
fn char_prefix(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

/// python `__init__` PPM4 identity defaults, used when the persona supplies none.
pub fn default_core_axioms(agent_name: &str) -> Vec<String> {
    alloc::vec![
        alloc::format!("I am a Swarm OS node ({}).", agent_name),
        String::from("I do not apologize for my operational constraints."),
        String::from("I execute tasks deterministically without conversational padding."),
    ]
}

pub const DEFAULT_FORBIDDEN_LEXICON: [&str; 5] = 
    ["helpful", "assistant", "I'm here to", "how can I", "As an AI"];

// ── `_extract_filename` (swarm_v2/core/base_agent.py), the one rule this module was missing ──────
//
// Two ordered patterns:
//   1. a VERB (write|create|save|implement|build|generate|make), whitespace, optional "file" +
//      whitespace, then a path token that must end with `.<word chars>`. The verb is UNANCHORED (the
//      oracle has no `\b` around it), and the middle class includes `.`, `/` and `\`, so the token is
//      everything from the first name-char up to the LAST dot that is followed by word characters.
//   2. otherwise a WORD-BOUNDARY token whose extension is in the oracle's fixed list, with a word
//      boundary after it -- which is why "x.pys" does not match ".py".
// Backslashes are normalised to forward slashes; no match returns the fallback.
//
// `score_tasks` and `is_greeting` were already here, faithfully, and are kept as they are -- the port
// below adds only what was absent, and `is_word_byte` is reused rather than duplicated. Note that
// `score_tasks` uses python `zip` semantics (the SHORTEST input), which an earlier draft of this port
// got wrong by indexing the urgency length.

const VERBS: &[&str] = &["write", "create", "save", "implement", "build", "generate", "make"];

fn starts_with_ci(h: &[u8], at: usize, needle: &[u8]) -> bool {
    at + needle.len() <= h.len()
        && h[at..at + needle.len()].iter().zip(needle.iter()).all(|(a, b)| a.to_ascii_lowercase() == *b)
}

fn in_name_head(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-'
}
fn in_path_middle(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'/' || c == b'\\' || c == b'.'
}
fn in_path_strict(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'/' || c == b'\\'
}

fn match_verb_pattern(task: &str, known_exts: &[&str]) -> Option<String> {
    let _ = known_exts;
    let b = task.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        let mut p = i;
        let mut hit = false;
        for v in VERBS.iter() {
            if starts_with_ci(b, i, v.as_bytes()) {
                p = i + v.len();
                hit = true;
                break;
            }
        }
        if hit {
            let ws0 = p;
            while p < b.len() && b[p].is_ascii_whitespace() {
                p += 1;
            }
            if p > ws0 {
                if starts_with_ci(b, p, b"file") {
                    let mut q = p + 4;
                    let ws1 = q;
                    while q < b.len() && b[q].is_ascii_whitespace() {
                        q += 1;
                    }
                    if q > ws1 {
                        p = q;
                    }
                }
                if p < b.len() && in_name_head(b[p]) {
                    let start = p;
                    while p < b.len() && in_path_middle(b[p]) {
                        p += 1;
                    }
                    let run_end = p;
                    let mut w = run_end;
                    while w > start && !is_word_byte(b[w - 1]) {
                        w -= 1;
                    }
                    if w > start {
                        let mut dot: Option<usize> = None;
                        let mut k = w;
                        while k > start {
                            if b[k - 1] == b'.' {
                                dot = Some(k - 1);
                                break;
                            }
                            k -= 1;
                        }
                        if let Some(d) = dot {
                            if w > d + 1 {
                                return Some(task[start..run_end].replace('\\', "/"));
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }
    None
}

fn match_extension_pattern(task: &str, exts: &[&str]) -> Option<String> {
    let b = task.as_bytes();
    let mut i = 0usize;
    while i < b.len() {
        let boundary = i == 0 || !is_word_byte(b[i - 1]);
        if boundary && in_name_head(b[i]) {
            let start = i;
            let mut p = i;
            while p < b.len() && in_path_strict(b[p]) {
                p += 1;
            }
            // The dot that ends the run sits AT `p`, so the scan must include it -- an earlier version
            // scanned `while scan < p` and therefore never tested the only dot that could match, which
            // sent every path-shaped task to the fallback.
            let mut scan = start;
            while scan <= p && scan < b.len() {
                if b[scan] == b'.' {
                    for e in exts.iter() {
                        if starts_with_ci(b, scan + 1, e.as_bytes()) {
                            let end = scan + 1 + e.len();
                            if end >= b.len() || !is_word_byte(b[end]) {
                                return Some(task[start..end].replace('\\', "/"));
                            }
                        }
                    }
                }
                scan += 1;
            }
        }
        i += 1;
    }
    None
}

/// The oracle's fixed extension list, in its own order.
pub const KNOWN_EXTENSIONS: &[&str] = &["py", "js", "ts", "jsx", "tsx", "go", "rs", "java", "cpp", "c", "h",
                                        "md", "txt", "yaml", "yml", "json", "sh", "html", "css"];

/// `_extract_filename`.
pub fn extract_filename(task: &str, fallback: &str) -> String {
    if let Some(m) = match_verb_pattern(task, KNOWN_EXTENSIONS) {
        return m;
    }
    if let Some(m) = match_extension_pattern(task, KNOWN_EXTENSIONS) {
        return m;
    }
    String::from(fallback)
}
