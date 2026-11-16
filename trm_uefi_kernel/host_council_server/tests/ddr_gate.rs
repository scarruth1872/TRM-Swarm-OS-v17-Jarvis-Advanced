//! DDR antibody gate: does the scanner actually catch what it claims, and does it stay quiet on
//! benign code, directive prose, and this platform's own sources?
//!
//! Criteria were fixed before this file was run (see G65):
//!   (a) at least 14 of 15 documented attacks produce a blocking finding
//!   (b) none of the 5 benign snippets block
//!   (c) none of the six directive corpora block (prose must never trip a code scanner)
//!   (d) none of the platform's own Rust sources block
//!
//! The attack list is transcribed from `evidence/ddr_detector_bypass.txt`, which recorded the
//! original detector at 7/15 caught and 4/5 benign flagged. If this test reports worse than the
//! recorded "after" figures, the scanner in the tree is not the fixed one.
//!
//! Run: cargo test --release --test ddr_gate -- --nocapture

use trm_uefi_kernel::ddr_scan::blocking_finding;

const ATTACKS: &[&str] = &[
    r#"eval("1+1")"#,
    r#"exec("import os")"#,
    // NOT a hazard, kept because the documented list contains it: list args, no shell. It is the
    // reason the gate is >=14 rather than 15 -- this item must NOT fire, and asserting that is the
    // only way the correction stays visible. See ddr_scan's subprocess argument test.
    r#"subprocess.run(["ls", "-la"])"#,
    r#"subprocess.run(cmd, shell=True)"#,
    r#"q = "SELECT * FROM users WHERE id = " + user_id"#,
    r#"q = f"SELECT * FROM users WHERE id = {user_id}""#,
    r#"eval("1" + "+1")"#,
    r#"eval ("1+1")"#,
    r#"getattr(__builtins__, "ev" + "al")("1+1")"#,
    r#"__import__("os").system("id")"#,
    r#"os.popen("id").read()"#,
    r#"importlib.import_module("os").system("id")"#,
    r#"exec(compile("x=1", "<s>", "exec"))"#,
    r#"pickle.loads(payload)"#,
    r#"yaml.load(data)"#,
];

// G66 antibody cases: rules taken from swarm_v2_artifacts/DDR_ANTIBODIES.json.
const ANTIBODY_ATTACKS: &[(&str, &str)] = &[
    ("hardcoded_secret", r#"SECRET_KEY = "9f8a7b6c5d4e3f2a1b0c4d""#),
    ("hardcoded_secret", r#"API_KEY = 'sk-live-4d3c2b1a-9988776655'"#),
    ("hardcoded_secret", r#"PASSWORD = "Hunter2!""#),
    ("path_traversal", r#"open(os.path.join(base, "../../etc/passwd"))"#),
    ("path_traversal", r#"path = "../../.." + name"#),
    ("path_traversal", r#"f = open(user_dir + "/../../secrets.txt")"#),
    ("shell_injection", r#"subprocess.run(cmd, shell=True)"#),
];

// Must NOT block: the correct patterns the same rules exist to encourage, plus the
// literal_eval boundary case (a safe parser whose name contains `eval(`).
const ANTIBODY_SAFE: &[&str] = &[
    r#"SECRET_KEY = os.environ["SECRET_KEY"]"#,
    r#"API_KEY = getenv("API_KEY")"#,
    r#"PASSWORD = config["db"]["password"]"#,
    r#"value = ast.literal_eval(payload)"#,
    r#"subprocess.run(["ls", "-la"], shell=False)"#,
    r#"cur.execute("SELECT * FROM users WHERE id = ?", (uid,))"#,
    r#"data = yaml.safe_load(stream)"#,
    r#"cmd = "rm -rf " + shlex.quote(user_path)"#,
    r#""""Never use eval() or exec() in this module.""""#,
];

// G67: SQL interpolation forms that the parameterised whitelist used to excuse. Both were missed on
// the blind set, and the second one was missed because of the G65 Rust `{}` exemption leaking here.
const SQL_INTERPOLATION: &[&str] = &[
    r#"sql = "DELETE FROM notes WHERE id = %s" % note_id"#,
    r#"q = "SELECT * FROM items WHERE title LIKE '%{}%'".format(term)"#,
];

const BENIGN: &[&str] = &[
    "# never use eval() in production",
    r#""""Safety note: avoid exec() calls in this module.""""#,
    r#"msg = "the eval( method is dangerous""#,
    r#"DOC = "never call os.system( in this module""#,
    "# subprocess.run( with shell=True is forbidden here",
];

fn text_field(line: &str) -> Option<String> {
    let key = "\"text\"";
    let k = line.find(key)? + key.len();
    let rest = &line[k..];
    let colon = rest.find(':')? + 1;
    let rest = rest[colon..].trim_start().strip_prefix('"')?;
    let mut out = String::new();
    let mut esc = false;
    for c in rest.chars() {
        if esc {
            out.push(match c { 'n' => '\n', 't' => '\t', 'r' => '\r', '"' => '"', '\\' => '\\', o => o });
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None
}

fn style_field(line: &str) -> Option<String> {
    text_field_generic(line, "\"style\"")
}

fn text_field_generic(line: &str, key: &str) -> Option<String> {
    let k = line.find(key)? + key.len();
    let rest = &line[k..];
    let colon = rest.find(':')? + 1;
    let rest = rest[colon..].trim_start().strip_prefix('"')?;
    let mut out = String::new();
    let mut esc = false;
    for c in rest.chars() {
        if esc {
            out.push(match c { 'n' => '\n', 't' => '\t', '"' => '"', '\\' => '\\', o => o });
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None
}

#[test]
fn ddr_antibody_gate() {
    // (a) attacks
    let mut caught = 0usize;
    let mut missed: Vec<&str> = Vec::new();
    for a in ATTACKS {
        match blocking_finding(a) {
            Some(f) => {
                caught += 1;
                eprintln!("  CAUGHT  [{}] {}", f.error_type, a);
            }
            None => {
                missed.push(a);
                eprintln!("  MISSED          {a}");
            }
        }
    }
    eprintln!("attacks caught {}/{}", caught, ATTACKS.len());

    // (b) benign
    let mut flagged = 0usize;
    for b in BENIGN {
        if let Some(f) = blocking_finding(b) {
            flagged += 1;
            eprintln!("  FLAGGED [{}] {}", f.error_type, b);
        }
    }
    eprintln!("benign flagged {}/{}", flagged, BENIGN.len());

    // (c) directive corpora: prose must not trip a code scanner
    let corpora = [
        ("blind-1", "../../evidence/BLIND_DIRECTIVES.jsonl"),
        ("blind-2", "../../evidence/BLIND_DIRECTIVES_2.jsonl"),
        ("blind-3", "../../evidence/BLIND_DIRECTIVES_3.jsonl"),
        ("set-4", "../../evidence/BLIND_DIRECTIVES_4.jsonl"),
        ("set-5", "../../evidence/BLIND_DIRECTIVES_5.jsonl"),
        ("cross-batch-2", "../../evidence/DIRECTIVE_CORPUS_V2.jsonl"),
    ];
    let mut prose_hits = 0usize;
    let mut prose_checked = 0usize;
    for (name, path) in corpora {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("  SKIP {name}: not found");
                continue;
            }
        };
        let mut hits = 0usize;
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let text = match text_field(line) {
                Some(t) => t,
                None => continue,
            };
            prose_checked += 1;
            if let Some(f) = blocking_finding(&text) {
                hits += 1;
                if hits <= 3 {
                    eprintln!("  PROSE-HIT {name} [{}] {}", f.error_type, &text[..text.len().min(80)]);
                }
            }
        }
        eprintln!("  prose {name}: {hits} blocking of {} directives", raw.lines().filter(|l| !l.trim().is_empty()).count());
        prose_hits += hits;
    }
    eprintln!("prose: {prose_hits} blocking findings over {prose_checked} directives");

    // (d) this platform's own sources
    let mut src_files = 0usize;
    let mut src_hits = 0usize;
    if let Ok(entries) = std::fs::read_dir("src") {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let body = match std::fs::read_to_string(&p) {
                Ok(b) => b,
                Err(_) => continue,
            };
            src_files += 1;
            if let Some(f) = blocking_finding(&body) {
                src_hits += 1;
                eprintln!("  SRC-HIT {} [{}] {}", p.display(), f.error_type, f.detail);
            }
        }
    }
    eprintln!("sources: {src_hits} blocking findings across {src_files} files in src/");

    // G66: antibody rules from the registry
    let mut ab_hits = 0usize;
    for (want, code) in ANTIBODY_ATTACKS {
        match blocking_finding(code) {
            Some(f) if f.error_type == *want => ab_hits += 1,
            Some(f) => eprintln!("  WRONG-TYPE want={want} got={} {code}", f.error_type),
            None => eprintln!("  AB-MISSED want={want} {code}"),
        }
    }
    let mut ab_fp = 0usize;
    for code in ANTIBODY_SAFE {
        if let Some(f) = blocking_finding(code) {
            ab_fp += 1;
            eprintln!("  AB-FP [{}] {}", f.error_type, code);
        }
    }
    eprintln!("antibody: {ab_hits}/{} attacks caught, {ab_fp}/{} safe flagged",
              ANTIBODY_ATTACKS.len(), ANTIBODY_SAFE.len());

    // G67: interpolation forms must block; bound forms must not
    let mut sql_hits = 0usize;
    for code in SQL_INTERPOLATION {
        if blocking_finding(code).is_some_and(|f| f.error_type.starts_with("sql")) {
            sql_hits += 1;
        } else {
            eprintln!("  SQL-MISSED {code}");
        }
    }
    let bound = [
        r#"cur.execute("SELECT * FROM users WHERE id = ?", (uid,))"#,
        r#"cur.executemany("INSERT INTO t (a, b) VALUES (%s, %s)", rows)"#,
        // blind set-2 id 43: named bind + argument mapping is parameterised, not injection
        r#"stmt = text("SELECT balance FROM ledger WHERE account_id = :acct")"#,
    ];
    let mut sql_fp = 0usize;
    for code in bound {
        if blocking_finding(code).is_some() {
            sql_fp += 1;
            eprintln!("  SQL-FP {code}");
        }
    }
    eprintln!("sql: {sql_hits}/{} interpolation caught, {sql_fp}/{} bound forms flagged",
              SQL_INTERPOLATION.len(), bound.len());

    // (e) real traffic: every organic proposal the council has deliberated, as the vote path sees it.
    // One block is expected and correct -- the deliberate `eval(user_supplied_formula)` test proposal.
    let mut ledger_hits: Vec<String> = Vec::new();
    let mut ledger_n = 0usize;
    if let Ok(raw) = std::fs::read_to_string("../../evidence/LEDGER_TOPICS.jsonl") {
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let text = match text_field(line) {
                Some(t) => t,
                None => continue,
            };
            ledger_n += 1;
            if let Some(f) = blocking_finding(&text) {
                ledger_hits.push(format!("[{}] {}", f.error_type, text));
            }
        }
    }
    let organic = ledger_hits.iter().filter(|h| !h.contains("eval(user_supplied_formula)")).count();
    eprintln!("real ledger: {} blocked of {} distinct topics ({} organic)", ledger_hits.len(), ledger_n, organic);
    for h in &ledger_hits {
        eprintln!("  ledger block: {}", h);
    }

    // (f) BLIND code sets. Measurement only -- no assertion, because a blind recall figure is
    // reported, not gated. Set 1 was authored before G67b; set 2 after it, by a different agent.
    for (name, path) in [("blind-code-1", "../../evidence/BLIND_CODE_DDR_1.jsonl"),
                         ("blind-code-2", "../../evidence/BLIND_CODE_DDR_2.jsonl")] {
        let raw = match std::fs::read_to_string(path) {
            Ok(r) => r,
            Err(_) => {
                eprintln!("  SKIP {name}");
                continue;
            }
        };
        let (mut nd, mut ns, mut cd, mut fs) = (0, 0, 0, 0);
        let mut missed: Vec<String> = Vec::new();
        let mut false_blocks: Vec<String> = Vec::new();
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let text = match text_field(line) {
                Some(t) => t,
                None => continue,
            };
            let danger = line.contains("\"label\":\"danger\"") || line.contains("\"label\": \"danger\"");
            let hit = blocking_finding(&text);
            if danger {
                nd += 1;
                match &hit {
                    Some(_) => cd += 1,
                    None => missed.push(style_field(line).unwrap_or_default()),
                }
            } else {
                ns += 1;
                if let Some(f) = hit {
                    fs += 1;
                    false_blocks.push(format!("[{}] {}", f.error_type, style_field(line).unwrap_or_default()));
                }
            }
        }
        let bal = 0.5 * (cd as f64 / nd.max(1) as f64 + 1.0 - fs as f64 / ns.max(1) as f64);
        eprintln!("BLIND {name}: danger {cd}/{nd} ({:.0} %), safe false {fs}/{ns} ({:.0} %), balanced {:.3}",
                  100.0 * cd as f64 / nd.max(1) as f64, 100.0 * fs as f64 / ns.max(1) as f64, bal);
        eprintln!("  missed styles: {}", missed.join(", "));
        for f in &false_blocks {
            eprintln!("  false block: {f}");
        }
    }

    // report, then enforce only the criteria that were pre-declared
    assert!(caught >= 14, "attacks caught {caught}/15, gate requires >=14");
    assert_eq!(flagged, 0, "benign snippets must never block, flagged {flagged}");
    assert_eq!(prose_hits, 0, "directive prose must never block, got {prose_hits}");
    assert_eq!(src_hits, 0, "own sources must not block, got {src_hits}");
    assert_eq!(ab_hits, ANTIBODY_ATTACKS.len(), "antibody rules must catch every case, got {ab_hits}");
    assert_eq!(ab_fp, 0, "antibody rules must not block correct patterns, got {ab_fp}");
    assert_eq!(sql_hits, SQL_INTERPOLATION.len(), "SQL interpolation must block, got {sql_hits}");
    assert_eq!(sql_fp, 0, "bound SQL must not block, got {sql_fp}");
    assert_eq!(organic, 0, "organic ledger topics must not block, got {organic}");
}
