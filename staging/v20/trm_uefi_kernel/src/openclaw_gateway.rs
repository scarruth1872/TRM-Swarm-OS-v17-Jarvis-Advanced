//! TRM Swarm OS - openclaw_gateway (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/openclaw_gateway.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS
// (host-side OpenClaw API URL removed — the no_std kernel makes no HTTP
//  calls; zero external API claim holds on-device)
pub const INBOX_FILE: &'static str = "swarm_v2_artifacts/openclaw_inbox.json";
pub const HEARTBEAT_FILE: &'static str = "HEARTBEAT.md";
pub const INTENT_ROUTING: u64 = 0;

//  InboundMessage 
#[derive(Debug, Clone)]
pub struct InboundMessage {
    pub text: String,
    pub sender: String,
    pub channel: String,
    pub channel_id: String,
    pub timestamp: String,
    pub intent: String,
    pub target_specialist: String,
    pub metadata: Vec<(String, f64)>,
}

// ── The strip's model (swarm_v2_artifacts/native_voltage_strips/strip_openclaw_gateway.rs) ───────
//
// `strip_batch_classify_intents` scores five intent classes by counting matching keywords
// (case-insensitive substrings) and keeps the best with a STRICT `>`, so a tie goes to the earlier
// class in this order: Code (1), Question (2), Review (3), Research (4), System (0) -- and System
// wins only when it is strictly greater. Note "deploy" appears in BOTH the code and system lists,
// so a message containing only "deploy" scores 1-1 and Code takes it by order; that is the strip's
// behaviour, reproduced rather than tidied.
//
// The module carried the InboundMessage struct and none of the classification.

pub const CODE_KWS: [&str; 8] = ["implement", "build", "create", "write code", "fix bug", "refactor", "add feature", "deploy"];
pub const QUESTION_KWS: [&str; 8] = ["explain", "why", "how does", "what is", "compare", "difference", "analyze", "understand"];
pub const REVIEW_KWS: [&str; 9] = ["review", "audit", "check", "security", "vulnerability", "scan", "validate", "verify", "test"];
pub const RESEARCH_KWS: [&str; 8] = ["research", "investigate", "explore", "find", "search", "discover", "latest", "benchmark"];
pub const SYSTEM_KWS: [&str; 9] = ["status", "restart", "configure", "setup", "monitor", "health", "deploy", "scale", "shutdown"];

fn kw_score(text: &[u8], kws: &[&str]) -> usize {
    let mut n = 0usize;
    for kw in kws.iter() {
        let k = kw.as_bytes();
        if k.len() <= text.len() && text.windows(k.len()).any(|w| w.iter().zip(k.iter()).all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())) {
            n += 1;
        }
    }
    n
}

/// The strip's intent code for one message: 0 System, 1 Code, 2 Question, 3 Review, 4 Research.
pub fn classify_intent(text: &[u8]) -> u8 {
    let mut best_intent = 0u8;
    let mut best_score = 0usize;
    let mut consider = |score: usize, intent: u8, best: &mut usize, bi: &mut u8| {
        if score > *best {
            *best = score;
            *bi = intent;
        }
    };
    consider(kw_score(text, &CODE_KWS), 1, &mut best_score, &mut best_intent);
    consider(kw_score(text, &QUESTION_KWS), 2, &mut best_score, &mut best_intent);
    consider(kw_score(text, &REVIEW_KWS), 3, &mut best_score, &mut best_intent);
    consider(kw_score(text, &RESEARCH_KWS), 4, &mut best_score, &mut best_intent);
    let sys = kw_score(text, &SYSTEM_KWS);
    if sys > best_score {
        best_intent = 0;
    }
    best_intent
}

/// Batch entry point, mirroring `strip_batch_classify_intents`.
pub fn classify_intents(messages: &[&[u8]]) -> Vec<u8> {
    messages.iter().map(|m| classify_intent(m)).collect()
}

/// The pure-Python `native_classify_intent` fallback (openclaw_gateway.py:72-81): a sequential
/// if/elif over SHORT keyword lists, priority code → question → review → research → system. This is
/// a DIFFERENT algorithm from `classify_intent` above (the strip's keyword-counting over the full
/// INTENT_ROUTING lists); both live in the oracle and disagree on the same input — reproduced here,
/// not reconciled. Returns (intent code, specialist): 0 system/Architect, 1 code/Lead Developer,
/// 2 question/Reasoning Engine, 3 review/Security Auditor, 4 research/Researcher.
pub fn classify_intent_fallback(text: &str) -> (u8, &'static str) {
    let tl = text.to_lowercase();
    let has = |kws: &[&str]| kws.iter().any(|k| tl.contains(k));
    if has(&["implement", "build", "create", "write code", "fix bug"]) {
        (1, "Lead Developer")
    } else if has(&["explain", "why", "how does", "what is"]) {
        (2, "Reasoning Engine")
    } else if has(&["review", "audit", "check", "security"]) {
        (3, "Security Auditor")
    } else if has(&["research", "investigate", "explore"]) {
        (4, "Researcher")
    } else {
        (0, "Architect")
    }
}

impl InboundMessage {
    pub fn new() -> Self {
        InboundMessage {
            text: String::new(),
            sender: String::new(),
            channel: String::new(),
            channel_id: String::new(),
            timestamp: String::new(),
            intent: String::new(),
            target_specialist: String::new(),
            metadata: vec![],
        }
    }
}

//  ChannelAdapter 
#[derive(Debug, Clone)]
pub struct ChannelAdapter {
    pub channel_name: String,
}

impl ChannelAdapter {
    pub fn new() -> Self {
        ChannelAdapter {
            channel_name: String::new(),
        }
    }
}
