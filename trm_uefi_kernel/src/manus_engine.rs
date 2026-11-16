//! TRM Swarm OS — Native Pure Rust Manus Engine & Agent-Q Orchestrator (no_std compatible).
//! 1:1 implementation of Quantum-Inspired Agentic Engineering (QIAE) Protocol:
//! - Multi-agent task superposition across parallel candidate paths
//! - Probabilistic QState wavefunction & priority operator boosting
//! - Dynamic coherence monitoring & deterministic measurement gate collapse (>0.85 threshold)

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

#[inline]
fn f32_sqrt(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut guess = if x > 1.0 { x * 0.5 } else { 1.0 };
    for _ in 0..10 {
        guess = 0.5 * (guess + x / guess);
    }
    guess
}

#[derive(Debug, Clone)]
pub struct QState {
    pub num_dimensions: usize,
    pub amplitudes: Vec<f32>,
}

impl QState {
    pub fn new(dim: usize) -> Self {
        let n = dim.max(1);
        let init_val = 1.0f32 / f32_sqrt(n as f32);
        QState {
            num_dimensions: n,
            amplitudes: vec![init_val; n],
        }
    }

    pub fn normalize(&mut self) {
        let mut sum_sq: f32 = 0.0;
        for &a in self.amplitudes.iter() {
            sum_sq += a * a;
        }
        let norm = f32_sqrt(sum_sq);
        if norm > 0.00001 {
            for a in self.amplitudes.iter_mut() {
                *a /= norm;
            }
        }
    }

    pub fn probabilities(&self) -> Vec<f32> {
        self.amplitudes.iter().map(|&a| a * a).collect()
    }

    pub fn apply_priority(&mut self, target_idx: usize, evidence_weight: f32) {
        if target_idx < self.amplitudes.len() {
            self.amplitudes[target_idx] *= 1.0 + evidence_weight.max(0.0);
            self.normalize();
        }
    }

    pub fn resolve_deterministic(&self) -> usize {
        let probs = self.probabilities();
        let mut max_idx = 0;
        let mut max_p = -1.0f32;
        for (i, &p) in probs.iter().enumerate() {
            if p > max_p {
                max_p = p;
                max_idx = i;
            }
        }
        max_idx
    }
}

#[derive(Debug, Clone)]
pub struct SuperpositionTask {
    pub task_id: u64,
    pub task_name: String,
    pub candidate_roles: Vec<&'static str>,
    pub state: QState,
    pub candidate_results: Vec<Option<String>>,
    pub coherence: f32,
    pub collapsed: bool,
    pub winning_role: Option<&'static str>,
}

pub struct ManusEngine {
    pub tasks_processed: u64,
    pub collapses_completed: u64,
}

impl ManusEngine {
    pub fn new() -> Self {
        ManusEngine {
            tasks_processed: 0,
            collapses_completed: 0,
        }
    }

    /// Create a new multi-agent superposition task across candidate roles.
    pub fn create_superposition(
        &mut self,
        task_id: u64,
        task_name: &str,
        roles: &[&'static str],
    ) -> SuperpositionTask {
        self.tasks_processed += 1;
        let dim = roles.len().max(1);
        let state = QState::new(dim);

        SuperpositionTask {
            task_id,
            task_name: String::from(task_name),
            candidate_roles: roles.to_vec(),
            state,
            candidate_results: vec![None; dim],
            coherence: 0.0,
            collapsed: false,
            winning_role: None,
        }
    }

    /// Record an agent path completion and update quantum superposition coherence.
    pub fn submit_attempt(
        &mut self,
        task: &mut SuperpositionTask,
        role_idx: usize,
        result: &str,
        evidence_score: f32,
    ) {
        if role_idx < task.candidate_results.len() {
            task.candidate_results[role_idx] = Some(String::from(result));
            task.state.apply_priority(role_idx, evidence_score);

            // Compute Coherence = (Max Probability) * (Completion Ratio)
            let probs = task.state.probabilities();
            let max_p = probs.iter().cloned().fold(0.0f32, f32::max);
            let completed = task.candidate_results.iter().filter(|r| r.is_some()).count();
            let completion_ratio = completed as f32 / task.candidate_roles.len() as f32;

            task.coherence = max_p * completion_ratio;
        }
    }

    /// Measurement Gate: Check if coherence threshold (>0.85) is reached and collapse state.
    pub fn check_and_collapse(&mut self, task: &mut SuperpositionTask) -> bool {
        let all_completed = task.candidate_results.iter().all(|r| r.is_some());
        if task.coherence >= 0.85 || all_completed {
            let winner_idx = task.state.resolve_deterministic();
            task.collapsed = true;
            task.winning_role = Some(task.candidate_roles[winner_idx]);
            self.collapses_completed += 1;
            true
        } else {
            false
        }
    }
}

// ── ported from the v20 base tree (G189) ─────────────────────────────────────────────────────────
// 1 column-0 item(s) only: nested fn definitions and `self` receivers are excluded, and the
// statics/consts each function names travel with it (grafting bare functions was the first failure).

fn sqrt_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut g = x;
    for _ in 0..12 {
        g = 0.5 * (g + x / g);
    }
    g
}
