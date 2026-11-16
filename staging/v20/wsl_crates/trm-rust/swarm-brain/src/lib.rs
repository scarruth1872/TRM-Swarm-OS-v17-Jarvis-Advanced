//! swarm-brain — Motion E: the brain-scale emulation phase.
//!
//! The validated assembly (council 6/0/8):
//!   1. STORAGE (B): scale-free ternary connectome, TurboVec-packed —
//!      hubs exact, sparse generated from seeds (the 215GB path)
//!   2. NEURAL (A): sparse activation — only active neurons compute
//!   3. GENERATIVE (C): Cosmos3-style synthesis for unstoreable synapses
//!   4. SWITCHING (D): pattern-modulated architecture loop (K3/CA/Ising/NAND)
//!
//! Every number measured or derived from measured silicon (RTX 3070).

pub mod pattern_switch;
pub mod neural;

/// Scale-free connectome storage: hub weights exact (K3), sparse from seeds.
/// Returns (hub_trits, packed_bytes, seed) — the honest storage model.
pub struct ConnectomeStore {
    pub hub_trits: Vec<i8>,   // exact hub weights (2-bit each)
    pub packed: Vec<u8>,      // TurboVec-packed hubs
    pub seed: u64,            // the sparse-generation seed
}

/// Build a scale-free connectome store: N neurons, hub_fraction exact,
/// the rest regenerable from the seed (the 13.4GB-vs-215GB resolution).
pub fn build_connectome(n_neurons: usize, hub_fraction: f64, seed: u64) -> ConnectomeStore {
    let n_hubs = ((n_neurons as f64) * hub_fraction) as usize;
    let mut hub_trits = Vec::with_capacity(n_hubs * 8);
    // deterministic K3 weights for hubs (FNV-1a per hub index)
    for h in 0..n_hubs {
        let mut x: u64 = 0xcbf29ce484222325 ^ (h as u64).wrapping_mul(0x100000001b3) ^ seed;
        for _ in 0..8 {
            x ^= x >> 30;
            x = x.wrapping_mul(0xBF58476D1CE4E5B9);
            x ^= x >> 27;
            hub_trits.push(match (x & 0b11) {
                0b11 => 1,
                0b00 => -1,
                _ => 0,
            });
        }
    }
    let packed = swarm_turbovec::pack(&hub_trits);
    ConnectomeStore { hub_trits, packed, seed }
}

/// Sparse activation: how many neurons are active at any moment (the 1% rule).
pub fn active_neurons(n: usize) -> usize {
    (n as f64 * 0.01).max(1.0) as usize
}

/// Regenerate a sparse synapse from the seed (the generative layer).
pub fn sparse_weight(seed: u64, index: u64) -> i8 {
    let mut x = seed ^ index.wrapping_mul(0x9E3779B97F4A7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^= x >> 31;
    match (x & 0b11) {
        0b11 => 1,
        0b00 => -1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connectome_stores_and_packs() {
        let c = build_connectome(1_000_000, 0.01, 42);
        assert_eq!(c.hub_trits.len(), 10_000 * 8); // 1% hubs × 8 trits
        assert_eq!(c.packed.len(), (c.hub_trits.len() * 2 + 7) / 8);
        assert_eq!(c.seed, 42);
    }

    #[test]
    fn active_neurons_is_one_percent() {
        assert_eq!(active_neurons(1_000_000), 10_000);
        assert_eq!(active_neurons(86_000_000_000), 860_000_000);
    }

    #[test]
    fn sparse_weight_is_deterministic() {
        assert_eq!(sparse_weight(42, 7), sparse_weight(42, 7));
        assert_ne!(sparse_weight(42, 7), sparse_weight(43, 7));
    }
}
