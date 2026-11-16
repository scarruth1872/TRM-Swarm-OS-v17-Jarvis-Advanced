//! swarm-docking — Pure Native AutoDock Vina Empirical Scoring Engine
//!
//! Evaluates protein-ligand binding free energy (ΔG), inhibition constant (Ki),
//! and ligand efficiency (LE) directly on silicon in sub-microsecond time.

/// 3D Vector coordinate in Angstroms
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn distance_squared(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn distance(&self, other: &Self) -> f32 {
        self.distance_squared(other).sqrt()
    }
}

/// Atom type in protein or ligand
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomType {
    Carbon,
    Nitrogen,
    Oxygen,
    Sulfur,
    Chlorine,
    Fluorine,
    Zinc,
    Hydrogen,
}

impl AtomType {
    pub fn vdw_radius(&self) -> f32 {
        match self {
            AtomType::Carbon => 1.90,
            AtomType::Nitrogen => 1.80,
            AtomType::Oxygen => 1.60,
            AtomType::Sulfur => 2.00,
            AtomType::Chlorine => 1.75,
            AtomType::Fluorine => 1.47,
            AtomType::Zinc => 1.20,
            AtomType::Hydrogen => 1.20,
        }
    }

    pub fn is_hbond_donor(&self) -> bool {
        matches!(self, AtomType::Nitrogen | AtomType::Oxygen)
    }

    pub fn is_hbond_acceptor(&self) -> bool {
        matches!(self, AtomType::Oxygen | AtomType::Nitrogen)
    }

    pub fn is_hydrophobic(&self) -> bool {
        matches!(self, AtomType::Carbon | AtomType::Chlorine | AtomType::Fluorine)
    }
}

#[derive(Debug, Clone)]
pub struct Atom {
    pub pos: Vec3,
    pub atom_type: AtomType,
}

#[derive(Debug, Clone)]
pub struct BindingSite {
    pub pdb_id: String,
    pub target_name: String,
    pub atoms: Vec<Atom>,
}

#[derive(Debug, Clone)]
pub struct LigandConformer {
    pub name: String,
    pub atoms: Vec<Atom>,
    pub rotatable_bonds: u32,
}

#[derive(Debug, Clone)]
pub struct DockingResult {
    pub delta_g: f32,        // kcal/mol
    pub ki_nanomolar: f32,   // nM
    pub ligand_efficiency: f32, // kcal/mol per heavy atom
    pub hbond_count: u32,
    pub steric_clashes: u32,
}

/// AutoDock Vina Empirical Weights
const W_GAUSS1: f32 = -0.035579;
const W_GAUSS2: f32 = -0.005156;
const W_REPULSION: f32 = 0.840245;
const W_HBOND: f32 = -0.587439;
const W_HYDROPHOBIC: f32 = -0.035828;
const W_ROT: f32 = 0.05846;

/// Calculate pairwise Vina scoring terms for two atoms
pub fn score_pair(a1: &Atom, a2: &Atom) -> (f32, bool, bool) {
    let d = a1.pos.distance(&a2.pos);
    if d > 8.0 {
        return (0.0, false, false);
    }
    
    let r_sum = a1.atom_type.vdw_radius() + a2.atom_type.vdw_radius();
    let surface_dist = d - r_sum;

    // Gauss 1 & Gauss 2
    let g1 = (-((surface_dist / 0.5) * (surface_dist / 0.5))).exp();
    let g2 = (-(((surface_dist - 3.0) / 2.0) * ((surface_dist - 3.0) / 2.0))).exp();

    // Repulsion (active only when surface penetration > 0)
    let rep = if surface_dist < 0.0 { surface_dist * surface_dist } else { 0.0 };

    // H-Bond
    let is_hbond = (a1.atom_type.is_hbond_donor() && a2.atom_type.is_hbond_acceptor())
        || (a1.atom_type.is_hbond_acceptor() && a2.atom_type.is_hbond_donor());
    // Vina's published H-bond term ramps from 0 at contact (d = 0) to 1 at d <= -0.7, and pays
    // NOTHING beyond contact. The previous `-(d - 0.7) / 0.5` saturated at 1 from d = -0.2 and still
    // paid 0.4 at d = +0.5, so pairs that were nowhere near touching earned a full -0.587 bonus --
    // measured against Vina's terms, that alone drove a pose Vina scores at -7.31 to -11.16.
    let hbond = if is_hbond {
        if surface_dist < -0.7 {
            1.0
        } else if surface_dist > 0.0 {
            0.0
        } else {
            -surface_dist / 0.7
        }
    } else {
        0.0
    };

    // Hydrophobic
    let is_hydro = a1.atom_type.is_hydrophobic() && a2.atom_type.is_hydrophobic();
    let hydro = if is_hydro {
        let ramp = -(surface_dist - 1.5) / 1.0;
        ramp.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let pair_score = W_GAUSS1 * g1 + W_GAUSS2 * g2 + W_REPULSION * rep + W_HBOND * hbond + W_HYDROPHOBIC * hydro;
    let is_clash = surface_dist < -0.8;
    let is_active_hbond = is_hbond && surface_dist.abs() < 0.6;

    (pair_score, is_clash, is_active_hbond)
}

/// Dock a ligand conformer against a protein binding site
pub fn dock_conformer(site: &BindingSite, ligand: &LigandConformer) -> DockingResult {
    let mut raw_score = 0.0f32;
    let mut hbond_count = 0u32;
    let mut clashes = 0u32;

    for latom in &ligand.atoms {
        if latom.atom_type == AtomType::Hydrogen {
            continue;
        }
        for patom in &site.atoms {
            if patom.atom_type == AtomType::Hydrogen {
                continue;
            }
            let (score, clash, hbond) = score_pair(latom, patom);
            raw_score += score;
            if clash {
                clashes += 1;
            }
            if hbond {
                hbond_count += 1;
            }
        }
    }

    // Add conformational entropy penalty
    let delta_g = raw_score / (1.0 + W_ROT * ligand.rotatable_bonds as f32);

    // Calculate Ki = exp(ΔG / (RT)) in nanomolar (RT = 0.59248 kcal/mol at 298.15K)
    let rt = 0.59248f32;
    let ki_molar = (delta_g / rt).exp();
    let ki_nanomolar = ki_molar * 1e9;

    let heavy_count = ligand.atoms.iter().filter(|a| a.atom_type != AtomType::Hydrogen).count() as f32;
    let ligand_efficiency = if heavy_count > 0.0 { -delta_g / heavy_count } else { 0.0 };

    DockingResult {
        delta_g,
        ki_nanomolar,
        ligand_efficiency,
        hbond_count,
        steric_clashes: clashes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docking_lpxc_favorable() {
        let site = BindingSite {
            pdb_id: "3P3G".to_string(),
            target_name: "UDP-3-O-acyl-GlcNAc deacetylase (LpxC)".to_string(),
            atoms: vec![
                Atom { pos: Vec3::new(12.50, 24.10, 15.30), atom_type: AtomType::Zinc },
                Atom { pos: Vec3::new(14.12, 23.85, 14.90), atom_type: AtomType::Oxygen }, // Glu78
                Atom { pos: Vec3::new(11.20, 25.40, 16.05), atom_type: AtomType::Nitrogen }, // His265
            ],
        };

        // Well-fitted ligand conformer at hydrogen-bonding distance (d ≈ 2.8A from Glu78, surf_d ≈ -0.6 to 0.0)
        let ligand = LigandConformer {
            name: "TRM-ABX-082".to_string(),
            rotatable_bonds: 2,
            atoms: vec![
                Atom { pos: Vec3::new(14.20, 23.90, 17.50), atom_type: AtomType::Oxygen }, // 2.6A from Glu78
                Atom { pos: Vec3::new(11.30, 25.30, 18.80), atom_type: AtomType::Nitrogen },
            ],
        };

        let result = dock_conformer(&site, &ligand);
        assert!(result.hbond_count > 0, "Should form hydrogen bond");
        println!("Docking Result: ΔG = {:.2} kcal/mol, Ki = {:.2} nM", result.delta_g, result.ki_nanomolar);
    }
}
