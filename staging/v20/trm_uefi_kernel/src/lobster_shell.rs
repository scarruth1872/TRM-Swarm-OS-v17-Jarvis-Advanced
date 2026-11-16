//! TRM Swarm OS - lobster_shell (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/lobster_shell.py.
//! Functions marked // MANUAL need human verification of math semantics.
//!
//! ORACLE = swarm_v2/core/lobster_shell.py. The native strip
//! (swarm_v2_artifacts/native_voltage_strips/lobster_shell.rs /
//! strip_lobster_shell.dll) is the GENERIC ternary-dot template
//! (lobster_shell_execute / _monad_commit / _phase_info) and contains NONE of
//! this module's rules, so it is not usable as an oracle.
//!
//! HOST-SIDE, deliberately NOT ported (no_std kernel has no OS):
//!   * subprocess.run(...) command execution inside audit_and_execute
//!     (only the Shell-01 validation half is ported),
//!   * ApprovalGate (JSON queue file read/write + os.makedirs + polling),
//!   * _log_violation (append to NEURAL_AUDIT_LOG.md),
//!   * os.getcwd() resolution of a relative cwd / abspath of a relative path,
//!   * asyncio run_pipeline + `await self.execute_tool(...)` (executor is a
//!     host callable; only its type is mirrored here).
//!
//! std-only helpers are HAND-ROLLED (no_std): ntpath.normpath/splitdrive/split
//! subset, Python int(), Python json.loads(). Known limits are documented at
//! each site.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

//  CONSTANTS 
pub const PROJECT_ROOT: &'static str = r#"f:\Development sites\TRM agent swarm"#;
/// ALLOWED_COMMANDS = {"git","python","npm","pip","npx","ollama","node","ls",
/// "dir","mkdir","echo"} (PY set; this port keeps the source-file order so that
/// membership behaviour is identical and multi-hit messages are deterministic).
pub static ALLOWED_COMMANDS: &[&str] = &[
    "git", "python", "npm", "pip", "npx", "ollama", "node", "ls", "dir", "mkdir", "echo",
];
/// BLACKLISTED_PATHS = {"C:\Windows","C:\Users","C:\Program Files","/etc","/usr","/root"}
/// (PY set iteration order is hash-randomised per process; the port uses
/// source-file order, so a command matching 2+ entries yields a deterministic
/// message instead of a per-process varying one.)
pub static BLACKLISTED_PATHS: &[&str] = &[
    r"C:\Windows", r"C:\Users", r"C:\Program Files", "/etc", "/usr", "/root",
];
pub const AUDIT_LOG_PATH: &'static str = r#"f:\Development sites\TRM agent swarm\swarm_v2_artifacts\NEURAL_AUDIT_LOG.md"#;
/// os.path.join(os.path.dirname(__file__), "..", "..", "swarm_v2_artifacts",
/// "approval_queue.json") - dirname(__file__) is the host module dir
/// ("swarm_v2/core"), which a no_std kernel does not have; the relative tail is
/// preserved verbatim (os.path.join does not normalise).
pub const APPROVAL_QUEUE_PATH: &'static str =
    r#"swarm_v2\core\..\..\swarm_v2_artifacts\approval_queue.json"#;

//  typed value (mirrors Python Any) ------------------------------------------
/// Domain of `_apply_transform`: Python `Any` where Dict|List|str are the types
/// the oracle discriminate on. Dict keeps Python dict semantics (insertion
/// order, last duplicate key wins).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Dict(Vec<(String, Value)>),
}

impl Value {
    /// Python dict.get(key) -> value | None
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Dict(d) => d.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn is_dict(&self) -> bool {
        matches!(self, Value::Dict(_))
    }
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }
    pub fn is_str(&self) -> bool {
        matches!(self, Value::Str(_))
    }
}

//  LobsterStep 
#[derive(Debug, Clone)]
pub struct LobsterStep {
    pub tool: String,
    pub args: Vec<(String, Value)>,
    pub gate: Option<String>,
    pub transform: Option<String>,
}

impl LobsterStep {
    pub fn new() -> Self {
        LobsterStep {
            tool: String::new(),
            args: vec![],
            gate: None,
            transform: None,
        }
    }
}

//  LobsterPipeline 
#[derive(Debug, Clone)]
pub struct LobsterPipeline {
    pub name: String,
    pub steps: Vec<LobsterStep>,
}

impl LobsterPipeline {
    pub fn new() -> Self {
        LobsterPipeline {
            name: String::new(),
            steps: vec![],
        }
    }
}

//  hand-rolled ntpath subset (std-only math, no_std) --------------------------

#[inline]
fn is_sep(c: char) -> bool {
    c == '\\' || c == '/'
}

/// ntpath.splitdrive subset: drive letter (X:) or UNC/device (\\host\share).
/// Always drivespec + pathspec == p. Char-indexed, mirroring the Python.
pub fn nt_splitdrive(p: &str) -> (String, String) {
    let c: Vec<char> = p.chars().collect();
    if c.len() >= 2 {
        let normp: Vec<char> = c.iter().map(|&ch| if ch == '/' { '\\' } else { ch }).collect();
        if normp[0] == '\\' && normp[1] == '\\' {
            let head8: String = normp[..8.min(normp.len())].iter().collect();
            let start = if head8.to_uppercase() == r"\\?\UNC\" { 8 } else { 2 };
            let find = |from: usize| -> Option<usize> {
                let mut i = from;
                while i < normp.len() {
                    if normp[i] == '\\' {
                        return Some(i);
                    }
                    i += 1;
                }
                None
            };
            let i1 = match find(start) {
                Some(v) => v,
                None => return (p.to_string(), String::new()),
            };
            let i2 = match find(i1 + 1) {
                Some(v) => v,
                None => return (p.to_string(), String::new()),
            };
            return (c[..i2].iter().collect(), c[i2..].iter().collect());
        }
        if normp[1] == ':' {
            return (c[..2].iter().collect(), c[2..].iter().collect());
        }
    }
    (String::new(), p.to_string())
}

/// ntpath.split: (head, tail); tail is everything after the final separator.
pub fn nt_split(p: &str) -> (String, String) {
    let (d, rest) = nt_splitdrive(p);
    let pc: Vec<char> = rest.chars().collect();
    let mut i = pc.len();
    while i > 0 && !is_sep(pc[i - 1]) {
        i -= 1;
    }
    let head_raw: String = pc[..i].iter().collect();
    let tail: String = pc[i..].iter().collect();
    let trimmed: String = {
        let t: String = head_raw.trim_end_matches(is_sep).to_string();
        if t.is_empty() {
            head_raw.clone()
        } else {
            t
        }
    };
    (format!("{}{}", d, trimmed), tail)
}

/// ntpath.basename(p) == ntpath.split(p)[1]
pub fn nt_basename(p: &str) -> String {
    nt_split(p).1
}

/// ntpath.normpath: port of the documented pure-Python implementation
/// (CPython Lib/ntpath.py). No filesystem access, no host cwd.
pub fn nt_normpath(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let replaced: String = path.chars().map(|c| if c == '/' { '\\' } else { c }).collect();
    let (mut prefix, rest) = nt_splitdrive(&replaced);
    let mut rest = rest;
    if rest.starts_with('\\') {
        prefix.push('\\');
        rest = rest.trim_start_matches('\\').to_string();
    }
    let mut comps: Vec<String> = rest.split('\\').map(|s| s.to_string()).collect();
    let mut i = 0usize;
    while i < comps.len() {
        if comps[i].is_empty() || comps[i] == "." {
            comps.remove(i);
        } else if comps[i] == ".." {
            if i > 0 && comps[i - 1] != ".." {
                comps.remove(i - 1);
                comps.remove(i - 1);
                i -= 1;
            } else if i == 0 && prefix.ends_with('\\') {
                comps.remove(i);
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    if prefix.is_empty() && comps.is_empty() {
        comps.push(".".to_string());
    }
    format!("{}{}", prefix, comps.join("\\"))
}

/// ntpath.isabs: CPython 3.11 port (s = s[:3].replace(altsep, sep);
/// s.startswith(sep) or s.startswith(':\\\\', 1)). Note the documented LEGACY
/// BUG kept for parity: isabs("/x") is True although there is no drive.
pub fn nt_isabs(p: &str) -> bool {
    let head: String = p.chars().take(3).collect();
    let s: String = head.chars().map(|c| if c == '/' { '\\' } else { c }).collect();
    if s.starts_with('\\') {
        return true;
    }
    s.chars().skip(1).collect::<String>().starts_with(":\\")
}

/// ntpath.join (CPython Lib/ntpath.py) for exactly one extra path component:
/// drive-relative handling, same/different-drive rules and the UNC separator
/// rule are all preserved.
pub fn nt_join(a: &str, b: &str) -> String {
    let (mut result_drive, mut result_path) = nt_splitdrive(a);
    let (p_drive, p_path) = nt_splitdrive(b);
    let pc: Vec<char> = p_path.chars().collect();
    if !pc.is_empty() && is_sep(pc[0]) {
        // Second path is absolute
        if !p_drive.is_empty() || result_drive.is_empty() {
            result_drive = p_drive.clone();
        }
        result_path = p_path;
    } else {
        if !p_drive.is_empty() && p_drive != result_drive {
            if p_drive.to_lowercase() != result_drive.to_lowercase() {
                // Different drives => ignore the first path entirely
                return format!("{}{}", p_drive, p_path);
            }
            result_drive = p_drive.clone(); // same drive, different case
        }
        let rpc: Vec<char> = result_path.chars().collect();
        if !rpc.is_empty() && !is_sep(rpc[rpc.len() - 1]) {
            result_path.push('\\');
        }
        result_path.push_str(&p_path);
    }
    // add separator between UNC and non-absolute path
    let rpc: Vec<char> = result_path.chars().collect();
    if !rpc.is_empty() && !is_sep(rpc[0]) && !result_drive.is_empty() && !result_drive.ends_with(':')
    {
        return format!("{}\\{}", result_drive, result_path);
    }
    format!("{}{}", result_drive, result_path)
}

/// Drive of a process cwd ("C:\\x" -> "C:"); None for UNC/relative/empty.
/// Windows keeps the process "current drive" equal to the drive of the current
/// directory (os.chdir changes both), so this stands in for it.
fn current_drive(cwd: &str) -> Option<String> {
    let c: Vec<char> = cwd.chars().collect();
    if c.len() >= 2 && c[1] == ':' {
        Some(c[..2].iter().collect())
    } else {
        None
    }
}

/// os.path.abspath for ABSOLUTE input on Windows:
/// nt._getfullpathname(ntpath.normpath(p)). A relative input needs os.getcwd()
/// (host state) - use `abspath_from` when the caller can supply it.
pub fn abspath(path: &str) -> String {
    nt_normpath(path)
}

/// os.path.abspath with the host cwd supplied explicitly (os.getcwd() stand-in).
/// Root-relative input (single leading separator, e.g. "/etc", "\\usr") is
/// resolved against the process's current drive, exactly like
/// GetFullPathNameW; with an empty `process_cwd` that drive is unknown (no_std
/// host dependency) and the normalized root-relative path is returned instead.
pub fn abspath_from(path: &str, process_cwd: &str) -> String {
    let n = nt_normpath(path);
    let single_root = (path.starts_with('\\') || path.starts_with('/'))
        && !path.starts_with("\\\\")
        && !path.starts_with("//");
    if single_root {
        return match current_drive(process_cwd) {
            Some(d) => format!("{}{}", d, n),
            None => n,
        };
    }
    if nt_isabs(path) {
        n
    } else {
        nt_normpath(&nt_join(process_cwd, path))
    }
}

/// Python int(str) subset (hand-rolled: Rust's i64::from_str rejects
/// surrounding whitespace and digit separators). Accepts optional surrounding
/// whitespace, one leading +/-, and single '_' separators between digits.
/// Limits: Unicode decimal digits and arbitrary precision are NOT supported.
fn py_int(s: &str) -> Option<i64> {
    let t = s.trim();
    let (neg, body) = match t.chars().next() {
        Some('+') => (false, &t[1..]),
        Some('-') => (true, &t[1..]),
        _ => (false, t),
    };
    if body.is_empty() {
        return None;
    }
    let mut digits = String::new();
    let mut prev_digit = false;
    for (idx, ch) in body.chars().enumerate() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            prev_digit = true;
        } else if ch == '_' && prev_digit && idx + 1 < body.len()
            && body.as_bytes()[idx + 1].is_ascii_digit()
        {
            prev_digit = false;
        } else {
            return None;
        }
    }
    if !prev_digit {
        return None;
    }
    match digits.parse::<i64>() {
        Ok(v) => Some(if neg { -v } else { v }),
        Err(_) => None,
    }
}

//  hand-rolled json.loads subset (std-only math, no_std) ---------------------

/// json.loads for the strict subset: object/array/string/number/true/false/null
/// (+ Python's non-standard NaN / Infinity / -Infinity). Int literals stay Int,
/// literals with '.' or an exponent become Float, duplicate object keys are
/// last-wins (Python dict). Limits: arbitrary-precision ints overflow to an
/// error (no bignum in no_std), lone surrogates are rejected (no Rust char),
/// big floats saturate to inf exactly like float().
pub fn json_loads_py(s: &str) -> Option<Value> {
    let mut p = JParser { c: s.chars().collect(), i: 0 };
    let v = p.parse_value()?;
    p.ws();
    if p.i != p.c.len() {
        return None;
    }
    Some(v)
}

struct JParser {
    c: Vec<char>,
    i: usize,
}

impl JParser {
    fn ws(&mut self) {
        while self.i < self.c.len() && matches!(self.c[self.i], ' ' | '\t' | '\n' | '\r') {
            self.i += 1;
        }
    }
    fn lit(&mut self, w: &str) -> Option<()> {
        for ch in w.chars() {
            if self.i >= self.c.len() || self.c[self.i] != ch {
                return None;
            }
            self.i += 1;
        }
        Some(())
    }
    fn looking_at(&self, w: &str) -> bool {
        let mut i = self.i;
        for ch in w.chars() {
            if i >= self.c.len() || self.c[i] != ch {
                return false;
            }
            i += 1;
        }
        true
    }
    fn parse_value(&mut self) -> Option<Value> {
        self.ws();
        if self.i >= self.c.len() {
            return None;
        }
        match self.c[self.i] {
            '{' => self.parse_obj(),
            '[' => self.parse_arr(),
            '"' => self.parse_str().map(Value::Str),
            't' => self.lit("true").map(|_| Value::Bool(true)),
            'f' => self.lit("false").map(|_| Value::Bool(false)),
            'n' => self.lit("null").map(|_| Value::Null),
            'N' => self.lit("NaN").map(|_| Value::Float(f64::NAN)),
            'I' => self.lit("Infinity").map(|_| Value::Float(f64::INFINITY)),
            '-' => {
                if self.looking_at("-Infinity") {
                    self.i += 9;
                    Some(Value::Float(f64::NEG_INFINITY))
                } else {
                    self.parse_num()
                }
            }
            c if c.is_ascii_digit() => self.parse_num(),
            _ => None,
        }
    }
    fn parse_num(&mut self) -> Option<Value> {
        let start = self.i;
        let neg = if self.i < self.c.len() && self.c[self.i] == '-' {
            self.i += 1;
            true
        } else {
            false
        };
        let ds = self.i;
        while self.i < self.c.len() && self.c[self.i].is_ascii_digit() {
            self.i += 1;
        }
        if self.i == ds {
            return None;
        }
        // Python's json rejects leading zeros ("01"), accepts "0"/"-0"/"0.5".
        if self.c[ds] == '0' && self.i - ds > 1 {
            return None;
        }
        let mut is_float = false;
        if self.i < self.c.len() && self.c[self.i] == '.' {
            self.i += 1;
            let fs = self.i;
            while self.i < self.c.len() && self.c[self.i].is_ascii_digit() {
                self.i += 1;
            }
            if self.i == fs {
                return None;
            }
            is_float = true;
        }
        if self.i < self.c.len() && (self.c[self.i] == 'e' || self.c[self.i] == 'E') {
            self.i += 1;
            if self.i < self.c.len() && (self.c[self.i] == '+' || self.c[self.i] == '-') {
                self.i += 1;
            }
            let es = self.i;
            while self.i < self.c.len() && self.c[self.i].is_ascii_digit() {
                self.i += 1;
            }
            if self.i == es {
                return None;
            }
            is_float = true;
        }
        let _ = neg;
        let lit: String = self.c[start..self.i].iter().collect();
        if is_float {
            lit.parse::<f64>().ok().map(Value::Float)
        } else {
            match lit.parse::<i64>() {
                Ok(v) => Some(Value::Int(v)),
                // Python yields an exact unbounded int here; no bignum in
                // no_std, so this is a documented divergence (parse error).
                Err(_) => None,
            }
        }
    }
    fn parse_str(&mut self) -> Option<String> {
        if self.i >= self.c.len() || self.c[self.i] != '"' {
            return None;
        }
        self.i += 1;
        let mut out = String::new();
        loop {
            if self.i >= self.c.len() {
                return None;
            }
            let ch = self.c[self.i];
            self.i += 1;
            match ch {
                '"' => return Some(out),
                '\\' => {
                    if self.i >= self.c.len() {
                        return None;
                    }
                    let e = self.c[self.i];
                    self.i += 1;
                    match e {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000C}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let hi = self.hex4()?;
                            if (0xD800..0xDC00).contains(&hi) {
                                // surrogate pair
                                if self.i + 1 < self.c.len() && self.c[self.i] == '\\'
                                    && self.c[self.i + 1] == 'u'
                                {
                                    self.i += 2;
                                    let lo = self.hex4()?;
                                    if (0xDC00..0xE000).contains(&lo) {
                                        let cp = 0x10000u32 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                                        out.push(char::from_u32(cp)?);
                                    } else {
                                        return None;
                                    }
                                } else {
                                    return None; // lone surrogate: no Rust char
                                }
                            } else if (0xDC00..0xE000).contains(&hi) {
                                return None; // lone low surrogate
                            } else {
                                out.push(char::from_u32(hi)?);
                            }
                        }
                        _ => return None,
                    }
                }
                c if (c as u32) < 0x20 => return None,
                c => out.push(c),
            }
        }
    }
    fn hex4(&mut self) -> Option<u32> {
        if self.i + 4 > self.c.len() {
            return None;
        }
        let mut v: u32 = 0;
        for _ in 0..4 {
            let d = self.c[self.i].to_digit(16)?;
            v = v * 16 + d;
            self.i += 1;
        }
        Some(v)
    }
    fn parse_arr(&mut self) -> Option<Value> {
        self.i += 1;
        let mut out: Vec<Value> = Vec::new();
        self.ws();
        if self.i < self.c.len() && self.c[self.i] == ']' {
            self.i += 1;
            return Some(Value::List(out));
        }
        loop {
            out.push(self.parse_value()?);
            self.ws();
            if self.i < self.c.len() && self.c[self.i] == ',' {
                self.i += 1;
                continue;
            }
            if self.i < self.c.len() && self.c[self.i] == ']' {
                self.i += 1;
                return Some(Value::List(out));
            }
            return None;
        }
    }
    fn parse_obj(&mut self) -> Option<Value> {
        self.i += 1;
        let mut out: Vec<(String, Value)> = Vec::new();
        self.ws();
        if self.i < self.c.len() && self.c[self.i] == '}' {
            self.i += 1;
            return Some(Value::Dict(out));
        }
        loop {
            self.ws();
            let k = self.parse_str()?;
            self.ws();
            if self.i >= self.c.len() || self.c[self.i] != ':' {
                return None;
            }
            self.i += 1;
            let v = self.parse_value()?;
            match out.iter_mut().find(|(ek, _)| ek == &k) {
                Some(e) => e.1 = v, // dict: last duplicate wins, position kept
                None => out.push((k, v)),
            }
            self.ws();
            if self.i < self.c.len() && self.c[self.i] == ',' {
                self.i += 1;
                continue;
            }
            if self.i < self.c.len() && self.c[self.i] == '}' {
                self.i += 1;
                return Some(Value::Dict(out));
            }
            return None;
        }
    }
}

//  LobsterShell.audit_and_execute (Shell-01 security middleware) -------------

/// Port of the VALIDATION half of LobsterShell.audit_and_execute.
/// Check order is the oracle's: (1) cwd inside abspath(PROJECT_ROOT),
/// (2) Shell-01 whitelist on basename(split()[0].lower()).replace(".exe","")
/// .replace(".ps1","") with a "."/"venv" prefix bypass, (3) blacklisted path
/// substring, case-insensitive. Returns (allowed, error_msg) with the oracle's
/// exact message text; on success the oracle returns (True, stdout) - the
/// subprocess/spawn is HOST-SIDE, so this returns (true, "").
/// Divergence: Python raises an uncaught IndexError on an empty/whitespace-only
/// command (split()[0]); here base_cmd is "" and the whitelist rule fires.
pub fn audit_and_execute(command: &str, cwd: &str) -> (bool, String) {
    audit_and_execute_from(command, cwd, "")
}

/// Same, with the host cwd supplied explicitly so a relative `cwd` can be
/// resolved exactly as os.path.abspath would (no_std has no os.getcwd()).
pub fn audit_and_execute_from(command: &str, cwd: &str, process_cwd: &str) -> (bool, String) {
    let abs_cwd = if process_cwd.is_empty() {
        abspath(cwd)
    } else {
        abspath_from(cwd, process_cwd)
    };
    let root = abspath(PROJECT_ROOT);
    if !abs_cwd.starts_with(&root) {
        return (
            false,
            format!(
                "Security Violation: Attempted execution outside project root ({})",
                abs_cwd
            ),
        );
    }

    let base_cmd = nt_basename(&command.split_whitespace().next().unwrap_or("").to_lowercase())
        .replace(".exe", "")
        .replace(".ps1", "");

    if !ALLOWED_COMMANDS.contains(&base_cmd.as_str()) {
        if !(base_cmd.starts_with('.') || base_cmd.starts_with("venv")) {
            return (
                false,
                format!(
                    "Security Violation: Command '{}' is not in the Shell-01 whitelist.",
                    base_cmd
                ),
            );
        }
    }

    let lowered = command.to_lowercase();
    for blocked in BLACKLISTED_PATHS.iter() {
        if lowered.contains(&blocked.to_lowercase()) {
            return (
                false,
                format!("Security Violation: Accessed blacklisted path '{}'.", blocked),
            );
        }
    }

    (true, String::new())
}

//  LobsterShell._apply_transform ---------------------------------------------

/// Port of LobsterShell._apply_transform (the {"pick:id","head:5","json"}
/// transform language). Any error/unknown transform returns the data unchanged,
/// exactly like the oracle's catch-all `except Exception: return data`.
pub fn _apply_transform(data: &Value, transform: Option<&str>) -> Value {
    let t = match transform {
        Some(s) if !s.is_empty() => s,
        _ => return data.clone(),
    };

    if let Some(rest) = t.strip_prefix("pick:") {
        let key = rest.split(':').next().unwrap_or("");
        return match data {
            Value::Dict(_) => data.get(key).cloned().unwrap_or(Value::Null),
            Value::List(items) => Value::List(
                items
                    .iter()
                    .filter(|it| it.is_dict())
                    .map(|it| it.get(key).cloned().unwrap_or(Value::Null))
                    .collect(),
            ),
            _ => data.clone(),
        };
    }

    if let Some(rest) = t.strip_prefix("head:") {
        let nstr = rest.split(':').next().unwrap_or("");
        return match py_int(nstr) {
            Some(n) => match data {
                Value::List(items) => Value::List(py_slice(items, n)),
                _ => data.clone(),
            },
            None => data.clone(),
        };
    }

    if t == "json" {
        return match data {
            Value::Str(s) => json_loads_py(s).unwrap_or_else(|| data.clone()),
            _ => data.clone(),
        };
    }

    data.clone()
}

/// Python list slicing data[:n] including negative n.
fn py_slice(items: &[Value], n: i64) -> Vec<Value> {
    let len = items.len() as i64;
    let end = if n < 0 { (len + n).max(0) } else { n.min(len) };
    items[..end as usize].to_vec()
}

//  LobsterShell / get_lobster_shell ------------------------------------------

/// Host tool executor: Python `Callable` (async (tool, args) -> Any). The
/// kernel mirrors only the signature; run_pipeline/await is host-side.
pub type ToolExecutor = fn(&str, &[(String, Value)]) -> Result<Value, String>;

pub struct LobsterShell {
    pub tool_executor: Option<ToolExecutor>,
}

impl LobsterShell {
    pub fn new(tool_executor: Option<ToolExecutor>) -> Self {
        LobsterShell { tool_executor }
    }

    /// @staticmethod audit_and_execute (validation half; see audit_and_execute)
    pub fn audit_and_execute(command: &str, cwd: &str) -> (bool, String) {
        audit_and_execute(command, cwd)
    }

    /// instance method _apply_transform
    pub fn _apply_transform(&self, data: &Value, transform: Option<&str>) -> Value {
        _apply_transform(data, transform)
    }
}

//  get_lobster_shell(executor) 
pub fn get_lobster_shell(executor: Option<ToolExecutor>) -> LobsterShell {
    // return LobsterShell(tool_executor=executor)
    LobsterShell::new(executor)
}
