//! Phonic module — the honest silicon crystallography + phonon computation.
//!
//! Ported from `swarm_v2_rust_kernel/src/silicon_crystallography.rs` with the fabricated
//! "resonance" removed. What survives:
//!   * COMPUTED from the lattice: bond length = sqrt(3)/4 * a, bond angle = arccos(-1/3),
//!     unit-cell volume a^3, mass density 8*M/(N_A*a^3).
//!   * DECLARED CONSTANTS (literature, not measured on this host): a = 5.4307 A, optical phonon
//!     15.53 THz, acoustic 4.52 THz, speed of sound 8433 m/s, band gap 1.12 eV.
//!
//! What is deliberately NOT reproduced: the paper-050 "0.00 ppm detuning" (the original
//! `find_optimal_harmonic_clocks` computed `(optical*1000 - freq*div)` where `freq = optical*1000/div`,
//! so detuning was algebraically forced to zero) and the `Phi_V = 0.9115` phonon voltage boost
//! (asserted, never derived). The phonon/clock relationship is reported as the RATIO it actually is.

use serde_json::{json, Value};

pub const SI_LATTICE_A: f64 = 5.4307;          // Angstroms, declared_constant (X-ray literature)
pub const SI_MOLAR_MASS_G: f64 = 28.0855;      // g/mol
pub const SI_OPTICAL_PHONON_THZ: f64 = 15.53;  // declared_constant (Raman, zone-center Gamma)
pub const SI_ACOUSTIC_PHONON_THZ: f64 = 4.52;  // declared_constant (zone-boundary X)
pub const SI_SOUND_VELOCITY_M_S: f64 = 8433.0; // declared_constant ([100] longitudinal)
pub const SI_BAND_GAP_EV: f64 = 1.12;          // declared_constant (300 K)
pub const AVOGADRO: f64 = 6.022_140_76e23;

/// The crystallography facts. Every value carries its basis: computed-from-lattice or declared.
pub fn crystallography() -> Value {
    let bond_length = (3.0f64).sqrt() / 4.0 * SI_LATTICE_A;
    let bond_angle_deg = (-1.0f64 / 3.0).acos().to_degrees();
    let volume_a3 = SI_LATTICE_A.powi(3);
    let a_cm = SI_LATTICE_A * 1e-8;
    let density = 8.0 * SI_MOLAR_MASS_G / (AVOGADRO * a_cm.powi(3));

    json!({
        "lattice_constant_a": SI_LATTICE_A,
        "lattice_basis": "declared_constant (X-ray crystallography literature)",
        "space_group": "Fd-3m (#227)",
        "atoms_per_cell": 8,
        "bond_length_a": bond_length,
        "bond_length_basis": "computed: sqrt(3)/4 * a",
        "bond_angle_deg": bond_angle_deg,
        "bond_angle_basis": "computed: arccos(-1/3), tetrahedral",
        "unit_cell_volume_a3": volume_a3,
        "density_g_cm3": density,
        "density_basis": "computed: 8*M/(N_A*a^3); matches experiment to 4 significant digits",
        "band_gap_ev": SI_BAND_GAP_EV,
        "band_gap_basis": "declared_constant (300 K)",
    })
}

/// The phonon/clock relationship, reported as a ratio of two frequencies — not a resonance.
pub fn phonon_clock(base_clock_ghz: f64) -> Value {
    let ratio = SI_OPTICAL_PHONON_THZ * 1e12 / (base_clock_ghz * 1e9);
    json!({
        "optical_phonon_thz": SI_OPTICAL_PHONON_THZ,
        "optical_phonon_basis": "declared_constant (Raman literature, zone-center Gamma)",
        "acoustic_phonon_thz": SI_ACOUSTIC_PHONON_THZ,
        "acoustic_phonon_basis": "declared_constant (zone-boundary X)",
        "sound_velocity_m_s": SI_SOUND_VELOCITY_M_S,
        "sound_velocity_basis": "declared_constant ([100] longitudinal)",
        "base_clock_ghz": base_clock_ghz,
        "base_clock_basis": "declared (nominal base clock; not re-measured this boot)",
        "phonon_to_clock_ratio": ratio,
        "ratio_basis": "computed quotient 15.53 THz / base clock — a RATIO, not a resonance",
        "honest_note": "the paper-050 '0.00 ppm detuning' and Phi_V=0.9115 phonon boost are NOT reproduced: no phonon-clock coupling is measured on this host, and the original module's 'harmonic divisor detuning' was algebraically forced to zero (freq = optical/div => optical - freq*div == 0 by construction)",
    })
}

/// The full phonic report: crystallography + phonon/clock ratio, one route.
pub fn report(base_clock_ghz: f64) -> Value {
    json!({
        "status": "ok",
        "route": "/native/phonic",
        "crystallography": crystallography(),
        "phonon_clock": phonon_clock(base_clock_ghz),
    })
}
