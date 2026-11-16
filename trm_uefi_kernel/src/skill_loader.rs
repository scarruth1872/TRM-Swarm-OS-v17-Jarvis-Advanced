//! TRM Swarm OS — skill_loader (no_std port, REAL-verified core).
//! 1:1 deterministic core of swarm_v2/core/skill_loader.py:
//!   - SKILL_SEARCH_PATHS, SkillDefinition dataclass fields
//!   - parse_skill_md: YAML-frontmatter split (--- markers), lightweight
//!     "key: value" frontmatter parse (lowercased keys, quote stripping),
//!     prerequisites/tags CSV split, defaults (version 1.0.0)
//!   - registry: register (overwrite by name, position kept), get_skill,
//!     get_skill_instructions, discover_skills, list_skills, get_stats
//! Filesystem scanning (_scan_directory/_auto_discover/_load_python_skills)
//! and load_skill(path) file reads are I/O — MANUAL skeletons.

extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

/// Python module constant SKILL_SEARCH_PATHS (1:1).
pub const SKILL_SEARCH_PATHS: &[&str] = &[
    ".agent/skills",
    ".agents/skills",
    "_agent/skills",
    "swarm_v2/skills",
];

/// A parsed SKILL.md skill definition (python dataclass SkillDefinition).
#[derive(Debug, Clone, PartialEq)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub version: String,
    pub instructions: String,
    pub prerequisites: Vec<String>,
    pub tags: Vec<String>,
    pub source_path: String,
    pub source_type: String, // "skill_md" or "python_class"
    pub loaded_at: String,
}

impl SkillDefinition {
    /// Python dataclass defaults: version "1.0.0", everything else empty.
    pub fn new() -> Self {
        SkillDefinition {
            name: String::new(),
            description: String::new(),
            version: "1.0.0".to_string(),
            instructions: String::new(),
            prerequisites: vec![],
            tags: vec![],
            source_path: String::new(),
            source_type: String::new(),
            loaded_at: String::new(),
        }
    }
}

impl Default for SkillDefinition {
    fn default() -> Self {
        SkillDefinition::new()
    }
}

/// Lightweight list metadata for list_skills (python dicts).
#[derive(Debug, Clone, PartialEq)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub source_type: String,
    pub tags: Vec<String>,
}

/// Skill loader statistics mirroring get_stats().
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkillStats {
    pub total_skills: usize,
    pub skill_md_count: usize,
    pub python_class_count: usize,
    pub search_path_count: usize,
}

/// Line starts with "---" and the remainder is whitespace only — matches the
/// python frontmatter open/close markers (`---` then \s*).
fn is_marker_line(line: &str) -> bool {
    match line.strip_prefix("---") {
        Some(rest) => rest.trim().is_empty(),
        None => false,
    }
}

/// Split content on YAML frontmatter, mirroring python
/// re.match(r'^---\s*\n(.*?)\n---\s*\n(.*)', content, DOTALL):
/// the file must START with a "---" marker line and the first later "---"
/// marker line that is followed by a newline closes the frontmatter.
/// Returns (frontmatter_text, instructions) — instructions empty when there
/// is no frontmatter (caller then treats the whole content as instructions).
fn split_frontmatter(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.split('\n').collect();
    if !lines.is_empty() && is_marker_line(lines[0]) {
        for (idx, line) in lines.iter().enumerate().skip(1) {
            // python requires a trailing \n after the closing marker, i.e.
            // the closing "---" line must not be the last line of the file.
            if is_marker_line(line) && idx + 1 < lines.len() {
                let fm = lines[1..idx].join("\n");
                let instr = lines[idx + 1..].join("\n");
                return (fm, instr);
            }
        }
    }
    (String::new(), content.to_string())
}

/// python `[p.strip() for p in value.split(",") if p.strip()]`.
fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect()
}

/// ASCII lowercase for a str key (non-ASCII chars pass through intact).
fn ascii_lower_key(s: &str) -> String {
    s.chars().map(|c| c.to_ascii_lowercase()).collect()
}

/// Deterministic core of python `_parse_skill_md(path)` given the file
/// CONTENT (the file read itself is I/O): extract frontmatter, parse
/// "key: value" lines (line.strip, key.strip().lower(), value
/// strip -> strip('"') -> strip("'")), default name = fallback_name (python:
/// basename(dirname(path)), computed host-side), version "1.0.0".
/// loaded_at mirrors datetime.now().isoformat() (host timestamp).
pub fn parse_skill_md(
    content: &str,
    fallback_name: &str,
    path_label: &str,
    loaded_at: &str,
) -> SkillDefinition {
    let (frontmatter, instr) = split_frontmatter(content);
    let instructions = instr.trim().to_string();

    let mut name = fallback_name.to_string();
    let mut description = String::new();
    let mut version = "1.0.0".to_string();
    let mut prerequisites: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();

    for raw_line in frontmatter.split('\n') {
        let line = raw_line.trim();
        if let Some(ci) = line.find(':') {
            let key = ascii_lower_key(line[..ci].trim());
            let value = line[ci + 1..].trim().trim_matches('"').trim_matches('\'');
            match key.as_str() {
                "name" => name = value.to_string(),
                "description" => description = value.to_string(),
                "version" => version = value.to_string(),
                "prerequisites" => prerequisites = split_csv(value),
                "tags" => tags = split_csv(value),
                _ => {}
            }
        }
    }

    SkillDefinition {
        name,
        description,
        version,
        instructions,
        prerequisites,
        tags,
        source_path: path_label.to_string(),
        source_type: "skill_md".to_string(),
        loaded_at: loaded_at.to_string(),
    }
}

/// Skill loader registry. Python stores skills in a dict keyed by name; a
/// Vec preserves the python insertion order (dict order) for iteration.
#[derive(Debug, Clone, Default)]
pub struct SkillLoader {
    skills: Vec<SkillDefinition>,
}

impl SkillLoader {
    /// python `__init__(project_root)`: the auto-discovery scan is filesystem
    /// bound (SKILL.md walking + python class imports) — see auto_discover().
    pub fn new() -> Self {
        SkillLoader { skills: Vec::new() }
    }

    /// python `register_skill(skill)`: self._skills[name] = skill — overwrites
    /// a same-name skill while keeping its original dict position.
    pub fn register_skill(&mut self, skill: SkillDefinition) {
        match self.skills.iter_mut().find(|s| s.name == skill.name) {
            Some(existing) => *existing = skill,
            None => self.skills.push(skill),
        }
    }

    /// python `get_skill(name)`.
    pub fn get_skill(&self, name: &str) -> Option<&SkillDefinition> {
        self.skills.iter().find(|s| s.name == name)
    }

    /// python `get_skill_instructions(name)`: "" for unknown skills.
    pub fn get_skill_instructions(&self, name: &str) -> String {
        match self.get_skill(name) {
            Some(s) => s.instructions.clone(),
            None => String::new(),
        }
    }

    /// python `discover_skills()`: all registered skills in dict order.
    pub fn discover_skills(&self) -> Vec<SkillDefinition> {
        self.skills.clone()
    }

    /// python `list_skills()`: metadata dicts in dict order.
    pub fn list_skills(&self) -> Vec<SkillMeta> {
        self.skills
            .iter()
            .map(|s| SkillMeta {
                name: s.name.clone(),
                description: s.description.clone(),
                version: s.version.clone(),
                source_type: s.source_type.clone(),
                tags: s.tags.clone(),
            })
            .collect()
    }

    /// python `get_stats()`: totals by source_type + search path list length.
    pub fn get_stats(&self) -> SkillStats {
        let mut skill_md = 0usize;
        let mut python_class = 0usize;
        for s in &self.skills {
            if s.source_type == "skill_md" {
                skill_md += 1;
            } else if s.source_type == "python_class" {
                python_class += 1;
            }
        }
        SkillStats {
            total_skills: self.skills.len(),
            skill_md_count: skill_md,
            python_class_count: python_class,
            search_path_count: SKILL_SEARCH_PATHS.len(),
        }
    }

    /// MANUAL: python `load_skill(path)` reads the file from disk then parses
    /// it — filesystem I/O. Use module fn parse_skill_md on already-read
    /// content. Returns None (python returns None when the file is unreadable).
    pub fn load_skill(&self, _path: &str) -> Option<SkillDefinition> {
        None
    }

    /// MANUAL: python `_auto_discover()` walks SKILL_SEARCH_PATHS on the
    /// filesystem for SKILL.md files and imports python skill classes —
    /// filesystem/import I/O. Deterministic register/parse live elsewhere.
    pub fn auto_discover(&mut self) -> bool {
        false
    }
}

/// Python module-level singleton `get_skill_loader()` — no_std handle
/// convention: 1 once initialized.
static SINGLETON_SKILL_LOADER: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
pub fn get_skill_loader() -> u64 {
    let v = SINGLETON_SKILL_LOADER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SKILL_LOADER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
