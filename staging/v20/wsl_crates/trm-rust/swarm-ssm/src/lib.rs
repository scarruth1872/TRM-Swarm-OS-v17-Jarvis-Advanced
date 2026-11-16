//! swarm-ssm — Mamba-3 state evolution in Rust.
//!
//! Port of c_mamba3_create_state / c_mamba3_step from the C++ kernel.
//! Semantics (verified against the C source):
//!   state: h[STATE_DIM], A=-0.5, B=1.0, C=1.0, dt=0.001 (or dt_init)
//!   step:  dA = 1.0 + A*dt
//!          h_new[i] = h[i]*dA + B[i]*input*dt
//!          y = sum_i(h_new[i] * C[i])
//!   step_count++
//!
//! Verify gate: identical y output and h evolution vs c_mamba3_step on
//! the same input sequence (FFI comparison).

pub const MAMBA_STATE_DIM: usize = 32; // VERIFIED against swarm_kernel_ops.h

#[derive(Clone, Debug)]
pub struct Mamba3State {
    pub h: [f32; MAMBA_STATE_DIM],
    pub a: [f32; MAMBA_STATE_DIM],
    pub b: [f32; MAMBA_STATE_DIM],
    pub c: [f32; MAMBA_STATE_DIM],
    pub dt: f32,
    pub step_count: u64,
}

impl Default for Mamba3State {
    fn default() -> Self {
        let mut s = Mamba3State {
            h: [0.0; MAMBA_STATE_DIM],
            a: [-0.5; MAMBA_STATE_DIM],
            b: [1.0; MAMBA_STATE_DIM],
            c: [1.0; MAMBA_STATE_DIM],
            dt: 0.001,
            step_count: 0,
        };
        s
    }
}

impl Mamba3State {
    pub fn create(dt_init: Option<f32>) -> Self {
        let mut s = Self::default();
        if let Some(dt) = dt_init {
            s.dt = dt;
        }
        s
    }

    /// One SSM step: h' = h*dA + B*x*dt ; y = Σ(h'*C)
    pub fn step(&mut self, input_x: f32) -> f32 {
        let dt = self.dt;
        let mut y_accum = 0.0f32;
        for i in 0..MAMBA_STATE_DIM {
            let d_a = 1.0 + self.a[i] * dt;
            let h_new = self.h[i] * d_a + self.b[i] * input_x * dt;
            self.h[i] = h_new;
            y_accum += h_new * self.c[i];
        }
        self.step_count += 1;
        y_accum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_matches_c_defaults() {
        let s = Mamba3State::create(None);
        assert_eq!(s.dt, 0.001);
        assert!(s.a.iter().all(|&v| v == -0.5));
        assert!(s.b.iter().all(|&v| v == 1.0));
        assert!(s.c.iter().all(|&v| v == 1.0));
        assert!(s.h.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn first_step_closed_form() {
        // h0=0, A=-0.5, B=1, C=1, dt=0.001, x=2.0
        // dA = 1 - 0.0005 = 0.9995
        // h_new = 0*0.9995 + 1*2*0.001 = 0.002
        // y = 32 * 0.002 * 1 = 0.064  (MAMBA_STATE_DIM=32, verified)
        let mut s = Mamba3State::create(None);
        let y = s.step(2.0);
        assert!((y - 0.064).abs() < 1e-6, "y={y}");
        assert!(s.h.iter().all(|&v| (v - 0.002).abs() < 1e-9));
        assert_eq!(s.step_count, 1);
    }

    #[test]
    fn decay_over_steps() {
        let mut s = Mamba3State::create(None);
        s.step(1.0);
        let y1 = s.step(0.0); // no input: h decays by dA
        let y2 = s.step(0.0);
        // h after step1 = 0.001; after step2 = 0.001*0.9995
        assert!(y1 > 0.0 && y1 < 0.128);
        assert!(y2 < y1, "decay expected: {y2} < {y1}");
    }
}
pub mod fractal_mamba3;
