//! TRM Swarm OS — Security Scanner (no_std port).
//! 1:1 core semantics from swarm_v2/core/security_scanner.py — Catalyst Phase 1.
//! Deterministic vulnerability scanning: pattern match against known threat
//! signatures, severity scoring, scan history with limit.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Severity levels.
pub const SEV_INFO: u64 = 0;
pub const SEV_LOW: u64 = 1;
pub const SEV_MEDIUM: u64 = 2;
pub const SEV_HIGH: u64 = 3;
pub const SEV_CRITICAL: u64 = 4;

/// Known threat patterns (deterministic signatures).
pub const THREAT_PATTERNS: &'static [(&'static str, &'static str, u64)] = &[
    ("sql_injection", "SELECT", SEV_CRITICAL),
    ("hardcoded_secret", "SECRET_KEY", SEV_HIGH),
    ("eval_usage", "eval(", SEV_HIGH),
    ("path_traversal", "../", SEV_HIGH),
    ("command_injection", "os.system", SEV_CRITICAL),
    ("debug_print", "print(", SEV_LOW),
    ("insecure_deserialization", "pickle.load", SEV_CRITICAL),
    ("weak_crypto", "md5(", SEV_MEDIUM),
    ("bare_except", "except:", SEV_MEDIUM),
    ("shell_true", "shell=True", SEV_CRITICAL),
];

/// A scan result entry.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub pattern: String,
    pub severity: u64,
    pub count: u64,
}

/// Security scanner state.
#[derive(Debug, Clone)]
pub struct SecurityScanner {
    pub history: Vec<ScanResult>,
    pub total_findings: u64,
    pub max_severity: u64,
}

impl SecurityScanner {
    pub fn new() -> Self {
        SecurityScanner {
            history: Vec::new(),
            total_findings: 0,
            max_severity: 0,
        }
    }

    /// Scan content against the threat patterns.
    /// Returns (findings, max_severity).
    pub fn scan(&mut self, content: &str) -> (u64, u64) {
        let mut findings = 0u64;
        let mut max_sev = 0u64;
        for (name, pat, sev) in THREAT_PATTERNS {
            let count = content.matches(pat).count() as u64;
            if count > 0 {
                findings += count;
                max_sev = max_sev.max(*sev);
                self.history.push(ScanResult {
                    pattern: name.to_string(),
                    severity: *sev,
                    count,
                });
            }
        }
        self.total_findings += findings;
        self.max_severity = self.max_severity.max(max_sev);
        (findings, max_sev)
    }

    /// Scan history, limited (1:1 get_scan_history(limit)).
    pub fn get_scan_history(&self, limit: u64) -> Vec<ScanResult> {
        let n = (self.history.len() as u64).min(limit) as usize;
        self.history[self.history.len().saturating_sub(n)..].to_vec()
    }

    /// Summary: (total_findings, max_severity, history_len).
    pub fn summary(&self) -> (u64, u64, u64) {
        (self.total_findings, self.max_severity, self.history.len() as u64)
    }
}

/// Singleton handle.
pub fn get_security_scanner() -> u64 {
    0x5345_4355_5249_5459
}
