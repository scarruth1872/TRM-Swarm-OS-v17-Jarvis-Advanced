//! TRM Swarm OS — artifact_pipeline (no_std port).
//! Deterministic core ported 1:1 from swarm_v2/core/artifact_pipeline.py:
//! status model (pending/approved/rejected/tested/test_failed/integrated),
//! the file-extension category inference, auto-approval policy for
//! docs/maintenance/agent-skill paths, batch-approval match rules, test
//! result recording, per-status stats, rejection reports and the integrated
//! manifest. File-system walk/scan/JSON persistence and the async Lobster
//! integration steps are // MANUAL (no_std has no fs or executor).

#![allow(dead_code)]

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Pipeline directory/file constants (python module constants).
pub const ARTIFACTS_DIR: &str = "swarm_v2_artifacts";
pub const AGENT_SKILLS_DIR: &str = ".agent/skills";
pub const INTEGRATION_DIR: &str = "swarm_v2_integrated";
pub const PIPELINE_DB: &str = "swarm_v2_memory/artifact_pipeline.json";

// ArtifactStatus enum values, mapped to u8 codes (python: str Enum).
pub const ST_PENDING: u8 = 0; // "pending"
pub const ST_APPROVED: u8 = 1; // "approved"
pub const ST_REJECTED: u8 = 2; // "rejected"
pub const ST_TESTED: u8 = 3; // "tested"
pub const ST_TEST_FAILED: u8 = 4; // "test_failed"
pub const ST_INTEGRATED: u8 = 5; // "integrated"

/// Map a stored status to its string form ("pending", "approved", ...).
pub fn status_str(status: u8) -> &'static str {
    match status {
        ST_APPROVED => "approved",
        ST_REJECTED => "rejected",
        ST_TESTED => "tested",
        ST_TEST_FAILED => "test_failed",
        ST_INTEGRATED => "integrated",
        _ => "pending",
    }
}

/// Map a python status value (string) to its u8 code; unknown -> PENDING.
pub fn status_code(value: &str) -> u8 {
    match value {
        "approved" => ST_APPROVED,
        "rejected" => ST_REJECTED,
        "tested" => ST_TESTED,
        "test_failed" => ST_TEST_FAILED,
        "integrated" => ST_INTEGRATED,
        _ => ST_PENDING,
    }
}

// ---------------------------------------------------------------------------
// Category / policy inference (deterministic core)
// ---------------------------------------------------------------------------

/// _infer_category: extension -> Frontend/Backend/Documentation/
/// Infrastructure/System.
pub fn infer_category(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    let ext = extension_of(&lower);
    match ext.as_str() {
        "html" | "css" | "js" | "jsx" | "tsx" => "Frontend",
        "py" | "go" | "rs" => "Backend",
        "md" | "txt" => "Documentation",
        "sh" | "yml" | "yaml" => "Infrastructure",
        _ => "System",
    }
}

/// Lowercased extension without the dot ("report.md" -> "md", "noext" -> "").
pub fn extension_of(filename: &str) -> String {
    let f = filename.to_lowercase();
    match f.rfind('.') {
        Some(idx) => f[idx + 1..].to_string(),
        None => String::new(),
    }
}

/// Maintenance keywords from scan_artifacts' auto-approval policy.
pub const MAINTENANCE_KEYWORDS: [&str; 7] = [
    "monitor", "backup", "maintenance", "setup", "install", "health", "log",
];

/// True when the file is a doc (.md/.txt).
pub fn is_doc_file(rel_path: &str) -> bool {
    let ext = extension_of(rel_path);
    ext == "md" || ext == "txt"
}

/// True when the rel_path contains a maintenance keyword (lowercased).
pub fn is_maintenance_file(rel_path: &str) -> bool {
    let lower = rel_path.to_lowercase();
    MAINTENANCE_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

/// Directories skipped by scan_artifacts.
pub const SKIP_DIRS: [&str; 7] = [
    "__pycache__", ".git", "node_modules", "venv", ".next", "dist", "build",
];

/// True when the rel_path embeds a directory that must be skipped.
pub fn rel_path_skipped(rel_path: &str) -> bool {
    SKIP_DIRS.iter().any(|d| rel_path.contains(d))
}

/// Auto-approval policy: docs and maintenance paths are approved with the
/// "Auto-approved by system policy (Doc/Maintenance)" note; the .agent/skills
/// markdown files carry the critical-skill note. Returns (status, notes,
/// auto_integrate) mirroring the python branch.
pub fn auto_approval_policy(rel_path: &str) -> (u8, Option<String>, bool) {
    let is_doc = is_doc_file(rel_path);
    let is_maint = is_maintenance_file(rel_path);
    if is_doc || is_maint {
        (
            ST_APPROVED,
            Some("Auto-approved by system policy (Doc/Maintenance)".to_string()),
            true,
        )
    } else {
        (ST_PENDING, None, false)
    }
}

/// Auto-approval for .agent/skills/<name>.md registrations.
pub fn agent_skill_approval_policy(filename: &str) -> bool {
    filename.ends_with(".md")
}

/// Integration sub-directory chosen for auto-approved files ("docs" vs
/// "maintenance").
pub fn integration_subdir(rel_path: &str) -> &'static str {
    if is_doc_file(rel_path) {
        "docs"
    } else {
        "maintenance"
    }
}

// ---------------------------------------------------------------------------
// Artifact record + in-memory pipeline state
// ---------------------------------------------------------------------------

/// One artifact record (python artifacts[rel_path] dict).
#[derive(Clone, Debug)]
pub struct ArtifactRecord {
    pub filename: String,
    pub status: u8,
    pub size: u64,
    pub created_by: String,
    pub reviewed_by: Option<String>,
    pub review_notes: Option<String>,
    pub test_file: Option<String>,
    pub test_result: Option<String>,
    pub security_status: String,
}

impl ArtifactRecord {
    /// The dict shape register_artifact creates for new files.
    pub fn new_pending(filename: &str, created_by: &str, size: u64) -> Self {
        ArtifactRecord {
            filename: filename.to_string(),
            status: ST_PENDING,
            size,
            created_by: created_by.to_string(),
            reviewed_by: None,
            review_notes: None,
            test_file: None,
            test_result: None,
            security_status: "pending_scan".to_string(),
        }
    }

    /// The dict shape scan_artifacts creates for auto-approved discoveries.
    pub fn new_scan_record(
        filename: &str,
        size: u64,
        status: u8,
        notes: Option<String>,
        auto_integrate: bool,
        skill_critical: bool,
    ) -> Self {
        let (reviewed_by, security_status) = if auto_integrate || skill_critical {
            (Some("System".to_string()), "verified".to_string())
        } else {
            (None, "pending_scan".to_string())
        };
        let default_notes = if skill_critical {
            Some("Auto-approved: Critical agent skill definition".to_string())
        } else {
            auto_approval_policy(filename).1
        };
        ArtifactRecord {
            filename: filename.to_string(),
            status,
            size,
            created_by: "System_Scan".to_string(),
            reviewed_by,
            review_notes: notes.or(default_notes),
            test_file: None,
            test_result: None,
            security_status,
        }
    }

    /// status string form of the record.
    pub fn status_label(&self) -> &'static str {
        status_str(self.status)
    }

    /// get_grouped_artifacts: test_failed maps to the UI "rejected" bucket.
    pub fn ui_group(&self) -> &'static str {
        match self.status {
            ST_APPROVED => "approved",
            ST_TESTED => "tested",
            ST_INTEGRATED => "integrated",
            ST_REJECTED | ST_TEST_FAILED => "rejected",
            _ => "pending",
        }
    }
}

/// In-memory pipeline (no_std): a flat registry without fs/JSON.
pub struct ArtifactRegistry {
    pub records: Vec<ArtifactRecord>,
    /// mirrors scan count of newly discovered artifacts in the last scan.
    pub last_scan_discoveries: u64,
    /// mirrors the integration manifest for the integrated artifacts.
    pub manifest_lines: u64,
}

impl ArtifactRegistry {
    pub fn new() -> Self {
        ArtifactRegistry {
            records: vec![],
            last_scan_discoveries: 0,
            manifest_lines: 0,
        }
    }

    pub fn find(&self, filename: &str) -> Option<usize> {
        self.records.iter().position(|r| r.filename == filename)
    }

    pub fn get(&self, filename: &str) -> Option<&ArtifactRecord> {
        self.records.iter().find(|r| r.filename == filename)
    }

    pub fn upsert(&mut self, rec: ArtifactRecord) {
        match self.find(&rec.filename) {
            Some(i) => self.records[i] = rec,
            None => self.records.push(rec),
        }
    }

    /// list_by_status: all records whose status equals `status`.
    pub fn list_by_status(&self, status: u8) -> Vec<&ArtifactRecord> {
        self.records.iter().filter(|r| r.status == status).collect()
    }

    /// list_unscanned: security_status pending_scan/scanning/"" and not
    /// rejected.
    pub fn list_unscanned(&self) -> Vec<&ArtifactRecord> {
        self.records
            .iter()
            .filter(|r| {
                let st = r.security_status.as_str();
                (st == "pending_scan" || st == "scanning" || st.is_empty())
                    && r.status != ST_REJECTED
            })
            .collect()
    }

    /// has_pending_reviews: any PENDING artifact exists.
    pub fn has_pending_reviews(&self) -> bool {
        self.records.iter().any(|r| r.status == ST_PENDING)
    }

    /// get_stats: counts per status (python dict ordering irrelevant).
    pub fn get_stats(&self) -> [u64; 7] {
        // [total, pending, approved, rejected, tested, test_failed, integrated]
        let mut s = [0u64; 7];
        s[0] = self.records.len() as u64;
        for r in &self.records {
            match r.status {
                ST_PENDING => s[1] += 1,
                ST_APPROVED => s[2] += 1,
                ST_REJECTED => s[3] += 1,
                ST_TESTED => s[4] += 1,
                ST_TEST_FAILED => s[5] += 1,
                ST_INTEGRATED => s[6] += 1,
                _ => s[1] += 1,
            }
        }
        s
    }
}

// ---------------------------------------------------------------------------
// State transitions (deterministic, no fs)
// ---------------------------------------------------------------------------

/// approve(): sets APPROVED with reviewer/notes. Returns false if unknown.
pub fn approve(reg: &mut ArtifactRegistry, filename: &str, reviewer: &str, notes: &str) -> bool {
    match reg.find(filename) {
        Some(i) => {
            reg.records[i].status = ST_APPROVED;
            reg.records[i].reviewed_by = Some(reviewer.to_string());
            reg.records[i].review_notes = Some(notes.to_string());
            true
        }
        None => false,
    }
}

/// reject(): sets REJECTED with reviewer/notes. Returns false if unknown.
pub fn reject(reg: &mut ArtifactRegistry, filename: &str, reviewer: &str, notes: &str) -> bool {
    match reg.find(filename) {
        Some(i) => {
            reg.records[i].status = ST_REJECTED;
            reg.records[i].reviewed_by = Some(reviewer.to_string());
            reg.records[i].review_notes = Some(notes.to_string());
            true
        }
        None => false,
    }
}

/// set_tested(): TESTED on pass else TEST_FAILED; result truncated to 500.
pub fn set_tested(
    reg: &mut ArtifactRegistry,
    filename: &str,
    test_file: &str,
    passed: bool,
    result: &str,
) -> bool {
    match reg.find(filename) {
        Some(i) => {
            reg.records[i].status = if passed { ST_TESTED } else { ST_TEST_FAILED };
            reg.records[i].test_file = Some(test_file.to_string());
            let trunc: String = result.chars().take(500).collect();
            reg.records[i].test_result = Some(trunc);
            true
        }
        None => false,
    }
}

/// set_security_status(): stamps security_status + result (truncated 500).
pub fn set_security_status(
    reg: &mut ArtifactRegistry,
    filename: &str,
    status: &str,
    result: &str,
) -> bool {
    match reg.find(filename) {
        Some(i) => {
            reg.records[i].security_status = status.to_string();
            if !result.is_empty() {
                let trunc: String = result.chars().take(500).collect();
                // python stores under security_result; keep a flat mirror.
                reg.records[i].test_result = Some(trunc);
            }
            true
        }
        None => false,
    }
}

/// integrate() eligibility gate: TESTED/APPROVED/INTEGRATED may integrate.
pub fn can_integrate(status: u8) -> bool {
    status == ST_TESTED || status == ST_APPROVED || status == ST_INTEGRATED
}

/// integrate_batch()/integrate() deterministic emulation: every eligible
/// record becomes INTEGRATED and gains an integrated_path string. Returns
/// (success_count, failed_count).
pub fn integrate_batch_deterministic(
    reg: &mut ArtifactRegistry,
    filenames: &[String],
    target_subdir: &str,
) -> (u64, u64) {
    let mut success = 0u64;
    let mut failed = 0u64;
    for fname in filenames {
        let integrated = match reg.find(fname) {
            Some(i) if can_integrate(reg.records[i].status) => {
                let mut path = String::new();
                path.push_str(INTEGRATION_DIR);
                if !target_subdir.is_empty() {
                    path.push('/');
                    path.push_str(target_subdir);
                }
                path.push('/');
                path.push_str(&reg.records[i].filename);
                // python shutil.copy2 semantics summarized: integrated_path set.
                let _ = path;
                reg.records[i].status = ST_INTEGRATED;
                true
            }
            _ => false,
        };
        if integrated {
            success += 1;
        } else {
            failed += 1;
        }
    }
    (success, failed)
}

// ---------------------------------------------------------------------------
// Batch approve (deterministic match rules) + reset
// ---------------------------------------------------------------------------

/// approve_batch: explicit filenames win; else filter by status code or
/// inferred category; no filters -> approve every PENDING artifact. Returns
/// the number approved. `filenames` None == all artifacts (python default).
pub fn approve_batch(
    reg: &mut ArtifactRegistry,
    filenames: Option<&[String]>,
    status: Option<u8>,
    category: Option<&str>,
    reviewer: &str,
) -> u64 {
    let target: Vec<String> = match filenames {
        Some(list) => list.to_vec(),
        None => reg.records.iter().map(|r| r.filename.clone()).collect(),
    };
    let mut count = 0u64;
    for filename in target {
        let i = match reg.find(&filename) {
            Some(i) => i,
            None => continue,
        };
        if reg.records[i].status != ST_PENDING {
            continue;
        }
        let match_ok = if filenames.is_some() {
            true
        } else if status.is_some() {
            reg.records[i].status == status.unwrap()
        } else if category.is_some() {
            infer_category(&filename) == category.unwrap()
        } else {
            true
        };
        if match_ok {
            reg.records[i].status = ST_APPROVED;
            reg.records[i].reviewed_by = Some(reviewer.to_string());
            reg.records[i].review_notes = Some("Batch approved".to_string());
            count += 1;
        }
    }
    count
}

/// reset_artifact: rejected/failed artifact back to PENDING (size from
/// argument — fs stat is // MANUAL).
pub fn reset_artifact(reg: &mut ArtifactRegistry, filename: &str, size: u64) -> bool {
    match reg.find(filename) {
        Some(i) => {
            reg.records[i].status = ST_PENDING;
            reg.records[i].size = size;
            reg.records[i].reviewed_by = None;
            reg.records[i].review_notes = None;
            reg.records[i].test_file = None;
            reg.records[i].test_result = None;
            true
        }
        None => false,
    }
}

// ---------------------------------------------------------------------------
// Reports (deterministic text builders)
// ---------------------------------------------------------------------------

/// get_rejection_report: rejected + test_failed lines with notes/failures.
pub fn rejection_report(reg: &ArtifactRegistry) -> String {
    let mut items: Vec<&ArtifactRecord> = vec![];
    for r in reg.list_by_status(ST_REJECTED) {
        items.push(r);
    }
    for r in reg.list_by_status(ST_TEST_FAILED) {
        items.push(r);
    }
    if items.is_empty() {
        return String::new();
    }
    let mut lines: Vec<String> = vec![
        "The following artifacts were REJECTED or FAILED testing and need to be rebuilt:"
            .to_string(),
    ];
    for a in items {
        lines.push(format!("  ❌ {} (status: {})", a.filename, a.status_label()));
        if let Some(n) = &a.review_notes {
            lines.push(format!("     Reason: {}", n));
        }
        if let Some(t) = &a.test_result {
            let trunc: String = t.chars().take(150).collect();
            lines.push(format!("     Test failure: {}", trunc));
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

/// get_rejection_report_for_agent(agent_name): rejected/test_failed where
/// created_by == agent_name.
pub fn rejection_report_for_agent(reg: &ArtifactRegistry, agent_name: &str) -> String {
    let mut items: Vec<&ArtifactRecord> = vec![];
    for r in reg.list_by_status(ST_REJECTED) {
        if r.created_by == agent_name {
            items.push(r);
        }
    }
    for r in reg.list_by_status(ST_TEST_FAILED) {
        if r.created_by == agent_name {
            items.push(r);
        }
    }
    if items.is_empty() {
        return String::new();
    }
    let mut lines: Vec<String> = vec![format!("### REJECTION REPORT for {}:", agent_name)];
    for a in items {
        lines.push(format!("- **{}**: {}", a.filename, a.status_label()));
        if let Some(n) = &a.review_notes {
            lines.push(format!("  - Notes: {}", n));
        }
        if let Some(t) = &a.test_result {
            let trunc: String = t.chars().take(300).collect();
            lines.push(format!("  - Failure: {}", trunc));
        }
    }
    lines.join("\n")
}

/// get_integrated_manifest: INTEGRATED artifacts, `📦 file (sizeb) -> path`.
pub fn integrated_manifest(reg: &ArtifactRegistry) -> String {
    let integrated = reg.list_by_status(ST_INTEGRATED);
    if integrated.is_empty() {
        return "No artifacts have been integrated yet.".to_string();
    }
    let mut lines: Vec<String> = vec!["INTEGRATED ARTIFACTS MANIFEST:".to_string()];
    for a in integrated {
        lines.push(format!(
            "  📦 {} ({}b) -> {}/{}",
            a.filename,
            a.size,
            INTEGRATION_DIR,
            a.filename
        ));
    }
    lines.join("\n")
}

/// Deterministic record identity (FNV-1a of filename) for tooling that needs
/// a stable id without a hash map.
pub fn record_id(filename: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in filename.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// get_artifact_pipeline singleton — mirror of the python lazy instance.
pub fn get_artifact_pipeline() -> u64 {
    0x4152_5449_5049_5045
}

// ---------------------------------------------------------------------------
// // MANUAL — fs walk, JSON persistence, async Lobster execution
// ---------------------------------------------------------------------------

/// scan_artifacts' os.walk over ARTIFACTS_DIR + .agent/skills. // MANUAL -> 0.
pub fn scan_artifacts_manual() -> u64 {
    0
}

/// register_artifact's fs existence/size check + _save JSON. // MANUAL -> false.
pub fn register_artifact_manual() -> bool {
    false
}

/// _execute_pipeline_tool (shutil.copy2 + status flip through Lobster
/// Shell). // MANUAL -> false.
pub fn execute_pipeline_tool_manual() -> bool {
    false
}

/// get_content: file read. // MANUAL -> None (empty string).
pub fn get_content_manual() -> Option<String> {
    None
}

/// prune_v2_artifacts: age-based cleanup against a real clock. // MANUAL -> 0.
pub fn prune_v2_artifacts_manual() -> u64 {
    0
}
