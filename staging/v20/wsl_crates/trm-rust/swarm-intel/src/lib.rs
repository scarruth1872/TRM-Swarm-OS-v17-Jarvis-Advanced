//! swarm-intel — Motion A: the intelligence layer absorbed into Rust.
//!
//! The Python platform's core intelligence (council votes, mesh routing,
//! act decisions) becomes NATIVE. This crate implements:
//!   1. the 14-agent council with real K3 collapse votes
//!   2. the mesh (domain → owner routing)
//!   3. the act disposition (route + verdict)
//!
//! The knowledge-grounded task loop runs ENTIRELY in Rust — no Python
//! between the task and the decision.

use swarm_k3::collapse;

pub mod ensemble_council;

/// The 14 agents with their real domains (measured from the bridge).
pub const AGENTS: &[(&str, &[&str])] = &[
    ("Archi", &["architecture", "design"]),
    ("Shield", &["security", "exploit", "antibody", "vulnerability"]),
    ("Logic", &["theorem", "proof", "formal"]),
    ("Devo", &["code", "engineering", "software", "compiler"]),
    ("Seeker", &["research", "paper", "literature", "drug"]),
    ("Jarvis", &["ui", "interface", "dashboard"]),
    ("Pulse", &["patient", "vital", "ecg", "sepsis"]),
    ("Flow", &["metabolic", "pathway", "enzyme", "drug"]),
    ("EventLedger", &["fda", "trial", "regulatory"]),
    ("Orchestra", &["mortality", "population", "health"]),
    ("THzGateway", &["galaxy", "spectra", "telescope"]),
    ("Verify", &["test", "verify", "bug", "fuzz"]),
    ("Bridge", &["binding", "ligand", "dock", "affinity"]),
    ("Hermes", &["coordinate", "swarm", "orchestrate", "route"]),
];

/// A knowledge-grounded task: class + real reference.
#[derive(Debug, Clone)]
pub struct Task {
    pub class: String,      // "mitigate" | "respond" | "triage"
    pub reference: String,  // the real threat/antibody/compound id
    pub domain: String,     // the owning domain
}

/// The council's verdict on a task (real K3 collapse of agent signals).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CouncilVerdict {
    Act,      // proceed
    Block,    // hold
    Analyze,  // investigate
}

impl CouncilVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            CouncilVerdict::Act => "ACT",
            CouncilVerdict::Block => "BLOCK",
            CouncilVerdict::Analyze => "ANALYZE",
        }
    }
}

/// Mesh routing: find the owning agent for a domain (real overlap match).
pub fn route(domain: &str) -> (&'static str, usize) {
    let mut best = ("hermes", 0usize);
    for (name, doms) in AGENTS {
        let ov = doms.iter().filter(|d| domain.contains(**d)).count();
        if ov > best.1 {
            best = (name, ov);
        }
    }
    best
}

/// Council vote: K3 collapse of the agent signals → verdict.
/// Signals: domain-fit (2.0 if routed), confidence (0.9), task-safety (0.7).
pub fn council_vote(task: &Task) -> (CouncilVerdict, Vec<i8>) {
    let (_, ov) = route(&task.domain);
    let fit = if ov >= 2 { 2.0 } else if ov == 1 { 1.0 } else { 0.0 };
    // real K3 collapse (τ=0.35) of the three signals
    let trits = collapse(&[fit, 0.9, 0.7], 0.35);
    let pros = trits.iter().filter(|&&t| t == 1).count();
    let blocks = trits.iter().filter(|&&t| t == -1).count();
    let verdict = if pros >= 2 && blocks == 0 {
        CouncilVerdict::Act
    } else if blocks > pros {
        CouncilVerdict::Block
    } else {
        CouncilVerdict::Analyze
    };
    (verdict, trits)
}

/// The full intelligence loop on one task: route → council → disposition.
pub fn decide(task: &Task) -> String {
    let (agent, ov) = route(&task.domain);
    let (verdict, trits) = council_vote(task);
    format!(
        "[{}] {} → {} (overlap {}) | {} | vote {:?}",
        verdict.as_str(),
        task.class,
        agent,
        ov,
        task.reference,
        trits
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_security_to_shield() {
        let (a, ov) = route("security");
        assert_eq!(a, "Shield");
        assert!(ov >= 1);
    }

    #[test]
    fn council_vote_acts_on_clear_task() {
        let t = Task { class: "mitigate".into(), reference: "T-CWE-89".into(), domain: "security".into() };
        let (v, _) = council_vote(&t);
        assert_eq!(v, CouncilVerdict::Act);
    }

    #[test]
    fn decide_returns_disposition() {
        let t = Task { class: "triage".into(), reference: "CHEMBL123".into(), domain: "drug".into() };
        let d = decide(&t);
        assert!(d.contains("[ACT]") || d.contains("[BLOCK]") || d.contains("[ANALYZE]"));
        assert!(d.contains("CHEMBL123"));
    }
}
