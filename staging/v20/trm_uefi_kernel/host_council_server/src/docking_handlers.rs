// /native/docking -- the Vina empirical scoring decided by swarm-docking itself.
//
// swarm-docking is a v18 crate that no route reached. Its `score_pair` and `dock_conformer` are pure functions of
// their inputs, so it is wired as a path dependency the way P2 wired swarm-proof and the drug harvest wired
// swarm-drug: the crate decides and this kernel adds no rule, clamp, default or correction on top of it.
//
// Recorded rather than smoothed:
//   * the crate carries its OWN correction of Vina's H-bond term -- the old ramp saturated at 1 from d = -0.2 and
//     still paid 0.4 beyond contact, which the crate's comment says drove a pose Vina scores at -7.31 to -11.16.
//     This route reports whatever the crate computes; the correction belongs to the crate, not to the boundary.
//   * `score_pair` returns (score, is_clash, is_active_hbond) with the crate's own thresholds (d > 8.0 A is
//     zero, clash below -0.8 A of surface distance, active H-bond inside |0.6|). None of them is reimplemented.
//   * hydrogens are skipped by `dock_conformer`, so a caller sending them gets no contribution from them --
//     that is the crate's behaviour and it is reported, not "fixed".

use serde_json::{json, Value};

use swarm_docking::{Atom, AtomType, BindingSite, LigandConformer, Vec3};

fn atom_type(name: &str) -> Result<AtomType, String> {
    Ok(match name.to_ascii_lowercase().as_str() {
        "c" | "carbon" => AtomType::Carbon,
        "n" | "nitrogen" => AtomType::Nitrogen,
        "o" | "oxygen" => AtomType::Oxygen,
        "s" | "sulfur" | "sulphur" => AtomType::Sulfur,
        "cl" | "chlorine" => AtomType::Chlorine,
        "f" | "fluorine" => AtomType::Fluorine,
        "zn" | "zinc" => AtomType::Zinc,
        "h" | "hydrogen" => AtomType::Hydrogen,
        other => {
            return Err(format!(
                "unknown atom type `{}`; expected one of C, N, O, S, Cl, F, Zn, H (the crate's AtomType)",
                other
            ))
        }
    })
}

fn atom(row: &Value, i: usize, what: &str) -> Result<Atom, String> {
    let pos = row
        .get("pos")
        .and_then(|v| v.as_array())
        .ok_or(format!("{}[{}].pos (array of 3 numbers, Angstroms) is required", what, i))?;
    if pos.len() != 3 {
        return Err(format!("{}[{}].pos must have exactly 3 coordinates, got {}", what, i, pos.len()));
    }
    let v: Vec<f32> = pos
        .iter()
        .map(|x| x.as_f64().map(|f| f as f32).ok_or(()))
        .collect::<Result<_, _>>()
        .map_err(|_| format!("{}[{}].pos must be 3 numbers", what, i))?;
    let t = row
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or(format!("{}[{}].type (string, e.g. \"C\") is required", what, i))?;
    Ok(Atom { pos: Vec3::new(v[0], v[1], v[2]), atom_type: atom_type(t)? })
}

fn atoms(doc: &Value, key: &str) -> Result<Vec<Atom>, String> {
    let rows = doc.get(key).and_then(|v| v.as_array()).ok_or(format!("`{}` (array) is required", key))?;
    if rows.is_empty() {
        return Err(format!("`{}` is empty: there is nothing to score", key));
    }
    rows.iter().enumerate().map(|(i, r)| atom(r, i, key)).collect()
}

/// The pairwise term for two atoms, exactly as `swarm_docking::score_pair` computes it.
pub fn score_pair(doc: &Value) -> Result<Value, String> {
    let a = atom(doc.get("a").ok_or("`a` (object) is required")?, 0, "a")?;
    let b = atom(doc.get("b").ok_or("`b` (object) is required")?, 0, "b")?;
    let (score, clash, hbond) = swarm_docking::score_pair(&a, &b);
    Ok(json!({
        "score": score,
        "is_clash": clash,
        "is_active_hbond": hbond,
        "distance_angstrom": a.pos.distance(&b.pos),
        "surface_distance": a.pos.distance(&b.pos)
            - (a.atom_type.vdw_radius() + b.atom_type.vdw_radius()),
        "basis": "swarm_docking::score_pair -- the crate's Vina weights and its own H-bond ramp decide; this \
                  route adds nothing",
    }))
}

/// Score a ligand conformer against a binding site, exactly as `swarm_docking::dock_conformer` computes it.
pub fn dock(doc: &Value) -> Result<Value, String> {
    let site_atoms = atoms(doc, "site_atoms")?;
    let lig_atoms = atoms(doc, "ligand_atoms")?;
    let site = BindingSite {
        pdb_id: doc.get("pdb_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        target_name: doc.get("target_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        atoms: site_atoms,
    };
    let ligand = LigandConformer {
        name: doc.get("ligand_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        atoms: lig_atoms,
        rotatable_bonds: doc.get("rotatable_bonds").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
    };
    let r = swarm_docking::dock_conformer(&site, &ligand);
    Ok(json!({
        "delta_g_kcal_per_mol": r.delta_g,
        "ki_nanomolar": r.ki_nanomolar,
        "ligand_efficiency": r.ligand_efficiency,
        "hbond_count": r.hbond_count,
        "steric_clashes": r.steric_clashes,
        "site": {"pdb_id": site.pdb_id, "target_name": site.target_name, "atom_count": site.atoms.len()},
        "ligand": {"name": ligand.name, "atom_count": ligand.atoms.len(),
                   "rotatable_bonds": ligand.rotatable_bonds},
        "basis": "swarm_docking::dock_conformer -- the crate sums its own pairwise terms, applies its own \
                  rotatable-bond penalty and its own Ki/LE formulas",
        "notes": [
            "hydrogens are skipped by the crate on both sides, so they contribute nothing (its behaviour, \
             reported rather than changed)",
            "Ki = exp(dG / 0.59248) * 1e9 and LE = -dG / heavy_atom_count are the crate's formulas",
        ],
    }))
}

/// Count the ligand heavy-atoms within 4.5 A of any site atom — a raw geometric contact count,
/// deliberately simpler than `dock_conformer` (no Vina weights, no H-bond ramp, no rotatable-bond
/// penalty). The comparison between this and `dock_conformer` is the two-method disagreement Task 9
/// asks for: a contact counter can report a bound-looking geometry the Vina score rejects.
pub fn contacts(doc: &Value) -> Result<Value, String> {
    const CONTACT_A: f32 = 4.5;
    let site_atoms = atoms(doc, "site_atoms")?;
    let lig_atoms = atoms(doc, "ligand_atoms")?;
    let mut contacting = 0usize;
    let mut total_heavy = 0usize;
    for l in &lig_atoms {
        if l.atom_type == AtomType::Hydrogen {
            continue;
        }
        total_heavy += 1;
        if site_atoms.iter().any(|s| s.pos.distance(&l.pos) <= CONTACT_A) {
            contacting += 1;
        }
    }
    Ok(json!({
        "engine": "geometric_contact_counter",
        "ligand_heavy_atoms": total_heavy,
        "contacting_atoms": contacting,
        "contact_cutoff_angstrom": CONTACT_A,
        "contact_fraction": if total_heavy > 0 { contacting as f64 / total_heavy as f64 } else { 0.0 },
        "basis": "raw heavy-atom contact count within 4.5 A of the site; no Vina weights, no H-bond ramp -- the deliberately simpler method for the two-method comparison",
    }))
}
