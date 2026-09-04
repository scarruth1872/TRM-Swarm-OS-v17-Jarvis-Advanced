//! TRM Swarm OS — self_fuzzing_engine (faithful no_std port).
//! Continuous Swarm Self-Fuzzing Engine, from swarm_v2/core/self_fuzzing_engine.py:
//!   - `VULN_PATTERNS` vulnerability-pattern library: all 15 entries with
//!     severity / antibody_type / fix strings 1:1 from the Python dict.
//!   - `Vulnerability` dataclass + per-line scan: fingerprint format
//!     `"<basename>:<line>:<VULN_TYPE>"`, `line_content = line.strip()[:120]`,
//!     line-number enumeration — all 1:1.
//!   - Antibody generation: fields 1:1; `timestamp` is a caller parameter
//!     (Python reads the wall clock) and the Python antibody id uses the
//!     process-salted builtin `hash()` (`hash(fingerprint) & 0xFFFF`) which is
//!     NOT reproducible across runs — the port substitutes a deterministic
//!     FNV-1a 16-bit hash of the fingerprint (documented divergence, same
//!     format `DDR-<TYPE>-<04x>`).
//!   - Engine accumulators + summary (files/lines scanned, findings,
//!     antibodies, by-severity, by-type).
//! Regex caveat (honest): Python matches each line with `re.search(pattern,
//! line, re.IGNORECASE)`. The no_std kernel has no regex engine, so each
//! pattern carries a deterministic literal-token detector that is a faithful
//! *reduction* of that regex (documented per entry). Three patterns
//! (RACE_CONDITION, USE_AFTER_FREE) embed `\n` and therefore can never match
//! a single scanned line in Python either — ported as always-false, matching
//! real Python line-scan behavior. Filesystem walking + ThreadPoolExecutor
//! orchestration (`scan_all`) is host-bound → `// MANUAL`.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Severity levels, indexed by `severity_index`.
pub const SEVERITIES: [&str; 4] = ["CRITICAL", "HIGH", "MEDIUM", "LOW"];

pub fn severity_index(severity: &str) -> usize {
    match severity {
        "CRITICAL" => 0,
        "HIGH" => 1,
        "MEDIUM" => 2,
        _ => 3, // LOW / unknown sorts as LOW
    }
}

// ─── Vulnerability Pattern Library ───────────────────────────

/// One vulnerability pattern entry. `detect` is the deterministic
/// literal-token reduction of the Python regex (see module doc).
#[derive(Debug, Clone, Copy)]
pub struct VulnPattern {
    pub name: &'static str,
    pub severity: &'static str,
    pub antibody_type: &'static str,
    pub fix: &'static str,
    pub detect: fn(&str) -> bool,
}

fn lower(line: &str) -> String {
    line.to_lowercase()
}

fn any_of(hay_lower: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay_lower.contains(n))
}

// SQL_INJECTION — Python: `(execute|executemany)\s*\(.*[\+\%].*\) | f...execute | .format(...).execute`.
// Kernel reduction: flag the SQL driver call sites (the operator/format
// context check needs backtracking regex; call tokens are the stable core).
fn detect_sql_injection(line: &str) -> bool {
    let l = lower(line);
    any_of(&l, &["executemany(", "execute("])
}

// COMMAND_INJECTION — Python: `os.system(|subprocess.(call|run|Popen)(...shell\s*=\s*True|eval(|exec(`.
fn detect_command_injection(line: &str) -> bool {
    let l = lower(line);
    if any_of(&l, &["os.system(", "eval(", "exec("]) {
        return true;
    }
    if any_of(&l, &["subprocess.call(", "subprocess.run(", "subprocess.popen("]) {
        return any_of(
            &l,
            &["shell=true", "shell = true", "shell= true", "shell =true", "shell=  true"],
        );
    }
    false
}

// PATH_TRAVERSAL — Python: `open(.*\.\./|open(.*os.path.join(.*user_input|Path(.*user_input`.
fn detect_path_traversal(line: &str) -> bool {
    let l = lower(line);
    if any_of(&l, &["../", "..\\", "os.path.join("]) {
        return true;
    }
    l.contains("path(") && l.contains("user_input")
}

/// `(key)\s*=\s*['"](\w{min,})['"]` — assignment of a quoted literal after a
/// sensitive variable name (word chars = [A-Za-z0-9_.-], Python `\w` + . -).
fn assignment_after_key(line_lower: &str, keys: &[&str], min_len: usize) -> bool {
    let bytes = line_lower.as_bytes();
    for key in keys {
        let mut search_from = 0;
        while let Some(rel) = line_lower[search_from..].find(key) {
            let kstart = search_from + rel;
            // `\s*=\s*` after the key
            let mut i = kstart + key.len();
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'=' {
                i += 1;
                while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                    i += 1;
                }
                if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                    i += 1;
                    let mut n = 0usize;
                    while i < bytes.len()
                        && bytes[i] != b'"'
                        && bytes[i] != b'\''
                        && (bytes[i].is_ascii_alphanumeric()
                            || bytes[i] == b'_'
                            || bytes[i] == b'.'
                            || bytes[i] == b'-')
                    {
                        n += 1;
                        i += 1;
                    }
                    if n >= min_len {
                        return true;
                    }
                }
            }
            search_from = kstart + key.len();
        }
    }
    false
}

// HARDCODED_SECRET — `(api_key|secret|password|token|private_key)\s*=\s*['"]\w{8,}['"]`.
fn detect_hardcoded_secret(line: &str) -> bool {
    assignment_after_key(
        &lower(line),
        &["api_key", "secret", "password", "token", "private_key"],
        8,
    )
}

// HARDCODED_TOKEN — `(github_token|aws_key|azure_key|jwt_secret)\s*=\s*['"]\w{10,}['"]`.
fn detect_hardcoded_token(line: &str) -> bool {
    assignment_after_key(
        &lower(line),
        &["github_token", "aws_key", "azure_key", "jwt_secret"],
        10,
    )
}

// INSECURE_DESERIALIZATION — pickle/yaml/marshal load sites.
fn detect_insecure_deserialization(line: &str) -> bool {
    let l = lower(line);
    any_of(&l, &["pickle.loads(", "pickle.load(", "yaml.load(", "marshal.loads("])
}

// DEBUG_PRINT — `print\(.*(secret|password|token|key|private)` (leak word
// must appear after the print call).
fn detect_debug_print(line: &str) -> bool {
    let l = lower(line);
    match l.find("print(") {
        Some(idx) => {
            let after = &l[idx + 6..];
            any_of(after, &["secret", "password", "token", "key", "private"])
        }
        None => false,
    }
}

// BARE_EXCEPT — `except\s*:`.
fn detect_bare_except(line: &str) -> bool {
    let b = line.as_bytes();
    let n = b.len();
    let mut i = 0;
    while i + 6 <= n {
        if b[i..i + 6].eq_ignore_ascii_case(b"except") {
            let mut j = i + 6;
            while j < n && (b[j] == b' ' || b[j] == b'\t') {
                j += 1;
            }
            if j < n && b[j] == b':' {
                return true;
            }
        }
        i += 1;
    }
    false
}

// WEAK_CRYPTO — `hashlib.(md5|sha1)(| random.(random|randint)(...(password|token|crypto|auth)`.
fn detect_weak_crypto(line: &str) -> bool {
    let l = lower(line);
    if any_of(&l, &["hashlib.md5(", "hashlib.sha1("]) {
        return true;
    }
    any_of(&l, &["random.random(", "random.randint("])
        && any_of(&l, &["password", "token", "crypto", "auth"])
}

// RACE_CONDITION — the Python pattern embeds `.*\n.*`; `scan_file` matches
// each line separately, so this regex can never fire in real line-scanning.
// Faithful port: never matches. // MANUAL (would need whole-file scanning).
fn detect_race_condition(_line: &str) -> bool {
    false
}

// RESOURCE_LEAK — Python regex effectively reduces (per-line `$`) to:
// a line that ends right after an open(...) call with no .close()/with guard.
fn detect_resource_leak(line: &str) -> bool {
    let trimmed = line.trim_end();
    if !trimmed.ends_with(')') {
        return false;
    }
    let t = trimmed.to_lowercase();
    if t.contains(".close") || t.contains("with") {
        return false;
    }
    t.contains("open(")
}

// HTTP_NO_VERIFY — `verify\s*=\s*False|ssl\s*=\s*False` (incl. requests.get).
fn detect_http_no_verify(line: &str) -> bool {
    let l = lower(line);
    any_of(&l, &["verify=false", "ssl=false", "requests.get("]) && l.contains("verify=false")
}

// UNSAFE_REDIRECT — `redirect\(.*request\.(args|form|json|data)`.
fn detect_unsafe_redirect(line: &str) -> bool {
    let l = lower(line);
    l.contains("redirect(")
        && any_of(
            &l,
            &["request.args", "request.form", "request.json", "request.data"],
        )
}

// BUFFER_OVERFLOW — ctypes raw-memory call sites.
fn detect_buffer_overflow(line: &str) -> bool {
    let l = lower(line);
    any_of(
        &l,
        &[
            "ctypes.string_at(",
            "ctypes.memmove(",
            "ctypes.memset(",
            "ctypes.memcpy(",
        ],
    )
}

// USE_AFTER_FREE — Python pattern embeds `\n`; never fires in per-line
// scanning. Faithful port: never matches. // MANUAL.
fn detect_use_after_free(_line: &str) -> bool {
    false
}

/// The 15-entry pattern library, Python insertion order.
pub const VULN_PATTERNS: [VulnPattern; 15] = [
    VulnPattern {
        name: "SQL_INJECTION",
        severity: "CRITICAL",
        antibody_type: "sql_injection",
        fix: "Use parameterized queries with ? placeholders",
        detect: detect_sql_injection,
    },
    VulnPattern {
        name: "COMMAND_INJECTION",
        severity: "CRITICAL",
        antibody_type: "command_injection",
        fix: "Use subprocess.run with shell=False and argument lists",
        detect: detect_command_injection,
    },
    VulnPattern {
        name: "PATH_TRAVERSAL",
        severity: "HIGH",
        antibody_type: "path_traversal",
        fix: "Sanitize paths with os.path.realpath() validation",
        detect: detect_path_traversal,
    },
    VulnPattern {
        name: "HARDCODED_SECRET",
        severity: "CRITICAL",
        antibody_type: "hardcoded_secret",
        fix: "Use environment variables or a secrets manager",
        detect: detect_hardcoded_secret,
    },
    VulnPattern {
        name: "HARDCODED_TOKEN",
        severity: "CRITICAL",
        antibody_type: "hardcoded_token",
        fix: "Use os.environ.get() or a vault service",
        detect: detect_hardcoded_token,
    },
    VulnPattern {
        name: "INSECURE_DESERIALIZATION",
        severity: "HIGH",
        antibody_type: "insecure_deserialization",
        fix: "Use yaml.safe_load() or avoid pickle entirely",
        detect: detect_insecure_deserialization,
    },
    VulnPattern {
        name: "DEBUG_PRINT",
        severity: "MEDIUM",
        antibody_type: "debug_leak",
        fix: "Remove debug prints before production deployment",
        detect: detect_debug_print,
    },
    VulnPattern {
        name: "BARE_EXCEPT",
        severity: "LOW",
        antibody_type: "bare_except",
        fix: "Specify exception types: except (ValueError, KeyError) as e:",
        detect: detect_bare_except,
    },
    VulnPattern {
        name: "WEAK_CRYPTO",
        severity: "HIGH",
        antibody_type: "weak_crypto",
        fix: "Use hashlib.sha256 or secrets module for cryptographic operations",
        detect: detect_weak_crypto,
    },
    VulnPattern {
        name: "RACE_CONDITION",
        severity: "MEDIUM",
        antibody_type: "race_condition",
        fix: "Use atomic file operations with tempfile + os.replace()",
        detect: detect_race_condition,
    },
    VulnPattern {
        name: "RESOURCE_LEAK",
        severity: "LOW",
        antibody_type: "resource_leak",
        fix: "Use context managers: with open(...) as f:",
        detect: detect_resource_leak,
    },
    VulnPattern {
        name: "HTTP_NO_VERIFY",
        severity: "HIGH",
        antibody_type: "ssl_bypass",
        fix: "Remove verify=False in production; use proper SSL certificates",
        detect: detect_http_no_verify,
    },
    VulnPattern {
        name: "UNSAFE_REDIRECT",
        severity: "HIGH",
        antibody_type: "open_redirect",
        fix: "Validate redirect URLs against a whitelist",
        detect: detect_unsafe_redirect,
    },
    VulnPattern {
        name: "BUFFER_OVERFLOW",
        severity: "CRITICAL",
        antibody_type: "buffer_overflow",
        fix: "Use Python native types or bounds-checked ctypes alternatives",
        detect: detect_buffer_overflow,
    },
    VulnPattern {
        name: "USE_AFTER_FREE",
        severity: "CRITICAL",
        antibody_type: "use_after_free",
        fix: "Set deleted references to None",
        detect: detect_use_after_free,
    },
];

pub fn pattern_by_name(name: &str) -> Option<&'static VulnPattern> {
    VULN_PATTERNS.iter().find(|p| p.name == name)
}

// ─── Vulnerability dataclass ─────────────────────────────────

/// A detected vulnerability (Python `@dataclass Vulnerability`).
#[derive(Debug, Clone)]
pub struct Vulnerability {
    pub file: String,
    pub line: u64,
    pub line_content: String,
    pub vuln_type: String,
    pub severity: String,
    pub antibody_type: String,
    pub fix_suggestion: String,
    pub fingerprint: String,
}

/// Python `line.strip()[:120]`.
fn line_content(line: &str) -> String {
    line.trim().chars().take(120).collect()
}

/// `filepath.name` = final path component ('/' or '\\' separated).
pub fn base_name(path: &str) -> &str {
    let after_slash = path.rsplit('/').next().unwrap_or(path);
    after_slash.rsplit('\\').next().unwrap_or(after_slash)
}

/// Split content the way Python `str.splitlines()` enumerates lines for a
/// scan (handles trailing '\r' from CRLF files; no trailing empty line for a
/// trailing '\n').
fn splitlines(content: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    for line in content.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        out.push(line);
    }
    if let Some(last) = out.last() {
        if last.is_empty() {
            out.pop();
        }
    }
    out
}

/// Fuzz a single line against the whole pattern library (per-line
/// `re.search(pattern, line, re.IGNORECASE)` semantics — every matching
/// pattern yields its own Vulnerability, in library order).
pub fn scan_line(line: &str, file: &str, line_num: u64) -> Vec<Vulnerability> {
    let mut vulns = Vec::new();
    for pat in VULN_PATTERNS.iter() {
        if (pat.detect)(line) {
            let fingerprint = format!("{}:{}:{}", base_name(file), line_num, pat.name);
            vulns.push(Vulnerability {
                file: file.to_string(),
                line: line_num,
                line_content: line_content(line),
                vuln_type: pat.name.to_string(),
                severity: pat.severity.to_string(),
                antibody_type: pat.antibody_type.to_string(),
                fix_suggestion: pat.fix.to_string(),
                fingerprint,
            });
        }
    }
    vulns
}

/// Fuzz an entire file's content (the deterministic core of `scan_file`;
/// the file read itself is // MANUAL host IO).
pub fn scan_content(content: &str, file: &str) -> Vec<Vulnerability> {
    let mut vulns = Vec::new();
    for (line_num, line) in splitlines(content).iter().enumerate() {
        vulns.extend(scan_line(line, file, line_num as u64 + 1));
    }
    vulns
}

// ─── Antibody generation ─────────────────────────────────────

/// Deterministic FNV-1a 16-bit hash — replaces Python's process-salted
/// builtin `hash()` so antibody ids are reproducible across runs.
/// // MANUAL: divergence documented — format is identical
/// (`DDR-<TYPE>-<04x>`), entropy source differs by design.
pub fn fnv1a16(data: &str) -> u16 {
    let mut h: u32 = 0x811C_9DC5;
    for b in data.as_bytes() {
        h ^= *b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    (h & 0xFFFF) as u16
}

/// A generated DDR antibody (Python dict fields).
#[derive(Debug, Clone)]
pub struct Antibody {
    pub antibody_id: String,
    pub kind: String,
    pub severity: String,
    pub pattern: String, // = vuln.line_content in Python
    pub file: String,
    pub line: u64,
    pub fix: String,
    pub fingerprint: String,
    pub timestamp: String,
    pub generated_by: String,
}

/// `SelfFuzzingEngine.generate_antibody(vuln)`.
/// `timestamp`: Python reads the wall clock (`time.strftime`) → caller-supplied.
pub fn generate_antibody(vuln: &Vulnerability, timestamp: &str) -> Antibody {
    let antibody_id = format!(
        "DDR-{}-{:04x}",
        vuln.antibody_type.to_uppercase(),
        fnv1a16(&vuln.fingerprint)
    );
    Antibody {
        antibody_id,
        kind: vuln.antibody_type.clone(),
        severity: vuln.severity.clone(),
        pattern: vuln.line_content.clone(),
        file: vuln.file.clone(),
        line: vuln.line,
        fix: vuln.fix_suggestion.clone(),
        fingerprint: vuln.fingerprint.clone(),
        timestamp: timestamp.to_string(),
        generated_by: "Self-Fuzzing Engine v18.95".to_string(),
    }
}

// ─── Engine accumulators + summary ───────────────────────────

/// Aggregated scan stats (Python `self.stats` + scan_all return).
#[derive(Debug, Clone, Default)]
pub struct FuzzSummary {
    pub files_scanned: u64,
    pub lines_scanned: u64,
    pub vulnerabilities_found: u64,
    pub antibodies_generated: u64,
    /// by_severity in SEVERITIES order [CRITICAL, HIGH, MEDIUM, LOW].
    pub by_severity: [u64; 4],
    /// by_type in first-encounter order (Python dict insertion order).
    pub by_type: Vec<(String, u64)>,
}

/// Continuous self-fuzzing engine accumulator state.
#[derive(Debug, Clone, Default)]
pub struct SelfFuzzingEngine {
    pub vulnerabilities: Vec<Vulnerability>,
    pub antibodies: Vec<Antibody>,
    pub stats: FuzzSummary,
}

impl SelfFuzzingEngine {
    pub fn new() -> Self {
        SelfFuzzingEngine::default()
    }

    /// Scan one file's content, record findings + generate an antibody per
    /// vulnerability (mirrors the per-file body of `scan_all`).
    /// // MANUAL: filesystem walking, ThreadPoolExecutor fan-out and
    /// `save_antibodies` JSON vault IO are host-bound.
    pub fn scan_file_content(&mut self, file: &str, content: &str, timestamp: &str) -> u64 {
        let vulns = scan_content(content, file);
        let found = vulns.len() as u64;
        let lines = splitlines(content).len() as u64;
        self.stats.files_scanned += 1;
        self.stats.lines_scanned += lines;
        for v in vulns {
            self.stats.vulnerabilities_found += 1;
            self.stats.by_severity[severity_index(&v.severity)] += 1;
            match self.stats.by_type.iter_mut().find(|(t, _)| *t == v.vuln_type) {
                Some((_, c)) => *c += 1,
                None => self.stats.by_type.push((v.vuln_type.clone(), 1)),
            }
            let ab = generate_antibody(&v, timestamp);
            self.stats.antibodies_generated += 1;
            self.antibodies.push(ab);
            self.vulnerabilities.push(v);
        }
        found
    }

    pub fn summarize(&self) -> &FuzzSummary {
        &self.stats
    }
}
