//! TRM Swarm OS — Coherence Thresholds (no_std port).
//! 1:1 core semantics from swarm_v2/core/coherence_thresholds.py — Catalyst.
//! Deterministic stability thresholds: coherence check + stability verdict.

extern crate alloc;

/// Threshold constants (1:1 python).
pub const COHERENCE_MIN: f64 = 0.45;
pub const COHERENCE_MAX: f64 = 0.95;
pub const STABILITY_WINDOW: u64 = 8;

/// Check stability of a coherence value.
/// Returns (stable, reason_code): 0=stable, 1=below-min, 2=above-max.
pub fn check_stability(coherence: f64, history_len: u64) -> (bool, u64) {
    if coherence < COHERENCE_MIN {
        (false, 1)
    } else if coherence > COHERENCE_MAX {
        (false, 2)
    } else {
        (history_len >= STABILITY_WINDOW, 0)
    }
}

/// Threshold summary for the LCARS terminal.
pub fn get_thresholds() -> (f64, f64, u64) {
    (COHERENCE_MIN, COHERENCE_MAX, STABILITY_WINDOW)
}
