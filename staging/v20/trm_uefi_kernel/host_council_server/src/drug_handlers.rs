// /native/drug -- the K3 triage verdicts decided by swarm-drug itself.
//
// swarm-drug is a v18 crate that has never been reachable from this kernel. Its `triage` is a pure function of
// (Molecule, tau) and its `parse_sdf` streams a real gzipped SDF, so the crate is wired as a path dependency in
// the way P2 wired swarm-proof: the crate's own function decides and this kernel adds no rule, clamp or default
// on top of it.
//
// Two facts about the crate are recorded rather than smoothed:
//   * `triage(m, tau)` never reads `tau` -- every verdict is identical for every tau. The route reports the tau
//     it was handed and the ruler proves the invariance; the route does not hide it.
//   * `parse_block` reads the counts line at index 3 and the element at token 3 of each atom row, which is the
//     V2000 layout it documents; a block that is not V2000 yields zero counts rather than an error.

use serde_json::{json, Value};

use swarm_drug::Molecule;

fn f32_of(doc: &Value, key: &str) -> Option<f32> {
    doc.get(key).and_then(|v| v.as_f64()).map(|x| x as f32)
}

fn molecule(row: &Value, i: usize) -> Result<Molecule, String> {
    let u = |k: &str| row.get(k).and_then(|v| v.as_u64());
    let chembl_id = row.get("chembl_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    Ok(Molecule {
        chembl_id: if chembl_id.is_empty() { format!("UNKNOWN_{}", i) } else { chembl_id },
        atom_count: u("atom_count").ok_or(format!("molecules[{}].atom_count (u64) is required", i))? as u32,
        heavy_atoms: u("heavy_atoms").ok_or(format!("molecules[{}].heavy_atoms (u64) is required", i))? as u32,
        mw: f32_of(row, "mw").ok_or(format!("molecules[{}].mw (number) is required", i))?,
    })
}

/// The K3 verdict for each caller-supplied molecule: `swarm_drug::triage` decides, and the label and detail
/// strings are the crate's own.
pub fn triage(doc: &Value) -> Result<Value, String> {
    let rows = doc
        .get("molecules")
        .and_then(|v| v.as_array())
        .ok_or("`molecules` (array) is required")?;
    if rows.is_empty() {
        return Err("`molecules` is empty: there is nothing to decide".into());
    }
    let tau = f32_of(doc, "tau").unwrap_or(0.35);
    let mut out = Vec::with_capacity(rows.len());
    for (i, r) in rows.iter().enumerate() {
        let m = molecule(r, i)?;
        let (verdict, detail) = swarm_drug::triage(&m, tau);
        out.push(json!({
            "chembl_id": m.chembl_id,
            "atom_count": m.atom_count,
            "heavy_atoms": m.heavy_atoms,
            "mw": m.mw,
            "verdict": verdict,
            "label": match verdict { 1 => "drug-like", -1 => "blocked", _ => "analyze" },
            "detail": detail,
        }));
    }
    Ok(json!({
        "count": out.len(),
        "tau": tau,
        "results": out,
        "basis": "swarm_drug::triage(&Molecule, tau) -- the crate in wsl_crates/trm-rust/swarm-drug decides; \
                  this route adds no rule, clamp or default",
        "recorded_not_smoothed": "the crate's `tau` parameter is never read: verdicts are identical for every \
                                  tau, which the ruler proves instead of the route concealing",
    }))
}

/// Stream a real gzipped SDF through the crate's own parser. The path is the caller's: this route cannot know
/// where an operator keeps the data and invents nothing about it.
pub fn sdf(doc: &Value) -> Result<Value, String> {
    let path = doc
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("`path` (string) is required")?;
    let limit = doc.get("limit").and_then(|v| v.as_u64()).unwrap_or(16) as usize;
    if !std::path::Path::new(path).is_file() {
        return Err(format!(
            "no file at `{}`: the parser reads a real gzipped SDF, and this route will not invent one",
            path
        ));
    }
    let mols = swarm_drug::parse_sdf(path, limit);
    let rows: Vec<Value> = mols
        .iter()
        .map(|m| {
            json!({
                "chembl_id": m.chembl_id,
                "atom_count": m.atom_count,
                "heavy_atoms": m.heavy_atoms,
                "mw": m.mw,
            })
        })
        .collect();
    Ok(json!({
        "path": path,
        "limit": limit,
        "count": rows.len(),
        "molecules": rows,
        "basis": "swarm_drug::parse_sdf -- the crate streams the gzip itself, never the whole file in memory",
        "limit_note": "`limit` is the crate's own bound, so `count < limit` means the file ended first",
    }))
}
