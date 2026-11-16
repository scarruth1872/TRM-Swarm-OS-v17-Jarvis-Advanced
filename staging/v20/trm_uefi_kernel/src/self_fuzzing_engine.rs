//! TRM Swarm OS — Continuous Self-Fuzzing Engine (no_std).
//! Authentic bare-metal port of swarm_v2/core/self_fuzzing_engine.py.
//!
//! Monad Truth Anchor: 0x4D4F4E4144000001
//! Shield + Devo agent vulnerability audit and DDR threat antibody generator.

#![no_std]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const VOLTAGE_SCALE_FACTOR: f32 = 31.8712;
pub const VOLTAGE_PHASE: u32 = 17;
pub const VOLTAGE_RANK: u32 = 84;

/// Phase 17 Rank 84 Balanced Ternary Quantized Weights (64 trits, 2-bit packed).
pub static FUZZING_WEIGHTS_BT: [u8; 16] = [
    0x66, 0x99, 0xA5, 0x5A, 0x6A, 0x95, 0xA9, 0x56,
    0x96, 0x65, 0x55, 0xAA, 0x69, 0x96, 0xA6, 0x59,
];

/// Vulnerability severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Vulnerability report struct.
#[derive(Debug, Clone)]
pub struct Vulnerability {
    pub file: String,
    pub line: usize,
    pub line_content: String,
    pub vuln_type: String,
    pub severity: String,
    pub antibody_type: String,
    pub fix_suggestion: String,
    pub fingerprint: String,
}

impl Vulnerability {
    pub fn new() -> Self {
        Self {
            file: String::new(),
            line: 0,
            line_content: String::new(),
            vuln_type: String::new(),
            severity: String::new(),
            antibody_type: String::new(),
            fix_suggestion: String::new(),
            fingerprint: String::new(),
        }
    }
}

/// DDR Threat Antibody definition. Carries the fields the source's `generate_antibody` dict carries,
/// so the vault's dedup (by fingerprint) can key on the same value the source keys on.
#[derive(Debug, Clone)]
pub struct DdrAntibody {
    pub antibody_id: String,
    pub vuln_type: String,
    pub signature_hash: u64,
    pub action: String,
    pub quarantined: bool,
    pub severity: String,
    pub pattern: String,
    pub file: String,
    pub line: usize,
    pub fix: String,
    pub fingerprint: String,
}

// The source's 15 `VULN_PATTERNS` (swarm_v2/core/self_fuzzing_engine.py), in the dict's own order --
// the order the source emits per-line findings in. Each `matcher` is a predicate over a line that has
// ALREADY been lower-cased, reproducing that entry's `re.search(pattern, line, re.IGNORECASE)` by
// existence: the source asks "is there a match anywhere on this line", and for the shapes below an
// in-order literal scan answers exactly that. Two patterns contain `\n` and therefore can NEVER match
// inside the source's per-line loop: they are carried as explicit false predicates so their names and
// severities still count, with the reason stated rather than dropped.

pub struct VulnPatternRule {
    pub vuln_type: &'static str,
    pub severity: &'static str,
    pub antibody_type: &'static str,
    pub fix: &'static str,
    /// Predicate over a lower-cased line. See the per-rule comment for the regex it reproduces.
    pub matcher: fn(&str) -> bool,
}

const fn rule(
    vuln_type: &'static str,
    severity: &'static str,
    antibody_type: &'static str,
    fix: &'static str,
    matcher: fn(&str) -> bool,
) -> VulnPatternRule {
    VulnPatternRule { vuln_type, severity, antibody_type, fix, matcher }
}

pub static VULN_PATTERNS: [VulnPatternRule; 15] = [
    rule("SQL_INJECTION", "CRITICAL", "sql_injection", "Use parameterized queries with ? placeholders", m_sql),
    rule("COMMAND_INJECTION", "CRITICAL", "command_injection", "Use subprocess.run with shell=False and argument lists", m_cmd),
    rule("PATH_TRAVERSAL", "HIGH", "path_traversal", "Sanitize paths with os.path.realpath() validation", m_path),
    rule("HARDCODED_SECRET", "CRITICAL", "hardcoded_secret", "Use environment variables or a secrets manager", m_secret),
    rule("HARDCODED_TOKEN", "CRITICAL", "hardcoded_token", "Use os.environ.get() or a vault service", m_token),
    rule("INSECURE_DESERIALIZATION", "HIGH", "insecure_deserialization", "Use yaml.safe_load() or avoid pickle entirely", m_deser),
    rule("DEBUG_PRINT", "MEDIUM", "debug_leak", "Remove debug prints before production deployment", m_debug),
    rule("BARE_EXCEPT", "LOW", "bare_except", "Specify exception types: except (ValueError, KeyError) as e:", m_bare),
    rule("WEAK_CRYPTO", "HIGH", "weak_crypto", "Use hashlib.sha256 or secrets module for cryptographic operations", m_weak),
    rule("RACE_CONDITION", "MEDIUM", "race_condition", "Use atomic file operations with tempfile + os.replace()", m_never),
    rule("RESOURCE_LEAK", "LOW", "resource_leak", "Use context managers: with open(...) as f:", m_leak),
    rule("HTTP_NO_VERIFY", "HIGH", "ssl_bypass", "Remove verify=False in production; use proper SSL certificates", m_ssl),
    rule("UNSAFE_REDIRECT", "HIGH", "open_redirect", "Validate redirect URLs against a whitelist", m_redirect),
    rule("BUFFER_OVERFLOW", "CRITICAL", "buffer_overflow", "Use Python native types or bounds-checked ctypes alternatives", m_buffer),
    rule("USE_AFTER_FREE", "CRITICAL", "use_after_free", "Set deleted references to None", m_never),
];

/// The source's `A.*B` idiom: A exists, B exists later in the line. Using the FIRST A is exact here --
/// everything after it is a superset of what follows any later A.
fn seq(h: &str, parts: &[&str]) -> bool {
    let mut rest = h;
    for p in parts {
        match rest.find(p) {
            Some(i) => rest = &rest[i + p.len()..],
            None => return false,
        }
    }
    true
}

/// The source's `A.*(B|C|...)` idiom.
fn seq_any(h: &str, parts: &[&str], tails: &[&str]) -> bool {
    let mut rest = h;
    for p in parts {
        match rest.find(p) {
            Some(i) => rest = &rest[i + p.len()..],
            None => return false,
        }
    }
    tails.iter().any(|t| rest.contains(t))
}

/// `A\s*\(`: A, then nothing but whitespace, then an open paren.
fn call_open(h: &str, name: &str) -> bool {
    let mut from = 0usize;
    while let Some(rel) = h[from..].find(name) {
        let i = from + rel;
        if h[i + name.len()..].trim_start().starts_with('(') {
            return true;
        }
        from = i + name.len();
    }
    false
}

/// `name\s*=\s*value`.
fn assign_eq_value(h: &str, name: &str, value: &str) -> bool {
    let mut from = 0usize;
    while let Some(rel) = h[from..].find(name) {
        let i = from + rel;
        let r = h[i + name.len()..].trim_start();
        if let Some(t) = r.strip_prefix('=') {
            if t.trim_start().starts_with(value) {
                return true;
            }
        }
        from = i + name.len();
    }
    false
}

fn is_wordish(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '.'
}

/// `(n1|n2|...)\s*=\s*["'][\w\-\.]{min,}["']`.
fn assignment_literal(h: &str, names: &[&str], min: usize) -> bool {
    for name in names {
        let mut from = 0usize;
        while let Some(rel) = h[from..].find(name) {
            let i = from + rel;
            let r = h[i + name.len()..].trim_start();
            if let Some(t) = r.strip_prefix('=') {
                let r = t.trim_start();
                let mut cs = r.chars();
                if matches!(cs.next(), Some('"') | Some('\'')) {
                    // Count by hand: take_while() consumes and discards the first non-matching char,
                    // which here IS the closing quote the rule must see.
                    let mut run = 0usize;
                    let mut close = None;
                    for c in cs.by_ref() {
                        if is_wordish(c) {
                            run += 1;
                        } else {
                            close = Some(c);
                            break;
                        }
                    }
                    if run >= min && matches!(close, Some('"') | Some('\'')) {
                        return true;
                    }
                }
            }
            from = i + name.len();
        }
    }
    false
}

// (execute|executemany)\s*\(.*[\+%].*\)  |  f["'].*\{.*\}.*["'].*execute  |  \.format\(.*\).*execute
fn m_sql(l: &str) -> bool {
    for name in ["execute", "executemany"] {
        let mut from = 0usize;
        while let Some(rel) = l[from..].find(name) {
            let i = from + rel;
            let rest = &l[i + name.len()..];
            if let Some(p) = rest.find('(') {
                let after = &rest[p..];
                if let Some(k) = after.find(['+', '%']) {
                    if after[k..].contains(')') {
                        return true;
                    }
                }
            }
            from = i + name.len();
        }
    }
    for q1 in ["f\"", "f'"] {
        for q2 in ["\"", "'"] {
            if seq(l, &[q1, "{", "}", q2, "execute"]) {
                return true;
            }
        }
    }
    seq(l, &[".format(", ")", "execute"])
}

// os\.system\(  |  subprocess\.(call|run|Popen)\(.*shell\s*=\s*True  |  eval\(  |  exec\(
fn m_cmd(l: &str) -> bool {
    if l.contains("os.system(") || l.contains("eval(") || l.contains("exec(") {
        return true;
    }
    for name in ["subprocess.call(", "subprocess.run(", "subprocess.popen("] {
        if let Some(i) = l.find(name) {
            if seq(&l[i..], &["shell", "=", "true"]) {
                return true;
            }
        }
    }
    false
}

// open\(.*\.\.\/  |  open\(.*os\.path\.join\(.*user_input  |  Path\(.*user_input
fn m_path(l: &str) -> bool {
    if seq(l, &["open(", "../"]) {
        return true;
    }
    if seq(l, &["open(", "os.path.join(", "user_input"]) {
        return true;
    }
    seq(l, &["path(", "user_input"])
}

// (api_key|secret|password|token|private_key)\s*=\s*["'][\w\-\.]{8,}["']
fn m_secret(l: &str) -> bool {
    assignment_literal(l, &["api_key", "secret", "password", "token", "private_key"], 8)
}

// (github_token|aws_key|azure_key|jwt_secret)\s*=\s*["'][\w\-\.]{10,}["']
fn m_token(l: &str) -> bool {
    assignment_literal(l, &["github_token", "aws_key", "azure_key", "jwt_secret"], 10)
}

// pickle\.(loads|load)\s*\(  |  yaml\.load\s*\(  |  marshal\.loads\(
fn m_deser(l: &str) -> bool {
    call_open(l, "pickle.loads") || call_open(l, "pickle.load") || call_open(l, "yaml.load") || call_open(l, "marshal.loads")
}

// print\(.*(secret|password|token|key|private)
fn m_debug(l: &str) -> bool {
    seq_any(l, &["print("], &["secret", "password", "token", "key", "private"])
}

// except\s*:
fn m_bare(l: &str) -> bool {
    let mut from = 0usize;
    while let Some(rel) = l[from..].find("except") {
        let i = from + rel;
        if l[i + 6..].trim_start().starts_with(':') {
            return true;
        }
        from = i + 6;
    }
    false
}

// hashlib\.(md5|sha1)\s*\(  |  random\.(random|randint)\s*\(.*(password|token|crypto|auth)
fn m_weak(l: &str) -> bool {
    if call_open(l, "hashlib.md5") || call_open(l, "hashlib.sha1") {
        return true;
    }
    for name in ["random.random", "random.randint"] {
        if let Some(i) = l.find(name) {
            let rest = &l[i..];
            if call_open(rest, name) && ["password", "token", "crypto", "auth"].iter().any(|t| rest.contains(t)) {
                return true;
            }
        }
    }
    false
}

// The source's RACE_CONDITION and USE_AFTER_FREE patterns both contain `\n`, and the source searches
// each LINE with re.search: a pattern containing a newline cannot match a line, so these two can never
// fire in the source's own loop. Carried as explicit false predicates rather than dropped, so their
// names, severities and the reason they never fire are all visible at the route.
fn m_never(_l: &str) -> bool {
    false
}

// open\([^)]*\)\s*$(?!.*\.close|.*with) -- at end of line the negative lookahead sees nothing, so it
// holds; what the pattern actually asks for is a line ENDING with `open(...)` plus only whitespace.
fn m_leak(l: &str) -> bool {
    let mut from = 0usize;
    while let Some(rel) = l[from..].find("open(") {
        let i = from + rel;
        let rest = &l[i + 5..];
        if let Some(close) = rest.find(')') {
            if rest[close + 1..].trim().is_empty() {
                return true;
            }
        }
        from = i + 5;
    }
    false
}

// verify\s*=\s*False  |  ssl\s*=\s*False  |  requests\.get\([^)]*verify\s*=\s*False
fn m_ssl(l: &str) -> bool {
    if assign_eq_value(l, "verify", "false") || assign_eq_value(l, "ssl", "false") {
        return true;
    }
    if let Some(i) = l.find("requests.get(") {
        let rest = &l[i + 13..];
        if let Some(close) = rest.find(')') {
            if assign_eq_value(&rest[..close], "verify", "false") {
                return true;
            }
        }
    }
    false
}

// redirect\(.*request\.(args|form|json|data)
fn m_redirect(l: &str) -> bool {
    seq_any(l, &["redirect("], &["request.args", "request.form", "request.json", "request.data"])
}

// ctypes\.(string_at|memmove|memset|memcpy)\(
fn m_buffer(l: &str) -> bool {
    ["ctypes.string_at(", "ctypes.memmove(", "ctypes.memset(", "ctypes.memcpy("]
        .iter()
        .any(|n| l.contains(n))
}

/// Continuous Self-Fuzzing Engine.
pub struct SelfFuzzingEngine {
    pub scanned_lines: u64,
    pub vulnerabilities_found: u64,
    pub antibodies_generated: u64,
}

impl SelfFuzzingEngine {
    pub fn new() -> Self {
        Self {
            scanned_lines: 0,
            vulnerabilities_found: 0,
            antibodies_generated: 0,
        }
    }

    pub fn scan_line(&mut self, file: &str, line_no: usize, text: &str) -> Vec<Vulnerability> {
        self.scanned_lines += 1;
        let mut found = Vec::new();

        let lower = text.to_lowercase();
        for rule in &VULN_PATTERNS {
            if (rule.matcher)(&lower) {
                self.vulnerabilities_found += 1;
                // Simple FNV-1a hash fingerprint
                let mut hash: u64 = 0xcbf29ce484222325;
                for b in text.as_bytes() {
                    hash ^= *b as u64;
                    hash = hash.wrapping_mul(0x100000001b3);
                }

                let mut fp = String::new();
                let hex_chars = b"0123456789abcdef";
                for shift in (0..16).rev() {
                    let nibble = ((hash >> (shift * 4)) & 0xF) as usize;
                    fp.push(hex_chars[nibble] as char);
                }

                found.push(Vulnerability {
                    file: String::from(file),
                    line: line_no,
                    line_content: String::from(text),
                    vuln_type: String::from(rule.vuln_type),
                    severity: String::from(rule.severity),
                    antibody_type: String::from(rule.antibody_type),
                    fix_suggestion: String::from(rule.fix),
                    fingerprint: fp,
                });
            }
        }
        found
    }

    pub fn create_antibody(&mut self, vuln: &Vulnerability) -> DdrAntibody {
        self.antibodies_generated += 1;
        let mut sig_hash: u64 = 0x811c9dc5;
        for b in vuln.line_content.as_bytes() {
            sig_hash ^= *b as u64;
            sig_hash = sig_hash.wrapping_mul(0x01000193);
        }

        let mut aid = String::from("ddr_");
        aid.push_str(&vuln.antibody_type);
        aid.push_str("_");
        aid.push_str(&vuln.fingerprint[..6.min(vuln.fingerprint.len())]);

        DdrAntibody {
            antibody_id: aid,
            vuln_type: vuln.vuln_type.clone(),
            signature_hash: sig_hash,
            action: String::from("DROP_AND_QUARANTINE"),
            quarantined: true,
            severity: vuln.severity.clone(),
            pattern: vuln.line_content.clone(),
            file: vuln.file.clone(),
            line: vuln.line,
            fix: vuln.fix_suggestion.clone(),
            fingerprint: vuln.fingerprint.clone(),
        }
    }

    // --- The source's file-scan surface, ported rule-for-rule ---

    /// The source's `scan_file` scanning step, given content the CALLER read. File I/O stays
    /// host-side (the source's `read_text` has no `no_std` counterpart); everything the source then
    /// does with the text is here: per line, per rule, a case-insensitive containment test -- both
    /// sides folded, because the source's `re.IGNORECASE` folds the pattern too.
    ///
    /// `fingerprint` is the source's `f"{filepath.name}:{line_num}:{vuln_name}"` -- the BASENAME, not
    /// the whole path. `line_content` is the source's `line.strip()[:120]`, 120 CHARACTERS, so a
    /// multi-byte line truncates where the source truncates it and not at a byte boundary.
    pub fn scan_text(&mut self, file: &str, content: &str) -> Vec<Vulnerability> {
        let base = basename(file);
        let mut found = Vec::new();
        for (idx, raw) in content.lines().enumerate() {
            let line_no = idx + 1;
            self.scanned_lines += 1;
            let hay = raw.to_lowercase();
            for rule in VULN_PATTERNS.iter() {
                if (rule.matcher)(&hay) {
                    self.vulnerabilities_found += 1;
                    let trimmed = raw.trim();
                    let snippet: String = trimmed.chars().take(120).collect();
                    found.push(Vulnerability {
                        file: String::from(file),
                        line: line_no,
                        line_content: snippet,
                        vuln_type: String::from(rule.vuln_type),
                        severity: String::from(rule.severity),
                        antibody_type: String::from(rule.antibody_type),
                        fix_suggestion: String::from(rule.fix),
                        fingerprint: format!("{}:{}:{}", base, line_no, rule.vuln_type),
                    });
                }
            }
        }
        found
    }

    /// The source's `scan_lines_native` per-line code (0 Safe, 1 SQLi, 2 Command Injection, 3 Secret,
    /// 4 Deserialization, 5 Bare except, 6 SSL bypass). Ported with the source's PRIORITY: its chain
    /// is `if/elif`, so a line that is both a SQLi and a command injection reports 1 -- the first
    /// match wins and the rest are never tested.
    pub fn scan_line_code(line: &str) -> u8 {
        // ASCII-only folding, which is what the strip's `contains_ci` does: a Kelvin sign does not
        // become a `k` here, and the ruler pins that against the strip's own rule.
        let ll = line.to_ascii_lowercase();
        if ll.contains("execute") && (ll.contains('%') || ll.contains("format(")) {
            return 1;
        }
        if ll.contains("os.system(") || ll.contains("subprocess.") {
            return 2;
        }
        if ll.contains("api_key") || ll.contains("password") {
            return 3;
        }
        if ll.contains("pickle.loads") || ll.contains("yaml.load") {
            return 4;
        }
        if ll.contains("except:") {
            return 5;
        }
        if ll.contains("verify=false") || ll.contains("ssl=false") {
            return 6;
        }
        0
    }

    /// The source's `scan_all` counters, over files the caller supplied. The source walks its own
    /// directories and threads the walk; the walking is host-side I/O and stays there. Its `time_s`
    /// and `throughput` are MEASURED in the source and are not produced here: the module has no clock
    /// and will not put a number where a measurement belongs.
    pub fn scan_all(&mut self, files: &[(String, String)]) -> ScanSummary {
        let mut by_severity: BTreeMap<String, u64> = BTreeMap::new();
        let mut by_type: BTreeMap<String, u64> = BTreeMap::new();
        for (path, content) in files {
            let hits = self.scan_text(path, content);
            for v in &hits {
                *by_severity.entry(v.severity.clone()).or_insert(0) += 1;
                *by_type.entry(v.vuln_type.clone()).or_insert(0) += 1;
            }
        }
        ScanSummary {
            files_scanned: files.len() as u64,
            lines_scanned: self.scanned_lines,
            vulnerabilities: self.vulnerabilities_found,
            antibodies_generated: self.vulnerabilities_found,
            by_severity: by_severity.into_iter().collect(),
            by_type: by_type.into_iter().collect(),
        }
    }

    /// The source's `save_antibodies` dedup rule: append only the antibodies whose fingerprint is not
    /// already in the vault, and report previous / new / total. The read and the write stay
    /// host-side; the arithmetic and the rule are here.
    pub fn save_plan(&self, existing_fingerprints: &[String], antibodies: &[DdrAntibody]) -> AntibodySavePlan {
        let appended: Vec<DdrAntibody> = antibodies
            .iter()
            .filter(|a| !existing_fingerprints.iter().any(|f| f == &a.fingerprint))
            .cloned()
            .collect();
        let new = appended.len() as u64;
        let previous = existing_fingerprints.len() as u64;
        AntibodySavePlan {
            previous,
            new,
            total: previous + new,
            appended,
        }
    }
}

/// The source's file-name component (`filepath.name`), on both separators so a Windows path and a
/// POSIX path yield the same fingerprint.
pub fn basename(path: &str) -> String {
    let tail = match path.rfind(['/', '\\']) {
        Some(i) => &path[i + 1..],
        None => path,
    };
    String::from(tail)
}

/// The source's `scan_all` return dict, minus the two fields it times.
#[derive(Debug, Clone)]
pub struct ScanSummary {
    pub files_scanned: u64,
    pub lines_scanned: u64,
    pub vulnerabilities: u64,
    pub antibodies_generated: u64,
    pub by_severity: Vec<(String, u64)>,
    pub by_type: Vec<(String, u64)>,
}

/// The source's `save_antibodies` return dict, plus the list it appends. The count and the list are
/// built from ONE filter: reporting `new` from a filter and the fingerprints from a different
/// collection is how a payload starts contradicting itself.
#[derive(Debug, Clone)]
pub struct AntibodySavePlan {
    pub previous: u64,
    pub new: u64,
    pub total: u64,
    pub appended: Vec<DdrAntibody>,
}

static SINGLETON_SELF_FUZZING_ENGINE: AtomicU64 = AtomicU64::new(0);

/// Global handle initialization.
pub fn get_self_fuzzing_engine() -> u64 {
    let v = SINGLETON_SELF_FUZZING_ENGINE.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SELF_FUZZING_ENGINE.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// Query self-fuzzing engine telemetry.
pub fn self_fuzzing_telemetry() -> (u64, u32, u32, f32) {
    (MONAD_ANCHOR, VOLTAGE_PHASE, VOLTAGE_RANK, VOLTAGE_SCALE_FACTOR)
}
