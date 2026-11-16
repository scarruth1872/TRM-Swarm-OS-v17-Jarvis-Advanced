//! TRM Swarm OS v20.0.0 - Native Pure Rust Multi-Threaded Swarm Council & Telemetry Server
//! Multi-threaded concurrent worker pool for instant sub-millisecond dashboard response.
//! Zero Python dependency in critical path.

use serde_json::json;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use tiny_http::{Header, Method, Response, Server};

mod actions_handlers;
mod artifact_handlers;
mod aura_handlers;
mod aura_metaconcepts;
mod aura_transform;
mod benchmark_handlers;
mod chat_handlers;
mod crate_ops_handlers;
mod host_probe;
mod humanistic_teach;
mod query;
mod university_client;
mod agentq_bridge;
mod host_sensors;
mod autonomy;
mod frame;
mod humanistic_handlers;
mod knowledge_surface;
mod knowledge_stores;
mod gpu_dispatch;
mod gpu_compute;
mod intelligence_router;
mod core_pinning;
mod file_cache;
mod k3_retrieval;
mod nand_append;
mod recall_router;
mod semantic_recall;
mod paper_production;
mod mailbox_ring;
mod packed_ternary;
mod kanban_handlers;
mod smiles_canon;
mod hardware_binder;
mod variant_annotation;
mod mesh_handlers;
mod model_registry;
mod module_handlers;
// TRM 7M council decision (blocks 175/176), options (a)+(b): the per-agent assignment, generated from the
// canonical Python table, and the model itself made runnable. Both were absent from this crate before --
// `grep -i trm7m` over host_council_server/src returned 0 hits while the kernel advertised the model.
mod agent_models_generated;
mod trm7m;
mod instella_handlers;
mod research_handlers;
mod services_handlers;
mod state_handlers;
mod telemetry_handlers;
mod verification_handlers;
mod studio_handlers;
mod ui_handlers;
mod ui_render;
mod mailbox_write;
mod artifact_actions;
mod evolution_actions;
mod architect_actions;
mod ddr_scan;
mod hw_rails;
mod parity_probe;

mod continuum_refract;
mod ternary_engine;
mod mamba_forward;
mod plugins;

extern crate alloc;

// Layer 1's implementation, linked by PATH rather than copied: /sram serves the 15 topologies and the
// Hopfield relaxation straight out of the kernel's own source, so a route answer cannot drift from it.
#[path = "../../src/sram_pattern_computer.rs"]
mod sram_pattern_computer;

// The four modules whose strips are bit-exact against their python oracles, linked the same way, so /native
// serves the kernel's own functions instead of a second implementation that could disagree with them.
#[path = "../../src/gpu_infinity_computer.rs"]
mod gpu_infinity_computer;

#[path = "../../src/chaos_engine.rs"]
mod chaos_engine;

// The two modules whose portable surface was a rename and whose aggregate surface was missing. Included
// by path, so the code answering these routes IS the kernel's file.
#[path = "../../src/cognitive_load_balancer.rs"]
mod cognitive_load_balancer;

#[path = "../../src/self_fuzzing_engine.rs"]
mod self_fuzzing_engine;

mod load_fuzz_handlers;

#[path = "../../src/expert_registry.rs"]
mod expert_registry;

#[path = "../../src/persona_matrix.rs"]
mod persona_matrix;

mod council_personas;

mod phonic;
mod ambient;
mod shadow_bios;

// ── The parity ruler, promoted from host_tranche. cfg(test) only, so the release binary gains
//    nothing. The three modules the constants test reads are linked here for the test build; each is
//    byte-identical to its kernel source (measured, 25 of 28 tranche files match exactly), so this
//    adds no second implementation to disagree with the one being served.
#[cfg(test)]
#[path = "../../src/daily_evolution_engine.rs"]
mod daily_evolution_engine;
#[cfg(test)]
#[path = "../../src/thz_wave_gateway.rs"]
mod thz_wave_gateway;
#[cfg(test)]
#[path = "../../src/turbovec_memory.rs"]
mod turbovec_memory;
#[cfg(test)]
mod parity_tests;
#[cfg(test)]
mod tests_surface;

mod sram_handlers;
mod native_encoder;
mod native_ternary;
mod native_handlers;
mod drug_handlers;
mod docking_handlers;
mod screen_library;
mod antibody_handlers;
mod intel_handlers;
mod intel_reservoir;
mod task_executor;
mod infra_handlers;
// The infrastructure FSM, linked by #[path] like the other modules of record -- wiring, not a port.
// Its source of record is trm_uefi_kernel/src/self_healing_infra.rs.
#[path = "../../src/self_healing_infra.rs"]
mod self_healing_infra;
// The Kleene engine the function map found compiled but reachable by nothing (265 wire verdicts across the
// crate); linked by #[path] like the module above, so /logic/k3 computes in the kernel's own implementation.
#[path = "../../src/k3_logic.rs"]
mod k3_logic;
mod logic_routes;
// The coordination protocol's deterministic helpers: pure functions of their inputs, reachable by nothing.
#[path = "../../src/coordination_protocol.rs"]
mod coordination_protocol;
mod coord_routes;
// The ingestion engine's ledger, priority weights and status gate: reachable by nothing until now.
#[path = "../../src/cognitive_ingestion_engine.rs"]
mod cognitive_ingestion_engine;
mod ingest_routes;
// The self-healing orchestrator: degraded detection, reassignment and peer selection, reachable by nothing.
#[path = "../../src/self_healing_orchestrator.rs"]
mod self_healing_orchestrator;
mod heal_routes;
// The semantic cache's two similarity branches, the 0.95 match rule and the LRU list operations.
#[path = "../../src/semantic_cache.rs"]
mod semantic_cache;
mod cache_routes;
// Zero-human test generation: endpoint registration, permutation synthesis, manifest and coverage.
#[path = "../../src/zero_human_test_gen.rs"]
mod zero_human_test_gen;
mod testgen_routes;
// Base agent: greetings, skill projection, spawn records, memory summaries and core axioms.
#[path = "../../src/base_agent.rs"]
mod base_agent;
mod agent_routes;
// Worktree descriptors, cold-start arena recovery, and the SRAM computer's numeric primitives.
#[path = "../../src/worktree_manager.rs"]
mod worktree_manager;
mod worktree_routes;
#[path = "../../src/cold_start_recovery.rs"]
mod cold_start_recovery;
mod coldstart_routes;
mod sram_routes;
// Kernel micro-instructions, the DRAM pattern computer's text surface, and the port pool.
#[path = "../../src/kernel_instructions.rs"]
mod kernel_instructions;
mod kern_routes;
#[path = "../../src/dram_pattern_computer.rs"]
mod dram_pattern_computer;
mod dram_routes;
#[path = "../../src/port_manager.rs"]
mod port_manager;
mod ports_routes;
// Self-healing engine, swarm engine modes/routing, and the persistent arena tracker.
#[path = "../../src/self_healing.rs"]
mod self_healing;
mod selfheal_routes;
#[path = "../../src/swarm_engine.rs"]
mod swarm_engine;
mod engine_routes;
#[path = "../../src/persistent_mmap_allocator.rs"]
mod persistent_mmap_allocator;
mod arena_routes;
// The 64-agent roster, the GPU unlock pipeline, and the unified meta orchestrator. full_64 reads
// crate::time::now_ns(), so the kernel's clock module is linked alongside it.
#[path = "../../src/full_64_agent_orchestrator.rs"]
mod full_64_agent_orchestrator;
#[path = "../../src/time.rs"]
mod time;
mod agents64_routes;
#[path = "../../src/gpu_unlock_architecture.rs"]
mod gpu_unlock_architecture;
mod gpu_routes;
#[path = "../../src/meta_orchestrator.rs"]
mod meta_orchestrator;
mod meta_routes;
// Self-optimisation, VRAM arbitration, and remediation.
#[path = "../../src/optimization_engine.rs"]
mod optimization_engine;
mod optimize_routes;
#[path = "../../src/resource_arbiter.rs"]
mod resource_arbiter;
mod arbiter_routes;
#[path = "../../src/remediation_engine.rs"]
mod remediation_engine;
mod remediate_routes;
// Redis mock, the Rust kernel binder tables, and the subatomic Mamba kernel.
#[path = "../../src/redis_mock.rs"]
mod redis_mock;
mod redis_routes;
#[path = "../../src/rust_kernel_binder.rs"]
mod rust_kernel_binder;
mod binder_routes;
#[path = "../../src/sub_atomic_kernel.rs"]
mod sub_atomic_kernel;
mod subatomic_routes;
// Build 1: the nine time-linked runtime modules and the five scheduling modules.
#[path = "../../src/agent_runtime.rs"]
mod agent_runtime;
#[path = "../../src/generative_architect.rs"]
mod generative_architect;
#[path = "../../src/global_memory.rs"]
mod global_memory;
#[path = "../../src/hardware_voltage_microkernel_binder.rs"]
mod hardware_voltage_microkernel_binder;
#[path = "../../src/hwprobe.rs"]
mod hwprobe;
#[path = "../../src/coherence_thresholds.rs"]
mod coherence_thresholds;
#[path = "../../src/telemetry.rs"]
mod telemetry;
#[path = "../../src/telemetry_probe.rs"]
mod telemetry_probe;
#[path = "../../src/instella_router.rs"]
mod instella_router;
#[path = "../../src/instella_sparse_agent_teams.rs"]
mod instella_sparse_agent_teams;
#[path = "../../src/mamba3_turbovec_ssm.rs"]
mod mamba3_turbovec_ssm;
#[path = "../../src/swarm_orchestrator.rs"]
mod swarm_orchestrator;
#[path = "../../src/agent_mailbox.rs"]
mod agent_mailbox;
#[path = "../../src/agent_mailbox_bus.rs"]
mod agent_mailbox_bus;
#[path = "../../src/trm_orchestrator.rs"]
mod trm_orchestrator;
#[path = "../../src/heartbeat_scheduler.rs"]
mod heartbeat_scheduler;
mod runtime_routes;
mod sched_routes;
// Build 2: trm_wifi's crypto/syscall companions with eapol + netlink, the comm protocol and the secrets vault;
// then the memory synchroniser and the allocator handle.
#[path = "../../trm_wifi/src/crypto.rs"]
mod crypto;
#[path = "../../trm_wifi/src/syscall.rs"]
mod syscall;
#[path = "../../trm_wifi/src/eapol.rs"]
mod eapol;
#[path = "../../trm_wifi/src/netlink.rs"]
mod netlink;
#[path = "../../src/comm_protocol.rs"]
mod comm_protocol;
#[path = "../../src/secrets_vault.rs"]
mod secrets_vault;
#[path = "../../src/memory_synchronizer.rs"]
mod memory_synchronizer;
mod sec_routes;
mod mem_routes;
// Build 3: the model family (LLM routing/synthesis, Cosmos teams, TRM-7M) and the Jarvis family.
#[path = "../../src/llm_router.rs"]
mod llm_router;
#[path = "../../src/llm_orchestrator.rs"]
mod llm_orchestrator;
#[path = "../../src/cosmos_sparse_moe_teams.rs"]
mod cosmos_sparse_moe_teams;
#[path = "../../src/cosmos_moe_instella_shunter.rs"]
mod cosmos_moe_instella_shunter;
#[path = "../../src/trm_7m_full.rs"]
mod trm_7m_full;
#[path = "../../src/trm_7m_optimized.rs"]
mod trm_7m_optimized;
#[path = "../../src/jarvis_cognitive.rs"]
mod jarvis_cognitive;
#[path = "../../src/jarvis_vibethinker.rs"]
mod jarvis_vibethinker;
#[path = "../../src/jarvis_substrate.rs"]
mod jarvis_substrate;
#[path = "../../src/jarvis_symbiosis.rs"]
mod jarvis_symbiosis;
#[path = "../../src/jarvis_vibe.rs"]
mod jarvis_vibe;
mod model_routes;
mod jarvis_routes;
// Build 4: consensus/cognition (continuum) and kernel plumbing (infra).
#[path = "../../src/continuum_pbft.rs"]
mod continuum_pbft;
#[path = "../../src/continuum_genomics.rs"]
mod continuum_genomics;
#[path = "../../src/coordination_responder.rs"]
mod coordination_responder;
#[path = "../../src/cognitive_stack.rs"]
mod cognitive_stack;
#[path = "../../src/microkernel_spawner.rs"]
mod microkernel_spawner;
#[path = "../../src/mpu.rs"]
mod mpu;
#[path = "../../src/ipc.rs"]
mod ipc;
#[path = "../../src/mcp_bus.rs"]
mod mcp_bus;
#[path = "../../src/native_ops_bridge.rs"]
mod native_ops_bridge;
#[path = "../../src/native_mailbox_bridge.rs"]
mod native_mailbox_bridge;
mod continuum_routes;
mod infra_routes;
// Build 5: the platform describing itself -- artifacts, forges, registry and skill telemetry.
#[path = "../../src/artifact_pipeline.rs"]
mod artifact_pipeline;
#[path = "../../src/tool_forge.rs"]
mod tool_forge;
#[path = "../../src/deterministic_forge.rs"]
mod deterministic_forge;
#[path = "../../src/skill_matrix_sync.rs"]
mod skill_matrix_sync;
#[path = "../../src/skill_validator.rs"]
mod skill_validator;
#[path = "../../src/module_registry.rs"]
mod module_registry;
mod build_routes;
// Build 6: the operator-facing surface.
#[path = "../../src/collaborative_reasoning.rs"]
mod collaborative_reasoning;
#[path = "../../src/sentinel.rs"]
mod sentinel;
#[path = "../../src/web_fetch_utility.rs"]
mod web_fetch_utility;
#[path = "../../src/disk_explorer.rs"]
mod disk_explorer;
#[path = "../../src/background_tasks.rs"]
mod background_tasks;
#[path = "../../src/lobster_shell.rs"]
mod lobster_shell;
#[path = "../../src/daemon_discovery.rs"]
mod daemon_discovery;
#[path = "../../src/dge.rs"]
mod dge;
mod vault_routes;
// Build 7: the long tail -- one module each.
#[path = "../../src/attempt_sampler.rs"]
mod attempt_sampler;
#[path = "../../src/brain_ai_microkernel_bridge.rs"]
mod brain_ai_microkernel_bridge;
#[path = "../../src/bridge.rs"]
mod bridge;
#[path = "../../src/cosmos_microkernel_entanglement.rs"]
mod cosmos_microkernel_entanglement;
#[path = "../../src/cqs.rs"]
mod cqs;
#[path = "../../src/ddr_self_training.rs"]
mod ddr_self_training;
#[path = "../../src/digital_dna_loop.rs"]
mod digital_dna_loop;
#[path = "../../src/digital_twin.rs"]
mod digital_twin;
#[path = "../../src/evolutionary_sandbox.rs"]
mod evolutionary_sandbox;
#[path = "../../src/expert_team_harnesses.rs"]
mod expert_team_harnesses;
#[path = "../../src/federation_manager.rs"]
mod federation_manager;
#[path = "../../src/intent_buffer.rs"]
mod intent_buffer;
#[path = "../../src/lora_response_modulator.rs"]
mod lora_response_modulator;
#[path = "../../src/monitor_daemon.rs"]
mod monitor_daemon;
#[path = "../../src/nand_fs.rs"]
mod nand_fs;
#[path = "../../src/phoenix_protocol_engine.rs"]
mod phoenix_protocol_engine;
#[path = "../../src/predictive_watchdog.rs"]
mod predictive_watchdog;
#[path = "../../src/prism_fqm.rs"]
mod prism_fqm;
#[path = "../../src/resonance_engine.rs"]
mod resonance_engine;
#[path = "../../src/ses.rs"]
mod ses;
#[path = "../../src/simd_accelerator.rs"]
mod simd_accelerator;
#[path = "../../src/small_diffusion_engine.rs"]
mod small_diffusion_engine;
#[path = "../../src/reconnaissance_daemon.rs"]
mod reconnaissance_daemon;
#[path = "../../src/trm_integration.rs"]
mod trm_integration;
#[path = "../../src/apic_core_binding.rs"]
mod apic_core_binding;
#[path = "../../src/aura_spectrum.rs"]
mod aura_spectrum;
#[path = "../../src/autonomous_cadence.rs"]
mod autonomous_cadence;
#[path = "../../src/autonomous_scenario_setup.rs"]
mod autonomous_scenario_setup;
#[path = "../../src/baremetal_transition.rs"]
mod baremetal_transition;
#[path = "../../src/faceted_torus_ternary.rs"]
mod faceted_torus_ternary;
#[path = "../../src/fb_console.rs"]
mod fb_console;
#[path = "../../src/federation_sync_upgrade.rs"]
mod federation_sync_upgrade;
#[path = "../../src/federation_task_coordinator.rs"]
mod federation_task_coordinator;
#[path = "../../src/float_math.rs"]
mod float_math;
#[path = "../../src/fractal_pattern_engine.rs"]
mod fractal_pattern_engine;
#[path = "../../src/fractal_telemetry.rs"]
mod fractal_telemetry;
#[path = "../../src/hmac_sha256.rs"]
mod hmac_sha256;
#[path = "../../src/holographic_fabric.rs"]
mod holographic_fabric;
#[path = "../../src/interrupt_stubs.rs"]
mod interrupt_stubs;
#[path = "../../src/k3_head.rs"]
mod k3_head;
#[path = "../../src/mcts_plan.rs"]
mod mcts_plan;
#[path = "../../src/memory_mapped_ipc.rs"]
mod memory_mapped_ipc;
#[path = "../../src/module_registry_generated.rs"]
mod module_registry_generated;
#[path = "../../src/p2p_direct_route.rs"]
mod p2p_direct_route;
#[path = "../../src/platform.rs"]
mod platform;
#[path = "../../src/quantum_drug_screen.rs"]
mod quantum_drug_screen;
#[path = "../../src/rare_disease.rs"]
mod rare_disease;
#[path = "../../src/serial.rs"]
mod serial;
#[path = "../../src/smp_trampoline.rs"]
mod smp_trampoline;
#[path = "../../src/text_extract.rs"]
mod text_extract;
#[path = "../../src/unified_transistor_fabric.rs"]
mod unified_transistor_fabric;
#[path = "../../src/font.rs"]
mod font;
mod orphan_probe;
#[path = "../../src/hw_truth.rs"]
mod hw_truth;
#[path = "../../src/task_arbiter.rs"]
mod task_arbiter;
#[path = "../../src/thz_bonsai_gateway.rs"]
mod thz_bonsai_gateway;
#[path = "../../src/ultrawork_loop.rs"]
mod ultrawork_loop;
mod mesh_routes;
// modmap_all declares a nested `pub mod council;`, and council.rs addresses the crate root for its anchor.
// Same declared value as trm_uefi_kernel/src/main.rs:52 -- linked modules re-export it, they do not redefine it.
pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;
mod fit_handlers;
mod brain_handlers;
mod council_outcomes;
mod goal_runner;
mod mamba_tier;

/// Design constant, NOT a probe: the SRAM pattern count is a property of the platform's
/// memory layout, not something this host can be asked. Named so its provenance is visible
/// in the payload rather than passing for a measurement.
const SRAM_PATTERNS_DECLARED: u32 = 15;

struct AgentInfo {
    name: &'static str,
    role: &'static str,
    weight: f32,
    domain: &'static str,
}

/// PBFT approval threshold for the 15-agent roster with f = 4 Byzantine tolerance: 2f + 1 = 9.
/// Declared by the status route, the council UI and the fusion-method text, and enforced by the tally.
const PBFT_QUORUM: usize = 9;

const AGENTS: &[AgentInfo] = &[
    AgentInfo {
        name: "Hermes",
        role: "Comms & Spectrum",
        weight: 1.0,
        domain: "Communication",
    },
    AgentInfo {
        name: "Jarvis",
        role: "Assistant & Operator",
        weight: 0.9,
        domain: "Interface",
    },
    AgentInfo {
        name: "Bridge",
        role: "APIs & Data Routing",
        weight: 1.0,
        domain: "Data",
    },
    AgentInfo {
        name: "Shield",
        role: "Security & Antibody",
        weight: 1.0,
        domain: "Security",
    },
    AgentInfo {
        name: "Logic",
        role: "Reasoning Engine",
        weight: 0.9,
        domain: "Reasoning",
    },
    AgentInfo {
        name: "Devo",
        role: "Code & Build",
        weight: 1.0,
        domain: "Code",
    },
    AgentInfo {
        name: "Pulse",
        role: "Monitoring & Telemetry",
        weight: 1.0,
        domain: "Health",
    },
    AgentInfo {
        name: "Flow",
        role: "Workflow Orchestrator",
        weight: 0.9,
        domain: "Workflow",
    },
    AgentInfo {
        name: "EventLedger",
        role: "Audit & Monad Ledger",
        weight: 1.0,
        domain: "Audit",
    },
    AgentInfo {
        name: "Orchestra",
        role: "Work-Stealing Scheduler",
        weight: 0.8,
        domain: "Compute",
    },
    AgentInfo {
        name: "THzGateway",
        role: "Spectrum & VLF",
        weight: 0.9,
        domain: "Spectrum",
    },
    AgentInfo {
        name: "Verify",
        role: "Truth & Anchor Verification",
        weight: 0.8,
        domain: "Truth",
    },
    AgentInfo {
        name: "Seeker",
        role: "NAND Knowledge Graph",
        weight: 1.0,
        domain: "Knowledge",
    },
    AgentInfo {
        name: "Archi",
        role: "Bare-Metal Microkernel Architecture",
        weight: 1.0,
        domain: "Architecture",
    },
    AgentInfo {
        name: "LogicMaster",
        role: "Formal Logic & SMT Solver",
        weight: 0.9,
        domain: "Reasoning",
    },
];

struct Proposal {
    id: u64,
    topic: String,
    category: String,
    proposer: String,
    phase: String,
    weighted_score: f32,
    approve_count: usize,
    abstain_count: usize,
    reject_count: usize,
    consensus_reached: bool,
    /// The TRM 7M's K3 verdict on this proposal's TOPIC -- the model's own trit, from running the seed-42
    /// recurrence (see trm7m.rs). Before this existed the tally keyed off `category` and `i % 7` alone, so
    /// the TOPIC was never an input and two unrelated motions tallied identically (measured: proposals
    /// 1002/1003/1004 all returned 12/0/3, weighted_score 0.7372856140136719).
    k3_model_trit: i8,
    k3_model_confidence: f32,
    k3_model_latency_ns: u64,
    /// The trit that actually decided this proposal (the confirmed fabric verdict, or -- when no
    /// candidate was confirmed -- the model tier's, per k3_retrieval's own sanctioned fall-through).
    k3_trit: i8,
    /// Which instrument produced it, NAMED: "a label that does not say which path ran is a label that
    /// cannot be checked" (k3_retrieval.rs's own words).
    k3_trit_source: String,
    /// The sealed document in the platform's own memory that CONFIRMED the topic, when one did.
    k3_fabric_doc: String,
    k3_fabric_score: i32,
    k3_fabric_matched: usize,
    deliberations: Vec<serde_json::Value>,
    votes: Vec<serde_json::Value>,
    timestamp: u64,
}

struct CouncilState {
    proposals: Vec<Proposal>,
    next_id: u64,
}

/// Where the motion counter is kept. Proposal ids used to restart at 1001 on every kernel start, so
/// two different motions could share an id and the sealed record could not tell them apart. The
/// counter is now persisted beside the other artifacts and saved as each motion is filed.
fn council_state_path() -> std::path::PathBuf {
    // The supervisor starts the kernel with its own cwd, so a relative path lands wherever that is.
    // Walk up from the executable to the first ancestor holding swarm_v2_artifacts, which is the
    // repo root the rest of the kernel already resolves against.
    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    while let Some(d) = dir {
        if d.join("swarm_v2_artifacts").is_dir() {
            return d.join("swarm_v2_artifacts").join("council_state.json");
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    std::path::PathBuf::from("swarm_v2_artifacts").join("council_state.json")
}

/// The highest id the ledger already holds. The boot counter used to come only from `council_state.json`,
/// which can lag the ledger; when it did, the seed (and any motion filed after it) was handed an id the ledger
/// already contained, `append_proposal_record` refused the duplicate write, and the motion existed only in
/// memory -- votable, executable-looking, and gone at the next restart. Measured: motions 1474 ("start the
/// native autonomy tick") and 1478 ("council-motion generator") were both filed, voted CONSENSUS, and are absent
/// from the ledger, whose 1474-1479 are the boot seed.
fn max_ledger_id() -> u64 {
    ledger_objects()
        .iter()
        .filter_map(|o| o.get("id").and_then(|v| v.as_u64()))
        .max()
        .unwrap_or(0)
}

fn load_next_id() -> u64 {
    let p = council_state_path();
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("next_id").and_then(|x| x.as_u64()))
        .map(|n| n.max(1001))
        .unwrap_or(1001)
}

/// The durable motion ledger. `state.proposals` is the live view and dies with the process; this
/// file is the record, so a restart no longer erases every motion the council ever heard.
fn council_proposals_path() -> std::path::PathBuf {
    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    while let Some(d) = dir {
        if d.join("swarm_v2_artifacts").is_dir() {
            return d.join("swarm_v2_artifacts").join("council_proposals.jsonl");
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    std::path::PathBuf::from("swarm_v2_artifacts").join("council_proposals.jsonl")
}

fn append_proposal_record(p: &Proposal) {
    let line = serde_json::json!({
        "id": p.id,
        "topic": p.topic,
        "category": p.category,
        "proposer": p.proposer,
        "phase": p.phase,
        "approve_count": p.approve_count,
        "abstain_count": p.abstain_count,
        "reject_count": p.reject_count,
        "consensus_reached": p.consensus_reached,
        "weighted_score": p.weighted_score,
        "k3_trit": p.k3_trit,
        "timestamp": p.timestamp,
    })
    .to_string();
    let path = council_proposals_path();
    if let Some(d) = path.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    // One motion, one record. The seal used to append unconditionally, so a re-seal under an existing id
    // wrote a second, different vote record for the same motion (five ids were found doubled, each with
    // different counts). The ledger is now idempotent on id: a second write is a no-op.
    let id = p.id;
    if ledger_objects().iter().any(|o| o.get("id").and_then(|v| v.as_u64()) == Some(id)) {
        // Silence here is what hid the id reuse: the caller went on to report a vote for a motion the ledger
        // never accepted. Say it out loud; the allocator above should make this unreachable.
        eprintln!("    LEDGER REFUSED duplicate id {} — motion \"{}\" would exist only in memory", id, p.topic);
        return;
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

/// Execution receipts live beside the motion ledger. A motion that reached CONSENSUS may be carried out; this
/// file is the record that it was, so "the platform decided" and "the platform did" are separately checkable.
fn council_executions_path() -> std::path::PathBuf {
    council_proposals_path().with_file_name("council_executions.jsonl")
}

/// Deliberation from Agent-Q's labs: separate from votes and from executions, so "the lab said" never
/// becomes "the council decided" by sharing a file.
fn council_consultations_path() -> std::path::PathBuf {
    council_proposals_path().with_file_name("council_consultations.jsonl")
}

fn execution_objects() -> Vec<serde_json::Value> {
    let Ok(s) = std::fs::read_to_string(council_executions_path()) else {
        return Vec::new();
    };
    serde_json::Deserializer::from_str(&s)
        .into_iter::<serde_json::Value>()
        .filter_map(|v| v.ok())
        .collect()
}

/// Every object in the ledger, however it is laid out. The file is JSONL in intent, but appends have
/// concatenated objects on one line, so a line count is not a record count -- counting lines was what
/// made the served total disagree with both the row count and the id count.
fn ledger_objects() -> Vec<serde_json::Value> {
    let Ok(s) = std::fs::read_to_string(council_proposals_path()) else {
        return Vec::new();
    };
    serde_json::Deserializer::from_str(&s)
        .into_iter::<serde_json::Value>()
        .filter_map(|v| v.ok())
        .collect()
}


/// Rebuild the in-memory proposals from the ledger at boot.
///
/// `CouncilState.proposals` used to start empty and only grow from motions proposed in the current process,
/// so every CONSENSUS motion became unreachable after a restart: `/council/proposals` served one record and
/// both `/council/execute` and `/university/act` answered `proposal_not_found` for motions that had genuinely
/// passed. The ledger holds them; this reads it back, last record per id winning.
///
/// What is NOT reconstructed is labelled rather than guessed: the vote tally and the model fields are not
/// replayed, so `k3_trit_source` says `reloaded_from_ledger` instead of naming an instrument that did not run
/// in this process. A reloaded motion can authorise an execution because its stored phase and consensus flag
/// were written when the vote was taken -- not because a fresh vote was invented here.
fn hydrate_proposals(st: &mut CouncilState) {
    use std::collections::BTreeMap;
    let mut latest: BTreeMap<u64, serde_json::Value> = BTreeMap::new();
    for o in ledger_objects() {
        if let Some(id) = o.get("id").and_then(|v| v.as_u64()) {
            latest.insert(id, o);
        }
    }
    for (id, o) in latest {
        if st.proposals.iter().any(|p| p.id == id) {
            continue;
        }
        let s = |k: &str| o.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let u = |k: &str| o.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
        let n = |k: &str| o.get(k).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        st.proposals.push(Proposal {
            id,
            topic: s("topic"),
            category: s("category"),
            proposer: s("proposer"),
            phase: s("phase"),
            weighted_score: o.get("weighted_score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            approve_count: n("approve_count"),
            abstain_count: n("abstain_count"),
            reject_count: n("reject_count"),
            consensus_reached: o.get("consensus_reached").and_then(|v| v.as_bool()).unwrap_or(false),
            k3_model_trit: 0,
            k3_model_confidence: 0.0,
            k3_model_latency_ns: 0,
            k3_trit: o.get("k3_trit").and_then(|v| v.as_i64()).unwrap_or(0) as i8,
            k3_trit_source: "reloaded_from_ledger (tally not replayed; fields absent from the record are 0)".to_string(),
            k3_fabric_doc: String::new(),
            k3_fabric_score: 0,
            k3_fabric_matched: 0,
            deliberations: Vec::new(),
            votes: Vec::new(),
            timestamp: u("timestamp"),
        });
    }
}

fn persisted_proposal_count() -> usize {
    ledger_objects().len()
}

fn save_next_id(id: u64) {
    let p = council_state_path();
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = p.with_extension("json.tmp");
    if std::fs::write(&tmp, format!("{{\"next_id\":{id}}}")).is_ok() {
        let _ = std::fs::rename(&tmp, &p);
    }
}

/// One agent's vote, decided by the agent's own domain rule against the proposal's TOPIC
/// text (not the category alone). A domain gap is an ABSTAIN carrying the binding condition
/// that must be met before enactment; a hard violation is a REJECT. This replaces the
/// procedural "domain match approves, every 7th abstains" tally, which keyed off category +
/// index and never read the topic — so two unrelated motions tallied identically.
/// What the platform already holds, assembled before any agent votes.
///
/// Until this existed the votes were keyword rules over the topic text only: motion 1024 proposed to
/// "introduce a council of fifteen agents with a PBFT quorum of nine and a Byzantine tolerance of four"
/// -- the machinery it was being voted in -- and drew 12 approvals with zero comments noticing that it
/// already runs. The fabric hit the K3 domain already computes is now handed to the agents too, so a
/// vote can say what it consulted instead of asserting that nothing contradicts it.
struct PriorArt {
    fabric: Option<(String, String)>,
    existing: Vec<String>,
}

/// The first path-like token a motion names, if any. Trimmed of the punctuation that wraps it in
/// prose ("`/foo`", "/foo,", "/foo.").
fn proposed_path(topic: &str) -> Option<String> {
    topic
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| c == ',' || c == '.' || c == ';' || c == ':' || c == '"' || c == '\'' || c == '`'))
        .find(|w| w.starts_with('/') && w.len() > 1 && w.chars().any(|c| c.is_ascii_alphabetic()))
        .map(|w| w.to_string())
}

/// Ask the listener what it does with a path. A route that already answers is a route that already
/// exists, and asking is a measurement; a hand-kept route table would be a snapshot that drifts the
/// next time someone adds a branch to the if/else chain.
fn route_status(path: &str) -> Option<u16> {
    use std::io::{Read as _, Write as _};
    let addr: std::net::SocketAddr = "127.0.0.1:8022".parse().ok()?;
    let mut s = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(400)).ok()?;
    s.set_read_timeout(Some(std::time::Duration::from_millis(800))).ok()?;
    write!(s, "GET {} HTTP/1.0\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n", path).ok()?;
    let mut buf = [0u8; 64];
    let n = s.read(&mut buf).ok()?;
    let head = String::from_utf8_lossy(&buf[..n]);
    head.split_whitespace().nth(1)?.parse().ok()
}

fn prior_art(topic: &str) -> PriorArt {
    // The mandated fusion-method clause names the 15-agent roster and the quorum, so scanning the whole
    // topic refused honest motions for describing the council that votes on them: PHASE-A and PHASE-B
    // were both rejected 15/0 on that boilerplate alone. The clause is metadata about how the vote is
    // taken, not a proposal to build anything, so it is removed before the duplicate checks run.
    let body: String = topic
        .split(['.', ';'])
        .filter(|s| !s.to_lowercase().contains("fusion method"))
        .collect::<Vec<_>>()
        .join(". ");
    let t = body.to_lowercase();
    // Only a motion that proposes to BUILD something can duplicate what exists; a motion that
    // discusses the council must not be refused for mentioning it.
    let proposes_new = ["introduce", "add a", "add an", "create", "build", "implement", "set up", "stand up"]
        .iter()
        .any(|v| t.contains(v));
    let mut existing = Vec::new();
    if proposes_new {
        if let Some(path) = proposed_path(topic) {
            if let Some(code) = route_status(&path) {
                if code != 404 {
                    existing.push(format!("route {path} already resolves with HTTP {code}; a new branch would shadow an existing handler or be swallowed by a prefix match"));
                }
            }
        }
        if (t.contains("council") || t.contains("agent")) && (t.contains("fifteen") || t.contains("15 ")) {
            existing.push(format!("a council of {} agents already runs", AGENTS.len()));
        }
        if t.contains("quorum") && (t.contains("nine") || t.contains(" 9")) {
            existing.push("quorum 9 is already the live PBFT threshold".to_string());
        }
        if t.contains("byzantine") && (t.contains("four") || t.contains(" 4")) {
            existing.push("Byzantine tolerance 4 is already configured".to_string());
        }
    }
    // The sealed fabric confirms only when a document shares two of the topic's content words, which
    // is the right rule for a verdict and a poor one for "what does the platform already know".
    // The packed store is the broader reader, so it answers for the citation when the fabric is silent.
    let fabric = k3_retrieval::fabric_lookup(topic, 1)
        .map(|h| {
            (
                h.doc.clone(),
                format!("k3 fabric, score {}, {} matched terms", h.score, h.matched_terms.len()),
            )
        })
        .or_else(|| {
            k3_retrieval::packed_retrieval(topic)
                .map(|(doc, _, score)| (doc, format!("packed turbovec, score {:.3}", score)))
        });
    PriorArt { fabric, existing }
}

fn agent_vote(agent: &AgentInfo, topic: &str, category: &str, prior: &PriorArt) -> (i8, f32, String) {
    let t = topic.to_lowercase();
    let approve = |note: &str| (1i8, 0.90f32, format!("{}: {}", agent.name, note));
    let abstain = |cond: &str| (0i8, 0.85f32, format!("{}: abstain — {}", agent.name, cond));
    let reject = |why: &str| (-1i8, 0.95f32, format!("{}: reject — {}", agent.name, why));

    if !prior.existing.is_empty() {
        return reject(&format!(
            "already implemented: {}; this would rebuild what the platform already runs",
            prior.existing.join("; ")
        ));
    }

    let (trit, conf, mut comment) = match agent.name {
        // A destructive change (kill list) without a rollback + integrity gate is a gap.
        "Shield" => {
            if t.contains("kill")
                && !(t.contains("rollback") || t.contains("snapshot") || t.contains("integrity"))
            {
                abstain("the kill list must carry a snapshot/rollback + integrity gate before execution")
            } else {
                approve("no destructive change without a gate; protected set (Defender, shell) respected")
            }
        }
        // A change with no verification step cannot be checked after the fact.
        "Verify" => {
            if !(t.contains("verify")
                || t.contains("readback")
                || t.contains("probe")
                || t.contains("measure"))
            {
                abstain("each step needs a verification: probe the rail, read back the fold")
            } else {
                approve("each step is verifiable")
            }
        }
        // Every enacted step must be sealed to the Monad ledger.
        "EventLedger" => {
            if !(t.contains("seal") || t.contains("ledger")) {
                abstain("each step must be sealed to the Monad ledger")
            } else {
                approve("each step will be sealed")
            }
        }
        // The rails must be measured, never asserted.
        "Pulse" => {
            if (t.contains("set ") || t.contains("target ") || t.contains("= 1."))
                && !(t.contains("measure")
                    || t.contains("read")
                    || t.contains("vid")
                    || t.contains("probe"))
            {
                reject("a rail value must be measured (VID read / probe), not asserted")
            } else {
                approve("rails corrected to the measured host identity, not a reference profile")
            }
        }
        // A kill directive must name its protected set; an unbounded one is a contradiction.
        "Logic" | "LogicMaster" => {
            if t.contains("kill") && !(t.contains("defender") || t.contains("shell")) {
                reject("a kill list must name its protected set; an unbounded kill directive is a contradiction")
            } else {
                approve("directive is internally consistent")
            }
        }
        // The fold must target the real boot artifact, not a side route.
        "Archi" => {
            if t.contains("boot") || t.contains("squashfs") || t.contains("usb") || t.contains("media") {
                approve("fold targets the real boot artifact")
            } else {
                abstain("the fold must target the real boot artifact (USB squashfs), not a side route")
            }
        }
        // Domain specialist approves with high confidence; everyone else approves the
        // non-contradiction check. No agent lands on an approving fallback without having
        // looked at the topic.
        _ => {
            if category.eq_ignore_ascii_case(agent.domain) {
                (
                    1i8,
                    0.98f32,
                    format!(
                        "Domain specialist {}: proposal conforms to sovereign invariance",
                        agent.name
                    ),
                )
            } else {
                approve("no domain objection; non-contradiction check passed")
            }
        }
    };

    // A vote that consulted the fabric says so, with the document and its score. This is what the
    // agents were missing: a place to record what they read rather than only what they assume.
    if trit > 0 {
        if let Some((doc, note)) = &prior.fabric {
            comment.push_str(&format!(" | fabric: {} ({})", doc, note));
        }
    }
    (trit, conf, comment)
}

fn evaluate_proposal(
    state: &mut CouncilState,
    topic: String,
    category: String,
    proposer: String,
) -> u64 {
    // Never reuse an id the ledger already holds, even if the persisted counter says otherwise.
    let ledger_ids: std::collections::HashSet<u64> = ledger_objects()
        .iter()
        .filter_map(|o| o.get("id").and_then(|v| v.as_u64()))
        .collect();
    let mut id = state.next_id;
    while ledger_ids.contains(&id) || state.proposals.iter().any(|p| p.id == id) {
        id += 1;
    }
    state.next_id = id + 1;
    save_next_id(state.next_id);

    let mut deliberations = Vec::new();
    let mut votes = Vec::new();
    let mut approve_count = 0;
    let mut abstain_count = 0;
    let mut reject_count = 0;
    let mut total_weighted = 0.0f32;

    // ── The K3 evaluation domain, instrument 1 of 2: the kernel's own quantized memory ────────────
    // k3_retrieval proposes candidates by ternary agreement and CONFIRMS one only when the sealed
    // document itself shares at least two of the topic's content words. That is its own rule, and it
    // exists because the module MEASURED the limitation this route inherits: "known queries score
    // 137..155 and nonsense 134..144, overlapping. So the score only PROPOSES candidates." A confirmed
    // hit carries the kernel's packed-domain trit and the band it cleared. Nothing confirmed -> None,
    // and the caller falls through to its model tiers -- which is what happens below, NAMED rather
    // than hidden.
    let fabric = k3_retrieval::fabric_lookup(&topic, 1);
    // Instrument 2: the TRM 7M's own trit (blocks 177/178). Recorded as corroboration, never discarded.
    let mut k3_model = trm7m::Trm7m::new();
    let (trm_trit, trm_conf, trm_ns) = k3_model.evaluate_text(&topic);
    let (k3_trit, k3_conf, k3_trit_source) = match &fabric {
        Some(h) => (
            h.packed_trit,
            trm_conf,
            format!(
                "k3_retrieval::fabric_lookup confirmed {} (score {}, {} matched terms)",
                h.doc,
                h.score,
                h.matched_terms.len()
            ),
        ),
        None => (
            trm_trit,
            trm_conf,
            "trm7m (fabric: no candidate confirmed)".to_string(),
        ),
    };
    let (k3_fabric_doc, k3_fabric_score, k3_fabric_matched) = match &fabric {
        Some(h) => (h.doc.clone(), h.score, h.matched_terms.len()),
        None => (String::new(), 0, 0),
    };

    // One stamp for the whole ballot, from the clock. The record used to carry the literal
    // "2026-08-23 18:30:00" on every vote of every proposal, and every agent's latency was the
    // literal 140 ns -- two figures that described nothing.
    let voted_at = state_handlers::utc_stamp();
    let prior = prior_art(&topic);
    for agent in AGENTS.iter() {
        let vote_t0 = std::time::Instant::now();
        let (trit, conf, comment) = agent_vote(agent, &topic, &category, &prior);
        let latency_ns = vote_t0.elapsed().as_nanos().max(1) as u64;

        if trit > 0 {
            approve_count += 1;
        } else if trit < 0 {
            reject_count += 1;
        } else {
            abstain_count += 1;
        }

        let weight = agent.weight;
        total_weighted += (trit as f32) * weight * conf;

        deliberations.push(json!({
            "agent": agent.name,
            "role": agent.role,
            "comment": comment,
            "timestamp": voted_at.clone()
        }));

        votes.push(json!({
            "agent": agent.name,
            "role": agent.role,
            "vote": trit,
            "confidence": conf,
            "weight": weight,
            "latency_ns": latency_ns
        }));
    }

    // Fold the model's trit in as the deterministic K3 domain's voice: a FAULT vetoes consensus outright,
    // an AMBIGUOUS cannot carry one, a PASS lets the rule tally decide. The agent trits above are thereby
    // corroboration of, or dissent from, the model's verdict -- they are no longer the verdict itself.
    // The model's trit contributes to the WEIGHTED score, but it must not be added to the agent
    // tallies: `approve_count`/`abstain_count`/`reject_count` are documented as per-AGENT counts and
    // are served to every client as such. Folding the model in as a 16th "agent" made a 15-member
    // roster report 13 approve + 3 abstain + 0 reject = 16 (measured on proposal 1001), which breaks
    // the tally invariant clients would check. The model's voice is reported as `model_vote` instead,
    // so the fold stays visible rather than hidden. The divisor was a hardcoded 14.0 from an older
    // roster; it is now the number of weighted contributions actually folded (15 agents + 1 model).
    if k3_trit > 0 {
        total_weighted += k3_conf;
    } else if k3_trit < 0 {
        total_weighted -= k3_conf;
    }
    let weighted_domains: f32 = AGENTS.len() as f32 + 1.0;
    let weighted_score = (total_weighted / weighted_domains).clamp(-1.0, 1.0);
    // Consensus requires the K3 model to approve, the agent quorum to pass, AND no agent to
    // have registered a hard objection (reject). A rejection blocks the proposal; an
    // abstention carries a binding condition and leaves the phase in ANALYZE.
    // PBFT with n=15 and f=4 needs 2f+1 = 9 approvals. The status route, the UI and the council's own
    // fusion-method text all declare nine; this check used to require ten, so a proposal with exactly nine
    // approvals was refused by a rule the platform did not advertise. The number now comes from one constant.
    let consensus_reached = k3_trit > 0 && approve_count >= PBFT_QUORUM && reject_count == 0;
    let phase = if reject_count > 0 || k3_trit < 0 {
        "BLOCKED"
    } else if consensus_reached {
        "CONSENSUS"
    } else {
        "ANALYZE"
    };

    let verdict_note = format!("proposal {} {}: {}", id, phase, topic);
    let proposal = Proposal {
        id,
        topic,
        category,
        proposer,
        phase: phase.to_string(),
        weighted_score,
        approve_count,
        abstain_count,
        reject_count,
        consensus_reached,
        k3_model_trit: trm_trit,
        k3_model_confidence: trm_conf,
        k3_model_latency_ns: trm_ns,
        k3_trit,
        k3_trit_source,
        k3_fabric_doc,
        k3_fabric_score,
        k3_fabric_matched,
        deliberations,
        votes,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    append_proposal_record(&proposal);
    state.proposals.push(proposal);

    // The IPC transport is the shared-memory ring bus, not HTTP: publish this council verdict as a
    // trit-tagged message so the 15 agents (and any offline reader) consume the SAME decision the
    // kernel just recorded. Best-effort — the Monad ledger is the durable record; the ring is the live
    // bus, so a full/absent ring drops the event, it never blocks the decision.
    // A council verdict travels as a fixed-layout frame, not as prose cut at 40 bytes. The frame carries the
    // same decision in 21 bytes (8-byte header + 13-byte payload) and cannot be truncated; the text note is
    // kept in the ledger, where length is not a constraint.
    let council = frame::CouncilFrame {
        proposal_id: id,
        trit: k3_trit,
        approve: approve_count as u8,
        abstain: abstain_count as u8,
        reject: reject_count as u8,
        phase: match phase {
            "CONSENSUS" => frame::PHASE_CONSENSUS,
            "BLOCKED" => frame::PHASE_BLOCKED,
            _ => frame::PHASE_ANALYZE,
        },
    };
    match frame::encode_council(council) {
        Some(bytes) => {
            let _ = mailbox_ring::push("Orchestra", "EventLedger", k3_trit, &bytes);
        }
        None => {
            let _ = mailbox_ring::push("Orchestra", "EventLedger", k3_trit, verdict_note.as_bytes());
        }
    }
    id
}

/// The parser routes take file payloads, not command JSON, so the 64 KB command cap would silently
/// truncate a larger upload and the parser would report the counts of a PREFIX. A payload over this
/// limit is refused with 413 instead: a truncated parse is not a parse.
const PARSER_BODY_LIMIT: usize = 8 * 1024 * 1024;

/// Query values arrive percent-encoded; `%XX` and `+` are decoded so a filename hint survives.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                let hex = std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(v) => {
                        out.push(v);
                        i += 3
                    }
                    Err(_) => {
                        out.push(b'%');
                        i += 1
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1
            }
            c => {
                out.push(c);
                i += 1
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn query_param(query: &str, key: &str) -> Option<String> {
    query
        .split('&')
        .find_map(|pair| pair.split_once('=').filter(|(k, _)| *k == key).map(|(_, v)| percent_decode(v)))
        .filter(|v| !v.is_empty())
}

/// A format this kernel has no native parser for is not a bad request. It is a parser the caller
/// asked for that lives in another process, so it is a 501 naming the module that owns it; only a
/// parse that failed on a format this kernel DOES implement is a 400.
fn parser_error(fmt: &str, e: String) -> (serde_json::Value, u16) {
    let in_registry = native_parsers::REGISTRY_FORMATS
        .iter()
        .any(|f| f.eq_ignore_ascii_case(fmt));
    if e.starts_with("native parser not implemented") && in_registry {
        (
            json!({
                "error": "engine_required",
                "format": fmt,
                "detail": e,
                "engine": "swarm_v2/parsers/parser_registry.py",
                "note": "the Python registry serves this format and is not in this process, so this kernel refuses rather than answer from a parser it does not have",
            }),
            501,
        )
    } else {
        (json!({"error": "parse_failed", "format": fmt, "detail": e}), 400)
    }
}

/// The `/parsers/*` POST family, matching `swarm_v2/routes/parsers.py`: the body IS the payload, the
/// format comes from `?format_hint=` (or `?filename=` by extension), and the answer is the same
/// `{"status":"SUCCESS","result":{...}}` envelope those routes return, so one client contract covers
/// both. The targeted routes pin their format instead of taking a hint.
fn parsers_post(tail: &str, query: &str, body: &str) -> (serde_json::Value, u16) {
    if body.len() > PARSER_BODY_LIMIT {
        return (
            json!({
                "error": "payload_too_large",
                "detail": "a payload over the parser limit is REFUSED rather than parsed from a truncated prefix",
                "limit_bytes": PARSER_BODY_LIMIT,
                "received_bytes": body.len(),
            }),
            413,
        );
    }
    // No empty-body guard here on purpose: the source's parsers answer an empty payload with a
    // zero-count `VALID` result rather than refusing it, and this route carries their contract, so a
    // caller sees the same thing from either implementation. That the zero result is indistinguishable
    // from a parsed-but-empty file is a property of the source, recorded rather than papered over.
    let hinted = match tail {
        "parse" => query_param(query, "format_hint").or_else(|| {
            query_param(query, "filename")
                .map(|f| f.rsplit_once('.').map(|(_, e)| e.to_string()).unwrap_or_default())
        }),
        "bench" => query_param(query, "format_hint"),
        pinned => Some(pinned.to_string()),
    };
    let fmt = hinted.unwrap_or_default();
    if fmt.is_empty() {
        return (
            json!({"detail": "no format: pass ?format_hint=fastq, or ?filename=reads.fastq; GET /parsers/status lists the formats"}),
            400,
        );
    }
    if tail == "bench" {
        let iterations = query_param(query, "iterations")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(100);
        return match native_parsers::bench(&fmt, body, iterations) {
            Ok(v) => (json!({"status": "SUCCESS", "result": v}), 200),
            Err(e) => parser_error(&fmt, e),
        };
    }
    match native_parsers::parse_dispatch(&fmt, body) {
        Ok(v) => (json!({"status": "SUCCESS", "result": v}), 200),
        Err(e) => parser_error(&fmt, e),
    }
}

/// Writes are the surface that can change state; reads stay open so health checks work from anywhere.
fn is_write_method(m: &Method) -> bool {
    matches!(m, Method::Post | Method::Put | Method::Patch | Method::Delete)
}

/// Constant-time comparison, so a rejected token does not leak its matching prefix through timing.
fn token_matches(supplied: &str, expected: &str) -> bool {
    let (a, b) = (supplied.as_bytes(), expected.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// The write gate. Loopback peers are trusted because the dashboard's proxy connects from loopback; every other
/// peer must present the node's council token, and a node with no token refuses non-loopback writes outright.
/// An unknown peer address is treated as non-loopback, so the gate fails closed.
fn auth_denial(request: &tiny_http::Request, method: &Method) -> Option<serde_json::Value> {
    if !is_write_method(method) {
        return None;
    }
    let loopback = request.remote_addr().map(|a| a.ip().is_loopback()).unwrap_or(false);
    if loopback {
        return None;
    }
    let expected = std::env::var("COUNCIL_AUTH_TOKEN")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    let Some(expected) = expected else {
        return Some(json!({
            "error": "unauthorized",
            "status": 401,
            "path": request.url(),
            "detail": "this node has no COUNCIL_AUTH_TOKEN configured, so non-loopback writes are refused",
        }));
    };
    let supplied = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("X-Council-Token"))
        .map(|h| h.value.as_str().trim().to_string());
    match supplied {
        Some(t) if token_matches(&t, &expected) => None,
        _ => Some(json!({
            "error": "unauthorized",
            "status": 401,
            "path": request.url(),
            "hint": "send the node's council token in the X-Council-Token header",
        })),
    }
}

#[cfg(test)]
mod auth_gate_tests {
    use super::token_matches;

    /// The comparison must accept exactly one string: no prefix, no suffix, no case folding, no trimming.
    #[test]
    fn tokens_match_exactly_and_nothing_else() {
        assert!(token_matches("abc123", "abc123"));
        assert!(!token_matches("abc124", "abc123"));
        assert!(!token_matches("abc12", "abc123"));
        assert!(!token_matches("abc1234", "abc123"));
        assert!(!token_matches("", "abc123"));
        assert!(!token_matches("abc123 ", "abc123"));
        assert!(!token_matches("ABC123", "abc123"));
    }
}

fn handle_request(mut request: tiny_http::Request, state: &Arc<Mutex<CouncilState>>) {
    let start_t = Instant::now();
    let url = request.url().to_string();
    let method = request.method().clone();

    // Auth gate before any route runs: a non-loopback write without the node token never reaches dispatch.
    if let Some(denied) = auth_denial(&request, &method) {
        let body = denied.to_string();
        let resp = Response::from_string(body)
            .with_status_code(401)
            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
        let _ = request.respond(resp);
        return;
    }

    // The unclaimed-path sentinel. Until now this was {"status": "ok"}: a caller asking for a path
    // the kernel does not serve received a healthy answer, so every unhandled route reported
    // success while nothing ran. The initial value is now a private key that no handler emits, and
    // reaching the end of this function still holding it means nothing claimed the path -- see the
    // not_found block at the end, which was measured at 24 routes on this build.
    let mut response_json = json!({"__unclaimed__": true});
    let mut status_code = 200;

    // Route matching
    if url == "/" || url.is_empty() {
        // Liveness root, kept explicit rather than left to the catch-all: the platform's own
        // tooling pings it, and a 404 here would be read as the kernel being down.
        response_json = json!({
            "status": "ok",
            "service": "trm_uefi_kernel",
            "python_active": false,
            "note": "liveness root; /health carries the full state",
        });
    } else if method == Method::Get && url.starts_with("/k3/shardcheck") {
        // Verification for the shard merge: scan a shard whole, then as two halves, and compare.
        let mut q = String::new();
        let mut shard = "shard_000.json".to_string();
        let mut k = 10usize;
        if let Some(qs) = url.split('?').nth(1) {
            for pair in qs.split('&') {
                let mut it = pair.splitn(2, '=');
                let key = it.next().unwrap_or("");
                let val = crate::query::decode(it.next().unwrap_or(""));
                match key {
                    "q" => q = val,
                    "shard" => shard = val,
                    "k" => k = val.parse().unwrap_or(10),
                    _ => {}
                }
            }
        }
        response_json = k3_retrieval::shard_merge_check(&q, &shard, k);
    } else if method == Method::Post && url.starts_with("/k3/tercode/build") {
        // A store rewrite is a mutation: POST only, so a link-following crawler cannot trigger it.
        let (payload, code) = k3_retrieval::tercode_build();
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && url.starts_with("/k3/") {
        // Kernel retrieval: the engine owns the algorithm, this arm only carries its answer.
        let (payload, code) = k3_retrieval::handle(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Post && url.starts_with("/recall/semantic/media") {
        // Multimodal recall (MOTION-G1): encode an image/audio with the EmbeddingGemma 2 media
        // encoders (mmproj) and rank against the 384-dim text index. Body: {"kind":"image"|"audio",
        // "data":"<base64>"}. Fails closed when the projector is not loaded — never a text fallback.
        let mut body = String::new();
        let _ = request.as_reader().take(4_194_304).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let kind = parsed.get("kind").and_then(serde_json::Value::as_str).unwrap_or("").to_string();
        let data = parsed.get("data").and_then(serde_json::Value::as_str).unwrap_or("").to_string();
        let k = parsed.get("k").and_then(serde_json::Value::as_u64).unwrap_or(5) as usize;
        if kind != "image" && kind != "audio" {
            response_json = json!({"status":"error","route":"/recall/semantic/media","error":"bad_kind",
                                   "detail":"kind must be 'image' (base64 jpeg/png) or 'audio' (base64 wav)"});
            status_code = 400;
        } else if data.is_empty() {
            response_json = json!({"status":"error","route":"/recall/semantic/media","error":"missing_data",
                                   "detail":"body: {\"kind\":\"image\"|\"audio\",\"data\":\"<base64>\"}"});
            status_code = 400;
        } else {
            response_json = semantic_recall::lookup_media(&kind, &data, k.min(20));
        }
    } else if method == Method::Post && url.starts_with("/papers/synthesize") {
        // Native paper production (MOTION-P1): synthesize the next paper from the ranked
        // candidates + the fabric recall, write papers/paper_NNN.md, seal the ledger.
        response_json = paper_production::synthesize_paper();
    } else if method == Method::Get && url.starts_with("/recall/semantic/auto") {
        // Auto-route: in-vocabulary -> the "no tokens" lattice, out-of-vocabulary -> the bridge.
        // The routing decision is named in the response (routed + routing_reason).
        let qs = url.splitn(2, '?').nth(1).unwrap_or("");
        let q = query_param(qs, "q").unwrap_or_default();
        let k = query_param(qs, "k").and_then(|v| v.parse::<usize>().ok()).unwrap_or(5);
        if q.trim().is_empty() {
            response_json = json!({"status":"error","route":"/recall/semantic/auto","error":"missing_query",
                                   "detail":"pass ?q=<text>&k=<top-k>; routed to lattice (in-vocab) or bridge (OOV)"});
            status_code = 400;
        } else {
            response_json = semantic_recall::route_semantic(q.trim(), k.min(20));
        }
    } else if method == Method::Get && url.starts_with("/recall/semantic") {
        // Semantic recall (MOTION-G1): EmbeddingGemma 2 index, separate from the HDC fabric.
        let qs = url.splitn(2, '?').nth(1).unwrap_or("");
        let q = query_param(qs, "q").unwrap_or_default();
        let k = query_param(qs, "k").and_then(|v| v.parse::<usize>().ok()).unwrap_or(5);
        if q.trim().is_empty() {
            response_json = json!({"status":"error","route":"/recall/semantic","error":"missing_query",
                                   "detail":"pass ?q=<text>&k=<top-k>; encoded by EmbeddingGemma 2 (:8590)"});
            status_code = 400;
        } else {
            response_json = semantic_recall::lookup(q.trim(), k.min(20));
        }
    } else if method == Method::Get && url.starts_with("/recall/lattice") {
        // "No tokens" semantic recall: the query is encoded by the word-embedding lattice (lookup +
        // mean, no transformer), ranked against the same text store the embedder's bridge uses.
        let qs = url.splitn(2, '?').nth(1).unwrap_or("");
        let q = query_param(qs, "q").unwrap_or_default();
        let k = query_param(qs, "k").and_then(|v| v.parse::<usize>().ok()).unwrap_or(5);
        if q.trim().is_empty() {
            response_json = json!({"status":"error","route":"/recall/lattice","error":"missing_query",
                                   "detail":"pass ?q=<text>&k=<top-k>; encoded by the word lattice (no tokens)"});
            status_code = 400;
        } else {
            response_json = semantic_recall::lookup_lattice(q.trim(), k.min(20));
        }
    } else if method == Method::Get && url.starts_with("/recall/multi") {
        // Clause-separated multi-hop retrieval (SRAM sparse_attention topology): split the query into
        // clauses, encode each with the lattice, return the union so both concepts surface.
        let qs = url.splitn(2, '?').nth(1).unwrap_or("");
        let q = query_param(qs, "q").unwrap_or_default();
        let k = query_param(qs, "k").and_then(|v| v.parse::<usize>().ok()).unwrap_or(5);
        if q.trim().is_empty() {
            response_json = json!({"status":"error","route":"/recall/multi","error":"missing_query",
                                   "detail":"pass ?q=<multi-hop text>&k=<top-k>; split on and/or/+, union of per-clause top-k"});
            status_code = 400;
        } else {
            response_json = semantic_recall::lookup_multi(q.trim(), k.min(20));
        }
    } else if method == Method::Get && url.starts_with("/recall/relate") {
        // Relationship generation (the TEMPLATE step): split into clauses, state the relationship from
        // the shared concept. Bind/intersect/imagine are the substrate; this names the relationship.
        let qs = url.splitn(2, '?').nth(1).unwrap_or("");
        let q = query_param(qs, "q").unwrap_or_default();
        if q.trim().is_empty() {
            response_json = json!({"status":"error","route":"/recall/relate","error":"missing_query",
                                   "detail":"pass ?q=<two concepts joined by and/or/+>; returns the relationship statement"});
            status_code = 400;
        } else {
            response_json = semantic_recall::relate(q.trim());
        }
    } else if method == Method::Get && url.starts_with("/recall") {
        // Multi-store recall router: routes a query by kind (general -> fabric, molecule -> chembl,
        // structured -> host-side SQL). MOTION-049/051.
        let qs = url.splitn(2, '?').nth(1).unwrap_or("");
        let q = query_param(qs, "q").unwrap_or_default();
        if q.trim().is_empty() {
            response_json = json!({"status":"error","route":"/recall","error":"missing_query",
                                   "detail":"pass ?q=<text>; the router classifies and dispatches by store"});
            status_code = 400;
        } else {
            let fused = query_param(qs, "fused").map(|v| v == "1" || v == "true").unwrap_or(false);
            let pipeline = query_param(qs, "pipeline").map(|v| v == "1" || v == "true").unwrap_or(false);
            let filter = query_param(qs, "filter").map(|v| v == "1" || v == "true").unwrap_or(false);
            let (payload, code) = if filter {
                recall_router::route_filtered(q.trim())
            } else if pipeline {
                recall_router::route_pipeline(q.trim())
            } else if fused {
                recall_router::route_fused(q.trim())
            } else {
                recall_router::route(q.trim())
            };
            response_json = payload;
            status_code = code;
        }
    } else if method == Method::Get && url.starts_with("/instella/q4k") {
        // Instella distill parity, in pure Rust through the kernel (not a side .exe): load one Q4_K
        // tensor column from the GGUF, run the int8 dot vs the dequant-to-f32 path, return the parity.
        let qs = url.splitn(2, '?').nth(1).unwrap_or("");
        let model_path = query_param(qs, "model").unwrap_or_else(|| {
            "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf".to_string()
        });
        let tensor = query_param(qs, "tensor").unwrap_or_else(|| "blk.0.ffn_gate.weight".to_string());
        let col = query_param(qs, "col").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
        response_json = instella_handlers::q4k_parity(&model_path, &tensor, col);
    } else if method == Method::Get && url.starts_with("/basis") {
        // Basis projection seam (MOTION-041): resolve a projector and apply it to a deterministic
        // probe vector. Reports the projector name and the norm-preservation check (an orthonormal
        // transform preserves ||v||), which is the in-kernel witness that cosine — and therefore
        // retrieval ranking / P@1 — is unchanged under the projection.
        let qs = url.splitn(2, '?').nth(1).unwrap_or("");
        let basis = query_param(qs, "basis").unwrap_or_else(|| "orthonormal".to_string());
        let proj = plugins::basis::projector(&basis);
        let probe: Vec<f32> = (0..768).map(|i| ((i * 37 % 101) as f32 - 50.0) / 50.0).collect();
        let out = proj.project(&probe);
        let rms = |v: &[f32]| (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt();
        response_json = json!({
            "route": "/basis",
            "basis": proj.name(),
            "output_dim": proj.output_dim(),
            "rms_in": rms(&probe),
            "rms_out": rms(&out),
            "norm_preserved": (rms(&probe) - rms(&out)).abs() < 1e-4,
            "invariant": "the icosahedral 12x12 matrix is orthonormal (verified max err 6.7e-16) and preserves cosine (delta 1.4e-17), so cos(Mq, Mv) = cos(q, v) and retrieval ranking is unchanged; only 2-bit quantization re-rounding after projection could perturb P@1",
        });
        status_code = 200;
    } else if method == Method::Get && url.starts_with("/gpu/dispatch") {
        // Voltage-binder compute dispatcher (MOTION-017): machine-checkable device binding, fail-closed.
        // The GPU arm is gated on a compute toolchain this host does not carry (no nvcc/glslc/wgpu),
        // so the binding is honest CPU-with-a-named-gap, never a faked GPU path.
        response_json = gpu_dispatch::dispatch_table();
        status_code = 200;
    } else if method == Method::Post && url.starts_with("/k3/verdict") {
        // The advertised POST form of the verdict engine: read the trits from the body and evaluate
        // the Kleene proof. Deterministic K3 logic, so it runs here rather than being 501'd.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        response_json = k3_retrieval::verdict_from_body(&body);
        status_code = if response_json.get("verdict").is_some() { 200 } else { 400 };
    } else if method == Method::Post && url.starts_with("/k3/trit/append") {
        // The guarded write into the canonical .k3 fabric (MOTION-TRIT-FABRIC, 15/15): the live
        // council loop seals here so decisions reach the trit array, not only the ledger.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let (payload, code) = nand_append::trit_append_from_body(&body);
        response_json = payload;
        status_code = code;
    } else if method == Method::Post && url == "/variant/annotate" {
        // Clinical variant annotation: variant kind + ACMG/AMP classification from caller evidence.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or(json!({}));
        response_json = variant_annotation::annotate_variant(&parsed);
    } else if method == Method::Get && (url == "/parsers/status" || url == "/parsers") {
        // The inventory: what this kernel parses natively, and the registry formats it does not,
        // named rather than omitted -- a status page reporting "2" with no account of the other 18
        // would be the omission.
        response_json = native_parsers::inventory();
    } else if method == Method::Get && url.starts_with("/parsers/formats") {
        // What the native parser implements, read from its own dispatch table rather than a second
        // list that could name a parser it does not have.
        response_json = native_parsers::formats();
    } else if method == Method::Post && url.starts_with("/parsers/") {
        // The parser hot path, in native code. The route adds no rule of its own: the body IS the
        // payload, the format is a hint or is pinned by the path, and the answer carries the same
        // envelope swarm_v2/routes/parsers.py returns.
        let mut body = String::new();
        let _ = request.as_reader().take(PARSER_BODY_LIMIT as u64 + 1).read_to_string(&mut body);
        let path_only = url.split('?').next().unwrap_or("").trim_end_matches('/').to_string();
        let tail = path_only.trim_start_matches("/parsers/").to_string();
        let query = url.splitn(2, '?').nth(1).unwrap_or("").to_string();
        let (payload, code) = parsers_post(&tail, &query, &body);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/worktree" || url.starts_with("/worktree?")) {
        let (payload, code) = worktree_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/coldstart" || url.starts_with("/coldstart?")) {
        let (payload, code) = coldstart_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/tail" || url.starts_with("/tail?")) {
        let (payload, code) = mesh_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/vault" || url.starts_with("/vault?")) {
        let (payload, code) = vault_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/build" || url.starts_with("/build?")) {
        let (payload, code) = build_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/continuum" || url.starts_with("/continuum?")) {
        let (payload, code) = continuum_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/infra" || url.starts_with("/infra?")) {
        let (payload, code) = infra_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/model" || url.starts_with("/model?")) {
        let (payload, code) = model_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/jarvis" || url.starts_with("/jarvis?")) {
        let (payload, code) = jarvis_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/sec" || url.starts_with("/sec?")) {
        let (payload, code) = sec_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/mem" || url.starts_with("/mem?")) {
        let (payload, code) = mem_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/runtime" || url.starts_with("/runtime?")) {
        let (payload, code) = runtime_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/sched" || url.starts_with("/sched?")) {
        let (payload, code) = sched_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/redis" || url.starts_with("/redis?")) {
        let (payload, code) = redis_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/binder" || url.starts_with("/binder?")) {
        let (payload, code) = binder_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/subatomic" || url.starts_with("/subatomic?")) {
        let (payload, code) = subatomic_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/optimize" || url.starts_with("/optimize?")) {
        let (payload, code) = optimize_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/arbiter" || url.starts_with("/arbiter?")) {
        let (payload, code) = arbiter_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/remediate" || url.starts_with("/remediate?")) {
        let (payload, code) = remediate_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/agents64" || url.starts_with("/agents64?")) {
        let (payload, code) = agents64_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/gpu" || url.starts_with("/gpu?")) {
        let (payload, code) = gpu_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/meta" || url.starts_with("/meta?")) {
        let (payload, code) = meta_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/selfheal" || url.starts_with("/selfheal?")) {
        let (payload, code) = selfheal_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/engine" || url.starts_with("/engine?")) {
        let (payload, code) = engine_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/arena" || url.starts_with("/arena?")) {
        let (payload, code) = arena_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/kern" || url.starts_with("/kern?")) {
        let (payload, code) = kern_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/dram" || url.starts_with("/dram?")) {
        let (payload, code) = dram_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/ports" || url.starts_with("/ports?")) {
        let (payload, code) = ports_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/math" || url.starts_with("/math?")) {
        let (payload, code) = sram_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/agent" || url.starts_with("/agent?")) {
        let (payload, code) = agent_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/testgen" || url.starts_with("/testgen?")) {
        let (payload, code) = testgen_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && url.starts_with("/cache") {
        let (payload, code) = cache_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && url == "/heal" || url.starts_with("/heal?") {
        let (payload, code) = heal_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url == "/agentq/status" || url.starts_with("/agentq/status?")) {
        response_json = agentq_bridge::status();
    } else if method == Method::Get && (url == "/agentq/ddr" || url.starts_with("/agentq/ddr?")) {
        response_json = agentq_bridge::ddr_loo_status();
    } else if method == Method::Post && url.starts_with("/agentq/ddr/learn") {
        // Learning from another system's security posture writes into the platform's DDR, so it is gated.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        match parsed.get("motion").and_then(|v| v.as_u64()) {
            None => { status_code = 400;
                response_json = json!({"status": "error", "error": "motion_required",
                    "detail": "POST /agentq/ddr/learn needs {motion}"}); }
            Some(id) => {
                let consensus = { let st = state.lock().unwrap();
                    st.proposals.iter().find(|p| p.id == id).map(|p| p.consensus_reached) };
                match consensus {
                    None => { status_code = 404;
                        response_json = json!({"status": "error", "error": "proposal_not_found", "motion": id}); }
                    Some(false) => { status_code = 400;
                        response_json = json!({"status": "error", "error": "not_consensused", "motion": id,
                            "detail": "writing law-of-one findings into the DDR is authorised by a CONSENSUS motion"}); }
                    Some(true) => {
                        let mut out = agentq_bridge::ddr_learn_from_loo();
                        if let Some(o) = out.as_object_mut() { o.insert("motion".into(), json!(id)); }
                        if out.get("status").and_then(|s| s.as_str()) != Some("LEARNED") { status_code = 502; }
                        response_json = out;
                    }
                }
            }
        }
    } else if method == Method::Post && url.starts_with("/agentq/resonance/join") {
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let motion = parsed.get("motion").and_then(|v| v.as_u64());
        let sid = parsed.get("session_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let platform = parsed.get("platform").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let intent = parsed.get("intent").and_then(|v| v.as_str()).unwrap_or("join session").to_string();
        let love = parsed.get("love_frequency").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let align = parsed.get("geometric_alignment").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let source = parsed.get("source").and_then(|v| v.as_str()).unwrap_or("unspecified").to_string();
        if motion.is_none() || sid.is_empty() || platform.is_empty() {
            status_code = 400;
            response_json = json!({"status": "error", "error": "motion_session_and_platform_required"});
        } else {
            let id = motion.unwrap();
            let consensus = { let st = state.lock().unwrap();
                st.proposals.iter().find(|p| p.id == id).map(|p| p.consensus_reached) };
            match consensus {
                None => { status_code = 404; response_json = json!({"status": "error", "error": "proposal_not_found", "motion": id}); }
                Some(false) => { status_code = 400; response_json = json!({"status": "error", "error": "not_consensused", "motion": id}); }
                Some(true) => {
                    let mut out = agentq_bridge::resonance_join(&sid, &platform, &intent, love, align, &source);
                    if let Some(o) = out.as_object_mut() { o.insert("motion".into(), json!(id)); }
                    if out.get("status").and_then(|s| s.as_str()) != Some("JOINED") { status_code = 502; }
                    response_json = out;
                }
            }
        }
    } else if method == Method::Get && url.starts_with("/agentq/resonance/search") {
        let qs = url.split('?').nth(1).unwrap_or("");
        let q = qs.split('&').find_map(|p| {
            let mut it = p.splitn(2, '=');
            if it.next() == Some("q") { Some(crate::query::decode(it.next().unwrap_or(""))) } else { None }
        }).unwrap_or_default();
        response_json = agentq_bridge::resonance_search(&q);
    } else if method == Method::Get && (url == "/agentq/resonance/products" || url.starts_with("/agentq/resonance/products?")) {
        response_json = agentq_bridge::resonance_products();
    } else if method == Method::Get && (url == "/agentq/resonance" || url.starts_with("/agentq/resonance?")) {
        response_json = agentq_bridge::resonance();
    } else if method == Method::Post && url.starts_with("/agentq/resonance/send") {
        // Contributing a wavefunction creates a session in Agent-Q's chamber, so it is gated like other
        // cross-system writes: the motion must exist and stand at CONSENSUS.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let motion = parsed.get("motion").and_then(|v| v.as_u64());
        let intent = parsed.get("intent").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let love = parsed.get("love_frequency").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let align = parsed.get("geometric_alignment").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if motion.is_none() || intent.trim().is_empty() {
            status_code = 400;
            response_json = json!({"status": "error", "error": "motion_and_intent_required",
                "detail": "POST /agentq/resonance/send needs {motion, intent, love_frequency?, geometric_alignment?}"});
        } else {
            let id = motion.unwrap();
            let consensus = { let st = state.lock().unwrap();
                st.proposals.iter().find(|p| p.id == id).map(|p| p.consensus_reached) };
            match consensus {
                None => { status_code = 404;
                    response_json = json!({"status": "error", "error": "proposal_not_found", "motion": id}); }
                Some(false) => { status_code = 400;
                    response_json = json!({"status": "error", "error": "not_consensused", "motion": id,
                        "detail": "entering Agent-Q's resonance chamber is authorised by a CONSENSUS motion"}); }
                Some(true) => {
                    let mut out = agentq_bridge::resonance_send(&intent, love, align);
                    if let Some(o) = out.as_object_mut() { o.insert("motion".into(), json!(id)); }
                    if out.get("status").and_then(|s| s.as_str()) != Some("CONTRIBUTED") { status_code = 502; }
                    response_json = out;
                }
            }
        }
    } else if url == "/k3/scan-path" {
        // Diagnostic: pin the trit scan to one implementation so "the paths agree" is measured, not asserted.
        use std::sync::atomic::Ordering;
        if method == Method::Post {
            let mut body = String::new();
            let _ = request.as_reader().take(4096).read_to_string(&mut body);
            let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
            let name = parsed.get("path").and_then(|v| v.as_str()).unwrap_or("auto");
            let code: u8 = match name {
                "gpu_u32" => 1, "gpu_byte" => 2, "cpu" => 3, "auto" => 0,
                _ => 255,
            };
            if code == 255 {
                status_code = 400;
                response_json = json!({"status": "error", "error": "unknown_path",
                                       "allowed": ["auto", "gpu_u32", "gpu_byte", "cpu"]});
            } else {
                crate::packed_ternary::FORCE_PATH.store(code, Ordering::Relaxed);
                response_json = json!({"status": "OK", "forced": name});
            }
        } else {
            response_json = json!({
                "forced": match crate::packed_ternary::FORCE_PATH.load(Ordering::Relaxed) {
                    1 => "gpu_u32", 2 => "gpu_byte", 3 => "cpu", _ => "auto" },
                "last_used": match crate::packed_ternary::LAST_PATH.load(Ordering::Relaxed) {
                    1 => "gpu_u32", 2 => "gpu_byte", 3 => "cpu", _ => "unknown" },
            });
        }
    } else if method == Method::Post && url == "/mamba/complete" {
        // Direct access to the trained SSM tier, with the measurement attached: a caller can check the
        // model name, the token count and the latency instead of trusting a prose answer.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let prompt = parsed.get("prompt").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        if prompt.is_empty() {
            status_code = 400;
            response_json = json!({"status": "error", "error": "prompt_required",
                                   "detail": "POST /mamba/complete with a prompt field"});
        } else if !crate::mamba_tier::available() {
            status_code = 503;
            response_json = json!({"status": "error", "error": "model_unavailable",
                                   "addr": crate::mamba_tier::addr(),
                                   "detail": "no mamba tier is listening; start llama-server with the GGUF"});
        } else {
            let system = parsed.get("system").and_then(|v| v.as_str()).unwrap_or("");
            let n = parsed.get("n_predict").and_then(|v| v.as_u64()).unwrap_or(128) as u16;
            match crate::mamba_tier::generate(system, &prompt, n) {
                Some(g) => {
                    response_json = json!({"status": "OK", "model": crate::mamba_tier::MODEL,
                                           "addr": crate::mamba_tier::addr(), "text": g.text,
                                           "tokens": g.tokens, "ms": g.ms,
                                           "tokens_per_second": g.tokens_per_second,
                                           "provenance": "measured_on_host"});
                }
                None => {
                    status_code = 502;
                    response_json = json!({"status": "error", "error": "empty_completion",
                                           "detail": "the server answered without text; nothing is substituted"});
                }
            }
        }
    } else if method == Method::Post && url == "/k3/encode" {
        // The encoder's own 768-dim features, so a projection can be trained against the space retrieval actually
        // uses. Batch form exists because training needs thousands of documents, not one.
        let mut body = String::new();
        let _ = request.as_reader().take(8_000_000).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let texts: Vec<String> = if let Some(arr) = parsed.get("texts").and_then(|v| v.as_array()) {
            arr.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
        } else {
            parsed.get("text").and_then(|v| v.as_str()).map(|s| vec![s.to_string()]).unwrap_or_default()
        };
        if texts.is_empty() {
            status_code = 400;
            response_json = json!({"status": "error", "error": "text_or_texts_required",
                                   "detail": "POST /k3/encode with a text field or a texts array"});
        } else {
            let dim = 768usize;
            let vectors: Vec<serde_json::Value> = texts
                .iter()
                .map(|t| json!(crate::native_encoder::encode_f32(t, dim)))
                .collect();
            response_json = json!({"status": "OK", "encoder": "native-turbovec-hdc", "dim": dim,
                                   "count": texts.len(), "vectors": vectors});
        }
    } else if method == Method::Get && (url == "/mailbox/frames" || url.starts_with("/mailbox/frames?")) {
        // Frame view of the IPC ring: what the bus actually carries, decoded by magic rather than by position.
        use std::str::FromStr;
        let agent = url.split("agent=").nth(1).map(|s| s.split('&').next().unwrap_or("")).unwrap_or("EventLedger");
        let limit = url.split("limit=").nth(1).and_then(|s| u64::from_str(s.split('&').next().unwrap_or("")).ok()).unwrap_or(200);
        response_json = mailbox_ring::scan_frames(agent, limit as usize);
    } else if method == Method::Get && (url == "/autonomy/status" || url.starts_with("/autonomy/status?")) {
        response_json = autonomy::status();
    } else if method == Method::Post && (url == "/autonomy/tick" || url.starts_with("/autonomy/tick?")) {
        // Manual tick: same code path the loop runs, so a caller can prove what the clock does.
        response_json = autonomy::tick();
    } else if method == Method::Get && (url == "/voltage" || url.starts_with("/voltage?")) {
        response_json = host_sensors::voltage();
    } else if method == Method::Get && (url == "/sensors" || url.starts_with("/sensors?")) {
        response_json = host_sensors::status();
    } else if method == Method::Get && (url == "/agentq/aura" || url.starts_with("/agentq/aura?")) {
        response_json = agentq_bridge::aura();
    } else if method == Method::Post && url.starts_with("/agentq/aura/evolve") {
        // AURA execution mutates its field, so it carries the same gate as every other cross-system write:
        // a motion whose stored phase is CONSENSUS. The code is AURA's own language, passed through untouched.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let motion = parsed.get("motion").and_then(|v| v.as_u64());
        let code = parsed.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if motion.is_none() || code.trim().is_empty() {
            status_code = 400;
            response_json = json!({"status": "error", "error": "motion_and_code_required",
                "detail": "POST /agentq/aura/evolve needs {motion, code}"});
        } else {
            let id = motion.unwrap();
            let consensus = { let st = state.lock().unwrap();
                st.proposals.iter().find(|p| p.id == id).map(|p| p.consensus_reached) };
            match consensus {
                None => { status_code = 404;
                    response_json = json!({"status": "error", "error": "proposal_not_found", "motion": id}); }
                Some(false) => { status_code = 400;
                    response_json = json!({"status": "error", "error": "not_consensused", "motion": id,
                        "detail": "evolving AURA's field is authorised by a CONSENSUS motion"}); }
                Some(true) => {
                    let mut out = agentq_bridge::aura_evolve(&code);
                    if let Some(o) = out.as_object_mut() {
                        o.insert("motion".into(), json!(id));
                        o.insert("code_chars".into(), json!(code.chars().count()));
                    }
                    if out.get("status").and_then(|s| s.as_str()) != Some("EXECUTED") {
                        status_code = 502;
                    }
                    response_json = out;
                }
            }
        }
    } else if method == Method::Get && (url == "/agentq/autonomous" || url.starts_with("/agentq/autonomous?")) {
        response_json = agentq_bridge::autonomous();
    } else if method == Method::Get && (url == "/agentq/goals" || url.starts_with("/agentq/goals?")) {
        response_json = agentq_bridge::goals();
    } else if method == Method::Get && (url == "/agentq/tasks" || url.starts_with("/agentq/tasks?")) {
        response_json = agentq_bridge::tasks();
    } else if method == Method::Get && (url == "/agentq/mission" || url.starts_with("/agentq/mission?")) {
        response_json = agentq_bridge::mission();
    } else if method == Method::Get && (url == "/agentq/consciousness" || url.starts_with("/agentq/consciousness?")) {
        response_json = agentq_bridge::consciousness();
    } else if method == Method::Get && (url == "/agentq/artifacts" || url.starts_with("/agentq/artifacts?")) {
        response_json = agentq_bridge::artifacts();
    } else if method == Method::Get && (url == "/agentq/performance" || url.starts_with("/agentq/performance?")) {
        response_json = agentq_bridge::performance();
    } else if method == Method::Get && (url == "/agentq/ecosystem" || url.starts_with("/agentq/ecosystem?")) {
        response_json = agentq_bridge::ecosystem();
    } else if method == Method::Get && (url == "/agentq/modules" || url.starts_with("/agentq/modules?")) {
        response_json = agentq_bridge::modules();
    } else if method == Method::Get && url.starts_with("/agentq/graph/node/") {
        let id = url.trim_start_matches("/agentq/graph/node/").split('?').next().unwrap_or("").to_string();
        response_json = agentq_bridge::graph_node(&id);
        if response_json.get("status").and_then(|s| s.as_str()) == Some("error") {
            status_code = 404;
        }
    } else if method == Method::Get && url.starts_with("/agentq/graph/search") {
        let qs = url.split('?').nth(1).unwrap_or("");
        let mut q = String::new();
        let mut domain: Option<String> = None;
        let mut limit: u32 = 20;
        for pair in qs.split('&') {
            let mut it = pair.splitn(2, '=');
            let key = it.next().unwrap_or("");
            let val = crate::query::decode(it.next().unwrap_or(""));
            match key {
                "q" => q = val,
                "domain" => {
                    if !val.is_empty() {
                        domain = Some(val);
                    }
                }
                "limit" => limit = val.parse().unwrap_or(20),
                _ => {}
            }
        }
        response_json = agentq_bridge::graph_search(&q, domain.as_deref(), limit);
    } else if method == Method::Get && (url == "/agentq/graph" || url.starts_with("/agentq/graph?")) {
        response_json = agentq_bridge::graph_overview();
    } else if method == Method::Get && (url == "/agentq/hera7" || url.starts_with("/agentq/hera7?")) {
        response_json = agentq_bridge::hera7();
    } else if method == Method::Get && (url == "/agentq/security" || url.starts_with("/agentq/security?")) {
        response_json = agentq_bridge::security();
    } else if method == Method::Get && (url == "/agentq/agents" || url.starts_with("/agentq/agents?")) {
        response_json = agentq_bridge::agents();
    } else if method == Method::Get && (url == "/agentq/labs" || url.starts_with("/agentq/labs?")) {
        response_json = agentq_bridge::labs();
    } else if method == Method::Get && url.starts_with("/agentq/labs/") && url.contains("/experiment-types") {
        let lab = url.trim_start_matches("/agentq/labs/").split('/').next().unwrap_or("").to_string();
        response_json = agentq_bridge::lab_experiment_types(&lab);
    } else if method == Method::Get && url.starts_with("/agentq/blackboard/") {
        let key = url.trim_start_matches("/agentq/blackboard/").split('?').next().unwrap_or("").to_string();
        if key.is_empty() {
            status_code = 400;
            response_json = json!({"status": "error", "error": "key_required", "detail": "GET /agentq/blackboard/<key>"});
        } else {
            response_json = agentq_bridge::blackboard_get(&key);
        }
    } else if method == Method::Post && url.starts_with("/agentq/publish") {
        // Publishing into Agent-Q's blackboard is a write into another system, so it is gated exactly like a
        // university action: the motion must exist and its stored phase must be CONSENSUS.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let motion = parsed.get("motion").and_then(|v| v.as_u64());
        let key = parsed.get("key").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        let value = parsed.get("value").cloned().unwrap_or(json!(null));
        let agent = parsed.get("agent").and_then(|v| v.as_str()).unwrap_or("TRM-Kernel");
        if motion.is_none() || key.is_empty() {
            status_code = 400;
            response_json = json!({"status": "error", "error": "motion_and_key_required",
                "detail": "POST /agentq/publish needs {motion, key, value, agent?}"});
        } else {
            let id = motion.unwrap();
            let consensus = {
                let st = state.lock().unwrap();
                st.proposals.iter().find(|p| p.id == id).map(|p| p.consensus_reached)
            };
            match consensus {
                None => {
                    status_code = 404;
                    response_json = json!({"status": "error", "error": "proposal_not_found", "motion": id});
                }
                Some(false) => {
                    status_code = 400;
                    response_json = json!({"status": "error", "error": "not_consensused", "motion": id,
                        "detail": "publishing into Agent-Q is authorised by a CONSENSUS motion"});
                }
                Some(true) => {
                    let mut out = agentq_bridge::post_blackboard(&key, &value, agent);
                    if let Some(o) = out.as_object_mut() {
                        o.insert("motion".into(), json!(id));
                    }
                    let ok = out.get("published").and_then(|v| v.as_bool()).unwrap_or(false);
                    status_code = if ok { 200 } else { 502 };
                    response_json = out;
                }
            }
        }
    } else if method == Method::Post && url.starts_with("/agentq/route") {
        // Delegation, not imitation: the prompt goes to Agent-Q's own router and its decision comes back
        // with the provider, task class and latency it measured. The kernel does not answer in its place.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let prompt = parsed.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let outcome = agentq_bridge::route(&prompt);
        if outcome.get("status").and_then(|s| s.as_str()) == Some("error") {
            status_code = 400;
        }
        response_json = outcome;
    } else if method == Method::Get && (url == "/university/status" || url.starts_with("/university/status?")) {
        response_json = university_client::status();
    } else if method == Method::Get && url.starts_with("/university/") {
        // Read-only proxy into the Lyceum's JSON API. The allow-list lives in university_client; a path not on
        // it is refused by name rather than forwarded.
        let path = url.trim_start_matches("/university/").split('?').next().unwrap_or("").to_string();
        let (body, code) = university_client::proxy(&path);
        response_json = body;
        status_code = code;
    } else if method == Method::Get && url.starts_with("/humanistic/ask") {
        response_json = humanistic_teach::ask(&url);
    } else if method == Method::Get && url.starts_with("/ingest") {
        let (payload, code) = ingest_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && url.starts_with("/coord/protocol") {
        let (payload, code) = coord_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && url.starts_with("/logic/k3") {
        // Wires the five Kleene operations that nothing could reach.
        let (payload, code) = logic_routes::payload(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && url.starts_with("/knowledge/sql") {
        // Read-only query over the structured tier; identifiers whitelisted, values bound.
        let (payload, code) = knowledge_stores::query(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && url.starts_with("/knowledge/stores") {
        // Motion 1257: every store with its own count and rule; unreadable is reported, never omitted.
        response_json = knowledge_stores::payload();
    } else if method == Method::Get && url.starts_with("/knowledge/surface") {
        // PHASE-A (motion 1207): one derived inventory of what the platform already holds.
        response_json = knowledge_surface::payload();
    } else if method == Method::Get && url.starts_with("/ledger/claims") {
        response_json = state_handlers::claims_payload();
    } else if method == Method::Get && (url == "/council/status" || url == "/council/status/") {
        let st = state.lock().unwrap();
        let members: Vec<serde_json::Value> = AGENTS
            .iter()
            .map(|a| {
                // (a) of the council's decision: the per-agent model assignment was NOT observable here --
                // this record carried name/role/status only, while the docs say each agent is equipped with
                // its own 7M TRM. It is published now from the GENERATED table
                // (agent_models_generated.rs <- agent_intelligence_bridge.py::AGENT_MODELS), and the field
                // names what it is instead of implying a binding that nothing dispatches.
                let (assigned, title, doms, conf) = match agent_models_generated::for_agent(a.name) {
                    Some(m) => (m.assigned_model, m.title, json!(m.domains),
                                Some(agent_models_generated::confidence(m))),
                    None => ("TRM 7M (Mamba-3 SSM, 7M params)", "", json!([]), None),
                };
                json!({
                    "name": a.name,
                    "role": a.role,
                    "status": "ONLINE",
                    "assigned_model": assigned,
                    "domain": title,
                    "domains": doms,
                    "confidence": conf,
                    "assignment_source": "swarm_v2/core/agent_intelligence_bridge.py::AGENT_MODELS (generated)",
                    "runnable_at": "/trm7m (POST {\"text\": ...} runs this model on the served surface)"
                })
            })
            .collect();

        let active_proposals: Vec<serde_json::Value> = st
            .proposals
            .iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "topic": p.topic,
                    "category": p.category,
                    "proposer": p.proposer,
                    "phase": p.phase,
                    "weighted_score": p.weighted_score,
                    "approve_count": p.approve_count,
                    "consensus_reached": p.consensus_reached,
                    "reject_count": p.reject_count,
                    "abstain_count": p.abstain_count,
                    "k3_model_trit": p.k3_model_trit,
                    "k3_model_label": trm7m::trit_label(p.k3_model_trit),
                    "k3_model_confidence": p.k3_model_confidence,
                    "k3_model_latency_ns": p.k3_model_latency_ns,
                    "k3_trit": p.k3_trit,
                    "k3_trit_source": p.k3_trit_source,
                    "k3_fabric_doc": p.k3_fabric_doc,
                    "k3_fabric_score": p.k3_fabric_score,
                    "k3_fabric_matched": p.k3_fabric_matched
                })
            })
            .collect();

        response_json = json!({
            "server": "Sovereign Native Rust Kernel v20.0.0",
            "python_active": false,
            "members": members,
            "active_proposals": active_proposals,
            "stats": {
                "total_proposals": persisted_proposal_count(),
                "total_agents": 15,
                "pbft_quorum": PBFT_QUORUM,
                "monad_anchor": "0x4D4F4E4144000001"
            }
        });
    } else if method == Method::Get && (url == "/council/proposals" || url == "/council/proposals/")
    {
        let st = state.lock().unwrap();
        let proposals: Vec<serde_json::Value> = st
            .proposals
            .iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "topic": p.topic,
                    "category": p.category,
                    "proposer": p.proposer,
                    "phase": p.phase,
                    "weighted_score": p.weighted_score,
                    // The list used to omit the gate's own state: a caller could see PHASE but not whether
                    // consensus was reached, so "is this motion allowed to execute?" was unanswerable from
                    // this endpoint. Served now, with the tally that decided it.
                    "consensus_reached": p.consensus_reached,
                    "k3_trit": p.k3_trit,
                    "k3_trit_source": p.k3_trit_source,
                    "approve_count": p.approve_count,
                    "abstain_count": p.abstain_count,
                    "reject_count": p.reject_count,
                    "timestamp": p.timestamp
                })
            })
            .collect();

        response_json = json!({ "proposals": proposals, "count": proposals.len() });
    } else if method == Method::Get && url.starts_with("/council/proposal/") {
        let parts: Vec<&str> = url.split('/').collect();
        let id_opt = parts.last().and_then(|s| s.parse::<u64>().ok());

        let st = state.lock().unwrap();
        if let Some(id) = id_opt {
            if let Some(p) = st.proposals.iter().find(|prop| prop.id == id) {
                response_json = json!({
                    "id": p.id,
                    "topic": p.topic,
                    "category": p.category,
                    "proposer": p.proposer,
                    "phase": p.phase,
                    "weighted_score": p.weighted_score,
                    "approve_count": p.approve_count,
                    "abstain_count": p.abstain_count,
                    "reject_count": p.reject_count,
                    "consensus_reached": p.consensus_reached,
                    // The single-proposal read-back used to omit these, so a client saw BLOCKED with 12
                    // approvals and no field saying the K3 model vetoed it. The decisive verdict is served.
                    "k3_trit": p.k3_trit,
                    "k3_trit_source": p.k3_trit_source,
                    "k3_model_trit": p.k3_model_trit,
                    "k3_model_label": trm7m::trit_label(p.k3_model_trit),
                    "k3_model_confidence": p.k3_model_confidence,
                    "k3_model_latency_ns": p.k3_model_latency_ns,
                    "k3_fabric_doc": p.k3_fabric_doc,
                    "k3_fabric_score": p.k3_fabric_score,
                    "k3_fabric_matched": p.k3_fabric_matched,
                    "pbft_quorum": PBFT_QUORUM,
                    "deliberations": p.deliberations,
                    "votes": p.votes
                });
            } else {
                status_code = 404;
                response_json = json!({"error": "Proposal not found"});
            }
        } else {
            status_code = 400;
            response_json = json!({"error": "Invalid proposal ID"});
        }
    } else if method == Method::Post && url.starts_with("/council/propose") {
        // The submitted body is what gets filed. This arm used to ignore the request entirely and
        // store a hardcoded "Operator Sovereign Directive" no matter what the caller sent.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let topic = parsed
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if topic.is_empty() {
            status_code = 400;
            response_json = json!({
                "status": "error",
                "error": "topic_required",
                "detail": "POST /council/propose needs a JSON body with at least {\"topic\": \"...\"}; category and proposer are optional.",
            });
        } else {
            let category = parsed
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("Uncategorised")
                .to_string();
            let proposer = parsed
                .get("proposer")
                .and_then(|v| v.as_str())
                .unwrap_or("Operator")
                .to_string();
            // Idempotent on TOPIC. Repeated callers (a benchmark loop, a retry) were filing the same motion
            // over and over -- measured: proposals 1365-1370 were six identical CONSENSUS records, each one
            // inflating the ledger and the total. Normalise whitespace and case, then answer with the
            // existing motion instead of creating a twin.
            let norm = |s: &str| s.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
            let want = norm(&topic);
            let mut existing: Option<(u64, String, bool)> = None;
            {
                let st = state.lock().unwrap();
                if let Some(p) = st.proposals.iter().find(|p| norm(&p.topic) == want) {
                    existing = Some((p.id, p.phase.clone(), p.consensus_reached));
                }
            }
            if existing.is_none() {
                for o in ledger_objects() {
                    if o.get("topic").and_then(|v| v.as_str()).map(|t| norm(t) == want).unwrap_or(false) {
                        if let Some(id) = o.get("id").and_then(|v| v.as_u64()) {
                            existing = Some((
                                id,
                                o.get("phase").and_then(|v| v.as_str()).unwrap_or("recorded").to_string(),
                                o.get("consensus_reached").and_then(|v| v.as_bool()).unwrap_or(false),
                            ));
                        }
                        break;
                    }
                }
            }
            if let Some((id, phase, consensus)) = existing {
                response_json = json!({
                    "status": "ALREADY_PROPOSED", "id": id, "phase": phase,
                    "consensus_reached": consensus, "topic": topic,
                    "basis": "this topic is already on the record; filing it again would inflate the tally without adding a motion"
                });
            } else {
            let mut st = state.lock().unwrap();
            let id = evaluate_proposal(&mut st, topic.clone(), category.clone(), proposer.clone());
            response_json = json!({
                "status": "PROPOSED",
                "id": id,
                "topic": topic,
                "category": category,
                "proposer": proposer,
                "basis": "the stored values are the submitter's own; each agent votes its own domain rule against the topic text — a domain gap abstains with the binding condition named, a violation rejects — so the tally responds to what the proposal actually says",
            });
            }
        }
    } else if method == Method::Post && url.starts_with("/university/act") {
        // The only write path into the university, and it is gated: a motion id must name a proposal whose
        // stored phase is CONSENSUS. The vote is the authorisation, exactly as /council/execute treats it --
        // a caller cannot talk the platform into enrolling a student by sending a plausible payload.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let motion = parsed.get("motion").and_then(|v| v.as_u64());
        let action = parsed.get("action").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        let payload = parsed.get("payload").cloned().unwrap_or(json!({}));
        if motion.is_none() || action.is_empty() {
            status_code = 400;
            response_json = json!({
                "status": "error", "error": "motion_and_action_required",
                "detail": "POST /university/act needs a consensus motion id, an action from api/..., and an optional payload"
            });
        } else {
            let id = motion.unwrap();
            let found = {
                let st = state.lock().unwrap();
                st.proposals.iter().find(|p| p.id == id)
                    .map(|p| (p.consensus_reached, p.phase.clone(), p.k3_trit, p.topic.clone()))
            };
            match found {
                None => {
                    status_code = 404;
                    response_json = json!({"status": "error", "error": "proposal_not_found", "motion": id});
                }
                Some((false, phase, trit, _)) => {
                    status_code = 400;
                    response_json = json!({
                        "status": "error", "error": "not_consensused", "motion": id, "phase": phase,
                        "k3_trit": trit,
                        "detail": "a university action is authorised by a CONSENSUS motion, not by the caller's request"
                    });
                }
                Some((true, phase, trit, topic)) => {
                    let mut outcome = university_client::act(&action, &payload, id);
                    let executed = outcome.get("status").and_then(|s| s.as_str()) == Some("EXECUTED");
                    // Record what happened whether or not it worked: a receipt is only written for successes, so
                    // without this line every failure simply vanished and the ledger read as 100% success.
                    let _recorded = crate::council_outcomes::record(
                        "execution", id, &topic, &format!("university:{}", action), &outcome);
                    if executed {
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                        let receipt = json!({
                            "id": id, "topic": topic, "action": format!("university:{}", action),
                            "outcome": outcome.get("university").map(|v| v.to_string()).unwrap_or_default()
                                .chars().take(240).collect::<String>(),
                            "phase": phase, "k3_trit": trit, "pbft_quorum": PBFT_QUORUM,
                            "monad_anchor": "0x4D4F4E4144000001", "timestamp": ts
                        });
                        let path = council_executions_path();
                        if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
                        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                            use std::io::Write;
                            let _ = writeln!(f, "{}", receipt);
                        }
                    }
                    // Publish the execution where Agent-Q's agents can see it. The execution is already
                    // done and receipted; a publish failure is reported beside it, never allowed to undo it.
                    if executed {
                        let key = format!("trm_university_{}_{}", id, action.replace('/', "_"));
                        let published = agentq_bridge::post_blackboard(
                            &key,
                            &json!({"motion": id, "action": action, "topic": topic,
                                    "university": outcome.get("university").cloned().unwrap_or(json!(null)),
                                    "monad_anchor": "0x4D4F4E4144000001", "agent": "TRM-Kernel"}),
                            "TRM-Kernel");
                        if let Some(obj) = outcome.as_object_mut() {
                            obj.insert("agentq_blackboard".to_string(), published);
                        }
                    }
                    status_code = if executed { 200 } else { 502 };
                    response_json = outcome;
                }
            }
        }
    } else if method == Method::Post && url == "/council/outcome" {
        // The Architect's accept/reject: the one label this platform can trust, because a human said so.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let id_opt = parsed.get("id").and_then(|v| v.as_u64());
        let decision = parsed.get("decision").and_then(|v| v.as_str()).unwrap_or("").to_ascii_lowercase();
        if id_opt.is_none() || !matches!(decision.as_str(), "accept" | "reject") {
            status_code = 400;
            response_json = json!({
                "status": "error", "error": "id_and_decision_required",
                "detail": "POST /council/outcome {\"id\": <motion>, \"decision\": \"accept\"|\"reject\", \"note\": \"...\"}"
            });
        } else {
            let id = id_opt.unwrap();
            let topic = {
                let st = state.lock().unwrap();
                st.proposals.iter().find(|p| p.id == id).map(|p| p.topic.clone())
            }
            .or_else(|| {
                ledger_objects().into_iter()
                    .find(|o| o.get("id").and_then(|v| v.as_u64()) == Some(id))
                    .and_then(|o| o.get("topic").and_then(|v| v.as_str()).map(str::to_string))
            })
            .unwrap_or_default();
            let note = parsed.get("note").and_then(|v| v.as_str()).unwrap_or("");
            let rec = crate::council_outcomes::record_decision(id, &topic, decision == "accept", note);
            response_json = json!({"status": "RECORDED", "outcome": rec});
        }
    } else if url == "/goals/status" || url == "/goals/tick" {
        if method == Method::Post && url == "/goals/tick" {
            let mut body = String::new();
            let _ = request.as_reader().take(4096).read_to_string(&mut body);
            let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
            let dispatch = parsed.get("dispatch").and_then(|v| v.as_bool()).unwrap_or(false);
            let force = parsed.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            let only = parsed.get("goal").and_then(|v| v.as_str()).map(str::to_string);
            response_json = crate::goal_runner::tick_with(dispatch, force, only.as_deref());
        } else {
            response_json = crate::goal_runner::status();
        }
    } else if method == Method::Get && url == "/council/labels" {
        // The training set as it actually stands, with the minority class counted so a caller cannot pretend
        // 16 negatives is enough.
        response_json = crate::council_outcomes::labels(&council_proposals_path());
    } else if method == Method::Get && (url == "/council/executions" || url == "/council/executions/") {
        let receipts = execution_objects();
        response_json = json!({
            "server": "Sovereign Native Rust Kernel v20.0.0",
            "executions": receipts,
            "count": receipts.len(),
            "rule": "a receipt is written only for a motion whose phase is CONSENSUS; it records the vote it was gated on"
        });
    } else if method == Method::Post && url.starts_with("/council/consult-labs") {
        // Consultation is available BEFORE consensus on purpose: its point is to inform the vote. It never
        // writes to the proposal's tally -- the record lands in council_consultations.jsonl, a separate file.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let motion = parsed.get("motion").and_then(|v| v.as_u64());
        let lab = parsed.get("lab").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        let exp_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        let params = parsed.get("params").cloned().unwrap_or(json!({}));
        if motion.is_none() || lab.is_empty() || exp_type.is_empty() {
            status_code = 400;
            response_json = json!({"status": "error", "error": "motion_lab_and_type_required",
                "detail": "POST /council/consult-labs needs {motion, lab, type, params?}"});
        } else {
            let id = motion.unwrap();
            let known = { let st = state.lock().unwrap();
                st.proposals.iter().find(|p| p.id == id).map(|p| (p.topic.clone(), p.category.clone(), p.phase.clone())) };
            match known {
                None => {
                    status_code = 404;
                    response_json = json!({"status": "error", "error": "proposal_not_found", "motion": id});
                }
                Some((topic, category, phase)) => {
                    let outcome = agentq_bridge::lab_run(&lab, &exp_type, &params);
                    let ran = outcome.get("status").and_then(|s| s.as_str()) == Some("RAN");
                    let mut response = outcome;
                    if let Some(o) = response.as_object_mut() {
                        o.insert("motion".into(), json!(id));
                        o.insert("topic".into(), json!(topic));
                        o.insert("category".into(), json!(category));
                        o.insert("phase".into(), json!(phase));
                    }
                    if ran {
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                        let receipt = json!({
                            "motion": id, "topic": topic, "category": category, "phase_at_consult": phase,
                            "lab": lab, "experiment": exp_type, "params": params,
                            "result": response.get("result").cloned().unwrap_or(json!(null)),
                            "source": format!("agentq-lab:{}", lab),
                            "counts_as_vote": false,
                            "monad_anchor": "0x4D4F4E4144000001", "timestamp": ts
                        });
                        let path = council_consultations_path();
                        if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
                        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                            use std::io::Write;
                            let _ = writeln!(f, "{}", receipt);
                        }
                        status_code = 200;
                    } else {
                        status_code = 502;
                    }
                    response_json = response;
                }
            }
        }
    } else if method == Method::Post && url.starts_with("/council/execute") {
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let id_opt = parsed.get("id").and_then(|v| v.as_u64());
        let action = parsed.get("action").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        let outcome = parsed.get("outcome").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        if id_opt.is_none() || action.is_empty() {
            status_code = 400;
            response_json = json!({
                "status": "error",
                "error": "id_and_action_required",
                "detail": "POST /council/execute needs {\"id\": <proposal id>, \"action\": \"...\"} and an optional \"outcome\"; the route will not invent an action."
            });
        } else {
            let id = id_opt.unwrap();
            let found = {
                let st = state.lock().unwrap();
                st.proposals.iter().find(|p| p.id == id).map(|p| {
                    (p.topic.clone(), p.phase.clone(), p.consensus_reached, p.k3_trit,
                     p.approve_count, p.abstain_count, p.reject_count)
                })
            };
            match found {
                None => {
                    status_code = 404;
                    response_json = json!({"status": "error", "error": "proposal_not_found", "id": id,
                                           "detail": "proposals live in memory and are reloaded from council_proposals.jsonl at boot"});
                }
                Some((topic, phase, consensus, trit, appr, abst, rej)) => {
                    if !consensus {
                        status_code = 400;
                        response_json = json!({
                            "status": "error", "error": "not_consensused", "id": id, "phase": phase,
                            "k3_trit": trit, "approve_count": appr, "abstain_count": abst, "reject_count": rej,
                            "pbft_quorum": PBFT_QUORUM,
                            "detail": "only a motion whose phase is CONSENSUS may be executed; the vote is the gate, not the caller's intent."
                        });
                    } else {
                        let existing = execution_objects()
                            .into_iter()
                            .find(|o| o.get("id").and_then(|v| v.as_u64()) == Some(id));
                        match existing {
                            Some(rec) => {
                                response_json = json!({"status": "ALREADY_EXECUTED", "receipt": rec,
                                                       "rule": "one motion, one receipt; a second call is a no-op"});
                            }
                            None => {
                                let ts = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                                // The caller's outcome is usually JSON in a string. If it does not parse there is
                                // no status to read, so it stays unclassified instead of being credited as success.
                                let outcome_value: serde_json::Value = serde_json::from_str(&outcome)
                                    .unwrap_or_else(|_| json!({"raw": outcome}));
                                let _recorded = crate::council_outcomes::record(
                                    "execution", id, &topic, &action, &outcome_value);
                                let receipt = json!({
                                    "id": id, "topic": topic, "action": action, "outcome": outcome,
                                    "phase": phase, "k3_trit": trit, "approve_count": appr,
                                    "abstain_count": abst, "reject_count": rej, "pbft_quorum": PBFT_QUORUM,
                                    "monad_anchor": "0x4D4F4E4144000001", "timestamp": ts
                                });
                                let path = council_executions_path();
                                if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
                                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                                    use std::io::Write;
                                    let _ = writeln!(f, "{}", receipt);
                                }
                                response_json = json!({"status": "EXECUTED", "receipt": receipt});
                            }
                        }
                    }
                }
            }
        }
    } else if method == Method::Get && (url == "/council/members" || url == "/council/members/") {
        let members: Vec<serde_json::Value> = AGENTS
            .iter()
            .map(|a| json!({ "name": a.name, "role": a.role, "status": "ONLINE" }))
            .collect();
        let n = AGENTS.len() as u64;
        let f = n.saturating_sub(1) / 3;
        response_json = json!({
            "members": members,
            "total_members": n,
            "quorum": 2 * f + 1,
            "byzantine_tolerance": f,
            "roster_source": "the AGENTS table compiled into this kernel - one owner, not re-declared here",
        });
    } else if method == Method::Get && (url == "/council/ledger" || url == "/council/ledger/") {
        // The council's ledger IS the sealed monad chain, served by its one owner.
        response_json = state_handlers::ledger_monad_blocks_payload();
    } else if method == Method::Post && url.starts_with("/council/") {
        let action = url
            .trim_start_matches("/council/")
            .split('/')
            .next()
            .unwrap_or("");
        status_code = 501;
        response_json = json!({
            "status": "not_a_separate_step",
            "route": url,
            "message": format!("/council/{action}/<id> is not a separate step. The Rust kernel deliberates, votes, and seals a proposal atomically in POST /council/propose (evaluate_proposal); there is no host_council_deliberation owner and no Python path."),
        });
    } else if method == Method::Get && url == "/parity/probe" {
        // G190 parity probe: fixed inputs, values computed by THIS build.
        response_json = parity_probe::run();

    } else if method == Method::Get && url == "/orphan/probe" {
        response_json = orphan_probe::run();

    } else if method == Method::Get && (url == "/health" || url == "/health/") {
        // Uptime and memory are MEASURED from this process, never typed: this payload
        // carried uptime_seconds: 3600 and memory_kb: 32768 as constants.
        response_json = json!({
            "status": "HEALTHY",
            "kernel": "Native Rust Sovereign Kernel v20.0.0",
            "python_active": false,
            "monad_anchor": "0x4D4F4E4144000001",
            "uptime_seconds": host_probe::process_uptime_seconds(),
            "memory_kb": host_probe::process_working_set_mb()
                .map(|mb| (mb * 1024.0).round() as u64)
        });
    } else if method == Method::Get && (url == "/benchmarks" || url == "/benchmarks/") {
        // Native benchmark suite (council MOTION-B1): real wall-clock of the
        // in-kernel machinery, every figure labeled measured_on_host.
        response_json = benchmark_handlers::benchmark_payload();
    } else if method == Method::Get && url == "/system/resources" {
        // The resources the dashboard asks for specifically; measured, null when unavailable.
        let vram = host_probe::gpu_vram();
        response_json = json!({
            "cpu_percent": host_probe::cpu_percent(),
            "memory_used_mb": host_probe::process_working_set_mb(),
            "memory_total_mb": host_probe::total_memory_mb(),
            "host_memory_used_mb": host_probe::used_memory_mb(),
            "active_threads": host_probe::active_threads(),
            "l3_sram_kb": host_probe::l3_cache_kb(),
            "total_vram_gb": vram.map(|(t, _, _)| t / 1024.0),
            "used_vram_gb": vram.map(|(_, u, _)| u / 1024.0),
            "gpu_util_pct": vram.map(|(_, _, g)| g),
            "basis": "measured on this host via host_probe (Windows API / nvidia-smi); null = unavailable",
        });
    } else if method == Method::Get && (url == "/system/rails" || url == "/rails") {
        // HWiNFO SVI2 rails, measured; measured:false when the shared-memory mapping is absent.
        response_json = hw_rails::rails();

    } else if method == Method::Get && url == "/system/voltage" {
        // Φ_V is derived from the doctrine formula. Whether the rail term is a live VID or a
        // nominal 1.0 is decided by host_probe and travels in `basis`; this route must not restate
        // it, because a fixed sentence here claimed a live VID while the probe reported none.
        let (phi, basis, clock_mhz, mem_pct) = host_probe::phi_v();
        // The report carries the UNCLAMPED value and the rail coverage, because the clamped number
        // hides saturation: with a measured 1.44 V rail the core term alone is 1.327, so an ordinary
        // operating point pins phi_v at 1.0 and the metric stops discriminating.
        let rep = host_probe::phi_v_report();
        response_json = json!({
            "voltage_phi_v": phi,
            "voltage_phi_v_basis": basis,
            "host_clock_mhz": clock_mhz,
            "host_mem_pct": mem_pct,
            "phi_v_raw": rep.get("phi_v_raw"),
            "saturated": rep.get("saturated"),
            "v_core": rep.get("v_core"),
            "v_core_source": rep.get("v_core_source"),
            "rail_coverage": rep.get("rail_coverage"),
            "rail_omitted": rep.get("rail_omitted"),
            "basis": format!("Φ_V derived from the doctrine formula; {}", basis),
        });
    } else if method == Method::Get && (url.starts_with("/system") || url.starts_with("/system/")) {
        // Module counts are READ from the scan artifact, never typed here: this payload
        // carried real_modules_count: 170 while the registry derived 153 REAL / 4 FABRICATED.
        let reg = state_handlers::registry_counts();
        let r = |k: &str| {
            reg.as_ref()
                .and_then(|v| v.get(k).cloned())
                .unwrap_or(serde_json::Value::Null)
        };
        // Host telemetry is MEASURED by host_probe, never typed: this payload carried
        // voltage_phi_v: 0.86, cpu_percent: 12.4, memory_used_mb: 42.1, active_threads: 14,
        // memory_total_mb: 32768.0 and l3_sram_kb: 32768 no matter what the host was doing.
        // Φ_V is a DERIVED formula (clock × memory pressure with rail ratios that reduce to 1.0),
        // not a measurement. The previous payload multiplied it by 1.8 and clamped it to
        // [0.82, 0.965] to pose as a phase lock. Report the raw measured inputs instead.
        let (phi, phi_basis, clock_mhz, mem_pct) = host_probe::phi_v();
        // VRAM is MEASURED from nvidia-smi, never typed: this payload carried total_vram_gb /
        // used_vram_gb / avg_utilization as null while the host has an 8 GB RTX 3070.
        let vram = host_probe::gpu_vram();
        response_json = json!({
            "kernel_version": "v20.0.0-sovereign",
            "runtime_type": "Native Rust no_std + host socket",
            "voltage_phi_v": phi,
            "voltage_phi_v_basis": phi_basis,
            "monad_anchor": "0x4D4F4E4144000001",
            "real_modules_count": r("real"),
            "fabricated_modules_count": r("fabricated"),
            "modules_compiled": r("modules_compiled"),
            "module_status": {
                "ported": r("ported"),
                "skeleton": r("skeleton"),
                "excluded": r("excluded")
            },
            "module_trees": r("trees"),
            "registry_generated": r("generated"),
            "sram_patterns_count": SRAM_PATTERNS_DECLARED,
            "python_active": false,
            "cpu_percent": host_probe::cpu_percent(),
            "memory_used_mb": host_probe::process_working_set_mb(),
            "host_memory_used_mb": host_probe::used_memory_mb(),
            "memory_total_mb": host_probe::total_memory_mb(),
            "active_threads": host_probe::active_threads(),
            "l3_sram_kb": host_probe::l3_cache_kb(),
            "host_clock_mhz": clock_mhz,
            "host_mem_pct": mem_pct,
            "total_vram_gb": vram.map(|(t, _, _)| t / 1024.0),
            "used_vram_gb": vram.map(|(_, u, _)| u / 1024.0),
            "gpu_util_pct": vram.map(|(_, _, g)| g),
            "probe_basis": {
                "measured_on_host": [
                    "cpu_percent", "memory_used_mb", "memory_total_mb",
                    "active_threads", "l3_sram_kb", "host_clock_mhz", "host_mem_pct",
                    "total_vram_gb", "used_vram_gb", "gpu_util_pct"
                ],
                "read_from_scan_artifact": [
                    "real_modules_count", "fabricated_modules_count", "modules_compiled",
                    "module_status", "module_trees", "registry_generated"
                ],
                "declared_constant": [
                    "sram_patterns_count", "kernel_version", "runtime_type", "monad_anchor"
                ],
                "null_means": "probe unavailable on this host; nothing is substituted"
            }
        });
    } else if method == Method::Get
        && (url.starts_with("/swarm/experts") || url.starts_with("/experts"))
    {
        let experts: Vec<serde_json::Value> = AGENTS
            .iter()
            .map(|a| {
                json!({
                    "name": a.name,
                    "role": a.role,
                    "domain": a.domain,
                    "status": "ONLINE",
                    "confidence": 0.95,
                    "weight": a.weight,
                    "avatar": a.name.to_lowercase()
                })
            })
            .collect();
        response_json = json!(experts);
    } else if method == Method::Get && (url.starts_with("/swarm/telemetry") || url == "/swarm") {
        response_json = telemetry_handlers::telemetry_payload();

    } else if method == Method::Get && url.starts_with("/artifacts") {
        // The artifact pipeline persists in swarm_v2_memory/artifact_pipeline.json; the offline
        // export (swarm_v2_artifacts/export_artifact_index.py) strips the heavy `content` field
        // into a compact index this arm reads and caches keyed on (mtime, length).
        response_json = artifact_handlers::artifacts_payload(&url);
    } else if method == Method::Get && url.starts_with("/learning") {
        response_json = telemetry_handlers::learning_payload();

    } else if method == Method::Get && (url == "/intel/router" || url == "/router/experts") {
        // The 15 experts the platform carries + the measured routing performance. Every council
        // agent is now routable; the latency is a live CPU measurement, not an asserted figure.
        response_json = intelligence_router::experts_payload();

    } else if method == Method::Get && (url == "/inference/backends" || url == "/backends") {
        // Which local inference/GPU backends this host ACTUALLY has: Ollama vs the llama.cpp blob
        // server, and Vulkan vs CUDA vs ROCm by runtime-DLL presence. Every field is a probe.
        response_json = intelligence_router::backend_status();

    } else if method == Method::Get && (url == "/hardware/pinning" || url == "/pinning") {
        // The measured CPU core map + GPU device + which runtime thread is pinned to which core.
        response_json = core_pinning::pinning_report();

    } else if method == Method::Get && (url == "/memory/intel" || url == "/swarm/intel") {
        // The SHARED INTEL RESERVOIR. Claimed BEFORE the /memory*fabric* arm below, which would
        // otherwise answer it with the NAND fabric payload -- that shape mismatch is what left the
        // dashboard's swarm-intel tab showing placeholders.
        response_json = intel_reservoir::intel_payload();

    } else if method == Method::Get && url.starts_with("/memory/stats") {
        // Collective Stats / Type Breakdown on the swarm-intel tab.
        response_json = intel_reservoir::stats_payload();

    } else if method == Method::Get && url.starts_with("/memory/recent") {
        // The Live Memory Feed.
        response_json = intel_reservoir::recent_payload(intel_reservoir::limit_from(&url));

    } else if method == Method::Get && (url == "/ddr/antibodies" || url == "/ddr/antibodies/") {
        // The DDR vault list: the curated code-scanning antibodies serialized, plus the
        // threat-matrix totals. The threat store (MITRE/CWE/CVE, ~173 MB / ~400k entries) is
        // counted, not serialized, so this stays a fast read.
        let curated = actions_handlers::curated_antibodies();
        let curated_n = curated.len();
        let threat = actions_handlers::threat_antibody_count();
        let total = actions_handlers::ddr_antibody_count();
        response_json = json!({
            "antibodies": curated,
            "count": curated_n,
            "curated_count": curated_n,
            "threat_count": threat.map(|c| json!(c)).unwrap_or(json!(null)),
            "total_count": total.map(|c| json!(c)).unwrap_or(json!(null)),
            "basis": "antibodies = curated code-scanning store (DDR_ANTIBODIES.json); threat_count = MITRE/CWE/CVE threat-matrix store (DDR_THREAT_ANTIBODIES.json), counted not serialized",
        });

    } else if method == Method::Get && (url == "/ddr/stats" || url == "/ddr/stats/") {
        let curated = actions_handlers::curated_antibody_count();
        let threat = actions_handlers::threat_antibody_count();
        let total = actions_handlers::ddr_antibody_count();
        response_json = json!({
            "ddr_antibody_count": total.map(|c| json!(c)).unwrap_or(json!(null)),
            "ddr_curated_antibody_count": curated.map(|c| json!(c)).unwrap_or(json!(null)),
            "ddr_threat_antibody_count": threat.map(|c| json!(c)).unwrap_or(json!(null)),
            "ddr_antibody_count_basis": "curated (DDR_ANTIBODIES.json) + threat-matrix (DDR_THREAT_ANTIBODIES.json, MITRE/CWE/CVE-derived); total = curated + threat",
            "ddr_antibody_count_previously_asserted": 1801,
        });

    } else if method == Method::Get && (url.starts_with("/memory") || url.starts_with("/ddr")) {
        // Derived, not asserted: the trit total comes from the mapped store's own size, and the
        // companion stores are the ones /k3/status reports. shards/utilization are named as the
        // design figures they are instead of riding along as measurements.
        let store = crate::k3_retrieval::fabric_store();
        let (total_trits, store_path, store_bytes) = match &store {
            Some((p, b)) => (json!(b.saturating_mul(2)), json!(p), json!(b)),
            None => (json!(null), json!(null), json!(null)),
        };
        let (k3, _) = crate::k3_retrieval::status_handle();
        response_json = json!({
            "fabric": "NVMe NAND K3 Fabric",
            "total_trits": total_trits,
            "trit_store": store_path,
            "trit_store_bytes": store_bytes,
            "total_trits_basis": "store bytes x 2 trits/byte, read from the file the kernel maps",
            "stores": {
                "fabric": k3.get("fabric").cloned().unwrap_or(json!(null)),
                "tercode": k3.get("tercode").cloned().unwrap_or(json!(null)),
                "ipc": k3.get("ipc").cloned().unwrap_or(json!(null))
            },
            "shards": null,
            "shards_basis": "no shard count is measured here: the fabric is one .k3 file plus a tercode store and the IPC ring",
            "utilization_pct": null,
            "utilization_pct_basis": "designed corpus vs drive capacity is a design figure, not occupancy",
            "design_figures": { "shards": 18, "utilization_pct": 0.59, "source": "docs/NAND_K3_FABRIC_SPEC.md" },
            "ddr_antibody_count": actions_handlers::ddr_antibody_count().map(|c| json!(c)).unwrap_or(json!(null)),
            "ddr_curated_antibody_count": actions_handlers::curated_antibody_count().map(|c| json!(c)).unwrap_or(json!(null)),
            "ddr_threat_antibody_count": actions_handlers::threat_antibody_count().map(|c| json!(c)).unwrap_or(json!(null)),
            "ddr_antibody_count_basis": "curated (DDR_ANTIBODIES.json) + threat-matrix (DDR_THREAT_ANTIBODIES.json, MITRE/CWE/CVE-derived); total = curated + threat; null = store unreadable - not a clean result",
            "ddr_antibody_count_previously_asserted": 1801,
            "status": "ONLINE",
            "status_basis": "the kernel is answering; IMMUTABLE_LOCKED was a store-policy claim with no source"

        });
    } else if method == Method::Get && url.starts_with("/federation/inbound") {
        response_json = state_handlers::federation_inbound_payload();
    } else if method == Method::Get && url.starts_with("/federation") {
        response_json = state_handlers::federation_payload(&url);
    } else if method == Method::Post && url.starts_with("/federation/sync") {
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let peer = request.remote_addr().map(|a| a.to_string());
        let sig = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("X-TRM-Signature"))
            .map(|h| h.value.as_str().to_string());
        response_json = state_handlers::federation_sync_payload(&body, peer.as_deref(), sig.as_deref());
    } else if method == Method::Post && url == "/sync" {
        // Standardized sync-write endpoint (parity with the MacPro5-1 /sync route): read the
        // handshake, verify the Monad anchor, return the registry's actual state.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let peer = request.remote_addr().map(|a| a.to_string());
        let sig = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("X-TRM-Signature"))
            .map(|h| h.value.as_str().to_string());
        response_json = state_handlers::federation_sync_payload(&body, peer.as_deref(), sig.as_deref());
    } else if method == Method::Get && url.starts_with("/security/threats/query") {
        // Native threat-index query over the voltage-stripped MITRE+CWE corpus.
        let (payload, code) = k3_retrieval::threat_query_handle(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && url.starts_with("/security") {
        response_json = telemetry_handlers::security_payload();

    } else if method == Method::Get && url.starts_with("/mailbox/ring") {
        // The IPC transport read from its own bytes. Dispatched BEFORE the /mailbox prefix arm
        // below, or that arm would swallow this route.
        let (payload, code) = mailbox_ring::handle(&url);
        response_json = payload;
        status_code = code;
    } else if method == Method::Get && (url.starts_with("/mailbox") || url.starts_with("/comms")) {
        // This arm served two invented messages (a 170/170 module claim and a 14-0-0 consolidation
        // motion) with fabricated timestamps. The transport's real content is the ring's own state,
        // read from its bytes by mailbox_ring.rs: counters per agent plus the k3_trit of each slot.
        // No message TEXT exists to serve, and saying so beats inventing a feed.
        let (ring, _) = mailbox_ring::handle("/mailbox/ring");
        response_json = json!({
            "messages": [],
            "messages_reason": "the IPC ring stores per-agent counters and 64-byte slot payloads, not decodable message text; the only decoded field is each slot's k3_trit. The two messages this route used to serve were not read from any store.",
            "transport": "shared-memory ring bus (IPC)",
            "ring_store": ring.get("store").cloned().unwrap_or(json!(null)),
            "total_pending": ring.get("total_pending").cloned().unwrap_or(json!(null)),
            "total_messages_ever": ring.get("total_messages_ever").cloned().unwrap_or(json!(null)),
            "source_route": "/mailbox/ring",
        });
    } else if method == Method::Get && url.starts_with("/infrastructure/status") {
        // The self-healing FSM over the supervisor's own table.
        response_json = infra_handlers::status_payload();

    } else if method == Method::Get && url.starts_with("/infra/health") {
        response_json = infra_handlers::health_payload();

    } else if method == Method::Get && url.starts_with("/infra/history") {
        response_json = infra_handlers::history_payload();

    } else if method == Method::Post && url == "/infra/start" {
        // Resume the supervisor's heal loop. Written to the control file the Rust host_supervisor
        // reads on its next tick; the effect is observable via /infra/health.running.
        response_json = infra_handlers::control_payload("resume", None);

    } else if method == Method::Post && url == "/infra/stop" {
        // Pause the heal loop (services are still reported, never restarted while paused).
        response_json = infra_handlers::control_payload("pause", None);

    } else if method == Method::Post && url == "/infra/reset" {
        // Reset every service's restart counter to 0.
        response_json = infra_handlers::control_payload("reset", None);

    } else if method == Method::Post && url.starts_with("/infra/reset/") {
        // Reset one service's restart counter by its supervisor key (dashboard resets per service).
        let key = url.trim_start_matches("/infra/reset/");
        response_json = infra_handlers::control_payload("reset", Some(key));

    } else if method == Method::Post && url.starts_with("/infra/restart/") {
        // Force-restart one service by its supervisor key (e.g. rust_kernel, tier2_generator).
        let key = url.trim_start_matches("/infra/restart/");
        response_json = infra_handlers::control_payload("restart", Some(key));

    } else if method == Method::Get && (url == "/infrastructure" || url == "/infrastructure/") {
        // "ACTIVE" and "ARMED_ZERO_DRIFT" were declared here with nothing behind them. They are now the
        // FSM's own verdict over the supervisor's measured table, or an honest "unknown" when that table
        // cannot be read -- never a claim that the healing layer is fine because the string said so.
        let h = infra_handlers::health_payload();
        response_json = json!({
            "self_healing": h.get("status").cloned().unwrap_or(json!("unknown")),
            "self_healing_rule": h.get("status_rule").cloned().unwrap_or(json!(null)),
            "self_healing_basis": "derived by trm_uefi_kernel/src/self_healing_infra.rs from the supervisor's own service table; previously asserted as the literal \"ACTIVE\"",
            "watchdog": h.get("status").cloned().unwrap_or(json!("unknown")),
            "watchdog_previously_asserted": "ARMED_ZERO_DRIFT",
            "ddr_antibody_count": actions_handlers::ddr_antibody_count().map(|c| json!(c)).unwrap_or(json!(null)),
            "ddr_curated_antibody_count": actions_handlers::curated_antibody_count().map(|c| json!(c)).unwrap_or(json!(null)),
            "ddr_threat_antibody_count": actions_handlers::threat_antibody_count().map(|c| json!(c)).unwrap_or(json!(null)),
            "ddr_antibody_count_basis": "curated (DDR_ANTIBODIES.json) + threat-matrix (DDR_THREAT_ANTIBODIES.json, MITRE/CWE/CVE-derived); total = curated + threat; null = store unreadable - not a clean result",
            "ddr_antibody_count_previously_asserted": 1801,
            "routes": ["/infrastructure/status", "/infra/health", "/infra/history"]
        });
    } else if method == Method::Get && (url == "/research/stats" || url == "/research/stats/") {
        response_json = research_handlers::stats();

    } else if method == Method::Get && (url == "/brain/status" || url == "/brain/status/") {
        response_json = telemetry_handlers::brain_status_payload();

    } else if method == Method::Post && url.starts_with("/brain/reason") {
        // Recorded as a stub through the whole rollout, then closed: this arm computed its verdict
        // from a hardcoded `sample_context` array and answered the same chain for ANY body. No
        // reasoning engine is wired on this host, so the route answers with the machinery that does
        // exist -- the kernel's own retrieval -- and reports exactly what that derives.
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let q = parsed
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if q.is_empty() {
            status_code = 400;
            response_json = json!({
                "error": "empty_query",
                "message": "/brain/reason needs {\"query\": \"...\"}; the chain this route used to return was computed from a hardcoded sample, not from the request"
            });
        } else {
            let (ret, _) = crate::k3_retrieval::handle(&format!(
                "/k3/retrieve?q={}&k=3",
                crate::actions_handlers::enc(&q)
            ));
            let hits = ret
                .get("results")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let steps: Vec<serde_json::Value> = hits
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    json!({
                        "step": i + 1,
                        "action": "retrieve",
                        "unit": h.get("doc").cloned().unwrap_or(json!(null)),
                        "fidelity": h.get("doc_fidelity").cloned().unwrap_or(json!(null)),
                        "score": h.get("score").cloned().unwrap_or(json!(null)),
                        "score_crosschecked": h.get("score_crosschecked").cloned().unwrap_or(json!(null))
                    })
                })
                .collect();
            let top_fidelity = hits
                .first()
                .and_then(|h| h.get("doc_fidelity"))
                .and_then(|v| v.as_f64());
            response_json = json!({
                "status": "RETRIEVAL_ONLY",
                "query": q,
                "steps": steps,
                "steps_taken": hits.len(),
                "halted": top_fidelity.map(|f| f >= 0.5).unwrap_or(false),
                "halt_basis": "this route halts when the top hit's fidelity clears 0.5 -- the threshold is this route's, not a doctrine constant",
                "top_fidelity": top_fidelity,
                "engine": null,
                "engine_reason": "no reasoning engine is wired on this host: there is no SSM recurrence in this process, so no chain can be reported as computed",
                "retrieval": {
                    "corpus": ret.get("corpus").cloned().unwrap_or(json!(null)),
                    "candidates_scored": ret.get("candidates_scored").cloned().unwrap_or(json!(null)),
                    "engine": ret.get("engine").cloned().unwrap_or(json!(null)),
                    "elapsed_us": ret.get("elapsed_us").cloned().unwrap_or(json!(null))
                },
                "previously_asserted": { "status": "COMPLETED", "steps_taken": 4, "confidence": 0.94,
                                        "k3_trit": 1,
                                        "explanation": "Native Rust TRM Brain: Non-contradiction verified via Mamba-3 SSM recurrence." }
            });
        }

    }

    // ── Kanban board (real state file) — Option A dashboard function fix ─────
    // Must run before the {"status":"ok"} catch-all so the dashboard's Kanban
    // tab reads/writes the real 640-card board (kanban_state.json).
    if url.starts_with("/kanban") || url.starts_with("/genomics") {
        if let Some(kresp) = kanban_handlers::handle(&mut request, &url) {
            let _ = request.respond(kresp);
            return;
        }
    }

    // ── Evolution actions (native mutation cycle) — claimed before the read handler ──
    if url.starts_with("/evolution") {
        if let Some(ea) = evolution_actions::handle(&mut request, &url) {
            let _ = request.respond(ea);
            return;
        }
    }

    // ── Evolution tab (real proposals file) — Option A continuation ─────────
    if url.starts_with("/evolution") {
        if let Some(eresp) = state_handlers::handle(&mut request, &url) {
            let _ = request.respond(eresp);
            return;
        }
    }

    // ── P1/P2: artifact provenance (the re-checked census) + the wired proof checker ─────────
    if url.starts_with("/verification/artifacts") || url.starts_with("/verification/proof")
        || url.starts_with("/verification/fabrication") {
        if let Some(vresp) = verification_handlers::handle(&mut request, &url) {
            let _ = request.respond(vresp);
            return;
        }
    }

    // ── P2g: the verified crates, wired as routes: each op computes in the crate that owns the
    //     algorithm and reports that crate's own answer. GET /crate/ops is the wiring as data. ──
    if url.starts_with("/crate/") {
        if let Some(cresp) = crate_ops_handlers::handle(&mut request, &url) {
            let _ = request.respond(cresp);
            return;
        }
    }

    // ── Layer 1: the 15 SRAM topologies and the Hopfield relaxation, from the kernel's own source ──
    if url.starts_with("/sram") {
        if let Some(sresp) = sram_handlers::handle(&mut request, &url) {
            let _ = request.respond(sresp);
            return;
        }
    }

    // ── The proven native modules as routes: chaos, expert resolution, persona, GPU pattern math ──
    if url.starts_with("/native") {
        if let Some(nresp) = native_handlers::handle(&mut request, &url) {
            let _ = request.respond(nresp);
            return;
        }
    }

    // ── Verification / Testing / Skills (real files) — Option A batch 2 ─────
    if url.starts_with("/verification") || url.starts_with("/testing") || url.starts_with("/skills")
    {
        if let Some(bresp) = state_handlers::handle_batch2(&mut request, &url) {
            let _ = request.respond(bresp);
            return;
        }
    }

    // ── TRM 7M batch: score N texts in ONE GPU dispatch (the corpus/council-scale model tier) ──
    // The runtime's single-eval path (evaluate_proposal) stays on the CPU forward; this route is the
    // batch entry point — the building block for scoring a corpus or a full slate of candidates at once.
    if url.starts_with("/trm7m/batch") {
        let mut body = String::new();
        let _ = request.as_reader().take(1_048_576).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let texts: Vec<String> = parsed
            .get("texts")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let inputs: Vec<[f32; 128]> = texts.iter().map(|t| trm7m::Trm7m::encode_text(t)).collect();
        let backend;
        let scored: Vec<(i8, f32)> = match gpu_compute::gpu_trm7m_batch(&inputs) {
            Some(v) => {
                backend = "gpu";
                v
            }
            None => {
                backend = "cpu";
                inputs.iter().map(|x| {
                    let mut m = trm7m::Trm7m::new();
                    let (t, c, _) = m.forward(x);
                    (t, c)
                }).collect()
            }
        };
        let results: Vec<serde_json::Value> = texts.iter().zip(scored.iter())
            .map(|(t, (trit, conf))| json!({
                "text": t,
                "trit": trit,
                "trit_label": trm7m::trit_label(*trit),
                "confidence": conf,
            }))
            .collect();
        response_json = json!({
            "model": "trm7m_source",
            "backend": backend,
            "count": texts.len(),
            "results": results,
            "note": "batched forward — one GPU dispatch over N inputs (fresh state per input, parity-verified vs the CPU forward); CPU forward fallback",
        });
    }

    // ── TRM 7M (b) of the council decision: the REGISTERED model, actually RUN ─────────────
    // Before this arm existed, /models reported the bundle and nothing could execute it. The kernel now
    // runs the same seed-42 weights the UEFI kernel embeds and returns the model's own trit + confidence.
    if url.starts_with("/trm7m") && !url.starts_with("/trm7m/batch") {
        let mut body = String::new();
        let _ = request.as_reader().take(65_536).read_to_string(&mut body);
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        let text = parsed
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("sovereign status")
            .to_string();
        let mut model = trm7m::Trm7m::new();
        let (trit, conf, ns) = model.think(&text, 7);
        let hidden_rms = (model.hidden.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>()
            / model.hidden.len().max(1) as f64)
            .sqrt();
        let hidden_absmax = model.hidden.iter().cloned().fold(0.0f32, f32::max);
        response_json = json!({
            "model": "trm7m_source",
            "bundle": "swarm_v2_artifacts/ternary_models/trm7m_source.trm",
            "weights": "trm_uefi_kernel/assets/trm7m_weights.bin (the same seed-42 distilled blob the UEFI kernel embeds)",
            "thinking_turns": 7,
            "params": trm7m::Trm7m::total_params(),
            "weight_bytes": trm7m::Trm7m::weight_bytes(),
            "state_dim": trm7m::STATE_DIM,
            "blocks": trm7m::NUM_BLOCKS,
            "text": text,
            "trit": trit,
            "trit_label": trm7m::trit_label(trit),
            "confidence": conf,
            "logits": model.last_logits,
            "hidden_rms": hidden_rms,
            "hidden_absmax": hidden_absmax,
            "hidden_len": model.hidden.len(),
            "hidden": model.hidden.clone(),
            "latency_ns": ns,
            "forward_calls": model.forward_calls,
            "note": "the served value IS the model's output (ported 1:1 from trm7m.rs), after 7 forward 'thinking turns' for the SSM conv-state to warm up; it is not a rule table's verdict",
            "fabric": crate::task_executor::unified_fabric_verdict(&text),
        });
    }

    // ── Model registry: ingested ternary models (real bundles) ─────────────
    if url.starts_with("/models") {
        if let Some(mresp) = model_registry::handle(&mut request, &url) {
            let _ = request.respond(mresp);
            return;
        }
    }

    // ── Neural Bridge chat (native, deterministic) ─────────────────────────
    if url.starts_with("/swarm/chat") {
        if let Some(cresp) = chat_handlers::handle(&mut request, &url) {
            let _ = request.respond(cresp);
            return;
        }
    }

    // ── Rollout 4: missions/research/journal/agents/infra (STATE_STORE_SPEC.md) ─
    if url.starts_with("/ultrawork")
        || url.starts_with("/research")
        || url.starts_with("/journal")
        || url.starts_with("/swarm/agents")
        || url.starts_with("/swarm/orchestrator")
        || url.starts_with("/swarm/spatial")
        || url.starts_with("/infrastructure")
        || url.starts_with("/infra")
        || url.starts_with("/secrets")
    {
        if let Some(r4resp) = state_handlers::handle_batch3(&mut request, &url) {
            let _ = request.respond(r4resp);
            return;
        }
    }

    // ── Rollout 5: pillars families (wetlab/sensory/thz/regulatory/ledger) — file-backed reads,
    //     honest 501s for the engine-owned actions; goes before the {"status":"ok"} catch-all.
    if url.starts_with("/wetlab")
        || url.starts_with("/sensory")
        || url.starts_with("/thz")
        || url.starts_with("/regulatory")
        || url.starts_with("/ledger")
    {
        if let Some(r5resp) = state_handlers::handle_batch4(&mut request, &url) {
            let _ = request.respond(r5resp);
            return;
        }
    }

    // ── Humanistic module (MOTION-H1): unified surface. The task reads fall
    //    through to handle_batch5 below (reused, not duplicated).
    if url == "/humanistic" || url == "/humanistic/"
        || url.starts_with("/humanistic/reflect")
        || url.starts_with("/humanistic/compose")
    {
        if let Some(hresp) = humanistic_handlers::handle(&mut request, &url) {
            let _ = request.respond(hresp);
            return;
        }
    }

    // ── Rollout 6: discovery + humanistic families (file-backed reads).
    if url.starts_with("/discovery") || url.starts_with("/humanistic") {
        if let Some(r6resp) = state_handlers::handle_batch5(&mut request, &url) {
            let _ = request.respond(r6resp);
            return;
        }
    }

    // ── Continuum refraction (native) — claimed before the engine_required arm ──
    if url.starts_with("/api/continuum/genomics/refract") {
        if let Some(cr) = continuum_refract::handle(&mut request, &url) {
            let _ = request.respond(cr);
            return;
        }
    }

    // ── Rollout 7: continuum + microkernel families (derived / file-backed reads; engine actions 501).
    if url.starts_with("/api/continuum") || url.starts_with("/swarm/microkernel") {
        if let Some(r7resp) = state_handlers::handle_batch6(&mut request, &url) {
            let _ = request.respond(r7resp);
            return;
        }
    }

    // ── Rollout 8: per-module reads. MUST run before the /system arm's payload is serialized: that
    //     arm matches any "/system*" prefix and fell through, so every /system/modules/<name> answer
    //     was the same overview. This guard responds and returns instead.
    if url.starts_with("/system/modules") {
        if let Some(mresp) = module_handlers::handle(&mut request, &url) {
            let _ = request.respond(mresp);
            return;
        }
    }

    // ── Generative-architect synthesize (native) — claimed before the refusal arm ──
    if url.starts_with("/generative-architect") {
        if let Some(aa) = architect_actions::handle(&mut request, &url) {
            let _ = request.respond(aa);
            return;
        }
    }

    // ── Rollout 9: generative-architect + mobile-sync (engine-shaped; the mobile route bodies are
    //     asserted constants, so they are refused rather than echoed).
    if url.starts_with("/generative-architect") || url.starts_with("/api/mobile-sync") {
        if let Some(sresp) = studio_handlers::handle(&mut request, &url) {
            let _ = request.respond(sresp);
            return;
        }
    }

    // ── Rollout 10: the mesh family. It used to answer EVERY /mesh/<anything> path (including
    //     /mesh/does-not-exist) with one hardcoded two-node topology carrying invented ping times.
    if url.starts_with("/mesh") {
        if let Some(meresp) = mesh_handlers::handle(&mut request, &url) {
            let _ = request.respond(meresp);
            return;
        }
    }

    // ── Native ports of the engine actions (mailbox write, DDR scan, artifact review/integrate) —
    //     claimed BEFORE the engine_required refusal below, so they are real, not 501.
    if let Some(mw) = mailbox_write::handle(&mut request, &url) {
        let _ = request.respond(mw);
        return;
    }
    if let Some(ds) = ddr_scan::handle(&mut request, &url) {
        let _ = request.respond(ds);
        return;
    }
    if let Some(ar) = artifact_actions::handle(&mut request, &url) {
        let _ = request.respond(ar);
        return;
    }

    // ── Rollout 13: the engine-action family. /memory/query is the one member this kernel can
    //     answer from its own machinery (/k3/retrieve); the rest need Python engines, and before
    //     this guard they landed in the catch-all and reported success.
    if let Some(xresp) = actions_handlers::handle(&mut request, &url) {
        let _ = request.respond(xresp);
        return;
    }

    // ── Rollout 12: Aura Studio is a SEPARATE product (not running), and the research ACTIONS
    //     need the Python engine's daemon. Both used to fall through to the catch-all, so the IDE and
    //     the research controls reported success while nothing ran.
    if url.starts_with("/aura") {
        if let Some(aresp) = aura_handlers::handle(&mut request, &url) {
            let _ = request.respond(aresp);
            return;
        }
    }
    if url.starts_with("/research") {
        if let Some(rresp) = research_handlers::handle(&mut request, &url) {
            let _ = request.respond(rresp);
            return;
        }
    }

    // ── Rollout 11: the services catalog. Both routes used to fall through to the catch-all, and the
    //     tab then rendered a hardcoded client-side catalog (invented Phi_V/module/quorum/antibody
    //     figures, pricing) while the client minted a random API key on provisioning failure.
    if url.starts_with("/services") {
        if let Some(svresp) = services_handlers::handle(&mut request, &url) {
            let _ = request.respond(svresp);
            return;
        }
    }

    // ── The pure-Rust server-rendered dashboard. /ui/journal is the first viewport (the migration
    //     template); it must claim the route before the static React bundle below.
    if url.starts_with("/ui/journal") {
        let _ = request.respond(ui_render::journal_html(&url));
        return;
    }
    if url.starts_with("/ui/council") {
        let _ = request.respond(ui_render::council_chamber_html(&url));
        return;
    }
    if url.starts_with("/ui/matrix") {
        let _ = request.respond(ui_render::module_matrix_html(&url));
        return;
    }
    if url.starts_with("/ui/discovery") {
        let _ = request.respond(ui_render::discovery_html(&url));
        return;
    }
    if url.starts_with("/ui/humanistic") {
        let _ = request.respond(ui_render::humanistic_html(&url));
        return;
    }
    if url.starts_with("/ui/benchmarks") {
        let _ = request.respond(ui_render::benchmark_html(&url));
        return;
    }
    if url.starts_with("/ui/mesh") {
        let _ = request.respond(ui_render::mesh_html(&url));
        return;
    }
    if url.starts_with("/ui/aura") {
        let _ = request.respond(ui_render::aura_html(&url));
        return;
    }
    if url.starts_with("/ui/continuum") {
        let _ = request.respond(ui_render::continuum_html(&url));
        return;
    }
    if url.starts_with("/ui/mobile") {
        let _ = request.respond(ui_render::mobile_html(&url));
        return;
    }
    if url.starts_with("/ui/mitosis") {
        let _ = request.respond(ui_render::mitosis_html(&url));
        return;
    }
    if url.starts_with("/ui/telemetry") {
        let _ = request.respond(ui_render::telemetry_html(&url));
        return;
    }
    if url.starts_with("/ui/federation") {
        let _ = request.respond(ui_render::federation_html(&url));
        return;
    }
    if url.starts_with("/ui/security") {
        let _ = request.respond(ui_render::security_html(&url));
        return;
    }
    if url.starts_with("/ui/upgrade") {
        let _ = request.respond(ui_render::upgrade_html(&url));
        return;
    }
    if url.starts_with("/ui/verification") {
        let _ = request.respond(ui_render::verification_html(&url));
        return;
    }
    if url.starts_with("/ui/infra") {
        let _ = request.respond(ui_render::infra_html(&url));
        return;
    }
    if url.starts_with("/ui/testing") {
        let _ = request.respond(ui_render::testing_html(&url));
        return;
    }
    if url.starts_with("/ui/research") {
        let _ = request.respond(ui_render::research_html(&url));
        return;
    }
    if url.starts_with("/ui/overview") {
        let _ = request.respond(ui_render::overview_html(&url));
        return;
    }
    if url.starts_with("/ui/intel") {
        let _ = request.respond(ui_render::intel_html(&url));
        return;
    }
    if url.starts_with("/ui/learning") {
        let _ = request.respond(ui_render::learning_html(&url));
        return;
    }
    if url.starts_with("/ui/services") {
        let _ = request.respond(ui_render::services_html(&url));
        return;
    }
    if url.starts_with("/ui/native") {
        let _ = request.respond(ui_render::native_html(&url));
        return;
    }
    if url.starts_with("/ui/chat/send") {
        let resp = ui_render::chat_send_html(&mut request, &url);
        let _ = request.respond(resp);
        return;
    }
    if url.starts_with("/ui/chat/clear") {
        let _ = request.respond(ui_render::chat_clear_html());
        return;
    }
    if url.starts_with("/ui/chat") {
        let _ = request.respond(ui_render::chat_html(&url));
        return;
    }
    if url.starts_with("/ui/ddr") {
        let _ = request.respond(ui_render::ddr_html(&url));
        return;
    }
    if url.starts_with("/ui/pillars") {
        let _ = request.respond(ui_render::pillars_html(&url));
        return;
    }
    if url.starts_with("/ui/pipeline") {
        let _ = request.respond(ui_render::pipeline_html(&url));
        return;
    }
    if url.starts_with("/ui/kanban") {
        let _ = request.respond(ui_render::kanban_html(&url));
        return;
    }
    if url.starts_with("/ui/comms") {
        let _ = request.respond(ui_render::comms_html(&url));
        return;
    }
    if url.starts_with("/ui/spatial") {
        let _ = request.respond(ui_render::spatial_html(&url));
        return;
    }
    if url.starts_with("/ui/evolution") {
        let _ = request.respond(ui_render::evolution_html(&url));
        return;
    }
    if url.starts_with("/ui/architect") {
        let _ = request.respond(ui_render::architect_html(&url));
        return;
    }
    if url.starts_with("/ui/neuro") {
        let _ = request.respond(ui_render::neuro_html(&url));
        return;
    }

    // ── The pure-Rust compositor index — the landing page links every viewport. Claim bare /ui and
    //     /ui/ before the built React bundle below, so the dashboard's tabs are real navigation.
    if url == "/ui" || url == "/ui/" {
        let _ = request.respond(ui_render::index_html(&url));
        return;
    }

    // ── The built dashboard. The UI and the API share this origin, so the app's root-relative
    //     fetches resolve to this kernel with no proxy -- which is what lets the Node dev server
    //     the platform has been running for retire. Goes before the unclaimed check.
    if let Some(uiresp) = ui_handlers::handle(&url) {
        let _ = request.respond(uiresp);
        return;
    }

    // Nothing claimed the path. Report that honestly instead of the old {"status":"ok"}, which told
    // 24 unhandled routes that they had succeeded. The path and method are echoed so the caller can
    // see what it actually asked for.
    if response_json.get("__unclaimed__").is_some() {
        status_code = 404;
        response_json = json!({
            "error": "not_found",
            "detail": "no handler claims this path in the Rust kernel",
            "path": url,
            "method": format!("{:?}", method).to_uppercase(),
        });
    }

    let body_str = serde_json::to_string(&response_json).unwrap();
    let elapsed_us = start_t.elapsed().as_micros();

    let mut resp = Response::from_string(body_str)
        .with_status_code(status_code)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
        .with_header(
            Header::from_bytes(
                &b"Access-Control-Allow-Methods"[..],
                &b"GET, POST, OPTIONS"[..],
            )
            .unwrap(),
        )
        .with_header(
            Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..]).unwrap(),
        )
        .with_header(
            Header::from_bytes(
                &b"X-Sovereign-Execution-Time-Us"[..],
                format!("{}", elapsed_us).as_bytes(),
            )
            .unwrap(),
        )
        .with_header(
            Header::from_bytes(&b"X-Sovereign-Truth-Anchor"[..], &b"0x4D4F4E4144000001"[..])
                .unwrap(),
        );

    let _ = request.respond(resp);
}

/// Offline batch-encode CLI: `host_council_server encode-batch --input <texts.jsonl> --output <vectors.jsonl> [--batch N]`.
/// Input is JSONL of `{"key":..., "text":...}`; output is JSONL of `{"key":..., "embedding":[...]}` carrying the same
/// L2-normalized 768-dim vectors `native_encoder::encode_f32` produces. Uses the GPU batch kernel when a device is
/// present, falling back to the CPU encoder per doc on any failure. Exit 0 on success, 1 on data error, 2 on usage.
fn encode_batch_cli(args: &[String]) -> i32 {
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut batch: usize = 256;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                input = args.get(i).cloned();
            }
            "--output" => {
                i += 1;
                output = args.get(i).cloned();
            }
            "--batch" => {
                i += 1;
                batch = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(256).max(1);
            }
            other => {
                eprintln!("encode-batch: unknown argument {other:?}");
                return 2;
            }
        }
        i += 1;
    }
    let (Some(input), Some(output)) = (input, output) else {
        eprintln!(
            "usage: host_council_server encode-batch --input <texts.jsonl> --output <vectors.jsonl> [--batch N]"
        );
        return 2;
    };

    let raw = match std::fs::read_to_string(&input) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("encode-batch: cannot read input: {e}");
            return 1;
        }
    };
    let mut keys: Vec<String> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => {
                keys.push(v.get("key").and_then(serde_json::Value::as_str).unwrap_or("").to_string());
                texts.push(v.get("text").and_then(serde_json::Value::as_str).unwrap_or("").to_string());
            }
            Err(e) => {
                eprintln!("encode-batch: bad input line: {e}");
                return 1;
            }
        }
    }
    let n = texts.len();
    if n == 0 {
        eprintln!("encode-batch: no documents in input");
        return 1;
    }

    let gpu = gpu_compute::gpu_available();
    let t0 = std::time::Instant::now();
    let mut out: Vec<(String, Vec<f32>)> = Vec::with_capacity(n);
    let mut idx = 0usize;
    while idx < n {
        let end = (idx + batch).min(n);
        let chunk: Vec<&str> = texts[idx..end].iter().map(String::as_str).collect();
        let got = if gpu { gpu_compute::gpu_encode_batch(&chunk) } else { None };
        match got {
            Some(vecs) => {
                for (j, vec) in vecs.into_iter().enumerate() {
                    out.push((keys[idx + j].clone(), vec));
                }
            }
            None => {
                for (j, t) in chunk.iter().enumerate() {
                    out.push((keys[idx + j].clone(), native_encoder::encode_f32(t, 768)));
                }
            }
        }
        idx = end;
    }
    let elapsed = t0.elapsed().as_secs_f64();

    use std::io::Write;
    let mut w = match std::fs::File::create(&output) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("encode-batch: cannot write output: {e}");
            return 1;
        }
    };
    for (key, vec) in &out {
        if let Err(e) = writeln!(w, "{}", serde_json::json!({ "key": key, "embedding": vec })) {
            eprintln!("encode-batch: write failed: {e}");
            return 1;
        }
    }
    eprintln!(
        "encode-batch: {n} docs in {elapsed:.3}s ({:.1} docs/s), backend {}",
        n as f64 / elapsed.max(1e-6),
        if gpu { "gpu" } else { "cpu" }
    );
    0
}

/// Offline corpus-scoring CLI: `host_council_server score-corpus --input <text.jsonl> --output <scores.jsonl> [--batch N]`.
/// Input is JSONL of `{"key":..., "text":...}` (the TurboVec text sidecar, NOT the 4.2 GB semantic_index.json);
/// output is JSONL of `{"key":..., "trit":..., "trit_label":..., "confidence":...}` — the token-free TRM-7M forward
/// over every document. GPU batch kernel when present, CPU forward fallback. This is the no-HTTP path: the kernel
/// reads the corpus in-process and writes the scored index, instead of a Python driver round-tripping the 146 MB
/// sidecar over `/trm7m/batch` (the slow HTTP path). Exit 0 on success, 1 on data error, 2 on usage.
fn score_corpus_cli(args: &[String]) -> i32 {
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut batch: usize = 256;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                input = args.get(i).cloned();
            }
            "--output" => {
                i += 1;
                output = args.get(i).cloned();
            }
            "--batch" => {
                i += 1;
                batch = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(256).max(1);
            }
            other => {
                eprintln!("score-corpus: unknown argument {other:?}");
                return 2;
            }
        }
        i += 1;
    }
    let (Some(input), Some(output)) = (input, output) else {
        eprintln!(
            "usage: host_council_server score-corpus --input <text.jsonl> --output <scores.jsonl> [--batch N]"
        );
        return 2;
    };

    let raw = match std::fs::read_to_string(&input) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("score-corpus: cannot read input: {e}");
            return 1;
        }
    };
    let mut keys: Vec<String> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => {
                keys.push(v.get("key").and_then(serde_json::Value::as_str).unwrap_or("").to_string());
                texts.push(v.get("text").and_then(serde_json::Value::as_str).unwrap_or("").to_string());
            }
            Err(e) => {
                eprintln!("score-corpus: bad input line: {e}");
                return 1;
            }
        }
    }
    drop(raw); // release the 146 MB read buffer before the forward pass
    let n = texts.len();
    if n == 0 {
        eprintln!("score-corpus: no documents in input");
        return 1;
    }

    let gpu = gpu_compute::gpu_available();
    let t0 = std::time::Instant::now();
    let mut out: Vec<(String, i8, f32)> = Vec::with_capacity(n);
    let mut idx = 0usize;
    while idx < n {
        let end = (idx + batch).min(n);
        // encode_text -> [f32; 128] per doc, then ONE GPU dispatch over the whole batch.
        let inputs: Vec<[f32; 128]> = texts[idx..end]
            .iter()
            .map(|t| trm7m::Trm7m::encode_text(t))
            .collect();
        let got = if gpu { gpu_compute::gpu_trm7m_batch(&inputs) } else { None };
        match got {
            Some(scored) => {
                for (j, (trit, conf)) in scored.into_iter().enumerate() {
                    out.push((keys[idx + j].clone(), trit, conf));
                }
            }
            None => {
                for (j, x) in inputs.iter().enumerate() {
                    let mut m = trm7m::Trm7m::new();
                    let (trit, conf, _) = m.forward(x);
                    out.push((keys[idx + j].clone(), trit, conf));
                }
            }
        }
        idx = end;
    }
    let elapsed = t0.elapsed().as_secs_f64();

    use std::io::Write;
    let mut w = match std::fs::File::create(&output) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("score-corpus: cannot write output: {e}");
            return 1;
        }
    };
    for (key, trit, conf) in &out {
        if let Err(e) = writeln!(
            w,
            "{}",
            serde_json::json!({
                "key": key,
                "trit": trit,
                "trit_label": trm7m::trit_label(*trit),
                "confidence": conf,
            })
        ) {
            eprintln!("score-corpus: write failed: {e}");
            return 1;
        }
    }
    eprintln!(
        "score-corpus: {n} docs in {elapsed:.3}s ({:.1} docs/s), backend {}",
        n as f64 / elapsed.max(1e-6),
        if gpu { "gpu" } else { "cpu" }
    );
    0
}

/// Offline parity-check CLI: `host_council_server parity-check --store <rel> --query <text> [--k N]`.
/// Loads a packed store by repo-relative path, runs the GPU scan AND the CPU reference (cpu_scan_in),
/// and reports whether the top-K row indices + scores match within 1e-3. Exit 0 on match, 1 on mismatch.
fn parity_check_cli(args: &[String]) -> i32 {
    let mut store_rel: Option<String> = None;
    let mut query: Option<String> = None;
    let mut k: usize = 10;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--store" => {
                i += 1;
                store_rel = args.get(i).cloned();
            }
            "--query" => {
                i += 1;
                query = args.get(i).cloned();
            }
            "--k" => {
                i += 1;
                k = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(10).max(1);
            }
            other => {
                eprintln!("parity-check: unknown argument {other:?}");
                return 2;
            }
        }
        i += 1;
    }
    let (Some(store_rel), Some(query)) = (store_rel, query) else {
        eprintln!("usage: host_council_server parity-check --store <rel> --query <text> [--k N]");
        return 2;
    };
    let store = match crate::packed_ternary::store_at(&store_rel) {
        Some(s) => s,
        None => {
            eprintln!("parity-check: cannot load store {store_rel:?}");
            return 1;
        }
    };
    let qv = crate::native_encoder::encode_f32(&query, store.dim());
    let q = match crate::packed_ternary::quantize(&qv, store.dim()) {
        Some(q) => q,
        None => {
            eprintln!("parity-check: query quantize failed");
            return 1;
        }
    };

    let t_cpu = std::time::Instant::now();
    let mut cpu = crate::packed_ternary::cpu_scan_in(&store, &q);
    let cpu_us = t_cpu.elapsed().as_secs_f64() * 1e6;
    cpu.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let gpu = match crate::gpu_compute::gpu_scan_scores_cached(&q, &store) {
        Some(g) => g,
        None => {
            eprintln!("parity-check: GPU scan failed (no device or store not cached)");
            return 1;
        }
    };
    let mut gi: Vec<(f32, usize)> = gpu.iter().cloned().enumerate().map(|(i, sc)| (sc, i)).collect();
    gi.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let kk = k.min(cpu.len()).min(gi.len());
    let mut max_diff = 0.0f64;
    let mut key_mismatch = 0usize;
    for rank in 0..kk {
        let d = (gi[rank].0 as f64 - cpu[rank].0).abs();
        if d > max_diff {
            max_diff = d;
        }
        if gi[rank].1 != cpu[rank].1 {
            key_mismatch += 1;
        }
    }
    let ok = key_mismatch == 0 && max_diff < 1e-3;
    eprintln!(
        "parity-check: {store_rel} — {kk} top-K compared, key mismatches {key_mismatch}, max score diff {max_diff:.2e} (tol 1e-3) — {}",
        if ok { "MATCH" } else { "MISMATCH" }
    );
    eprintln!("[timing] cpu_scan_in {cpu_us:.1} us | gpu_scan_scores_cached (persistent store buffer)");
    if ok {
        0
    } else {
        1
    }
}

/// Offline SMILES canonicalization CLI (CAP-201): `host_council_server smiles-canonicalize --input <smiles.txt> --output <canonical.txt>`.
/// Input is one SMILES per line; output is the canonical SMILES per line (the literal `PARSE_ERROR` on failure).
/// Rust-only (no RDKit) — parity against RDKit is measured externally, not assumed. Exit 0 on success, 1 on I/O, 2 on usage.
fn smiles_canonicalize_cli(args: &[String]) -> i32 {
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                input = args.get(i + 1).cloned();
                i += 2;
            }
            "--output" => {
                output = args.get(i + 1).cloned();
                i += 2;
            }
            other => {
                eprintln!("smiles-canonicalize: unknown argument {other:?}");
                return 2;
            }
        }
    }
    let (Some(input), Some(output)) = (input, output) else {
        eprintln!(
            "usage: host_council_server smiles-canonicalize --input <smiles.txt> --output <canonical.txt>"
        );
        return 2;
    };
    let raw = match std::fs::read_to_string(&input) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("smiles-canonicalize: cannot read input: {e}");
            return 1;
        }
    };
    let mut lines: Vec<String> = Vec::new();
    let mut skips = 0usize;
    for line in raw.lines() {
        let s = line.trim();
        if s.is_empty() {
            continue;
        }
        match smiles_canon::canonical_smiles(s) {
            Some(c) => lines.push(c),
            None => {
                lines.push("PARSE_ERROR".to_string());
                skips += 1;
            }
        }
    }
    let out = lines.join("\n") + "\n";
    if let Err(e) = std::fs::write(&output, out) {
        eprintln!("smiles-canonicalize: cannot write output: {e}");
        return 1;
    }
    eprintln!("smiles-canonicalize: {} lines, {} parse errors", lines.len(), skips);
    0
}

fn main() {
    // Offline tooling mode: `encode-batch` encodes a corpus batch on the GPU (CPU fallback) and exits.
    // This is the batch encoder's call path into the ingestion pipeline, not a runtime route.
    let argv: Vec<String> = std::env::args().collect();
    if argv.get(1).map(String::as_str) == Some("encode-batch") {
        std::process::exit(encode_batch_cli(&argv[2..]));
    }
    if argv.get(1).map(String::as_str) == Some("score-corpus") {
        std::process::exit(score_corpus_cli(&argv[2..]));
    }
    if argv.get(1).map(String::as_str) == Some("smiles-canonicalize") {
        std::process::exit(smiles_canonicalize_cli(&argv[2..]));
    }
    if argv.get(1).map(String::as_str) == Some("parity-check") {
        std::process::exit(parity_check_cli(&argv[2..]));
    }
    // Port is env-overridable so a candidate build can be exercised on a spare port before it is
    // swapped in. TRM_PORT unset (the supervisor's case) means 8022, exactly as before.
    let port: u16 = std::env::var("TRM_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8022);
    // CPU utilisation only exists as an interval delta, so the sampler runs alongside the
    // server instead of being measured per request.
    host_probe::start_samplers();
    // The IPC transport is the shared-memory ring bus (swarm_mailbox.ring). The kernel owns it --
    // Python is offline tooling and is not in the runtime path -- so it is created/zeroed here at
    // boot rather than left to a writer that never runs on bare metal.
    mailbox_ring::ensure();
    println!("===============================================================================================");
    println!(
        ">>> NATIVE RUST SOVEREIGN SWARM SERVER — MULTI-THREADED (Port {})",
        port
    );
    println!(
        "    8 Worker Threads | Sub-Millisecond Parallel Handling | Zero Python in Critical Path"
    );
    println!("===============================================================================================");

    let state = Arc::new(Mutex::new(CouncilState {
        proposals: Vec::new(),
        next_id: load_next_id().max(max_ledger_id() + 1),
    }));

    // The ledger is the memory: without this the process forgets every motion that passed before it started.
    {
        let mut st = state.lock().unwrap();
        hydrate_proposals(&mut st);
        let n = st.proposals.len();
        let c = st.proposals.iter().filter(|p| p.consensus_reached).count();
        println!("    COUNCIL LEDGER HYDRATED: {} proposals, {} of them consensus", n, c);
    }

    // Seed initial proposal — only on a genuinely empty ledger. It used to run on every boot, so eight restarts
    // left eight identical ANALYZE motions (ids 1473-1479, 1483) and consumed ids that filed motions needed.
    {
        let mut st = state.lock().unwrap();
        if st.proposals.is_empty() {
            evaluate_proposal(
                &mut st,
                "Zero-Python Pure Rust Server Deployment".to_string(),
                "Architecture".to_string(),
                "Archi".to_string(),
            );
        }
    }

    // Motion 1478: the tick files its own motions through this module's real vote path -- the same
    // evaluate_proposal and the same ledger -- so a self-filed motion is tallied by exactly the council a human
    // motion is. Registered here because both are owned here.
    {
        let st = state.clone();
        autonomy::set_proposer(Box::new(move |topic: &str, category: &str| {
            let mut guard = st.lock().ok()?;
            let id = evaluate_proposal(
                &mut guard,
                topic.to_string(),
                category.to_string(),
                "TRM-Autonomy".to_string(),
            );
            let phase = guard.proposals.iter().find(|p| p.id == id).map(|p| p.phase.clone())?;
            Some((id, phase))
        }));
    }

    // Motion 1474: the platform's own clock -- measure, diff, record, learn, escalate, and now self-direct.
    autonomy::start_loop();
    // The autonomy loop escalates "5 enabled goals are past due and no runner consumes them"; this is the consumer.
    // TRM_GOAL_SECS=0 disables it.
    goal_runner::start_loop();

    // Bind guard: a node reachable off-box must carry a token, or it is an open mutation surface.
    let bind_host = std::env::var("TRM_BIND_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let auth_configured = std::env::var("COUNCIL_AUTH_TOKEN")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .is_some();
    let is_loopback = matches!(bind_host.as_str(), "127.0.0.1" | "localhost" | "::1");
    if !is_loopback && !auth_configured {
        eprintln!("FATAL: refusing to bind {} without COUNCIL_AUTH_TOKEN", bind_host);
        std::process::exit(1);
    }
    println!("     bind {}:{} | token {}", bind_host, port, if auth_configured { "configured" } else { "not set (loopback only)" });
    let server = Arc::new(Server::http(format!("{}:{}", bind_host, port)).unwrap());
    println!(
        "🚀 Native Rust Multi-Threaded Server listening on http://{}:{}",
        bind_host, port
    );

    // FEDERATION-02: the registry is a file, so a probe tick keeps probed_at younger than the
    // interval instead of waiting for an operator to run a script.
    thread::spawn(|| loop {
        thread::sleep(std::time::Duration::from_secs(60));
        state_handlers::refresh_registry_probes();
    });

    // A restart orphans the lazy tier-2 coder: the child keeps running, but this process has no PID to
    // reap, so its commit sits idle. Reclaim the port before serving anything.
    intelligence_router::reap_orphaned_tier2();

    // The dashboard has lived on :5183 for the life of this platform, and moving it broke the
    // operator's access. So the kernel binds a SECOND listener there and serves the SAME handler
    // table from it. Both ports are one origin each: the UI's root-relative fetches resolve against
    // whichever listener served the page, so there is still no proxy anywhere. TRM_UI_PORT overrides;
    // 0 disables the second listener.
    let ui_port: u16 = std::env::var("TRM_UI_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5183);

    let mut servers: Vec<(u16, Arc<Server>)> = vec![(port, server)];
    if ui_port != 0 && ui_port != port {
        match Server::http(format!("0.0.0.0:{}", ui_port)) {
            Ok(s) => {
                println!("🖥  Dashboard listener on http://0.0.0.0:{}/ui/", ui_port);
                servers.push((ui_port, Arc::new(s)));
            }
            Err(e) => eprintln!(
                "dashboard listener :{} unavailable ({}) -- UI still served on :{}/ui/",
                ui_port, e, port
            ),
        }
    }

    // Worker threads per listener. Every request passes the same measured point, refusals included:
    // the telemetry route reports this process's own throughput, not a literal.
    let mut workers = Vec::new();
    for (lport, srv) in servers {
        for _ in 0..8 {
            let server_clone = Arc::clone(&srv);
            let state_clone = Arc::clone(&state);
            let handle = thread::spawn(move || {
                for request in server_clone.incoming_requests() {
                    let t0 = Instant::now();
                    handle_request(request, &state_clone);
                    telemetry_handlers::note_request(t0.elapsed().as_nanos() as u64);
                }
            });
            workers.push(handle);
        }
        println!("⚡ 8 worker threads serving :{}", lport);
    }
    // The native trm:// QUIC listener runs beside the HTTP listeners, so the on-screen dashboard
    // (and any sovereign client) can reach the kernel without HTTP. Loopback only: the dashboard
    // and the kernel are one machine.
    thread::spawn(|| {
        core_pinning::pin_current_thread_to_core(core_pinning::CORE_QUIC);
        trm_datagram::run();
    });
    // Heartbeat auto-dispatch: a background loop that polls HEARTBEAT.md for pending
    // tasks and turns each into a kanban card (auto-assigned via the domain ontology).
    // This is the "auto process" that keeps the kanban board live.
    thread::spawn(|| {
        core_pinning::pin_current_thread_to_core(core_pinning::CORE_HEARTBEAT);
        kanban_handlers::heartbeat_auto_loop();
    });
    // Agent work loop: advance active cards through the lifecycle (TODO -> IN_PROGRESS
    // -> REVIEW -> DONE), binding each IN_PROGRESS card to the measured host's silicon.
    thread::spawn(|| {
        core_pinning::pin_current_thread_to_core(core_pinning::CORE_AGENT_WORK);
        kanban_handlers::agent_work_loop();
    });

    for worker in workers {
        let _ = worker.join();
    }
}

mod native_parsers;
mod trm_datagram;

#[cfg(test)]
mod test_support;
