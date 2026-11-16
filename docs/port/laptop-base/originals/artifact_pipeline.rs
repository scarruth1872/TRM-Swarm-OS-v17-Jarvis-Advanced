//! TRM Swarm OS - artifact_pipeline (Deterministic bare-metal no_std + alloc implementation).
//! Fully mirrors swarm_v2/core/artifact_pipeline.py & strip_artifact_scorer.rs.
//! Manages artifact lifecycles: Pending -> Approved -> Tested -> Integrated,
//! with Monad cryptographic sealing, FNV-1a content scoring, and category routing.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

// --- CONSTANTS ---
pub const ARTIFACTS_DIR: &'static str = "swarm_v2_artifacts";
pub const AGENT_SKILLS_DIR: &'static str = ".agent/skills";
pub const INTEGRATION_DIR: &'static str = "swarm_v2_integrated";
pub const PIPELINE_DB: &'static str = "swarm_v2_memory/artifact_pipeline.json";
pub const MONAD_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

// --- ARTIFACT STATUS ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactStatus {
    Pending,
    Approved,
    Rejected,
    Tested,
    TestFailed,
    Integrated,
}

impl ArtifactStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactStatus::Pending => "pending",
            ArtifactStatus::Approved => "approved",
            ArtifactStatus::Rejected => "rejected",
            ArtifactStatus::Tested => "tested",
            ArtifactStatus::TestFailed => "test_failed",
            ArtifactStatus::Integrated => "integrated",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "approved" => ArtifactStatus::Approved,
            "rejected" => ArtifactStatus::Rejected,
            "tested" => ArtifactStatus::Tested,
            "test_failed" => ArtifactStatus::TestFailed,
            "integrated" => ArtifactStatus::Integrated,
            _ => ArtifactStatus::Pending,
        }
    }
}

// --- ARTIFACT CATEGORY ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactCategory {
    Frontend,
    Backend,
    Documentation,
    Infrastructure,
    System,
}

impl ArtifactCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactCategory::Frontend => "Frontend",
            ArtifactCategory::Backend => "Backend",
            ArtifactCategory::Documentation => "Documentation",
            ArtifactCategory::Infrastructure => "Infrastructure",
            ArtifactCategory::System => "System",
        }
    }

    pub fn infer_from_filename(filename: &str) -> Self {
        match infer_category(filename) {
            "Frontend" => ArtifactCategory::Frontend,
            "Backend" => ArtifactCategory::Backend,
            "Documentation" => ArtifactCategory::Documentation,
            "Infrastructure" => ArtifactCategory::Infrastructure,
            _ => ArtifactCategory::System,
        }
    }
}

/// `_infer_category` — `os.path.splitext(filename)[1].lower()` then an exact-match table.
///
/// `splitext` splits at the LAST dot, with the leading-dot exception (a dot at position 0
/// is part of the basename, not an extension), and the oracle LOWER-CASES the extension, so
/// `.TSX` and `.tsx` both route to Frontend. The prior `infer_from_filename` used case-sensitive
/// `ends_with`, which misrouted `App.TSX` to System — this is the oracle-faithful version.
pub fn infer_category(filename: &str) -> &'static str {
    let ext: String = match filename.rfind('.') {
        Some(i) if i > 0 => filename[i..].to_ascii_lowercase(),
        _ => String::new(),
    };
    match ext.as_str() {
        ".html" | ".css" | ".js" | ".jsx" | ".tsx" => "Frontend",
        ".py" | ".go" | ".rs" => "Backend",
        ".md" | ".txt" => "Documentation",
        ".sh" | ".yml" | ".yaml" => "Infrastructure",
        _ => "System",
    }
}

// --- ARTIFACT METADATA ---
#[derive(Debug, Clone)]
pub struct ArtifactRecord {
    pub filename: String,
    pub status: ArtifactStatus,
    pub category: ArtifactCategory,
    pub size_bytes: usize,
    pub created_at_epoch_s: u64,
    pub created_by: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at_epoch_s: Option<u64>,
    pub review_notes: Option<String>,
    pub test_result: Option<String>,
    pub integrated_path: Option<String>,
    pub fnv_checksum: u32,
    pub alphanumeric_density: f32,
    pub monad_seal: u64,
    pub security_verified: bool,
}

impl ArtifactRecord {
    pub fn is_actionable_for_integration(&self) -> bool {
        self.status == ArtifactStatus::Approved || self.status == ArtifactStatus::Tested
    }
}

// --- NATIVE ARTIFACT CONTENT SCORER (fnv-1a + alphanumeric density) ---
pub fn score_artifact_content(buffer: &[u8]) -> (u32, f32) {
    if buffer.is_empty() {
        return (2166136261, 0.0);
    }
    let mut h: u32 = 2166136261;
    let mut alphanum_count: usize = 0;

    for &b in buffer {
        h ^= b as u32;
        h = h.wrapping_mul(16777619);
        if (b >= b'A' && b <= b'Z') || (b >= b'a' && b <= b'z') || (b >= b'0' && b <= b'9') {
            alphanum_count += 1;
        }
    }

    let density = (alphanum_count as f32) / (buffer.len() as f32);
    (h, density)
}

// --- MONAD SEAL GENERATOR ---
pub fn generate_monad_seal(filename: &str, fnv_checksum: u32) -> u64 {
    let mut name_h: u64 = 0xcbf29ce484222325;
    for &b in filename.as_bytes() {
        name_h ^= b as u64;
        name_h = name_h.wrapping_mul(0x100000001b3);
    }
    let combined = name_h ^ ((fnv_checksum as u64) << 32 | (fnv_checksum as u64));
    MONAD_ANCHOR ^ combined
}

// --- PIPELINE MANAGER ---
pub struct ArtifactPipeline {
    records: Vec<ArtifactRecord>,
    last_scan_epoch_s: u64,
}

impl ArtifactPipeline {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            last_scan_epoch_s: 0,
        }
    }

    pub fn register_artifact(
        &mut self,
        filename: &str,
        content: &[u8],
        created_by: &str,
        epoch_s: u64,
    ) -> usize {
        let (fnv, density) = score_artifact_content(content);
        let seal = generate_monad_seal(filename, fnv);
        let category = ArtifactCategory::infer_from_filename(filename);

        // Auto-approval policy for documentation, configs, and agent skills
        let is_doc = filename.ends_with(".md") || filename.ends_with(".txt");
        let is_skill = filename.starts_with(".agent/skills/");
        let auto_approve = is_doc || is_skill;

        let status = if auto_approve {
            ArtifactStatus::Approved
        } else {
            ArtifactStatus::Pending
        };

        let record = ArtifactRecord {
            filename: filename.to_string(),
            status,
            category,
            size_bytes: content.len(),
            created_at_epoch_s: epoch_s,
            created_by: created_by.to_string(),
            reviewed_by: if auto_approve { Some("System_Policy".to_string()) } else { None },
            reviewed_at_epoch_s: if auto_approve { Some(epoch_s) } else { None },
            review_notes: if auto_approve { Some("Auto-approved by bare-metal policy".to_string()) } else { None },
            test_result: None,
            integrated_path: None,
            fnv_checksum: fnv,
            alphanumeric_density: density,
            monad_seal: seal,
            security_verified: auto_approve,
        };

        if let Some(pos) = self.records.iter().position(|r| r.filename == filename) {
            self.records[pos] = record;
            pos
        } else {
            self.records.push(record);
            self.records.len() - 1
        }
    }

    pub fn approve(&mut self, filename: &str, reviewer: &str, notes: &str, epoch_s: u64) -> bool {
        if let Some(r) = self.records.iter_mut().find(|r| r.filename == filename) {
            r.status = ArtifactStatus::Approved;
            r.reviewed_by = Some(reviewer.to_string());
            r.reviewed_at_epoch_s = Some(epoch_s);
            r.review_notes = Some(notes.to_string());
            r.security_verified = true;
            true
        } else {
            false
        }
    }

    pub fn reject(&mut self, filename: &str, reviewer: &str, notes: &str, epoch_s: u64) -> bool {
        if let Some(r) = self.records.iter_mut().find(|r| r.filename == filename) {
            r.status = ArtifactStatus::Rejected;
            r.reviewed_by = Some(reviewer.to_string());
            r.reviewed_at_epoch_s = Some(epoch_s);
            r.review_notes = Some(notes.to_string());
            true
        } else {
            false
        }
    }

    pub fn set_tested(&mut self, filename: &str, passed: bool, result_msg: &str) -> bool {
        if let Some(r) = self.records.iter_mut().find(|r| r.filename == filename) {
            r.status = if passed { ArtifactStatus::Tested } else { ArtifactStatus::TestFailed };
            r.test_result = Some(result_msg.to_string());
            true
        } else {
            false
        }
    }

    pub fn integrate(&mut self, filename: &str, target_subdir: &str) -> Option<String> {
        if let Some(r) = self.records.iter_mut().find(|r| r.filename == filename) {
            if r.is_actionable_for_integration() {
                let dest = if target_subdir.is_empty() {
                    format!("{}/{}", INTEGRATION_DIR, filename)
                } else {
                    format!("{}/{}/{}", INTEGRATION_DIR, target_subdir, filename)
                };
                r.status = ArtifactStatus::Integrated;
                r.integrated_path = Some(dest.clone());
                Some(dest)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn batch_approve(&mut self, category_filter: Option<ArtifactCategory>, reviewer: &str, epoch_s: u64) -> usize {
        let mut count = 0;
        for r in self.records.iter_mut() {
            if r.status == ArtifactStatus::Pending {
                let matches = match category_filter {
                    Some(cat) => r.category == cat,
                    None => true,
                };
                if matches {
                    r.status = ArtifactStatus::Approved;
                    r.reviewed_by = Some(reviewer.to_string());
                    r.reviewed_at_epoch_s = Some(epoch_s);
                    r.review_notes = Some("Batch approved".to_string());
                    r.security_verified = true;
                    count += 1;
                }
            }
        }
        count
    }

    pub fn count_by_status(&self, status: ArtifactStatus) -> usize {
        self.records.iter().filter(|r| r.status == status).count()
    }

    pub fn get_record(&self, filename: &str) -> Option<&ArtifactRecord> {
        self.records.iter().find(|r| r.filename == filename)
    }

    pub fn total_artifacts(&self) -> usize {
        self.records.len()
    }

    pub fn verify_monad_seal(&self, filename: &str) -> bool {
        if let Some(r) = self.get_record(filename) {
            let expected_seal = generate_monad_seal(filename, r.fnv_checksum);
            r.monad_seal == expected_seal
        } else {
            false
        }
    }
}

// --- SINGLETON INTERFACE ---
static SINGLETON_ARTIFACT_PIPELINE: AtomicU64 = AtomicU64::new(0);

pub fn get_artifact_pipeline() -> u64 {
    let v = SINGLETON_ARTIFACT_PIPELINE.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_ARTIFACT_PIPELINE.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

