//! TRM Swarm OS - Instella MoE single-expert router (no_std Rust port).
//! 1:1 with swarm_v2/core/instella_single_expert_router.py:
//! 8 teams x 8 experts = 64 experts, only 1 team activates per task via
//! domain substring matching. Confidence = min(score/5, 1.0).

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

pub struct ExpertRoute {
    pub team: &'static str,
    pub lead_agent: &'static str,
    pub experts_active: u8,
    pub latency_us: u64,
    pub confidence: f32,
}

struct Team {
    name: &'static str,
    lead: &'static str,
    domains: &'static [&'static str],
}

/// 8 teams, 64 experts total (8 per team) - identical to python TEAMS dict.
const TEAMS: [Team; 8] = [
    Team { name: "Security", lead: "Shield", domains: &["security", "audit", "vulnerability", "ddr", "antibody", "threat", "cve", "attack", "defense", "firewall", "vault"] },
    Team { name: "Research", lead: "Seeker", domains: &["research", "paper", "arxiv", "pubmed", "discover", "find", "search", "protein", "alphafold", "literature", "knowledge"] },
    Team { name: "Engineering", lead: "Devo", domains: &["code", "compile", "python", "api", "build", "deploy", "module", "function", "class", "import", "train", "test"] },
    Team { name: "Architecture", lead: "Archi", domains: &["architecture", "design", "blueprint", "microkernel", "mamba", "ssm", "synthesize", "topology", "gan", "structure", "system"] },
    Team { name: "Logic", lead: "Logic", domains: &["prove", "theorem", "logic", "k3", "kleene", "coq", "lean4", "proof", "boolean", "axiom", "ternary", "sat", "smt"] },
    Team { name: "Integration", lead: "Flow", domains: &["integrate", "connect", "rest", "graphql", "grpc", "websocket", "mesh", "gateway", "endpoint", "pipeline", "api"] },
    Team { name: "Operations", lead: "Orchestra", domains: &["orchestrate", "coordinate", "schedule", "workflow", "dispatch", "twin", "deploy", "manage", "task", "pipeline", "health", "status"] },
    Team { name: "Intelligence", lead: "Jarvis", domains: &["memory", "turbovec", "knowledge", "cognitive", "skill", "graph", "semantic", "retrieve", "context", "recall", "chat"] },
];

pub struct InstellaSingleExpertRouter;

impl InstellaSingleExpertRouter {
    pub fn new() -> Self {
        InstellaSingleExpertRouter
    }

    /// Route a task to the single best expert team. Only 1 team activates.
    pub fn route(&self, task: &str) -> ExpertRoute {
        let t0 = crate::time::now_ns();
        let task_lower = task.to_lowercase();

        let mut best_team: Option<&Team> = None;
        let mut best_score = 0usize;

        for team in TEAMS.iter() {
            let score = team
                .domains
                .iter()
                .filter(|d| task_lower.contains(**d))
                .count();
            if score > best_score {
                best_score = score;
                best_team = Some(team);
            }
        }

        let (team, score) = match best_team {
            Some(t) if best_score > 0 => (t, best_score),
            _ => {
                // Default to Operations for coordination
                (&TEAMS[6], 1usize)
            }
        };

        let elapsed_us = (crate::time::now_ns().wrapping_sub(t0)) / 1000;

        ExpertRoute {
            team: team.name,
            lead_agent: team.lead,
            experts_active: if best_score > 0 { 1 } else { 0 },
            latency_us: elapsed_us,
            confidence: (best_score as f32 / 5.0).min(1.0),
        }
    }

    /// Convenience: expert names for a team (for the `moe` command display).
    pub fn team_experts(&self, team: &str) -> Vec<&'static str> {
        for t in TEAMS.iter() {
            if t.name.eq_ignore_ascii_case(team) {
                // 8 expert slots per team
                let names: Vec<&'static str> = (0..8).map(|i| match (t.name, i) {
                    ("Security", 0) => "Shield",
                    ("Security", _) => "Sentinel",
                    ("Research", 0) => "Seeker",
                    ("Research", _) => "Researcher",
                    ("Engineering", 0) => "Devo",
                    ("Engineering", _) => "Engineer",
                    ("Architecture", 0) => "Archi",
                    ("Architecture", _) => "Architect",
                    ("Logic", 0) => "Logic",
                    ("Logic", _) => "Prover",
                    ("Integration", 0) => "Flow",
                    ("Integration", _) => "Integrator",
                    ("Operations", 0) => "Orchestra",
                    ("Operations", _) => "Operator",
                    ("Intelligence", 0) => "Jarvis",
                    _ => "Cognitor",
                }).collect();
                return names;
            }
        }
        Vec::new()
    }

    /// Total expert count (64 = 8 teams x 8).
    pub fn total_experts() -> usize {
        64
    }
}

/// Keep String import used (task_lower). Helper for display.
pub fn _assert_used(_s: &String) {}
