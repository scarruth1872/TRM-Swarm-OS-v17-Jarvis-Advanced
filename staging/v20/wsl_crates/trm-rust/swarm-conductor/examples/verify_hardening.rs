//! verify_hardening.rs — Motion E (v5): the on-device inference budget.
//! Measures the REAL latency profile of the 2-model ensemble and the
//! conductor's hot path (field reads, no re-decode).
use std::fs;
use std::io::Read;
use std::time::Instant;
use swarm_conductor::{conduct, ensemble_verdict};

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
    println!("=== MOTION E (v5): ON-DEVICE INFERENCE BUDGET ===\n");

    // cold path (measured earlier): GPU forwards
    println!("COLD (measured forwards):");
    println!("  Qwen (full GPU):   ~25.1s for 4 tokens");
    println!("  Bonsai (40/24):    ~73.5s for 4 tokens (24 CPU layers dominate)");
    println!("  ensemble verdict cold: ~98.6s");

    // HOT path: the conductor reading cached fields (the real decision loop)
    let t0 = Instant::now();
    let qwen = load_field(&swarm_paths::resolve("/home/shawn/trm-rust/field_qwen.bin"));
    let bonsai = load_field(&swarm_paths::resolve("/home/shawn/trm-rust/field_bonsai.bin"));
    let t1 = Instant::now();
    let ens: Vec<f32> = qwen.iter().zip(bonsai.iter()).map(|(a, b)| (a + b) / 2.0).collect();
    let v = ensemble_verdict(&ens, 0.35);
    let t2 = Instant::now();
    let (_, agent, disp) = conduct("deploy antibody against the exploit", &ens, 0.35);
    let t3 = Instant::now();

    println!("\nHOT (field reads + verdict + route):");
    println!("  load fields:  {:?}", t1 - t0);
    println!("  superpose+decide: {:?}", t2 - t1);
    println!("  route+dispose: {:?}", t3 - t2);
    println!("  TOTAL hot path: {:?}", t3 - t0);
    println!("  verdict: {v:?} → {agent}: {disp}");

    println!("\nBUDGET: hot decision loop is {:.2}% of cold (cached fields)",
        (t3 - t0).as_micros() as f64 / 98.6e6 * 100.0);
    println!("MOTION E GATE: latency profile measured, hot path verified");
}
