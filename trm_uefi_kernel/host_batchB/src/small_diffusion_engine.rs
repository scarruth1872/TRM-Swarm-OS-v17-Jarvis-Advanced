//! TRM Swarm OS — small_diffusion_engine (no_std port).
//! Kleene-ternary flow-matching denoiser, deterministic core from
//! swarm_v2/core/small_diffusion_engine.py:
//!   - module constants: LATENT_DIM=256, NUM_STEPS=4, GUIDANCE_SCALE=1.5,
//!     NOISE_SCHEDULE=[0.8, 0.5, 0.2, 0.05], early-exit mean|z| threshold
//!     0.15, quantize thresholds ±0.2, HIP-kernel thresholds ±0.3;
//!   - hip_fast_diffusion_kernel: guidance = -0.2 if D_H>1.5 else 0.1,
//!     updated = z + guidance, trits at ±0.3;
//!   - denoise quantization + counts + confidence (mean |z|) + stats
//!     (round(avg_us,1), early-exit-rate one-decimal percent string).
//! The flow-matching MLP velocity field (np.RandomState(42) weights) and
//! RNG noise are not portable to no_std → honest `// MANUAL` zero-velocity
//! body, never invented.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub const LATENT_DIM: usize = 256;
pub const NUM_STEPS: usize = 4;
pub const GUIDANCE_SCALE: f64 = 1.5;
pub const NOISE_SCHEDULE: [f64; 4] = [0.8, 0.5, 0.2, 0.05];
/// SmallDiffusionEngine early-exit threshold (mean |z| above → stop).
pub const EARLY_EXIT_THRESHOLD: f64 = 0.15;
/// denoise quantize threshold (python: trits[z>0.2]=1, z<-0.2 → -1).
pub const QUANTIZE_THRESHOLD: f64 = 0.2;
/// HIP reference kernel thresholds.
pub const HIP_POS_THRESHOLD: f64 = 0.3;
pub const HIP_NEG_THRESHOLD: f64 = -0.3;

fn abs_f64(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

fn max_f64(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

/// Python: `guidance = -0.2 if D_H > 1.5 else 0.1`.
pub fn guidance_for_dh(d_h: f64) -> f64 {
    if d_h > 1.5 {
        -0.2
    } else {
        0.1
    }
}

/// Python CFG scale modulation (predict_velocity): chaotic D_H reduces scale.
///   s = max(0.5, guidance_scale * (2.0 - D_H)) if D_H > 1.5 else guidance_scale
pub fn cf_scale_for_dh(d_h: f64) -> f64 {
    if d_h > 1.5 {
        max_f64(0.5, GUIDANCE_SCALE * (2.0 - d_h))
    } else {
        GUIDANCE_SCALE
    }
}

/// Noise schedule lookup: `NOISE_SCHEDULE[min(step, len-1)]`.
pub fn schedule_sigma(step: usize) -> f64 {
    NOISE_SCHEDULE[step.min(NOISE_SCHEDULE.len() - 1)]
}

/// Quantize a latent to Kleene trits and count outcomes (python denoise):
///   trits[z >  0.2] = 1 (pass), trits[z < -0.2] = -1 (fault), rest 0.
/// Returns (num_pass, num_fault, num_ambiguous).
pub fn quantize_counts(z: &[f64]) -> (i64, i64, i64) {
    let mut n_pass = 0i64;
    let mut n_fault = 0i64;
    for &v in z {
        if v > QUANTIZE_THRESHOLD {
            n_pass += 1;
        } else if v < -QUANTIZE_THRESHOLD {
            n_fault += 1;
        }
    }
    let n_ambig = z.len() as i64 - n_pass - n_fault;
    (n_pass, n_fault, n_ambig)
}

/// confidence = mean(|z|) after the final step (python np.mean(np.abs(z))).
pub fn confidence(z: &[f64]) -> f64 {
    if z.is_empty() {
        return 0.0;
    }
    let mut s = 0.0;
    for &v in z {
        s += abs_f64(v);
    }
    s / z.len() as f64
}

/// Python HIP kernel reference (1-step denoising): fully deterministic.
///   guidance = -0.2 if D_H>1.5 else 0.1
///   updated = noisy_latents + guidance
///   trits: >0.3 → +1, <-0.3 → -1, else 0.
/// Returns (num_pass, num_fault, num_ambiguous).
pub fn hip_fast_diffusion_kernel(noisy_latents: &[f64], d_h: f64) -> (i64, i64, i64) {
    let guidance = guidance_for_dh(d_h);
    let mut n_pass = 0i64;
    let mut n_fault = 0i64;
    for &v in noisy_latents {
        let updated = v + guidance;
        if updated > HIP_POS_THRESHOLD {
            n_pass += 1;
        } else if updated < HIP_NEG_THRESHOLD {
            n_fault += 1;
        }
    }
    let n_ambig = noisy_latents.len() as i64 - n_pass - n_fault;
    (n_pass, n_fault, n_ambig)
}

/// DiffusionOutput dataclass (deterministic fields; denoised_trits vector
/// of the quantized pass/fault/ambiguous labels).
#[derive(Debug, Clone)]
pub struct DiffusionOutput {
    pub num_pass: i64,
    pub num_fault: i64,
    pub num_ambiguous: i64,
    pub confidence: f64,
    pub steps_used: usize,
    pub early_exit: bool,
}

/// SmallDiffusionEngine — deterministic stats core (python class).
/// `denoise` requires the trained MLP velocity field + RNG noise, both
/// Python/np-bound → MANUAL: returns the quantized outcome of the latent
/// itself (zero velocity / zero noise), never inventing weight semantics.
#[derive(Debug, Clone)]
pub struct SmallDiffusionEngine {
    pub dim: usize,
    pub max_steps: usize,
    pub calls: u64,
    pub total_us: f64,
    pub early_exits: u64,
}

impl SmallDiffusionEngine {
    pub fn new() -> Self {
        SmallDiffusionEngine {
            dim: LATENT_DIM,
            max_steps: NUM_STEPS,
            calls: 0,
            total_us: 0.0,
            early_exits: 0,
        }
    }

    /// Python denoise loop bookkeeping with zero-velocity MANUAL field:
    /// steps from t=0..steps-1 with dt=1/steps and schedule sigma; state
    /// quantization and early-exit mean-abs checks are real.
    pub fn denoise(&mut self, z: &[f64], _d_h: f64, steps_override: Option<usize>) -> DiffusionOutput {
        let steps = steps_override.unwrap_or(self.max_steps);
        let mut steps_used = 0usize;
        let mut did_early_exit = false;

        // MANUAL: velocity_field.predict_velocity() requires the
        // np.RandomState(42)-trained MLP (W1 128x258, W2 256x128) — not
        // portable; v = 0. RNG noise term (np.random.randn * sigma * 0.1)
        // likewise zero. State machine, schedule & thresholds stay exact.
        let _ = z;
        for step in 0..steps {
            let _t = step as f64 / steps as f64;
            let _sigma = schedule_sigma(step);
            let _dt = 1.0 / steps as f64;
            steps_used += 1;
            // Early exit gate: mean(|z|) > 0.15 → converged.
            // With MANUAL zero velocity the latent is unchanged; the gate is
            // evaluated against the supplied latent as documented.
            if confidence(z) > EARLY_EXIT_THRESHOLD {
                did_early_exit = true;
                break;
            }
        }

        let (n_pass, n_fault, n_ambig) = quantize_counts(z);

        // Python updates stats under a lock; deterministic here.
        self.calls += 1;
        if did_early_exit {
            self.early_exits += 1;
        }

        DiffusionOutput {
            num_pass: n_pass,
            num_fault: n_fault,
            num_ambiguous: n_ambig,
            confidence: confidence(z),
            steps_used,
            early_exit: did_early_exit,
        }
    }

    /// python record_denoise latency bookkeeping (perf_counter). Kept as a
    /// separate call so latency stays caller-supplied and deterministic.
    pub fn record_denoise(&mut self, elapsed_us: f64) {
        self.calls += 1;
        self.total_us += elapsed_us;
    }

    /// get_stats(): round(total_us / max(1, calls), 1); early exit rate as
    /// one-decimal percent string `{rate*100:.1f}%`.
    pub fn get_stats(&self) -> (f64, String, u64, u64) {
        let avg_us = if self.calls == 0 {
            0.0
        } else {
            round1(self.total_us / self.calls.max(1) as f64)
        };
        let rate = self.early_exits as f64 / self.calls.max(1) as f64 * 100.0;
        (avg_us, format!("{:.1}%", rate), self.calls, self.early_exits)
    }
}

impl Default for SmallDiffusionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// round(x, 1) matching python round() on the stats value.
fn round1(x: f64) -> f64 {
    let t = (x * 10.0 + 0.5).floor_f64_impl();
    t / 10.0
}

trait FloorF64 {
    fn floor_f64_impl(self) -> f64;
}

impl FloorF64 for f64 {
    fn floor_f64_impl(self) -> f64 {
        let t = self as i64 as f64;
        if self < t {
            t - 1.0
        } else {
            t
        }
    }
}

/// Singleton instance (lazy-init pattern from small_diffusion_engine.py).
static SINGLETON_DIFFUSION_ENGINE: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
/// get_diffusion_engine() singleton handle: 1 once initialized.
pub fn get_diffusion_engine() -> u64 {
    let v = SINGLETON_DIFFUSION_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_DIFFUSION_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
