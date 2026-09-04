//! TRM Swarm OS — Native Pure Rust TRM Brain (no_std compatible).
//! 1:1 cognitive recursive reasoning engine with Mamba-3 SSM state-space updates,
//! deterministic halt decisions, and K3 balanced ternary collapses (+1, 0, -1).

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub const REASONING_STEPS_MAX: usize = 16;
pub const STATE_DIMENSION: usize = 128;

#[derive(Debug, Clone)]
pub struct ReasoningResult {
    pub final_state: Vec<f32>,
    pub k3_trit: i8,
    pub confidence: f32,
    pub steps_taken: usize,
    pub halted: bool,
    pub explanation: String,
}

pub struct TRMBrain {
    pub hidden_state: Vec<f32>,
    pub step_count: u64,
    pub total_inferences: u64,
}

#[inline]
fn f32_tanh(x: f32) -> f32 {
    if x > 3.0 {
        1.0
    } else if x < -3.0 {
        -1.0
    } else {
        let x2 = x * x;
        x * (27.0 + x2) / (27.0 + 9.0 * x2)
    }
}

impl TRMBrain {
    pub fn new() -> Self {
        TRMBrain {
            hidden_state: vec![0.0f32; STATE_DIMENSION],
            step_count: 0,
            total_inferences: 0,
        }
    }

    /// Execute a recursive reasoning pass over an input context vector.
    pub fn infer(&mut self, context: &[f32], max_steps: usize) -> ReasoningResult {
        self.total_inferences += 1;
        let steps = max_steps.min(REASONING_STEPS_MAX).max(1);

        let mut current_state = vec![0.0f32; STATE_DIMENSION];
        for i in 0..STATE_DIMENSION {
            current_state[i] = if i < context.len() { context[i] } else { 0.0f32 };
        }

        let mut steps_taken = 0;
        let mut halted = false;

        for s in 0..steps {
            steps_taken += 1;
            self.step_count += 1;

            // Recurrent SSM Step: h(t+1) = A * h(t) + B * x(t)
            let mut energy = 0.0f32;
            for i in 0..STATE_DIMENSION {
                let prev = current_state[i];
                let input_val = if i < context.len() { context[i] } else { 0.0f32 };
                let next_val = f32_tanh(0.85 * prev + 0.15 * input_val);
                current_state[i] = next_val;
                energy += next_val.abs();
            }

            // Check dynamic halt condition (convergence or energy threshold)
            let mean_energy = energy / STATE_DIMENSION as f32;
            if s >= 2 && (mean_energy > 0.75 || mean_energy < 0.05) {
                halted = true;
                break;
            }
        }

        self.hidden_state = current_state.clone();

        // Collapse state into K3 balanced ternary decision
        let mut sum: f32 = 0.0;
        for &v in current_state.iter() {
            sum += v;
        }
        let normalized_sum = sum / (STATE_DIMENSION as f32);

        let (k3_trit, confidence) = if normalized_sum > 0.15 {
            (1i8, (0.5 + normalized_sum.abs() * 0.5).min(0.99))
        } else if normalized_sum < -0.15 {
            (-1i8, (0.5 + normalized_sum.abs() * 0.5).min(0.99))
        } else {
            (0i8, 0.70)
        };

        let explanation = match k3_trit {
            1 => format!("TRM Brain: Approved (+1) with {:.1}% confidence after {} recursive steps.", confidence * 100.0, steps_taken),
            -1 => format!("TRM Brain: Rejected (-1) with {:.1}% confidence after {} recursive steps.", confidence * 100.0, steps_taken),
            _ => format!("TRM Brain: Neutral/Unknown (0) after {} recursive steps.", steps_taken),
        };

        ReasoningResult {
            final_state: current_state,
            k3_trit,
            confidence,
            steps_taken,
            halted,
            explanation,
        }
    }
}
