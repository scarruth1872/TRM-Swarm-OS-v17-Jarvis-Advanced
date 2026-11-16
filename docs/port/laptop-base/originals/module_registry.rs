//! TRM Swarm OS — Module Recognition Registry (no_std port).
//! The Catalyst failure fix: the platform compiled its kernel modules but did
//! not RECOGNIZE them. This module gives the booted kernel a queryable catalog
//! of what it knows and the honest status of each module.
//!
//! Counts and per-module status are NOT authored here. They are generated from
//! the live scan by build_module_registry.py into module_registry_generated.rs
//! and included below, so the kernel's self-knowledge cannot drift from what was
//! actually compiled.
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

// Derived scan results: TOTAL, REAL, PORTED, SKELETON, EXCLUDED, STATUSES.
// Regenerate with: python swarm_v2/core/build_module_registry.py
include!("module_registry_generated.rs");

/// Registry summary: (total, real, ported, skeleton, excluded, fabricated).
/// Derived from the generated scan — it cannot disagree with what compiled.
pub fn summary() -> (u64, u64, u64, u64, u64, u64) {
    (TOTAL, REAL, PORTED, SKELETON, EXCLUDED, FABRICATED)
}

/// The hand-curated catalog of kernel module names the platform recognizes.
///
/// This is a NAME catalog, not a claim about promotion level: a name listed here
/// may be REAL, PORTED or SKELETON. Per-module truth lives in STATUSES.
pub const RECOGNIZED_MODULES: &'static [&'static str] = &[
    "advanced_scene_fusion", "agent_mailbox", "agent_mailbox_bus",
    "agent_mesh", "agent_runtime", "allocator", "architectural_rl_agent",
    "artifact_pipeline", "attempt_sampler", "auto_changelog",
    "background_tasks", "base_agent", "benchmark_epistemology", "bridge",
    "chain_of_verification", "chaos_engine", "chaos_simulator",
    "cognitive_ingestion_engine", "cognitive_load_balancer", "cognitive_stack",
    "coherence_thresholds", "cold_start_recovery", "collaborative_reasoning",
    "comm_protocol", "continuum_genomics", "continuum_pbft",
    "coordination_protocol", "coordination_responder",
    "cosmos_microkernel_entanglement", "cosmos_moe_instella_shunter",
    "cosmos_sparse_moe_teams", "council", "cqs", "daemon_discovery",
    "daily_evolution_engine", "ddr_antibody", "ddr_self_training",
    "deterministic_forge", "dge", "digital_dna_loop", "digital_twin",
    "dram_pattern_computer", "epistemology_core", "evolutionary_sandbox",
    "expert_registry", "expert_team_harnesses", "federation", "feedback_loops",
    "font", "formal_verification_engine", "full_64_agent_orchestrator",
    "gan_design_synthesizer", "generative_architect", "global_memory",
    "gpu_cuda_microkernel_binder", "gpu_infinity_computer",
    "gpu_unlock_architecture", "hardware_voltage_microkernel_binder",
    "heartbeat_scheduler", "heuristic_consensus", "hf_brain", "higher_mind",
    "human_swarm_interaction", "hwprobe", "instella_router",
    "instella_sparse_agent_teams", "intent_buffer", "ipc", "jarvis_cognitive",
    "jarvis_quantum", "jarvis_substrate", "jarvis_symbiosis", "jarvis_vibe",
    "jarvis_vibethinker", "k3_logic", "kanban_board", "kernel_instructions",
    "kernel_lora_adapter", "llm_brain", "llm_orchestrator", "llm_router",
    "lobster_shell", "lora_response_modulator", "main", "mamba3_turbovec_ssm",
    "manus_engine", "mcp_bus", "mcp_tool_evolver", "memory",
    "memory_synchronizer", "meta_orchestrator", "microkernel_spawner",
    "modmap_all", "moltbook", "monad", "monitor_daemon", "mpu",
    "multimodal_fusion", "multimodal_ingestion",
    "multimodal_predictive_analytics", "mutation_engine", "nand_fs",
    "native_ops_bridge", "neural_wall", "neuro_continuum", "openclaw_gateway",
    "optimization_engine", "pbft_consensus", "persistent_mmap_allocator",
    "persona_matrix", "phoenix_protocol_engine", "platform", "port_manager",
    "predictive_watchdog", "prism_fqm", "proactive_loop", "qisa_optimizer",
    "quantum_debugger", "quic_protocol", "radix_trie_matcher",
    "reconnaissance_daemon", "redis_mock", "remediation_engine",
    "resonance_engine", "resource_arbiter", "rust_kernel_binder",
    "secrets_vault", "security_scanner", "self_fuzzing_engine", "self_healing",
    "self_healing_infra", "self_healing_orchestrator", "self_modification_api",
    "semantic_cache", "semantic_index", "semantic_knowledge_graph", "sentinel",
    "ses", "simd_accelerator", "skill_loader", "skill_matrix_sync",
    "skill_validator", "small_diffusion_engine", "sram_pattern_computer",
    "star_matrix", "star_matrix_lattice", "sub_atomic_kernel", "swarm_engine",
    "swarm_orchestrator", "swarm_performance_insights",
    "symbology_etymology_monad", "task_arbiter", "telemetry",
    "telemetry_fabric", "thz_bonsai_gateway", "thz_wave_gateway", "time",
    "tool_forge", "trm7m", "trm_7m_full", "trm_7m_optimized", "trm_brain",
    "trm_integration", "trm_orchestrator", "turbovec", "turbovec_memory",
    "ultrawork_loop", "voltage_node_engine", "worktree_manager",
    "zero_human_test_gen",
];

/// Modules recognized by the booted command terminal (22 commands).
pub const COMMANDS: &'static [&'static str] = &[
    "agents", "bench", "bridge", "clear", "council", "epistemology",
    "help", "hw", "jarvis", "k3", "mesh", "moe", "monad", "nand", "quic",
    "radar", "setup", "status", "trm", "twin", "verify-epistemology", "vep",
];

/// Names whose scanned status is REAL — what the kernel can actually execute.
fn status_names(want: &str) -> Vec<&'static str> {
    let mut v = Vec::new();
    for (n, s) in STATUSES.iter() {
        if *s == want {
            v.push(*n);
        }
    }
    v
}

/// Seed names in the curated catalog that the generated scan no longer contains.
/// Non-empty means the catalog has drifted from what actually compiles.
pub fn unrecognized_seed() -> Vec<&'static str> {
    let mut v = Vec::new();
    for m in RECOGNIZED_MODULES.iter() {
        if !STATUSES.iter().any(|(n, _)| n == m) {
            v.push(*m);
        }
    }
    v
}

/// Render the registry for the LCARS terminal.
pub fn render_registry() -> Vec<String> {
    let mut out = Vec::new();
    let (total, real, ported, skeleton, excluded, fabricated) = summary();
    out.push(format!(
        "REGISTRY: {} modules compiled | {} REAL | {} PORTED | {} SKELETON | {} EXCLUDED | {} FABRICATED",
        total, real, ported, skeleton, excluded, fabricated
    ));
    out.push(format!(
        "THE CATALYST: the One recognizes {} verified intelligence modules",
        real_count()
    ));
    out.push(format!("EXECUTABLE COMMANDS: {}", COMMANDS.len()));
    let mut line = String::from("  REAL: ");
    let real_names = status_names("REAL");
    for (i, m) in real_names.iter().enumerate() {
        if i > 0 {
            line.push(' ');
        }
        line.push_str(m);
        if line.len() > 70 && i < real_names.len() - 1 {
            out.push(line.clone());
            line = String::from("        ");
        }
    }
    out.push(line);
    let drift = unrecognized_seed();
    if !drift.is_empty() {
        out.push(format!("  CATALOG DRIFT: {} seed names not in scan", drift.len()));
    }
    out
}

/// Query: does the platform recognize a module by name?
/// Returns the module's TRUE scanned status, never an assumed "REAL".
pub fn recognize(name: &str) -> Option<&'static str> {
    for (n, s) in STATUSES.iter() {
        if *n == name {
            return Some(*s);
        }
    }
    if COMMANDS.contains(&name) {
        Some("COMMAND")
    } else {
        None
    }
}

/// Count of modules that publish numbers they did not measure.
pub fn fabricated_count() -> u64 {
    FABRICATED
}

/// Query: which kernel tree a recognized module was compiled from
/// ("uefi" = no_std boot kernel, "dll" = swarm_rust_kernel.dll).
pub fn origin_of(name: &str) -> Option<&'static str> {
    ORIGIN.iter().find(|(n, _)| *n == name).map(|(_, t)| *t)
}

/// Count of REAL modules (for the council's self-knowledge).
pub fn real_count() -> u64 {
    REAL
}
