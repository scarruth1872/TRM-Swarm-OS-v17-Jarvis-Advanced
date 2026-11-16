//! TRM Swarm OS - small_diffusion_engine (no_std port of
//! swarm_v2/core/small_diffusion_engine.py).
//!
//! PORTED (deterministic arithmetic only):
//!   * `native_wavefront_denoise` - the pure-Python fallback path of the strip wrapper:
//!     `guidance = -0.2 if D_H > 1.5 else 0.1`, `updated = latents + guidance`,
//!     `trits = +1 where updated > 0.3`, `-1 where updated < -0.3`, else 0.
//!     Ported in f32 because the oracle compares float32 arrays against float32
//!     thresholds there (a f64 port flips the `updated > 0.3` decision for an input
//!     of exactly 0.2f32 - see the probe in the campaign notes).
//!   * `hip_fast_diffusion_kernel` - the oracle body is literally
//!     `return native_wavefront_denoise(noisy_latents, D_H)`.
//!   * `FlowMatchingVelocityField::predict_velocity` - the 128-hidden ReLU MLP plus
//!     the classifier-free-guidance combination
//!     `v_cfg = v_uncond + s * (v_cond - v_uncond)` with
//!     `v_uncond = v_pred*0.5`, `v_cond = v_pred + cond_vec*D_H_norm`, and
//!     `s = max(0.5, guidance_scale*(2.0 - D_H))` when `D_H > 1.5`.
//!   * `SmallDiffusionEngine::denoise` - the 1..4 step Euler loop (`t = step/steps`,
//!     `dt = 1/steps`, decreasing NOISE_SCHEDULE residual noise), the early-exit test
//!     `mean(|z|) > 0.15`, the Kleene quantization `z > 0.2 -> +1`, `z < -0.2 -> -1`,
//!     and the `num_pass / num_fault / num_ambiguous / confidence` reductions.
//!
//! NOT PORTED (host-side in the oracle, injected here instead):
//!   * the native strip `strip_diffusion_wavefront.dll` (`ctypes.CDLL`) - UEFI has no DLL
//!     loading; the port implements the wrapper's pure-Python fallback that the oracle
//!     itself uses when the strip is absent;
//!   * `np.random.RandomState(42)` weight initialization (W1/b1/W2/b2/cond_vec are
//!     injected as slices - numpy's Mersenne Twister is not reproduced here);
//!   * `np.random.randn(dim)` residual noise draws (injected per step as `residual_noise`);
//!   * `time.perf_counter_ns()` latency accounting (`latency_us` is left 0.0),
//!     `threading.Lock` / singleton / `get_stats` string formatting / logging.
//!
//! math base: only `abs`, `max` and comparisons are needed - no sqrt/exp/ln/powf, so
//! nothing had to be hand-rolled. Accumulators are f64 (the oracle accumulates in f32
//! numpy arrays); the measured residual is reported in the campaign notes.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

// === Diffusion Model Parameters (oracle lines 57-61) ===
pub const LATENT_DIM: u64 = 256;
pub const NUM_STEPS: u64 = 4; // 1-to-4 step denoising
pub const GUIDANCE_SCALE: f64 = 1.5; // CFG scale (modulated by D_H)
pub const NOISE_SCHEDULE: &'static [f64] = &[0.8, 0.5, 0.2, 0.05]; // Decreasing noise per step
pub const HIDDEN_DIM: usize = 128; // W1 shape (128, dim+2), W2 shape (dim, 128)

// === DiffusionOutput (oracle lines 64-74) ===
#[derive(Debug, Clone)]
pub struct DiffusionOutput {
    /// 256-dim Kleene trits {-1, 0, +1} (oracle: `np.ndarray` of int8).
    pub denoised_trits: Vec<i8>,
    pub num_pass: i64,      // Count of +1 (execute)
    pub num_fault: i64,     // Count of -1 (reject)
    pub num_ambiguous: i64, // Count of 0 (still ambiguous)
    pub confidence: f64,    // Mean absolute confidence
    pub steps_used: i64,    // How many denoising steps were needed
    pub latency_us: f64,    // Host-side wall clock in the oracle; left 0.0 here
    pub early_exit: bool,   // Did we converge early?
}

impl DiffusionOutput {
    pub fn new() -> Self {
        DiffusionOutput {
            denoised_trits: Vec::new(),
            num_pass: 0,
            num_fault: 0,
            num_ambiguous: 0,
            confidence: 0.0,
            steps_used: 0,
            latency_us: 0.0,
            early_exit: false,
        }
    }
}

// === native_wavefront_denoise (oracle lines 39-53), pure-Python fallback path ===
/// Native Wavefront fast 1-step denoising kernel's Python fallback.
///
/// Oracle:
/// ```text
/// guidance = -0.2 if D_H > 1.5 else 0.1
/// updated  = latents.astype(float32) + guidance
/// trits    = 0; trits[updated >  0.3] =  1; trits[updated < -0.3] = -1
/// ```
/// Arithmetic is kept in f32 to mirror the oracle's float32 array/threshold dtype.
#[allow(non_snake_case)]
pub fn native_wavefront_denoise(latents: &[f32], D_H: f64) -> Vec<i8> {
    let guidance: f32 = if D_H > 1.5 { -0.2f32 } else { 0.1f32 };
    let mut trits: Vec<i8> = vec![0i8; latents.len()];
    for i in 0..latents.len() {
        let updated = latents[i] + guidance;
        if updated > 0.3f32 {
            trits[i] = 1;
        } else if updated < -0.3f32 {
            trits[i] = -1;
        }
    }
    trits
}

// === hip_fast_diffusion_kernel (oracle lines 247-261) ===
/// Python reference for the ROCm HIP fast 1-step denoising kernel; the oracle body is
/// `return native_wavefront_denoise(noisy_latents, D_H)` (the DLL when loaded, the
/// fallback above otherwise).
#[allow(non_snake_case)]
pub fn hip_fast_diffusion_kernel(noisy_latents: &[f32], D_H: f64) -> Vec<i8> {
    native_wavefront_denoise(noisy_latents, D_H)
}

// === FlowMatchingVelocityField (oracle lines 77-131) ===
/// Flow-matching velocity predictor: v_theta(z_t, t, D_H).
///
/// The oracle builds the weights with `np.random.RandomState(42)`; that RNG is
/// host-side and is NOT reproduced here - weights are injected as row-major slices.
pub struct FlowMatchingVelocityField {
    pub dim: usize,
    /// (128, dim+2) row-major: last two columns are the t and D_H conditioning inputs.
    pub w1: Vec<f64>,
    pub b1: Vec<f64>,
    /// (dim, 128) row-major.
    pub w2: Vec<f64>,
    pub b2: Vec<f64>,
    /// (dim,) classifier-free-guidance conditioning vector.
    pub cond_vec: Vec<f64>,
}

impl FlowMatchingVelocityField {
    pub fn new(dim: usize, w1: Vec<f64>, b1: Vec<f64>, w2: Vec<f64>, b2: Vec<f64>, cond_vec: Vec<f64>) -> Self {
        FlowMatchingVelocityField { dim, w1, b1, w2, b2, cond_vec }
    }

    /// Predict v_theta: the velocity (denoising direction) for the current latent.
    ///
    /// Oracle:
    /// ```text
    /// t_norm = t / NUM_STEPS;  D_H_norm = (D_H - 1.0) / 1.0
    /// x = concat([z_t, [t_norm, D_H_norm]])
    /// h = relu(W1 @ x + b1);  v_pred = W2 @ h + b2
    /// v_uncond = v_pred * 0.5
    /// v_cond   = v_pred + cond_vec * D_H_norm
    /// s = max(0.5, guidance_scale*(2.0 - D_H))  if D_H > 1.5  else guidance_scale
    /// v_cfg = v_uncond + s * (v_cond - v_uncond)
    /// ```
    #[allow(non_snake_case)]
    pub fn predict_velocity(&self, z_t: &[f64], t: f64, D_H: f64, guidance_scale: f64) -> Vec<f64> {
        let dim = self.dim;
        let in_dim = dim + 2;

        // Normalize timestep to [0, 1]; D_H in [1.0, 2.0] -> D_H_norm in [0, 1].
        let t_norm = t / (NUM_STEPS as f64);
        let d_h_norm = (D_H - 1.0) / 1.0;

        // h = relu(W1 @ x + b1), x = [z_t, t_norm, D_H_norm]
        let mut h = vec![0.0f64; HIDDEN_DIM];
        for j in 0..HIDDEN_DIM {
            let row = &self.w1[j * in_dim..(j + 1) * in_dim];
            let mut acc = self.b1[j];
            for k in 0..dim {
                acc += row[k] * z_t[k];
            }
            acc += row[dim] * t_norm;
            acc += row[dim + 1] * d_h_norm;
            h[j] = if acc > 0.0 { acc } else { 0.0 }; // ReLU
        }

        // v_pred = W2 @ h + b2
        let mut v_pred = vec![0.0f64; dim];
        for i in 0..dim {
            let row = &self.w2[i * HIDDEN_DIM..(i + 1) * HIDDEN_DIM];
            let mut acc = self.b2[i];
            for k in 0..HIDDEN_DIM {
                acc += row[k] * h[k];
            }
            v_pred[i] = acc;
        }

        // Classifier-free guidance: v = v_uncond + s * (v_cond - v_uncond)
        let s = if D_H > 1.5 {
            let m = guidance_scale * (2.0 - D_H);
            if m > 0.5 { m } else { 0.5 }
        } else {
            guidance_scale
        };

        let mut v_cfg = vec![0.0f64; dim];
        for i in 0..dim {
            let v_uncond = v_pred[i] * 0.5;
            let v_cond = v_pred[i] + self.cond_vec[i] * d_h_norm;
            v_cfg[i] = v_uncond + s * (v_cond - v_uncond);
        }
        v_cfg
    }
}

// === SmallDiffusionEngine (oracle lines 134-243) ===
pub struct SmallDiffusionEngine {
    pub dim: usize,
    pub max_steps: usize,
    pub noise_schedule: &'static [f64],
    pub velocity_field: FlowMatchingVelocityField,
    /// Ambiguity threshold: if mean_abs(z) exceeds this, stop early.
    pub early_exit_threshold: f64,
    // Stats (oracle: calls / total_us / early_exits, guarded by threading.Lock).
    pub calls: u64,
    pub early_exits: u64,
}

impl SmallDiffusionEngine {
    pub fn new(velocity_field: FlowMatchingVelocityField) -> Self {
        SmallDiffusionEngine {
            dim: LATENT_DIM as usize,
            max_steps: NUM_STEPS as usize,
            noise_schedule: NOISE_SCHEDULE,
            velocity_field,
            early_exit_threshold: 0.15,
            calls: 0,
            early_exits: 0,
        }
    }

    /// `early_exits / calls * 100` - the numeric part of the oracle's
    /// `f"{self.early_exits/max(1,self.calls)*100:.1f}%"` stat.
    pub fn early_exit_rate_pct(&self) -> f64 {
        (self.early_exits as f64) / (if self.calls == 0 { 1 } else { self.calls } as f64) * 100.0
    }

    /// Denoise a Kleene ternary latent vector.
    ///
    /// `noisy_latent` must hold at least `self.dim` values (the oracle slices
    /// `noisy_latent[:self.dim]`, which raises on a shorter input).
    ///
    /// `residual_noise` supplies the oracle's `np.random.randn(dim).astype(float32)`
    /// residual draws, step-major: row `step` is `residual_noise[step*dim .. (step+1)*dim]`.
    /// Rows beyond the slice are treated as zero.
    #[allow(non_snake_case)]
    pub fn denoise(
        &mut self,
        noisy_latent: &[f32],
        D_H: f64,
        max_steps: Option<usize>,
        early_exit: bool,
        residual_noise: &[f32],
    ) -> DiffusionOutput {
        let dim = self.dim;
        assert!(noisy_latent.len() >= dim, "noisy_latent shorter than LATENT_DIM");
        let steps = max_steps.unwrap_or(self.max_steps);

        // z = noisy_latent[:dim].astype(float32).copy()
        let mut z: Vec<f64> = vec![0.0f64; dim];
        for i in 0..dim {
            z[i] = noisy_latent[i] as f64;
        }

        let mut steps_used: i64 = 0;
        let mut did_early_exit = false;
        let sched_len = self.noise_schedule.len();

        for step in 0..steps {
            let t = (step as f64) / (steps as f64);
            let sigma = self.noise_schedule[if step < sched_len { step } else { sched_len - 1 }];

            // Predict velocity (denoising direction)
            let v = self.velocity_field.predict_velocity(&z, t, D_H, GUIDANCE_SCALE);

            // Euler step: z_{t+1} = z_t + v * (1/N)
            let dt = 1.0 / (steps as f64);
            for i in 0..dim {
                z[i] += v[i] * dt;
            }

            // Add residual noise scaled by schedule
            let base = step * dim;
            for i in 0..dim {
                let n = if base + i < residual_noise.len() { residual_noise[base + i] as f64 } else { 0.0 };
                z[i] += n * sigma * 0.1;
            }

            steps_used += 1;

            // Early exit: if state is mostly resolved
            if early_exit {
                let mut sum = 0.0f64;
                for i in 0..dim {
                    sum += if z[i] < 0.0 { -z[i] } else { z[i] };
                }
                if sum / (dim as f64) > self.early_exit_threshold {
                    did_early_exit = true;
                    break;
                }
            }
        }

        // Quantize to Kleene trits
        let mut trits: Vec<i8> = vec![0i8; dim];
        let mut n_pass: i64 = 0;
        let mut n_fail: i64 = 0;
        let mut abs_sum = 0.0f64;
        for i in 0..dim {
            if z[i] > 0.2 {
                trits[i] = 1;
                n_pass += 1;
            } else if z[i] < -0.2 {
                trits[i] = -1;
                n_fail += 1;
            }
            abs_sum += if z[i] < 0.0 { -z[i] } else { z[i] };
        }
        let n_ambi = (dim as i64) - n_pass - n_fail;
        let confidence = abs_sum / (dim as f64);

        self.calls += 1;
        if did_early_exit {
            self.early_exits += 1;
        }

        DiffusionOutput {
            denoised_trits: trits,
            num_pass: n_pass,
            num_fault: n_fail,
            num_ambiguous: n_ambi,
            confidence,
            steps_used,
            latency_us: 0.0, // host-side perf_counter_ns(); not measured here
            early_exit: did_early_exit,
        }
    }
}
