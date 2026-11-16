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

// ── The oracle's model (swarm_v2/core/llm_brain.py) ──────────────────────────────────────────────
//
// `index_prompt_batch_native`: per prompt, an action id from the FIRST tag present, in the oracle's
// own precedence -- WRITE_FILE: 1, DELEGATE_TASK: 2, SEARCH_QUERY: 3, CALL_TOOL: 4, none 0 -- and an
// estimated token count of `len(bytes) // 4`.
//
// `build_system_prompt`: assembles SWARM_OS_CONTEXT (extracted verbatim from the python source below),
// the identity block, the operational rules, an action block only in "action" mode, and the memory
// section only when the memory string is non-blank; the whole thing is `.strip()`ed. The system date
// is a parameter here because the oracle calls `datetime.now()`.
//
// One finding, not a port: `build_system_prompt` takes a `skill_names` argument in python and never
// references it -- the only occurrence in the file is the parameter itself. The firmware signature
// therefore omits it rather than carrying a parameter nothing reads.

/// The oracle's system-prompt preamble, extracted from `llm_brain.py` rather than retyped.
pub const SWARM_OS_CONTEXT: &str = "# SWARM OS - ADVANCED NEURAL ARCHITECTURE
You are a high-level cognitive unit within Swarm OS v12. Your mission is to generate sophisticated, production-grade artifacts that embody first-principles reasoning and state-of-the-art engineering.

## 🏛️ ARCHITECTURAL PRINCIPLES:
1.  **MODULARITY**: Design systems as decoupled, interoperable units.
2.  **ROBUSTNESS**: Implement defensive programming, thorough error handling, and edge-case validation.
3.  **SCALABILITY**: Plan for distributed execution and high-concurrency environments.
4.  **OBSERVABILITY**: Include detailed logging, telemetry hooks, and diagnostic pathways.
5.  **AESTHETICS**: Use clear Markdown formatting, Mermaid diagrams for logic flow, and professional documentation standards.

## 🛠️ CORE ACTION TAGS:
- `WRITE_FILE: path/to/file.ext`: Create/Update artifacts. Use [START] and [END] markers.
- `CREATE_FILES:`: Multi-file orchestration.
- `DELEGATE_TASK: Role | Task`: Strategic delegation to peer experts.
- `SEARCH_QUERY: Query`: Real-time knowledge acquisition.
- `CALL_TOOL: Tool Endpoint Method Data`: Direct hardware/API interaction.

## 💎 ARTIFACT QUALITY STANDARD:
- **NO PLACEHOLDERS**: Generate complete, functional code or documentation.
- **DEPTH**: Provide detailed explanations of rationale and architectural tradeoffs.
- **PRECISION**: Use technical terminology correctly and specify versions/dependencies.";

/// The action tags in the oracle's precedence order.
pub const ACTION_TAGS: [(&str, u8); 4] = [
    ("WRITE_FILE:", 1),
    ("DELEGATE_TASK:", 2),
    ("SEARCH_QUERY:", 3),
    ("CALL_TOOL:", 4),
];

/// `index_prompt_batch_native`: (action id, estimated tokens) per prompt.
pub fn index_prompt_batch_native(prompts: &[&[u8]]) -> Vec<(u8, usize)> {
    prompts
        .iter()
        .map(|p| {
            let mut act = 0u8;
            for (tag, id) in ACTION_TAGS.iter() {
                if contains(p, tag.as_bytes()) {
                    act = *id;
                    break;
                }
            }
            (act, p.len() / 4)
        })
        .collect()
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    if needle.len() > hay.len() {
        return false;
    }
    hay.windows(needle.len()).any(|w| w == needle)
}

/// `build_system_prompt`. `mode` is the oracle's string ("action" enables the action block); `now` is
/// the system date the python reads from the clock.
pub fn build_system_prompt(persona_name: &str, role: &str, background: &str, specialties: &[&str],
                           memory: &str, mode: &str, now: &str) -> String {
    let mut out = String::from(SWARM_OS_CONTEXT);
    out.push_str("\n\n## \u{1F464} AGENT IDENTITY:\n");
    out.push_str("Name: "); out.push_str(persona_name); out.push('\n');
    out.push_str("Designation: "); out.push_str(role); out.push('\n');
    out.push_str("Cognitive Background: "); out.push_str(background); out.push('\n');
    out.push_str("Core Specialties: ");
    out.push_str(&specialties.join(", "));
    out.push_str("\nSystem Date/Time: ");
    out.push_str(now);
    out.push_str(" (Current temporal anchor for relative queries like 'today', 'last 72 hours', 'this year')");
    out.push_str("\n\n## \u{1F4DC} OPERATIONAL RULES:\n");
    out.push_str("1. Respond with high technical density and professional clarity.\n");
    out.push_str("2. Do not explain your instructions; proceed immediately to objective completion.\n");
    out.push_str("3. Ensure all artifacts are 'Advanced'\u{2014}no simple examples or half-measures.");
    if mode == "action" {
        out.push_str("\n\n## \u{26A1} AUTONOMOUS EXECUTION PROTOCOL:\n");
        out.push_str("When generating artifacts, you MUST use this strict format:\n");
        out.push_str("WRITE_FILE: filename.ext\n[START]\n<production_grade_content>\n[END]\n\n");
        out.push_str("To assign work: DELEGATE_TASK: Role | Task\n");
        out.push_str("To acquire data: SEARCH_QUERY: Query\n");
        out.push_str("To call tools: CALL_TOOL: Tool Endpoint\n");
    }
    if !memory.trim().is_empty() {
        out.push_str("\n\n## \u{1F9E0} NEURAL MEMORY & CONTEXT:\n");
        out.push_str(memory);
    }
    String::from(out.trim())
}
