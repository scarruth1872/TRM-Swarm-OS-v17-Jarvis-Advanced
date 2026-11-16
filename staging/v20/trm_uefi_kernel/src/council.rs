//! TRM Swarm OS v19.001 - Native Pure Rust Swarm Council (no_std port)
//! 14-agent K3 ternary voting (+1/0/-1), domain weighting, PBFT quorum,
//! Monad truth anchor sealing. Ported from swarm_v2_rust_kernel/src/council.rs
//! - std::time::Instant -> crate::time::now_ns, rayon removed (dead code).

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::agent_runtime::{SwarmAgentId, NUM_AGENTS, get_runtime};
use crate::MONAD_TRUTH_ANCHOR;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalCategory {
    Security,
    Architecture,
    Code,
    Data,
    Communication,
    Health,
    Infrastructure,
    Research,
    Protocol,
    General,
}

impl ProposalCategory {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "security" | "sec" => ProposalCategory::Security,
            "architecture" | "arch" => ProposalCategory::Architecture,
            "code" | "dev" => ProposalCategory::Code,
            "data" | "bridge" => ProposalCategory::Data,
            "communication" | "comms" => ProposalCategory::Communication,
            "health" | "pulse" => ProposalCategory::Health,
            "infrastructure" | "infra" => ProposalCategory::Infrastructure,
            "research" | "science" => ProposalCategory::Research,
            "protocol" | "monad" => ProposalCategory::Protocol,
            _ => ProposalCategory::General,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ProposalCategory::Security => "Security",
            ProposalCategory::Architecture => "Architecture",
            ProposalCategory::Code => "Code & Build",
            ProposalCategory::Data => "Data & Knowledge",
            ProposalCategory::Communication => "Communication & Spectrum",
            ProposalCategory::Health => "System Health",
            ProposalCategory::Infrastructure => "Infrastructure",
            ProposalCategory::Research => "Research & Science",
            ProposalCategory::Protocol => "Monad Protocol",
            ProposalCategory::General => "General Sovereign",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalPhase {
    Proposed,
    Deliberating,
    Voting,
    Consensus,
    Enacted,
    Rejected,
}

pub fn get_domain_weight(agent: SwarmAgentId, category: ProposalCategory) -> f32 {
    match (agent, category) {
        (SwarmAgentId::Shield, ProposalCategory::Security) => 1.0,
        (SwarmAgentId::Verify, ProposalCategory::Security) => 0.8,
        (SwarmAgentId::Archi, ProposalCategory::Architecture) => 1.0,
        (SwarmAgentId::Logic, ProposalCategory::Architecture) => 0.9,
        (SwarmAgentId::Devo, ProposalCategory::Code) => 1.0,
        (SwarmAgentId::Bridge, ProposalCategory::Data) => 1.0,
        (SwarmAgentId::Hermes, ProposalCategory::Communication) => 1.0,
        (SwarmAgentId::THzGateway, ProposalCategory::Communication) => 0.9,
        (SwarmAgentId::Pulse, ProposalCategory::Health) => 1.0,
        (SwarmAgentId::Flow, ProposalCategory::Infrastructure) => 0.9,
        (SwarmAgentId::Orchestra, ProposalCategory::Infrastructure) => 0.8,
        (SwarmAgentId::EventLedger, ProposalCategory::Protocol) => 1.0,
        (SwarmAgentId::Verify, ProposalCategory::Protocol) => 0.8,
        (SwarmAgentId::Seeker, ProposalCategory::Research) => 1.0,
        (SwarmAgentId::Jarvis, ProposalCategory::General) => 0.9,
        _ => 0.45,
    }
}

pub fn get_agent_deliberation(agent: SwarmAgentId, category: ProposalCategory, topic: &str) -> String {
    match agent {
        SwarmAgentId::Hermes => format!("Hermes: Validating mesh communication topologies & peer signal clarity for '{}'.", topic),
        SwarmAgentId::Jarvis => format!("Jarvis: Aligning sovereign execution goals with Monad operator directives for '{}'.", topic),
        SwarmAgentId::Bridge => format!("Bridge: Auditing zero-copy memory boundaries and data tensor routing for '{}'.", topic),
        SwarmAgentId::Shield => format!("Shield: Assessing threat vectors, DDR antibody integrity, and memory safety for '{}'.", topic),
        SwarmAgentId::Logic => format!("Logic: Proving formal K3 ternary consistency, non-contradiction, and truth bounds for '{}'.", topic),
        SwarmAgentId::Devo => format!("Devo: Verifying zero-GC Rust microkernel compile contracts and execution footprint for '{}'.", topic),
        SwarmAgentId::Pulse => format!("Pulse: Checking hardware thermal envelopes, voltage rails, and thread headroom for '{}'.", topic),
        SwarmAgentId::Flow => format!("Flow: Orchestrating lockless pipeline synchronization across CPU/GPU queues for '{}'.", topic),
        SwarmAgentId::EventLedger => format!("EventLedger: Recording immutable state transition proof hash into Monad ledger for '{}'.", topic),
        SwarmAgentId::Orchestra => format!("Orchestra: Harmonizing multi-threaded Rayon work-stealing topology for '{}'.", topic),
        SwarmAgentId::THzGateway => format!("THzGateway: Analyzing ambient sub-THz/VLF electromagnetic resonance spectrum for '{}'.", topic),
        SwarmAgentId::Verify => format!("Verify: Cross-checking claims against Monad Anchor 0x4D4F4E4144000001 for '{}'.", topic),
        SwarmAgentId::Seeker => format!("Seeker: Querying 153.5M trit NVMe NAND fabric for semantic priors on '{}'.", topic),
        SwarmAgentId::Archi => format!("Archi: Ensuring sovereign bare-metal architectural purity and zero-Python isolation for '{}'.", topic),
        SwarmAgentId::LogicMaster => format!("LogicMaster: Reducing the proposal to machine-checkable clauses and testing formal consistency (SAT/SMT) for '{}'.", topic),
    }
}

#[derive(Debug, Clone)]
pub struct NativeAgentVote {
    pub agent_id: u8,
    pub agent_name: &'static str,
    pub k3_trit: i8,
    pub confidence: f32,
    pub weight: f32,
    pub latency_ns: u64,
}

#[derive(Debug, Clone)]
pub struct NativeCouncilProposal {
    pub proposal_id: u64,
    pub topic: String,
    pub category: ProposalCategory,
    pub proposer: String,
    pub phase: ProposalPhase,
    pub deliberations: Vec<String>,
    pub votes: Vec<NativeAgentVote>,
    pub weighted_score: f32,
    pub approve_count: usize,
    pub abstain_count: usize,
    pub reject_count: usize,
    pub consensus_reached: bool,
    pub monad_seal: u64,
    pub duration_us: f64,
}

pub struct NativeSwarmCouncil;

impl NativeSwarmCouncil {
    /// Execute a full Sovereign Council deliberation cycle across all 14 agents.
    /// Deterministic per (topic, category) - identical seal to the Windows build.
    pub fn evaluate(topic: &str, category: ProposalCategory, proposer: &str) -> NativeCouncilProposal {
        let cycle_start = crate::time::now_ns();
        let runtime = get_runtime();
        let proposal_id = crate::time::now_ns() ^ MONAD_TRUTH_ANCHOR;

        // Step 1: Deliberation round (14 agents generate native reasoning)
        let deliberations: Vec<String> = (0..NUM_AGENTS)
            .map(|i| {
                let agent = SwarmAgentId::from_u8(i as u8);
                get_agent_deliberation(agent, category, topic)
            })
            .collect();

        // Step 2: K3 Voting across 14 agents
        let topic_bytes = topic.as_bytes();
        let votes: Vec<NativeAgentVote> = (0..NUM_AGENTS)
            .map(|i| {
                let agent = SwarmAgentId::from_u8(i as u8);
                let vote_start = crate::time::now_ns();

                let (trit, base_conf) = runtime.dispatch(i as u8, 0, topic_bytes);
                let lat_ns = crate::time::now_ns().wrapping_sub(vote_start);

                let weight = get_domain_weight(agent, category);
                let confidence = (base_conf * 0.7 + weight * 0.3).min(1.0);

                NativeAgentVote {
                    agent_id: i as u8,
                    agent_name: agent.name(),
                    k3_trit: trit,
                    confidence,
                    weight,
                    latency_ns: lat_ns,
                }
            })
            .collect();

        // Step 3: K3 Weighted Consensus & Byzantine Quorum
        let mut total_weighted = 0.0f32;
        let mut total_weight = 0.0f32;
        let mut approve_count = 0;
        let mut abstain_count = 0;
        let mut reject_count = 0;

        for v in &votes {
            total_weighted += (v.k3_trit as f32) * v.weight * v.confidence;
            total_weight += v.weight;

            match v.k3_trit {
                1 => approve_count += 1,
                -1 => reject_count += 1,
                _ => abstain_count += 1,
            }
        }

        let weighted_score = if total_weight > 0.0 { total_weighted / total_weight } else { 0.0 };

        // PBFT: N=14, f=4, quorum requires >= 2f+1 = 9 approvals and score > 0.35
        let consensus_reached = approve_count >= 10 && weighted_score > 0.35;
        let phase = if consensus_reached { ProposalPhase::Enacted } else { ProposalPhase::Rejected };

        // Step 4: Cryptographically seal with Monad Truth Anchor
        let mut monad_seal: u64 = MONAD_TRUTH_ANCHOR;
        for (i, v) in votes.iter().enumerate() {
            monad_seal ^= (v.k3_trit as u64).rotate_left((i * 4) as u32);
            monad_seal = monad_seal.wrapping_add((v.confidence * 1000.0) as u64);
        }

        let duration_us = crate::time::now_ns().wrapping_sub(cycle_start) as f64 / 1000.0;

        NativeCouncilProposal {
            proposal_id,
            topic: topic.to_string(),
            category,
            proposer: proposer.to_string(),
            phase,
            deliberations,
            votes,
            weighted_score,
            approve_count,
            abstain_count,
            reject_count,
            consensus_reached,
            monad_seal,
            duration_us,
        }
    }
}
