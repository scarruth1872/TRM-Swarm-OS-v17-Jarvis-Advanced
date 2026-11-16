//! Real expert gate: model-produced embeddings -> K3 trits -> SRAM CAM -> cosine re-rank.
//! Embeddings come from nomic-embed-text (768d) projected to 64d via Johnson-Lindenstrauss.
#[path = "../../src/time.rs"]
mod time;
#[path = "../../src/float_math.rs"]
mod float_math;
#[path = "../../src/expert_team_harnesses.rs"]
mod expert_team_harnesses;
#[path = "../../src/k3_logic.rs"]
mod k3_logic;
#[path = "../../src/sram_pattern_computer.rs"]
mod sram_pattern_computer;
#[path = "../../host_council_server/src/k3_ternary_sram.rs"]
mod k3_ternary_sram;
use k3_ternary_sram::K3Sram;

const D: usize = 64;
const NEXP: usize = 64;

fn trits(v: &[f32]) -> Vec<i8> {
    v.iter().map(|&x| if x > 0.02 { 1 } else if x < -0.02 { -1 } else { 0 }).collect()
}

fn cos(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn load_vectors(path: &str) -> Vec<f32> {
    let bytes = std::fs::read(path).expect("read embedding file");
    bytes.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

fn domain_of(eid: usize) -> &'static str {
    match eid / 10 {
        0 => "Security & Cryptography",
        1 => "Logic & Mathematical Proof",
        2 => "Code Engineering & Refactoring",
        3 => "Telemetry & Memory Sync",
        4 => "Synthesis & Chain-of-Thought",
        _ => "Autonomous Healing & Resilience",
    }
}

fn main() {
    let v = load_vectors("expert_emb_64.bin");
    assert_eq!(v.len(), (NEXP + 4) * D, "expected 68 x 64 f32");
    let mut sram = K3Sram::new();
    for e in 0..NEXP {
        let row = &v[e * D..e * D + D];
        sram.store_pattern_multi(&(e as u32).to_le_bytes(), &trits(row));
    }
    println!("experts {} resident {}", NEXP, sram.patterns_resident());
    let expect = [2usize, 1, 4, 0];
    let names = ["buffer overflow in c", "prove theorem by induction", "microkernel architecture", "audit for vulnerabilities"];
    let mut hits = 0;
    for qi in 0..4 {
        let base = (NEXP + qi) * D;
        let q = &v[base..base + D];
        // Two-stage: CAM prefilters on a 16-trit coarse prefix (wildcard the rest),
        // then cosine re-ranks only the surviving candidates.
        let mut mask = vec![0u8; D];
        for m in mask.iter_mut().take(16) { *m = 1; }
        let mut cand: Vec<usize> = Vec::new();
        sram.tcam_match_all(&trits(q), &mask, &mut cand);
        let cam = if cand.is_empty() { None } else { Some(cand.len()) };
        let pool: Vec<usize> = if cand.is_empty() { (0..NEXP).collect() } else { cand.clone() };
        let mut best = 0usize; let mut bs = -2.0f32;
        for &e in pool.iter() {
            let s = cos(q, &v[e * D..e * D + D]);
            if s > bs { bs = s; best = e; }
        }
        let got = best / 10;
        if got == expect[qi] { hits += 1; }
        println!("{:<28} cam={:?} best={:3} [{:<32}] cos={:.3} {}", names[qi], cam, best, domain_of(best), bs, if got == expect[qi] { "OK" } else { "MISS" });
    }
    println!("");
    println!("REAL-EMBEDDING GATE: {}/4 correct", hits);
}
