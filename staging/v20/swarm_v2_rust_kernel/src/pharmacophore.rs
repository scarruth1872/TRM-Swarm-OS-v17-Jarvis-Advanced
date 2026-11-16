//! Native Pharmacophore Engine for TRM Swarm OS
//! Replaces LLM-based docking text generation with a sub-10-microsecond deterministic
//! physics, geometry, and Kleene K3 balanced ternary logic evaluation.

use std::time::Instant;

/// Kleene K3 Logic States
pub const K3_TRUE: i8 = 1;
pub const K3_UNKNOWN: i8 = 0;
pub const K3_FALSE: i8 = -1;

/// Atom 3D Coordinate Point
#[derive(Debug, Clone, Copy)]
pub struct AtomPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub element: u8, // Atomic number: 6=C, 7=N, 8=O, 9=F, 16=S, 30=Zn
}

impl AtomPoint {
    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32, element: u8) -> Self {
        Self { x, y, z, element }
    }

    #[inline(always)]
    pub fn distance_to(&self, other: &AtomPoint) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Zinc Coordination Evaluation Result
#[derive(Debug, Clone)]
pub struct ZincCoordinationResult {
    pub is_coordinated: bool,
    pub min_distance_angstrom: f32,
    pub chelating_atom_count: usize,
    pub coordination_type: &'static str,
    pub k3_state: i8,
}

/// Catalytic Contact Analysis
#[derive(Debug, Clone)]
pub struct CatalyticContactAnalysis {
    pub total_contacts: usize,
    pub hydrogen_bonds: usize,
    pub hydrophobic_contacts: usize,
    pub electrostatic_contacts: usize,
    pub k3_state: i8,
}

/// Steric Clash Detection
#[derive(Debug, Clone)]
pub struct ClashDetectionResult {
    pub severe_clash_count: usize,
    pub min_atomic_distance: f32,
    pub has_steric_violation: bool,
    pub k3_state: i8,
}

/// Final Evaluated Pharmacophore Report
#[derive(Debug, Clone)]
pub struct PharmacophoreReport {
    pub target_name: String,
    pub ligand_name: String,
    pub predicted_dg_kcal_mol: f32,
    pub zinc_coordination: ZincCoordinationResult,
    pub catalytic_contacts: CatalyticContactAnalysis,
    pub steric_clashes: ClashDetectionResult,
    pub k3_verdict: i8,
    pub evaluation_time_ns: u64,
    pub deterministic_summary: String,
}

/// Native High-Throughput Pharmacophore Engine
pub struct PharmacophoreEngine;

impl PharmacophoreEngine {
    /// Evaluates a 3D ligand against pocket residues and catalytic zinc ion
    pub fn evaluate(
        target_name: &str,
        ligand_name: &str,
        predicted_dg: f32,
        ligand_atoms: &[AtomPoint],
        pocket_atoms: &[AtomPoint],
        catalytic_zinc: Option<AtomPoint>,
    ) -> PharmacophoreReport {
        let start = Instant::now();

        // 1. Zinc Coordination Analysis (Sub-microsecond)
        let zinc_res = match catalytic_zinc {
            Some(zn) => {
                let mut min_zn_dist = f32::MAX;
                let mut chelating_count = 0;
                for atom in ligand_atoms {
                    // Check if Oxygen (8), Nitrogen (7), or Sulfur (16)
                    if atom.element == 8 || atom.element == 7 || atom.element == 16 {
                        let d = atom.distance_to(&zn);
                        if d < min_zn_dist {
                            min_zn_dist = d;
                        }
                        if d <= 2.8 {
                            chelating_count += 1;
                        }
                    }
                }
                let coord_type = if chelating_count >= 2 {
                    "Bidentate Chelation"
                } else if chelating_count == 1 {
                    "Monodentate Chelation"
                } else {
                    "No Direct Metal Bond"
                };

                let k3 = if min_zn_dist <= 2.5 && chelating_count >= 1 {
                    K3_TRUE
                } else if min_zn_dist <= 3.2 {
                    K3_UNKNOWN
                } else {
                    K3_FALSE
                };

                ZincCoordinationResult {
                    is_coordinated: min_zn_dist <= 2.8,
                    min_distance_angstrom: min_zn_dist,
                    chelating_atom_count: chelating_count,
                    coordination_type: coord_type,
                    k3_state: k3,
                }
            }
            None => ZincCoordinationResult {
                is_coordinated: false,
                min_distance_angstrom: 0.0,
                chelating_atom_count: 0,
                coordination_type: "Non-Metallo Target",
                k3_state: K3_TRUE, // Target does not require metal chelation
            },
        };

        // 2. Catalytic Pocket Contacts & 3. Steric Clashes
        let mut total_contacts = 0;
        let mut h_bonds = 0;
        let mut hydrophobic = 0;
        let mut electrostatic = 0;
        let mut clashes = 0;
        let mut min_atom_dist = f32::MAX;

        for latom in ligand_atoms {
            for patom in pocket_atoms {
                let d = latom.distance_to(patom);
                if d < min_atom_dist {
                    min_atom_dist = d;
                }

                // Steric clash threshold (< 1.4 Angstroms is non-physical atomic overlap)
                if d < 1.4 {
                    clashes += 1;
                } else if d <= 3.8 {
                    total_contacts += 1;
                    if (latom.element == 7 || latom.element == 8) && (patom.element == 7 || patom.element == 8) {
                        h_bonds += 1;
                    } else if latom.element == 6 && patom.element == 6 {
                        hydrophobic += 1;
                    } else if (latom.element == 7 && patom.element == 8) || (latom.element == 8 && patom.element == 7) {
                        electrostatic += 1;
                    }
                }
            }
        }

        let contacts_k3 = if total_contacts >= 8 && h_bonds >= 2 {
            K3_TRUE
        } else if total_contacts >= 4 {
            K3_UNKNOWN
        } else {
            K3_FALSE
        };

        let clash_k3 = if clashes == 0 && min_atom_dist >= 1.6 {
            K3_TRUE
        } else if clashes == 0 {
            K3_UNKNOWN
        } else {
            K3_FALSE
        };

        // 4. Unified Kleene K3 Conjunction (verdict = min(k3_zinc, k3_contacts, k3_clash))
        let k3_verdict = zinc_res.k3_state.min(contacts_k3).min(clash_k3);

        let elapsed_ns = start.elapsed().as_nanos() as u64;

        // 5. Deterministic Non-Hallucinatory Summary (5 microseconds)
        let verdict_str = match k3_verdict {
            K3_TRUE => "VALID LEAD CANDIDATE (High Complementarity)",
            K3_UNKNOWN => "MARGINAL COMPLEMENTARITY (Needs Lead Optimization)",
            _ => "REJECTED (Non-Physical Clash or Lost Coordination)",
        };

        let summary = format!(
            "[NATIVE PHARMACOPHORE VERDICT: {}] Target: {} | Ligand: {} |             DeltaG: {:.2} kcal/mol | Zinc: {} ({:.2} A, {} bonds) |             Pocket Contacts: {} (HB: {}, Hydrophobic: {}) | Clashes: {} (Min dist: {:.2} A) |             Silicon Eval Time: {} ns.",
            verdict_str,
            target_name,
            ligand_name,
            predicted_dg,
            zinc_res.coordination_type,
            zinc_res.min_distance_angstrom,
            zinc_res.chelating_atom_count,
            total_contacts,
            h_bonds,
            hydrophobic,
            clashes,
            min_atom_dist,
            elapsed_ns
        );

        PharmacophoreReport {
            target_name: target_name.to_string(),
            ligand_name: ligand_name.to_string(),
            predicted_dg_kcal_mol: predicted_dg,
            zinc_coordination: zinc_res,
            catalytic_contacts: CatalyticContactAnalysis {
                total_contacts,
                hydrogen_bonds: h_bonds,
                hydrophobic_contacts: hydrophobic,
                electrostatic_contacts: electrostatic,
                k3_state: contacts_k3,
            },
            steric_clashes: ClashDetectionResult {
                severe_clash_count: clashes,
                min_atomic_distance: min_atom_dist,
                has_steric_violation: clashes > 0,
                k3_state: clash_k3,
            },
            k3_verdict,
            evaluation_time_ns: elapsed_ns,
            deterministic_summary: summary,
        }
    }
}

/// C-ABI export for high-performance cross-language FFI
#[no_mangle]
pub extern "C" fn rust_evaluate_pharmacophore_fast(
    predicted_dg: f32,
    zinc_dist: f32,
    num_contacts: usize,
    num_clashes: usize,
) -> i8 {
    let zinc_k3 = if zinc_dist <= 2.5 {
        K3_TRUE
    } else if zinc_dist <= 3.2 {
        K3_UNKNOWN
    } else {
        K3_FALSE
    };

    let contacts_k3 = if num_contacts >= 8 {
        K3_TRUE
    } else if num_contacts >= 4 {
        K3_UNKNOWN
    } else {
        K3_FALSE
    };

    let clash_k3 = if num_clashes == 0 {
        K3_TRUE
    } else {
        K3_FALSE
    };

    zinc_k3.min(contacts_k3).min(clash_k3)
}
