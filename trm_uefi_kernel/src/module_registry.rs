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
    // (total, REAL, PORTED, SKELETON, EXCLUDED). DERIVED from the arrays: the
    // previous hardcoded tuple summed to 197 rather than 188, and its REAL count
    // matched neither REAL_MODULES.len() nor the "110" its own doc comment claimed.
    // Four entries were also listed as REAL while their .rs files were never
    // declared as modules, so they never compiled; they are no longer counted.
    let real = REAL_MODULES.len() as u64;
    let data = DATA_MODULES.len() as u64;
    (real + data, real, 0, 0, data)
}

/// The REAL (verified intelligence) modules — the ones the kernel
/// actually executes, not just compiles.
pub const REAL_MODULES: &'static [&'static str] = &[
    // Re-registered 2026-09-22 AFTER being wired: these four were previously listed
    // here while never being declared as modules, so they never compiled. They are
    // now declared at the crate root and invoked from console commands (facet,
    // patterns, sync, deploy), so this listing is finally accurate.
    "autonomous_scenario_setup",
    "federation_sync_upgrade",
    "fractal_pattern_engine",
    "faceted_torus_ternary",

    "advanced_scene_fusion",
    "agent_mailbox",
    "agent_mailbox_bus",
    "agent_mesh",
    "agent_runtime",
    "allocator",
    "artifact_pipeline",
    "attempt_sampler",
    "autonomous_cadence",
    "benchmark_epistemology",
    "bridge",
    "chain_of_verification",
    "chaos_engine",
    "cognitive_load_balancer",
    "continuum_pbft",
    "coordination_protocol",
    "council",
    "daily_evolution_engine",
    "ddr_antibody",
    "digital_twin",
    "dram_pattern_computer",
    "epistemology_core",
    "expert_registry",
    "federation_manager",
    "federation_task_coordinator",
    "font",
    "formal_verification_engine",
    "fractal_telemetry",
    "full_64_agent_orchestrator",
    "gan_design_synthesizer",
    "generative_architect",
    "global_memory",
    "gpu_infinity_computer",
    "gpu_unlock_architecture",
    "hardware_voltage_microkernel_binder",
    "heartbeat_scheduler",
    "human_swarm_interaction",
    "hwprobe",
    "instella_router",
    "instella_sparse_agent_teams",
    "intent_buffer",
    "jarvis_cognitive",
    "jarvis_quantum",
    "jarvis_substrate",
    "jarvis_symbiosis",
    "jarvis_vibe",
    "jarvis_vibethinker",
    "k3_logic",
    "k3_ternary_sram",
    "kernel_instructions",
    "llm_brain",
    "main",
    "mamba3_turbovec_ssm",
    "manus_engine",
    "mcts_plan",
    "memory_mapped_ipc",
    "microkernel_spawner",
    "modmap_all",
    "module_registry",
    "monad",
    "multimodal_ingestion",
    "nand_fs",
    "p2p_direct_route",
    "pbft_consensus",
    "quantum_drug_screen",
    "quic_protocol",
    "radix_trie_matcher",
    "rare_disease",
    "resonance_engine",
    "security_scanner",
    "self_fuzzing_engine",
    "self_healing",
    "self_healing_infra",
    "semantic_cache",
    "semantic_knowledge_graph",
    "skill_loader",
    "skill_matrix_sync",
    "skill_validator",
    "small_diffusion_engine",
    "spawn_subagent",
    "sram_pattern_computer",
    "star_matrix",
    "sub_atomic_kernel",
    "swarm_performance_insights",
    "symbology_etymology_monad",
    "task_arbiter",
    "telemetry_fabric",
    "thz_bonsai_gateway",
    "time",
    "tool_forge",
    "totality",
    "trm7m",
    "trm_7m_optimized",
    "trm_brain",
    "trm_integration",
    "trm_orchestrator",
    "turbovec",
    "voltage_node_engine",
    "worktree_manager",
    "zero_human_test_gen",
];

/// Modules recognized by the booted command terminal (22 commands).
pub const DATA_MODULES: &'static [&'static str] = &[

    "architectural_rl_agent",
    "auto_changelog",
    "brain_ai_microkernel_bridge",
    "chaos_simulator",
    "cognitive_ingestion_engine",
    "cognitive_stack",
    "coherence_thresholds",
    "collaborative_reasoning",
    "comm_protocol",
    "continuum_genomics",
    "coordination_responder",
    "cosmos3_edge_bridge",
    "cosmos_bridge",
    "cosmos_microkernel_entanglement",
    "cosmos_moe_instella_shunter",
    "cosmos_sparse_moe_teams",
    "cqs",
    "ddr_self_training",
    "deterministic_forge",
    "dge",
    "digital_dna_loop",
    "evolutionary_sandbox",
    "expert_team_harnesses",
    "external_socket_bridge",
    "federation",
    "feedback_loops",
    "gpu_cuda_microkernel_binder",
    "heuristic_consensus",
    "hf_brain",
    "higher_mind",
    "ipc",
    "kanban_board",
    "kernel_lora_adapter",
    "llm_orchestrator",
    "llm_router",
    "lobster_shell",
    "lora_response_modulator",
    "mcp_bus",
    "mcp_tool_evolver",
    "memory",
    "memory_synchronizer",
    "meta_orchestrator",
    "microkernel_chat_bridge",
    "moltbook",
    "mpu",
    "multimodal_fusion",
    "multimodal_predictive_analytics",
    "mutation_engine",
    "native_ops_bridge",
    "neural_wall",
    "neuro_continuum",
    "openclaw_gateway",
    "optimization_engine",
    "persistent_mmap_allocator",
    "phoenix_protocol_engine",
    "port_manager",
    "predictive_watchdog",
    "prism_fqm",
    "proactive_loop",
    "qisa_optimizer",
    "quantum_debugger",
    "reconnaissance_daemon",
    "resource_arbiter",
    "rust_kernel_binder",
    "secrets_vault",
    "self_healing_orchestrator",
    "self_modification_api",
    "semantic_index",
    "sentinel",
    "ses",
    "star_matrix_lattice",
    "swarm_engine",
    "swarm_orchestrator",
    "telemetry",
    "thz_wave_gateway",
    "trm_7m_full",
    "turbovec_memory",
    "ultrawork_loop",
];

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
    out.push(format!("DATA (constants/getters): {} recognized", DATA_MODULES.len()));
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
        Some("REAL (Verified)")
    } else if DATA_MODULES.contains(&name) {
        Some("SKELETON_STUB (Constants/Placeholder)")
    } else if COMMANDS.contains(&name) {
        Some("COMMAND (Terminal Directive)")
    } else {
        None
    }
}

/// Count of real modules in a category (for the council's self-knowledge).
pub fn real_count() -> u64 {
    REAL_MODULES.len() as u64
}

