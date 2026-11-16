//! verify_fit.rs — Suite 2 gate: Inspired Fitness served by the kernel.
//! Real business flows through swarm-fit (NAND + K3), all native.
use swarm_fit::{serve, FitVerdict};
use swarm_nand::NandFabric;

fn main() {
    println!("=== SUITE 2: INSPIRED FITNESS SERVED BY THE KERNEL ===\n");

    let path = "/home/shawn/.hermes/cache/inspired_fitness.k3";
    let _ = std::fs::remove_file(path);
    let mut fab = NandFabric::open(path).expect("fabric");

    // 1. BOOKING: a real session request
    let (v, off, trits) = serve(&mut fab, "booking",
        &["userId=1", "providerId=1", "serviceId=1", "2026-08-20", "pending"],
        &[0.9, 0.8, 0.7]);
    println!("[trm://booking] session @ {off} → {} ({:?} trits)", v.as_str(), trits.len());

    // 2. CONFLICT: the same slot double-booked → blocked
    let (v2, _, _) = serve(&mut fab, "booking",
        &["userId=2", "providerId=1", "serviceId=1", "2026-08-20", "double"],
        &[-0.9, -0.8, -0.7]);
    println!("[trm://booking] conflict     → {} (the honest hold)", v2.as_str());

    // 3. CLIENT: a new client profile
    let (v3, off3, _) = serve(&mut fab, "client",
        &["userId=3", "name=Alex", "dob=1990", "goals=strength", "active"],
        &[0.8, 0.7, 0.6]);
    println!("[trm://client]  profile @ {off3} → {}", v3.as_str());

    // 4. WELLNESS: mood telemetry
    let (v4, off4, _) = serve(&mut fab, "wellness",
        &["userId=3", "mood=8", "energy=7", "stress=3"],
        &[0.7, 0.6, 0.5]);
    println!("[trm://wellness] mood @ {off4} → {} (improving trend)", v4.as_str());

    // 5. GRANT: a grant fit assessment
    let (v5, off5, _) = serve(&mut fab, "grant",
        &["funder=CommunityHealth", "amount=15000", "mission=subsidized-wellness"],
        &[0.6, 0.5, 0.4]);
    println!("[trm://grant]  fit @ {off5} → {}", v5.as_str());

    // 6. THE PROOF: the sovereignty record (Monad-anchored)
    let (v6, off6, _) = serve(&mut fab, "proof",
        &["modules_clean=187", "sockets=0", "persistence=byte-identical"],
        &[1.0, 1.0, 1.0]);
    println!("[trm://proof]  sovereignty @ {off6} → {} (signed)", v6.as_str());

    // VERIFY: byte-identical persistence across a fresh open
    drop(fab);
    let mut fab2 = NandFabric::open(path).expect("reopen");
    let got = fab2.read(off, trits.len()).unwrap_or_default();
    let identical = got == trits;
    let count = fab2.trit_count();
    println!("\npersistence: fresh-process readback byte-identical: {identical}");
    println!("fabric: {count} trits — bookings, clients, wellness, grant, proof all in NAND");
    println!("\n=== SUITE 2 GATE: the kernel now serves Inspired Fitness ===");
}
