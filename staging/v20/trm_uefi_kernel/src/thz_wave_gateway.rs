//! TRM Swarm OS — THz Wave Gateway (no_std port).
//! 1:1 core semantics from swarm_v2/core/thz_wave_gateway.py — Catalyst.
//! THz waveform encode/step/process/decode with real constants:
//! F_THZ=1e12 Hz carrier, DELTA_F=1e9, PHI_V=0.76, V_RAIL=4.75, F_CPU=4GHz.

extern crate alloc;
use alloc::vec::Vec;

/// THz constants (1:1 python module constants).
pub const F_THZ: f64 = 1_000_000_000_000.0; // 1 THz carrier
pub const DELTA_F: f64 = 1_000_000_000.0;   // 1 GHz channel spacing
pub const PHI_V: f64 = 0.76;
pub const V_RAIL: f64 = 4.75;
pub const F_CPU: f64 = 4_000_000_000.0;

/// Encode a value onto the THz carrier (phase modulation).
pub fn encode(value: f64) -> f64 {
    // phase = value mod 2pi mapped to 0..V_RAIL
    let two_pi = 6.283185307179586;
    let phase = value - floor_f64(value / two_pi) * two_pi;
    phase / two_pi * V_RAIL
}

/// Step the waveform: next sample given carrier phase.
pub fn step(t: f64) -> f64 {
    // sin(2*pi*F_THZ*t) scaled by Phi_V rail (no_std polynomial approx)
    let w = 2.0 * 3.141592653589793 * F_THZ * t;
    (sin_approx(w) * PHI_V * V_RAIL) + PHI_V * V_RAIL
}

/// no_std floor (truncation + sign correction).
fn floor_f64(x: f64) -> f64 {
    let t = x as i64 as f64;
    if x < t {
        t - 1.0
    } else {
        t
    }
}

/// no_std sin: Taylor series (adequate for the phase domain).
fn sin_approx(x: f64) -> f64 {
    // reduce to [-pi, pi]
    let two_pi = 6.283185307179586;
    let mut r = x - floor_f64(x / two_pi) * two_pi;
    if r > 3.141592653589793 {
        r -= two_pi;
    }
    let x2 = r * r;
    // sin(x) ~ x - x^3/6 + x^5/120 - x^7/5040
    r * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0)))
}

/// Process: throughput = F_CPU / channels.
pub fn process(channels: u64) -> f64 {
    if channels == 0 {
        0.0
    } else {
        F_CPU / channels as f64
    }
}

/// Decode a rail value back to the phase domain.
pub fn decode(rail: f64) -> f64 {
    let v = rail.clamp(0.0, V_RAIL);
    v / V_RAIL * 6.283185307179586
}

/// Singleton handle.
pub fn get_thz_gateway() -> u64 {
    0x5448_5A47_4154_4557
}

// ── The strip's model (native_voltage_strips/strip_thz_wave_gateway.rs) ──────────────────────────
//
// `strip_thz_fourier_encode` turns a byte payload into trits through an optical-phase analogy:
//     per byte b:  val = b as f32;  phase = 2*PI*val/256
//                  re = val*cos(phase); im = val*sin(phase);  intensity = re*re + im*im
//     max_int tracks the largest intensity (starting at 1e-9, strict >)
//     norm_i  = intensity / max_int
//     trit_i  = +1 if norm > 0.5, -1 if norm < 0.1, else 0
//     mean    = sum(norm) / count
// all in f32, with `+1` as its K3 status.
//
// Bare metal has no `sin`/`cos`, so both are implemented here in **f64** and rounded once to f32 --
// the same approach that made `sqrt_f32` agree with its strip bit for bit. The range reduction folds
// the phase into [-PI/2, PI/2] and the Taylor pairs are Horner-nested (coefficients 1/6, 1/20, 1/42,
// 1/72, 1/110 for sine and 1/2, 1/12, 1/30, 1/56, 1/90 for cosine), which is far past f32 precision
// at these magnitudes.

const PI_F64: f64 = 3.141592653589793;
const TAU_F64: f64 = 6.283185307179586;

fn sin_taylor(y: f64) -> f64 {
    let y2 = y * y;
    y * (1.0 - y2 / 6.0 * (1.0 - y2 / 20.0 * (1.0 - y2 / 42.0 * (1.0 - y2 / 72.0 * (1.0 - y2 / 110.0)))))
}

fn cos_taylor(y: f64) -> f64 {
    let y2 = y * y;
    1.0 - y2 / 2.0 * (1.0 - y2 / 12.0 * (1.0 - y2 / 30.0 * (1.0 - y2 / 56.0 * (1.0 - y2 / 90.0))))
}

/// (sin, cos) over the reduced range, computed in f64.
pub fn sin_cos_f64(x: f64) -> (f64, f64) {
    let mut r = x % TAU_F64;
    if r > PI_F64 {
        r -= TAU_F64;
    } else if r < -PI_F64 {
        r += TAU_F64;
    }
    let (s_sign, c_sign, y) = if r > PI_F64 / 2.0 {
        (1.0, -1.0, PI_F64 - r)
    } else if r < -PI_F64 / 2.0 {
        (-1.0, -1.0, -PI_F64 - r)
    } else {
        (1.0, 1.0, r)
    };
    (s_sign * sin_taylor(y), c_sign * cos_taylor(y))
}

/// The trit thresholds the strip uses.
pub const THZ_TRIT_HIGH: f32 = 0.5;
pub const THZ_TRIT_LOW: f32 = 0.1;

/// The strip's encode: (trits, mean intensity). Empty input yields no trits and a mean of 0.
pub fn native_thz_fourier_encode(bytes: &[u8]) -> (Vec<i32>, f32) {
    if bytes.is_empty() {
        return (Vec::new(), 0.0);
    }
    let mut intensities: Vec<f32> = Vec::with_capacity(bytes.len());
    let mut max_int: f32 = 1e-9;
    for &b in bytes {
        let val = b as f32;
        let phase = 2.0f32 * (PI_F64 as f32) * val / 256.0f32;
        let (s, c) = sin_cos_f64(phase as f64);
        let re = val * (c as f32);
        let im = val * (s as f32);
        let intensity = re * re + im * im;
        if intensity > max_int {
            max_int = intensity;
        }
        intensities.push(intensity);
    }
    let mut trits: Vec<i32> = Vec::with_capacity(bytes.len());
    let mut sum_norm = 0.0f32;
    for i in 0..bytes.len() {
        let norm_i = intensities[i] / max_int;
        sum_norm += norm_i;
        trits.push(if norm_i > THZ_TRIT_HIGH {
            1
        } else if norm_i < THZ_TRIT_LOW {
            -1
        } else {
            0
        });
    }
    (trits, sum_norm / (bytes.len() as f32))
}
