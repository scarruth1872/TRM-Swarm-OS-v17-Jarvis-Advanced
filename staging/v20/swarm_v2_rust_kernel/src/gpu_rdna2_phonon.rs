//! TRM Swarm OS — GPU RDNA2 Substrate Awareness & Real-Time Phonon Mapping Engine
//!
//! Extends substrate awareness beyond the x86-64 CPU into the discrete GPU:
//! - AMD Navi 22 (RDNA2) Die: 335 mm² (3.35 cm² TSMC 7nm)
//! - 17.2 Billion Transistors, 40 Compute Units, 2560 Stream Processors
//! - Game Clock: 2424 MHz, Boost Clock: 2581 MHz
//! - 96 MB Infinity Cache (High-density on-die SRAM)
//!
//! Real-Time Phonon Spectrum Mapping:
//! - Computes dynamic Doppler and thermal softening shifts in acoustic (4.52 THz) and optical (15.53 THz) phonons.
//! - Tracks phonon occupation numbers n_q(T) = 1 / (exp(hbar*omega / kT) - 1).

use crate::MONAD_TRUTH_ANCHOR;
use crate::silicon_crystallography::{SI_OPTICAL_PHONON_THZ, SI_ACOUSTIC_PHONON_THZ};

pub const RDNA2_NAVI22_DIE_AREA_CM2: f64 = 3.35; // 335 mm²
pub const RDNA2_NAVI22_TRANSISTORS: u64 = 17_200_000_000;
pub const RDNA2_COMPUTE_UNITS: usize = 40;
pub const RDNA2_STREAM_PROCESSORS: usize = 2560;
pub const RDNA2_BOOST_CLOCK_MHZ: f64 = 2581.0;
pub const RDNA2_INFINITY_CACHE_MB: usize = 96;

#[derive(Debug, Clone)]
pub struct RDNA2SubstrateTelemetry {
    pub die_name: &'static str,
    pub architecture: &'static str,
    pub die_area_cm2: f64,
    pub transistor_count: u64,
    pub compute_units: usize,
    pub stream_processors: usize,
    pub boost_clock_ghz: f64,
    pub infinity_cache_mb: usize,
    pub muon_rate_per_min: f64,
    pub muon_rate_per_day: f64,
    pub optical_harmonic_divisor: usize,
    pub optical_harmonic_detuning_ppm: f64,
}

#[derive(Debug, Clone)]
pub struct PhononModeState {
    pub branch: &'static str,
    pub base_frequency_thz: f64,
    pub active_frequency_thz: f64,
    pub thermal_shift_ghz: f64,
    pub bose_einstein_occupation: f64,
    pub mean_free_path_nm: f64,
    pub linewidth_ghz: f64,
}

#[derive(Debug, Clone)]
pub struct RealTimePhononSpectrum {
    pub substrate_temp_k: f64,
    pub cpu_workload_pct: f64,
    pub gpu_workload_pct: f64,
    pub modes: Vec<PhononModeState>,
    pub total_thermal_flux_w_cm2: f64,
    pub aggregate_phonon_density: f64,
    pub monad_anchor: u64,
}

pub struct SubstrateAwarenessEngine;

impl SubstrateAwarenessEngine {
    /// Telemetry and substrate physics for the discrete RDNA2 GPU (AMD RX 6700 XT / Navi 22)
    pub fn get_rdna2_telemetry() -> RDNA2SubstrateTelemetry {
        let die_area = RDNA2_NAVI22_DIE_AREA_CM2;
        let muon_min = 1.0 * die_area;
        let muon_day = muon_min * 60.0 * 24.0;

        let boost_ghz = RDNA2_BOOST_CLOCK_MHZ / 1000.0;
        let optical_ratio = (SI_OPTICAL_PHONON_THZ * 1000.0) / (boost_ghz * 1000.0);
        let divisor = optical_ratio.round() as usize; // ~6017
        let exact_freq = (SI_OPTICAL_PHONON_THZ * 1000.0) / (divisor as f64);
        let detuning = ((boost_ghz * 1000.0 - exact_freq).abs() / (boost_ghz * 1000.0)) * 1e6;

        RDNA2SubstrateTelemetry {
            die_name: "AMD Navi 22 XT (Radeon RX 6700 XT)",
            architecture: "RDNA2 7nm TSMC",
            die_area_cm2: die_area,
            transistor_count: RDNA2_NAVI22_TRANSISTORS,
            compute_units: RDNA2_COMPUTE_UNITS,
            stream_processors: RDNA2_STREAM_PROCESSORS,
            boost_clock_ghz: boost_ghz,
            infinity_cache_mb: RDNA2_INFINITY_CACHE_MB,
            muon_rate_per_min: muon_min,
            muon_rate_per_day: muon_day,
            optical_harmonic_divisor: divisor,
            optical_harmonic_detuning_ppm: detuning,
        }
    }

    /// Maps the phonon spectrum in real-time during computation at temperature T (K) and compute load.
    pub fn map_realtime_phonon_spectrum(temp_k: f64, cpu_load: f64, gpu_load: f64) -> RealTimePhononSpectrum {
        let kb_ev = 8.617333262e-5; // eV / K
        let hbar_ev_s = 6.582119569e-16; // eV * s

        // Thermal softening coefficient for bulk silicon: d omega / dT ~ -0.022 cm^-1 / K ~ -0.66 GHz / K
        let delta_t = temp_k - 300.0;
        let softening_ghz = -0.66 * delta_t;

        // Phonon modes under compute stress
        let base_modes = [
            ("Longitudinal Acoustic (LA - X Point)", SI_ACOUSTIC_PHONON_THZ, 8433.0, 42.0),
            ("Transverse Acoustic (TA - L Point)", 2.40, 5840.0, 68.0),
            ("Transverse Optical (TO - Gamma Point)", SI_OPTICAL_PHONON_THZ, 1200.0, 11.5),
            ("Longitudinal Optical (LO - Gamma Point)", 15.10, 1400.0, 12.8),
        ];

        let mut modes = Vec::with_capacity(4);
        for &(name, base_thz, sound_speed, mfp_base) in &base_modes {
            let active_freq = base_thz + (softening_ghz / 1000.0);
            let omega = 2.0 * std::f64::consts::PI * (active_freq * 1e12);
            let energy_ev = hbar_ev_s * omega;
            let exponent = (energy_ev / (kb_ev * temp_k)).clamp(0.01, 50.0);
            let occupation = 1.0 / (exponent.exp() - 1.0);

            // Anharmonic scattering linewidth: Gamma ~ Gamma_0 * (1 + 2 * n)
            let linewidth = 18.5 * (1.0 + 2.0 * occupation) * (1.0 + (cpu_load + gpu_load) * 0.1);
            let mfp = mfp_base * (300.0 / temp_k) * (1.0 - (cpu_load + gpu_load) * 0.05);

            modes.push(PhononModeState {
                branch: name,
                base_frequency_thz: base_thz,
                active_frequency_thz: active_freq,
                thermal_shift_ghz: softening_ghz,
                bose_einstein_occupation: occupation,
                mean_free_path_nm: mfp,
                linewidth_ghz: linewidth,
            });
        }

        let thermal_flux = 45.0 * (1.0 + (cpu_load + gpu_load) * 0.85); // W/cm²
        let density = 1.84e22 * (temp_k / 300.0);

        RealTimePhononSpectrum {
            substrate_temp_k: temp_k,
            cpu_workload_pct: cpu_load * 100.0,
            gpu_workload_pct: gpu_load * 100.0,
            modes,
            total_thermal_flux_w_cm2: thermal_flux,
            aggregate_phonon_density: density,
            monad_anchor: MONAD_TRUTH_ANCHOR,
        }
    }
}
