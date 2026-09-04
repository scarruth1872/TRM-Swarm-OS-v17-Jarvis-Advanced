//! TRM Swarm OS — tool_forge (no_std port).
//! Role-gap analysis & gap aggregation, 1:1 from swarm_v2/core/tool_forge.py:
//!   - ROLE_CAPABILITIES: expected skills per agent role (12 roles, 1:1);
//!   - assess_agent_needs(role, current_skills): expected − current, order
//!     preserved from the capability list;
//!   - aggregate_gaps: per-gap tool_name/needed_by/score with score =
//!     count of roles needing it, stable-sorted by score descending
//!     (python sorted(..., key=score, reverse=True) is stable → equal-score
//!     gaps keep first-seen order).
//! The multi-agent forge pipeline (Archi → Devo → Shield → QA → Scribe agent
//! process_task orchestration + learning-engine registration) is agent-bound
//! → honest `// MANUAL` skeleton, never invented.

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// ROLE_CAPABILITIES — expected skills per role (python module dict, 1:1).
pub const ROLE_CAPABILITIES: [(&str, &[&str]); 12] = [
    ("Architect", &["DiagramSkill", "SystemDesignSkill", "ConstraintSolver"]),
    (
        "Lead Developer",
        &["GitSkill", "RefactorSkill", "TestGeneratorSkill"],
    ),
    (
        "Researcher",
        &["CitationSkill", "SummarizationSkill", "TrendAnalysis"],
    ),
    (
        "Reasoning Engine",
        &["MathSolver", "LogicProver", "BenchmarkSkill"],
    ),
    (
        "Security Auditor",
        &["VulnScanSkill", "CryptoAuditSkill", "ComplianceCheck"],
    ),
    (
        "DevOps Engineer",
        &[
            "ContainerSkill",
            "MonitoringSkill",
            "DeploymentSkill",
            "SystemMetricsSkill",
        ],
    ),
    (
        "ui_ux Designer",
        &["MockupSkill", "AccessibilityAudit", "ComponentLibrary"],
    ),
    (
        "QA Engineer",
        &[
            "FuzzTestSkill",
            "RegressionSkill",
            "CoverageAnalysis",
            "OutputValidatorSkill",
        ],
    ),
    (
        "Swarm Manager",
        &[
            "LoadBalancerSkill",
            "AgentProfiler",
            "MissionPlanner",
            "SystemMetricsSkill",
        ],
    ),
    (
        "Technical Writer",
        &["APIDocGenerator", "TutorialBuilder", "ChangelogSkill"],
    ),
    (
        "Integration Specialist",
        &["ProtocolBridgeSkill", "DataMapperSkill", "WebhookSkill"],
    ),
    (
        "Data Analyst",
        &[
            "VisualizationSkill",
            "StatModelSkill",
            "AnomalyDetector",
            "SystemMetricsSkill",
        ],
    ),
];

pub const ROLE_COUNT: usize = ROLE_CAPABILITIES.len();

/// Expected skill list for a role (ROLE_CAPABILITIES.get(role, [])).
pub fn role_capabilities(role: &str) -> &'static [&'static str] {
    ROLE_CAPABILITIES
        .iter()
        .find(|(r, _)| *r == role)
        .map(|(_, skills)| *skills)
        .unwrap_or(&[])
}

/// assess_agent_needs(role, current_skills):
/// `[skill for skill in expected if skill not in current_skills]`.
pub fn assess_agent_needs(role: &str, current_skills: &[&str]) -> Vec<&'static str> {
    role_capabilities(role)
        .iter()
        .filter(|skill| !current_skills.contains(skill))
        .copied()
        .collect()
}

/// One aggregated gap (python dict in the sorted list).
#[derive(Debug, Clone)]
pub struct GapSummary {
    pub tool_name: String,
    /// Roles needing this tool, in first-seen order (python needed_by list).
    pub needed_by: Vec<String>,
    /// Number of roles needing the tool (python score).
    pub score: u32,
}

/// aggregate_gaps(all_gaps): count per gap across roles, emit only gaps
/// actually present, stable-sorted by score descending. Role iteration
/// follows the caller-provided (role, gaps) order — python dict order.
pub fn aggregate_gaps(all_gaps: &[(String, Vec<String>)]) -> Vec<GapSummary> {
    let mut counts: Vec<GapSummary> = vec![];
    for (role, gaps) in all_gaps {
        for gap in gaps {
            if let Some(existing) = counts.iter_mut().find(|g| g.tool_name == *gap) {
                existing.needed_by.push(role.clone());
                existing.score += 1;
            } else {
                counts.push(GapSummary {
                    tool_name: gap.clone(),
                    needed_by: vec![role.clone()],
                    score: 1,
                });
            }
        }
    }
    // python: sorted(counts.values(), key=lambda x: x["score"], reverse=True)
    // — stable, so equal scores preserve the first-seen order above.
    let mut order: Vec<usize> = (0..counts.len()).collect();
    order.sort_by(|&a, &b| counts[b].score.cmp(&counts[a].score));
    order.into_iter().map(|i| counts[i].clone()).collect()
}

/// ToolForge — deterministic subset: build/assessment history registers.
/// `__init__(learning_engine)` keeps the engine reference MANUAL.
#[derive(Debug, Clone, Default)]
pub struct ToolForge {
    pub build_log: Vec<String>,
    pub assessment_history: Vec<String>,
}

impl ToolForge {
    pub fn new() -> Self {
        ToolForge {
            build_log: vec![],
            assessment_history: vec![],
        }
    }

    /// Assess one agent and record the assessment; returns gaps.
    pub fn assess_agent_needs(&mut self, role: &str, current_skills: &[&str]) -> Vec<String> {
        let gaps = assess_agent_needs(role, current_skills)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        self.assessment_history
            .push(role.to_string());
        gaps
    }

    /// run_forge_pipeline — MANUAL skeleton. The python body orchestrates
    /// five agents (Architect/Lead Developer/Security Auditor/QA Engineer/
    /// Technical Writer) plus learning-engine registration; the only
    /// deterministic gate ported is the Lead-Developer failure early-return.
    /// Returns (completed, has_lead_developer).
    pub fn run_forge_pipeline(&mut self, tool_name: &str, has_lead_developer: bool) -> (bool, bool) {
        self.build_log.push(tool_name.to_string());
        if !has_lead_developer {
            // python: stage Lead Developer failed → build_entry["status"]="failed", return.
            return (false, false);
        }
        // MANUAL: remaining stages require live agents.
        (true, true)
    }

    pub fn build_log_len(&self) -> usize {
        self.build_log.len()
    }
}
