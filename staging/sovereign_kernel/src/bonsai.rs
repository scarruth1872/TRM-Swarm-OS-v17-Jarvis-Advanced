//! Bonsai ternary state-space kernel.
//!
//! Ported from `swarm_v2/core/thz_bonsai_gateway.py`. The Python `BonsaiTernaryCore` integrates a
//! linear SSM (dh = A·h + B·x, Euler at 0.05) and quantizes y = C·h + D·x into K3 trits. That is the
//! only numerically substantive module among the 30 synthesized strips: 29 numpy call sites and four
//! file operations, against near-zero arithmetic everywhere else.
//!
//! The state lives here and is addressed by an opaque handle, so the 25,600 weights are handed over
//! once instead of on every call.

use std::os::raw::{c_double, c_int};

pub struct BonsaiState {
    dim: usize,
    state_dim: usize,
    a: Vec<f64>,
    b: Vec<f64>,
    c: Vec<f64>,
    d: Vec<f64>,
    h: Vec<f64>,
    threshold: f64,
}

#[inline]
fn matvec(m: &[f64], v: &[f64], rows: usize, cols: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; rows];
    for i in 0..rows {
        let row = &m[i * cols..(i + 1) * cols];
        let mut acc = 0.0f64;
        for j in 0..cols {
            acc += row[j] * v[j];
        }
        out[i] = acc;
    }
    out
}

impl BonsaiState {
    pub fn new(
        dim: usize,
        state_dim: usize,
        a: Vec<f64>,
        b: Vec<f64>,
        c: Vec<f64>,
        d: Vec<f64>,
        threshold: f64,
    ) -> Self {
        assert_eq!(a.len(), state_dim * state_dim);
        assert_eq!(b.len(), state_dim * dim);
        assert_eq!(c.len(), dim * state_dim);
        assert_eq!(d.len(), dim * dim);
        Self { dim, state_dim, a, b, c, d, h: vec![0.0; state_dim], threshold }
    }

    /// One step: integrate the state, produce y, quantize to trits. Returns (trits, energy, conf).
    pub fn process(&mut self, x_in: &[f64]) -> (Vec<i8>, f64, f64) {
        let mut x = vec![0.0f64; self.dim];
        let take = x_in.len().min(self.dim);
        x[..take].copy_from_slice(&x_in[..take]);

        let ah = matvec(&self.a, &self.h, self.state_dim, self.state_dim);
        let bx = matvec(&self.b, &x, self.state_dim, self.dim);
        for i in 0..self.state_dim {
            self.h[i] += 0.05 * (ah[i] + bx[i]);
        }

        let ch = matvec(&self.c, &self.h, self.dim, self.state_dim);
        let dx = matvec(&self.d, &x, self.dim, self.dim);
        let y: Vec<f64> = (0..self.dim).map(|i| ch[i] + dx[i]).collect();

        let trits: Vec<i8> = y
            .iter()
            .map(|&v| {
                if v > self.threshold {
                    1
                } else if v < -self.threshold {
                    -1
                } else {
                    0
                }
            })
            .collect();

        let energy: f64 = self.h.iter().map(|v| v * v).sum();
        let nonzero: f64 = trits.iter().map(|t| (*t as f64).abs()).sum();
        let conf = nonzero / self.dim as f64;
        (trits, energy, conf)
    }

    pub fn h_norm(&self) -> f64 {
        self.h.iter().map(|v| v * v).sum::<f64>().sqrt()
    }

    pub fn dim(&self) -> usize {
        self.dim
    }
}

#[no_mangle]
pub extern "C" fn rust_bonsai_new(
    dim: usize,
    state_dim: usize,
    a: *const f64,
    b: *const f64,
    c: *const f64,
    d: *const f64,
    threshold: f64,
) -> *mut BonsaiState {
    if a.is_null() || b.is_null() || c.is_null() || d.is_null() || dim == 0 || state_dim == 0 {
        return std::ptr::null_mut();
    }
    unsafe {
        let copy = |p: *const f64, n: usize| std::slice::from_raw_parts(p, n).to_vec();
        let st = BonsaiState::new(
            dim,
            state_dim,
            copy(a, state_dim * state_dim),
            copy(b, state_dim * dim),
            copy(c, dim * state_dim),
            copy(d, dim * dim),
            threshold,
        );
        Box::into_raw(Box::new(st))
    }
}

/// Writes `dim` trits, the energy and the route confidence. Returns 0 on success.
#[no_mangle]
pub extern "C" fn rust_bonsai_process(
    handle: *mut BonsaiState,
    x: *const f64,
    n: usize,
    trits_out: *mut i8,
    energy_out: *mut f64,
    conf_out: *mut f64,
) -> c_int {
    if handle.is_null() || x.is_null() || trits_out.is_null() {
        return -1;
    }
    unsafe {
        let st = &mut *handle;
        let input = std::slice::from_raw_parts(x, n);
        let (trits, energy, conf) = st.process(input);
        std::ptr::copy_nonoverlapping(trits.as_ptr(), trits_out, trits.len());
        if !energy_out.is_null() {
            *energy_out = energy as c_double;
        }
        if !conf_out.is_null() {
            *conf_out = conf as c_double;
        }
        0
    }
}

#[no_mangle]
pub extern "C" fn rust_bonsai_h_norm(handle: *mut BonsaiState) -> c_double {
    if handle.is_null() {
        return 0.0;
    }
    unsafe { (*handle).h_norm() as c_double }
}

#[no_mangle]
pub extern "C" fn rust_bonsai_free(handle: *mut BonsaiState) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> BonsaiState {
        // dim=2, state_dim=2, A=I, B=0, C=I, D=0 -> h integrates nothing, y mirrors h
        BonsaiState::new(
            2, 2,
            vec![1.0, 0.0, 0.0, 1.0],
            vec![0.0; 4],
            vec![1.0, 0.0, 0.0, 1.0],
            vec![0.0; 4],
            0.02,
        )
    }

    #[test]
    fn zero_input_stays_zero() {
        let mut s = tiny();
        let (trits, energy, conf) = s.process(&[0.0, 0.0]);
        assert_eq!(trits, vec![0, 0]);
        assert_eq!(energy, 0.0);
        assert_eq!(conf, 0.0);
    }

    #[test]
    fn state_stays_bounded_with_identity_dynamics() {
        let mut s = tiny();
        for _ in 0..50 {
            s.process(&[1.0, -1.0]);
            assert!(s.h_norm() < 1e6);
        }
    }

    #[test]
    fn short_input_is_zero_padded() {
        let mut s = tiny();
        let (trits, _, _) = s.process(&[5.0]);
        assert_eq!(trits.len(), 2);
    }
}
