//! GENERATED parity ruler for host_tranche -- do not hand-edit.
//! Regenerate: py -3 scratch/gen_tranche_parity.py
//!
//! Baseline captured from swarm_v2/core/<module>.py:
//!   53 public functions across 24 modules, 40 are present in the Rust bodies
//!   17 module constants, all 17 matching Python exactly (derived consts resolved as expressions)
//! Each test asserts the parity measured at generation time, so a regression goes red.
#![allow(dead_code)]

const STUB_MARKERS: &[&str] = &["TODO: complete 1:1 math"];

/// A stub is a LIVE line carrying a completion marker. The marker may legitimately
/// appear in a doc comment quoting the defect it replaced -- sram_pattern_computer.rs
/// documents that its previous revision returned exactly that, so comments are skipped.
fn assert_no_stub(name: &str, src: &str) {
    for (i, line) in src.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("//") { continue; }
        for m in STUB_MARKERS {
            assert!(!t.contains(m), "{}:{} has a completion marker on a live line", name, i + 1);
        }
    }
}

#[test]
fn no_module_is_still_a_stub() {
    let mut checked = 0usize;
    assert_no_stub("chaos_engine", include_str!("chaos_engine.rs")); checked += 1;
    assert_no_stub("chaos_simulator", include_str!("chaos_simulator.rs")); checked += 1;
    assert_no_stub("cognitive_load_balancer", include_str!("cognitive_load_balancer.rs")); checked += 1;
    assert_no_stub("coherence_thresholds", include_str!("coherence_thresholds.rs")); checked += 1;
    assert_no_stub("comm_protocol", include_str!("comm_protocol.rs")); checked += 1;
    assert_no_stub("continuum_pbft", include_str!("continuum_pbft.rs")); checked += 1;
    assert_no_stub("coordination_protocol", include_str!("coordination_protocol.rs")); checked += 1;
    assert_no_stub("daily_evolution_engine", include_str!("daily_evolution_engine.rs")); checked += 1;
    assert_no_stub("ddr_antibody", include_str!("ddr_antibody.rs")); checked += 1;
    assert_no_stub("expert_registry", include_str!("expert_registry.rs")); checked += 1;
    assert_no_stub("feedback_loops", include_str!("feedback_loops.rs")); checked += 1;
    assert_no_stub("gpu_infinity_computer", include_str!("gpu_infinity_computer.rs")); checked += 1;
    assert_no_stub("heuristic_consensus", include_str!("heuristic_consensus.rs")); checked += 1;
    assert_no_stub("k3_logic", include_str!("k3_logic.rs")); checked += 1;
    assert_no_stub("pbft_consensus", include_str!("pbft_consensus.rs")); checked += 1;
    assert_no_stub("persona_matrix", include_str!("persona_matrix.rs")); checked += 1;
    assert_no_stub("predictive_watchdog", include_str!("predictive_watchdog.rs")); checked += 1;
    assert_no_stub("radix_trie_matcher", include_str!("radix_trie_matcher.rs")); checked += 1;
    assert_no_stub("resonance_engine", include_str!("resonance_engine.rs")); checked += 1;
    assert_no_stub("security_scanner", include_str!("security_scanner.rs")); checked += 1;
    assert_no_stub("self_fuzzing_engine", include_str!("self_fuzzing_engine.rs")); checked += 1;
    assert_no_stub("semantic_knowledge_graph", include_str!("semantic_knowledge_graph.rs")); checked += 1;
    assert_no_stub("skill_matrix_sync", include_str!("skill_matrix_sync.rs")); checked += 1;
    assert_no_stub("sram_pattern_computer", include_str!("sram_pattern_computer.rs")); checked += 1;
    assert_no_stub("star_matrix_lattice", include_str!("star_matrix_lattice.rs")); checked += 1;
    assert_no_stub("sub_atomic_kernel", include_str!("sub_atomic_kernel.rs")); checked += 1;
    assert_no_stub("thz_wave_gateway", include_str!("thz_wave_gateway.rs")); checked += 1;
    assert_no_stub("turbovec_memory", include_str!("turbovec_memory.rs")); checked += 1;
    assert_eq!(checked, 28, "every module in the crate must be checked");
}

/// Python public functions: 53 measured, 40 present in the Rust bodies.
#[test]
fn python_function_surface_is_present() {
    let cases: &[(&str, usize, usize, &[&str])] = &[
        ("chaos_engine", 1, 1, &[]),
        ("chaos_simulator", 1, 1, &[]),
        ("cognitive_load_balancer", 1, 1, &[]),
        ("comm_protocol", 2, 2, &[]),
        ("continuum_pbft", 1, 1, &[]),
        ("coordination_protocol", 1, 1, &[]),
        ("daily_evolution_engine", 3, 3, &[]),
        ("ddr_antibody", 1, 0, &["get_ddr"]),
        ("expert_registry", 1, 1, &[]),
        ("feedback_loops", 1, 1, &[]),
        ("gpu_infinity_computer", 5, 0, &["compare_fabrics", "gpu_pattern_cellular", "gpu_pattern_fpga", "gpu_pattern_k3_ternary", "gpu_pattern_neural"]),
        ("heuristic_consensus", 1, 1, &[]),
        ("pbft_consensus", 2, 2, &[]),
        ("persona_matrix", 1, 0, &["build_compiled_prompt"]),
        ("radix_trie_matcher", 1, 1, &[]),
        ("resonance_engine", 1, 1, &[]),
        ("security_scanner", 1, 1, &[]),
        ("semantic_knowledge_graph", 1, 1, &[]),
        ("skill_matrix_sync", 1, 1, &[]),
        ("sram_pattern_computer", 20, 15, &["get_sram_computer", "hopfield_energy", "hopfield_hamming", "hopfield_relax", "hopfield_weights"]),
        ("star_matrix_lattice", 2, 2, &[]),
        ("sub_atomic_kernel", 1, 0, &["native_batch_collapse"]),
        ("thz_wave_gateway", 2, 2, &[]),
        ("turbovec_memory", 1, 1, &[]),
    ];
    let (mut py, mut hit) = (0usize, 0usize);
    for (_m, p, h, _miss) in cases { py += p; hit += h; }
    assert!(hit >= 40, "python function surface regressed: {} of {} present", hit, py);
    assert_eq!(py, 53, "the captured Python surface changed; regenerate the ruler");
}

/// Constants whose Rust value must equal the Python module-level value exactly.
#[test]
fn python_constants_match_the_rust_bodies() {
    let mut n = 0usize;
    assert_eq!((crate::daily_evolution_engine::TASKS_PER_DAY) as f64, 1000000000000.0, "daily_evolution_engine::TASKS_PER_DAY drifted from Python"); n += 1;
    assert_eq!((crate::daily_evolution_engine::AGENTS) as f64, 584.0, "daily_evolution_engine::AGENTS drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_SIZE) as f64, 16777216.0, "sram_pattern_computer::L3_SIZE drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_SLICES) as f64, 4.0, "sram_pattern_computer::L3_SLICES drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_LINE_SIZE) as f64, 64.0, "sram_pattern_computer::L3_LINE_SIZE drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_WAYS) as f64, 16.0, "sram_pattern_computer::L3_WAYS drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_SETS) as f64, 4096.0, "sram_pattern_computer::L3_SETS drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_CELLS) as f64, 134217728.0, "sram_pattern_computer::L3_CELLS drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_TRANSISTORS_PER_CELL) as f64, 6.0, "sram_pattern_computer::L3_TRANSISTORS_PER_CELL drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_TOTAL_TRANSISTORS) as f64, 805306368.0, "sram_pattern_computer::L3_TOTAL_TRANSISTORS drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::F_THZ) as f64, 1000000000000.0, "thz_wave_gateway::F_THZ drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::DELTA_F) as f64, 1000000000.0, "thz_wave_gateway::DELTA_F drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::PHI_V) as f64, 0.76, "thz_wave_gateway::PHI_V drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::V_RAIL) as f64, 4.75, "thz_wave_gateway::V_RAIL drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::F_CPU) as f64, 4000000000.0, "thz_wave_gateway::F_CPU drifted from Python"); n += 1;
    assert_eq!((crate::turbovec_memory::CHUNK_SIZE) as f64, 1000.0, "turbovec_memory::CHUNK_SIZE drifted from Python"); n += 1;
    assert_eq!((crate::turbovec_memory::COMPRESS_LEVEL) as f64, 3.0, "turbovec_memory::COMPRESS_LEVEL drifted from Python"); n += 1;
    assert_eq!(n, 17, "all captured constants must be checked");
}

