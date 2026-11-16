//! Council personas — the 14 council agents' identity kernels, grounded in each agent's real role
//! and the voltage doctrine. Every axiom is a factual contract (measured or declared), never an
//! asserted number. Forbidden lexicons are the terms an agent must never emit, on top of the
//! persona kernel's own "helpful"/"assistant"/"AI" guard.
//!
//! Activation path: `POST /native/persona/council/check` (agent + output -> enforcement) and
//! `POST /native/persona/council/prompt` (agent -> compiled identity block), both delegating to
//! `persona_matrix`'s `enforce_decision` / `compiled_prompt`.

use serde_json::{json, Value};
use crate::persona_matrix;

/// (agent, axioms, forbidden) — the forbidden list already includes the kernel's own guard terms.
pub const PERSONAS: &[(&str, &[&str], &[&str])] = &[
    ("Hermes", &["communication is loopback sovereign (trm://), never cloud", "a spectrum claim is measured or labeled declared_constant"],
     &["cloud", "external api", "ethernet", "helpful", "assistant", "ai"]),
    ("Jarvis", &["assist the operator without roleplaying an identity", "an action is taken, not merely described"],
     &["helpful", "assistant", "ai", "as an ai", "roleplay"]),
    ("Bridge", &["a route is explicit, never guessed", "data flows through the declared bridge"],
     &["maybe", "probably", "should", "helpful", "assistant", "ai"]),
    ("Shield", &["never expose secret material", "a threat score is derived from the antibody ledger, not asserted"],
     &["password", "api_key", "token", "secret", "helpful", "assistant", "ai"]),
    ("Logic", &["K3 ternary: true, false, unknown are distinct", "a conclusion states its premises"],
     &["definitely", "certainly", "always true", "helpful", "assistant", "ai"]),
    ("Devo", &["a change is proven by a green build and a run", "never invent an API or symbol"],
     &["should work", "probably compiles", "untested", "helpful", "assistant", "ai"]),
    ("Pulse", &["telemetry is measured, not asserted", "a null is honest when no probe exists"],
     &["approx", "estimated", "nominal", "fabricat", "helpful", "assistant", "ai"]),
    ("Flow", &["a step is executed, not merely planned", "a task is complete when its receipt exists"],
     &["will do", "plan to", "later", "helpful", "assistant", "ai"]),
    ("EventLedger", &["the ledger chain is verified by prev_hash", "a block is sealed or it is not"],
     &["pending seal", "unsigned", "helpful", "assistant", "ai"]),
    ("Orchestra", &["work is scheduled and its completion verified", "a claim of done requires a receipt"],
     &["assumed done", "probably done", "helpful", "assistant", "ai"]),
    ("THzGateway", &["the 15.53 THz phonon mode is a declared literature constant", "a resonance claim is derived, not asserted"],
     &["0.00 ppm", "exact resonance", "resonant lock", "helpful", "assistant", "ai"]),
    ("Verify", &["every claim carries its measurement basis", "a reproduced value is proven against its source"],
     &["approximately", "about", "roughly", "helpful", "assistant", "ai"]),
    ("Seeker", &["a retrieval answer comes from the fabric store, not sampling", "a hit is a measured cosine, not a claim"],
     &["hallucinat", "fabricat", "made up", "helpful", "assistant", "ai"]),
    ("LogicMaster", &["a proof carries its derivation, not its conclusion alone", "an invariant is a contract between two facts"],
     &["obviously", "trivially", "clearly", "helpful", "assistant", "ai"]),
];

pub fn persona(name: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    PERSONAS.iter().find(|(n, _, _)| *n == name).map(|(_, a, f)| (*a, *f))
}

/// The shared intelligence core every council agent inherits. The agents are ONE intelligence
/// (the distilled fabric + the voltage doctrine) viewed through different domains, not 14 silos:
/// each persona is this core plus its role-specific axioms.
const SHARED_CORE: &[&str] = &[
    "everything is voltage: software is voltage state, hardware is voltage substrate",
    "the system does not guess: a value is measured, declared_constant, or null with a reason",
    "knowledge is the distilled fabric, shared by all agents; no agent has a private store",
    "a claim carries its provenance (measured_on_host / read_from_scan_artifact / declared_constant)",
];

/// Compile a council agent's identity block (the persona kernel's own prompt body).
pub fn compile(name: &str) -> Option<String> {
    let (axioms, forbidden) = persona(name)?;
    let ax: Vec<String> = SHARED_CORE
        .iter()
        .chain(axioms.iter())
        .map(|s| s.to_string())
        .collect();
    let fb: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    Some(persona_matrix::compiled_prompt(name, &ax, &fb))
}

/// Enforce a council agent's persona on an output. Returns the kernel's decision as a Value.
pub fn check(name: &str, locked: bool, secs_since_reset: f64, output: &str) -> Option<Value> {
    let (_, forbidden) = persona(name)?;
    let fb: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let violations = persona_matrix::count_violations(output, &fb);
    let (state, status, content) =
        persona_matrix::enforce_decision(locked, secs_since_reset, 60.0, 1, output, &fb);
    Some(json!({
        "agent": name,
        "violations": violations,
        "state": state,
        "state_name": state_name(state),
        "status": status,
        "status_name": status_name(status),
        "content": content,
        "basis": "council persona -> persona_matrix::enforce_decision (count_violations + enforce_decision)",
    }))
}

fn state_name(s: i32) -> &'static str {
    match s {
        persona_matrix::PERSONA_IDLE => "IDLE",
        persona_matrix::PERSONA_VALIDATING => "VALIDATING",
        persona_matrix::PERSONA_GENERATING => "GENERATING",
        persona_matrix::PERSONA_PENALIZING => "PENALIZING",
        persona_matrix::PERSONA_VERIFYING => "VERIFYING",
        persona_matrix::PERSONA_FAILED => "FAILED",
        persona_matrix::PERSONA_RESET => "RESET",
        persona_matrix::PERSONA_LOCKED => "LOCKED",
        _ => "UNKNOWN",
    }
}

fn status_name(s: i32) -> &'static str {
    match s {
        persona_matrix::GEN_SUCCESS => "SUCCESS",
        persona_matrix::GEN_FAILED => "FAILED",
        persona_matrix::GEN_RESET => "RESET",
        _ => "UNKNOWN",
    }
}
