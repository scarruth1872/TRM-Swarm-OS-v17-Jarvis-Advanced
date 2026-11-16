//! PHOENIX Protocol — 3-phase design structure and target registry.
//!
//! Port of `swarm_v2/core/phoenix_protocol_engine.py`, with an important
//! difference in what it will and will not return.
//!
//! WHY THIS PORT DIFFERS FROM THE PYTHON (see
//! docs/FINDING-phoenix-protocol-efficacy-claims-are-asserted.md):
//!
//! The Python's `execute_full_phoenix_protocol()` returns
//!
//!     "predicted_cure_rate_pct": 96.8
//!     "remission_durability":    "> 36 Months Drug-Free"
//!     "bbb_penetration_rate":     94.8
//!     "marrow_endosteum_depletion": 99.2
//!     "tlr7_knockdown_pct":       96.4
//!     "ifn_neutralization_pct":   98.7
//!
//! None of those numbers is computed. The module imports no docking or affinity
//! code, contains no arithmetic beyond `perf_counter` deltas around dict
//! construction, and the generator script that produced its metrics JSON
//! hardcodes `"cure_rate_pct": 96.8` as a literal. The 10 binding affinities are
//! a hardcoded list, fed directly. The only real operation in the pipeline is
//! `np.random.randn(128)` — a RANDOM vector.
//!
//! A kernel must not report a cure rate it did not compute. So this port carries
//! what is real and labels the rest:
//!
//!   REAL  - the 3-phase structure (a coherent design)
//!         - the 10 gene targets, which are genuine lupus-associated loci with
//!           correct UniProt accessions
//!         - the step-latency measurement, which the CALLER supplies because
//!           only the caller can measure it
//!   LABELLED - every efficacy figure is returned inside `DesignTarget`, whose
//!           `status` is always `UNVALIDATED_DESIGN_TARGET`. The type makes the
//!           epistemic state impossible to drop accidentally.
//!
//! Nothing in this module will tell you a protocol succeeded.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const PROTOCOL_VERSION: &str = "v18.1.0-PHOENIX";

/// The only status any efficacy figure in this module may carry.
pub const UNVALIDATED: &str = "UNVALIDATED_DESIGN_TARGET";

/// A stated design goal. NOT a measured or predicted result.
///
/// The Python returned bare floats under keys like `predicted_cure_rate_pct`,
/// where the name itself asserts a prediction. Here the value and its epistemic
/// status travel together, so a caller cannot read the number without the label.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignTarget {
    pub name: String,
    pub value: f64,
    pub unit: String,
    /// Always `UNVALIDATED`. Present so callers must decide what to do with an
    /// unvalidated figure rather than mistaking it for an output.
    pub status: String,
    /// What would be required to validate it. Empty means nobody has stated one.
    pub would_require: String,
}

impl DesignTarget {
    fn new(name: &str, value: f64, unit: &str, would_require: &str) -> Self {
        Self {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            status: UNVALIDATED.to_string(),
            would_require: would_require.to_string(),
        }
    }
    pub fn is_validated(&self) -> bool { false }
}

/// A real gene target. Symbols and UniProt accessions are genuine
/// lupus-associated loci; the affinity is NOT a computed docking score.
#[derive(Clone, Debug, PartialEq)]
pub struct GeneTarget {
    pub symbol: String,
    pub uniprot: String,
    pub full_name: String,
}

/// The 10 PHOENIX targets. Domain knowledge, kept — these are real loci.
/// CORRECTED 2026-09-22: the original table carried P42226 for STAT4 (which
/// resolves to STAT6) and Q9NYB2 for TLR7 (which resolves to CASK). Both were
/// asserted without verification. Verified against the UniProt REST API.
pub const PHOENIX_TARGETS: &[(&str, &str, &str)] = &[
    ("STAT4",   "Q14765", "Signal Transducer & Activator of Transcription 4"),
    ("TYK2",    "P29597", "Non-receptor Tyrosine-Protein Kinase TYK2"),
    ("TNFAIP3", "P21580", "Tumor Necrosis Factor Alpha-Induced Protein 3 (A20)"),
    ("TLR7",    "Q9NYK1", "Toll-like Receptor 7 (Endosomal RNA Sensor)"),
    ("IFIH1",   "Q9BYX4", "Interferon Induced with Helicase C Domain 1 (MDA5)"),
    ("IRF5",    "Q13568", "Interferon Regulatory Factor 5"),
    ("PTPN22",  "Q9Y2R2", "Tyrosine-protein Phosphatase Non-receptor Type 22"),
    ("ITGAM",   "P11215", "Integrin Alpha M (CD11b)"),
    ("BLK",     "P51451", "B Lymphoid Tyrosine Kinase"),
    ("C1QA",    "P02745", "Complement C1q Subcomponent Subunit A"),
];

pub fn target_registry() -> Vec<GeneTarget> {
    PHOENIX_TARGETS.iter()
        .map(|(s, u, n)| GeneTarget {
            symbol: s.to_string(), uniprot: u.to_string(), full_name: n.to_string() })
        .collect()
}

pub fn find_target(symbol: &str) -> Option<GeneTarget> {
    PHOENIX_TARGETS.iter()
        .find(|(s, _, _)| s.eq_ignore_ascii_case(symbol))
        .map(|(s, u, n)| GeneTarget {
            symbol: s.to_string(), uniprot: u.to_string(), full_name: n.to_string() })
}

/// Phase I — Ares Breach. Structure only; efficacy figures are unvalidated.
pub fn phase_ares_breach() -> (String, String, Vec<String>, Vec<DesignTarget>) {
    (
        "I - The Ares Breach".to_string(),
        "TfR-Functionalized Polymeric Nanoparticles (PNPs)".to_string(),
        alloc::vec!["CD19 (B-Cells)".to_string(), "BCMA (Long-Lived Plasma Cells)".to_string()],
        alloc::vec![
            DesignTarget::new("bbb_penetration_rate", 94.8, "pct",
                "an in vivo BBB permeability study; no cohort, model or assay is on record"),
            DesignTarget::new("marrow_endosteum_depletion", 99.2, "pct",
                "an in vivo marrow depletion assay; no cohort, model or assay is on record"),
        ],
    )
}

/// Phase II — Athena Shield.
pub fn phase_athena_shield() -> (String, String, Vec<String>, Vec<DesignTarget>, u32) {
    (
        "II - The Athena Shield".to_string(),
        "LNP-Delivered CRISPR-Cas13 (RNA-Targeting)".to_string(),
        alloc::vec!["TLR7 mRNA".to_string(), "IFN-alpha mRNA".to_string()],
        alloc::vec![
            DesignTarget::new("tlr7_knockdown_pct", 96.4, "pct",
                "an in vitro/in vivo knockdown assay; none is on record"),
            DesignTarget::new("ifn_neutralization_pct", 98.7, "pct",
                "an IFN-alpha neutralization assay; none is on record"),
        ],
        75,   // cas13_window_days - a design parameter, not a measurement
    )
}

/// Phase III — Hera Watch.
pub fn phase_hera_watch() -> (String, String, Vec<String>, Vec<DesignTarget>, u32) {
    (
        "III - The Hera Watch".to_string(),
        "20 Synthetic Aptamer Nanoprobes".to_string(),
        alloc::vec![
            "sCD25".to_string(), "CXCL10".to_string(), "BAFF/BlyS".to_string(),
            "Anti-dsDNA Titers".to_string(), "Treg/Th17 Ratio".to_string(),
        ],
        alloc::vec![
            DesignTarget::new("pre_symptomatic_lead_time_days", 14.0, "days",
                "a longitudinal patient cohort with a pre-symptomatic arm; none is on record"),
        ],
        20,   // aptamer panel size - design parameter
    )
}

/// The overall efficacy statement, returned as UNVALIDATED targets.
///
/// The Python returned `predicted_cure_rate_pct: 96.8` from a function named
/// `execute_full_phoenix_protocol`. This port returns the same figure as an
/// explicitly unvalidated design target and will not describe it as a
/// prediction or a result.
pub fn protocol_design_targets() -> Vec<DesignTarget> {
    alloc::vec![
        DesignTarget::new("cure_rate_pct", 96.8, "pct",
            "a controlled clinical trial with a defined cohort, blinded endpoint and follow-up.              No model, simulation or cohort exists in this repository that produces this number -              it is a hardcoded literal in research_phoenix_protocol_lupus_cure.py"),
        DesignTarget::new("remission_durability_months", 36.0, "months",
            "a longitudinal trial past the 36-month mark; the Python states '> 36 Months' as a string literal"),
    ]
}

/// Record a real GPU step-latency measurement. This is the only quantity in the
/// protocol pipeline that is genuinely measured, and only the CALLER can measure
/// it - so it is a parameter here, never a constant.
#[derive(Clone, Debug, PartialEq)]
pub struct StepLatency {
    pub gpu_name: String,
    pub gpu_cores: u32,
    pub step_latency_us: f64,
    pub measured: bool,
}

/// Build a step-latency record from a real measurement. Deliberately has no
/// default and no constant: if you did not measure it, you cannot construct one.
pub fn record_step_latency(gpu_name: &str, gpu_cores: u32, step_latency_us: f64) -> StepLatency {
    StepLatency {
        gpu_name: gpu_name.to_string(), gpu_cores, step_latency_us, measured: true,
    }
}

/// Summary for display. States plainly that no efficacy figure was computed.
pub fn protocol_summary() -> String {
    alloc::format!(
        "PHOENIX {} | {} gene targets (real loci) | 3 phases (design) |          efficacy figures: {} - NOT COMPUTED",
        PROTOCOL_VERSION, PHOENIX_TARGETS.len(), UNVALIDATED)
}
