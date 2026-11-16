//! TRM Swarm OS — Self Healing & Security Sanitization (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/self_healing.py & strip_self_healing.rs.
//! Implements batch zero-copy security inspection for SQLi, XSS, and Path Traversal attacks,
//! plus autonomous circuit-breaker supervisor logic for crashed microkernel routines.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- SECURITY THREAT TYPE (1:1 with strip_self_healing.rs) ---
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityThreatType {
    Safe = 0,
    SqlInjection = 1,
    CrossSiteScripting = 2,
    PathTraversal = 3,
}

impl SecurityThreatType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityThreatType::Safe => "safe",
            SecurityThreatType::SqlInjection => "sql_injection",
            SecurityThreatType::CrossSiteScripting => "xss",
            SecurityThreatType::PathTraversal => "path_traversal",
        }
    }
}

// --- CASE-INSENSITIVE SUBSTRING SEARCH ---
fn contains_ci(hay: &[u8], needle: &[u8]) -> bool {
    if needle.len() > hay.len() {
        return false;
    }
    for w in hay.windows(needle.len()) {
        if w.iter().zip(needle.iter()).all(|(&a, &b)| a.to_ascii_lowercase() == b.to_ascii_lowercase()) {
            return true;
        }
    }
    false
}

// --- PURE BARE-METAL PAYLOAD SCANNER ---
pub fn scan_payload_security(payload: &[u8]) -> SecurityThreatType {
    // SQL Injection checks
    if contains_ci(payload, b"union select")
        || contains_ci(payload, b"select ")
        || contains_ci(payload, b"insert into")
        || contains_ci(payload, b"drop table")
        || contains_ci(payload, b"drop database")
        || contains_ci(payload, b"--")
    {
        return SecurityThreatType::SqlInjection;
    }

    // XSS checks
    if contains_ci(payload, b"<script")
        || contains_ci(payload, b"alert(")
        || contains_ci(payload, b"document.cookie")
    {
        return SecurityThreatType::CrossSiteScripting;
    }

    // Path Traversal checks
    if contains_ci(payload, b"../")
        || contains_ci(payload, b"..\\")
        || contains_ci(payload, b"/etc/")
        || contains_ci(payload, b"/windows/")
    {
        return SecurityThreatType::PathTraversal;
    }

    SecurityThreatType::Safe
}

// --- PROCESS SUPERVISION STATE ---
#[derive(Debug, Clone)]
pub struct ProcessSupervisionState {
    pub target_name: String,
    pub restart_count: u32,
    pub circuit_breaker_tripped: bool,
    pub last_failure_epoch_s: u64,
    pub max_restarts_allowed: u32,
}

impl ProcessSupervisionState {
    pub fn new(target: &str) -> Self {
        Self {
            target_name: target.to_string(),
            restart_count: 0,
            circuit_breaker_tripped: false,
            last_failure_epoch_s: 0,
            max_restarts_allowed: 5,
        }
    }

    pub fn record_failure(&mut self, epoch_s: u64) -> bool {
        self.last_failure_epoch_s = epoch_s;
        self.restart_count += 1;
        if self.restart_count >= self.max_restarts_allowed {
            self.circuit_breaker_tripped = true;
        }
        self.circuit_breaker_tripped
    }

    pub fn reset(&mut self) {
        self.restart_count = 0;
        self.circuit_breaker_tripped = false;
    }
}

// --- SELF HEALING ENGINE ---
pub struct SelfHealingEngine {
    supervision_records: Vec<ProcessSupervisionState>,
    pub total_scans: u64,
    pub threats_detected: u64,
}

impl SelfHealingEngine {
    pub fn new() -> Self {
        Self {
            supervision_records: Vec::new(),
            total_scans: 0,
            threats_detected: 0,
        }
    }

    pub fn scan_and_audit(&mut self, payload: &[u8]) -> SecurityThreatType {
        self.total_scans += 1;
        let threat = scan_payload_security(payload);
        if threat != SecurityThreatType::Safe {
            self.threats_detected += 1;
        }
        threat
    }

    pub fn record_target_failure(&mut self, target: &str, epoch_s: u64) -> bool {
        if let Some(proc) = self.supervision_records.iter_mut().find(|p| p.target_name == target) {
            proc.record_failure(epoch_s)
        } else {
            let mut proc = ProcessSupervisionState::new(target);
            let tripped = proc.record_failure(epoch_s);
            self.supervision_records.push(proc);
            tripped
        }
    }

    pub fn reset_circuit_breaker(&mut self, target: &str) -> bool {
        if let Some(proc) = self.supervision_records.iter_mut().find(|p| p.target_name == target) {
            proc.reset();
            true
        } else {
            false
        }
    }

    pub fn is_tripped(&self, target: &str) -> bool {
        self.supervision_records.iter()
            .find(|p| p.target_name == target)
            .map(|p| p.circuit_breaker_tripped)
            .unwrap_or(false)
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_SELF_HEALING: AtomicU64 = AtomicU64::new(0);

pub fn get_self_healing() -> u64 {
    let v = SINGLETON_SELF_HEALING.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SELF_HEALING.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}




// ── `TokenRateMonitor.check_rate` (swarm_v2/core/self_healing.py) -- the part this module lacked ──
//
// `scan_payload_security` and `contains_ci` were ALREADY here and are a faithful port of the strip's
// scanner (same six SQL markers, same three XSS, same four traversal, same precedence, same
// case-folding) -- they are kept as they are and are VERIFIED against strip_self_healing.rs rather than
// rewritten. What was missing is the rate limiter:
//
//   * a loopback session id ("127.0.0.1", "::1", "localhost", "testclient") is ALWAYS allowed;
//   * the per-session budget resets once more than 3600 s have passed since the session's timestamp;
//   * `total + token_count > max_budget` -> budget exceeded;
//   * bucket entries STRICTLY older than `now - window_size` are dropped (an entry exactly at that
//     instant is KEPT), then `len(bucket) / window_size + token_count > max_rate` -> rate exceeded;
//   * otherwise `now` is appended once per token and the total grows.
// `current_rate` is a FLOAT division, so the comparison is against a fractional rate and not a count.
// The oracle RAISES; this returns the decision so the caller can act on it.

pub const RATE_WINDOW_SIZE: f64 = 1.0;
pub const RATE_MAX_RATE: f64 = 50_000.0;
pub const RATE_MAX_BUDGET: f64 = 100_000_000.0;
pub const RATE_RESET_SECONDS: f64 = 3600.0;

#[derive(PartialEq, Debug)]
pub enum RateDecision {
    Allow,
    BudgetExceeded,
    RateExceeded,
}

pub struct RateSession {
    pub id: String,
    pub total: f64,
    pub timestamp: f64,
    pub bucket: Vec<f64>,
}

pub struct TokenRateMonitor {
    pub window_size: f64,
    pub max_rate: f64,
    pub max_budget: f64,
    pub sessions: Vec<RateSession>,
}

fn is_loopback(id: &str) -> bool {
    matches!(id, "127.0.0.1" | "::1" | "localhost" | "testclient")
}

impl TokenRateMonitor {
    pub fn new() -> Self {
        TokenRateMonitor { window_size: RATE_WINDOW_SIZE, max_rate: RATE_MAX_RATE, max_budget: RATE_MAX_BUDGET, sessions: Vec::new() }
    }

    fn slot(&mut self, id: &str, now: f64) -> usize {
        if self.sessions.iter().all(|s| s.id != id) {
            self.sessions.push(RateSession { id: id.to_string(), total: 0.0, timestamp: now, bucket: Vec::new() });
        }
        self.sessions.iter().position(|s| s.id == id).unwrap()
    }

    /// `check_rate`. Returns the decision rather than raising.
    pub fn check_rate(&mut self, session_id: &str, token_count: usize, now: f64) -> RateDecision {
        if is_loopback(session_id) {
            return RateDecision::Allow;
        }
        let i = self.slot(session_id, now);
        if now - self.sessions[i].timestamp > RATE_RESET_SECONDS {
            self.sessions[i].total = 0.0;
            self.sessions[i].timestamp = now;
        }
        if self.sessions[i].total + token_count as f64 > self.max_budget {
            return RateDecision::BudgetExceeded;
        }
        let cutoff = now - self.window_size;
        let bucket = &mut self.sessions[i].bucket;
        while !bucket.is_empty() && bucket[0] < cutoff {
            bucket.remove(0);
        }
        let current_rate = bucket.len() as f64 / self.window_size;
        if current_rate + token_count as f64 > self.max_rate {
            return RateDecision::RateExceeded;
        }
        for _ in 0..token_count {
            bucket.push(now);
        }
        self.sessions[i].total += token_count as f64;
        RateDecision::Allow
    }
}
