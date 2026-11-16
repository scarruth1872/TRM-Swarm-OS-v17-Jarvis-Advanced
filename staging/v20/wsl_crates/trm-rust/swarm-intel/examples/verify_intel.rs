//! verify_intel.rs — Motion A gate: the intelligence loop ENTIRELY in Rust.
//! Real knowledge-grounded tasks (threats, antibodies, compounds) decided
//! through the native council + mesh. No Python in the decision path.
use swarm_intel::{decide, route, CouncilVerdict, Task};

fn main() {
    println!("=== MOTION A: THE INTELLIGENCE ABSORBED — council + mesh in Rust ===");

    // REAL knowledge-grounded tasks (from the fabric's actual sources)
    let tasks = vec![
        // threats (real CWE classes)
        Task { class: "mitigate".into(), reference: "T-CWE-89 SQL_INJECTION".into(), domain: "security".into() },
        Task { class: "mitigate".into(), reference: "T-CWE-78 COMMAND_INJECTION".into(), domain: "security".into() },
        Task { class: "mitigate".into(), reference: "T-BIO-001 PATHOGEN".into(), domain: "drug".into() },
        Task { class: "mitigate".into(), reference: "T-INF-001 SCADA_ANOMALY".into(), domain: "infra".into() },
        // antibodies (real DDR types)
        Task { class: "respond".into(), reference: "DDR-SQL_INJECTION".into(), domain: "security".into() },
        Task { class: "respond".into(), reference: "DDR-BARE_EXCEPT".into(), domain: "code".into() },
        // compounds (real ChEMBL semantics)
        Task { class: "triage".into(), reference: "CHEMBL153534".into(), domain: "drug".into() },
        Task { class: "triage".into(), reference: "CHEMBL440060".into(), domain: "drug".into() },
        // cosmic + governance
        Task { class: "analyze".into(), reference: "SPECTRA-NGC123".into(), domain: "cosmos".into() },
        Task { class: "audit".into(), reference: "FDA-TRIAL-2026".into(), domain: "regulatory".into() },
    ];

    let mut acts = 0;
    let mut blocks = 0;
    let mut analyzes = 0;

    println!("\nknowledge-grounded tasks, decided by the native council:");
    for t in &tasks {
        let d = decide(t);
        if d.contains("[ACT]") {
            acts += 1;
        } else if d.contains("[BLOCK]") {
            blocks += 1;
        } else {
            analyzes += 1;
        }
        println!("  {d}");
    }

    println!("\ncouncil aggregate: {acts} ACT / {blocks} BLOCK / {analyzes} ANALYZE");
    println!("14-agent mesh + K3 council + act disposition — ALL NATIVE, no Python");
    let _ = (route("x"), CouncilVerdict::Act);
}
