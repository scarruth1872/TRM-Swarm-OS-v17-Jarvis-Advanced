//! Canonical platform facts, emitted FROM CODE so they cannot drift from what is running.
//!
//! WHY THIS EXISTS (measured 2026-11-24, docs/TRM_MoE_Inventory_Probe_2026-11-24.md): the council's
//! retrieval index is 558 chunks of narrative docs, and every top-1 for a platform question was
//! semantically unrelated (cosine 0.60-0.65). Asked directly, both resident MoE models answered
//! platform questions **wrong and confidently** - gpt-oss-20b said the council listens on "TCP port
//! 443 (HTTPS)" and invented axiom A2 as a set-theoretic relation. 0/4 platform questions were answered
//! by the weights, and 0/4 were answered by the index.
//!
//! The facts those questions asked about were never *in* the index: they live in constants, in
//! `sovereign_axioms::describe()`, and in the report generated from the live module scan. This module
//! publishes them on `GET /platform/facts` so the index builder can embed them, and so a reader can check
//! them without trusting a model.
//!
//! Honesty rules kept here:
//!   * every route listed must appear verbatim in `main.rs` - enforced by a test, so a removed route
//!     fails the build instead of leaving a stale fact in the index;
//!   * ports come from the same constants the servers use (`local_inference::{INFER_PORT, EMBED_PORT}`),
//!     never from a literal typed twice;
//!   * nothing here claims a measurement it did not take. Model *identity* is reported as the port it is
//!     served on plus a note, because the weights are chosen by the launcher, not by this process.

use serde_json::{json, Value};

/// The port this server binds. Must match `main.rs`; asserted by `council_port_matches_main`.
pub const COUNCIL_PORT: u16 = 8022;

/// Canonical routes and what they answer. Every string is asserted present in `main.rs`.
///
/// This is deliberately not "all 159 dispatch literals": those include per-resource variants that would
/// bury a reader. It is the set a caller needs to find the truth about the platform.
pub const ROUTES: &[(&str, &str)] = &[
    ("GET  /health", "/health"),
    ("GET  /council/status", "/council/status"),
    ("GET  /council/axioms", "/council/axioms"),
    ("POST /council/propose", "/council/propose"),
    ("GET  /council/proposals", "/council/proposals"),
    ("POST /k3/classify", "/k3/classify"),
    ("GET  /k3/ternary/bench", "/k3/ternary/bench"),
    ("GET  /platform/facts", "/platform/facts"),
    ("GET  /platform/cadence", "/platform/cadence"),
    ("GET  /platform/auth_policy", "/platform/auth_policy"),
    ("GET  /platform/skills_registry", "/platform/skills_registry"),
    ("POST /platform/hands_inspect", "/platform/hands_inspect"),
    ("GET  /learning/skills", "/learning/skills"),
    ("GET  /federation/ofp", "/federation/ofp"),
    ("GET  /ddr/stats", "/ddr/stats"),
    ("GET  /system/status", "/system/status"),
    ("GET  /journal/status", "/journal/status"),
    ("GET  /kanban/board", "/kanban/board"),
    ("GET  /inbox", "/inbox"),
    ("GET  /voltage/telemetry", "/voltage/telemetry"),
    ("GET  /turbovec/bench", "/turbovec/bench"),
    ("GET  /research/tasks", "/research/tasks"),
    ("GET  /testing/runs", "/testing/runs"),
    ("GET  /verification/queue", "/verification/queue"),
    ("GET  /v1/models", "/v1/models"),
    ("POST /v1/embeddings", "/v1/embeddings"),
];

/// Facts about the code as compiled, plus the constants the services actually use.
pub fn facts() -> Value {
    let modules = &trm_uefi_kernel::module_registry_generated::TOTAL;
    let _ = modules;
    json!({
        "kind": "canonical-platform-facts",
        "why": "published from code so retrieval can index the truth instead of a model's recollection",
        "service": {
            "binary": "host_council_server",
            "version": env!("CARGO_PKG_VERSION"),
            "council_port": COUNCIL_PORT,
            "council_url": format!("http://127.0.0.1:{}", COUNCIL_PORT),
        },
        "ports": {
            "council": COUNCIL_PORT,
            "vote_model": crate::local_inference::INFER_PORT,
            "embedder": crate::local_inference::EMBED_PORT,
        },
        "model_roles": {
            "vote_model": "served on the vote_model port; identity lives in the launcher/task config, not here",
            "note": "never state a model's name from this process - read /v1/models on its port",
        },
        "axioms": crate::sovereign_axioms::describe(),
        "kernel_modules": {
            "total": trm_uefi_kernel::module_registry_generated::TOTAL,
            "real": trm_uefi_kernel::module_registry_generated::REAL,
            "ported": trm_uefi_kernel::module_registry_generated::PORTED,
            "source": "module_registry_generated.rs, regenerated from the live module scan",
        },
        "routes": ROUTES.iter().map(|(sig, path)| json!({"signature": sig, "path": path})).collect::<Vec<_>>(),
        "known_absences": [
            "there is no per-expert routing exposed by llama.cpp, so no claim about which MoE expert answers a domain is verifiable here",
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The facts must not outlive the code they describe.
    #[test]
    fn every_listed_route_exists_in_the_dispatch() {
        let main_rs = include_str!("main.rs");
        for (sig, path) in ROUTES {
            assert!(
                main_rs.contains(&format!("\"{path}\"")),
                "ROUTES lists {sig} but main.rs has no literal \"{path}\""
            );
        }
    }

    #[test]
    fn council_port_matches_main() {
        let main_rs = include_str!("main.rs");
        assert!(
            main_rs.contains(&COUNCIL_PORT.to_string()),
            "COUNCIL_PORT {} does not appear in main.rs",
            COUNCIL_PORT
        );
    }

    #[test]
    fn facts_carry_all_six_axioms_and_three_ports() {
        let f = facts();
        let ax = f["axioms"]["axioms"].as_array().expect("axioms array");
        assert_eq!(ax.len(), 6, "expected six axioms, got {}", ax.len());
        let ports = f["ports"].as_object().expect("ports object");
        assert_eq!(ports.len(), 3, "expected council/vote_model/embedder");
        assert_ne!(f["ports"]["council"], f["ports"]["vote_model"]);
    }
}
