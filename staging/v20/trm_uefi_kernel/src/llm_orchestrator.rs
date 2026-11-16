//! TRM Swarm OS - llm_orchestrator
//! Bare-metal LLM Orchestrator engine for dynamic multi-provider cognitive routing.
//! Integrates TurboVec 16x BT-Quant native voltage strip with high-level Swarm OS routing.

#![allow(non_snake_case, dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

// --- TurboVec 16x Balanced Ternary Quantization Weights (Phase 41, Rank 198) ---
static TW: &[i8] = &[
    1, -1, -1, -1, -1, -1, 1, 0, 0, 0, 1, -1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, -1,
    1, 1, -1, -1, 1, -1, 1, -1, 1, -1, -1, 0, -1, -1, 0, 0, 0, -1, 1, -1, 0, 1,
    1, 1, 1, 0, 1, 1, 1, 1, 1, -1, 1, 0, 0, -1, 1, -1, 0, 0, 0, -1, 0, -1, -1,
    -1, 1, 1, 0, -1, 1, -1, -1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, -1, -1, 0,
    -1, 1, -1, 1, 0, -1, -1, -1, -1, 1, 0, 0, -1, 1, -1, -1, 1, 1, 0, 1, 1, 0, 1,
    0, 1, 0, 0, 1, 1, 0, -1, 0, 0, 1, 0, 1, -1, 0, -1, -1, -1, 0, 1, -1, 0, 1,
    -1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, -1, 1, 0, 0, -1, 1, 0, 1, -1, 1, -1,
    -1, -1, -1, 0, 1, 1, -1, -1, 1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1,
    -1, -1, 1, 0, 1, 0, 1, -1, 0, -1, -1, -1, 0, 0, -1, 0, 1, 0, 0, 1, 1, 1, 0,
    0, 1, 0, 1, 1, 1, -1, 0, 1, -1, -1, 1, -1, 1, -1, 1, -1, -1, -1, -1, -1, 1,
    1, -1, 0, 1, -1, -1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, -1, 1, 1, -1, 0, 1, -1,
    1, 0
];

pub static SCALE_FACTOR: f32 = 31.9956;
pub static INVOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
pub static MONAD_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;
pub static PHASE: u32 = 41;
pub static RANK: u32 = 198;

// --- Kleene K3 3-Valued Logic ---
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum K3 {
    False = 0,
    Unknown = 1,
    True = 2,
}

impl K3 {
    #[inline(always)]
    pub fn and(self, other: K3) -> K3 {
        match (self as u8).min(other as u8) {
            0 => K3::False,
            1 => K3::Unknown,
            _ => K3::True,
        }
    }

    #[inline(always)]
    pub fn or(self, other: K3) -> K3 {
        match (self as u8).max(other as u8) {
            0 => K3::False,
            1 => K3::Unknown,
            _ => K3::True,
        }
    }

    #[inline(always)]
    pub fn not(self) -> K3 {
        match self {
            K3::False => K3::True,
            K3::Unknown => K3::Unknown,
            K3::True => K3::False,
        }
    }
}

// --- Ternary Dot Product ---
#[inline(always)]
pub fn ternary_dot(inputs: &[f32]) -> f32 {
    let n = inputs.len().min(TW.len());
    let mut acc = 0.0f32;
    let mut i = 0;
    while i < n {
        acc += inputs[i] * (TW[i] as f32) * (SCALE_FACTOR / 16.0);
        i += 1;
    }
    acc
}

// --- Native C ABI Exports ---
#[no_mangle]
pub extern "C" fn llm_orchestrator_execute(i: *const f32, n: usize, o: *mut f32) -> u32 {
    if i.is_null() || o.is_null() || n == 0 {
        return 0xDEAD_BEEF;
    }
    INVOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
    let s = unsafe { core::slice::from_raw_parts(i, n) };
    let r = ternary_dot(s);
    let k = if r.abs() < 1e-3 {
        K3::Unknown
    } else if r > 0.0 {
        K3::True
    } else {
        K3::False
    };
    let _ = k.and(K3::True);
    unsafe { *o = r };
    0x0000_0001
}

#[no_mangle]
pub extern "C" fn llm_orchestrator_monad_commit(bid: u64) -> u64 {
    let inv = INVOCATION_COUNT.load(Ordering::Relaxed);
    MONAD_ANCHOR ^ bid ^ inv
}

#[no_mangle]
pub extern "C" fn llm_orchestrator_phase_info(op: *mut u32, or_: *mut u32, oa: *mut u64) {
    if !op.is_null() {
        unsafe { *op = PHASE };
    }
    if !or_.is_null() {
        unsafe { *or_ = RANK };
    }
    if !oa.is_null() {
        unsafe { *oa = MONAD_ANCHOR };
    }
}

// --- High-Level LLM Orchestration Architecture ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelTier {
    Executive,
    Reasoning,
    Fast,
    LocalTRM,
}

#[derive(Clone, Debug)]
pub struct RoutingDecision {
    pub target_model: &'static str,
    pub tier: ModelTier,
    pub complexity_score: f32,
    pub rationale: &'static str,
    pub verification: K3,
}

pub struct LLMOrchestrator {
    pub executive_model: &'static str,
    pub reasoning_model: &'static str,
    pub fast_model: &'static str,
    pub local_model: &'static str,
    pub complexity_threshold: f32,
    pub total_routed: u64,
    pub total_reasoning: u64,
    pub total_executive: u64,
}

impl LLMOrchestrator {
    pub const fn new() -> Self {
        Self {
            executive_model: "gemma4:e2b",
            reasoning_model: "deepseek-r1:8b",
            fast_model: "gemma2:2b",
            local_model: "local-trm",
            complexity_threshold: 0.70,
            total_routed: 0,
            total_reasoning: 0,
            total_executive: 0,
        }
    }

    /// Determines the optimal model routing given task requirements and complexity score.
    pub fn determine_routing(&mut self, task: &str, complexity_score: f32) -> RoutingDecision {
        self.total_routed += 1;

        // 1. Explicit Deep Reasoning Directive
        if contains_substr(task, "[DEEP_REASONING]") || contains_substr(task, "[REASONING]") {
            self.total_reasoning += 1;
            return RoutingDecision {
                target_model: self.reasoning_model,
                tier: ModelTier::Reasoning,
                complexity_score,
                rationale: "Explicit deep reasoning override tag detected",
                verification: K3::True,
            };
        }

        // 2. High Complexity Threshold Evaluation
        if complexity_score >= self.complexity_threshold {
            self.total_reasoning += 1;
            return RoutingDecision {
                target_model: self.reasoning_model,
                tier: ModelTier::Reasoning,
                complexity_score,
                rationale: "Complexity score exceeded 0.70 threshold",
                verification: K3::True,
            };
        }

        // 3. Keyword-Based Architectural & Formal Proof Heuristics
        let high_keywords: &[&str] = &[
            "architecture",
            "refactor",
            "security audit",
            "optimize",
            "proof",
            "formal",
            "monad",
            "consensus",
            "cryptography",
        ];

        for kw in high_keywords {
            if contains_substr_ci(task, kw) {
                self.total_reasoning += 1;
                return RoutingDecision {
                    target_model: self.reasoning_model,
                    tier: ModelTier::Reasoning,
                    complexity_score: complexity_score.max(0.75),
                    rationale: "High-reasoning domain keyword detected in task",
                    verification: K3::True,
                };
            }
        }

        // 4. Default: Agile Executive Model
        self.total_executive += 1;
        RoutingDecision {
            target_model: self.executive_model,
            tier: ModelTier::Executive,
            complexity_score,
            rationale: "Task suitable for agile executive model",
            verification: K3::True,
        }
    }

    /// Synthesizes disparate agent perspectives into a structured Swarm Intelligence context.
    pub fn synthesize_context(&self, agent_responses: &[(&str, &str)]) -> String {
        let mut unified = String::from("# Unified Swarm Intelligence Context\n\n");
        for (agent, content) in agent_responses {
            unified.push_str("### Perspective: ");
            unified.push_str(agent);
            unified.push_str("\n");
            unified.push_str(content);
            unified.push_str("\n\n");
        }
        unified
    }

    /// Returns telemetry metrics (total, reasoning, executive).
    pub fn stats(&self) -> (u64, u64, u64) {
        (self.total_routed, self.total_reasoning, self.total_executive)
    }
}

// --- Case-Insensitive Substring Helper ---
fn contains_substr(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}

fn contains_substr_ci(haystack: &str, needle: &str) -> bool {
    let h_bytes = haystack.as_bytes();
    let n_bytes = needle.as_bytes();
    if n_bytes.is_empty() {
        return true;
    }
    if h_bytes.len() < n_bytes.len() {
        return false;
    }

    let limit = h_bytes.len() - n_bytes.len();
    let mut i = 0;
    while i <= limit {
        let mut matched = true;
        let mut j = 0;
        while j < n_bytes.len() {
            let b_h = h_bytes[i + j].to_ascii_lowercase();
            let b_n = n_bytes[j].to_ascii_lowercase();
            if b_h != b_n {
                matched = false;
                break;
            }
            j += 1;
        }
        if matched {
            return true;
        }
        i += 1;
    }
    false
}

// --- Global Singleton Interface ---
static SINGLETON_ORCHESTRATOR: AtomicU64 = AtomicU64::new(0);

pub fn get_orchestrator() -> u64 {
    let v = SINGLETON_ORCHESTRATOR.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_ORCHESTRATOR.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}
