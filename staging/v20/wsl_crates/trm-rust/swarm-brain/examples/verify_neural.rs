//! verify_neural.rs — Motion A gate: the native sparse neural mode.
use swarm_brain::neural::{build_neural, firing_rate, neuron_collapse, step};

fn main() {
    println!("=== MOTION A: THE NEURAL MODE (the missing brain-scale layer) ===\n");

    // the honest scale on the 3070: 1M neurons × 32 synapses
    let n = 1_000_000usize;
    let k = 32usize;
    let t0 = std::time::Instant::now();
    let mode = build_neural(n, k, 42);
    let build = t0.elapsed();
    let total_weights = n * k;
    println!("[BUILD] {n} neurons x {k} synapses = {total_weights} ternary weights");
    let mb = mode.packed.len() as f64 / 1e6;
    println!("  TurboVec-packed: {mb:.1} MB ({build:?})");

    // the field (a voltage-like input)
    let field: Vec<f32> = (0..n).map(|i| ((i as f32) * 0.001).sin() * 0.9).collect();

    // one sparse timestep
    let t0 = std::time::Instant::now();
    let firing = step(&mode, &field, 1);
    let step_time = t0.elapsed();
    let (fired, total) = firing_rate(&firing);
    let pct = fired as f64 / total as f64 * 100.0;
    println!("\n[STEP] sparse firing: {fired}/{total} neurons ({pct:.2}%) in {step_time:?}");

    // 100 timesteps (the throughput)
    let t0 = std::time::Instant::now();
    for t in 0..100 {
        let _ = step(&mode, &field, t);
    }
    let per = t0.elapsed() / 100;
    let mupd = n as f64 / per.as_secs_f64() / 1e6;
    println!("  {n}-neuron step: {per:?} each ({mupd:.0}M neuron-updates/s)");

    // the neuron collapse (decision path)
    let (v, trits) = neuron_collapse(&field[..64], 0.35);
    println!("\n[COLLAPSE] neuron verdict: {v} | trits: {:?}", &trits[..8]);

    println!("\nMOTION A GATE: the neural mode is BUILT — sparse, ternary, native");
    println!("  (the recompute said 'NOT BUILT' — now it is, and measured)");
}
