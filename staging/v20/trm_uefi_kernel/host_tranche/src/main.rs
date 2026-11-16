//! TRM Swarm OS - host_tranche entry point.
//!
//! Every module in this crate is declared here. The tranche is the portable,
//! dependency-free form of the kernel: same no_std bodies, buildable for any
//! target with a single rustc invocation (see the strip build recipe).
//!
//! A module that is not declared is not compiled -- which is how the stubs
//! survived here unnoticed.

pub mod time { use std::time::Instant; static T0: std::sync::OnceLock<Instant> = std::sync::OnceLock::new(); pub fn now_ns() -> u64 { let t0 = *T0.get_or_init(Instant::now); t0.elapsed().as_nanos() as u64 } }

mod chaos_engine;
mod chaos_simulator;
mod cognitive_load_balancer;
mod coherence_thresholds;
mod comm_protocol;
mod continuum_pbft;
mod coordination_protocol;
mod daily_evolution_engine;
mod ddr_antibody;
mod expert_registry;
mod feedback_loops;
mod gpu_infinity_computer;
mod heuristic_consensus;
mod k3_logic;
mod pbft_consensus;
mod persona_matrix;
mod predictive_watchdog;
mod radix_trie_matcher;
mod resonance_engine;
mod security_scanner;
mod self_fuzzing_engine;
mod semantic_knowledge_graph;
mod skill_matrix_sync;
mod sram_pattern_computer;
mod star_matrix_lattice;
mod sub_atomic_kernel;
mod thz_wave_gateway;
mod turbovec_memory;

#[cfg(test)]
mod parity_tests;

fn main() {
    // A value the stub could not produce: get_expert_registry() returned a literal 0
    // with "TODO: complete 1:1 math" before the body was ported.
    println!("expert_registry singleton = {}", expert_registry::get_expert_registry());
}
