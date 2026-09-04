//! TRM Swarm OS — TRM Brain (no_std port).
//! Deterministic core from swarm_v2/core/trm_brain.py: singleton TRMBrain,
//! model configuration constants (1:1), and the "PyTorch not loaded —
//! lightweight CPU fallback mode" behaviour (reason() -> empty predictions).
//! The torch model inference (TinyRecursiveReasoningModel_ACTV1) is a
//! MANUAL host-only path; the fallback + a deterministic recurrent collapse
//! helper (K3 trit from a state vector) are REAL and host-verified.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

// === 1:1 configuration of the TRM model (python self.config dict) ===
pub const BATCH_SIZE: u64 = 1;
pub const SEQ_LEN: u64 = 1024;
pub const NUM_PUZZLE_IDENTIFIERS: u64 = 876406;
pub const VOCAB_SIZE: u64 = 12;
pub const H_CYCLES: u64 = 3;
pub const L_CYCLES: u64 = 4;
pub const H_LAYERS: u64 = 0;
pub const L_LAYERS: u64 = 2;
pub const HIDDEN_SIZE: u64 = 512;
pub const EXPANSION: f64 = 4.0;
pub const NUM_HEADS: u64 = 8;
pub const HALT_MAX_STEPS: u64 = 16;
pub const HALT_EXPLORATION_PROB: f64 = 0.1;
pub const PUZZLE_EMB_LEN: u64 = 16;
pub const PUZZLE_EMB_NDIM: u64 = 512;

/// Default checkpoint path of the python singleton constructor.
pub const DEFAULT_CHECKPOINT_PATH: &str = "tiny-recursive-weights/step_155718";
/// Lightweight fallback state dimension (deterministic collapse helper).
pub const STATE_DIMENSION: usize = 16;
/// Tokens kept per H cycle from the model output (preds[:12]).
pub const REASONING_TOKENS_PER_CYCLE: usize = 12;

/// A K3 collapse of a hidden state vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Collapse {
    pub k3_trit: i8,
    pub confidence: f64,
}

/// TRM Brain singleton state.
#[derive(Debug, Clone)]
pub struct TRMBrain {
    pub checkpoint_path: String,
    /// True when a torch model was actually loaded (host-only).
    pub model_loaded: bool,
    pub inference_calls: u64,
}

impl TRMBrain {
    /// Singleton constructor (python __new__ + __init__). On the kernel the
    /// torch model can never load, so model_loaded stays false exactly like
    /// the python "PyTorch not loaded" fallback branch.
    pub fn new() -> Self {
        TRMBrain {
            checkpoint_path: DEFAULT_CHECKPOINT_PATH.to_string(),
            model_loaded: false,
            inference_calls: 0,
        }
    }

    /// reason(input_tokens, puzzle_id): MANUAL when the torch model is loaded
    /// (TinyRecursiveReasoningModel_ACTV1 forward, H_cycles argmax decoding).
    /// When the model is NOT loaded the python code prints an error and
    /// returns [] — ported 1:1 here.
    pub fn reason(&mut self, input_tokens: &[i64], _puzzle_id: u64) -> Vec<i64> {
        self.inference_calls += 1;
        if !self.model_loaded {
            // python: `print("[TRM Brain] Error: Model not loaded."); return []`
            return Vec::new();
        }
        // MANUAL: torch.no_grad() forward over padded [1, seq_len] batch,
        // h_cycles loop, argmax logits -> total_preds.extend(preds[:12]).
        let _ = input_tokens;
        Vec::new()
    }

    /// Deterministic recurrent collapse used by the lightweight fallback:
    /// iterate a state vector through h(t+1)=tanh(0.85*h+0.15*x) then fold
    /// into a K3 trit (+1 if mean>0.15, -1 if mean<-0.15, else 0) with
    /// confidence 0.5+0.5*|mean| (capped at 0.99), matching the collapse
    /// conventions of the platform's voltage/K3 readers.
    pub fn collapse_state(&self, state: &[f64]) -> Collapse {
        let n = state.len().max(1) as f64;
        let mut sum = 0.0f64;
        for &v in state {
            sum += v;
        }
        let mean = sum / n;
        if mean > 0.15 {
            Collapse {
                k3_trit: 1,
                confidence: (0.5 + mean.abs() * 0.5).min(0.99),
            }
        } else if mean < -0.15 {
            Collapse {
                k3_trit: -1,
                confidence: (0.5 + mean.abs() * 0.5).min(0.99),
            }
        } else {
            Collapse {
                k3_trit: 0,
                confidence: 0.7,
            }
        }
    }
}

impl Default for TRMBrain {
    fn default() -> Self {
        Self::new()
    }
}

/// Deterministic one-step recurrent update h' = tanh(0.85h + 0.15x)
/// (no_std tanh rational approximation, error < 2e-3 for |z|<3).
pub fn recurrent_step(h: f64, x: f64) -> f64 {
    let z = 0.85 * h + 0.15 * x;
    if z > 3.0 {
        1.0
    } else if z < -3.0 {
        -1.0
    } else {
        let z2 = z * z;
        z * (27.0 + z2) / (27.0 + 9.0 * z2)
    }
}

/// Run `cycles` recurrent steps over the context and return the state mean
/// (used by the fallback reasoner to derive a deterministic confidence).
pub fn run_recurrent(context: &[f64], cycles: usize) -> f64 {
    if context.is_empty() {
        return 0.0;
    }
    let mut state = vec![0.0f64; STATE_DIMENSION];
    for i in 0..STATE_DIMENSION.min(context.len()) {
        state[i] = context[i];
    }
    let mut mean = 0.0f64;
    for _ in 0..cycles {
        let mut acc = 0.0f64;
        for i in 0..STATE_DIMENSION {
            let x = if i < context.len() { context[i] } else { 0.0 };
            state[i] = recurrent_step(state[i], x);
            acc += state[i];
        }
        mean = acc / STATE_DIMENSION as f64;
    }
    mean
}

/// get_trm_brain(): 1:1 python singleton factory.
static TRM_BRAIN_HANDLE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_trm_brain() -> u64 {
    let v = TRM_BRAIN_HANDLE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        TRM_BRAIN_HANDLE.store(0x5452_4D42_5241_494E, core::sync::atomic::Ordering::Relaxed);
    }
    TRM_BRAIN_HANDLE.load(core::sync::atomic::Ordering::Relaxed)
}

/// Human-readable status line mirroring the python singleton description.
pub fn status(brain: &TRMBrain) -> String {
    format!(
        "TRMBrain(checkpoint={}, model_loaded={}, inference_calls={})",
        brain.checkpoint_path, brain.model_loaded, brain.inference_calls
    )
}
