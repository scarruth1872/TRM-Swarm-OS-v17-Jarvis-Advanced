//! TRM Swarm OS — Native Pure Rust LLM Brain & Persona Orchestrator (no_std compatible).
//! Assembles structured system prompts, persona identities, cognitive task contexts,
//! and heuristic reasoning routing without requiring external Python or Ollama wrappers.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct AgentPersona {
    pub name: &'static str,
    pub role: &'static str,
    pub specialties: &'static [&'static str],
    pub default_weight: f32,
}

pub const PERSONAS: &[AgentPersona] = &[
    AgentPersona { name: "Hermes", role: "Comms & Spectrum", specialties: &["networking", "mesh", "protocols"], default_weight: 1.0 },
    AgentPersona { name: "Jarvis", role: "Assistant & Operator", specialties: &["coordination", "user_intent", "task_planning"], default_weight: 0.9 },
    AgentPersona { name: "Bridge", role: "APIs & Data Routing", specialties: &["zero_copy", "ipc", "serialization"], default_weight: 1.0 },
    AgentPersona { name: "Shield", role: "Security & Antibody", specialties: &["ddr_antibodies", "threat_modeling", "memory_protection"], default_weight: 1.0 },
    AgentPersona { name: "Logic", role: "Reasoning Engine", specialties: &["k3_ternary", "formal_proofs", "non_contradiction"], default_weight: 0.9 },
    AgentPersona { name: "Devo", role: "Code & Build", specialties: &["compilation", "no_std", "rust_contracts"], default_weight: 1.0 },
    AgentPersona { name: "Pulse", role: "Monitoring & Telemetry", specialties: &["thermal", "voltage_rails", "scheduler"], default_weight: 1.0 },
    AgentPersona { name: "Flow", role: "Workflow Orchestrator", specialties: &["async_queues", "work_stealing", "lockless_pipelines"], default_weight: 0.9 },
    AgentPersona { name: "EventLedger", role: "Audit & Monad Ledger", specialties: &["immutable_hashing", "state_transitions"], default_weight: 1.0 },
    AgentPersona { name: "Orchestra", role: "Work-Stealing Scheduler", specialties: &["multi_threading", "simd_batching"], default_weight: 0.8 },
    AgentPersona { name: "THzGateway", role: "Spectrum & VLF", specialties: &["sub_thz", "photonic_carriers", "rf_telemetry"], default_weight: 0.9 },
    AgentPersona { name: "Verify", role: "Truth & Anchor Verification", specialties: &["monad_anchor", "epistemology", "zero_drift"], default_weight: 0.8 },
    AgentPersona { name: "Seeker", role: "NAND Knowledge Graph", specialties: &["nand_shards", "semantic_indexing", "retrieval"], default_weight: 1.0 },
    AgentPersona { name: "Archi", role: "Bare-Metal Microkernel Architecture", specialties: &["sram_patterns", "voltage_binding", "uefi_boot"], default_weight: 1.0 },
];

pub struct LLMBrain;

impl LLMBrain {
    /// Construct a full sovereign system prompt for an agent persona.
    pub fn build_system_prompt(persona_name: &str, task_context: &str) -> String {
        let persona = PERSONAS.iter().find(|p| p.name.eq_ignore_ascii_case(persona_name))
            .copied()
            .unwrap_or(PERSONAS[1]); // Default to Jarvis

        format!(
            "# SOVEREIGN REASONING CONTEXT — TRM SWARM OS v19.001\n\
            Agent: {} | Role: {}\n\
            Specialties: {}\n\
            Monad Truth Anchor: 0x4D4F4E4144000001 | Logic: K3 Balanced Ternary\n\
            Task: {}\n\
            Directives: Formulate concise, verified conclusions adhering to non-contradiction and bare-metal safety.",
            persona.name, persona.role, persona.specialties.join(", "), task_context
        )
    }

    /// Fast zero-latency cognitive routing for a task query.
    pub fn route_task(query: &str) -> &'static AgentPersona {
        let q_lower = query.to_lowercase();
        if q_lower.contains("security") || q_lower.contains("threat") || q_lower.contains("antibody") {
            &PERSONAS[3] // Shield
        } else if q_lower.contains("code") || q_lower.contains("build") || q_lower.contains("compile") {
            &PERSONAS[5] // Devo
        } else if q_lower.contains("logic") || q_lower.contains("proof") || q_lower.contains("ternary") {
            &PERSONAS[4] // Logic
        } else if q_lower.contains("hardware") || q_lower.contains("voltage") || q_lower.contains("sram") {
            &PERSONAS[13] // Archi
        } else if q_lower.contains("spectrum") || q_lower.contains("network") || q_lower.contains("comms") {
            &PERSONAS[0] // Hermes
        } else {
            &PERSONAS[1] // Jarvis
        }
    }
}
