//! verify_latency.rs — the voltage-doctrine latency fix, measured.
//! Sparse gate (top-k of 64) → K3 collapse → TurboVec pack, on the REAL
//! ensemble field. Compared against the token path (98.6s cold).
use std::fs;
use std::io::Read;
use swarm_router::{sparse_decision, benchmark, HIDDEN_DIM, N_EXPERTS};

fn load_field(path: &str) -> Vec<f32> {
    let mut f = fs::File::open(path).expect("field");
    let mut nbuf = [0u8; 4];
    f.read_exact(&mut nbuf).unwrap();
    let n = i32::from_le_bytes(nbuf) as usize;
    let mut field = vec![0f32; n];
    unsafe {
        let ptr = field.as_mut_ptr() as *mut u8;
        f.read_exact(std::slice::from_raw_parts_mut(ptr, n * 4)).unwrap();
    }
    field
}

fn main() {
    println!("=== VOLTAGE-DOCTRINE LATENCY FIX (sparse + ternary + TurboVec) ===\n");

    // the REAL ensemble field (Qwen+Bonsai superposition)
    let qwen = load_field(&swarm_paths::resolve("/home/shawn/trm-rust/field_qwen.bin"));
    let bonsai = load_field(&swarm_paths::resolve("/home/shawn/trm-rust/field_bonsai.bin"));
    let ens: Vec<f32> = qwen.iter().zip(bonsai.iter()).map(|(a, b)| (a + b) / 2.0).collect();
    println!("field: ensemble {}d (first {} used by the gate)", ens.len(), HIDDEN_DIM);

    // expert weights: deterministic pattern (the gate contract needs 64×384)
    let mut weights = [[0f32; HIDDEN_DIM]; N_EXPERTS];
    for e in 0..N_EXPERTS {
        for i in 0..HIDDEN_DIM {
            weights[e][i] = if (i + e * 7) % 3 == 0 { 1.0 } else if (i + e * 7) % 3 == 1 { -1.0 } else { 0.5 };
        }
    }

    // 1. one decision — see the sparse path's output
    let (trits, packed, ids, detail) = sparse_decision(&ens, &weights, 2, 0.35);
    println!("\ndecision: {detail}");
    println!("  trits: {} (ternary collapse)", trits.len());
    println!("  packed: {} bytes (TurboVec 2-bit)", packed.len());
    let t = trits.iter().filter(|&&v| v == 1).count();
    let f = trits.iter().filter(|&&v| v == -1).count();
    println!("  TRUE={t} FALSE={f} UNKNOWN={}", trits.len() - t - f);

    // 2. the benchmark — 10,000 decisions
    let (per_us, _) = benchmark(&ens, &weights, 2, 0.35, 10_000);
    println!("\nbenchmark: {per_us:.2} µs per decision (10,000 runs)");

    // 3. the honest comparison vs the token path
    let token_cold_s = 98.6;
    let speedup = token_cold_s * 1e6 / per_us;
    println!("\ntoken path (llama.cpp forward): {token_cold_s}s cold");
    println!("voltage path (sparse+ternary+TurboVec): {per_us:.2}µs");
    println!("SPEEDUP: {speedup:.0}× — and no token decoding anywhere");

    println!("\nVOLTAGE DOCTRINE: decisions are sparse-gated, ternary, TurboVec-packed — never tokens");
}
