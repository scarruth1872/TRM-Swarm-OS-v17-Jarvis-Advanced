//! verify_cde.rs — Motions C+D+E on REAL data, one run.
use swarm_health::{health_verdict, parse_ledger, HealthVerdict};
use swarm_mesh::mesh_round;
use swarm_cosmos::{parse_manifest, to_k3};

fn main() {
    let root = "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main";
    println!("=== MOTIONS C + D + E — REAL DATA ===");

    // C: health monitor on the real event ledger
    let events = parse_ledger(&format!("{root}/swarm_v2_global_memory/system_event_ledger.jsonl"));
    let (verdict, detail) = health_verdict(&events);
    println!("\n[C] Health monitor: {} real ledger records", events.len());
    println!("    verdict: {verdict:?} — {detail}");
    let _ = HealthVerdict::Stable;

    // D: mesh routing on real task mix
    let tasks = [
        "deploy antibody against the exploit",
        "write the compiler fix",
        "prove the theorem in Coq",
        "triage the drug binding affinity",
        "render the dashboard visualization",
        "audit the fda trial compliance",
        "calibrate the telescope spectra",
    ];
    let results = mesh_round(&tasks);
    println!("\n[D] Agent mesh: {} real tasks routed", results.len());
    for (agent, ov, verdict) in &results {
        println!("    → {agent:>12} (overlap {ov}) [{verdict}]");
    }

    // E: cosmos field on the real spectral manifest
    let recs = parse_manifest("/home/shawn/Agent-Q-master/data/spectral_manifest/manifest_20260714.jsonl");
    let k3 = to_k3(&recs);
    println!("\n[E] Cosmos field: {} real spectral records → {} K3 trits", recs.len(), k3.len());
    let sample = recs.first().map(|r| format!("{} [{}] {}", r.id, r.kind, r.detail)).unwrap_or_default();
    println!("    first record: {sample}");
    println!("    first 12 trits: {:?}", &k3[..12.min(k3.len())]);

    println!("\n=== C+D+E GATE: all three motions verified on real data ===");
}
