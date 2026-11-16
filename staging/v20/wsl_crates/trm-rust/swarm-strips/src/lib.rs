//! swarm-strips — the 19 new-strip agents implemented as native Rust strips (MOTION-052).
//! Each strip is a real, honest computation: derive, don't guess. No Ollama, no Python agent.
//! `run_strip(name, input)` routes to the strip; every strip returns a JSON string.

pub mod strips_security;
pub mod strips_logic;
pub mod strips_code;
pub mod strips_telemetry;
pub mod strips_synthesis;
pub mod strips_healing;
pub mod strips_scientific;

pub const STRIPS: [&str; 19] = [
    "ebpf_probe",
    "math_solver",
    "attempt_sampler",
    "symbol_solver",
    "refactor_skill",
    "test_generator",
    "fuzz_test_skill",
    "hot_patch",
    "memory_synchronizer",
    "monitoring_skill",
    "shared_memory",
    "synthesis_pipeline",
    "intent_buffer",
    "persona_matrix",
    "chaos_simulator",
    "star_matrix",
    "dynamic_router",
    "alpha_genome",
    "sequence_aligner",
];

/// Run a strip by its snake_case name, returning its JSON result string.
pub fn run_strip(name: &str, input: &str) -> Option<String> {
    Some(match name {
        "ebpf_probe" => strips_security::ebpf_probe(input),
        "math_solver" => strips_logic::math_solver(input),
        "attempt_sampler" => strips_logic::attempt_sampler(input),
        "symbol_solver" => strips_logic::symbol_solver(input),
        "refactor_skill" => strips_code::refactor_skill(input),
        "test_generator" => strips_code::test_generator(input),
        "fuzz_test_skill" => strips_code::fuzz_test_skill(input),
        "hot_patch" => strips_code::hot_patch(input),
        "memory_synchronizer" => strips_telemetry::memory_synchronizer(input),
        "monitoring_skill" => strips_telemetry::monitoring_skill(input),
        "shared_memory" => strips_telemetry::shared_memory(input),
        "synthesis_pipeline" => strips_synthesis::synthesis_pipeline(input),
        "intent_buffer" => strips_synthesis::intent_buffer(input),
        "persona_matrix" => strips_synthesis::persona_matrix(input),
        "chaos_simulator" => strips_healing::chaos_simulator(input),
        "star_matrix" => strips_healing::star_matrix(input),
        "dynamic_router" => strips_healing::dynamic_router(input),
        "alpha_genome" => strips_scientific::alpha_genome(input),
        "sequence_aligner" => strips_scientific::sequence_aligner(input),
        _ => return None,
    })
}

/// JSON-string escape for embedding a result into a strip's JSON reply.
pub fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

/// FNV-1a 64-bit content hash (dependency-free; used by several strips).
pub fn fnv1a(s: &str) -> u64 {
    s.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}
