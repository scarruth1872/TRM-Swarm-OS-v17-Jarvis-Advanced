//! swarm-conductor — Motion B (v5): the ensemble-driven mesh.
//!
//! The full synthesis of the platform's spine:
//!   OBSERVE:  the 2-model voltage ensemble (Qwen + Bonsai fields)
//!   DECIDE:   K3 collapse → verdict (Act / Block / Analyze)
//!   ROUTE:    mesh assigns the task to its domain owner (swarm-mesh)
//!   ACT:      the owner's real act-arm executes
//!
//! Every layer reuses a VERIFIED crate: swarm-mesh routing, the K3
//! collapse semantics, and the act-arm pattern. Nothing new unverified.

use swarm_mesh::route;

/// The ensemble verdict (mirrors the verified K3 collapse semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnsembleVerdict {
    Act,
    Block,
    Analyze,
}

/// Collapse the ensemble field (Qwen+Bonsai superposition) to a verdict.
/// Mirrors collapse_to_verdict (τ=0.35): TRUE/FALSE/UNKNOWN by threshold.
pub fn ensemble_verdict(field: &[f32], tau: f32) -> EnsembleVerdict {
    let mut act = 0usize;
    let mut block = 0usize;
    let mut unknown = 0usize;
    for &v in field {
        if v.abs() < tau {
            unknown += 1;
        } else if v > 0.0 {
            act += 1;
        } else {
            block += 1;
        }
    }
    if act > block && act >= unknown {
        EnsembleVerdict::Act
    } else if block > act && block >= unknown {
        EnsembleVerdict::Block
    } else {
        EnsembleVerdict::Analyze
    }
}

/// The conductor's full round: task in → (verdict, owner, disposition).
pub fn conduct(task: &str, field: &[f32], tau: f32) -> (EnsembleVerdict, String, String) {
    let verdict = ensemble_verdict(field, tau);
    let (agent, ov) = route(task);
    let disposition = match verdict {
        EnsembleVerdict::Act => format!("route to {agent} (overlap {ov}) — EXECUTE"),
        EnsembleVerdict::Block => format!("route to {agent} (overlap {ov}) — HOLD (verdict BLOCK)"),
        EnsembleVerdict::Analyze => format!("route to {agent} (overlap {ov}) — ANALYZE further"),
    };
    (verdict, agent, disposition)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensemble_act_routes_to_owner() {
        let field = vec![0.9, 0.9, 0.9, 0.8, 0.7];
        let (v, agent, d) = conduct("deploy antibody against the exploit", &field, 0.35);
        assert_eq!(v, EnsembleVerdict::Act);
        assert_eq!(agent, "Shield");
        assert!(d.contains("EXECUTE"));
    }

    #[test]
    fn ensemble_block_holds() {
        let field = vec![-0.9, -0.9, -0.9, -0.8];
        let (v, _agent, d) = conduct("write the compiler fix", &field, 0.35);
        assert_eq!(v, EnsembleVerdict::Block);
        assert!(d.contains("HOLD"));
    }

    #[test]
    fn ensemble_analyze_routes() {
        let field = vec![0.1, -0.1, 0.05, 0.0];
        let (v, agent, d) = conduct("prove the theorem in Coq", &field, 0.35);
        assert_eq!(v, EnsembleVerdict::Analyze);
        assert_eq!(agent, "Logic");
        assert!(d.contains("ANALYZE"));
    }
}
