//! In-kernel compound library screening.
//!
//! Loads the CHMB 3D conformer library written by scratch/build_chembl_library.py and docks every
//! molecule against the measured 1M17 EGFR pocket with swarm_docking::dock_conformer. The scorer
//! decides; this module adds no clamp, correction or default on top of it.
//!
//! The pocket geometry below is measured from 1M17.pdb (68 protein atoms within 8.0 A of the
//! erlotinib centroid), translated so the AQ4 centroid is the origin. It is generated, not
//! hand-authored: see scratch/build_chembl_library.py's companion extractor.

use std::time::Instant;

use swarm_docking::{Atom, AtomType, BindingSite, LigandConformer, Vec3};

/// One molecule from the CHMB file.
pub struct LibMol {
    pub id: String,
    pub atoms: Vec<(u8, f32, f32, f32)>,
}

fn atom_type_of(z: u8) -> AtomType {
    match z {
        1 => AtomType::Hydrogen,
        7 => AtomType::Nitrogen,
        8 => AtomType::Oxygen,
        9 => AtomType::Fluorine,
        16 => AtomType::Sulfur,
        30 => AtomType::Zinc,
        _ => AtomType::Carbon,
    }
}

/// Read the CHMB binary: magic + version + count, then (id_len, id, n_atoms, [element, x, y, z]).
pub fn load_library(path: &str) -> Result<Vec<LibMol>, String> {
    let data = std::fs::read(path).map_err(|e| format!("read {}: {}", path, e))?;
    if data.len() < 12 || &data[0..4] != b"CHMB" {
        return Err(format!("{} is not a CHMB library", path));
    }
    let count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    let mut p = 12usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if p + 2 > data.len() {
            return Err("truncated id length".into());
        }
        let idlen = u16::from_le_bytes([data[p], data[p + 1]]) as usize;
        p += 2;
        if p + idlen + 2 > data.len() {
            return Err("truncated id".into());
        }
        let id = String::from_utf8_lossy(&data[p..p + idlen]).to_string();
        p += idlen;
        let n = u16::from_le_bytes([data[p], data[p + 1]]) as usize;
        p += 2;
        let mut atoms = Vec::with_capacity(n);
        for _ in 0..n {
            if p + 13 > data.len() {
                return Err("truncated atom".into());
            }
            let e = data[p];
            let x = f32::from_le_bytes([data[p + 1], data[p + 2], data[p + 3], data[p + 4]]);
            let y = f32::from_le_bytes([data[p + 5], data[p + 6], data[p + 7], data[p + 8]]);
            let z = f32::from_le_bytes([data[p + 9], data[p + 10], data[p + 11], data[p + 12]]);
            p += 13;
            atoms.push((e, x, y, z));
        }
        out.push(LibMol { id, atoms });
    }
    Ok(out)
}

fn build_site() -> BindingSite {
    BindingSite {
        pdb_id: "1M17".to_string(),
        target_name: "EGFR kinase domain".to_string(),
        atoms: EGFR_POCKET
            .iter()
            .map(|&(x, y, z, e)| Atom { pos: Vec3::new(x, y, z), atom_type: atom_type_of(e) })
            .collect(),
    }
}

fn pose_transform(p: usize) -> (f32, f32, f32, f32, f32, f32, f32) {
    if p == 0 {
        return (0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    }
    let s = p as f32;
    let a = (s * 1.618).cos() * std::f32::consts::PI;
    let b = (s * 0.927).sin() * std::f32::consts::PI;
    let (sa, ca) = a.sin_cos();
    let (sb, cb) = b.sin_cos();
    let tx = (s * 0.713).sin() * 2.0;
    let ty = (s * 1.137).cos() * 2.0;
    let tz = (s * 0.451).sin() * 2.0;
    (sa, ca, sb, cb, tx, ty, tz)
}

/// Score one molecule: best of `poses` rotations, each swept over a 3x3x3 translation grid.
fn score_mol(site: &BindingSite, m: &LibMol, poses: usize) -> Option<(String, f32, u32, u32)> {
    const OFFSETS: [f32; 3] = [-2.0, 0.0, 2.0];
    let mut best = f32::INFINITY;
    let mut bh = 0u32;
    let mut bc = 0u32;
    for p in 0..poses {
        let (sa, ca, sb, cb, tx, ty, tz) = pose_transform(p);
        let rotated: Vec<(u8, f32, f32, f32)> = m
            .atoms
            .iter()
            .map(|&(e, x, y, z)| {
                let (x1, z1) = (x * ca + z * sa, -x * sa + z * ca);
                let (y2, z2) = (y * cb - z1 * sb, y * sb + z1 * cb);
                (e, x1 + tx, y2 + ty, z2 + tz)
            })
            .collect();
        for &ox in OFFSETS.iter() {
            for &oy in OFFSETS.iter() {
                for &oz in OFFSETS.iter() {
                    let atoms: Vec<Atom> = rotated
                        .iter()
                        .map(|&(e, x, y, z)| Atom {
                            pos: Vec3::new(x + ox, y + oy, z + oz),
                            atom_type: atom_type_of(e),
                        })
                        .collect();
                    let lig = LigandConformer { name: m.id.clone(), atoms, rotatable_bonds: 5 };
                    let d = swarm_docking::dock_conformer(site, &lig);
                    if d.delta_g < best {
                        best = d.delta_g;
                        bh = d.hbond_count;
                        bc = d.steric_clashes;
                    }
                }
            }
        }
    }
    if best.is_finite() {
        Some((m.id.clone(), best, bh, bc))
    } else {
        None
    }
}

/// Screen every molecule in the library. Returns the top hits and the measured distribution.
pub fn screen(path: &str, poses: usize, top_n: usize) -> Result<serde_json::Value, String> {
    let mols = load_library(path)?;
    let site = build_site();
    let poses = poses.max(1);

    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).max(1);
    let chunk = (mols.len() + threads - 1) / threads.max(1);
    let t0 = Instant::now();
    let parts: Vec<(Vec<(String, f32, u32, u32)>, usize)> = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for slice in mols.chunks(chunk.max(1)) {
            let site = &site;
            handles.push(scope.spawn(move || {
                let mut local: Vec<(String, f32, u32, u32)> = Vec::with_capacity(slice.len());
                let mut bad = 0usize;
                for m in slice.iter() {
                    match score_mol(site, m, poses) {
                        Some(v) => local.push(v),
                        None => bad += 1,
                    }
                }
                (local, bad)
            }));
        }
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let mut scored: Vec<(String, f32, u32, u32)> = Vec::new();
    let mut failed = 0usize;
    for (part, bad) in parts {
        scored.extend(part);
        failed += bad;
    }
    let total = scored.len();
    let mean = if total > 0 {
        scored.iter().map(|s| s.1).sum::<f32>() / total as f32
    } else {
        0.0
    };
    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let min_dg = scored.first().map(|s| s.1).unwrap_or(0.0);
    let max_dg = scored.last().map(|s| s.1).unwrap_or(0.0);
    let top: Vec<serde_json::Value> = scored
        .iter()
        .take(top_n)
        .map(|(id, dg, hb, cl)| serde_json::json!({"id": id, "dg_kcal_mol": dg, "hbonds": hb, "clashes": cl}))
        .collect();

    Ok(serde_json::json!({
        "molecules": total,
        "failed": failed,
        "poses": poses,
        "threads": threads,
        "elapsed_ms": elapsed_ms,
        "throughput_per_sec": if elapsed_ms > 0.0 { total as f64 / (elapsed_ms / 1000.0) } else { 0.0 },
        "min_dg": min_dg,
        "mean_dg": mean,
        "max_dg": max_dg,
        "top": top,
        "caveat": "scorer is uncalibrated (crystal pose -5.26 vs Vina ~-8.7); rotatable_bonds fixed at 5 for every molecule"
    }))
}


/// (x, y, z, atomic_number) measured from 1M17.pdb: every protein atom within 8.0 A of the
/// erlotinib (AQ4) centroid, translated so that centroid is the origin.
pub const EGFR_POCKET: [(f32, f32, f32, u8); 68] = [
    (-2.066, -3.783, -6.202, 7),
    (-2.734, -2.711, -5.470, 6),
    (-2.578, -1.373, -6.203, 6),
    (-3.406, -0.474, -6.045, 8),
    (-2.178, -2.602, -4.041, 6),
    (-2.391, -3.791, -3.093, 6),
    (-1.631, -3.584, -1.793, 6),
    (-3.855, -3.992, -2.810, 6),
    (-1.526, -1.252, -7.011, 7),
    (-1.292, -0.022, -7.744, 6),
    (2.894, -2.200, -6.657, 7),
    (2.769, -2.662, -5.284, 6),
    (1.750, -3.792, -5.312, 6),
    (0.616, -3.610, -5.729, 8),
    (2.283, -1.534, -4.326, 6),
    (2.093, -2.073, -2.914, 6),
    (3.290, -0.402, -4.289, 6),
    (2.171, -4.971, -4.891, 7),
    (1.292, -6.123, -4.874, 6),
    (0.808, -6.422, -3.468, 6),
    (1.338, -5.912, -2.485, 8),
    (2.656, -6.830, 1.643, 7),
    (2.604, -5.968, 0.474, 6),
    (3.988, -6.116, -0.149, 6),
    (5.010, -6.111, 0.555, 8),
    (2.357, -4.516, 0.864, 6),
    (4.005, -6.279, -1.468, 7),
    (4.621, -5.046, -4.064, 8),
    (5.389, -3.362, 3.095, 8),
    (2.113, -5.084, 5.590, 6),
    (2.239, -3.910, 5.223, 8),
    (1.048, -5.827, 5.306, 7),
    (-0.084, -5.288, 4.584, 6),
    (-0.887, -4.318, 5.465, 6),
    (-0.995, -6.425, 4.138, 6),
    (-2.201, -5.995, 3.316, 6),
    (-1.696, -5.441, 2.012, 6),
    (-1.116, -3.119, 4.950, 7),
    (-1.923, -2.125, 5.642, 6),
    (-3.231, -2.126, 4.811, 6),
    (-3.449, -1.280, 3.954, 8),
    (-1.196, -0.786, 5.617, 6),
    (0.183, -0.832, 6.253, 6),
    (-4.150, -3.048, 5.135, 7),
    (-5.444, -3.280, 4.490, 6),
    (-6.277, -2.089, 4.053, 6),
    (-6.911, -2.141, 3.004, 8),
    (-6.450, 1.202, 3.705, 6),
    (-5.273, 0.867, 3.170, 7),
    (-4.525, 1.780, 2.325, 6),
    (-4.053, 3.062, 2.974, 6),
    (-4.028, 3.159, 4.198, 8),
    (-3.698, 4.052, 2.153, 7),
    (-3.204, 5.334, 2.649, 6),
    (-2.448, 6.105, 1.556, 6),
    (-3.472, 6.897, 0.327, 16),
    (0.492, 7.633, 1.322, 8),
    (-0.072, 4.941, 6.285, 7),
    (-0.916, 3.803, 5.954, 6),
    (-0.055, 2.605, 5.515, 6),
    (0.779, 2.876, 4.241, 6),
    (1.640, 1.695, 3.882, 6),
    (-0.127, 3.210, 3.075, 6),
    (-2.947, 2.817, 6.804, 7),
    (4.843, 3.896, 4.726, 6),
    (5.543, 2.819, 4.093, 8),
    (3.960, 4.598, 3.735, 6),
    (4.813, 5.257, 0.453, 8),
];
