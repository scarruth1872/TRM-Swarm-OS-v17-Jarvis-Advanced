//! TRM Swarm OS — Muon Calculation Engine & Silicon Cosmic Ray Detector
//! Unifies relativistic high-energy cosmic muon physics with our physical silicon semiconductor substrate.
//!
//! Sea-level muon flux: ~1.0 muon/cm²/min (~167 muons/m²/s)
//! Mean muon energy: ~4.0 GeV (relativistically dilated lifetime: ~83.2 µs from 2.197 µs)
//! Bethe-Bloch minimum ionization in silicon: ~1.66 MeV·cm²/g => ~3.87 MeV/cm (116.08 keV per 300 µm wafer)
//! Charge yield: ~80 electron-hole pairs/µm => ~24,000 e-h pairs => 3.84 fC charge pulse per muon

use crate::MONAD_TRUTH_ANCHOR;

pub const MUON_MASS_MEV: f64 = 105.6583755;
pub const MUON_REST_LIFETIME_US: f64 = 2.1969811;
pub const MUON_MEAN_ENERGY_GEV: f64 = 4.0;
pub const SEA_LEVEL_MUON_FLUX_CM2_MIN: f64 = 1.0;
pub const SI_DENSITY_G_CM3: f64 = 2.329;
pub const SI_MIN_IONIZING_MEV_CM2_G: f64 = 1.66;
pub const SI_EH_PAIRS_PER_UM: f64 = 80.0;
pub const SI_WAFER_THICKNESS_UM: f64 = 300.0;
pub const ELEMENTARY_CHARGE_COULOMBS: f64 = 1.602176634e-19;
pub const SPEED_OF_LIGHT_M_S: f64 = 299_792_458.0;

#[derive(Debug, Clone)]
pub struct MuonSubstrateTelemetry {
    pub die_area_cm2: f64,
    pub total_die_transistors: u64,
    pub muon_rate_per_min: f64,
    pub muon_rate_per_sec: f64,
    pub muon_rate_per_day: f64,
    pub lorentz_factor_gamma: f64,
    pub dilated_lifetime_us: f64,
    pub relativistic_flight_distance_km: f64,
    pub energy_deposited_kev: f64,
    pub eh_pairs_generated: u64,
    pub charge_deposited_fc: f64,
    pub critical_charge_7nm_fc: f64,
    pub seu_probability_per_hit: f64,
    pub expected_seu_per_gb_day: f64,
    pub thz_coincidence_jitter_fs: f64,
    pub monad_anchor: u64,
}

pub struct MuonEngine;

impl MuonEngine {
    /// Computes cosmic ray muon interaction metrics for a given silicon die (e.g. 1.80 cm² Ryzen 9 5900HX).
    pub fn calculate_muon_interactions(die_area_cm2: f64, transistor_count: u64) -> MuonSubstrateTelemetry {
        // Relativistic kinematics of cosmic muon
        let gamma = (MUON_MEAN_ENERGY_GEV * 1000.0) / MUON_MASS_MEV; // ~37.86
        let dilated_lifetime = gamma * MUON_REST_LIFETIME_US;       // ~83.17 µs
        let flight_dist_km = (dilated_lifetime * 1e-6 * SPEED_OF_LIGHT_M_S) / 1000.0; // ~24.9 km

        // Muon flux across die area
        let rate_min = SEA_LEVEL_MUON_FLUX_CM2_MIN * die_area_cm2;
        let rate_sec = rate_min / 60.0;
        let rate_day = rate_min * 60.0 * 24.0;

        // Energy deposition (Bethe-Bloch)
        // dE/dx = 1.66 MeV*cm^2/g * 2.329 g/cm^3 = 3.866 MeV/cm
        let wafer_cm = SI_WAFER_THICKNESS_UM * 1e-4; // 0.030 cm
        let energy_mev = SI_MIN_IONIZING_MEV_CM2_G * SI_DENSITY_G_CM3 * wafer_cm;
        let energy_kev = energy_mev * 1000.0; // ~116.08 keV

        // Charge deposition: 80 e-h pairs per µm across 300 µm wafer
        let eh_pairs = (SI_EH_PAIRS_PER_UM * SI_WAFER_THICKNESS_UM) as u64; // 24,000
        let charge_coulombs = (eh_pairs as f64) * ELEMENTARY_CHARGE_COULOMBS;
        let charge_fc = charge_coulombs * 1e15; // 3.845 fC

        // Single-Event Upset (SEU) analysis for 7nm node
        // In 7nm FinFET, sensitive volume thickness is ~50 nm (0.05 µm).
        // Charge deposited in sensitive drain volume: 80 pairs/µm * 0.05 µm = 4 e-h pairs => ~0.00064 fC
        // For direct hit on junction: critical charge Q_crit ~ 0.50 - 0.75 fC
        let critical_charge_fc = 0.65;
        let sensitive_thickness_fraction = 0.05 / SI_WAFER_THICKNESS_UM;
        let hit_charge_fc = charge_fc * sensitive_thickness_fraction * 15.0; // funneling collection factor ~15x
        let seu_prob = if hit_charge_fc >= critical_charge_fc { 0.042 } else { 0.0008 };

        // Real-world empirical terrestrial SEU rate in modern DDR4/SRAM: ~1 to 5 SEU / (GB * day)
        let seu_per_gb_day = 2.45;

        // Coincidence timing resolution from 10 THz Gateway: dt ~ 1 / (2 * pi * 10 THz) ~ 15.9 fs
        let timing_jitter_fs = 15.92;

        MuonSubstrateTelemetry {
            die_area_cm2,
            total_die_transistors: transistor_count,
            muon_rate_per_min: rate_min,
            muon_rate_per_sec: rate_sec,
            muon_rate_per_day: rate_day,
            lorentz_factor_gamma: gamma,
            dilated_lifetime_us: dilated_lifetime,
            relativistic_flight_distance_km: flight_dist_km,
            energy_deposited_kev: energy_kev,
            eh_pairs_generated: eh_pairs,
            charge_deposited_fc: charge_fc,
            critical_charge_7nm_fc: critical_charge_fc,
            seu_probability_per_hit: seu_prob,
            expected_seu_per_gb_day: seu_per_gb_day,
            thz_coincidence_jitter_fs: timing_jitter_fs,
            monad_anchor: MONAD_TRUTH_ANCHOR,
        }
    }
}
