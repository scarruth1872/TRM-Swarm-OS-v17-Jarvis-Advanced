//! Higuchi fractal dimension over a telemetry window.
//!
//! Ported from `swarm_v2/core/fractal_telemetry.py` so the numeric core runs in Rust. The ring buffer
//! stays with the caller: this is a pure function over the current window, which keeps the ABI small
//! and lets parity be checked sample-for-sample rather than through counters.
//!
//! L(m,k) = 1/k * sum|v[m+ik] - v[m+(i-1)k]| * (N-1)/(count*k);  <L(k)> ~ k^-D;  D = (ln L1 - ln L4)/ln 4.

use std::os::raw::{c_double, c_int};

const K_MAX: usize = 4;
/// The value the Python calculator reports before it has enough samples to compute anything.
const COLD_START_D_H: f64 = 1.05;

/// Higuchi fractal dimension of `values`, clamped to [1.0, 2.0].
///
/// Differences are taken in f32 (the buffer is f32, and numpy subtracts in f32) and accumulated in
/// f64, which is what the numpy reference does implicitly.
pub fn higuchi_d_h(values: &[f32]) -> f64 {
    let n = values.len();
    if n < 8 {
        return COLD_START_D_H;
    }

    let mut l_k = [0.0f64; K_MAX];
    for k in 1..=K_MAX {
        // numpy promotes `float + np.float32` to float32, so the reference accumulates in f32 until
        // the logarithm. Mirroring that is what makes the two agree.
        let mut total_length: f32 = 0.0;
        let mut valid_m = 0usize;

        for m in 0..k {
            let count = (n - 1 - m) / k;
            if count == 0 {
                continue;
            }
            let mut abs_sum: f32 = 0.0;
            for i in 1..=count {
                let i1 = m + i * k;
                let i2 = m + (i - 1) * k;
                abs_sum += (values[i1] - values[i2]).abs();
            }
            let norm = (n - 1) as f32 / (count * k) as f32;
            total_length += (abs_sum * norm) / k as f32;
            valid_m += 1;
        }

        if valid_m > 0 {
            l_k[k - 1] = (total_length / valid_m as f32) as f64;
        }
    }

    let l1 = l_k[0].max(1e-10);
    let l4 = l_k[3].max(1e-10);
    let slope = (l1.ln() - l4.ln()) / 4.0f64.ln();
    slope.clamp(1.0, 2.0)
}

/// Kleene quantization of D_H: +1 deterministic (<=1.25), -1 chaotic (>=1.70), 0 otherwise.
pub fn kleene_trit(d_h: f64) -> i32 {
    if (1.0..=1.25).contains(&d_h) {
        1
    } else if d_h >= 1.70 {
        -1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn rust_higuchi_d_h(ptr: *const f32, n: usize) -> c_double {
    if ptr.is_null() || n == 0 {
        return COLD_START_D_H;
    }
    let values = unsafe { std::slice::from_raw_parts(ptr, n) };
    higuchi_d_h(values) as c_double
}

#[no_mangle]
pub extern "C" fn rust_higuchi_kleene(d_h: c_double) -> c_int {
    kleene_trit(d_h as f64) as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_start_returns_constant() {
        assert_eq!(higuchi_d_h(&[1.0, 2.0, 3.0]), COLD_START_D_H);
    }

    #[test]
    fn a_straight_line_is_deterministic() {
        // A linear ramp has a well-defined, low dimension; it must land in the +1 band.
        let v: Vec<f32> = (0..64).map(|i| i as f32).collect();
        let d = higuchi_d_h(&v);
        assert!((1.0..=2.0).contains(&d), "d={d}");
        assert_eq!(kleene_trit(d), 1);
    }

    #[test]
    fn white_noise_is_higher_dimension_than_a_ramp() {
        let ramp: Vec<f32> = (0..64).map(|i| i as f32).collect();
        // deterministic pseudo-noise
        let mut s: u64 = 0x2545F4914F6CDD1D;
        let noise: Vec<f32> = (0..64)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                (s % 1000) as f32
            })
            .collect();
        assert!(higuchi_d_h(&noise) >= higuchi_d_h(&ramp));
    }

    #[test]
    fn result_is_always_clamped() {
        let mut s: u64 = 99;
        for _ in 0..50 {
            let v: Vec<f32> = (0..64)
                .map(|_| {
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    ((s >> 33) % 5000) as f32 - 2500.0
                })
                .collect();
            let d = higuchi_d_h(&v);
            assert!((1.0..=2.0).contains(&d));
        }
    }
}
