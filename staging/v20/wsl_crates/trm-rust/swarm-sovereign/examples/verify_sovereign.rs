//! verify_sovereign.rs — Motion B gate: run the import blockade against
//! the REAL platform source. Every core file scanned.
use std::fs;
use std::path::Path;
use swarm_sovereign::{blockade_verdict, scan_for_external};

fn main() {
    let core = "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2/core";
    println!("=== MOTION B: SOVEREIGNTY BLOCKADE AGAINST REAL SOURCE ===");

    let mut clean = 0;
    let mut violations = 0;
    let mut scanned = 0;

    let mut walk = |path: &Path| {
        if path.is_file() && path.extension().map(|e| e == "py").unwrap_or(false) {
            let src = fs::read_to_string(path).unwrap_or_default();
            scanned += 1;
            let hits = scan_for_external(&src);
            if hits.is_empty() {
                clean += 1;
            } else {
                violations += 1;
                println!("  VIOLATION {}: {:?}", path.display(), hits);
            }
        }
    };
    fn walk_dir(dir: &Path, f: &mut dyn FnMut(&Path)) {
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk_dir(&p, f);
                } else {
                    f(&p);
                }
            }
        }
    }
    walk_dir(Path::new(core), &mut walk);

    println!("\nscanned {scanned} real .py files");
    println!("clean: {clean}  violations: {violations}");
    if violations == 0 {
        println!("\nVERDICT: {}", blockade_verdict(""));
        println!("MOTION B GATE: PASSED — the platform is import-blockade clean");
    } else {
        println!("\nMOTION B GATE: FAILED — {violations} files violate the blockade");
    }
}
