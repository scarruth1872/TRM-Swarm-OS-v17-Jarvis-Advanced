//! verify_switch.rs — Motion D gate: pattern-modulated switching, measured.
use swarm_brain::pattern_switch::{switch_loop, Architecture, compute};

fn main() {
    println!("=== MOTION D: PATTERN-MODULATED ARCHITECTURE SWITCHING ===\n");
    println!("the brain's insight: one substrate, multiple minds — switch by pattern\n");

    // a real field (e.g., a voltage state from the model)
    let field: Vec<f32> = (0..64).map(|i| {
        let x = i as f32;
        (x * 0.3).sin() * 0.8 + (x * 0.11).cos() * 0.4
    }).collect();

    // 1. the switching loop (all four architectures, same silicon)
    let (results, us) = switch_loop(&field, 42);
    println!("switching loop: {} architectures, {us:.2} µs total\n", results.len());
    for (arch, note) in &results {
        println!("  {:>18} → {note}", arch.as_str());
    }

    // 2. per-architecture detail on the same field
    println!("\nper-architecture outputs (same field, different minds):");
    let (k3, _) = compute(Architecture::K3, &field, 42);
    let (ca, _) = compute(Architecture::CellularAutomaton, &field, 42);
    let (ising, _) = compute(Architecture::Ising, &field, 42);
    println!("  K3   trits:   {:?}", &k3[..8]);
    println!("  CA   trits:   {:?}", &ca[..8]);
    println!("  Ising trits:  {:?}", &ising[..8]);

    // 3. the switching rate (10,000 loops)
    let t0 = std::time::Instant::now();
    for _ in 0..10_000 {
        let _ = switch_loop(&field, 42);
    }
    let per = t0.elapsed().as_secs_f64() / 10_000.0 * 1e6;
    println!("\n10,000 switching loops: {per:.2} µs per full 4-architecture cycle");

    println!("\nMOTION D GATE: one substrate, four minds, {per:.2} µs to switch between them");
}
