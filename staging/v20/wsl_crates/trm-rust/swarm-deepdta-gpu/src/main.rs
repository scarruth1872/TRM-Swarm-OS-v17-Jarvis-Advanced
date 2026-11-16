//! swarm-deepdta-gpu — GPU (wgpu) implementation of the DeepDTA ligand branch,
//! validated against the CPU reference in `swarm-deepdta`.
//!
//! Flow: load + sha256-gate the .bin (same gate as the reference), score the
//! protein branch ONCE on the CPU (it is constant for a fixed sequence), then
//! batch the ligand branch + MLP head on the GPU and compare every score to
//! the CPU reference path, reporting molecules/second for both.

mod gpu;

use std::time::Instant;
use swarm_deepdta::model::{DeepDtaModel, LIG_VOCAB, SMILEN};
use swarm_deepdta::{tokenize_ligand, tokenize_protein};

const REGISTERED_SHA256: &str = "77300fe6829ddd308657feeedfefdaabd0b9c1d9472ecad7bc369cf91230c7f2";

const CHANNEL: usize = 32;
const K: usize = 8;
const PRO_VOCAB: usize = 24;

/// SARS-CoV-2 main protease (nsp5), 306 aa — padded with 'dummy' to SEQLEN=2000
/// by the reference tokenizer. Any fixed sequence works; the branch is cached.
const MPRO: &str = "SGFRKMAFPSGKVEGCMVQVTCGTTTLNGLWLDDVVYCPRHVICTSEDMLNPNYEDLLIRKSNHNFLVQAGNVQLRVIGHSMQNCVLKLKVDTANPKTPKYKFVRIQPGQTFSVLACYNGSPSGVYQCAMRPNFTIKGSFLNGSCGSVGFNIDYDCVSFCYMHHMELPTGVHAGTDLEGNFYGPFVDRQTAQAAGTDTTITVNVLAWLYAAVINGDRWFLNRFTTTLNDFNLVAMKYNYEPLTQDHVDILGPLSAQTGIAVLDMCASLKELLQNGMNGRTILGSALLEDEFTPFDVVRQCSGVTFQ";

struct Args {
    model: String,
    smiles: String,
    batch: usize,
    limit: usize,
    threads: usize,
    single: usize,
    no_cpu: bool,
    sequence: Option<String>,
}

fn repo_root() -> std::path::PathBuf {
    // crate lives at <repo>/wsl_crates/trm-rust/swarm-deepdta-gpu
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..3 {
        p.pop();
    }
    p
}

fn parse_args() -> Args {
    let mut a = Args {
        model: String::new(),
        smiles: "C:/Users/shawn/AppData/Local/hermes/cache/scratch/chembl_smiles.txt".into(),
        batch: 1024,
        limit: 4096,
        threads: std::thread::available_parallelism().map(|v| v.get()).unwrap_or(8),
        single: 256,
        no_cpu: false,
        sequence: None,
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let key = argv[i].clone();
        match key.as_str() {
            "--model" => {
                i += 1;
                a.model = argv.get(i).cloned().unwrap_or_default();
            }
            "--smiles" => {
                i += 1;
                a.smiles = argv.get(i).cloned().unwrap_or_default();
            }
            "--batch" => {
                i += 1;
                a.batch = argv.get(i).and_then(|v| v.parse().ok()).unwrap_or(1024);
            }
            "--limit" => {
                i += 1;
                a.limit = argv.get(i).and_then(|v| v.parse().ok()).unwrap_or(4096);
            }
            "--threads" => {
                i += 1;
                a.threads = argv.get(i).and_then(|v| v.parse().ok()).unwrap_or(8);
            }
            "--single" => {
                i += 1;
                a.single = argv.get(i).and_then(|v| v.parse().ok()).unwrap_or(256);
            }
            "--no-cpu" => {
                a.no_cpu = true;
            }
            "--sequence" => {
                i += 1;
                a.sequence = argv.get(i).cloned();
            }
            "-h" | "--help" => {
                println!("--model <bin> --smiles <tsv> --batch <n> --limit <n> --threads <n>");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }
    if a.model.is_empty() {
        a.model = repo_root()
            .join("swarm_v2_artifacts/deepdta/deepdta_mpro_ft.bin")
            .to_string_lossy()
            .replace('\\', "/");
    }
    a
}

struct RawWeights {
    fc1b: Vec<f32>, fc1w: Vec<f32>, fc2b: Vec<f32>, fc2w: Vec<f32>,
    fc3b: Vec<f32>, fc3w: Vec<f32>, fc4b: Vec<f32>, fc4w: Vec<f32>,
    lig_c1b: Vec<f32>, lig_c1w: Vec<f32>, lig_c2b: Vec<f32>, lig_c2w: Vec<f32>,
    lig_c3b: Vec<f32>, lig_c3w: Vec<f32>, lig_emb: Vec<f32>,
}

/// Same sequential layout walk as `swarm-deepdta::model::load_weights`.
fn load_raw(bin: &[u8]) -> RawWeights {
    let f: Vec<f32> = bin
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    let mut o = 0usize;
    let mut take = |n: usize| {
        let s = o;
        o += n;
        f[s..o].to_vec()
    };
    let fc1b = take(1024);
    let fc1w = take(1024 * 192);
    let fc2b = take(1024);
    let fc2w = take(1024 * 1024);
    let fc3b = take(512);
    let fc3w = take(512 * 1024);
    let fc4b = take(1);
    let fc4w = take(512);
    let lig_c1b = take(CHANNEL);
    let lig_c1w = take(CHANNEL * 128 * K);
    let lig_c2b = take(CHANNEL * 2);
    let lig_c2w = take(CHANNEL * 2 * CHANNEL * K);
    let lig_c3b = take(CHANNEL * 3);
    let lig_c3w = take(CHANNEL * 3 * CHANNEL * 2 * K);
    let lig_emb = take(LIG_VOCAB * 128);
    // Remaining bytes are the protein branch (unused here: cached on the CPU).
    let pro_block = PRO_VOCAB * 128
        + (CHANNEL + CHANNEL * 128 * K)
        + (CHANNEL * 2 + CHANNEL * 2 * CHANNEL * K)
        + (CHANNEL * 3 + CHANNEL * 3 * CHANNEL * 2 * K);
    assert_eq!(f.len() - o, pro_block, "unexpected weight file length");
    RawWeights {
        fc1b, fc1w, fc2b, fc2w, fc3b, fc3w, fc4b, fc4w,
        lig_c1b, lig_c1w, lig_c2b, lig_c2w, lig_c3b, lig_c3w, lig_emb,
    }
}

/// [c][d][k] -> [k][d][c] so warps read contiguous weights.
fn transpose_conv(src: &[f32], in_c: usize, out_c: usize) -> Vec<f32> {
    let mut dst = vec![0.0f32; src.len()];
    for c in 0..out_c {
        for d in 0..in_c {
            for k in 0..K {
                dst[(k * in_c + d) * out_c + c] = src[(c * in_c + d) * K + k];
            }
        }
    }
    dst
}

/// [c][d] -> [d][c]
fn transpose_fc(src: &[f32], in_c: usize, out_c: usize) -> Vec<f32> {
    let mut dst = vec![0.0f32; src.len()];
    for c in 0..out_c {
        for d in 0..in_c {
            dst[d * out_c + c] = src[c * in_c + d];
        }
    }
    dst
}

fn main() {
    let a = parse_args();
    println!("# model      {}", a.model);

    // ---- load + gate + reference model -------------------------------------
    let bytes = std::fs::read(&a.model).expect("read model");
    let digest = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(&bytes))
    };
    if digest != REGISTERED_SHA256 {
        panic!("sha256 gate: {digest} != {REGISTERED_SHA256}");
    }
    println!("# sha256     ok {digest}");

    let model = DeepDtaModel::load(&a.model).expect("reference model load");
    // The sequence is a caller input: the kernel route accepts any protein, so the binary must
    // not hardcode Mpro. MPRO stays the default when none is given.
    let sequence: &str = a.sequence.as_deref().unwrap_or(MPRO);
    let pro_idx = tokenize_protein(sequence);
    let t = Instant::now();
    let prot = model.protein_branch(&pro_idx);
    println!(
        "# protein    {} aa, branch cached in {:.3}s (CPU, once)",
        sequence.len(),
        t.elapsed().as_secs_f64()
    );
    assert_eq!(prot.len(), 96);

    // ---- molecules ----------------------------------------------------------
    let text = std::fs::read_to_string(&a.smiles).expect("read smiles");
    let mut smiles: Vec<&str> = Vec::new();
    for line in text.lines() {
        let s = line.split('\t').next().unwrap_or("").trim();
        if !s.is_empty() {
            smiles.push(s);
        }
        if smiles.len() >= a.limit {
            break;
        }
    }
    let n = smiles.len();
    let toks: Vec<[usize; SMILEN]> = smiles
        .iter()
        .map(|s| {
            let v = tokenize_ligand(s);
            let mut arr = [19usize; SMILEN];
            arr.copy_from_slice(&v);
            arr
        })
        .collect();
    println!("# molecules  {n} (limit {})", a.limit);

    // ---- CPU reference ------------------------------------------------------
    // At library scale the CPU reference would take ~98 minutes for 2.9M molecules, so --no-cpu
    // scores a 4096-molecule prefix on the CPU for parity and leaves the GPU timing over all n.
    let cpu_n = if a.no_cpu { n.min(4096) } else { n };
    if a.no_cpu {
        println!("# parity     CPU reference limited to the first {cpu_n} molecules (--no-cpu)");
    }
    let mut cpu = vec![0.0f32; cpu_n];
    let nt = a.threads.max(1).min(cpu_n.max(1));
    let t = Instant::now();
    std::thread::scope(|s| {
        let mut rest = cpu.as_mut_slice();
        let mut off = 0usize;
        for th in 0..nt {
            let left = cpu_n - off;
            let take = if th == nt - 1 { left } else { (left / (nt - th)).max(1) };
            let (head, tail) = rest.split_at_mut(take);
            rest = tail;
            let toks = &toks[off..off + take];
            let branch = &prot;
            let m = &model;
            s.spawn(move || {
                for (j, tk) in toks.iter().enumerate() {
                    let mut idx = [0usize; SMILEN];
                    idx.copy_from_slice(&tk[..]);
                    head[j] = m.predict_with_protein(branch, &idx);
                }
            });
            off += take;
        }
    });
    let cpu_secs = t.elapsed().as_secs_f64();
    let cpu_rate = cpu_n as f64 / cpu_secs;

    // single-thread reference rate on a sample
    let ns = a.single.min(n);
    let t = Instant::now();
    let mut sink = 0.0f32;
    for row in toks.iter().take(ns) {
        let mut idx = [0usize; SMILEN];
        idx.copy_from_slice(&row[..]);
        sink += model.predict_with_protein(&prot, &idx);
    }
    let st_rate = ns as f64 / t.elapsed().as_secs_f64();
    println!("# cpu        1 thread {st_rate:.1} mol/s (n={ns}, sink={sink:.3})");

    // ---- GPU ---------------------------------------------------------------
    let raw = load_raw(&bytes);
    let gw = gpu::GpuWeights {
        wt1: transpose_conv(&raw.lig_c1w, 128, 32),
        wt2: transpose_conv(&raw.lig_c2w, 32, 64),
        wt3: transpose_conv(&raw.lig_c3w, 64, 96),
        b1: raw.lig_c1b.clone(),
        b2: raw.lig_c2b.clone(),
        b3: raw.lig_c3b.clone(),
        emb: raw.lig_emb.clone(),
        prot: prot.clone(),
        f1w: transpose_fc(&raw.fc1w, 192, 1024),
        f1b: raw.fc1b.clone(),
        f2w: transpose_fc(&raw.fc2w, 1024, 1024),
        f2b: raw.fc2b.clone(),
        f3w: transpose_fc(&raw.fc3w, 1024, 512),
        f3b: raw.fc3b.clone(),
        f4w: raw.fc4w.clone(),
        f4b: raw.fc4b.clone(),
    };
    let g = gpu::Gpu::new(&gw, a.batch).expect("gpu init");
    println!("# gpu        {} ({})", g.adapter_name, g.backend);

    // flatten tokens
    let mut flat = Vec::with_capacity(n * SMILEN);
    for t in &toks {
        flat.extend(t.iter().map(|&v| v as u32));
    }

    let mut gpu_out = vec![0.0f32; n];
    let batch = g.batch();
    if n > 0 {
        let w = n.min(batch);
        let _ = g.score(&flat[..w * SMILEN], w);
    }
    let t = Instant::now();
    let mut done = 0usize;
    while done < n {
        let take = (n - done).min(batch);
        let got = g.score(&flat[done * SMILEN..(done + take) * SMILEN], take);
        gpu_out[done..done + take].copy_from_slice(&got);
        done += take;
    }
    let gpu_secs = t.elapsed().as_secs_f64();
    let gpu_rate = n as f64 / gpu_secs;

    // ---- parity ------------------------------------------------------------
    let mut max_abs = 0.0f32;
    let mut max_rel = 0.0f32;
    let mut exact = 0usize;
    let mut bad = 0usize;
    for i in 0..cpu_n {
        let d = (gpu_out[i] - cpu[i]).abs();
        if d > max_abs {
            max_abs = d;
        }
        let r = d / cpu[i].abs().max(1e-6);
        if r > max_rel {
            max_rel = r;
        }
        if gpu_out[i].to_bits() == cpu[i].to_bits() {
            exact += 1;
        }
        if !(d <= 1e-4) {
            bad += 1;
        }
    }
    let (mut cmin, mut cmax, mut csum) = (f32::MAX, f32::MIN, 0.0f64);
    for v in &cpu {
        cmin = cmin.min(*v);
        cmax = cmax.max(*v);
        csum += *v as f64;
    }
    println!(
        "# scores     cpu min={cmin:.4} max={cmax:.4} mean={:.4} spread={:.4}",
        csum / cpu_n.max(1) as f64,
        cmax - cmin
    );
    for i in 0..3.min(n) {
        println!("# sample     [{i}] cpu={:.7} gpu={:.7}", cpu[i], gpu_out[i]);
    }
    println!(
        "# parity     max_abs={max_abs:.3e} max_rel={max_rel:.3e} bit_exact={exact}/{n} over_tol={bad}"
    );
    println!(
        "# throughput cpu({nt}t)={cpu_rate:.1} mol/s  gpu={gpu_rate:.1} mol/s  batch={batch}  speedup={:.2}x",
        gpu_rate / cpu_rate
    );
    println!(
        "{{\"n\":{n},\"batch\":{batch},\"cpu_threads\":{nt},\"cpu_mol_per_s\":{cpu_rate:.2},\"cpu_single_mol_per_s\":{st_rate:.2},\"gpu_mol_per_s\":{gpu_rate:.2},\"speedup\":{:.3},\"max_abs_diff\":{max_abs:.9e},\"max_rel_diff\":{max_rel:.9e},\"bit_exact\":{exact},\"over_tol\":{bad},\"gpu_secs\":{gpu_secs:.4},\"cpu_secs\":{cpu_secs:.4},\"adapter\":\"{}\",\"backend\":\"{}\"}}",
        gpu_rate / cpu_rate,
        g.adapter_name.replace('"', ""),
        g.backend
    );
}
