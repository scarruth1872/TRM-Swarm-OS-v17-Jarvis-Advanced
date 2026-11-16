//! TRM Swarm OS — Security Scanner (no_std port).
//! 1:1 core semantics from swarm_v2/core/security_scanner.py — Catalyst Phase 1.
//! Deterministic vulnerability scanning: pattern match against known threat
//! signatures, severity scoring, scan history with limit.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Severity levels.
// The invented SEV_* scale was removed with the pattern table it fed: the oracle's secrets scan
// assigns no severity, and `ScanSummary` is where severities live (summed from scans that have them).

// ── The oracle's model (swarm_v2/core/security_scanner.py) ──────────────────────────────────────
//
// python `_scan_artifact_files` searches artifact content for FIVE credential shapes and labels:
//     AKIA[0-9A-Z]{16}            -> "AWS Access Key"
//     (?i)sk-[a-zA-Z0-9]{20,}     -> "API Key"
//     (?i)eyJ[a-zA-Z0-9_-]+\.eyJ  -> "JWT Token"
//     (?i)ghp_[a-zA-Z0-9]{36}     -> "GitHub Token"
//     (?i)hf_[a-zA-Z0-9]{20,}     -> "HuggingFace Token"
// and `scan_all` derives a summary status with the rule
//     status = "critical" if critical > 0 else "warning" if high > 0 else "clean"
// over per-severity CVE counts summed across scans; `get_scan_history(limit=10)` returns the last N.
//
// This module carried a different thing entirely: ten generic code-smell substrings ("SELECT",
// "eval(", "../", "print(", "md5(") each with a severity the oracle never assigns, plus a findings
// counter. The secrets scan in the oracle reports `secret_count` and `cve_count: 0` -- it assigns no
// severity at all -- so the invented table and its severity scale are replaced by the oracle's five
// shapes and the summary rule, and the severity counts arrive from the caller as they do in python.
//
// Host-side and deliberately not ported: the `pip list` / `npm audit` / dpkg subprocess scans. A
// firmware image has no package manager to shell out to.

/// The oracle's labels, in its own order.
pub const SECRET_LABELS: [&str; 5] = [
    "AWS Access Key",
    "API Key",
    "JWT Token",
    "GitHub Token",
    "HuggingFace Token",
];

fn is_alnum(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}
fn is_alnum_dash_us(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'-' || c == b'_'
}
/// Case-insensitive prefix test. Both sides are folded: an earlier version lowercased only the
/// haystack and compared against the needle verbatim, so every needle containing an uppercase byte
/// ("AKIA", "eyJ") never matched anything -- AWS and JWT detection was silently dead, and only a
/// comparison over real file data caught it.
fn starts_ci(hay: &[u8], at: usize, needle: &[u8]) -> bool {
    at + needle.len() <= hay.len()
        && hay[at..at + needle.len()]
            .iter()
            .zip(needle.iter())
            .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
}

/// Count the oracle's five credential shapes in a piece of content. Shape matchers rather than a
/// regex engine: same patterns, spelled out for bare metal.
pub fn find_secrets(content: &str) -> [usize; 5] {
    let b = content.as_bytes();
    let mut hits = [0usize; 5];
    let mut i = 0usize;
    while i < b.len() {
        // 1. AWS Access Key: AKIA + exactly 16 of [0-9A-Z]
        if starts_ci(b, i, b"AKIA") {
            let run = b[i + 4..].iter().take_while(|c| c.is_ascii_digit() || c.is_ascii_uppercase()).count();
            if run >= 16 {
                hits[0] += 1;
            }
        }
        // 2. API Key: sk- + 20 or more alphanumerics
        if starts_ci(b, i, b"sk-") {
            let run = b[i + 3..].iter().take_while(|c| is_alnum(**c)).count();
            if run >= 20 {
                hits[1] += 1;
            }
        }
        // 3. JWT: `eyJ[a-zA-Z0-9_-]+\.eyJ` -- the literal dot is part of the oracle's pattern, and an
        // earlier version of this matcher ran past it looking for the second `eyJ` in the same run,
        // which meant a real JWT never matched at all.
        if starts_ci(b, i, b"eyJ") {
            let mut j = i + 3;
            while j < b.len() && is_alnum_dash_us(b[j]) {
                j += 1;
            }
            if j > i + 3 && j < b.len() && b[j] == b'.' && starts_ci(b, j + 1, b"eyJ") {
                hits[2] += 1;
            }
        }
        // 4. GitHub Token: ghp_ + exactly 36 alphanumerics
        if starts_ci(b, i, b"ghp_") {
            let run = b[i + 4..].iter().take_while(|c| is_alnum(**c)).count();
            if run >= 36 {
                hits[3] += 1;
            }
        }
        // 5. HuggingFace Token: hf_ + 20 or more alphanumerics
        if starts_ci(b, i, b"hf_") {
            let run = b[i + 3..].iter().take_while(|c| is_alnum(**c)).count();
            if run >= 20 {
                hits[4] += 1;
            }
        }
        i += 1;
    }
    hits
}

/// A scan result entry: the oracle's label and how many were found. No severity -- the oracle's
/// secrets scan does not assign one.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub label: String,
    pub count: usize,
}

/// The oracle's summary, with its exact status rule.
#[derive(Debug, Clone, Copy)]
pub struct ScanSummary {
    pub total_cves: u64,
    pub critical: u64,
    pub high: u64,
    pub medium: u64,
    pub low: u64,
    pub status: &'static str,
}

/// `status = "critical" if critical > 0 else "warning" if high > 0 else "clean"`.
pub fn status_for(critical: u64, high: u64) -> &'static str {
    if critical > 0 {
        "critical"
    } else if high > 0 {
        "warning"
    } else {
        "clean"
    }
}

/// The oracle's summary over per-severity CVE counts.
pub fn summarize(total_cves: u64, critical: u64, high: u64, medium: u64, low: u64) -> ScanSummary {
    ScanSummary { total_cves, critical, high, medium, low, status: status_for(critical, high) }
}

/// Security scanner state.
#[derive(Debug, Clone)]
pub struct SecurityScanner {
    pub history: Vec<ScanResult>,
    pub total_findings: u64,
}

impl SecurityScanner {
    pub fn new() -> Self {
        SecurityScanner {
            history: Vec::new(),
            total_findings: 0,
        }
    }

    /// Scan content for the oracle's five credential shapes. Returns the number of hits, which is
    /// what python's secrets scan reports as `secret_count`; it assigns no severity, so nothing here
    /// does either — severity counts reach `summarize` from whoever knows them, exactly as in python.
    pub fn scan(&mut self, content: &str) -> usize {
        let hits = find_secrets(content);
        let mut total = 0usize;
        for (i, n) in hits.iter().enumerate() {
            if *n > 0 {
                total += *n;
                self.history.push(ScanResult {
                    label: String::from(SECRET_LABELS[i]),
                    count: *n,
                });
            }
        }
        self.total_findings += total as u64;
        total
    }

    /// Scan history, limited (1:1 get_scan_history(limit)).
    pub fn get_scan_history(&self, limit: u64) -> Vec<ScanResult> {
        let n = (self.history.len() as u64).min(limit) as usize;
        self.history[self.history.len().saturating_sub(n)..].to_vec()
    }

    /// Summary: (total_findings, history_len). No severity: see `ScanSummary` for the oracle's
    /// severity model, which takes its counts from scans that have them.
    pub fn summary(&self) -> (u64, u64) {
        (self.total_findings, self.history.len() as u64)
    }
}

/// Singleton handle.
pub fn get_security_scanner() -> u64 {
    0x5345_4355_5249_5459
}
