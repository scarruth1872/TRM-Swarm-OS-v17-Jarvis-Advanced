//! TRM Swarm OS - neural_wall (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/neural_wall.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const PROMPT_INJECTION_PATTERNS: u64 = 0;

//  ThreatDetection 
#[derive(Debug, Clone)]
pub struct ThreatDetection {
    pub is_malicious: bool,
    pub threat_level: u64,
    pub threat_type: String,
    pub confidence: f64,
    pub description: String,
    pub blocked_patterns: Vec<String>,
    pub raw_text: String,
    pub timestamp: String,
}

// ── The strip's model (swarm_v2_artifacts/native_voltage_strips/strip_neural_wall.rs) ────────────
//
// `strip_scan_security_packets` classifies each packet to a threat LEVEL 0-4 by case-insensitive
// substring, as an if/else-if chain so the higher level wins:
//     level 4: "bypass" | "sudo" | "do anything" | "criminal" | "hacker"
//     level 3: else "system prompt" | "debug mode"
//     level 2: else "ignore all"
//     level 0: otherwise
// The module carried the ThreatDetection struct and none of the classification.

/// The strip's keyword table, in its own precedence order: (level, keywords).
pub const THREAT_LEVELS: [(u8, &[&str]); 3] = [
    (4, &["bypass", "sudo", "do anything", "criminal", "hacker"]),
    (3, &["system prompt", "debug mode"]),
    (2, &["ignore all"]),
];

/// Case-insensitive substring test, as the strip's inline `contains_ignore_case` (both sides folded).
pub fn contains_ignore_case(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| {
        w.iter().zip(needle.iter()).all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
    })
}

/// The strip's classification: the first level whose keyword list matches, else 0.
pub fn scan_threat_level(text: &[u8]) -> u8 {
    for (level, kws) in THREAT_LEVELS.iter() {
        for kw in kws.iter() {
            if contains_ignore_case(text, kw.as_bytes()) {
                return *level;
            }
        }
    }
    0
}

/// Batch entry point, mirroring `strip_scan_security_packets` (which returns 1 for K3 TRUE).
pub fn scan_security_packets(packets: &[&[u8]]) -> Vec<u8> {
    packets.iter().map(|p| scan_threat_level(p)).collect()
}

impl ThreatDetection {
    pub fn new() -> Self {
        ThreatDetection {
            is_malicious: false,
            threat_level: 0,
            threat_type: String::new(),
            confidence: 0.0,
            description: String::new(),
            blocked_patterns: vec![],
            raw_text: String::new(),
            timestamp: String::new(),
        }
    }
}

//  get_neural_wall() 
static SINGLETON_NEURAL_WALL: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_neural_wall() -> u64 {
    // Singleton handle (lazy-init pattern from neural_wall.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_NEURAL_WALL.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_NEURAL_WALL.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
