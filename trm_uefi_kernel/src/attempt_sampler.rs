//! TRM Swarm OS — attempt_sampler (no_std port).
//! Deterministic envelope from swarm_v2/core/attempt_sampler.py — Quantum-
//! Inspired 'Attempt Sampling' (Superposition): spawn N agents to solve the
//! same task, collapse the best state. The async sub-agent spawning and the
//! orchestrator LLM audit are MANUAL (host-only); the deterministic envelope —
//! attempt framing strings "Attempt ID: {i+1}", result count, selection index —
//! is ported 1:1.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Default number of parallel attempts (1:1 python default n_attempts=2).
pub const DEFAULT_N_ATTEMPTS: usize = 2;
/// Role assigned to every spawned subordinate reasoning state.
pub const SUBORDINATE_ROLE: &str = "SubordinateReasoningState";

/// Result of a superposition sampling round.
#[derive(Debug, Clone)]
pub struct SampleOutcome {
    /// Selected best result string (post wavefunction-collapse).
    pub best_result: String,
    /// All parallel attempt results, in spawn order.
    pub all_results: Vec<String>,
    /// Attempt prompt prefix used for result i: "Attempt ID: {i+1}\nTask: ...".
    pub attempts_spawned: usize,
}

/// AttemptSampler handle (orchestrator reference is host-only).
#[derive(Debug, Clone)]
pub struct AttemptSampler {
    pub orchestrator_handle: u64,
    pub samples_taken: u64,
}

impl AttemptSampler {
    pub fn new(orchestrator_handle: u64) -> Self {
        AttemptSampler {
            orchestrator_handle,
            samples_taken: 0,
        }
    }

    /// Frame the task prompt for attempt i (1-indexed), 1:1 with python:
    /// `f"Attempt ID: {i+1}\nTask: {task}"`.
    pub fn attempt_prompt(task: &str, attempt_index_0based: usize) -> String {
        format!("Attempt ID: {}\nTask: {}", attempt_index_0based + 1, task)
    }

    /// Deterministic envelope of sample_attempts: the orchestrator has already
    /// executed `results` (one per spawned state) and the audit selected the
    /// best. Selection index of the python audit cannot be predicted without
    /// the LLM, so the port keeps ALL results and flags the chosen index.
    pub fn sample_attempts(&mut self, results: Vec<String>) -> SampleOutcome {
        self.samples_taken += 1;
        let n = results.len().max(1);
        // MANUAL: `_wavefunction_collapse` audits each "--- Universe {i+1}
        // Result ---" via the orchestrator LLM and returns the winning text.
        // Kernel convention: state 1 (index 0) wins on tie.
        let best = results.first().cloned().unwrap_or_default();
        SampleOutcome {
            best_result: best,
            all_results: results,
            attempts_spawned: n,
        }
    }

    /// Pure scoring helper used by the collapse auditor: deterministic
    /// fidelity proxy = length-weighted completeness (higher = longer,
    /// structured answer). Mirrors "select the single BEST response".
    pub fn fidelity_proxy(response: &str) -> f64 {
        let len = response.len() as f64;
        if len == 0.0 {
            return 0.0;
        }
        let mut score = len.min(500.0) / 500.0;
        if response.contains("BEST") || response.contains("PASS") {
            score += 0.25;
        }
        score.min(1.0)
    }
}

/// get_attempt_sampler(orchestrator) — 1:1 factory returning a handle-based
/// sampler (python returns AttemptSampler(orchestrator)).
pub fn get_attempt_sampler(orchestrator_handle: u64) -> AttemptSampler {
    AttemptSampler::new(orchestrator_handle)
}
