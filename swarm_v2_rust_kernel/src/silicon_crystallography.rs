//! TRM Swarm OS — Silicon Crystallography & Phonon Resonance Engine
//! Unveils the Diamond Cubic lattice (Fd-3m #227) as an active computational substrate.
//! Computes THz phonon modes (optical ~15.5 THz, acoustic ~4.5 THz) and clock harmonic coupling.

use crate::MONAD_TRUTH_ANCHOR;

pub const SI_LATTICE_CONSTANT_A: f64 = 5.4307; // Angstroms at 300K
pub const SI_BAND_GAP_EV: f64 = 1.12;           // Indirect bandgap at 300K
pub const SI_BOND_LENGTH_A: f64 = 2.3516;       // Nearest neighbor distance: a * sqrt(3) / 4
pub const SI_BOND_ANGLE_DEG: f64 = 109.4712;    // Tetrahedral arccos(-1/3)
pub const SI_DENSITY_G_CM3: f64 = 2.329;        // Mass density
pub const SI_OPTICAL_PHONON_THZ: f64 = 15.53;   // Zone-center optical phonon (Gamma point)
pub const SI_ACOUSTIC_PHONON_THZ: f64 = 4.52;   // Zone-boundary acoustic phonon (X point)
pub const SI_SOUND_VELOCITY_M_S: f64 = 8433.0;  // Longitudinal acoustic speed of sound [100]

#[derive(Debug, Clone, Copy)]
pub struct SiliconAtom {
    pub id: usize,
    pub fractional: (f64, f64, f64),
    pub cartesian: (f64, f64, f64),
}

#[derive(Debug, Clone)]
pub struct SiliconUnitCell {
    pub lattice_a: f64,
    pub space_group: &'static str,
    pub space_group_number: u32,
    pub volume_angstrom3: f64,
    pub atoms: Vec<SiliconAtom>,
}

#[derive(Debug, Clone)]
pub struct SiliconResonanceMetrics {
    pub base_clock_ghz: f64,
    pub boost_clock_ghz: f64,
    pub optical_phonon_thz: f64,
    pub acoustic_phonon_thz: f64,
    pub phonon_to_base_clock_ratio: f64,
    pub phonon_oscillations_per_cycle: f64,
    pub thermal_energy_mev: f64,
    pub sound_velocity_m_s: f64,
    pub de_broglie_thermal_wavelength_nm: f64,
    pub monad_anchor: u64,
}

pub struct SiliconCrystallography;

impl SiliconCrystallography {
    /// Generates the standard 8-atom conventional diamond cubic unit cell of Silicon.
    /// Space Group: Fd-3m (#227), origin choice 2 (inversion center at 0,0,0 or origin at -1/8,-1/8,-1/8).
    pub fn build_unit_cell() -> SiliconUnitCell {
        let a = SI_LATTICE_CONSTANT_A;
        let volume = a * a * a;

        // Face-Centered Cubic basis positions
        let fcc = [
            (0.0, 0.0, 0.0),
            (0.0, 0.5, 0.5),
            (0.5, 0.0, 0.5),
            (0.5, 0.5, 0.0),
        ];
        // Interpenetrating sub-lattice offset by (1/4, 1/4, 1/4)
        let offset = (0.25, 0.25, 0.25);

        let mut atoms = Vec::with_capacity(8);
        let mut id = 1;

        for &(fx, fy, fz) in &fcc {
            atoms.push(SiliconAtom {
                id,
                fractional: (fx, fy, fz),
                cartesian: (fx * a, fy * a, fz * a),
            });
            id += 1;
            let ox = fx + offset.0;
            let oy = fy + offset.1;
            let oz = fz + offset.2;
            atoms.push(SiliconAtom {
                id,
                fractional: (ox, oy, oz),
                cartesian: (ox * a, oy * a, oz * a),
            });
            id += 1;
        }

        SiliconUnitCell {
            lattice_a: a,
            space_group: "Fd-3m",
            space_group_number: 227,
            volume_angstrom3: volume,
            atoms,
        }
    }

    /// Evaluates energetic phonon resonance against CPU physical clock rates (3.3 GHz base / 4.6 GHz boost)
    pub fn compute_resonance(base_clock_ghz: f64, boost_clock_ghz: f64) -> SiliconResonanceMetrics {
        let clock_thz = base_clock_ghz / 1000.0;
        let ratio = SI_OPTICAL_PHONON_THZ / clock_thz;
        let oscillations_per_cycle = (SI_OPTICAL_PHONON_THZ * 1e12) / (base_clock_ghz * 1e9);

        // Thermal energy at T = 300K: kT = 25.85 meV
        let thermal_energy_mev = 25.852;
        // Thermal de Broglie wavelength for electron in Si: lambda_th = h / sqrt(2 * pi * m_eff * k_B * T) ~ 4.3 nm
        let de_broglie_nm = 4.305;

        SiliconResonanceMetrics {
            base_clock_ghz,
            boost_clock_ghz,
            optical_phonon_thz: SI_OPTICAL_PHONON_THZ,
            acoustic_phonon_thz: SI_ACOUSTIC_PHONON_THZ,
            phonon_to_base_clock_ratio: ratio,
            phonon_oscillations_per_cycle: oscillations_per_cycle,
            thermal_energy_mev,
            sound_velocity_m_s: SI_SOUND_VELOCITY_M_S,
            de_broglie_thermal_wavelength_nm: de_broglie_nm,
            monad_anchor: MONAD_TRUTH_ANCHOR,
        }
    }

    /// Explores quantum wave mechanics at sub-10nm FinFET / GAA-FET gate lengths.
    /// Computes quantum tunneling transmission probability T(E) through WKB approximation:
    /// T = exp( -2 * int_0^L sqrt(2 * m* * (V_0 - E)) / hbar dx )
    pub fn evaluate_quantum_gate_mechanics(gate_length_nm: f64, barrier_height_ev: f64, carrier_energy_ev: f64) -> QuantumGateProfile {
        let m_eff = 0.19 * 9.1093837e-31; // Silicon transverse electron effective mass (0.19 m0)
        let hbar = 1.054571817e-34;       // Reduced Planck constant (J*s)
        let q = 1.602176634e-19;          // Elementary charge (C)

        let delta_e_joules = ((barrier_height_ev - carrier_energy_ev).max(0.001)) * q;
        let length_meters = gate_length_nm * 1e-9;

        // Decay parameter kappa = sqrt(2 * m* * (V0 - E)) / hbar
        let kappa = (2.0 * m_eff * delta_e_joules).sqrt() / hbar;
        let exponent = -2.0 * kappa * length_meters;
        let transmission_prob = exponent.exp();

        // Unit cell span: how many lattice constants (a = 0.54307 nm) fit across this gate
        let unit_cell_span = gate_length_nm / (SI_LATTICE_CONSTANT_A * 0.1);
        let ballistic_fraction = (1.0 - transmission_prob).clamp(0.0, 1.0);

        QuantumGateProfile {
            gate_length_nm,
            unit_cell_span,
            barrier_height_ev,
            carrier_energy_ev,
            tunneling_probability: transmission_prob,
            ballistic_containment_ratio: ballistic_fraction,
            subthreshold_swing_mv_dec: 60.0 * (1.0 + transmission_prob * 1.5),
        }
    }

    /// Evaluates quantum containment for Gate-All-Around (GAA) Nanosheets in the 2nm regime.
    /// Incorporates 4-sided electrostatic gate wrap-around with High-κ HfO2 dielectric (κ = 25.0),
    /// effective barrier height enhancement, and subband quantum quantization energy shift.
    pub fn evaluate_2nm_gaa_nanosheet(nanosheet_thickness_nm: f64, gate_length_nm: f64) -> QuantumGateProfile {
        let m_eff = 0.19 * 9.1093837e-31;
        let hbar = 1.054571817e-34;
        let q = 1.602176634e-19;

        // Quantum confinement energy shift in 2D nanosheet: E_conf = (hbar^2 * pi^2) / (2 * m* * t^2)
        let t_meters = nanosheet_thickness_nm * 1e-9;
        let e_conf_joules = (hbar * hbar * std::f64::consts::PI * std::f64::consts::PI) / (2.0 * m_eff * t_meters * t_meters);
        let e_conf_ev = e_conf_joules / q;

        // GAA 4-sided gate wrap-around electrostatic boost adds +0.85 eV to effective barrier
        let effective_barrier_ev = 3.10 + 0.85 + e_conf_ev;
        let carrier_energy_ev = 0.20;

        let delta_e_joules = (effective_barrier_ev - carrier_energy_ev) * q;
        let length_meters = gate_length_nm * 1e-9;
        let kappa = (2.0 * m_eff * delta_e_joules).sqrt() / hbar;
        let exponent = -2.0 * kappa * length_meters;
        let gaa_tunneling_prob = exponent.exp();

        let unit_cell_span = gate_length_nm / (SI_LATTICE_CONSTANT_A * 0.1);
        let ballistic_fraction = (1.0 - gaa_tunneling_prob).clamp(0.0, 1.0);

        QuantumGateProfile {
            gate_length_nm,
            unit_cell_span,
            barrier_height_ev: effective_barrier_ev,
            carrier_energy_ev,
            tunneling_probability: gaa_tunneling_prob,
            ballistic_containment_ratio: ballistic_fraction,
            subthreshold_swing_mv_dec: 60.0 * (1.0 + gaa_tunneling_prob * 1.5),
        }
    }

    /// Optimizes clock frequency to lock onto an exact integer harmonic divisor of the 15.53 THz optical phonon mode.
    pub fn find_optimal_harmonic_clocks() -> Vec<HarmonicClockTarget> {
        let optical_thz = SI_OPTICAL_PHONON_THZ;
        let candidate_divisors = [3376, 3882, 4706, 4853, 5176];

        candidate_divisors.iter().map(|&div| {
            let freq_ghz = (optical_thz * 1000.0) / (div as f64);
            let detuning_ppm = (((optical_thz * 1000.0) - (freq_ghz * div as f64)).abs() / (optical_thz * 1000.0)) * 1e6;
            HarmonicClockTarget {
                harmonic_divisor: div,
                resonant_clock_ghz: freq_ghz,
                detuning_ppm,
                phonon_oscillations_exact: div,
            }
        }).collect()
    }

    /// Harmonizes voltage potential Φ_V using phonon resonance coupling
    pub fn compute_phonon_harmonized_voltage(raw_phi_v: f64, clock_ghz: f64) -> f64 {
        // Multiplier gamma derived from phonon density of states coupling
        let clock_ratio = clock_ghz / 3.30;
        let resonance_coupling = 1.0 + 0.05 * (1.0 - (clock_ratio - 1.0).abs());
        (raw_phi_v * resonance_coupling).min(0.9999)
    }

    /// Exports unit cell as standardized STAR mmCIF format string
    pub fn to_cif(cell: &SiliconUnitCell) -> String {
        let mut out = String::new();
        out.push_str("data_Silicon_Diamond_Cubic\n");
        out.push_str("# TRM Swarm OS — Silicon Computational Substrate Crystallography\n");
        out.push_str(&format!("# Monad Truth Anchor: 0x{:016X}\n\n", MONAD_TRUTH_ANCHOR));
        out.push_str("_audit_creation_method 'TRM Swarm OS v19 Silicon Crystallography Engine'\n");
        out.push_str("_chemical_name_common 'Silicon'\n");
        out.push_str("_chemical_formula_sum 'Si8'\n");
        out.push_str(&format!("_cell.length_a {:.4}\n", cell.lattice_a));
        out.push_str(&format!("_cell.length_b {:.4}\n", cell.lattice_a));
        out.push_str(&format!("_cell.length_c {:.4}\n", cell.lattice_a));
        out.push_str("_cell.angle_alpha 90.000\n");
        out.push_str("_cell.angle_beta 90.000\n");
        out.push_str("_cell.angle_gamma 90.000\n");
        out.push_str(&format!("_cell.volume {:.2}\n", cell.volume_angstrom3));
        out.push_str(&format!("_symmetry.space_group_name_H-M '{}'\n", cell.space_group));
        out.push_str(&format!("_symmetry.Int_Tables_number {}\n\n", cell.space_group_number));

        out.push_str("loop_\n");
        out.push_str("  _atom_site.id\n");
        out.push_str("  _atom_site.type_symbol\n");
        out.push_str("  _atom_site.label_atom_id\n");
        out.push_str("  _atom_site.label_comp_id\n");
        out.push_str("  _atom_site.label_asym_id\n");
        out.push_str("  _atom_site.Cartn_x\n");
        out.push_str("  _atom_site.Cartn_y\n");
        out.push_str("  _atom_site.Cartn_z\n");
        out.push_str("  _atom_site.occupancy\n");
        out.push_str("  _atom_site.b_iso_or_equiv\n");

        for atom in &cell.atoms {
            out.push_str(&format!(
                "  {:<2} Si Si{:<2} SIL A {:>8.4} {:>8.4} {:>8.4} 1.00 0.45\n",
                atom.id, atom.id, atom.cartesian.0, atom.cartesian.1, atom.cartesian.2
            ));
        }

        out
    }
}

#[derive(Debug, Clone)]
pub struct QuantumGateProfile {
    pub gate_length_nm: f64,
    pub unit_cell_span: f64,
    pub barrier_height_ev: f64,
    pub carrier_energy_ev: f64,
    pub tunneling_probability: f64,
    pub ballistic_containment_ratio: f64,
    pub subthreshold_swing_mv_dec: f64,
}

#[derive(Debug, Clone)]
pub struct HarmonicClockTarget {
    pub harmonic_divisor: usize,
    pub resonant_clock_ghz: f64,
    pub detuning_ppm: f64,
    pub phonon_oscillations_exact: usize,
}

