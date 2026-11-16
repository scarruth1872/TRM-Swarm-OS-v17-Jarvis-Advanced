//! swarm-drug — Motion C: the drug-triage act-arm (Bridge/Flow real work).
//!
//! Reads the REAL ChEMBL 37 SDF (733MB, /home/shawn/drugs/), extracts
//! molecule properties (atoms, heavy atoms, MW, chembl_id), and triages
//! candidates through the K3 collapse — the voltage-native decision.
//!
//! Honest scope: a real parser + real K3 triage on real data. No template,
//! no fake molecules. The SDF parse is streamed (gzip, block by block).

use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub mod model;

/// One parsed molecule from the real SDF.
#[derive(Debug, Clone)]
pub struct Molecule {
    pub chembl_id: String,
    pub atom_count: u32,
    pub heavy_atoms: u32,
    /// rough molecular weight from the SDF's M  ISO line (real data)
    pub mw: f32,
}

/// Parse the real ChEMBL 37 SDF, returning up to `limit` molecules.
/// Streams the gzip file — never loads the whole 733MB.
pub fn parse_sdf(path: &str, limit: usize) -> Vec<Molecule> {
    let f = File::open(path).expect("sdf open");
    let gz = GzDecoder::new(f);
    let reader = BufReader::new(gz);
    let mut out = Vec::new();
    let mut block: Vec<String> = Vec::new();
    let mut in_mol = false;

    for line in reader.lines().map_while(Result::ok) {
        if line == "$$$$" {
            if let Some(m) = parse_block(&block) {
                out.push(m);
                if out.len() >= limit {
                    break;
                }
            }
            block.clear();
            in_mol = false;
            continue;
        }
        if !in_mol && !line.trim().is_empty() && !line.starts_with('>') {
            in_mol = true;
        }
        block.push(line);
    }
    // last block
    if out.len() < limit {
        if let Some(m) = parse_block(&block) {
            out.push(m);
        }
    }
    out
}

fn parse_block(lines: &[String]) -> Option<Molecule> {
    // name line is lines[0]; counts line is lines[3] (V2000): "aa bb cc dd ee ff gg hh"
    let mut chembl_id = String::new();
    let mut atom_count = 0u32;
    let mut heavy = 0u32;
    let mut mw = 0.0f32;
    let mut counts_line: Option<&str> = None;

    for (i, l) in lines.iter().enumerate() {
        if l.starts_with("> <chembl_id>") {
            if let Some(next) = lines.get(i + 1) {
                chembl_id = next.trim().to_string();
            }
        }
        if i == 3 && l.len() >= 6 {
            counts_line = Some(l);
        }
        if l.starts_with("M  CHG") {
            // skip charge lines in the atom block; not needed for counts
        }
    }

    if let Some(c) = counts_line {
        let parts: Vec<&str> = c.split_whitespace().collect();
        if parts.len() >= 2 {
            atom_count = parts[0].parse().unwrap_or(0);
            // heavy atoms = atom block count minus hydrogens (not directly
            // in counts line; approximate from the atom block lines)
        }
    }

    // count heavy atoms from the atom block (V2000: rows after counts line)
    // atoms are lines[4..4+atom_count]; heavy = non-H first token
    let start = 4usize;
    for j in start..(start + atom_count as usize).min(lines.len()) {
        let l = lines[j].trim();
        let elem = l.split_whitespace().nth(3).unwrap_or("");
        if !elem.is_empty() && elem != "H" {
            heavy += 1;
        }
        // MW: the element symbol is at index 3; atomic weight approx from symbol
        mw += atomic_weight(elem);
    }

    if chembl_id.is_empty() {
        chembl_id = format!("UNKNOWN_{}", lines.first().unwrap_or(&String::new()).trim());
    }

    Some(Molecule { chembl_id, atom_count, heavy_atoms: heavy, mw })
}

fn atomic_weight(elem: &str) -> f32 {
    match elem {
        "C" => 12.011, "N" => 14.007, "O" => 15.999, "S" => 32.06,
        "P" => 30.974, "F" => 18.998, "Cl" => 35.45, "Br" => 79.904,
        "I" => 126.904, "Na" => 22.99, "K" => 39.098, "Ca" => 40.078,
        _ => 0.0,
    }
}

/// K3 triage: collapse a molecule's properties into a verdict.
/// TRUE = drug-like (pass), FALSE = block (too heavy/few atoms), UNKNOWN = analyze.
pub fn triage(m: &Molecule, tau: f32) -> (i8, String) {
    // Lipinski-ish signals: MW 150-500, heavy atoms 15-40
    let mw_ok = if m.mw >= 150.0 && m.mw <= 500.0 { 2.0 } else if m.mw > 0.0 { -2.0 } else { 0.0 };
    let ha_ok = if m.heavy_atoms >= 15 && m.heavy_atoms <= 40 { 2.0 } else if m.heavy_atoms > 0 { -1.0 } else { 0.0 };
    let atoms_ok = if m.atom_count >= 20 && m.atom_count <= 80 { 1.0 } else { -1.0 };
    // collapse: majority of signals
    let sum = mw_ok + ha_ok + atoms_ok;
    let verdict = if sum >= 2.0 {
        1
    } else if sum <= -2.0 {
        -1
    } else {
        0
    };
    let label = match verdict {
        1 => "drug-like",
        -1 => "blocked",
        _ => "analyze",
    };
    (verdict, format!("{} | MW {:.1} heavy {}", label, m.mw, m.heavy_atoms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_known_count_line() {
        let lines = vec![
            "MOL1".to_string(),
            "  TEST".to_string(),
            "".to_string(),
            "  8  8  0  0  0  0  0  0  0  0999 V2000".to_string(),
            "  1.0  2.0  3.0 C  0  0  0  0  0  0  0  0  0  0  0  0".to_string(),
            "  1.0  2.0  3.0 C  0  0  0  0  0  0  0  0  0  0  0  0".to_string(),
            "  1.0  2.0  3.0 N  0  0  0  0  0  0  0  0  0  0  0  0".to_string(),
        ];
        let m = parse_block(&lines).unwrap();
        assert_eq!(m.atom_count, 8);
        assert_eq!(m.heavy_atoms, 3);
        assert!(m.mw > 30.0);
    }

    #[test]
    fn triage_druglike() {
        let m = Molecule { chembl_id: "CHEMBL1".into(), atom_count: 40, heavy_atoms: 25, mw: 350.0 };
        assert_eq!(triage(&m, 0.35).0, 1);
    }

    #[test]
    fn triage_blocked_too_heavy() {
        let m = Molecule { chembl_id: "CHEMBL2".into(), atom_count: 100, heavy_atoms: 70, mw: 1200.0 };
        assert_eq!(triage(&m, 0.35).0, -1);
    }
}
