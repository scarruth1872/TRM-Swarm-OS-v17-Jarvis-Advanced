//! pattern_switch.rs — Motion D: pattern-modulated architecture switching.
//!
//! The brain's multi-architecture insight, made native: ONE substrate
//! switches between computational architectures by pattern modulation.
//!   K3   → decision collapse (the verified chamber)
//!   CA   → cellular automaton (spatial maps)
//!   Ising → stochastic synaptic release (DRAM RowHammer-style)
//!   NAND → persistent synaptic weights (the fabric)
//!
//! The switch is measured — the same silicon, four minds, one loop.

use swarm_k3::collapse;

/// The four brain architectures the substrate can become.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    K3,     // decision collapse
    CellularAutomaton, // spatial maps (hippocampus)
    Ising,  // stochastic release
    Nand,   // persistent weights
}

impl Architecture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Architecture::K3 => "K3-decision",
            Architecture::CellularAutomaton => "CA-spatial",
            Architecture::Ising => "Ising-stochastic",
            Architecture::Nand => "NAND-persistent",
        }
    }
}

/// One computation in the current architecture. Returns (output, note).
/// The SAME input field is interpreted differently by each architecture —
/// that is pattern modulation.
pub fn compute(arch: Architecture, field: &[f32], seed: u64) -> (Vec<i8>, String) {
    match arch {
        Architecture::K3 => {
            // decision collapse (the verified semantics)
            let trits = collapse(field, 0.35);
            let t = trits.iter().filter(|&&v| v == 1).count();
            let note = format!("collapsed: {} TRUE / {} FALSE", t, trits.len() - t);
            (trits, note)
        }
        Architecture::CellularAutomaton => {
            // elementary CA rule 30 on the field as a binary state row
            let n = field.len().min(64);
            let mut state: Vec<u8> = field.iter().take(n)
                .map(|&v| if v > 0.0 { 1 } else { 0 }).collect();
            let mut x = seed as u64;
            for _ in 0..8 {
                let mut next = state.clone();
                for i in 0..n {
                    let l = state[(i + n - 1) % n];
                    let c = state[i];
                    let r = state[(i + 1) % n];
                    // rule 30: (l XOR (c OR r))
                    next[i] = l ^ (c | r);
                    x = x.wrapping_mul(0x100000001b3) ^ (next[i] as u64);
                }
                state = next;
            }
            let alive = state.iter().filter(|&&s| s == 1).count();
            let trits: Vec<i8> = state.iter().map(|&s| if s == 1 { 1 } else { -1 }).collect();
            (trits, format!("rule-30: {alive}/{} cells alive after 8 gens", n))
        }
        Architecture::Ising => {
            // stochastic release: flip trits by a seeded coin (thermal noise)
            let trits = collapse(field, 0.35);
            let mut x = seed;
            let mut flips = 0;
            let out: Vec<i8> = trits.iter().map(|&t| {
                x = x.wrapping_mul(0x100000001b3).wrapping_add(0x9E3779B97F4A7C15);
                x ^= x >> 33;
                if (x & 0xFF) < 20 { // ~8% release probability
                    flips += 1;
                    -t
                } else {
                    t
                }
            }).collect();
            (out, format!("stochastic release: {flips} trits flipped (~8%)"))
        }
        Architecture::Nand => {
            // persistence: the collapse result as the fabric record
            let trits = collapse(field, 0.35);
            let note = format!("persisted: {} trits ready for the fabric", trits.len());
            (trits, note)
        }
    }
}

/// The pattern-modulated loop: switch through all four architectures
/// on the same field, measure each. Returns (results, total_us).
pub fn switch_loop(field: &[f32], seed: u64) -> (Vec<(Architecture, String)>, f64) {
    let archs = [
        Architecture::K3,
        Architecture::CellularAutomaton,
        Architecture::Ising,
        Architecture::Nand,
    ];
    let t0 = std::time::Instant::now();
    let mut results = Vec::new();
    for a in archs {
        let (_, note) = compute(a, field, seed);
        results.push((a, note));
    }
    let dt = t0.elapsed();
    (results, dt.as_secs_f64() * 1e6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_architectures_run() {
        let field = vec![0.8, -0.6, 0.2, -0.9, 0.5, 0.1, -0.4, 0.7];
        for a in [Architecture::K3, Architecture::CellularAutomaton,
                  Architecture::Ising, Architecture::Nand] {
            let (out, note) = compute(a, &field, 42);
            assert!(!out.is_empty());
            assert!(!note.is_empty());
        }
    }

    #[test]
    fn switch_loop_is_fast() {
        let field = vec![0.8, -0.6, 0.2, -0.9, 0.5, 0.1, -0.4, 0.7];
        let (results, us) = switch_loop(&field, 42);
        assert_eq!(results.len(), 4);
        assert!(us < 1000.0, "switching too slow: {us}us");
    }

    #[test]
    fn ising_release_is_deterministic() {
        let field = vec![0.8, -0.6, 0.2, -0.9];
        let (a, _) = compute(Architecture::Ising, &field, 7);
        let (b, _) = compute(Architecture::Ising, &field, 7);
        assert_eq!(a, b);
    }
}
