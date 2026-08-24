//! TRM Swarm OS — microkernel_spawner (no_std port).
//! Deterministic microkernel intercept/spawn logic. Python body embeds
//! stdout examples; clean no_std skeleton.

extern crate alloc;

/// Deterministic microkernel routing of an operation by op_type.
/// Returns 1 when the microkernel intercepts (bypass), 0 when it falls
/// through to neural resolution.
pub fn microkernel_intercept(op_type: &str) -> u64 {
        // classify_deterministic: keyword op-type lookup.
        let lower = op_type.to_lowercase();
        let mut code: u64 = 0;
        for kw in ["memory", "query", "search", "recall"] {
            if lower.contains(kw) { code = 1; }
        }
        for kw in ["status", "health", "monitor", "ping"] {
            if lower.contains(kw) && code == 0 { code = 2; }
        }
        for kw in ["heal", "repair", "fix", "recover"] {
            if lower.contains(kw) && code == 0 { code = 3; }
        }
        for kw in ["port", "compile", "build", "test"] {
            if lower.contains(kw) && code == 0 { code = 4; }
        }
        code
}
