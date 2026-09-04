//! TRM Swarm OS — skill_validator (no_std port).
//! 'audited_secv4' compliance protocol, deterministic core from
//! swarm_v2/core/skill_validator.py:
//!   - skill record schema checks: skill_name present and matching
//!     `^[a-zA-Z0-9_]+$`, description present, instructions present;
//!   - drift detection: invalid records are pruned (kept list = compliant);
//!   - final audit status logic:
//!       errors>0 → WARNING if healed else FAIL
//!       healed (no errors) → WARNING, else PASS;
//!   - audit message format:
//!       "Skill '{name or Unknown}' contract drift: {e1}, {e2}".
//! Filesystem (manifest.json / backup restore / JSON parse) and
//! broadcaster bodies are host-bound → `// MANUAL` below, never invented.

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// `^[a-zA-Z0-9_]+$` — the secv4 skill_name contract (regex-free check).
fn valid_skill_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// One audited skill record (deterministic subset of the manifest entry).
#[derive(Debug, Clone)]
pub struct SkillRecord {
    pub skill_name: String,
    pub description: String,
    pub instructions: String,
}

/// Per-skill schema audit. Returns the python error list:
///   name empty → ["Missing skill_name"]
///   name invalid → ["Invalid characters in name: {name}"]  (elif branch)
///   description empty → adds "Missing description field"
///   instructions empty → adds "Missing instructions usage guidelines"
pub fn audit_skill_record(record: &SkillRecord) -> Vec<String> {
    let mut errors: Vec<String> = vec![];
    if record.skill_name.is_empty() {
        errors.push("Missing skill_name".to_string());
    } else if !valid_skill_name(&record.skill_name) {
        errors.push(format!("Invalid characters in name: {}", record.skill_name));
    }
    if record.description.is_empty() {
        errors.push("Missing description field".to_string());
    }
    if record.instructions.is_empty() {
        errors.push("Missing instructions usage guidelines".to_string());
    }
    errors
}

/// Final audit status decision (python tail of verify_skill_manifest):
///   errors non-empty → "WARNING" if healed else "FAIL"
///   else if healed → "WARNING", else "PASS" (initial status preserved).
pub fn final_status(healed: bool, error_count: usize) -> &'static str {
    if error_count > 0 {
        if healed {
            "WARNING"
        } else {
            "FAIL"
        }
    } else if healed {
        "WARNING"
    } else {
        "PASS"
    }
}

/// Python message for one drifted skill:
///   f"Skill '{name or 'Unknown'}' contract drift: {', '.join(errors)}"
pub fn drift_message(name: &str, errors: &[String]) -> String {
    let display = if name.is_empty() { "Unknown" } else { name };
    let joined = errors.join(", ");
    format!("Skill '{}' contract drift: {}", display, joined)
}

/// Result of auditing a full in-memory manifest record list.
#[derive(Debug, Clone)]
pub struct AuditResult {
    pub status: &'static str,
    /// Number of compliant records retained after pruning.
    pub valid_count: usize,
    /// Number of drifted records pruned.
    pub pruned_count: usize,
    /// Whether any self-heal action was taken (drift prune / restore).
    pub healed: bool,
    /// All drift messages in manifest order.
    pub errors: Vec<String>,
}

/// verify_skill_manifest() deterministic core over an in-memory record list:
/// python iterates `data["skills"]`, prunes non-compliant records (drift
/// detected → healed=True on write-back), keeps compliant ones.
pub fn audit_manifest(records: &[SkillRecord]) -> AuditResult {
    let mut errors: Vec<String> = vec![];
    let mut valid: Vec<&SkillRecord> = vec![];
    let mut drift = false;

    for skill in records {
        let errs = audit_skill_record(skill);
        if !errs.is_empty() {
            drift = true;
            errors.push(drift_message(&skill.skill_name, &errs));
        } else {
            valid.push(skill);
        }
    }

    // python: drift → self-heal manifest by keeping only compliant records,
    // audit_result["healed"] = True (write is host-bound, subset retained).
    let healed = drift;
    let status = final_status(healed, errors.len());

    AuditResult {
        status,
        valid_count: valid.len(),
        pruned_count: records.len() - valid.len(),
        healed,
        errors,
    }
}

/// SkillValidator — deterministic subset (audit log capped at 50 like python
/// `_audit_log` pop(0); manifest filesystem state is host-bound).
#[derive(Debug, Clone)]
pub struct SkillValidator {
    pub manifest_dir: String,
    pub active: bool,
    pub audit_log: Vec<AuditResult>,
}

impl SkillValidator {
    pub fn new(manifest_dir: &str) -> Self {
        SkillValidator {
            manifest_dir: manifest_dir.to_string(),
            active: false,
            audit_log: vec![],
        }
    }

    /// Run an audit over records and append to the capped log.
    pub fn verify_manifest(&mut self, records: &[SkillRecord]) -> AuditResult {
        let res = audit_manifest(records);
        self.audit_log.push(res.clone());
        if self.audit_log.len() > 50 {
            self.audit_log.remove(0);
        }
        res
    }

    /// start_validation_loop / stop — async loop flags (python 1:1 fields).
    pub fn start_loop(&mut self) {
        self.active = true;
    }

    pub fn stop_loop(&mut self) {
        self.active = false;
    }

    pub fn audit_log_len(&self) -> usize {
        self.audit_log.len()
    }
}

/// Singleton instance (lazy-init pattern from skill_validator.py).
static SINGLETON_SKILL_VALIDATOR: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
/// get_skill_validator() singleton handle: 1 once initialized.
pub fn get_skill_validator() -> u64 {
    let v = SINGLETON_SKILL_VALIDATOR.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SKILL_VALIDATOR.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
