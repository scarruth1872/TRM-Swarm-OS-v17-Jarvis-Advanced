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
