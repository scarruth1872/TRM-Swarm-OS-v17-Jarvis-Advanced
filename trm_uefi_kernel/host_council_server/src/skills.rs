//! SKILL.md registry/loader, ported from `openfang-skills` v0.6.9 (loader + registry only).
//!
//! PROVENANCE: field model follows `openfang-skills::SkillManifest` / `SkillMeta`
//! (github.com/RightNow-AI/openfang, Apache-2.0 OR MIT; original preserved at
//! `preserved_sources/openfang_original/crates/openfang-skills/`).
//!
//! NOT PORTED, deliberately: `marketplace.rs`, `clawhub.rs`, `installer.rs` — those pull `reqwest`,
//! `tokio` and a remote skill marketplace. The kernel has no network surface and must not grow one.
//! What is ported is the part that makes `/learning` honest: read a SKILL.md, parse its
//! frontmatter, and report per-file parse errors instead of silently counting directories.

use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub license: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillManifest {
    pub meta: SkillMeta,
    /// Explicit trigger phrases (`triggers:` in frontmatter), if declared.
    pub triggers: Vec<String>,
    pub tags: Vec<String>,
    /// Size of the markdown body after the frontmatter — proves the file was actually read.
    pub body_chars: usize,
}

fn unquote(v: &str) -> String {
    let t = v.trim();
    if t.len() >= 2
        && ((t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')))
    {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

fn parse_inline_list(v: &str) -> Vec<String> {
    let t = v.trim();
    let inner = t.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(|s| unquote(s))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse a SKILL.md: YAML-ish frontmatter between `---` fences, then a markdown body.
///
/// This is a deliberately small subset — top-level `key: value` pairs and inline `[a, b]` lists.
/// Nested blocks (e.g. `metadata:` with indented children) are skipped rather than misparsed, so a
/// richer file parses what it can instead of failing outright.
pub fn parse_skill_md(raw: &str) -> Result<SkillManifest, String> {
    let mut lines = raw.lines();
    match lines.next() {
        Some(l) if l.trim() == "---" => {}
        _ => return Err("missing frontmatter: file must start with '---'".to_string()),
    }

    let mut name = String::new();
    let mut description = String::new();
    let mut version = String::new();
    let mut author = String::new();
    let mut license = String::new();
    let mut triggers: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();
    let mut closed = false;
    let mut body_chars = 0usize;

    for line in lines.by_ref() {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        // Nested (indented) keys belong to a sub-map we do not model.
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            let val = v.trim();
            match key.as_str() {
                "name" => name = unquote(val),
                "description" => description = unquote(val),
                "version" => version = unquote(val),
                "author" => author = unquote(val),
                "license" => license = unquote(val),
                "triggers" => triggers = parse_inline_list(val),
                "tags" => tags = parse_inline_list(val),
                _ => {}
            }
        }
    }

    if !closed {
        return Err("unterminated frontmatter: no closing '---'".to_string());
    }
    for l in lines {
        body_chars += l.chars().count() + 1;
    }

    if name.trim().is_empty() {
        return Err("frontmatter has no 'name'".to_string());
    }
    if description.trim().is_empty() {
        return Err("frontmatter has no 'description'".to_string());
    }

    Ok(SkillManifest {
        meta: SkillMeta {
            name,
            description,
            version: if version.is_empty() { "0.0.0".to_string() } else { version },
            author: if author.is_empty() { "unknown".to_string() } else { author },
            license: if license.is_empty() { "unspecified".to_string() } else { license },
        },
        triggers,
        tags,
        body_chars,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanEntry {
    pub path: String,
    pub ok: bool,
    pub name: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub dir: String,
    pub scanned: usize,
    pub parsed: usize,
    pub failed: usize,
    pub entries: Vec<ScanEntry>,
}

/// Scan a directory for `SKILL.md` files (one level deep and one level of subdirectories, which is
/// the layout both Hermes and OpenFang use). Every file gets a row: a parse failure is reported,
/// never skipped — a registry that silently drops bad files is the defect this replaces.
pub fn scan_skills_dir(dir: &Path) -> ScanReport {
    let mut targets: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let nested = p.join("SKILL.md");
                if nested.is_file() {
                    targets.push(nested);
                }
            } else if p.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
                targets.push(p);
            }
        }
    }
    targets.sort();

    let mut entries = Vec::with_capacity(targets.len());
    let mut parsed = 0usize;
    for path in &targets {
        match std::fs::read_to_string(path) {
            Ok(raw) => match parse_skill_md(&raw) {
                Ok(m) => {
                    parsed += 1;
                    entries.push(ScanEntry {
                        path: path.display().to_string(),
                        ok: true,
                        name: Some(m.meta.name),
                        error: None,
                    });
                }
                Err(e) => entries.push(ScanEntry {
                    path: path.display().to_string(),
                    ok: false,
                    name: None,
                    error: Some(e),
                }),
            },
            Err(e) => entries.push(ScanEntry {
                path: path.display().to_string(),
                ok: false,
                name: None,
                error: Some(format!("read error: {e}")),
            }),
        }
    }

    ScanReport {
        dir: dir.display().to_string(),
        scanned: targets.len(),
        parsed,
        failed: targets.len().saturating_sub(parsed),
        entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "---\nname: cross-machine-codebase-convergence\ndescription: \"Use when porting or merging code between machines.\"\nversion: 1.0.0\nauthor: Hermes Agent\nlicense: MIT\nmetadata:\n  hermes:\n    tags: [migration, rust]\n---\n\n# Title\n\nBody text here.\n";

    #[test]
    fn parses_a_real_skill_header() {
        let m = parse_skill_md(SAMPLE).unwrap();
        assert_eq!(m.meta.name, "cross-machine-codebase-convergence");
        assert_eq!(m.meta.version, "1.0.0");
        assert_eq!(m.meta.license, "MIT");
        assert!(m.body_chars > 0, "body must have been counted");
        // nested keys are skipped, not misparsed into top-level fields
        assert_eq!(m.meta.description, "Use when porting or merging code between machines.");
    }

    #[test]
    fn rejects_missing_frontmatter() {
        assert!(parse_skill_md("# no frontmatter\n").is_err());
    }

    #[test]
    fn rejects_unterminated_frontmatter() {
        assert!(parse_skill_md("---\nname: x\n").is_err());
    }

    #[test]
    fn requires_name_and_description() {
        assert!(parse_skill_md("---\nversion: 1\n---\nbody\n").is_err());
        assert!(parse_skill_md("---\nname: x\n---\nbody\n").is_err());
    }

    #[test]
    fn parses_inline_lists() {
        let raw = "---\nname: t\ndescription: d\ntriggers: [\"a b\", c]\n---\n";
        let m = parse_skill_md(raw).unwrap();
        assert_eq!(m.triggers, vec!["a b".to_string(), "c".to_string()]);
    }

    #[test]
    fn scan_reports_failures_instead_of_hiding_them() {
        let dir = std::env::temp_dir().join(format!("trm-skill-scan-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("good")).unwrap();
        std::fs::create_dir_all(dir.join("bad")).unwrap();
        std::fs::write(dir.join("good/SKILL.md"), SAMPLE).unwrap();
        std::fs::write(dir.join("bad/SKILL.md"), "no frontmatter here\n").unwrap();

        let report = scan_skills_dir(&dir);
        assert_eq!(report.scanned, 2);
        assert_eq!(report.parsed, 1);
        assert_eq!(report.failed, 1);
        assert!(report.entries.iter().any(|e| !e.ok && e.error.is_some()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
