//! TRM Swarm OS — Instella Sparse Microkernel Agent Teams (no_std port).
//! Organizes 64 Microkernel Experts into 8 Specialized Agent Teams with 1-of-8 sparse activation.
//! 1:1 deterministic core from swarm_v2/core/instella_sparse_agent_teams.py:
//!   - 8 Teams (ZeroTrust, BioGenomics, FinTech, Quantum, AST Proofs, System Genomics, PBFT Mesh, Cosmos 3)
//!   - Active Lead + 8-expert team activation (1-of-8 sparse gating)
//!   - 7 teams (56 experts) kept in Dark Standby for 8.0x compute reduction

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct SparseTeamDef {
    pub team_id: &'static str,
    pub lead_agent: &'static str,
    pub domain: &'static str,
    pub expert_count: usize,
}

pub const EIGHT_SPARSE_TEAMS: [SparseTeamDef; 8] = [
    SparseTeamDef {
        team_id: "Team_1_ZeroTrust_Security",
        lead_agent: "Shield",
        domain: "Zero-Trust Security & DDR Antibodies",
        expert_count: 8,
    },
    SparseTeamDef {
        team_id: "Team_2_BioGenomics_Therapeutics",
        lead_agent: "Seeker",
        domain: "Bio-Genomics & PHOENIX Protocol",
        expert_count: 8,
    },
    SparseTeamDef {
        team_id: "Team_3_FinTech_HighFrequency_Risk",
        lead_agent: "Logic",
        domain: "FinTech Risk & Order Book Shunting",
        expert_count: 8,
    },
    SparseTeamDef {
        team_id: "Team_4_Quantum_Physics_Math",
        lead_agent: "LogicProver",
        domain: "128-Qubit MPS Tensor Networks & VQE",
        expert_count: 8,
    },
    SparseTeamDef {
        team_id: "Team_5_AST_Safety_Proofs",
        lead_agent: "Verify",
        domain: "Formal AST Invariants & C-Types Fuzzing",
        expert_count: 8,
    },
    SparseTeamDef {
        team_id: "Team_6_System_Genomics_4D",
        lead_agent: "Archi",
        domain: "4D Persona Alignment & Star Matrix",
        expert_count: 8,
    },
    SparseTeamDef {
        team_id: "Team_7_Autonomic_PBFT_Mesh",
        lead_agent: "Aether",
        domain: "PBFT 3-Phase Consensus & Mesh Self-Healing",
        expert_count: 8,
    },
    SparseTeamDef {
        team_id: "Team_8_Physical_AI_Cosmos3",
        lead_agent: "Vision",
        domain: "NVIDIA Cosmos 3 World Model & 3D Simulation",
        expert_count: 8,
    },
];

pub struct InstellaSparseAgentTeams {
    pub total_routed_tasks: u64,
}

impl InstellaSparseAgentTeams {
    pub fn new() -> Self {
        InstellaSparseAgentTeams { total_routed_tasks: 0 }
    }

    /// Route task domain to a single active team while keeping 7 teams in dark standby.
    pub fn route_task(&mut self, task_domain: &str) -> (&'static SparseTeamDef, usize, u64) {
        let t0 = crate::time::now_ns();
        self.total_routed_tasks += 1;
        let d_lower = task_domain.to_ascii_lowercase();

        let active_team = if d_lower.contains("bio") || d_lower.contains("dock") || d_lower.contains("drug") {
            &EIGHT_SPARSE_TEAMS[1]
        } else if d_lower.contains("math") || d_lower.contains("quantum") || d_lower.contains("vqe") {
            &EIGHT_SPARSE_TEAMS[3]
        } else if d_lower.contains("proof") || d_lower.contains("test") || d_lower.contains("ast") {
            &EIGHT_SPARSE_TEAMS[4]
        } else if d_lower.contains("archi") || d_lower.contains("persona") || d_lower.contains("matrix") {
            &EIGHT_SPARSE_TEAMS[5]
        } else if d_lower.contains("mesh") || d_lower.contains("pbft") || d_lower.contains("consensus") {
            &EIGHT_SPARSE_TEAMS[6]
        } else if d_lower.contains("cosmos") || d_lower.contains("3d") || d_lower.contains("vision") {
            &EIGHT_SPARSE_TEAMS[7]
        } else if d_lower.contains("risk") || d_lower.contains("fintech") {
            &EIGHT_SPARSE_TEAMS[2]
        } else {
            &EIGHT_SPARSE_TEAMS[0] // Default Zero-Trust Security
        };

        let dark_standby_experts = 56;
        let latency_ns = crate::time::now_ns().saturating_sub(t0);
        (active_team, dark_standby_experts, latency_ns)
    }
}

static SINGLETON_SPARSE_TEAMS: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

pub fn get_sparse_agent_teams() -> u64 {
    let v = SINGLETON_SPARSE_TEAMS.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SPARSE_TEAMS.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
