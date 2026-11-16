//! Score a docked pose with the crate's own engine, so a redocking study can report the platform's
//! affinity next to Vina's rather than only quoting Vina's number.
//!
//! Input is a two-block text file (no JSON dependency: the crate has none, and adding a parser to the
//! crate for one example would be surface without a consumer):
//!
//! ```text
//! SITE
//! C 12.500 24.100 15.300
//! N 14.120 23.850 14.900
//! LIGAND 3
//! O 14.200 23.900 17.500
//! ```
//!
//! Usage: `score_pose <pose.txt>`. Prints one line of JSON so the driver can parse it, and exits
//! non-zero on a malformed file or an element the engine does not model -- an unknown atom type is
//! never silently coerced to carbon.

use std::env;
use std::fs;
use std::process;

use swarm_docking::{dock_conformer, Atom, AtomType, BindingSite, LigandConformer, Vec3};

fn atom_type(token: &str) -> Option<AtomType> {
    match token.to_ascii_uppercase().as_str() {
        "C" => Some(AtomType::Carbon),
        "N" => Some(AtomType::Nitrogen),
        "O" => Some(AtomType::Oxygen),
        "S" => Some(AtomType::Sulfur),
        "CL" => Some(AtomType::Chlorine),
        "F" => Some(AtomType::Fluorine),
        "ZN" => Some(AtomType::Zinc),
        "H" => Some(AtomType::Hydrogen),
        _ => None,
    }
}

fn parse_atom(line: &str) -> Result<Atom, String> {
    let f: Vec<&str> = line.split_whitespace().collect();
    if f.len() != 4 {
        return Err(format!("expected `<element> <x> <y> <z>`, got {line:?}"));
    }
    let t = atom_type(f[0]).ok_or_else(|| format!("unmodelled element {:?}", f[0]))?;
    let c: Result<Vec<f32>, _> = f[1..].iter().map(|s| s.parse::<f32>()).collect();
    let c = c.map_err(|_| format!("unparseable coordinate in {line:?}"))?;
    Ok(Atom { pos: Vec3::new(c[0], c[1], c[2]), atom_type: t })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: score_pose <pose.txt>");
        process::exit(2);
    }
    let text = fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {e}", args[1]);
        process::exit(2);
    });

    let mut site: Vec<Atom> = Vec::new();
    let mut ligand: Vec<Atom> = Vec::new();
    let mut nrot: Option<u32> = None;
    let mut block = "";
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let head = line.split_whitespace().next().unwrap_or("");
        if head.eq_ignore_ascii_case("SITE") {
            block = "site";
            continue;
        }
        if head.eq_ignore_ascii_case("LIGAND") {
            block = "ligand";
            let n = line.split_whitespace().nth(1).unwrap_or("0");
            nrot = Some(n.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("LIGAND rotatable-bond count {n:?} is not an integer");
                process::exit(2);
            }));
            continue;
        }
        let atom = parse_atom(line).unwrap_or_else(|e| {
            eprintln!("{}: {e}", args[1]);
            process::exit(2);
        });
        match block {
            "site" => site.push(atom),
            "ligand" => ligand.push(atom),
            _ => {
                eprintln!("atom before any SITE/LIGAND header");
                process::exit(2);
            }
        }
    }
    let nrot = nrot.unwrap_or_else(|| {
        eprintln!("no LIGAND header");
        process::exit(2);
    });

    let s = BindingSite {
        pdb_id: String::new(),
        target_name: String::new(),
        atoms: site.clone(),
    };
    let l = LigandConformer {
        name: String::new(),
        atoms: ligand.clone(),
        rotatable_bonds: nrot,
    };
    let r = dock_conformer(&s, &l);
    println!(
        "{{\"delta_g\": {:.6}, \"ki_nanomolar\": {:.6}, \"ligand_efficiency\": {:.6}, \
         \"hbonds\": {}, \"clashes\": {}, \"site_atoms\": {}, \"ligand_atoms\": {}, \"rotatable\": {}}}",
        r.delta_g,
        r.ki_nanomolar,
        r.ligand_efficiency,
        r.hbond_count,
        r.steric_clashes,
        site.len(),
        ligand.len(),
        nrot
    );
}
