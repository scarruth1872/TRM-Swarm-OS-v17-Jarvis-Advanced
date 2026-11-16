//! swarm-mesh — Motion D: the agent mesh (Hermes orchestrates 14 workers).
//!
//! A real task-routing mesh: tasks are classified by domain and routed to
//! the owning agent; results flow back through the mesh. The 14 agents are
//! the registered domains (measured from agent_intelligence_bridge).

/// The 14 agents with their domains (measured from the real bridge).
pub const AGENTS: &[(&str, &[&str])] = &[
    ("Archi", &["architecture", "design", "synthetic biology"]),
    ("Shield", &["security", "vulnerability", "exploit", "antibody"]),
    ("Logic", &["theorem", "proof", "coq", "lean"]),
    ("Devo", &["code", "engineering", "software", "compiler"]),
    ("Seeker", &["research", "paper", "literature", "drug"]),
    ("Jarvis", &["ui", "interface", "dashboard", "visualization"]),
    ("Pulse", &["patient", "vital", "ecg", "sepsis"]),
    ("Flow", &["metabolic", "pathway", "enzyme", "drug"]),
    ("EventLedger", &["trial", "fda", "regulatory", "adverse"]),
    ("Orchestra", &["mortality", "population", "epidemic", "health"]),
    ("THzGateway", &["galaxy", "star", "spectra", "telescope"]),
    ("Verify", &["test", "verify", "bug", "fuzz"]),
    ("Bridge", &["binding", "ligand", "dock", "affinity"]),
    ("Hermes", &["coordinate", "swarm", "orchestrate", "route"]),
];

/// Route a task to the agent with the most domain overlap.
/// Returns (agent, overlap_count). Hermes coordinates when no owner.
pub fn route(task: &str) -> (String, usize) {
    let lower = task.to_lowercase();
    let mut best = ("hermes".to_string(), 0usize);
    for (name, domains) in AGENTS {
        let ov = domains.iter().filter(|d| lower.contains(**d)).count();
        if ov > best.1 {
            best = (name.to_string(), ov);
        }
    }
    best
}

/// The mesh round: route N tasks, count each agent's load.
pub fn mesh_round(tasks: &[&str]) -> Vec<(String, usize, String)> {
    let mut results = Vec::new();
    for t in tasks {
        let (agent, ov) = route(t);
        let verdict = if ov >= 2 {
            "direct"
        } else if ov == 1 {
            "coordinate"
        } else {
            "hermes-routed"
        };
        results.push((agent, ov, verdict.to_string()));
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_security_to_shield() {
        let (a, ov) = route("deploy antibody against the exploit");
        assert_eq!(a, "Shield");
        assert!(ov >= 2);
    }

    #[test]
    fn route_code_to_devo() {
        let (a, ov) = route("write the compiler fix");
        assert_eq!(a, "Devo");
        assert!(ov >= 1);
    }

    #[test]
    fn route_unknown_to_hermes() {
        let (a, ov) = route("zzz unknown nonsense task");
        assert_eq!(a, "hermes");
        assert_eq!(ov, 0);
    }
}
