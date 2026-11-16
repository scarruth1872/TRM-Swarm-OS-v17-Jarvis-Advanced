//! TRM Swarm OS — sub_atomic_kernel (bare-metal no_std + alloc).
//! Physical Substrate Binding Layer: Voltage rails, clock lattice,
//! and batch wavefunction collapse mapped to Mamba-3 SSM + K3 ternary logic.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;

/// Native batch wavefunction collapse & energy discharge (bare-metal strip twin).
#[inline]
pub fn strip_batch_wavefunction_collapse(
    sigs: &[f32],
    eps: f32,
    out_trits: &mut [i8],
    out_discharges: &mut [f32],
) {
    let count = sigs.len().min(out_trits.len()).min(out_discharges.len());
    for i in 0..count {
        let v = sigs[i];
        if v > eps {
            out_trits[i] = 1i8;
            out_discharges[i] = 1.0f32;
        } else if v < -eps {
            out_trits[i] = -1i8;
            out_discharges[i] = 0.0f32;
        } else {
            out_trits[i] = 0i8;
            out_discharges[i] = 0.5f32;
        }
    }
}

/// Physical silicon substrate parameters bound to microkernel state.
#[derive(Debug, Clone)]
pub struct SiliconMatrix {
    pub v_core: f64,
    pub v_rail: f64,
    pub v_ddr: f64,
    pub f_cpu: f64,
    pub t_cycle: f64,
    pub phi_v: f64,
    pub delta_0: f64,
    pub epsilon: f64,
    pub delta_v: f64,
    pub clock_cycles_per_step: f64,
}

impl SiliconMatrix {
    pub fn new() -> Self {
        let v_core = 1.25;
        let v_rail = 4.75;
        let v_ddr = 1.35;
        let f_cpu = 4.0;
        let t_cycle = 0.25; // 1.0 / 4.0 GHz = 0.25 ns
        let phi_v = 0.6089;
        let delta_0 = 0.001;
        let epsilon = 0.15;
        let delta_v = delta_0 * phi_v;
        let clock_cycles_per_step = delta_v / t_cycle * 1000.0;

        SiliconMatrix {
            v_core,
            v_rail,
            v_ddr,
            f_cpu,
            t_cycle,
            phi_v,
            delta_0,
            epsilon,
            delta_v,
            clock_cycles_per_step,
        }
    }

    pub fn recalibrate(&mut self, v_core: Option<f64>, f_cpu: Option<f64>) -> f64 {
        if let Some(vc) = v_core {
            self.v_core = vc;
        }
        if let Some(fc) = f_cpu {
            self.f_cpu = fc;
        }
        let v_ratio = self.v_core / 1.25;
        let f_ratio = self.f_cpu / 4.0;
        let ddr_ratio = self.v_ddr / 1.35;
        self.phi_v = v_ratio * v_ratio * f_ratio * ddr_ratio;
        self.t_cycle = if self.f_cpu > 0.0 { 1.0 / self.f_cpu } else { 0.25 };
        self.delta_v = self.delta_0 * self.phi_v;
        self.clock_cycles_per_step = self.delta_v / self.t_cycle * 1000.0;
        self.phi_v
    }
}

/// Kleene K3 ternary gate at silicon level.
#[derive(Debug, Clone)]
pub struct K3TernaryGate {
    pub epsilon: f64,
    pub proceed_count: u64,
    pub analyze_count: u64,
    pub block_count: u64,
}

impl K3TernaryGate {
    pub fn new(epsilon: f64) -> Self {
        Self {
            epsilon,
            proceed_count: 0,
            analyze_count: 0,
            block_count: 0,
        }
    }

    pub fn collapse(&mut self, wavefunction: &[f64]) -> (Vec<i8>, (usize, usize, usize)) {
        let mut result = Vec::with_capacity(wavefunction.len());
        let mut proceed = 0usize;
        let mut analyze = 0usize;
        let mut block = 0usize;

        for &val in wavefunction {
            if val > self.epsilon {
                result.push(1i8);
                proceed += 1;
            } else if val < -self.epsilon {
                result.push(-1i8);
                block += 1;
            } else {
                result.push(0i8);
                analyze += 1;
            }
        }

        self.proceed_count += proceed as u64;
        self.analyze_count += analyze as u64;
        self.block_count += block as u64;

        (result, (proceed, analyze, block))
    }

    pub fn energy_discharge(&self, state: &[i8], signal_strength: f64) -> Vec<f64> {
        state.iter().map(|&s| {
            if s == 1 {
                signal_strength
            } else if s == -1 {
                0.0
            } else {
                signal_strength * 0.5
            }
        }).collect()
    }
}

/// Sub-Atomic Mamba-3 SSM Engine operating at physical silicon clock step.
pub struct SubAtomicMamba3Engine {
    pub si: SiliconMatrix,
    pub state_dim: usize,
    pub input_dim: usize,
    pub a: Vec<f64>,
    pub b: Vec<f64>,
    pub c: Vec<f64>,
    pub h: Vec<f64>,
    pub step_count: u64,
    pub total_clock_cycles: f64,
}

impl SubAtomicMamba3Engine {
    pub fn new(silicon: SiliconMatrix, state_dim: usize, input_dim: usize) -> Self {
        let mut a = vec![0.0f64; state_dim * state_dim];
        let mut b = vec![0.0f64; state_dim * input_dim];
        let mut c = vec![0.0f64; state_dim];

        // Deterministic pseudo-random seed init
        let mut lcg = 42u64;
        for item in a.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = (((lcg >> 32) as i32 % 1000) as f64 / 10000.0) - 0.05;
        }
        for item in b.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = (((lcg >> 32) as i32 % 1000) as f64 / 10000.0) - 0.05;
        }
        for item in c.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = (((lcg >> 32) as i32 % 1000) as f64 / 10000.0) - 0.05;
        }

        Self {
            si: silicon,
            state_dim,
            input_dim,
            a,
            b,
            c,
            h: vec![0.0f64; state_dim],
            step_count: 0,
            total_clock_cycles: 0.0,
        }
    }

    pub fn step(&mut self, input_signal: &[f64], gate: &mut K3TernaryGate) -> (f64, i8, f64) {
        let (collapsed, _counts) = gate.collapse(input_signal);
        let energy = gate.energy_discharge(&collapsed, 1.0);

        let dt = self.si.delta_v;
        let mut h_next = vec![0.0f64; self.state_dim];

        // h_new = h * (I + A * dt) + B * energy * dt
        for i in 0..self.state_dim {
            let mut sum_a = 0.0f64;
            for j in 0..self.state_dim {
                let delta = if i == j { 1.0 } else { 0.0 };
                let a_val = self.a[i * self.state_dim + j];
                sum_a += self.h[j] * (delta + a_val * dt);
            }

            let mut sum_b = 0.0f64;
            let count_in = self.input_dim.min(energy.len());
            for k in 0..count_in {
                sum_b += self.b[i * self.input_dim + k] * energy[k];
            }

            h_next[i] = sum_a + sum_b * dt;
        }
        self.h = h_next;

        // Readout: output = h @ C
        let mut output = 0.0f64;
        for i in 0..self.state_dim {
            output += self.h[i] * self.c[i];
        }

        // Dominant ternary state
        let dominant_trit = if output > gate.epsilon {
            1i8
        } else if output < -gate.epsilon {
            -1i8
        } else {
            0i8
        };

        let cycles = self.si.clock_cycles_per_step;
        self.step_count += 1;
        self.total_clock_cycles += cycles;

        (output, dominant_trit, cycles)
    }
}

static SILICON_CALLS: AtomicU64 = AtomicU64::new(0);

pub fn get_silicon_matrix() -> SiliconMatrix {
    SILICON_CALLS.fetch_add(1, Ordering::Relaxed);
    SiliconMatrix::new()
}

pub fn recalibrate_silicon(v_core: f64, f_cpu: f64) -> f64 {
    let mut matrix = SiliconMatrix::new();
    matrix.recalibrate(Some(v_core), Some(f_cpu))
}

pub fn step_subatomic_mamba(inputs: &[f64]) -> (f64, i8) {
    let matrix = SiliconMatrix::new();
    let mut engine = SubAtomicMamba3Engine::new(matrix, 32, 96);
    let mut gate = K3TernaryGate::new(0.15);
    let (out, trit, _) = engine.step(inputs, &mut gate);
    (out, trit)
}

pub fn batch_wavefunction_collapse(sigs: &[f32], eps: f32) -> (Vec<i8>, Vec<f32>) {
    let n = sigs.len();
    let mut trits = vec![0i8; n];
    let mut discharges = vec![0.0f32; n];
    strip_batch_wavefunction_collapse(sigs, eps, &mut trits, &mut discharges);
    (trits, discharges)
}

