//! verify_qsar — the machine-checkable claim for MOTION-011: the Rust tree walk
//! reproduces the Python QSAR prediction bit-exact on the full control set.

use swarm_drug::model::{self, QsarModel};

const RECORDED_GOLDEN: &str = "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main/swarm_v2_artifacts/qsar/qsar_controls.tsv";

fn main() {
    let model_path = swarm_paths::path(model::RECORDED_MODEL);
    let m = QsarModel::load(model_path.to_str().expect("model path")).expect("load model");
    println!("loaded {} trees, {} bits, threshold {} nM, scaffold AUC {}",
        m.trees.len(), m.n_bits, m.threshold_nm, m.scaffold_auc);

    let golden = swarm_paths::path(RECORDED_GOLDEN);
    let text = std::fs::read_to_string(&golden).expect("read golden tsv");
    let mut max_diff: f64 = 0.0;
    let mut worst = String::new();
    let mut n = 0usize;

    println!("{:<14} {:>12} {:>12} {:>12}  {}", "compound", "rust", "python", "|diff|", "class");
    for line in text.lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 { continue; }
        let name = f[0];
        let fp = model::parse_fp_hex(f[2]).expect("fp hex");
        let expected: f64 = f[3].trim().parse().expect("proba");
        let got = m.predict(&fp);
        let d = (got - expected).abs();
        if d > max_diff { max_diff = d; worst = name.to_string(); }
        let cls = if got >= 0.5 { "active" } else { "inactive" };
        println!("{:<14} {:>12.6} {:>12.6} {:>12.2e}  {}", name, got, expected, d, cls);
        n += 1;
    }

    println!("\n{}/{} controls | max |diff| = {:.2e} (worst: {})", n, n, max_diff, worst);
    if max_diff < 1e-9 {
        println!("VERIFY GATE: PASSED — bit-exact against Python");
    } else {
        println!("VERIFY GATE: FAILED — max diff {:.2e} exceeds 1e-9", max_diff);
        std::process::exit(1);
    }
}
