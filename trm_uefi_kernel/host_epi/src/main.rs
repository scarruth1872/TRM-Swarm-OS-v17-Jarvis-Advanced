//! Host verification of the Benchmark Epistemology Map render.
mod benchmark_epistemology;

fn main() {
    let lines = benchmark_epistemology::render_map(5);
    for ln in &lines {
        println!("{}", ln);
    }
    println!();
    println!("VERIFY: count={} (expect 5), metrics={} (expect 5), seal=0x{:016X} (expect 6e30d528...)",
        benchmark_epistemology::benchmark_count(),
        benchmark_epistemology::REAL_WORLD_METRICS.len(),
        benchmark_epistemology::COUNCIL_SEAL);
}
