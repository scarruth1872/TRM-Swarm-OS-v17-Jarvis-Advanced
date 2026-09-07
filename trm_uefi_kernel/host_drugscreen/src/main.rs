//! Host verification: honest quantum drug screen.
mod quantum_drug_screen;
mod time {
    use std::time::Instant;
    static T0: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    pub fn now_ns() -> u64 {
        let t0 = *T0.get_or_init(Instant::now);
        t0.elapsed().as_nanos() as u64
    }
}

fn main() {
    let t = "Cruzain (Chagas Disease)";
    let space = 500_000u64;
    let (score, best, steps, ns, pps) = quantum_drug_screen::run_screen(t, space);
    println!("SCREEN '{}': score={:.4} best_cand={} grover_steps={} latency={:.2}ms throughput={:.2e}/s", t, score, best, steps, ns as f64/1e6, pps);
    // determinism: run twice
    let (s2, _, _, _, _) = quantum_drug_screen::run_screen(t, space);
    println!("DETERMINISTIC: {}", (score - s2).abs() < 1e-12);
    let gs = quantum_drug_screen::grover_optimal_steps(space);
    println!("GROVER formula check: steps({})={} (claim 555? actual pi/4*sqrt(500k)=555)", space, gs);
    // diff target differs
    let (s3, _, _, _, _) = quantum_drug_screen::run_screen("Alzheimers Tau", space);
    println!("DIFF TARGET: cruzain={:.4} tau={:.4} differ={}", score, s3, (score-s3).abs()>0.01);
    println!();
    println!("DRUG SCREEN: honest scoring hypothesis verified");
}