//! TRM Swarm OS - auto_changelog (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/auto_changelog.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;


//  ChangelogEntry 
#[derive(Debug, Clone)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub changes: Vec<Vec<(String, f64)>>,
}

impl ChangelogEntry {
    pub fn new() -> Self {
        ChangelogEntry {
            version: String::new(),
            date: String::new(),
            changes: vec![],
        }
    }
}

//  ModuleInfo 
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub path: String,
    pub name: String,
    pub category: u64,
    pub hash: String,
    pub last_modified: f64,
    pub status: String,
    pub description: String,
    pub exports: Vec<String>,
}

impl ModuleInfo {
    pub fn new() -> Self {
        ModuleInfo {
            path: String::new(),
            name: String::new(),
            category: 0,
            hash: String::new(),
            last_modified: 0.0,
            status: String::new(),
            description: String::new(),
            exports: vec![],
        }
    }
}

//  get_changelog_engine() 
static SINGLETON_CHANGELOG_ENGINE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_changelog_engine() -> u64 {
    // Singleton handle (lazy-init pattern from auto_changelog.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_CHANGELOG_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_CHANGELOG_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// `_classify_module`'s pure-Python fallback: path-based then content-based classification.
/// Category codes mirror `_NATIVE_MODULE_CATEGORY`: 0 UNKNOWN, 1 CORE, 2 API, 3 SKILL, 4 MCP,
/// 5 INFRASTRUCTURE, 6 INTEGRATION. `.lower()` is Unicode-aware in the oracle; this port uses
/// ASCII folding (exact for the ASCII paths this module carries).
pub fn classify_module(filepath: &str, content: Option<&str>) -> u8 {
    let lower = filepath.to_ascii_lowercase();
    if lower.contains("mcp") || lower.contains("tool") { return 4; }
    if lower.contains("skill") { return 3; }
    if lower.contains("api") || lower.contains("app") { return 2; }
    if lower.contains("infra") || lower.contains("daemon") { return 5; }
    if lower.contains("integrated") { return 6; }
    if lower.contains("core") { return 1; }
    if let Some(c) = content {
        // The oracle uses LITERAL substring checks here, not regex — `"class.*Skill" in content`
        // matches only the literal text `class.*Skill` (the `.*` is not a wildcard). Case is
        // significant: "MCP" is uppercase, "mcp_server" lowercase.
        if c.contains("FastAPI") || c.contains("@app") { return 2; }
        if c.contains("class.*Skill") { return 3; }
        if c.contains("MCP") || c.contains("mcp_server") { return 4; }
    }
    0
}

/// One `change` dict in `ChangelogEntry.changes` (type, category, module, description).
#[derive(Debug, Clone)]
pub struct Change {
    pub change_type: String,
    pub category: String,
    pub module: String,
    pub description: String,
}

/// `ChangelogEntry.to_markdown` — faithful reproduction of the oracle's exact join.
///
/// The oracle builds `lines = ["## [v] - d\n"]`, then per fixed type order appends
/// `"\n### {type}\n"` and one `- ...` line per change (no trailing newline), then
/// `"\n".join(lines)`. The header's trailing `\n` plus the join `\n` plus the section
/// line's LEADING `\n` produce the double blank line after the header; the last item
/// line carries no trailing newline. Reproduced literally.
pub fn to_markdown(version: &str, date: &str, changes: &[Change]) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("## [{}] - {}\n", version, date));
    for ty in ["Added", "Changed", "Fixed", "Security", "Deprecated", "Removed"] {
        let in_type: Vec<&Change> = changes.iter().filter(|c| c.change_type == ty).collect();
        if in_type.is_empty() { continue; }
        lines.push(format!("\n### {}\n", ty));
        for c in &in_type {
            if c.module.is_empty() {
                lines.push(format!("- ({}): {}", c.category, c.description));
            } else {
                lines.push(format!("- **{}** ({}): {}", c.module, c.category, c.description));
            }
        }
    }
    lines.join("\n")
}

/// `_extract_description` — the first docstring's first line, truncated to 100 chars.
///
/// `re.search(r'"""([\s\S]*?)"""', content)` is non-greedy: the FIRST `"""` through the
/// NEXT `"""`. If no triple-double-quote docstring, the same with `'''`. `.strip()` then
/// `split("\n")[0]` then `[:100]` (100 Unicode code points). Empty string when absent.
fn docstring_between<'a>(content: &'a str, delim: &str) -> Option<&'a str> {
    let start = content.find(delim)?;
    let after = start + delim.len();
    let end = content[after..].find(delim)?;
    Some(&content[after..after + end])
}

pub fn extract_description(content: &str) -> String {
    for delim in ["\"\"\"", "'''"] {
        if let Some(desc) = docstring_between(content, delim) {
            let first = desc.trim().split('\n').next().unwrap_or("");
            return first.chars().take(100).collect();
        }
    }
    String::new()
}
