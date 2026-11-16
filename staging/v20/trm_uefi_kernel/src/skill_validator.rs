//! TRM Swarm OS — Skill Manifest Validator & audited_secv4 Healer (no_std).
//! Authentic bare-metal port of swarm_v2/core/skill_validator.py.
//!
//! Monad Truth Anchor: 0x4D4F4E4144000001
//! Phase 24, Rank 114 TurboVec 16x Balanced Ternary Quantization | Kleene K3 Verified.

#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const VOLTAGE_SCALE_FACTOR: f32 = 31.9056;
pub const VOLTAGE_PHASE: u32 = 24;
pub const VOLTAGE_RANK: u32 = 114;

/// Phase 24 Rank 114 Balanced Ternary Quantized Weights (64 trits, 2-bit packed).
pub static SKILL_VALIDATOR_WEIGHTS_BT: [u8; 16] = [
    0x6A, 0x95, 0xA9, 0x56, 0x59, 0xA6, 0x65, 0x9A,
    0xAA, 0x55, 0x99, 0x66, 0x66, 0x99, 0xA5, 0x5A,
];

// ── Ported contract rules (swarm_v2/core/skill_validator.py) ────────────────────────────────────
//
// python:  a skill record is compliant when `skill_name` is non-empty AND matches ^[a-zA-Z0-9_]+$,
//          `description` is non-empty, and `instructions` is non-empty. Non-compliant records are
//          pruned (the manifest self-heals) and counted as drift.
//          status: errors -> WARNING if healed else FAIL; elif healed -> WARNING; else PASS
//          audit log: append, and drop the oldest once it exceeds 50
//          validation loop default interval: 30 s
//
// Two of those figures were absent from this module entirely (the 50-entry cap and the 30 s default)
// and "Healed" was modelled as a *status* rather than the flag it is in the oracle, so a repair made
// with errors could not be told apart from a clean pass. `description`/`instructions` were not on the
// record at all, which is why the charset and emptiness rules had nowhere to run.
//
// Host-side and deliberately NOT ported: reading/writing the manifest file, the backup restore, the
// asyncio validation loop and the spatial-broadcaster push. Verification here judges records handed
// to it and reports what it would have pruned.

/// The oracle's audit-log cap: the last 50 results are kept, oldest dropped.
pub const AUDIT_LOG_CAP: usize = 50;
/// The oracle's `start_validation_loop(interval: int = 30)` default, in seconds.
pub const VALIDATION_INTERVAL_SECS: u32 = 30;

/// Manifest compliance status. Three states, because the oracle has three: `Healed` is a flag the
/// result carries, not an outcome in its own right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestComplianceStatus {
    Pass,
    Warning,
    Fail,
}

/// Skill manifest entry record.
#[derive(Debug, Clone)]
pub struct SkillManifestEntry {
    pub name: String,
    pub version: String,
    pub checksum: u64,
    pub endpoint_count: u32,
    pub is_valid: bool,
    /// Contract fields the oracle requires alongside the name.
    pub description: String,
    pub instructions: String,
}

/// The oracle's per-record contract checks, in its own order and wording.
pub fn contract_errors(name: &str, description: &str, instructions: &str) -> Vec<&'static str> {
    let mut errs = Vec::new();
    if name.is_empty() {
        errs.push("Missing skill_name");
    } else if !is_contract_name(name) {
        errs.push("Invalid characters in name");
    }
    if description.is_empty() {
        errs.push("Missing description field");
    }
    if instructions.is_empty() {
        errs.push("Missing instructions usage guidelines");
    }
    errs
}

/// `^[a-zA-Z0-9_]+$`, spelled out rather than pulling in a regex engine on bare metal.
pub fn is_contract_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// One audit outcome: the status, whether a repair happened, how many records were pruned and how
/// many errors were seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditResult {
    pub status: ManifestComplianceStatus,
    pub healed: bool,
    pub pruned: usize,
    pub errors: usize,
}

/// Skill Validator implementing audited_secv4 compliance protocol.
pub struct SkillValidator {
    pub entries: Vec<SkillManifestEntry>,
    pub audit_count: u64,
    pub heal_count: u64,
    /// The bounded audit log the oracle keeps in `_audit_log` (cap 50, oldest dropped).
    pub audit_log: Vec<AuditResult>,
}

impl SkillValidator {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            audit_count: 0,
            heal_count: 0,
            audit_log: Vec::new(),
        }
    }

    pub fn add_entry(
        &mut self,
        name: &str,
        version: &str,
        checksum: u64,
        endpoint_count: u32,
        description: &str,
        instructions: &str,
    ) {
        let is_valid = contract_errors(name, description, instructions).is_empty();
        self.entries.push(SkillManifestEntry {
            name: String::from(name),
            version: String::from(version),
            checksum,
            endpoint_count,
            is_valid,
            description: String::from(description),
            instructions: String::from(instructions),
        });
    }

    /// `verify_skill_manifest()`: judge every record against the contract, prune the non-compliant
    /// ones (the oracle's self-heal writes back only compliant records), derive the three-state status
    /// and record the outcome in the bounded audit log.
    pub fn verify_skill_manifest(&mut self) -> AuditResult {
        self.audit_count += 1;
        // Counted before the drain, because `drain` empties the vector it is called on.
        let before = self.entries.len();
        let mut errors = 0usize;
        let mut kept: Vec<SkillManifestEntry> = Vec::new();
        for entry in self.entries.drain(..) {
            let errs = contract_errors(&entry.name, &entry.description, &entry.instructions);
            if errs.is_empty() {
                kept.push(entry);
            } else {
                errors += errs.len();
            }
        }
        let pruned = before - kept.len();
        self.entries = kept;
        let healed = pruned > 0;
        if healed {
            self.heal_count += 1;
            // Pruning everything is how the oracle recreates a clean default manifest (`skills: []`).
            // It does NOT invent a "baseline_core" record, which is what the previous port did.
        }
        let status = match (errors > 0, healed) {
            (true, true) => ManifestComplianceStatus::Warning,
            (true, false) => ManifestComplianceStatus::Fail,
            (false, true) => ManifestComplianceStatus::Warning,
            (false, false) => ManifestComplianceStatus::Pass,
        };
        let result = AuditResult { status, healed, pruned, errors };
        self.audit_log.push(result);
        if self.audit_log.len() > AUDIT_LOG_CAP {
            self.audit_log.remove(0);
        }
        result
    }

    /// `get_audit_log()`: the most recent results, capped at 50 like the oracle.
    pub fn get_audit_log(&self) -> &[AuditResult] {
        &self.audit_log
    }

    pub fn get_audit_metrics(&self) -> (u64, u64, usize) {
        (self.audit_count, self.heal_count, self.entries.len())
    }
}

static SINGLETON_SKILL_VALIDATOR: AtomicU64 = AtomicU64::new(0);

// ── The kernel's live validator ─────────────────────────────────────────────────────────────────
// Same reason as the skill matrix: a port nothing calls is stripped by LTO and the EFI is
// byte-identical without it. The `skill-validate` command audits the live instance and prints the
// status + log depth, which is what makes the ported rules load-bearing.
static mut LIVE_VALIDATOR: Option<SkillValidator> = None;
static LIVE_VALIDATOR_READY: AtomicU64 = AtomicU64::new(0);

/// The one validator the EFI audits with. Single-threaded firmware, so the access is not shared.
pub fn live_validator() -> &'static mut SkillValidator {
    unsafe {
        if LIVE_VALIDATOR_READY.load(Ordering::Relaxed) == 0 {
            LIVE_VALIDATOR = Some(SkillValidator::new());
            LIVE_VALIDATOR_READY.store(1, Ordering::Relaxed);
        }
        match LIVE_VALIDATOR.as_mut() {
            Some(v) => v,
            None => core::unreachable!(),
        }
    }
}

/// Global handle initialization.
pub fn get_skill_validator() -> u64 {
    let v = SINGLETON_SKILL_VALIDATOR.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SKILL_VALIDATOR.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// Query skill validator configuration and telemetry.
pub fn skill_validator_telemetry() -> (u64, u32, u32, f32) {
    (MONAD_ANCHOR, VOLTAGE_PHASE, VOLTAGE_RANK, VOLTAGE_SCALE_FACTOR)
}
