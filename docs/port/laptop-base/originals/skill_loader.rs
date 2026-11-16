//! TRM Swarm OS - skill_loader (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/skill_loader.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const SKILL_SEARCH_PATHS: &'static [&'static str] = &[".agent/skills", ".agents/skills", "_agent/skills", "swarm_v2/skills"];

//  SkillDefinition 
#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub version: String,
    pub instructions: String,
    pub prerequisites: Vec<String>,
    pub tags: Vec<String>,
    pub source_path: String,
    pub source_type: String,
    pub loaded_at: String,
}

impl SkillDefinition {
    /// The oracle's dataclass declares `version: str = "1.0.0"` and `source_type: str = "skill_md"`.
    /// This constructor was zeroing both -- the same dropped-default class as `ServiceConfig`'s timeouts
    /// in self_healing_infra, and just as consequential: a skill built through `new()` would report no
    /// version and would not be recognised as a SKILL.md skill. `loaded_at` is the oracle's
    /// `datetime.now().isoformat()`, left empty here because bare metal has no wall clock.
    pub fn new() -> Self {
        SkillDefinition {
            name: String::new(),
            description: String::new(),
            version: String::from("1.0.0"),
            instructions: String::new(),
            prerequisites: vec![],
            tags: vec![],
            source_path: String::new(),
            source_type: String::from("skill_md"),
            loaded_at: String::new(),
        }
    }
}

//  get_skill_loader() 
static SINGLETON_SKILL_LOADER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_skill_loader() -> u64 {
    // Singleton handle (lazy-init pattern from skill_loader.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_SKILL_LOADER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SKILL_LOADER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ── The oracle's SKILL.md parser (swarm_v2/core/skill_loader.py, `_parse_skill_md`) ───────────────
//
// The whole rule, from the oracle's own code:
//   * `re.match(r'^---\s*\n(.*?)\n---\s*\n(.*)', content, re.DOTALL)` -- anchored at the START of the
//     file (no MULTILINE), so the file must open with `---`; group 1 (non-greedy, DOTALL) is the
//     frontmatter, group 2 (greedy) is the body;
//   * no match -> the frontmatter is EMPTY and the whole file becomes the instructions;
//   * the instructions are `.strip()`ed;
//   * each frontmatter line is stripped and a line containing `:` is cut at the FIRST one, with the key
//     lower-cased and the value's surrounding quotes stripped (`strip('"')` then `strip("'")`);
//   * defaults: name -> the skill DIRECTORY's name, description "", version "1.0.0"; prerequisites and
//     tags are comma-split with each item stripped and empties dropped.
//
// The part that is easy to get wrong: after `---` the regex reads `\s*` and THEN a literal `\n`, and
// `\s` includes `\n`, so the greedy `\s*` takes the whole whitespace run and backtracks until the
// character it leaves is a newline. Group 1 therefore begins after the LAST newline of that run --
// trailing spaces before the newline stay IN the frontmatter. Reproduced here, including the case where
// the run holds no newline at all (the whole match then fails).
//
// `loaded_at` is the oracle's `datetime.now().isoformat()`; bare metal has no wall clock, so the caller
// passes it in. `source_type` is the oracle's "skill_md", which the generated `SkillDefinition::new()`
// was zeroing -- see the note there.

fn is_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// Every position `\s*\n` could end at, starting at `at`, in the order a greedy `\s*` tries them:
/// LONGEST first, then backing off. Returning all candidates matters -- greedy `\s*` backtracks when the
/// REST of the pattern fails, so the match can legitimately use a shorter run. On `---\n\n\n---\nbody`
/// the oracle ends up taking `\s*` = "" (the shortest), because only then does the closing `\n---` line
/// up; a version that returned just the longest candidate read the whole file as instructions.
fn ws_then_newline_candidates(b: &[u8], at: usize) -> Vec<usize> {
    let mut end = at;
    while end < b.len() && is_ws(b[end]) {
        end += 1;
    }
    let mut out: Vec<usize> = Vec::new();
    let mut l = end;
    while l > at {
        l -= 1;
        if b[l] == b'\n' {
            out.push(l + 1);
        }
    }
    out
}

/// `_parse_skill_md` over an already-read file; `dir_name` is the skill directory's name.
pub fn parse_skill_md(content: &str, dir_name: &str, source_path: &str, loaded_at: &str) -> SkillDefinition {
    let b = content.as_bytes();
    let mut frontmatter = "";
    let mut instructions = content.trim().to_string();
    let mut matched = false;
    if b.len() >= 3 && &b[0..3] == b"---" {
        // greedy `\s*`: longest candidate first, then backing off; the first one whose remaining pattern
        // matches is the one the engine uses.
        for g1 in ws_then_newline_candidates(b, 3) {
            let mut k = g1;
            while k < b.len() {
                if b[k] == b'\n' && k + 4 <= b.len() && &b[k + 1..k + 4] == b"---" {
                    // non-greedy `.*?`: the SMALLEST k that lets the rest match wins
                    if let Some(g2) = ws_then_newline_candidates(b, k + 4).first().copied() {
                        frontmatter = &content[g1..k];
                        instructions = content[g2..].trim().to_string();
                        matched = true;
                        break;
                    }
                }
                k += 1;
            }
            if matched {
                break;
            }
        }
    }
    if !matched {
        frontmatter = "";
    }

    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut version: Option<String> = None;
    let mut prereq_raw: Option<String> = None;
    let mut tags_raw: Option<String> = None;
    for line in frontmatter.split('\n') {
        let line = line.trim();
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_ascii_lowercase();
            let value = line[pos + 1..].trim();
            let value = value.trim_matches('"').trim_matches('\'').to_string();
            match key.as_str() {
                "name" => name = Some(value),
                "description" => description = Some(value),
                "version" => version = Some(value),
                "prerequisites" => prereq_raw = Some(value),
                "tags" => tags_raw = Some(value),
                _ => {}
            }
        }
    }
    let split_list = |raw: Option<String>| -> Vec<String> {
        match raw {
            None => Vec::new(),
            Some(s) => s.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()).map(|p| p.to_string()).collect(),
        }
    };
    SkillDefinition {
        name: name.unwrap_or_else(|| dir_name.to_string()),
        description: description.unwrap_or_default(),
        version: version.unwrap_or_else(|| String::from("1.0.0")),
        instructions,
        prerequisites: split_list(prereq_raw),
        tags: split_list(tags_raw),
        source_path: source_path.to_string(),
        source_type: String::from("skill_md"),
        loaded_at: loaded_at.to_string(),
    }
}
