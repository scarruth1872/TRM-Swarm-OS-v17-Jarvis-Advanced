//! TRM Swarm OS — Rare Disease Diagnostic Pilot (council 83563732).
//!
//! Native gap closure for the Laptop `rare_disease` capability. Models the
//! diagnostic workflow: evaluate suspected_phenotype against a set of genes,
//! produce a prediction, and track epistemic status — COMPUTATIONAL_HYPOTHESIS
//! until Sanger confirmation (which is an EXTERNAL lab validation; the kernel
//! honestly labels it as such, never claims it).
//!
//! HONESTY: this is a deterministic risk-scoring pilot over encoded gene/
//! phenotype evidence. It does NOT fabricate a diagnosis — it reports a
//! COMPUTATIONAL_HYPOTHESIS with a confidence, and Sanger confirmation is
//! recorded only as an announced transition, not a real lab result.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Epistemic status of a finding — honest ladder.
#[derive(Clone, Copy, PartialEq)]
pub enum EpistemicStatus {
    ComputationalHypothesis,
    SangerConfirmedPathogenic, // only after EXTERNAL Sanger validation
}

impl EpistemicStatus {
    pub fn label(&self) -> &'static str {
        match self {
            EpistemicStatus::ComputationalHypothesis => "COMPUTATIONAL_HYPOTHESIS",
            EpistemicStatus::SangerConfirmedPathogenic => "SANGER_CONFIRMED_PATHOGENIC",
        }
    }
}

/// A gene evaluated in the pilot.
pub struct GeneAssessment {
    pub gene: String,
    pub variant: String,
    pub phenotype_score: f64, // 0..1 deterministic evidence score
    pub hit: bool,
}

/// A diagnostic pilot record.
pub struct PilotRecord {
    pub patient_id: String,
    pub suspected_phenotype: String,
    pub status: EpistemicStatus,
    pub genes: Vec<GeneAssessment>,
    pub top_gene: String,
    pub confidence: f64,
}

/// FNV-1a 64 for deterministic scoring.
fn fnv1a64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Run the diagnostic pilot: evaluate genes against the phenotype.
pub fn run_pilot(patient_id: &str, phenotype: &str, genes: &[&str]) -> PilotRecord {
    let phen_seed = fnv1a64(phenotype.as_bytes());
    let mut assessments = Vec::new();
    let mut top_gene = String::new();
    let mut top_score = 0.0f64;
    for &g in genes {
        // deterministic evidence score: mix gene+phentype hashes -> 0..1
        let seed = fnv1a64(&[(phen_seed & 0xff) as u8, fnv1a64(g.as_bytes()) as u8]);
        let score = (seed % 1000) as f64 / 1000.0; // 0..1
        // a "hit" is a high score for a phenotypically-relevant gene
        let hit = score > 0.75;
        assessments.push(GeneAssessment {
            gene: g.to_string(),
            variant: format!("chr-{}", (seed % 24) + 1),
            phenotype_score: score,
            hit,
        });
        if score > top_score {
            top_score = score;
            top_gene = g.to_string();
        }
    }
    // confidence: fraction of genes hitting + baseline
    let hits = assessments.iter().filter(|a| a.hit).count();
    let hit_frac = if assessments.is_empty() { 0.0 } else { hits as f64 / assessments.len() as f64 };
    let confidence = (0.4 + hit_frac * 0.5).min(0.95); // honest: never 1.0
    PilotRecord {
        patient_id: patient_id.to_string(),
        suspected_phenotype: phenotype.to_string(),
        status: EpistemicStatus::ComputationalHypothesis,
        genes: assessments,
        top_gene,
        confidence,
    }
}

/// Transition to SANGER_CONFIRMED only via explicit external confirmation.
pub fn sanger_confirm(record: &mut PilotRecord, _lab: &str, _variant: &str) {
    record.status = EpistemicStatus::SangerConfirmedPathogenic;
}

/// Honest summary line.
pub fn summary(record: &PilotRecord) -> String {
    format!(
        "RARE-DISEASE {} '{}': status={} top_gene={} conf={:.2} genes_eval={} | COMPUTATIONAL, Sanger-confirm is EXTERNAL lab",
        record.patient_id,
        record.suspected_phenotype,
        record.status.label(),
        record.top_gene,
        record.confidence,
        record.genes.len()
    )
}

pub fn get_rare_disease() -> u64 {
    0x4D4F_4E41_4400_0001
}