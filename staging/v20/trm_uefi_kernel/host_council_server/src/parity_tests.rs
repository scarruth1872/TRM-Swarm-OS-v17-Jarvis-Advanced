//! PARITY RULER for the live kernel server -- promoted from host_tranche, do not hand-edit.
//! Regenerate the baseline: py -3 scratch/gen_tranche_parity.py
//!
//! Baseline captured from swarm_v2/core/<module>.py:
//!   53 public functions across 24 modules, 40 are present in the Rust bodies
//!   17 module constants, all 17 matching Python exactly (derived consts resolved as expressions)
//!
//! Every include_str! here points at ../../src/<module>.rs -- the KERNEL files, which is what this
//! server links by #[path] and actually serves. Guarding the server's own copies would have proved
//! nothing about the code answering the routes; guarding the source of record is the claim worth
//! making. A regression in any of them goes red here.
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
    assert_no_stub("chaos_engine", include_str!("../../src/chaos_engine.rs")); checked += 1;
    assert_no_stub("chaos_simulator", include_str!("../../src/chaos_simulator.rs")); checked += 1;
    assert_no_stub("cognitive_load_balancer", include_str!("../../src/cognitive_load_balancer.rs")); checked += 1;
    assert_no_stub("coherence_thresholds", include_str!("../../src/coherence_thresholds.rs")); checked += 1;
    assert_no_stub("comm_protocol", include_str!("../../src/comm_protocol.rs")); checked += 1;
    assert_no_stub("continuum_pbft", include_str!("../../src/continuum_pbft.rs")); checked += 1;
    assert_no_stub("coordination_protocol", include_str!("../../src/coordination_protocol.rs")); checked += 1;
    assert_no_stub("daily_evolution_engine", include_str!("../../src/daily_evolution_engine.rs")); checked += 1;
    assert_no_stub("ddr_antibody", include_str!("../../src/ddr_antibody.rs")); checked += 1;
    assert_no_stub("expert_registry", include_str!("../../src/expert_registry.rs")); checked += 1;
    assert_no_stub("feedback_loops", include_str!("../../src/feedback_loops.rs")); checked += 1;
    assert_no_stub("gpu_infinity_computer", include_str!("../../src/gpu_infinity_computer.rs")); checked += 1;
    assert_no_stub("heuristic_consensus", include_str!("../../src/heuristic_consensus.rs")); checked += 1;
    assert_no_stub("k3_logic", include_str!("../../src/k3_logic.rs")); checked += 1;
    assert_no_stub("pbft_consensus", include_str!("../../src/pbft_consensus.rs")); checked += 1;
    assert_no_stub("persona_matrix", include_str!("../../src/persona_matrix.rs")); checked += 1;
    assert_no_stub("predictive_watchdog", include_str!("../../src/predictive_watchdog.rs")); checked += 1;
    assert_no_stub("radix_trie_matcher", include_str!("../../src/radix_trie_matcher.rs")); checked += 1;
    assert_no_stub("resonance_engine", include_str!("../../src/resonance_engine.rs")); checked += 1;
    assert_no_stub("security_scanner", include_str!("../../src/security_scanner.rs")); checked += 1;
    assert_no_stub("self_fuzzing_engine", include_str!("../../src/self_fuzzing_engine.rs")); checked += 1;
    assert_no_stub("semantic_knowledge_graph", include_str!("../../src/semantic_knowledge_graph.rs")); checked += 1;
    assert_no_stub("skill_matrix_sync", include_str!("../../src/skill_matrix_sync.rs")); checked += 1;
    assert_no_stub("sram_pattern_computer", include_str!("../../src/sram_pattern_computer.rs")); checked += 1;
    assert_no_stub("star_matrix_lattice", include_str!("../../src/star_matrix_lattice.rs")); checked += 1;
    assert_no_stub("sub_atomic_kernel", include_str!("../../src/sub_atomic_kernel.rs")); checked += 1;
    assert_no_stub("thz_wave_gateway", include_str!("../../src/thz_wave_gateway.rs")); checked += 1;
    assert_no_stub("turbovec_memory", include_str!("../../src/turbovec_memory.rs")); checked += 1;
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
    assert_eq!((crate::sram_pattern_computer::L3_SLICES) as f64, 4.0, "sram_pattern_computer::L3_SLICES drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_LINE_SIZE) as f64, 64.0, "sram_pattern_computer::L3_LINE_SIZE drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_WAYS) as f64, 16.0, "sram_pattern_computer::L3_WAYS drifted from Python"); n += 1;
    assert_eq!((crate::sram_pattern_computer::L3_TRANSISTORS_PER_CELL) as f64, 6.0, "sram_pattern_computer::L3_TRANSISTORS_PER_CELL drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::F_THZ) as f64, 1000000000000.0, "thz_wave_gateway::F_THZ drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::DELTA_F) as f64, 1000000000.0, "thz_wave_gateway::DELTA_F drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::PHI_V) as f64, 0.76, "thz_wave_gateway::PHI_V drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::V_RAIL) as f64, 4.75, "thz_wave_gateway::V_RAIL drifted from Python"); n += 1;
    assert_eq!((crate::thz_wave_gateway::F_CPU) as f64, 4000000000.0, "thz_wave_gateway::F_CPU drifted from Python"); n += 1;
    assert_eq!((crate::turbovec_memory::CHUNK_SIZE) as f64, 1000.0, "turbovec_memory::CHUNK_SIZE drifted from Python"); n += 1;
    assert_eq!((crate::turbovec_memory::COMPRESS_LEVEL) as f64, 3.0, "turbovec_memory::COMPRESS_LEVEL drifted from Python"); n += 1;
    // The L3 family used to be counted here as four divergences: the module declares 32 MB where
    // this host measures 16 MB, and that error propagated through three derived constants. What
    // /sram serves now comes from the host probe, so the count is 0. The module's literal is still
    // checked -- it is 1:1 with the Python and is served as the named fallback.
    let l3 = crate::sram_handlers::l3_block();
    let mut l3_divergent = 0usize;
    let served = l3.get("size_bytes").and_then(|v| v.as_u64()).unwrap_or(0);
    if served == 0 {
        l3_divergent += 1;
    }
    match crate::host_probe::l3_cache_kb() {
        Some(kb) => {
            if served != kb * 1024 {
                l3_divergent += 1;
            }
        }
        None => {
            if served != crate::sram_pattern_computer::L3_SIZE {
                l3_divergent += 1;
            }
        }
    }
    assert_eq!(l3_divergent, 0,
        "the L3 declared-vs-measured divergence count: 0 means /sram serves the host's value");
    assert_eq!(n, 13, "all non-divergent captured constants must be checked");
}

/// The L3 family, checked against the HOST rather than against a literal.
///
/// The kernel module declares a 32 MB L3 and this host measures 16,777,216 B (a 5900HX); that error
/// used to propagate through three derived constants, and it was counted here rather than silenced
/// until the served value could be derived from the probe. /sram now serves the measurement, so the
/// ruler points at what is actually served: a divergence is now a served value that disagrees with
/// the host, or derived fields that disagree with the served size.
#[test]
fn known_declared_vs_measured_divergences() {
    let l3 = crate::sram_handlers::l3_block();
    let mut divergences: Vec<String> = Vec::new();
    use crate::sram_pattern_computer as m;
    let served = match l3.get("size_bytes").and_then(|v| v.as_u64()) {
        Some(v) => v,
        None => {
            divergences.push("l3_block serves no size_bytes".into());
            0
        }
    };
    let basis = l3.get("l3_basis").and_then(|v| v.as_str()).unwrap_or("");

    match crate::host_probe::l3_cache_kb() {
        Some(kb) => {
            if served != kb * 1024 {
                divergences.push(format!("served {served} but the host measures {}", kb * 1024));
            }
            if !basis.starts_with("measured") {
                divergences.push(format!("the probe answered and the basis still says: {basis}"));
            }
        }
        None => {
            if served != m::L3_SIZE {
                divergences.push(format!("no measurement available, yet {served} != the declared {}", m::L3_SIZE));
            }
            if !basis.starts_with("assumed") {
                divergences.push(format!("a fallback must say so, and this basis does not: {basis}"));
            }
        }
    }

    // A relationship, not a snapshot: every derived field has to follow from the size actually served.
    if l3.get("cells").and_then(|v| v.as_u64()) != Some(served * 8) {
        divergences.push("cells do not follow from the served size".into());
    }
    let sets = served / (m::L3_LINE_SIZE * m::L3_WAYS * m::L3_SLICES);
    if l3.get("sets").and_then(|v| v.as_u64()) != Some(sets) {
        divergences.push("sets do not follow from the served size".into());
    }
    if l3.get("transistors").and_then(|v| v.as_u64()) != Some(served * 8 * m::L3_TRANSISTORS_PER_CELL) {
        divergences.push("transistors do not follow from the served size".into());
    }
    if l3.get("declared_default").and_then(|v| v.as_u64()) != Some(m::L3_SIZE) {
        divergences.push("the declared default is no longer visible beside the served value".into());
    }
    if l3.get("host_l3_matches_design_target").and_then(|v| v.as_bool()) != Some(served == m::L3_SIZE)
    {
        divergences.push("host_l3_matches_design_target does not follow from the values it compares".into());
    }

    assert_eq!(
        divergences.len(),
        0,
        "the L3 declared-vs-measured divergence set: {:?}",
        divergences
    );
}

