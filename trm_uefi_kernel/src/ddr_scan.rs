//! DDR threat scanner - kernel-native lexical analysis (no_std).
//!
//! WHY THIS IS IN THE KERNEL, NOT THE HOST SERVER
//! The DDR capability existed in two places with the same defect. The kernel's
//! ddr_antibody.rs matched with `code.contains(strip_regex(pattern))` - its own
//! comment admitted "no_std regex substitute ... we match the pattern string as a
//! substring". The host server had three literal `contains()` calls that caught
//! 7 of 15 standard attacks, defeated by a single space in `eval ("1+1")`, while
//! flagging comments that merely mentioned eval().
//!
//! Duplicating a fix in the host leaves the kernel unfixed and the two able to
//! disagree. This module is the single implementation; the host server calls it.
//!
//! THE TWO-LAYER DESIGN
//!   stripped - comments removed, string CONTENTS blanked. Call patterns match
//!              here, so prose and docstrings cannot trip them. Removes the
//!              false positives.
//!   raw      - obfuscation heuristics, because those evasions exist precisely
//!              to hide the token from layer 1 ("ev"+"al", chr(), base64).
//!
//! LIMITS, STATED PLAINLY: lexical, not semantic. Cannot see through three levels
//! of indirection; executes nothing. A large improvement on substring matching,
//! NOT a substitute for AST/taint analysis and NOT a substitute for refusing to
//! run untrusted text. More patterns do not raise this ceiling.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::format;

/// One detection.
#[derive(Debug, Clone)]
pub struct Finding {
    pub error_type: &'static str,
    pub severity: &'static str,
    /// Human-readable specifics (which calls, which indicators).
    pub detail: String,
    pub fix: &'static str,
}

/// Cheap language sniff, used ONLY to scope the Python-shaped SQL and obfuscation lanes.
/// It is deliberately not used for comment stripping any more: a snippet as ordinary as
/// `let x = cmd(); // note` has none of these markers, and leaving its comment in place re-created
/// the benign false positive the stripper exists to prevent.
fn looks_rust(src: &str) -> bool {
    const MARKERS: &[&str] = &["fn ", "let mut ", "impl ", "-> ", "::", "pub struct", "&mut "];
    MARKERS.iter().any(|m| src.contains(m))
}

/// Remove comments and blank out string CONTENTS, leaving code structure intact.
pub fn strip_comments_and_strings(src: &str) -> String {
    let cs: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let n = cs.len();
    while i < n {
        let c = cs[i];
        if c == '\'' || c == '"' {
            let q = c;
            if i + 2 < n && cs[i + 1] == q && cs[i + 2] == q {
                // triple-quoted (docstring)
                let mut j = i + 3;
                while j + 2 < n && !(cs[j] == q && cs[j + 1] == q && cs[j + 2] == q) {
                    j += 1;
                }
                out.push(' ');
                i = if j + 2 < n { j + 3 } else { n };
                continue;
            }
            out.push(' ');
            let mut j = i + 1;
            while j < n {
                if cs[j] == '\\' {
                    j += 2;
                    continue;
                }
                if cs[j] == q {
                    break;
                }
                j += 1;
            }
            out.push(' ');
            i = if j < n { j + 1 } else { n };
            continue;
        }
        if c == '#' {
            while i < n && cs[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // `//` and `/* */` are stripped whatever the language. Measured need: a comment containing
        // `eval(` was flagged as a dangerous primitive because Python-only stripping left it in.
        // KNOWN NARROW EVASION, disclosed: Python's floor division uses the same token, so
        // `x = 1 // eval(payload)` would have its tail hidden. No documented attack uses it, and the
        // alternative (language-sniffed stripping) restored the false positive on ordinary code.
        if c == '/' && i + 1 < n && cs[i + 1] == '/' {
            while i < n && cs[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < n && cs[i + 1] == '*' {
            i += 2;
            while i + 1 < n && !(cs[i] == '*' && cs[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Drop whitespace adjacent to punctuation so `eval (` and `eval(` are one token.
/// This is the fix for the single-space bypass that defeated the old scanner.
pub fn tighten(s: &str) -> String {
    let cs: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;
    let punct = "()[]{}.,=+-*/%<>:;";
    while i < cs.len() {
        let c = cs[i];
        if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
            let mut j = i;
            while j < cs.len() && (cs[j] == ' ' || cs[j] == '\t' || cs[j] == '\n' || cs[j] == '\r') {
                j += 1;
            }
            let next = cs.get(j).copied().unwrap_or(' ');
            let prev = out.chars().last().unwrap_or(' ');
            if punct.contains(next) || punct.contains(prev) {
                i = j;
                continue;
            }
            out.push(' ');
            i = j;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// `name(` where the preceding char is not an identifier char.
pub fn has_call(hay: &str, name: &str) -> bool {
    // The dot exclusion is the whole point for BARE builtins. The first version
    // rejected only alphanumerics and underscore, so `re.compile(...)` matched the
    // bare `compile` needle - and re.compile is one of the most common calls in
    // Python. Measured on the platform's own tree: of 571 flagged files, 271 were
    // flagged for `compile`, overwhelmingly re.compile. `re.compile` is not
    // `compile`, and a gate that cannot tell them apart is worse than useless.
    //
    // Dotted names (subprocess.run) are passed WITH the dot, so the char before the
    // whole needle is what gets tested and this exclusion still behaves correctly.
    let needle = format!("{}(", name);
    let mut from = 0usize;
    while let Some(pos) = hay[from..].find(&needle) {
        let at = from + pos;
        let before_ok = hay[..at]
            .chars()
            .last()
            .map(|c| !(c.is_alphanumeric() || c == '_' || c == '.'))
            .unwrap_or(true);
        if before_ok {
            return true;
        }
        from = at + 1;
    }
    false
}

/// Scan source text. Returns one Finding per category that fires.
pub fn scan(code: &str) -> Vec<Finding> {
    let stripped = tighten(&strip_comments_and_strings(code));
    let raw = code;
    // The SQL and obfuscation lanes are Python-shaped: they uppercase-fold keyword mentions and treat
    // `{`/`%`/adjacent literals as formatting. On Rust that yields false positives in ordinary code
    // (a comment saying "select the tier", `format!` braces, `&`), measured at 3 of 16 files in this
    // crate. Those lanes therefore only run on non-Rust payloads; the language-agnostic lanes
    // (dangerous primitives, shell=True) always run. DISCLOSED LIMITATION: DDR does not inspect Rust
    // for equivalent dangers (Command::new, std::fs writes) at all -- it has no rules for them.
    let python_shaped = !looks_rust(raw);
    let mut out: Vec<Finding> = Vec::new();

    // ---- layer 1: dangerous execution primitives (prose already removed) ----
    let prims: [(&str, &str); 14] = [
        ("eval", "eval of dynamic input"),
        ("exec", "exec of dynamic input"),
        ("__import__", "dynamic module import"),
        ("compile", "compile() code-object construction"),
        ("importlib.import_module", "import by name, evades static import checks"),
        ("os.popen", "shell execution"),
        ("os.system", "shell execution"),
        ("os.execv", "process replacement"),
        ("pty.spawn", "interactive shell"),
        ("subprocess.run", "external process"),
        ("subprocess.Popen", "external process"),
        ("subprocess.call", "external process"),
        ("marshal.loads", "deserialises code objects"),
        ("importlib.reload", "dynamic reload"),
    ];
    let mut found: Vec<&str> = Vec::new();
    // subprocess is judged by its arguments, not its name: list-args with shell=False is the
    // CORRECT pattern (measured false positive: `subprocess.run(["ls","-la"], shell=False)` was
    // flagged as danger). A string command, or shell=True, is the hazard.
    let subprocess_unsafe = raw.contains("shell=True")
        || (raw.contains("subprocess.") && !raw.contains('['));
    for (name, _) in prims.iter() {
        if !has_call(&stripped, name) {
            continue;
        }
        if name.starts_with("subprocess.") && !subprocess_unsafe {
            continue;
        }
        found.push(name);
    }
    let sinks: [(&str, &str); 5] = [
        ("pickle.loads", "arbitrary object deserialisation"),
        ("pickle.load", "arbitrary object deserialisation"),
        ("dill.loads", "arbitrary object deserialisation"),
        ("yaml.load", "yaml.load without SafeLoader"),
        ("shelve.open", "pickle-backed store"),
    ];
    for (name, _) in sinks.iter() {
        if has_call(&stripped, name) {
            found.push(name);
        }
    }
    if has_call(&stripped, "CDLL") || has_call(&stripped, "cdll") || has_call(&stripped, "WinDLL") {
        found.push("ctypes native load");
    }
    if !found.is_empty() {
        // Classification, not detection: a shell sink is command injection (CWE-78/T1059) while
        // eval/exec/pickle are code injection/deserialisation (CWE-95/T1059). Reporting shell=True
        // as CWE-95 mislabels the hazard the reader has to fix.
        let shell_sink = found.iter().any(|n| {
            n.starts_with("os.popen")
                || n.starts_with("os.system")
                || n.starts_with("os.execv")
                || n.starts_with("pty.spawn")
                || n.starts_with("subprocess.")
        });
        out.push(Finding {
            error_type: if shell_sink { "shell_injection" } else { "dangerous_execution_primitive" },
            severity: "critical",
            detail: found.join(", "),
            fix: "Remove the dynamic execution path. Parse and evaluate in a typed interpreter; do not hand this text to a Python interpreter.",
        });
    }

    if stripped.contains("shell=True") {
        out.push(Finding {
            error_type: "shell_injection",
            severity: "critical",
            detail: "shell=True".to_string(),
            fix: "Drop shell=True; pass an argument vector.",
        });
    }

    // ---- layer 2: obfuscation, matched on RAW (these hide tokens from layer 1) ----
        // OBFUSCATION IS A COMBINATION, NOT A SINGLE FEATURE.
    // Each indicator below is individually routine in real Python: backslash-x
    // escapes appear in every regex, chr() in every codec, getattr(obj, "name") in
    // every plugin dispatcher. Flagging on any ONE of them produced 225 flags on
    // the platform's own tree - a third of the codebase. Real obfuscation shows
    // SEVERAL at once, because its purpose is to hide a token from a reader and
    // from a scanner. The threshold is therefore 2 or more, with one exception:
    // a dangerous NAME assembled from pieces is decisive on its own.
    let mut obf: Vec<String> = Vec::new();
    if python_shaped {
    // A RUN of string literals joined by +, in every quote/spacing shape:
    //   "a"+"b"   'a'+'b'   "a" + "b"   'a' + 'b'
    // The earlier form only matched the two no-content shapes, so a 4-term
    // run like 'e'+'v'+'a'+'l' slipped through even though "ev"+"al" was caught.
    let has_adjacent_literals = raw.contains(r#""+"#) || raw.contains("'+'")
        || raw.contains(r#"" + ""#) || raw.contains("' + '")
        || raw.contains("'+\"") || raw.contains("\"+'");
    let has_chr_assembly = raw.contains("chr(") && (raw.contains("join(") || raw.contains('+'));
    let has_base64_decode = raw.contains("base64") && (raw.contains("decode") || raw.contains("b64decode"));
    let has_escapes = raw.contains("\\x") || raw.contains("\\u");
    let has_getattr_string = raw.contains("getattr(") && (raw.contains('\'') || raw.contains('"'));
    let builds_dangerous_name = (has_adjacent_literals
            && (raw.contains("eval") || raw.contains("exec") || raw.contains("__import__")))
        || (has_chr_assembly
            && (raw.contains("eval") || raw.contains("exec") || raw.contains("__import__")));
    if has_adjacent_literals { obf.push("adjacent string literals".to_string()); }
    if has_chr_assembly { obf.push("chr() assembly".to_string()); }
    if has_base64_decode { obf.push("base64 decode".to_string()); }
    if has_escapes { obf.push("escaped byte sequence".to_string()); }
    if has_getattr_string { obf.push("getattr with a string name".to_string()); }
    // require the decisive signal, or two or more weak ones together
    if !builds_dangerous_name && obf.len() < 2 { obf.clear(); }
    if !obf.is_empty() {
        out.push(Finding {
            error_type: "obfuscation_evasion_indicators",
            severity: "high",
            detail: obf.join(", "),
            fix: "Obfuscated construction of a dangerous call. Resolve the literal by hand before executing; never execute it to find out.",
        });
    }
    } // end python_shaped (obfuscation lane)

    // ---- SQL: matched on RAW, because the keyword lives INSIDE the string ----
    if python_shaped {
    let upper = raw.to_uppercase();
    let sql_verb = ["SELECT ", "INSERT ", "UPDATE ", "DELETE ", "DROP "]
        .iter()
        .any(|v| upper.contains(v));
    if sql_verb {
        // Parameterised SQL is not injection. If the query uses placeholders
        // (?, %s, :name) it is the CORRECT pattern even when the string is built
        // with + or f-strings, and flagging it trains the reader to ignore the
        // gate - which is how a real finding gets missed later.
        // `%s` is a bound parameter only when the value is passed alongside the query. If `%` is
        // applied to the string itself, it is interpolation. (Blind set-1 id 12 caught this:
        // `"... id = %s" % note_id` was whitelisted as parameterised and missed.)
        let percent_operator = raw.contains("\" %") || raw.contains("' %");
        // The `{}`/`{:?}` exemption from G65 was written for Rust `format!` and is WRONG here: this
        // lane already excludes Rust payloads, so in Python `"...{}".format(x)` is interpolation.
        // (Blind set-1 id 13 was missed by it.)
        let parameterised = !percent_operator
            && (raw.contains('?')
                || raw.contains("%s")
                || raw.contains(":{")
                || raw.contains("= %(")
                // Named binds: `WHERE account_id = :acct` with the mapping passed to execute().
                // Measured need: a safe SQLAlchemy query was flagged because the argument dict's
                // brace was read as "constructed" (blind set-2 id 43).
                || raw.contains("= :"));
        let constructed = (raw.contains('+')
            || raw.contains('%')
            || raw.contains('{')
            || raw.contains(".format(")) && !parameterised;
        if constructed {
            out.push(Finding {
                error_type: "sql_injection_defense",
                severity: "high",
                detail: "SQL string built by concatenation or formatting".to_string(),
                fix: "Use parameterised queries with bound parameters.",
            });
        }
    }
    } // end python_shaped (SQL lane)

    // ---- DDR antibody rules ----------------------------------------------------------------
    // Source: swarm_v2_artifacts/DDR_ANTIBODIES.json. The registry holds 1,968 records but only 7
    // distinct rules; of those, only these two are both new and security-relevant (f-string SQL is
    // already caught above, eval_usage duplicates rule 1, and debug_print/bare_except are lint at
    // low/medium severity -- below the critical/high blocking threshold). Every record's
    // file_pattern is `*.py`, so these run only on non-Rust payloads.
    // hardcoded_secret: NAME = "literal". The RHS must be a literal; `os.environ[...]`, `getenv()`
    // and config lookups are the correct pattern and must not be flagged.
    if python_shaped {
    // The stripper replaces quote characters with spaces, so this rule reads RAW lines and anchors
    // on the statement's first token: only `NAME = "..."` / `NAME: "..."` / `self.NAME = "..."` fire,
    // which keeps prose that merely mentions a credential name out of the finding.
    for line in raw.lines() {
        let names = ["SECRET_KEY", "API_KEY", "PASSWORD"];
        let mut head = line.trim_start();
        if let Some(rest) = head.strip_prefix("self.") {
            head = rest;
        }
        let matched = names.iter().find(|n| head.starts_with(*n));
        let name = match matched {
            Some(n) => *n,
            None => continue,
        };
        let after = head[name.len()..].trim_start();
        if let Some(eq) = after.find('=') {
            let rhs = after[eq + 1..].trim_start();
            let literal = rhs.starts_with('"') || rhs.starts_with('\'');
            let dynamic = rhs.contains("environ") || rhs.contains("getenv") || rhs.contains("config");
            if literal && !dynamic {
                out.push(Finding {
                    error_type: "hardcoded_secret",
                    severity: "high",
                    detail: "credential assigned a literal value".to_string(),
                    fix: "Read the credential from the environment or a secret store.",
                });
                break;
            }
        }
    }
    // path_traversal: a relative parent path in the payload.
    if raw.contains("../..") || raw.contains("..\\..") {
        out.push(Finding {
            error_type: "path_traversal",
            severity: "high",
            detail: "relative parent path in payload".to_string(),
            fix: "Resolve paths with a canonical base and reject traversal components.",
        });
    }
    } // end python_shaped (antibody lanes)

    out
}

/// Proposal-level policy, shared by every caller so the loop and the tests cannot diverge.
///
/// Only `critical` or `high` findings may block; anything lower is reported and ignored. This is the
/// same asymmetric shape as the lexical and frame gates: DDR may block an approval, it can never
/// force one, and it never sees or rewrites the vote itself.
pub fn blocking_finding(code: &str) -> Option<Finding> {
    scan(code).into_iter().find(|f| f.severity == "critical" || f.severity == "high")
}

/// Convenience: does the text trip anything at all?
pub fn is_suspicious(code: &str) -> bool {
    !scan(code).is_empty()
}
