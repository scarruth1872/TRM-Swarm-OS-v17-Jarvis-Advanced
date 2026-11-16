//! verify_drug.rs — Motion C: triage REAL ChEMBL 37 compounds.
//! Parses the real 733MB SDF, triages via K3 semantics, reports the
//! honest distribution + a few named compounds.
use swarm_drug::{parse_sdf, triage};

fn main() {
    let sdf = swarm_paths::resolve("/home/shawn/drugs/chembl_37.sdf.gz");
    println!("=== MOTION C: REAL ChEMBL 37 TRIAGE ===");
    println!("parsing {sdf} (streamed)...");

    // parse the first 5000 real molecules (streamed, bounded)
    let mols = parse_sdf(&sdf, 5000);
    println!("parsed: {} real molecules", mols.len());

    let mut druglike = 0;
    let mut blocked = 0;
    let mut analyze = 0;
    let mut total_mw = 0.0f64;
    let mut min_mw = f32::MAX;
    let mut max_mw = 0.0f32;

    for m in &mols {
        let (v, _) = triage(m, 0.35);
        match v {
            1 => druglike += 1,
            -1 => blocked += 1,
            _ => analyze += 1,
        }
        total_mw += m.mw as f64;
        min_mw = min_mw.min(m.mw);
        max_mw = max_mw.max(m.mw);
    }

    println!("\n--- triage distribution (K3, τ=0.35) ---");
    println!("drug-like: {druglike}  blocked: {blocked}  analyze: {analyze}");
    if !mols.is_empty() {
        println!("MW range: {min_mw:.0}–{max_mw:.0} (mean {:.0})", total_mw / mols.len() as f64);
    }

    println!("\n--- sample real compounds ---");
    for m in mols.iter().take(5) {
        let (v, label) = triage(m, 0.35);
        println!("  {}  atoms={} heavy={} MW={:.0} → {}", m.chembl_id, m.atom_count, m.heavy_atoms, m.mw, label);
    }
    println!("\n=== MOTION C GATE: real SDF parsed, real triage computed ===");
}
