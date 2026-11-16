//! TRM Swarm OS — Sentinel Middleware (no_std port).
//! 1:1 core semantics from swarm_v2/core/sentinel.py — Catalyst.
//! Request gate: rate-limit + allowlist/blocklist check + verdict.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

// ── The oracle's rules (swarm_v2/core/sentinel.py + strip_sentinel_lexer.rs) ─────────────────────
//
// Three pure rules from this module's python, none of which the rust carried (it had a naive
// counter with no time window):
//
//   1. the fixed-window rate limiter (`_check_rate_limit`): a trusted internal source always passes;
//      an absent counter initialises the window; a window whose `reset` has passed (strictly, so
//      exactly at reset the window has NOT expired) resets; a count at or above the limit is
//      refused; otherwise the count increments. Defaults 100 requests per 60 seconds.
//   2. the sovereign-asset guard (`_sanitize_request`): a path whose lowercased text contains any of
//      five proprietary markers is refused outright.
//   3. the injection lexer (`strip_sentinel_lexer`): verdict 1 SQLi, 2 XSS, 3 traversal, 0 clean, as
//      an if/else-if chain so the earlier class wins.

/// Rate limiter defaults.
pub const RATE_LIMIT_DEFAULT: u64 = 100;
pub const RATE_WINDOW_DEFAULT: u64 = 60;

/// `_sanitize_request`'s proprietary markers, matched against the lowercased path.
pub const PROTECTED_PATH_MARKERS: [&str; 5] = [
    "swarm_rust_kernel",
    "trm_uefi_kernel",
    "ddr_antibodies.json",
    "nand_fabric",
    "sovereign_weights",
];

/// The sovereign-asset guard: `Some(marker)` when the path names a proprietary module or vault.
pub fn protected_path(path: &str) -> Option<&'static str> {
    let low = path.to_lowercase();
    PROTECTED_PATH_MARKERS.iter().find(|m| low.contains(**m)).copied()
}

/// Case-insensitive substring.
fn contains_ci(hay: &[u8], needle: &[u8]) -> bool {
    if needle.len() > hay.len() {
        return false;
    }
    hay.windows(needle.len()).any(|w| w.iter().zip(needle.iter()).all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase()))
}

/// The lexer's verdict: 1 SQLi, 2 XSS, 3 traversal, 0 clean.
pub fn scan_verdict(request: &[u8]) -> i32 {
    if contains_ci(request, b"union select") || contains_ci(request, b"from users") {
        1
    } else if contains_ci(request, b"<script") || contains_ci(request, b"alert(") {
        2
    } else if contains_ci(request, b"../") || contains_ci(request, b"/etc/") {
        3
    } else {
        0
    }
}

/// Batch entry point, mirroring `strip_batch_sentinel_scan`.
pub fn scan_verdicts(requests: &[&[u8]]) -> Vec<i32> {
    requests.iter().map(|r| scan_verdict(r)).collect()
}

/// The fixed-window rate limiter's state, with the oracle's three-branch check.
#[derive(Debug, Clone)]
pub struct RateWindow {
    pub count: u64,
    pub reset_at: u64,
    pub limit: u64,
    pub window: u64,
}

impl RateWindow {
    pub fn new(limit: u64, window: u64) -> Self {
        RateWindow { count: 0, reset_at: 0, limit, window }
    }

    /// `_check_rate_limit`: (allowed, reason). A trusted source short-circuits; an unset counter
    /// initialises the window; an expired window resets; a count at or above the limit refuses.
    pub fn check_rate_limit(&mut self, now: u64, trusted: bool) -> (bool, &'static str) {
        if trusted {
            return (true, "internal_trusted");
        }
        if self.count == 0 {
            self.count = 1;
            self.reset_at = now + self.window;
            return (true, "window initialised");
        }
        if now > self.reset_at {
            self.count = 1;
            self.reset_at = now + self.window;
            return (true, "window reset");
        }
        if self.count >= self.limit {
            return (false, "rate limit exceeded");
        }
        self.count += 1;
        (true, "counted")
    }
}

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
