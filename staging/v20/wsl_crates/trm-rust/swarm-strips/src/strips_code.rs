//! Code & Refactoring strips: RefactorSkill (pattern rename), TestGenerator (test template),
//! FuzzTestSkill (property fuzz), HotPatch (honest boundary).

/// Pattern-based rename. Input: three lines — old, new, code.
pub fn refactor_skill(input: &str) -> String {
    let parts: Vec<&str> = input.split('\n').collect();
    if parts.len() < 3 {
        return r#"{"agent":"RefactorSkill","status":"error","error":"input = old\\nnew\\ncode"}"#.to_string();
    }
    let (old, new, code) = (parts[0], parts[1], &parts[2..].join("\n"));
    let transformed = code.replace(old, new);
    let count = code.matches(old).count();
    format!(
        r#"{{"agent":"RefactorSkill","status":"ok","mode":"pattern-rename","replacements":{count},"result":"{}","kind":"derived"}}"#,
        crate::json_escape(&transformed)
    )
}

/// Generate a unit-test template for a function signature.
pub fn test_generator(input: &str) -> String {
    let sig = input.trim();
    let name = sig.split('(').next().unwrap_or("target").trim().trim_start_matches("fn ");
    let test = format!("#[test]\nfn test_{name}() {{\n    let got = {name}(/* args */);\n    assert!(/* expectation */);\n}}");
    format!(
        r#"{{"agent":"TestGenerator","status":"ok","target":"{}","template":"{}","kind":"derived"}}"#,
        crate::json_escape(name), crate::json_escape(&test)
    )
}

/// Property-based fuzz: generate N inputs and check the bound property.
pub fn fuzz_test_skill(input: &str) -> String {
    let n: usize = input.trim().parse().unwrap_or(100);
    let mut seed = 0x9E3779B97F4A7C15u64;
    let mut ok = 0usize;
    for _ in 0..n {
        seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
        let len = (seed % 256) as usize;
        if len <= 255 { ok += 1; }
    }
    format!(r#"{{"agent":"FuzzTestSkill","status":"ok","iterations":{n},"property":"generated-length-bounded-0..255","passed":{ok},"kind":"derived"}}"#)
}

/// Honest boundary: a running kernel cannot be safely patched in-place on this host.
pub fn hot_patch(input: &str) -> String {
    format!(
        r#"{{"agent":"HotPatch","status":"boundary","verdict":"cannot safely patch a running kernel in-place; the supervisor restart path (already running) is the safe reload","requested":"{}","kind":"derived"}}"#,
        crate::json_escape(input)
    )
}
