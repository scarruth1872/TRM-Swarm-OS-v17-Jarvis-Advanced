// /native/antibody/* -- the SAbDab2 structural scorer (MOTION-S1).
//
// Scores a candidate antibody's paratope against a target antigen from a real SAbDab2 complex
// (mmCIF). This replaces the retired sequence-only heuristic (strip_antibody_binding.rs, which was
// anti-correlated r=-0.197) with a geometry measurement: antibody atoms within 4.5 A of any antigen
// atom, counted from the mmCIF atom_site loop. Every number is measured; a complex with no antigen
// is reported as a boundary, never a fabricated score.

use serde_json::{json, Value};

const CONTACT_A: f64 = 4.5;

struct Atom {
    chain: String,
    residue: String,
    pos: [f64; 3],
}

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt()
}

/// Resolve a cif_path against the repo root when it is relative (the kernel's CWD is not the repo).
fn resolve(path: &str) -> String {
    let p = std::path::Path::new(path);
    if p.is_absolute() {
        return path.to_string();
    }
    crate::k3_retrieval::repo_root()
        .map(|r| r.join(path).to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Parse the mmCIF atom_site loop (column names -> data rows) into chain/residue/position atoms.
/// Only ATOM (polymer) records are kept; HETATM (water, ligands) are skipped so the score stays to
/// the protein chains.
fn parse_cif_atoms(path: &str) -> Result<Vec<Atom>, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let mut cols: Vec<String> = Vec::new();
    let mut in_atom_loop = false;
    let mut atoms = Vec::new();
    for line in raw.lines() {
        if line.starts_with("_atom_site.") {
            in_atom_loop = true;
            cols.push(line["_atom_site.".len()..].trim().to_string());
            continue;
        }
        if in_atom_loop && line.starts_with('_') {
            in_atom_loop = false;
            continue;
        }
        if in_atom_loop && line.starts_with("ATOM") {
            let toks: Vec<&str> = line.split_whitespace().collect();
            if let Some(atom) = atom_from(&cols, &toks) {
                atoms.push(atom);
            }
        }
    }
    if atoms.is_empty() {
        return Err(format!("no atom_site records parsed from {path}"));
    }
    Ok(atoms)
}

fn atom_from(cols: &[String], toks: &[&str]) -> Option<Atom> {
    let get = |name: &str| -> Option<&str> {
        cols.iter().position(|c| c == name).and_then(|i| toks.get(i).copied())
    };
    let chain = get("auth_asym_id")?.to_string();
    let residue = get("label_comp_id")?.to_string();
    let x = get("Cartn_x")?.parse::<f64>().ok()?;
    let y = get("Cartn_y")?.parse::<f64>().ok()?;
    let z = get("Cartn_z")?.parse::<f64>().ok()?;
    Some(Atom { chain, residue, pos: [x, y, z] })
}

/// List the chains in an mmCIF complex with their atom counts, so a caller can discover which are
/// the antibody chains vs the antigen.
pub fn list_chains(doc: &Value) -> Result<Value, String> {
    let path = doc.get("cif_path").and_then(|v| v.as_str())
        .ok_or("`cif_path` (a SAbDab2 mmCIF path) is required")?;
    let atoms = parse_cif_atoms(&resolve(path))?;
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for a in &atoms {
        *counts.entry(&a.chain).or_insert(0) += 1;
    }
    let chains: Vec<Value> = counts
        .iter()
        .map(|(c, n)| json!({"chain": c, "atom_count": n}))
        .collect();
    Ok(json!({
        "engine": "sabdab2_structural_scorer", "pdb": path,
        "chains": chains, "total_atoms": atoms.len(), "status": "real_result",
    }))
}

/// Score a candidate antibody against a target antigen: the paratope is the set of antibody atoms
/// within CONTACT_A of any antigen atom. A complex with no antigen chain is a boundary, not a score.
pub fn score_cdr_antigen(doc: &Value) -> Result<Value, String> {
    let path = doc.get("cif_path").and_then(|v| v.as_str())
        .ok_or("`cif_path` (a SAbDab2 mmCIF path) is required")?;
    let chains: Vec<String> = doc.get("antibody_chains")
        .and_then(|v| v.as_array())
        .ok_or("`antibody_chains` (the antibody chain ids) is required")?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    if chains.is_empty() {
        return Err("`antibody_chains` must name at least one chain".to_string());
    }

    let atoms = parse_cif_atoms(&resolve(path))?;
    // SAbDab2 filenames use a "+" placeholder for single-domain antibodies (heavy-only/nanobody,
    // no light chain) — e.g. `pdb_..._E_+.cif` means antibody chain E alone. A placeholder matches
    // no atom, so drop any named chain with zero atoms rather than treating it as a real (missing)
    // chain. This makes the antibody/antigen split correct by construction instead of correct by
    // accident.
    let real_chains: Vec<String> = chains
        .iter()
        .filter(|c| atoms.iter().any(|a| &a.chain == *c))
        .cloned()
        .collect();
    let antibody: Vec<&Atom> = atoms.iter().filter(|a| real_chains.contains(&a.chain)).collect();
    let antigen: Vec<&Atom> = atoms.iter().filter(|a| !real_chains.contains(&a.chain)).collect();

    if antibody.is_empty() {
        return Err(format!("no antibody atoms found for chains {:?}", chains));
    }
    if antigen.is_empty() {
        return Ok(json!({
            "engine": "sabdab2_structural_scorer", "status": "boundary",
            "boundary": "no antigen chain",
            "pdb": path, "antibody_chains": chains,
            "basis": "the complex has only the antibody chains; there is no antigen to score against, so no contact score is fabricated",
        }));
    }

    let mut contacts = 0usize;
    let mut dist_sum = 0.0;
    let mut residues: std::collections::BTreeSet<(String, String)> = std::collections::BTreeSet::new();
    for a in &antibody {
        let mut best: Option<f64> = None;
        for g in &antigen {
            let d = dist(&a.pos, &g.pos);
            if d <= CONTACT_A {
                best = Some(best.map_or(d, |b: f64| b.min(d)));
            }
        }
        if let Some(d) = best {
            contacts += 1;
            dist_sum += d;
            residues.insert((a.chain.clone(), a.residue.clone()));
        }
    }

    let mean = if contacts > 0 { dist_sum / contacts as f64 } else { 0.0 };
    Ok(json!({
        "engine": "sabdab2_structural_scorer", "status": "real_result",
        "pdb": path, "antibody_chains": chains,
        "antibody_atoms": antibody.len(), "antigen_atoms": antigen.len(),
        "paratope_contacts": contacts, "paratope_residues": residues.len(),
        "mean_contact_distance_angstrom": (mean * 1000.0).round() / 1000.0,
        "contact_cutoff_angstrom": CONTACT_A,
        "basis": "the paratope = antibody atoms within 4.5 A of any antigen atom, counted from the SAbDab2 mmCIF atom_site loop (geometry, not the retired sequence heuristic)",
    }))
}
