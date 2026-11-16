//! verify_conductor.rs — Motion B (v5) gate: the ensemble-driven mesh
//! on the REAL 2-model voltage field.
use std::fs;
use std::io::Read;
use swarm_conductor::{conduct, ensemble_verdict, EnsembleVerdict};

fn load_field(path: &str) -> Vec<f32> {
    let mut f = fs::File::open(path).expect("field");
    let mut nbuf = [0u8; 4];
    f.read_exact(&mut nbuf).unwrap();
    let n = i32::from_le_bytes(nbuf) as usize;
    let mut field = vec![0f32; n];
    unsafe {
        let ptr = field.as_mut_ptr() as *mut u8;
        f.read_exact(std::slice::from_raw_parts_mut(ptr, n * 4)).unwrap();
    }
    field
}

fn main() {
    println!("=== MOTION B (v5): ENSEMBLE-DRIVEN MESH — REAL FIELDS ===\n");

    // the REAL superposition from Motion D (v4)
    let qwen = load_field(&swarm_paths::resolve("/home/shawn/trm-rust/field_qwen.bin"));
    let bonsai = load_field(&swarm_paths::resolve("/home/shawn/trm-rust/field_bonsai.bin"));
    let ens: Vec<f32> = qwen.iter().zip(bonsai.iter()).map(|(a, b)| (a + b) / 2.0).collect();
    println!(
        "OBSERVE: ensemble {}d (Qwen |v|={:.1} + Bonsai |v|={:.1} superposition)",
        ens.len(),
        qwen.iter().map(|v| v * v).sum::<f32>().sqrt(),
        bonsai.iter().map(|v| v * v).sum::<f32>().sqrt()
    );

    let verdict = ensemble_verdict(&ens, 0.35);
    println!("DECIDE: ensemble K3 verdict = {verdict:?}\n");
    let _ = EnsembleVerdict::Act;

    // ROUTE + DISPOSE: real tasks through the mesh under the ensemble verdict
    let tasks = [
        "deploy antibody against the exploit",
        "write the compiler fix",
        "prove the theorem in Coq",
        "triage the drug binding affinity",
        "audit the fda trial compliance",
        "calibrate the telescope spectra",
        "render the dashboard visualization",
    ];
    println!("ROUTE + DISPOSE (under the ensemble verdict):");
    for t in tasks {
        let (v, agent, d) = conduct(t, &ens, 0.35);
        println!("  [{v:?}] {t}");
        println!("         → {d}");
    }
    println!("\n=== MOTION B GATE: ensemble field decided, mesh routed, disposition set ===");
}
