//! TRM Swarm OS — Sentinel Middleware (no_std port).
//! 1:1 core semantics from swarm_v2/core/sentinel.py — Catalyst.
//! Request gate: rate-limit + allowlist/blocklist check + verdict.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Sentinel verdicts.
pub const VERDICT_ALLOW: u64 = 0;
pub const VERDICT_BLOCKED: u64 = 1;
pub const VERDICT_RATE_LIMITED: u64 = 2;

/// Sentinel request gate.
#[derive(Debug, Clone)]
pub struct Sentinel {
    pub allowlist: Vec<String>,
    pub blocklist: Vec<String>,
    pub max_per_window: u64,
    pub window_requests: u64,
    pub blocked_count: u64,
}

impl Sentinel {
    pub fn new(max_per_window: u64) -> Self {
        Sentinel {
            allowlist: Vec::new(),
            blocklist: Vec::new(),
            max_per_window,
            window_requests: 0,
            blocked_count: 0,
        }
    }

    pub fn add_allow(&mut self, entry: &str) {
        self.allowlist.push(entry.into());
    }

    pub fn add_block(&mut self, entry: &str) {
        self.blocklist.push(entry.into());
    }

    /// Gate a request: (verdict, reason).
    pub fn check(&mut self, source: &str) -> (u64, String) {
        // blocklist first
        if self.blocklist.iter().any(|b| source.contains(b.as_str())) {
            self.blocked_count += 1;
            return (VERDICT_BLOCKED, String::from("blocklisted source"));
        }
        // allowlist: if non-empty, must match
        if !self.allowlist.is_empty() && !self.allowlist.iter().any(|a| source.contains(a.as_str())) {
            self.blocked_count += 1;
            return (VERDICT_BLOCKED, String::from("not allowlisted"));
        }
        // rate limit
        if self.window_requests >= self.max_per_window {
            self.blocked_count += 1;
            return (VERDICT_RATE_LIMITED, String::from("rate limit exceeded"));
        }
        self.window_requests += 1;
        (VERDICT_ALLOW, String::from("allowed"))
    }
}

/// Singleton handle.
pub fn create_sentinel_middleware() -> u64 {
    0x5345_4E54_494E_454C
}
