//! moe_router - sparse top-k MoE routing over real vectors, measured on device.
//!
//! Absorbs the *mathematical essence* of sparse mixture-of-experts (Switch/DeepSeek style gating):
//!   gate = softmax(x @ W_g)          -> per-expert probability
//!   top-k selection                  -> the model actually computes only k of E experts
//!   capacity factor                  -> bounded tokens per expert, overflow is dropped and counted
//!   aux load-balance loss            -> E * sum_e (f_e * p_e), Switch Transformer form
//!
//! Everything is local: no HTTP, no cloud, no pretrained download. Expert weights are K3-style
//! ternary {-1,0,+1}; the gate is f32. The point is not to look like a paper, it is to answer two
//! questions with numbers:
//!   1. does top-k routing preserve the dense mixture's output on REAL inputs, or does it break it?
//!   2. what does it cost - load balance, dropped tokens, and wall-clock, versus computing every expert?
//!
//! Input: the real memory fabric (f16 .npy, 768-dim text embeddings). Fallback: synthetic only when
//! explicitly requested with --synthetic, which is reported in the output so it can never be mistaken
//! for a real measurement.

use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::time::Instant;

const DIM: usize = 768;
const EXPERTS: usize = 8;
const HIDDEN: usize = 64;

// ---------------------------------------------------------------- deterministic weights
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self { Rng(seed | 1) }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.0 = x;
        x
    }
    fn f32(&mut self) -> f32 { (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32 }
    /// ternary draw biased toward zero, like a sparse K3 weight matrix
    fn ternary(&mut self) -> f32 {
        match self.next_u64() % 100 {
            0..=39 => 1.0,
            40..=79 => -1.0,
            _ => 0.0,
        }
    }
}

// ---------------------------------------------------------------- .npy (f16, 2-D) reader
struct Npy { rows: usize, cols: usize, data: Vec<f32> }

fn f16_to_f32(h: u16) -> f32 {
    let sign = ((h >> 15) & 1) as u32;
    let exp = ((h >> 10) & 0x1f) as u32;
    let frac = (h & 0x3ff) as u32;
    let bits = if exp == 0 {
        if frac == 0 { sign << 31 }
        else {
            // subnormal: normalise
            let mut e = -1i32;
            let mut f = frac;
            while f & 0x400 == 0 { f <<= 1; e -= 1; }
            f &= 0x3ff;
            (sign << 31) | (((127 - 15 + e + 1) as u32) << 23) | (f << 13)
        }
    } else if exp == 0x1f {
        (sign << 31) | (0xff << 23) | (frac << 13)
    } else {
        (sign << 31) | ((exp + 127 - 15) << 23) | (frac << 13)
    };
    f32::from_bits(bits)
}

fn load_npy(path: &str, max_rows: usize) -> std::io::Result<Npy> {
    let mut f = File::open(path)?;
    let mut magic = [0u8; 8];
    f.read_exact(&mut magic)?;
    if &magic[0..6] != b"\x93NUMPY" { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "not npy")); }
    let major = magic[6];
    let hlen = if major == 1 {
        let mut b = [0u8; 2]; f.read_exact(&mut b)?; u16::from_le_bytes(b) as usize
    } else {
        let mut b = [0u8; 4]; f.read_exact(&mut b)?; u32::from_le_bytes(b) as usize
    };
    let mut header = vec![0u8; hlen];
    f.read_exact(&mut header)?;
    let h = String::from_utf8_lossy(&header).to_string();
    if !h.contains("<f2") { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("unsupported dtype: {h}"))); }
    let shape = h.split("'shape':").nth(1).and_then(|s| s.split(')').next()).unwrap_or("");
    let nums: Vec<usize> = shape.split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty()).filter_map(|s| s.parse().ok()).collect();
    if nums.len() < 2 { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad shape")); }
    let (rows, cols) = (nums[0], nums[1]);
    let take: usize = rows.min(max_rows);
    let mut raw = vec![0u8; take * cols * 2];
    let pos = f.stream_position()?;
    f.seek(SeekFrom::Start(pos))?;
    f.read_exact(&mut raw)?;
    let data: Vec<f32> = raw.chunks_exact(2)
        .map(|c| f16_to_f32(u16::from_le_bytes([c[0], c[1]])))
        .collect();
    Ok(Npy { rows: take, cols, data })
}

fn synthetic(rows: usize) -> Npy {
    let mut rng = Rng::new(0x5EED_1234);
    let mut data = vec![0f32; rows * DIM];
    for v in data.iter_mut() { *v = rng.f32() * 2.0 - 1.0; }
    Npy { rows, cols: DIM, data }
}

// ---------------------------------------------------------------- experts
struct Experts { w: Vec<f32>, hidden: usize, dim: usize }

impl Experts {
    fn new(rng: &mut Rng) -> Self {
        let mut w = vec![0f32; EXPERTS * HIDDEN * DIM];
        for v in w.iter_mut() { *v = rng.ternary(); }
        Experts { w, hidden: HIDDEN, dim: DIM }
    }
    #[inline]
    fn apply(&self, e: usize, x: &[f32], out: &mut [f32]) {
        let base = e * self.hidden * self.dim;
        for h in 0..self.hidden {
            let row = base + h * self.dim;
            let mut acc = 0f32;
            for d in 0..self.dim { acc += self.w[row + d] * x[d]; }
            out[h] = acc;
        }
    }
}

// ---------------------------------------------------------------- router
struct Router { gate: Vec<f32> } // E x DIM

/// k-means over the same vectors the router will see. A trained MoE gate converges to exactly this
/// structure - cluster assignment - so this is the honest "what routing can do" condition, as opposed
/// to random init, which is "what routing does before anyone trains it".
fn kmeans(data: &[f32], rows: usize, cols: usize, k: usize, iters: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = Rng::new(seed);
    let mut cent: Vec<Vec<f32>> = (0..k).map(|_| {
        let r = (rng.next_u64() as usize) % rows;
        data[r * cols..r * cols + cols].to_vec()
    }).collect();
    let mut assign = vec![0usize; rows];
    for _ in 0..iters {
        for r in 0..rows {
            let x = &data[r * cols..r * cols + cols];
            let mut best = 0usize; let mut bd = f32::INFINITY;
            for (ci, c) in cent.iter().enumerate() {
                let mut d = 0f32;
                for j in 0..cols { let t = x[j] - c[j]; d += t * t; }
                if d < bd { bd = d; best = ci; }
            }
            assign[r] = best;
        }
        for ci in 0..k {
            let mut sum = vec![0f32; cols]; let mut n = 0usize;
            for r in 0..rows {
                if assign[r] == ci {
                    let x = &data[r * cols..r * cols + cols];
                    for j in 0..cols { sum[j] += x[j]; }
                    n += 1;
                }
            }
            if n > 0 {
                for j in 0..cols { cent[ci][j] = sum[j] / n as f32; }
            }
        }
    }
    cent
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let m = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut e: Vec<f32> = logits.iter().map(|v| (v - m).exp()).collect();
    let s: f32 = e.iter().sum();
    for v in e.iter_mut() { *v /= s; }
    e
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let limit: usize = args.iter().position(|a| a == "--limit")
        .and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(30_000);
    let k: usize = args.iter().position(|a| a == "--k")
        .and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(2);
    let capacity: f32 = args.iter().position(|a| a == "--capacity")
        .and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(1.25);
    let synth = args.iter().any(|a| a == "--synthetic");
    let fabric = "F:/Development sites/TRM-Swarm-OS-v2/swarm_v2_global_memory/fabric/fabric_vecs.npy";

    let np = if synth {
        println!("DATA: synthetic (--synthetic) - NOT a real measurement");
        synthetic(limit)
    } else {
        match load_npy(fabric, limit) {
            Ok(n) => { println!("DATA: real fabric {} rows x {} dims", n.rows, n.cols); n }
            Err(e) => {
                eprintln!("FAIL-CLOSED: cannot read fabric ({e}); pass --synthetic only for a smoke test");
                std::process::exit(2);
            }
        }
    };
    let n = np.rows.min(limit);
    let mut rng = Rng::new(0xC0FFEE_2026);
    let experts = Experts::new(&mut rng);
    let gate_mode: String = args.iter().position(|a| a == "--gate")
        .and_then(|i| args.get(i + 1)).cloned().unwrap_or_else(|| "random".to_string());
    let mut gate = vec![0f32; EXPERTS * DIM];
    let mut centroids: Vec<Vec<f32>> = Vec::new();
    if gate_mode == "kmeans" {
        centroids = kmeans(&np.data, n.min(2000), DIM, EXPERTS, 12, 0xA11CE);
        println!("GATE: kmeans ({} centroids fitted on {} real vectors)", EXPERTS, n.min(2000));
    } else {
        for v in gate.iter_mut() { *v = (rng.f32() * 2.0 - 1.0) * 0.03; }
        println!("GATE: random f32 init (untrained - the control condition)");
    }
    let router = Router { gate };

    let mut usage = vec![0usize; EXPERTS];
    let mut prob_sum = vec![0f64; EXPERTS];
    let mut dropped = 0usize;
    let mut cos_sum = 0f64;
    let mut cos_min = 1f64;
    let mut gate_entropy = 0f64;
    let mut moe_out = vec![0f32; HIDDEN];
    let mut dense_out = vec![0f32; HIDDEN];
    let mut top_out = vec![0f32; HIDDEN];
    let mut expert_out = vec![0f32; HIDDEN]

    ;
    let t0 = Instant::now();
    for row in 0..n {
        let width: usize = np.cols.min(DIM);
        let x = &np.data[row * np.cols..row * np.cols + width];
        // gate
        let mut logits = vec![0f32; EXPERTS];
        if gate_mode == "kmeans" {
            // negative squared distance / temperature - the shape a trained gate takes
            // cosine to centroid x temperature. Raw squared distance over 768 dims barely separates
            // clusters (measured: gate entropy pinned at ln(E) = uniform), which is exactly the kind of
            // silent degeneracy this harness exists to catch.
            for (e, c) in centroids.iter().enumerate() {
                let (mut dot, mut nx, mut nc) = (0f32, 0f32, 0f32);
                for j in 0..DIM { dot += x[j] * c[j]; nx += x[j] * x[j]; nc += c[j] * c[j]; }
                let denom = (nx.sqrt() * nc.sqrt()).max(1e-9);
                logits[e] = (dot / denom) * 25.0;
            }
        } else {
            for e in 0..EXPERTS {
                let base = e * DIM;
                let mut acc = 0f32;
                for d in 0..DIM { acc += router.gate[base + d] * x[d]; }
                logits[e] = acc;
            }
        }
        let probs = softmax(&logits);
        for e in 0..EXPERTS { prob_sum[e] += probs[e] as f64; }
        let ent: f64 = -probs.iter().map(|p| (*p as f64) * ((*p as f64) + 1e-12).ln()).sum::<f64>();
        gate_entropy += ent;

        // top-k selection
        let mut idx: Vec<usize> = (0..EXPERTS).collect();
        idx.sort_by(|a, b| probs[*b].partial_cmp(&probs[*a]).unwrap());
        let chosen = &idx[..k.min(EXPERTS)];
        for e in chosen { usage[*e] += 1; }

        // capacity overflow is measured AFTER the run (see below). Counting it here against cumulative
        // usage was wrong: once an expert passed its cap every later row looked "over", which inflated
        // the rate to 54-82% regardless of routing quality.

        // top-k output: weighted sum of the chosen experts only
        top_out.iter_mut().for_each(|v| *v = 0.0);
        for e in chosen {
            experts.apply(*e, x, &mut expert_out);
            let w = probs[*e];
            for h in 0..HIDDEN { top_out[h] += w * expert_out[h]; }
        }
        // dense reference: every expert, same weights
        dense_out.iter_mut().for_each(|v| *v = 0.0);
        for e in 0..EXPERTS {
            experts.apply(e, x, &mut expert_out);
            let w = probs[e];
            for h in 0..HIDDEN { dense_out[h] += w * expert_out[h]; }
        }
        // cosine agreement between sparse and dense output
        let (mut dot, mut na, mut nb) = (0f64, 0f64, 0f64);
        for h in 0..HIDDEN {
            dot += (top_out[h] as f64) * (dense_out[h] as f64);
            na += (top_out[h] as f64).powi(2);
            nb += (dense_out[h] as f64).powi(2);
        }
        let cos = if na > 0.0 && nb > 0.0 { dot / (na.sqrt() * nb.sqrt()) } else { 0.0 };
        cos_sum += cos;
        if cos < cos_min { cos_min = cos; }
        moe_out.copy_from_slice(&top_out);
    }
    let elapsed = t0.elapsed().as_secs_f64();

    // standard capacity semantics: each expert may keep ceil(capacity * n / E) of its assignments
    let cap = ((capacity * n as f32 / EXPERTS as f32).ceil()) as usize;
    let kept: usize = usage.iter().map(|u| (*u).min(cap)).sum();
    let total_assign: usize = usage.iter().sum();
    dropped = total_assign.saturating_sub(kept);
    let shares: Vec<f64> = usage.iter().map(|u| *u as f64 / total_assign as f64).collect();
    let (mx, mn) = (shares.iter().cloned().fold(0.0, f64::max), shares.iter().cloned().fold(1.0, f64::min));
    let p: Vec<f64> = prob_sum.iter().map(|s| s / n as f64).collect();
    let aux: f64 = EXPERTS as f64 * shares.iter().zip(p.iter()).map(|(f, q)| f * q).sum::<f64>();
    let ideal_aux = 1.0; // uniform routing gives E * sum(1/E * 1/E) = 1.0
    let flops_dense = (EXPERTS * HIDDEN * DIM) as f64;
    let flops_topk = (k * HIDDEN * DIM) as f64;

    println!("\n== sparse MoE top-{} over {} experts, capacity {:.2}", k, EXPERTS, capacity);
    println!("vectors routed        : {}", n);
    println!("wall clock            : {:.3} s  ({:.0} vec/s)", elapsed, n as f64 / elapsed);
    println!("expert share (min/max): {:.4} / {:.4}   (ideal {:.4})", mn, mx, 1.0 / EXPERTS as f64);
    println!("max/min imbalance     : {:.2}x", if mn > 0.0 { mx / mn } else { f64::INFINITY });
    println!("gate entropy (mean)   : {:.3} nats  (max {:.3})", gate_entropy / n as f64, (EXPERTS as f64).ln());
    println!("aux load-balance loss : {:.4}          (uniform = {:.4})", aux, ideal_aux);
    println!("dropped assignments   : {} / {}  ({:.2}%)   cap {}/expert", dropped, total_assign,
             100.0 * dropped as f64 / total_assign as f64, cap);
    println!("sparse vs dense cos   : mean {:.4}  min {:.4}  <- does top-k preserve the mixture?", cos_sum / n as f64, cos_min);
    println!("expert FLOPs per token: dense {:.0} vs top-k {:.0}  ({:.0}% saved)", flops_dense, flops_topk, 100.0 * (1.0 - flops_topk / flops_dense));
    println!("output[0..4]          : {:.4} {:.4} {:.4} {:.4}", moe_out[0], moe_out[1], moe_out[2], moe_out[3]);
}
