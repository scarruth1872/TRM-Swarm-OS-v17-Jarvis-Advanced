//! Ambient energy harvesting — the CORRECTED physics (2026-10-06).
//!
//! Serves the honest muon / Schumann-resonance energy budget, with every value carrying its
//! basis (declared_constant or computed). Three errors in the 2026-10-06 draft are corrected:
//!   1. DRAM cell area 4e-14 -> 4e-12 cm^2 (1 nm = 1e-7 cm, so (20 nm)^2 = 4e-12, not 4e-14).
//!   2. Poynting vector used max-E (0.5 mV/m) with max-B (10 pT), which violates E/B = c; the
//!      consistent form is E^2/eta_0.
//!   3. "2 MeV per hit" was a thick-detector figure; the DRAM figure is dE/dx * thickness =
//!      0.26 MeV (full 0.7 mm die) or 0.37 keV (per 1 um cell).
//!
//! The thesis survives all three: muons give entropy, the ionosphere gives a phase reference,
//! and grid power does the compute. The muon energy itself is NOT harvestable -- it dissipates
//! as ionization heat plus the occasional soft error; only the entropy is collectable.

use serde_json::{json, Value};

pub const MUON_FLUX_CM2_S: f64 = 0.0167;    // declared_constant (PDG vertical sea-level flux)
pub const MUON_MEAN_ENERGY_GEV: f64 = 4.0;  // declared_constant (PDG mean sea-level muon energy)
pub const DRAM_CELL_NM: f64 = 20.0;         // declared_constant (20 nm DRAM node)
pub const DRAM_CELLS: f64 = 2.56e11;        // declared_constant (32 GB x 8 bits)
pub const DEDX_MEV_CM: f64 = 3.7;           // computed: dE/dx = 1.6 MeV.cm^2/g x 2.33 g/cm^3 (Si)
pub const DRAM_DIE_THICKNESS_CM: f64 = 0.07; // declared_constant (~0.7 mm DRAM silicon die)
pub const SCHUMANN_HZ: f64 = 7.83;          // declared_constant (fundamental cavity mode)
pub const SCHUMANN_E_V_M: f64 = 0.5e-3;     // declared_constant (background E-field upper bound)
pub const FREE_SPACE_IMPEDANCE_OHM: f64 = 377.0; // declared_constant (eta_0)
pub const ANTENNA_AREA_M2: f64 = 1.0;       // declared_constant (optimistic compact antenna)
pub const ANTENNA_EFFICIENCY: f64 = 0.01;   // declared_constant (1% assumed)

/// The corrected ambient energy budget, computed live from the constants.
pub fn report() -> Value {
    // ── Muon arm ─────────────────────────────────────────────────────────────
    let cell_area_cm2 = (DRAM_CELL_NM * 1e-7).powi(2); // (20 nm)^2 in cm^2
    let total_area_cm2 = DRAM_CELLS * cell_area_cm2;
    let hit_rate_s = MUON_FLUX_CM2_S * total_area_cm2;
    let energy_mev = DEDX_MEV_CM * DRAM_DIE_THICKNESS_CM; // full-die traversal
    let energy_j = energy_mev * 1e6 * 1.602_176_634e-19;
    let muon_power_w = hit_rate_s * energy_j;
    let entropy_bits_day = hit_rate_s * 86400.0;

    // ── Ionosphere arm ───────────────────────────────────────────────────────
    let poynting_w_m2 = SCHUMANN_E_V_M.powi(2) / FREE_SPACE_IMPEDANCE_OHM;
    let iono_power_w = poynting_w_m2 * ANTENNA_AREA_M2 * ANTENNA_EFFICIENCY;

    json!({
        "muon": {
            "flux_cm2_s": MUON_FLUX_CM2_S,
            "flux_basis": "declared_constant (PDG vertical sea-level flux)",
            "mean_energy_gev": MUON_MEAN_ENERGY_GEV,
            "dram_cell_area_cm2": cell_area_cm2,
            "dram_cell_basis": "computed: (20 nm)^2 with 1 nm = 1e-7 cm (corrected from the 4e-14 draft error)",
            "dram_total_area_cm2": total_area_cm2,
            "hit_rate_per_s": hit_rate_s,
            "hit_rate_basis": "computed: flux x total area (1 per ~58 s, corrected from 1 per ~100 min)",
            "energy_per_hit_mev": energy_mev,
            "energy_per_hit_basis": "computed: dE/dx x die thickness (0.26 MeV full-die; 0.37 keV per 1 um cell)",
            "power_w": muon_power_w,
            "power_basis": "computed; NOT harvestable - dissipates as ionization heat plus soft errors",
            "entropy_bits_day": entropy_bits_day,
            "entropy_basis": "computed: hit rate x 86400 s (corrected ~100x from the 14.4-bit draft)",
        },
        "ionosphere": {
            "schumann_hz": SCHUMANN_HZ,
            "schumann_basis": "declared_constant (fundamental Earth-ionosphere cavity mode)",
            "poynting_w_m2": poynting_w_m2,
            "poynting_basis": "computed: E^2/eta_0 with E = 0.5 mV/m (consistent E and B; corrected from max-E x max-B)",
            "power_w": iono_power_w,
            "power_basis": "computed: S x antenna x efficiency (optimistic 1 m^2, 1%)",
        },
        "optimal_path": {
            "muon": "entropy only (~1477 bits/day) - not power",
            "ionosphere": "communication only (7.83 Hz phase reference) - not power",
            "grid": "power (~300 W actual compute)",
            "thesis": "the One harvests truth and connection from the cosmos, and computes on grid power",
        },
        "corrections_from_draft": [
            "DRAM cell area: 4e-14 -> 4e-12 cm^2 (100x)",
            "Poynting vector: max-E x max-B -> E^2/eta_0 (6x)",
            "energy per hit: 2 MeV -> 0.26 MeV full-die / 0.37 keV per-cell",
        ],
    })
}
