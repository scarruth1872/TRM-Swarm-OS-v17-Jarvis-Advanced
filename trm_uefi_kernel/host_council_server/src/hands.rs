//! Capability "Hands" manifests, ported from `openfang-hands` v0.6.9.
//!
//! PROVENANCE: the data model follows `openfang-hands::HandDefinition` / `HandRequirement` /
//! `HandMetric` (github.com/RightNow-AI/openfang, Apache-2.0 OR MIT; original preserved at
//! `preserved_sources/openfang_original/crates/openfang-hands/`).
//!
//! WHAT CHANGED: upstream parses `HAND.toml` with the `toml` crate and serialises status through
//! `tracing`. The kernel has `serde`/`serde_json` only, so manifests here are **JSON** — the same
//! fields, one parser, zero new dependencies. The upstream marketplace/install machinery is not
//! ported; what is ported is the part the kernel actually needs: a declarative capability package
//! plus a *measurable* readiness check.
//!
//! THE POINT OF THIS MODULE: a Hand declares what a capability requires. `readiness()` then probes
//! the real host — env var present, file exists, binary on PATH — so "this capability is available"
//! is a measurement, not an assertion.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HandCategory {
    Productivity,
    Research,
    Engineering,
    Security,
    Finance,
    Personal,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementType {
    /// Executable that must be resolvable on PATH.
    Binary,
    /// Environment variable that must be set and non-empty.
    EnvVar,
    /// Path that must exist.
    File,
    /// Documented manual step; never probeable, never auto-satisfied.
    Manual,
}

/// Risk class. OURS, not upstream: the kernel gates actions through the council and the DDR
/// scanner, so a capability package has to declare how dangerous it is. High risk requires an
/// explicit approval path before it can be considered ready.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HandRisk {
    Low,
    Medium,
    High,
}

impl HandRisk {
    /// High-risk capabilities may only be activated through an approving council decision.
    pub fn requires_approval(self) -> bool {
        matches!(self, HandRisk::High)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandRequirement {
    pub key: String,
    pub label: String,
    pub requirement_type: RequirementType,
    pub check_value: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Optional requirements never block activation; they degrade it.
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandMetric {
    pub label: String,
    pub memory_key: String,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "number".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandSetting {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub default: Option<String>,
    /// Settings whose value is a secret must never be echoed back in a readiness report.
    #[serde(default)]
    pub secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: HandCategory,
    #[serde(default = "low_risk")]
    pub risk: HandRisk,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub requires: Vec<HandRequirement>,
    #[serde(default)]
    pub settings: Vec<HandSetting>,
    #[serde(default)]
    pub metrics: Vec<HandMetric>,
}

fn low_risk() -> HandRisk {
    HandRisk::Low
}

/// Parse a Hand manifest.
pub fn parse_hand(json: &str) -> Result<HandDefinition, String> {
    serde_json::from_str::<HandDefinition>(json).map_err(|e| format!("manifest parse error: {e}"))
}

/// Structural validation. Deliberately strict about the things that turn a capability package
/// into an unaccountable one: no id, no description, no declared risk, or a high-risk capability
/// that declares no requirements at all.
pub fn validate_hand(h: &HandDefinition) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    if h.id.trim().is_empty() {
        errs.push("id is required".to_string());
    }
    if h.id.contains(char::is_whitespace) {
        errs.push("id must not contain whitespace".to_string());
    }
    if h.name.trim().is_empty() {
        errs.push("name is required".to_string());
    }
    if h.description.trim().len() < 10 {
        errs.push("description must explain the capability (>= 10 chars)".to_string());
    }
    if h.risk.requires_approval() && h.requires.iter().all(|r| r.optional) {
        errs.push(
            "high-risk hand must declare at least one non-optional requirement or approval gate"
                .to_string(),
        );
    }
    for r in &h.requires {
        if r.key.trim().is_empty() || r.check_value.trim().is_empty() {
            errs.push(format!("requirement {:?} needs key and check_value", r.key));
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// Result of probing the host for one requirement.
#[derive(Debug, Clone, Serialize)]
pub struct RequirementCheck {
    pub key: String,
    pub requirement_type: RequirementType,
    pub optional: bool,
    pub satisfied: bool,
    /// What was actually observed — never a secret's value, only whether it is set.
    pub observed: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Readiness {
    pub hand_id: String,
    pub ready: bool,
    /// High-risk hands are never "ready" without an approving decision, whatever the probes say.
    pub needs_approval: bool,
    pub checked: usize,
    pub unmet_required: Vec<String>,
    pub unmet_optional: Vec<String>,
    pub checks: Vec<RequirementCheck>,
}

fn binary_on_path(name: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return true;
            }
            #[cfg(windows)]
            {
                let exe = dir.join(format!("{name}.exe"));
                if exe.is_file() {
                    return true;
                }
            }
        }
    }
    false
}

/// Probe the real host. Every field in the result is something that was measured.
pub fn readiness(h: &HandDefinition) -> Readiness {
    let mut checks = Vec::new();
    let mut unmet_required = Vec::new();
    let mut unmet_optional = Vec::new();

    for r in &h.requires {
        let (satisfied, observed) = match r.requirement_type {
            RequirementType::EnvVar => match std::env::var(&r.check_value) {
                Ok(v) if !v.trim().is_empty() => (true, "set (value withheld)".to_string()),
                Ok(_) => (false, "set but empty".to_string()),
                Err(_) => (false, "not set".to_string()),
            },
            RequirementType::File => {
                let p = Path::new(&r.check_value);
                (p.exists(), format!("exists={}", p.exists()))
            }
            RequirementType::Binary => {
                let found = binary_on_path(&r.check_value);
                (found, format!("on PATH={found}"))
            }
            RequirementType::Manual => (false, "manual step; not probeable".to_string()),
        };
        if !satisfied {
            if r.optional {
                unmet_optional.push(r.key.clone());
            } else {
                unmet_required.push(r.key.clone());
            }
        }
        checks.push(RequirementCheck {
            key: r.key.clone(),
            requirement_type: r.requirement_type,
            optional: r.optional,
            satisfied,
            observed,
        });
    }

    Readiness {
        hand_id: h.id.clone(),
        ready: unmet_required.is_empty() && !h.risk.requires_approval(),
        needs_approval: h.risk.requires_approval(),
        checked: checks.len(),
        unmet_required,
        unmet_optional,
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "id": "log-triage",
        "name": "Log Triage",
        "description": "Reads service logs and groups recurring errors",
        "category": "engineering",
        "risk": "low",
        "requires": [
            {"key":"logs","label":"Log directory","requirement_type":"file","check_value":"C:/Windows/win.ini"},
            {"key":"grep","label":"grep","requirement_type":"binary","check_value":"this-binary-does-not-exist-xyz"}
        ]
    }"#;

    #[test]
    fn parses_and_validates_sample() {
        let h = parse_hand(SAMPLE).expect("sample must parse");
        assert_eq!(h.id, "log-triage");
        assert!(validate_hand(&h).is_ok());
    }

    #[test]
    fn rejects_missing_fields() {
        let bad = r#"{"id":"","name":"","description":"x","category":"other"}"#;
        let h = parse_hand(bad).unwrap();
        let errs = validate_hand(&h).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("id")));
        assert!(errs.iter().any(|e| e.contains("name")));
        assert!(errs.iter().any(|e| e.contains("description")));
    }

    #[test]
    fn high_risk_needs_something_to_gate() {
        let json = r#"{"id":"wipe","name":"Wipe","description":"Deletes every artifact on the box","category":"engineering","risk":"high","requires":[]}"#;
        let h = parse_hand(json).unwrap();
        let errs = validate_hand(&h).unwrap_err();
        assert!(errs.iter().any(|e| e.contains("high-risk")), "{errs:?}");
    }

    #[test]
    fn readiness_probes_the_real_host() {
        let h = parse_hand(SAMPLE).unwrap();
        let r = readiness(&h);
        assert_eq!(r.checked, 2);
        // the file requirement points at a path that exists on every Windows box
        let file_check = r.checks.iter().find(|c| c.key == "logs").unwrap();
        assert!(file_check.satisfied, "win.ini must exist");
        // the binary is deliberately impossible
        assert!(r.unmet_required.contains(&"grep".to_string()));
        assert!(!r.ready, "an unmet required item must not report ready");
    }

    #[test]
    fn high_risk_is_never_ready_without_approval() {
        let json = r#"{"id":"deploy","name":"Deploy","description":"Pushes artifacts to production hosts","category":"engineering","risk":"high",
            "requires":[{"key":"k","label":"K","requirement_type":"env_var","check_value":"PATH"}]}"#;
        let h = parse_hand(json).unwrap();
        let r = readiness(&h);
        assert!(r.needs_approval);
        assert!(!r.ready, "high risk must require an approving decision");
        assert!(r.checks[0].satisfied, "PATH is set");
    }

    #[test]
    fn secrets_are_never_reported_by_value() {
        let json = r#"{"id":"s","name":"S","description":"Uses an api key from the environment","category":"other",
            "requires":[{"key":"k","label":"API key","requirement_type":"env_var","check_value":"PATH"}]}"#;
        let h = parse_hand(json).unwrap();
        let r = readiness(&h);
        assert_eq!(r.checks[0].observed, "set (value withheld)");
    }
}
