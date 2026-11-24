//! Multi-Spectral Photoacoustic K3 Wave Superposition Simulator (Phase 4 R&D).
//!
//! Maps Kleene-3 agent states onto WDM optical bands:
//!   +1 -> Blue  (450 nm) -> phase 0.0 rad (constructive carrier)
//!    0 -> Green (532 nm) -> phase pi/2 rad + harmonic spread (ambiguous scatter)
//!   -1 -> Red   (650 nm) -> phase pi rad   (destructive cancellation)
//!
//! Evaluates linear acoustic pressure superposition across a simulated SAW cavity
//! coupled to live HVBE voltage telemetry (`voltage::phi_v()`), reporting measured
//! peak amplitude, SNR (dB), resolved K3 state, and compute latency (ns).

use crate::voltage;
use serde_json::{json, Value};
use std::f64::consts::PI;
use std::time::Instant;

const DEFAULT_CAVITY_UM: f64 = 25.0;
const BASE_SOUND_SPEED_M_S: f64 = 5900.0; // Quartz / LiNbO3 SAW substrate
const BASE_ACOUSTIC_HZ: f64 = 250_000_000.0; // 250 MHz ultrasonic carrier
const DETECTOR_POINTS: usize = 32;

pub struct WaveStats {
    pub peak_amplitude: f64,
    #[allow(dead_code)]
    pub mean_power: f64,
    pub snr_db: f64,
    pub resolved_trit: i8,
    pub sound_speed_m_s: f64,
    pub elapsed_ns: u128,
}

/// Simulate photoacoustic wave superposition for a slice of K3 agent states (`-1, 0, +1`).
pub fn simulate_superposition(trits: &[i8], phi_v: f64) -> WaveStats {
    let t0 = Instant::now();
    let n = trits.len().max(1) as f64;

    // Couple acoustic velocity shift to measured Phi_V headroom (1.0 nominal -> 0% drift).
    let drift_factor = 1.0 + (1.0 - phi_v.clamp(0.1, 1.5)) * 0.004;
    let v_s = BASE_SOUND_SPEED_M_S * drift_factor;
    let k_base = (2.0 * PI * BASE_ACOUSTIC_HZ) / v_s;
    let cavity_m = DEFAULT_CAVITY_UM * 1e-6;

    let mut pos_count = 0usize;
    let mut neg_count = 0usize;
    let mut unk_count = 0usize;
    for &t in trits {
        match t {
            1 => pos_count += 1,
            -1 => neg_count += 1,
            _ => unk_count += 1,
        }
    }

    let mut peak = 0.0f64;
    let mut power_sum = 0.0f64;

    // Evaluate spatial standing wave P(x) across detector array.
    for m in 0..DETECTOR_POINTS {
        let x = (m as f64 / DETECTOR_POINTS as f64) * cavity_m;
        let mut p_x = 0.0f64;
        for (i, &trit) in trits.iter().enumerate() {
            let (phase, k_scale, amp) = match trit {
                1 => (0.0, 1.0, 1.0),                 // 450 nm Blue: coherent constructive
                -1 => (PI, 1.0, 1.0),                 // 650 nm Red: anti-phase cancellation
                _ => (PI * 0.5 + (i as f64 * 0.37), 1.0 + ((i % 7) as f64) * 0.08, 0.6), // 532 nm Green: diffuse scatter
            };
            p_x += amp * (k_base * k_scale * x + phase).cos();
        }
        let norm_p = p_x / n;
        let abs_p = norm_p.abs();
        if abs_p > peak {
            peak = abs_p;
        }
        power_sum += norm_p * norm_p;
    }

    let mean_power = power_sum / DETECTOR_POINTS as f64;
    let noise_floor = 1e-6_f64;
    let snr_db = 10.0 * (mean_power.max(noise_floor) / noise_floor).log10();

    // K3 resolution: any destructive red intercept drops net consensus to -1;
    // high coherent peak (>0.65) resolves +1; diffuse scatter resolves 0.
    let resolved_trit = if neg_count > 0 && peak < 0.85 {
        -1
    } else if unk_count == 0 && neg_count == 0 && peak >= 0.65 {
        1
    } else {
        0
    };

    let _ = (pos_count, neg_count, unk_count);

    WaveStats {
        peak_amplitude: peak,
        mean_power,
        snr_db,
        resolved_trit,
        sound_speed_m_s: v_s,
        elapsed_ns: t0.elapsed().as_nanos(),
    }
}

pub fn bench_json(agent_count: usize) -> Value {
    let n = agent_count.clamp(1, 2048);
    let phi_v = voltage::phi_v();

    let all_blue = vec![1i8; n];
    let blue_stats = simulate_superposition(&all_blue, phi_v);

    let mut with_red = vec![1i8; n];
    for i in 0..(n / 2).max(1) {
        with_red[i] = -1;
    }
    let red_stats = simulate_superposition(&with_red, phi_v);

    let diffuse = vec![0i8; n];
    let green_stats = simulate_superposition(&diffuse, phi_v);

    json!({
        "simulator": "Multi-Spectral Photoacoustic K3 Wave Superposition (CPU Software Model)",
        "agent_channels": n,
        "hvbe_phi_v": phi_v,
        "cavity_um": DEFAULT_CAVITY_UM,
        "carrier_mhz": BASE_ACOUSTIC_HZ / 1e6,
        "wavelength_bands_nm": {
            "true_plus_1": 450,
            "unknown_0": 532,
            "false_minus_1": 650
        },
        "cases": {
            "all_blue_constructive": {
                "peak_amplitude": blue_stats.peak_amplitude,
                "snr_db": blue_stats.snr_db,
                "resolved_trit": blue_stats.resolved_trit,
                "sound_speed_m_s": blue_stats.sound_speed_m_s,
                "elapsed_ns": blue_stats.elapsed_ns
            },
            "red_phase_cancellation": {
                "peak_amplitude": red_stats.peak_amplitude,
                "snr_db": red_stats.snr_db,
                "resolved_trit": red_stats.resolved_trit,
                "elapsed_ns": red_stats.elapsed_ns
            },
            "green_diffuse_scatter": {
                "peak_amplitude": green_stats.peak_amplitude,
                "snr_db": green_stats.snr_db,
                "resolved_trit": green_stats.resolved_trit,
                "elapsed_ns": green_stats.elapsed_ns
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_photoacoustic_k3_resolution() {
        let all_pos = vec![1i8; 584];
        let s_pos = simulate_superposition(&all_pos, 0.75);
        assert_eq!(s_pos.resolved_trit, 1);
        assert!(s_pos.peak_amplitude > 0.9);

        let mut balanced = vec![1i8; 584];
        for i in 0..292 {
            balanced[i] = -1;
        }
        let s_cancel = simulate_superposition(&balanced, 0.75);
        assert_eq!(s_cancel.resolved_trit, -1);
        assert!(s_cancel.peak_amplitude < 1e-6);

        let diffuse = vec![0i8; 584];
        let s_unk = simulate_superposition(&diffuse, 0.75);
        assert_eq!(s_unk.resolved_trit, 0);
        assert!(s_unk.peak_amplitude < 0.25);
    }
}
