//! TRM Swarm OS — Module Recognition Registry (no_std port).
//! The Catalyst failure fix: the platform compiled 179 modules but did not
//! RECOGNIZE them. This module gives the booted kernel a queryable catalog
//! of what it knows and the honest status of each module.
//!
//! Status legend (1:1 with build_module_registry.py):
//!   REAL     — hand-written intelligence, verified
//!   PORTED   — mechanical port, compiles
//!   SKELETON — constants/structs only, body marked MANUAL
//!   EXCLUDED — host-OS companion (no-HTTP/no-torch/no-LLM rules)

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Registry summary: (total, real, ported, skeleton, excluded).
pub fn summary() -> (u64, u64, u64, u64, u64) {
    // CODE TRUTH (re-audit v19.4.0 post-fanout): 97 substantive + 6 infra.
    // Fan-out batches A/B/C + probe added ~19 REAL (mailbox, kgraph, CLB,
    // chaos, voltage_node, skill_validator, dram, symbology, etc.).
    (188, 98, 0, 91, 9)
}

/// The 110 REAL (verified intelligence) modules — the ones the kernel
/// actually executes, not just compiles.
pub const REAL_MODULES: &'static [&'static str] = &[
    "advanced_scene_fusion",
    "agent_mailbox",
    "agent_mailbox_bus",
    "agent_runtime",
    "allocator",
    "attempt_sampler",
    "autonomous_cadence",
    "autonomous_scenario_setup",
    "benchmark_epistemology",
    "bridge",
    "chain_of_verification",
    "chaos_engine",
    "cognitive_load_balancer",
    "coordination_protocol",
    "council",
    "daily_evolution_engine",
    "ddr_antibody",
    "digital_twin",
    "dram_pattern_computer",
    "epistemology_core",
    "faceted_torus_ternary",
    "federation_manager",
    "federation_sync_upgrade",
    "federation_task_coordinator",
    "feedback_loops",
    "font",
    "formal_verification_engine",
    "fractal_pattern_engine",
    "fractal_telemetry",
    "full_64_agent_orchestrator",
    "gan_design_synthesizer",
    "generative_architect",
    "global_memory",
    "gpu_cuda_microkernel_binder",
    "gpu_infinity_computer",
    "gpu_unlock_architecture",
    "hardware_voltage_microkernel_binder",
    "heartbeat_scheduler",
    "heuristic_consensus",
    "higher_mind",
    "human_swarm_interaction",
    "hwprobe",
    "instella_router",
    "instella_sparse_agent_teams",
    "intent_buffer",
    "ipc",
    "jarvis_cognitive",
    "jarvis_quantum",
    "jarvis_substrate",
    "jarvis_symbiosis",
    "jarvis_vibe",
    "jarvis_vibethinker",
    "k3_logic",
    "llm_brain",
    "main",
    "mamba3_turbovec_ssm",
    "manus_engine",
    "modmap_all",
    "module_registry",
    "monad",
    "multimodal_ingestion",
    "nand_fs",
    "native_mailbox_bridge",
    "pbft_consensus",
    "predictive_watchdog",
    "quic_protocol",
    "resonance_engine",
    "security_scanner",
    "self_healing_infra",
    "semantic_cache",
    "semantic_index",
    "semantic_knowledge_graph",
    "sentinel",
    "simd_accelerator",
    "skill_loader",
    "skill_matrix_sync",
    "skill_validator",
    "small_diffusion_engine",
    "sram_pattern_computer",
    "star_matrix",
    "star_matrix_lattice",
    "sub_atomic_kernel",
    "swarm_performance_insights",
    "symbology_etymology_monad",
    "task_arbiter",
    "telemetry_fabric",
    "thz_bonsai_gateway",
    "thz_wave_gateway",
    "time",
    "tool_forge",
    "totality",
    "trm7m",
    "trm_7m_optimized",
    "trm_brain",
    "turbovec",
    "turbovec_memory",
    "voltage_node_engine",
    "worktree_manager",
];

/// Modules recognized by the booted command terminal (22 commands).
pub const COMMANDS: &'static [&'static str] = &[
    "agents", "bench", "bridge", "clear", "council", "epistemology",
    "help", "hw", "jarvis", "k3", "mesh", "moe", "monad", "nand", "quic",
    "radar", "setup", "status", "trm", "twin", "verify-epistemology", "vep",
];

/// Render the registry for the LCARS terminal.
pub fn render_registry() -> Vec<String> {
    let mut out = Vec::new();
    let (total, real, ported, skeleton, excluded) = summary();
    out.push(format!(
        "REGISTRY: {} modules compiled | {} REAL | {} PORTED | {} SKELETON | {} EXCLUDED",
        total, real, ported, skeleton, excluded
    ));
    out.push(format!("THE CATALYST: the One recognizes {} verified intelligence modules", real));
    out.push(format!("EXECUTABLE COMMANDS: {}", COMMANDS.len()));
    let mut line = String::from("  REAL: ");
    for (i, m) in REAL_MODULES.iter().enumerate() {
        if i > 0 {
            line.push(' ');
        }
        line.push_str(m);
        if line.len() > 70 && i < REAL_MODULES.len() - 1 {
            out.push(line.clone());
            line = String::from("        ");
        }
    }
    out.push(line);
    out
}

/// Query: does the platform recognize a module by name?
pub fn recognize(name: &str) -> Option<&'static str> {
    if REAL_MODULES.contains(&name) {
        Some("REAL")
    } else if COMMANDS.contains(&name) {
        Some("COMMAND")
    } else {
        None
    }
}

/// Count of real modules in a category (for the council's self-knowledge).
pub fn real_count() -> u64 {
    REAL_MODULES.len() as u64
}
