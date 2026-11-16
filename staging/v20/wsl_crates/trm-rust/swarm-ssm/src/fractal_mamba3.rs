//! Fractal Mamba-3 Prototype — Phase 1, Phase 2 & Phase 3
//! ============================================================================
//! Multi-scale State Space Model for Swarm OS Telemetry Modeling.
//!
//! Scale Hierarchy:
//!   - s0: Voltage    (sub-ms, Φ_V samples)       -> dt = 0.0001
//!   - s1: SRAM       (ms, SRAM pattern activity) -> dt = 0.001
//!   - s2: Agent      (seconds, agent decisions)  -> dt = 0.01
//!   - s3: Council    (tens of seconds, PBFT)      -> dt = 0.1
//!   - s4: Operations (minutes, watch loops)      -> dt = 1.0
//!   - s5: Knowledge  (hours/days, NAND shards)    -> dt = 10.0
//!
//! Phase 3 Fractal Coupling Formulation:
//!   h_s(t) = Ā_s h_s(t-1) + B̄_s x_s(t) + C_up * z_{s-1}(t) + C_down * w_{s+1}(t-1)
//!   k_s(t) = K3(h_s(t))
//!   H(t)   = Combine(k_0(t), ..., k_5(t))
//! ============================================================================

pub const SCALE_COUNT: usize = 6;
pub const STATE_DIM_PER_SCALE: usize = 32;
pub const TOTAL_STATE_DIM: usize = SCALE_COUNT * STATE_DIM_PER_SCALE; // 192

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Scale {
    Voltage = 0,    // s0: sub-ms
    Sram = 1,       // s1: ms
    Agent = 2,      // s2: seconds
    Council = 3,    // s3: 10s
    Operations = 4, // s4: minutes
    Knowledge = 5,  // s5: hours/days
}

impl Scale {
    pub const ALL: [Scale; SCALE_COUNT] = [
        Scale::Voltage,
        Scale::Sram,
        Scale::Agent,
        Scale::Council,
        Scale::Operations,
        Scale::Knowledge,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Scale::Voltage => "s0_Voltage",
            Scale::Sram => "s1_SRAM",
            Scale::Agent => "s2_Agent",
            Scale::Council => "s3_Council",
            Scale::Operations => "s4_Operations",
            Scale::Knowledge => "s5_Knowledge",
        }
    }

    pub fn default_dt(&self) -> f32 {
        match self {
            Scale::Voltage => 0.0001,
            Scale::Sram => 0.001,
            Scale::Agent => 0.01,
            Scale::Council => 0.1,
            Scale::Operations => 1.0,
            Scale::Knowledge => 10.0,
        }
    }
}

/// K3 Balanced Ternary Quantization: maps f32 to {-1, 0, +1}
#[inline]
pub fn k3_quantize(val: f32, threshold: f32) -> i8 {
    if val > threshold {
        1
    } else if val < -threshold {
        -1
    } else {
        0
    }
}

/// Exact Zero-Order Hold (ZOH) Discretization
pub fn zoh_discretize_1d(a: f32, b: f32, dt: f32) -> (f32, f32) {
    let a_bar = (dt * a).exp();
    let b_bar = ((a_bar - 1.0) / a) * b;
    (a_bar, b_bar)
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 1: Flat Mamba-3 Baseline (Single Scale)
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub struct FlatMamba3Baseline {
    pub h: [f32; STATE_DIM_PER_SCALE],
    pub a_bar: [f32; STATE_DIM_PER_SCALE],
    pub b_bar: [f32; STATE_DIM_PER_SCALE],
    pub w_out: [f32; STATE_DIM_PER_SCALE],
    pub b_out: f32,
    pub dt: f32,
    pub step_count: u64,
}

impl Default for FlatMamba3Baseline {
    fn default() -> Self {
        let dt = 0.01;
        let mut a_bar = [0.0; STATE_DIM_PER_SCALE];
        let mut b_bar = [0.0; STATE_DIM_PER_SCALE];
        for i in 0..STATE_DIM_PER_SCALE {
            let a = -0.5 * ((i + 1) as f32);
            let b = 1.0;
            let (ab, bb) = zoh_discretize_1d(a, b, dt);
            a_bar[i] = ab;
            b_bar[i] = bb;
        }
        Self {
            h: [0.0; STATE_DIM_PER_SCALE],
            a_bar,
            b_bar,
            w_out: [0.0; STATE_DIM_PER_SCALE],
            b_out: 0.5,
            dt,
            step_count: 0,
        }
    }
}

impl FlatMamba3Baseline {
    pub fn new(dt: f32) -> Self {
        let mut b = Self::default();
        b.dt = dt;
        for i in 0..STATE_DIM_PER_SCALE {
            let a = -0.5 * ((i + 1) as f32);
            let (ab, bb) = zoh_discretize_1d(a, 1.0, dt);
            b.a_bar[i] = ab;
            b.b_bar[i] = bb;
        }
        b
    }

    pub fn predict_next(&self) -> f32 {
        let mut pred = self.b_out;
        for i in 0..STATE_DIM_PER_SCALE {
            pred += self.w_out[i] * self.h[i];
        }
        pred
    }

    pub fn step(&mut self, x: f32, target_next: f32, lr: f32) -> (f32, i8, f32) {
        let pred = self.predict_next();
        let err = pred - target_next;

        let mut norm_sq = 1e-4f32;
        for i in 0..STATE_DIM_PER_SCALE {
            norm_sq += self.h[i] * self.h[i];
        }
        let step_lr = lr / norm_sq;

        for i in 0..STATE_DIM_PER_SCALE {
            self.w_out[i] -= (step_lr * err * self.h[i]).clamp(-0.5, 0.5);
            self.h[i] = self.a_bar[i] * self.h[i] + self.b_bar[i] * x;
        }
        self.b_out -= (lr * err).clamp(-0.2, 0.2);
        self.step_count += 1;

        let k3 = k3_quantize(pred - 0.5, 0.05);
        (pred, k3, err * err)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3: Fractal Coupled Mamba-3 (Hierarchical Cross-Scale SSM)
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub struct CoupledScaleState {
    pub scale: Scale,
    pub h: [f32; STATE_DIM_PER_SCALE],
    pub a_bar: [f32; STATE_DIM_PER_SCALE],
    pub b_bar: [f32; STATE_DIM_PER_SCALE],
    pub c_up: f32,   // Coupling weight from faster scale (s-1)
    pub c_down: f32, // Coupling weight from slower scale (s+1)
    pub w_out: [f32; STATE_DIM_PER_SCALE],
    pub b_out: f32,
    pub dt: f32,
}

impl CoupledScaleState {
    pub fn new(scale: Scale) -> Self {
        let dt = scale.default_dt();
        let mut a_bar = [0.0; STATE_DIM_PER_SCALE];
        let mut b_bar = [0.0; STATE_DIM_PER_SCALE];
        for i in 0..STATE_DIM_PER_SCALE {
            let a = -0.5 * ((i + 1) as f32);
            let (ab, bb) = zoh_discretize_1d(a, 1.0, dt);
            a_bar[i] = ab;
            b_bar[i] = bb;
        }
        Self {
            scale,
            h: [0.0; STATE_DIM_PER_SCALE],
            a_bar,
            b_bar,
            c_up: 0.15,
            c_down: 0.10,
            w_out: [0.0; STATE_DIM_PER_SCALE],
            b_out: 0.5,
            dt,
        }
    }

    pub fn state_summary(&self) -> f32 {
        let sum: f32 = self.h.iter().sum();
        sum / (STATE_DIM_PER_SCALE as f32)
    }
}

#[derive(Clone, Debug)]
pub struct FractalCoupledMamba3 {
    pub scales: [CoupledScaleState; SCALE_COUNT],
    pub step_count: u64,
}

impl Default for FractalCoupledMamba3 {
    fn default() -> Self {
        Self {
            scales: [
                CoupledScaleState::new(Scale::Voltage),
                CoupledScaleState::new(Scale::Sram),
                CoupledScaleState::new(Scale::Agent),
                CoupledScaleState::new(Scale::Council),
                CoupledScaleState::new(Scale::Operations),
                CoupledScaleState::new(Scale::Knowledge),
            ],
            step_count: 0,
        }
    }
}

impl FractalCoupledMamba3 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn predict_next(&self) -> ([f32; SCALE_COUNT], f32) {
        let mut scale_preds = [0.0; SCALE_COUNT];
        let mut sum_pred = 0.0;
        for s in 0..SCALE_COUNT {
            let sc = &self.scales[s];
            let mut p = sc.b_out;
            for i in 0..STATE_DIM_PER_SCALE {
                p += sc.w_out[i] * sc.h[i];
            }
            scale_preds[s] = p;
            sum_pred += p;
        }
        (scale_preds, sum_pred / (SCALE_COUNT as f32))
    }

    /// Fractal Coupled Step:
    /// Faster scales inject upward summary z_{s-1} into slower scales.
    /// Slower scales inject downward context w_{s+1} into faster scales.
    pub fn step(
        &mut self,
        inputs: [f32; SCALE_COUNT],
        targets_next: [f32; SCALE_COUNT],
        lr: f32,
    ) -> ([f32; SCALE_COUNT], f32, [i8; SCALE_COUNT], i8, f32) {
        let (preds, comp_pred) = self.predict_next();
        let mut k3_truths = [0i8; SCALE_COUNT];
        let mut total_sq_err = 0.0f32;

        // 1. Gather pre-transition summaries
        let mut summaries = [0.0f32; SCALE_COUNT];
        for s in 0..SCALE_COUNT {
            summaries[s] = self.scales[s].state_summary();
        }

        // 2. Update each scale with cross-scale fractal coupling
        for s in 0..SCALE_COUNT {
            let target = targets_next[s];
            let pred = preds[s];
            let err = pred - target;
            total_sq_err += err * err;

            // Normalized LMS update
            let mut norm_sq = 1e-4f32;
            for i in 0..STATE_DIM_PER_SCALE {
                norm_sq += self.scales[s].h[i] * self.scales[s].h[i];
            }
            let step_lr = lr / norm_sq;

            for i in 0..STATE_DIM_PER_SCALE {
                self.scales[s].w_out[i] -= (step_lr * err * self.scales[s].h[i]).clamp(-0.5, 0.5);
            }
            self.scales[s].b_out -= (lr * err).clamp(-0.2, 0.2);

            // Upward coupling from faster scale (s-1)
            let up_signal = if s > 0 { summaries[s - 1] * self.scales[s].c_up } else { 0.0 };

            // Downward coupling from slower scale (s+1)
            let down_signal = if s < SCALE_COUNT - 1 { summaries[s + 1] * self.scales[s].c_down } else { 0.0 };

            // Coupling input
            let coupled_input = inputs[s] + up_signal + down_signal;

            // State update
            for i in 0..STATE_DIM_PER_SCALE {
                self.scales[s].h[i] =
                    self.scales[s].a_bar[i] * self.scales[s].h[i] + self.scales[s].b_bar[i] * coupled_input;
            }

            k3_truths[s] = k3_quantize(pred - 0.5, 0.05);
        }

        self.step_count += 1;
        let final_k3 = k3_quantize(comp_pred - 0.5, 0.05);
        (preds, comp_pred, k3_truths, final_k3, total_sq_err / (SCALE_COUNT as f32))
    }

    pub fn concatenated_state(&self) -> [f32; TOTAL_STATE_DIM] {
        let mut flat = [0.0f32; TOTAL_STATE_DIM];
        for s in 0..SCALE_COUNT {
            let offset = s * STATE_DIM_PER_SCALE;
            for i in 0..STATE_DIM_PER_SCALE {
                flat[offset + i] = self.scales[s].h[i];
            }
        }
        flat
    }
}
