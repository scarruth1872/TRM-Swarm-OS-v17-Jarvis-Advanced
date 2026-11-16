//! TRM Swarm OS — DDR Antibody Engine (no_std port).
//! 1:1 from swarm_v2/core/ddr_antibody.py: the Digital DNA Repository.
//! Security threats -> antibodies (pattern + fix + severity) -> scan code ->
//! matches prevented. The 8-stage threat-response pipeline:
//!   1 detect threat -> 2 extract pattern -> 3 record antibody ->
//!   4 persist -> 5 scan code -> 6 match -> 7 prevent (increment count) ->
//!   8 report stats.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Severity levels (1:1 with python low/medium/high/critical).
pub const SEV_LOW: u64 = 0;
pub const SEV_MEDIUM: u64 = 1;
pub const SEV_HIGH: u64 = 2;
pub const SEV_CRITICAL: u64 = 3;

/// A DDR antibody: error type + patterns + fix + severity + prevention count.
#[derive(Debug, Clone)]
pub struct Antibody {
    pub antibody_id: u64,
    pub error_type: String,
    pub file_pattern: String,
    pub line_pattern: String,
    pub fix_description: String,
    pub severity: u64,
    pub matches_prevented: u64,
}

impl Antibody {
    pub fn new(id: u64, error_type: &str, file_pattern: &str, line_pattern: &str,
               fix: &str, severity: u64) -> Self {
        Antibody {
            antibody_id: id,
            error_type: error_type.into(),
            file_pattern: file_pattern.into(),
            line_pattern: line_pattern.into(),
            fix_description: fix.into(),
            severity,
            matches_prevented: 0,
        }
    }
}

/// The Digital DNA Repository (in-memory antibody bank).
#[derive(Debug, Clone)]
pub struct DigitalDnaRepository {
    pub antibodies: Vec<Antibody>,
    next_id: u64,
}

impl DigitalDnaRepository {
    pub fn new() -> Self {
        DigitalDnaRepository {
            antibodies: Vec::new(),
            next_id: 1,
        }
    }

    /// Record an antibody from a detected threat (python record_antibody).
    pub fn record_antibody(&mut self, error_type: &str, file_pattern: &str,
                           line_pattern: &str, fix: &str, severity: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.antibodies.push(Antibody::new(
            id, error_type, file_pattern, line_pattern, fix, severity,
        ));
        id
    }

    /// Auto-inject from threat detection, with CoT trace appended (1:1
    /// python record_threat_antibody: fix + "[CoT Trace: ...]").
    pub fn record_threat_antibody(&mut self, threat_type: &str, file_pattern: &str,
                                  line_pattern: &str, fix_patch: &str,
                                  cot_trace: &str) -> u64 {
        let full_fix = if cot_trace.is_empty() {
            fix_patch.to_string()
        } else {
            let cap = cot_trace.len().min(120);
            format!("{} [CoT Trace: {}]", fix_patch, &cot_trace[..cap])
        };
        self.record_antibody(threat_type, file_pattern, line_pattern, &full_fix, SEV_HIGH)
    }

    /// Simple substring pattern match (no_std regex substitute — the kernel
    /// has no regex engine; python uses re.search, we match the pattern
    /// string as a substring, which covers the fixed-pattern antibodies).
    /// Returns match details (1:1 python check_antibodies shape).
    pub fn check_antibodies(&mut self, code: &str, filename: &str) -> Vec<String> {
        let mut matches: Vec<String> = Vec::new();
        for ab in self.antibodies.iter_mut() {
            // file pattern match (simple suffix/contains on filename)
            if !ab.file_pattern.is_empty() && !filename.is_empty() {
                if !filename.ends_with(ab.file_pattern.trim_start_matches("*.")) {
                    continue;
                }
            }
            // line pattern: substring search on the pattern token
            // (strip regex metacharacters for the fixed default antibodies)
            let pat = strip_regex(&ab.line_pattern);
            if !pat.is_empty() && code.contains(&pat) {
                ab.matches_prevented += 1;
                matches.push(format!(
                    "{}:{} severity={} fix={}",
                    ab.error_type, ab.antibody_id, ab.severity, ab.fix_description
                ));
            }
        }
        matches
    }

    /// Quick check: list matched error types (python match_patterns).
    pub fn match_patterns(&mut self, code: &str) -> Vec<String> {
        let mut out = Vec::new();
        for m in self.check_antibodies(code, "") {
            let t: String = m.chars().take_while(|c| *c != ':').collect();
            out.push(t);
        }
        out
    }

    /// Prevention statistics (python get_prevention_stats).
    pub fn get_prevention_stats(&self) -> (u64, u64, [u64; 4]) {
        let total = self.antibodies.len() as u64;
        let prevented: u64 = self.antibodies.iter().map(|a| a.matches_prevented).sum();
        let mut sev = [0u64; 4];
        for a in &self.antibodies {
            let idx = (a.severity.min(3)) as usize;
            sev[idx] += 1;
        }
        (total, prevented, sev)
    }

    /// Install the 6 default antibodies (1:1 python install_default_antibodies).
    pub fn install_default_antibodies(&mut self) -> u64 {
        let defaults: [(&str, &str, &str, &str, u64); 6] = [
            ("sql_injection", "*.py", "SELECT", 
             "Use parameterized queries instead of f-strings in SQL", SEV_CRITICAL),
            ("hardcoded_secret", "*.py", "SECRET_KEY",
             "Move secrets to SecretsVault or environment variables", SEV_HIGH),
            ("eval_usage", "*.py", "eval(",
             "Replace eval() with ast.literal_eval() or structured parsing", SEV_HIGH),
            ("debug_print", "*.py", "print(",
             "Replace debug print() with logger.debug()", SEV_LOW),
            ("bare_except", "*.py", "except:",
             "Catch specific exceptions instead of bare except", SEV_MEDIUM),
            ("path_traversal", "*.py", "../..",
             "Validate paths against allowlist; use os.path.abspath()", SEV_HIGH),
        ];
        for (error_type, file_pat, line_pat, fix, severity) in defaults {
            let dup = self.antibodies.iter().any(|a| a.error_type == error_type);
            if !dup {
                let id = self.next_id;
                self.next_id += 1;
                self.antibodies.push(Antibody::new(
                    id, error_type, file_pat, line_pat, fix, severity,
                ));
            }
        }
        self.antibodies.len() as u64
    }
}

/// Strip common regex metacharacters so fixed-pattern antibodies match as
/// substrings (the kernel has no regex engine; the 6 defaults are literal).
fn strip_regex(pat: &str) -> String {
    let mut out = String::new();
    for c in pat.chars() {
        match c {
            '\\' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '*' | '+' | '?' | '.' | '|' => {}
            _ => out.push(c),
        }
    }
    out
}

/// The 8-stage threat response pipeline. Returns (matched_count, prevented_now).
pub fn run_pipeline(code: &str) -> (u64, u64) {
    let mut repo = DigitalDnaRepository::new();
    repo.install_default_antibodies();
    // stages 1-4: record a fresh threat antibody
    repo.record_threat_antibody(
        "injection_attempt",
        "*.py",
        "shell=True",
        "Use parameterized execution; never shell=True with user input",
        "Instella detected user-controlled shell execution",
    );
    // stages 5-7: scan + match + prevent
    let matches = repo.check_antibodies(code, "target.py");
    let (total, prevented, _) = repo.get_prevention_stats();
    let _ = total;
    (matches.len() as u64, prevented)
}
