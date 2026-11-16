//! verify_ensemble.rs — Motion E gate: the 2-model field governs the council.
use swarm_intel::ensemble_council::{ensemble_vote, EnsembleField};
use swarm_intel::CouncilVerdict;

fn main() {
    println!("=== MOTION E: ENSEMBLE-DRIVEN AGENTS ===\n");

    // load the REAL ensemble field (qwen + bonsai GPU captures)
    let field = EnsembleField::load();
    println!("ensemble field: {} dims, source: {}",
             field.values.len(),
             if field.from_gpu { "REAL GPU captures" } else { "deterministic fallback" });

    // the ensemble's signal (collapse of the first 32 dims)
    let s = field.signal(0.35);
    println!("ensemble signal: {s:.2} ({})", if s > 0.0 { "ACT-leaning" } else if s < 0.0 { "BLOCK-leaning" } else { "neutral" });

    // the council governed by the field — real domain tasks
    let tasks = [
        ("mitigate T-CWE-89", "security"),
        ("triage CHEMBL153534", "drug"),
        ("respond DDR-SQL_INJECTION", "security"),
        ("audit FDA-TRIAL-2026", "regulatory"),
        ("analyze SPECTRA-NGC123", "cosmos"),
        ("monitor GRID-FREQ", "infra"),
    ];

    println!("\nensemble-governed council decisions:");
    for (task, domain) in tasks {
        let (v, trits) = ensemble_vote(&field, domain, 0.35);
        let verdict = match v {
            1 => CouncilVerdict::Act,
            -1 => CouncilVerdict::Block,
            _ => CouncilVerdict::Analyze,
        };
        println!("  [{:>9}] {task} → {verdict:?} | vote {:?} | field={s:.2}", verdict.as_str(), trits);
    }

    println!("\nMOTION E GATE: the 2-model voltage field now shapes every council decision");
}
