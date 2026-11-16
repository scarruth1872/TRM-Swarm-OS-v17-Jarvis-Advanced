//! TRM Swarm OS — THz Wave Gateway (no_std port).
//! 1:1 core semantics from swarm_v2/core/thz_wave_gateway.py — Catalyst.
//! THz waveform encode/step/process/decode with real constants:
//! F_THZ=1e12 Hz carrier, DELTA_F=1e9, PHI_V=0.76, V_RAIL=4.75, F_CPU=4GHz.

extern crate alloc;

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
