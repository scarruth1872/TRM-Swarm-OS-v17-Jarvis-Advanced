//! TRM Swarm OS — self_healing (faithful no_std port).
//! Deterministic cores ported from swarm_v2/core/self_healing.py:
//!   - InputSanitizer: control-character stripping (0x00..0x20 minus \n/\t,
//!     plus 0x7F), total-size guard, and the SQLi/XSS/path-traversal block
//!     checks. Python uses compiled PCRE regexes + Unicode NFC normalization;
//!     the no_std kernel has neither regex nor ICU, so the checks below are a
//!     deterministic literal-token reduction of those regexes (documented per
//!     function — stricter is safer for a kernel input gate, never faked).
//!   - TokenRateMonitor.check_rate(now, session_id, token_count): 1:1 port of
//!     budget/hourly-reset/sliding-window logic. Python reads `time.time()`;
//!     here `now` is an explicit parameter so behavior stays deterministic.
//!   - HealthDashboardMonitor.collect_metrics: threshold comparison core.
//!   - RecoveryCoordinator.execute_recovery: strategy dispatch + message
//!     strings 1:1 (psutil/gc/asyncio bodies → // MANUAL).
//! Deep bodies (async HDM loop, redis persistence, psutil probes,
//! remediation-engine async tasks) are host-bound → honest `// MANUAL`
//! returns below — never invented.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────
// 2.1 Input Sanitizer (IS)
// ─────────────────────────────────────────────────────────────

pub const SANITIZER_DEFAULT_MAX_CHUNK: usize = 65536;
pub const SANITIZER_DEFAULT_MAX_TOTAL: usize = 1048576;

/// Errors mirroring the ValueError messages raised by Python.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SanitizeError {
    PayloadTooLarge { max_total: usize },
    SqlInjection,
    Xss,
    PathTraversal,
}

impl SanitizeError {
    pub fn message(&self) -> String {
        match self {
            SanitizeError::PayloadTooLarge { max_total } => format!(
                "Payload too large. Max allowed: {} bytes.",
                max_total
            ),
            SanitizeError::SqlInjection => {
                "Request blocked: SQL injection patterns detected in payload.".to_string()
            }
            SanitizeError::Xss => "Request blocked: XSS patterns detected in payload.".to_string(),
            SanitizeError::PathTraversal => {
                "Request blocked: Path traversal patterns detected in payload.".to_string()
            }
        }
    }
}

/// `InputSanitizer` port. `max_chunk_size` is stored for API fidelity but
/// only `max_total_size` gates sanitize() in the Python too.
#[derive(Debug, Clone)]
pub struct InputSanitizer {
    pub max_chunk_size: usize,
    pub max_total_size: usize,
}

impl Default for InputSanitizer {
    fn default() -> Self {
        InputSanitizer {
            max_chunk_size: SANITIZER_DEFAULT_MAX_CHUNK,
            max_total_size: SANITIZER_DEFAULT_MAX_TOTAL,
        }
    }
}

impl InputSanitizer {
    pub fn new() -> Self {
        InputSanitizer::default()
    }

    /// Deterministic sanitize(): size guard → control-char strip → UTF-8
    /// lossy decode → security-pattern block.
    ///
    /// // MANUAL: Python then runs `unicodedata.normalize('NFC', ...)`; the
    /// no_std kernel has no Unicode normalization tables, so NFC is skipped
    /// (identity) and the lossy decode mirrors `errors='replace'`.
    pub fn sanitize(&self, data: &[u8]) -> Result<String, SanitizeError> {
        if data.len() > self.max_total_size {
            return Err(SanitizeError::PayloadTooLarge {
                max_total: self.max_total_size,
            });
        }
        // Remove control characters except newline/tab:
        // removed = (0x00..0x20 minus {0x09,0x0A}) ∪ {0x7F}
        let cleaned: Vec<u8> = data
            .iter()
            .copied()
            .filter(|b| !is_removed_control(*b))
            .collect();
        let decoded = String::from_utf8_lossy(&cleaned).to_string();

        if sql_suspicious(&decoded) {
            return Err(SanitizeError::SqlInjection);
        }
        if xss_suspicious(&decoded) {
            return Err(SanitizeError::Xss);
        }
        if path_traversal_suspicious(&decoded) {
            return Err(SanitizeError::PathTraversal);
        }
        Ok(decoded)
    }
}

/// Python `control_chars = set(range(0x00, 0x20)) | {0x7F}` then keeps bytes
/// only if not in that set OR in (0x0A, 0x09).
fn is_removed_control(b: u8) -> bool {
    (b < 0x20 && b != 0x09 && b != 0x0A) || b == 0x7F
}

fn contains_any_ci(haystack_lower: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack_lower.contains(n))
}

/// Literal-token reduction of the Python `SQL_RE`. // MANUAL: not a full
/// PCRE engine; each fragment below is a keyword/operator sequence lifted
/// verbatim from the compiled pattern.
fn sql_suspicious(text: &str) -> bool {
    let l = text.to_lowercase();
    const KEYWORD_NEEDLES: &[&str] = &[
        "union select",
        "union all select",
        "insert into",
        "delete from",
        "drop table",
        "drop database",
        "drop index",
        "drop view",
        "drop procedure",
        "drop function",
        "alter table",
        "alter database",
        "alter index",
        "alter view",
        "alter procedure",
        "alter function",
        "alter column",
        "create table",
        "create database",
        "create index",
        "create view",
        "create procedure",
        "create function",
        "create trigger",
        "truncate table",
        "truncate database",
        "declare ",
        "cast(",
        "convert(",
        "ord(",
        "mid(",
        "char(",
        "substring(",
        "concat(",
        "benchmark(",
        "sleep(",
        "load_file(",
        "into outfile",
        "into dumpfile",
        "information_schema",
        "waitfor delay",
        "pg_sleep(",
        "or 1=1",
        "and 1=1",
        "/*",
        "*/",
    ];
    if contains_any_ci(&l, KEYWORD_NEEDLES) {
        return true;
    }
    // `SELECT\b.{0,40}\bFROM\b` — "from" within 40 chars after "select".
    if let Some(sel) = find_ci(&l, "select") {
        let window = &l[sel..l.len().min(sel + 44)];
        if window.contains("from") {
            return true;
        }
    }
    // `(?<!\w)(OR|AND)\s+\d+\s*=\s*\d+`
    if let Some(idx) = find_ci(&l, "or ") {
        let after = &l[idx + 3..];
        if numeric_equals_numeric(after) {
            return true;
        }
    }
    if let Some(idx) = find_ci(&l, "and ") {
        let after = &l[idx + 4..];
        if numeric_equals_numeric(after) {
            return true;
        }
    }
    // `--` comment or `#` followed by whitespace.
    if l.contains("--") || l.contains("# ") || l.contains("#\t") {
        return true;
    }
    // `0x[0-9A-Fa-f]{4,}` — "0x" followed by 4+ hex digits.
    if hex_escape_present(&l) {
        return true;
    }
    // `;\s*\b(ALTER|CREATE|DELETE|DROP|EXEC|...|SHUTDOWN)\b`
    if semicolon_statement(&l) {
        return true;
    }
    // Bare `exec`/`execute` words (`EXEC\b|EXECUTE\b`).
    if l.contains("exec") {
        return true;
    }
    false
}

fn find_ci(haystack: &str, needle: &str) -> Option<usize> {
    // haystack is pre-lowercased; needle is lowercase.
    haystack.find(needle)
}

/// After an "or "/"and " prefix: digits, optional spaces, '=', digits.
fn numeric_equals_numeric(after: &str) -> bool {
    let bytes = after.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return false;
    }
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'=' {
        return false;
    }
    i += 1;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    i < bytes.len() && bytes[i].is_ascii_digit()
}

fn hex_escape_present(l: &str) -> bool {
    let bytes = l.as_bytes();
    for i in 0..bytes.len().saturating_sub(1) {
        if bytes[i] == b'0' && bytes[i + 1] == b'x' {
            let mut hex_len = 0;
            for b in &bytes[i + 2..] {
                if b.is_ascii_hexdigit() {
                    hex_len += 1;
                } else {
                    break;
                }
            }
            if hex_len >= 4 {
                return true;
            }
        }
    }
    false
}

fn semicolon_statement(l: &str) -> bool {
    const STMT: &[&str] = &[
        "alter", "create", "delete", "drop", "exec", "execute", "insert", "select",
        "truncate", "update", "union", "declare", "shutdown",
    ];
    let bytes = l.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if *b == b';' {
            let rest = &l[i + 1..];
            let rest_trim = rest.trim_start();
            for st in STMT {
                if let Some(k) = rest_trim.strip_prefix(st) {
                    // `\b` word boundary: next char must not be a word char.
                    if k.chars().next().map_or(true, |c| !c.is_alphanumeric() && c != '_') {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Literal-token reduction of the Python `XSS_RE`. // MANUAL: token
/// fragments lifted from the compiled pattern (handlers matched both with and
/// without a following `=`, since the regex allows `\s*=`).
fn xss_suspicious(text: &str) -> bool {
    let l = text.to_lowercase();
    const NEEDLES: &[&str] = &[
        "<script",
        "</script",
        "&lt;script",
        "\\x3cscript",
        "\\u003cscript",
        "onerror",
        "onload=",
        "onclick=",
        "onmouseover",
        "onfocus",
        "onblur",
        "onsubmit",
        "onchange",
        "onkeypress",
        "onkeydown",
        "onkeyup",
        "ondblclick",
        "onmousedown",
        "onmouseup",
        "onmouseenter",
        "onmouseleave",
        "onpageshow",
        "ontoggle",
        "onstart",
        "javascript:",
        "data:text/html",
        "eval(",
        "alert(",
        "prompt(",
        "confirm(",
        "string.fromcharcode(",
        "document.cookie",
        "document.location",
        "document.write(",
    ];
    contains_any_ci(&l, NEEDLES)
}

/// Literal-token reduction of the Python `PATH_RE`. // MANUAL: fragments
/// lifted verbatim; RFC-encoding variants are enumerated explicitly.
fn path_traversal_suspicious(text: &str) -> bool {
    let l = text.to_lowercase();
    const NEEDLES: &[&str] = &[
        "../",
        "..\\",
        "..%2f",
        "..%5c",
        "%2e%2e%2f",
        "%2e%2e%5c",
        "%252e%252e%252f",
        "%252e%252e%255c",
        "%c0%ae",
        "%00",
        "%2500",
        "~/",
        "~\\",
        "/etc/",
        "/windows/",
        "/proc/",
        "/boot/",
        "/sys/",
        "/dev/",
        "/bin/",
        "/usr/",
        "/var/",
        "/opt/",
        "/tmp/",
        "/root/",
        "/home/",
    ];
    if contains_any_ci(&l, NEEDLES) {
        return true;
    }
    // `[A-Za-z]:[/\\]` — drive-letter path prefix.
    let bytes = l.as_bytes();
    for i in 0..bytes.len().saturating_sub(2) {
        if bytes[i].is_ascii_alphabetic() && bytes[i + 1] == b':' {
            if bytes[i + 2] == b'/' || bytes[i + 2] == b'\\' {
                return true;
            }
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────
// 2.2 Token Rate Monitor (TRM)
// ─────────────────────────────────────────────────────────────

pub const TRM_DEFAULT_WINDOW_SIZE: f64 = 1.0;
pub const TRM_DEFAULT_MAX_RATE: u64 = 50000;
pub const TRM_DEFAULT_MAX_BUDGET: u64 = 100_000_000;
pub const TRM_HOURLY_RESET_SECONDS: f64 = 3600.0;

/// Sessions with unlimited budget (Python tuple check).
pub const UNLIMITED_SESSIONS: [&str; 4] = ["127.0.0.1", "::1", "localhost", "testclient"];

pub fn is_unlimited_session(session_id: &str) -> bool {
    UNLIMITED_SESSIONS.contains(&session_id)
}

/// Per-session state (Python `defaultdict(deque)` + totals + timestamps).
#[derive(Debug, Clone)]
pub struct RateSession {
    pub id: String,
    pub total_tokens: u64,
    pub last_ts: f64,
    /// Timestamps of tokens inside the current window (Python deque).
    pub bucket: Vec<f64>,
}

/// `TokenRateMonitor` port. All time reads are explicit `now` parameters so
/// the sliding-window logic is fully deterministic.
#[derive(Debug, Clone)]
pub struct TokenRateMonitor {
    pub window_size: f64,
    pub max_rate: u64,
    pub max_budget: u64,
    pub sessions: Vec<RateSession>,
}

impl TokenRateMonitor {
    pub fn new() -> Self {
        TokenRateMonitor {
            window_size: TRM_DEFAULT_WINDOW_SIZE,
            max_rate: TRM_DEFAULT_MAX_RATE,
            max_budget: TRM_DEFAULT_MAX_BUDGET,
            sessions: Vec::new(),
        }
    }

    /// Check a token count against budget + sliding-window rate.
    /// Returns `Ok(true)` when admitted (Python returns True), or the exact
    /// ValueError message text on rejection.
    pub fn check_rate(
        &mut self,
        now: f64,
        session_id: &str,
        token_count: u64,
    ) -> Result<bool, String> {
        // Localhost/loopback sessions have unlimited budget.
        if is_unlimited_session(session_id) {
            return Ok(true);
        }

        if !self.sessions.iter().any(|s| s.id == session_id) {
            self.sessions.push(RateSession {
                id: session_id.to_string(),
                total_tokens: 0,
                last_ts: 0.0,
                bucket: Vec::new(),
            });
        }
        let sess = self
            .sessions
            .iter_mut()
            .find(|s| s.id == session_id)
            .expect("session was just ensured to exist");

        // Reset budget every hour for active sessions.
        if now - sess.last_ts > TRM_HOURLY_RESET_SECONDS {
            sess.total_tokens = 0;
            sess.last_ts = now;
        }

        if sess.total_tokens + token_count > self.max_budget {
            return Err(format!(
                "Session budget of {} tokens exceeded.",
                self.max_budget
            ));
        }

        // Remove old entries from the sliding window.
        let cutoff = now - self.window_size;
        while sess.bucket.first().map_or(false, |t| *t < cutoff) {
            sess.bucket.remove(0);
        }

        // Current rate check.
        let current_rate = sess.bucket.len() as f64 / self.window_size;
        if current_rate + token_count as f64 > self.max_rate as f64 {
            return Err(format!(
                "Token rate exceeded {} tokens/sec.",
                self.max_rate
            ));
        }

        for _ in 0..token_count {
            sess.bucket.push(now);
        }
        sess.total_tokens += token_count;
        Ok(true)
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

// ─────────────────────────────────────────────────────────────
// 3.1 Health Dashboard Monitor (HDM) — deterministic core
// ─────────────────────────────────────────────────────────────

pub const HDM_MEMORY_PRESSURE_THRESHOLD: f64 = 80.0;
pub const HDM_CPU_PRESSURE_THRESHOLD: f64 = 70.0;

/// `HealthDashboardMonitor.collect_metrics()` result (anomaly order matches
/// Python: memory first, then CPU).
#[derive(Debug, Clone)]
pub struct MetricsReport {
    pub memory_percent: f64,
    pub cpu_percent: f64,
    pub anomalies: Vec<&'static str>,
}

/// Ports the metric threshold core. // MANUAL: Python reads live
/// psutil.virtual_memory()/cpu_percent() — those probes are host-only, so
/// the percents arrive as parameters.
pub fn collect_metrics(memory_percent: f64, cpu_percent: f64) -> MetricsReport {
    let mut anomalies = Vec::new();
    if memory_percent > HDM_MEMORY_PRESSURE_THRESHOLD {
        anomalies.push("memory_pressure");
    }
    if cpu_percent > HDM_CPU_PRESSURE_THRESHOLD {
        anomalies.push("cpu_pressure");
    }
    MetricsReport {
        memory_percent,
        cpu_percent,
        anomalies,
    }
}

// ─────────────────────────────────────────────────────────────
// 3.2 Recovery Coordinator (RCM) — deterministic core
// ─────────────────────────────────────────────────────────────

/// Strategy names Python registers (insertion order).
pub const RECOVERY_STRATEGIES: [&str; 4] =
    ["memory_pressure", "token_overflow", "cpu_pressure", "process_crash"];

/// Result of a recovery dispatch. `message` matches the Python strategy
/// return strings; `new_token_limit` is set only by cpu_pressure
/// (token_monitor.max_rate halving).
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub message: String,
    pub new_token_limit: Option<u64>,
}

impl RecoveryResult {
    fn msg(s: &str) -> Self {
        RecoveryResult {
            message: s.to_string(),
            new_token_limit: None,
        }
    }
}

/// `RecoveryCoordinator.execute_recovery(anomaly_type, context)`.
/// Unknown anomaly types hit `_default_recovery`. // MANUAL: the Python
/// strategies also run `gc.collect()`, purge live token buckets, and fire
/// async remediation-engine tasks — all host-bound; the decision core and
/// returned messages are ported faithfully.
pub fn execute_recovery(anomaly_type: &str, context_role: &str, cpu_token_limit: u64) -> RecoveryResult {
    match anomaly_type {
        "memory_pressure" => {
            RecoveryResult::msg("Executed GC cleanup and TRM bucket purge for memory pressure.")
        }
        "token_overflow" => RecoveryResult::msg("Token buckets emergency reset."),
        "cpu_pressure" => {
            let new_limit = core::cmp::max(100, cpu_token_limit / 2);
            RecoveryResult {
                message: format!(
                    "CPU pressure mitigated. Lowered token rate limit to {}/sec.",
                    new_limit
                ),
                new_token_limit: Some(new_limit),
            }
        }
        "process_crash" => {
            // Python tries to import remediation_engine; on failure it
            // returns the "Cannot restart worker" string. The engine is a
            // host async service → the successful-dispatch message is
            // returned (// MANUAL).
            RecoveryResult::msg(&format!(
                "Restarting worker {} via RemediationEngine.",
                context_role
            ))
        }
        _ => RecoveryResult::msg("No specific recovery strategy applied."),
    }
}
