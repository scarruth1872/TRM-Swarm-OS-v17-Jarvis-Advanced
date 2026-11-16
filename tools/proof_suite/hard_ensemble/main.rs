extern crate alloc;
mod float_math;
mod turbovec;
mod time;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use std::collections::HashMap;

fn expert_view(v: &[f32], e: usize) -> Vec<f32> {
    let mut o = v.to_vec();
    for (j, x) in o.iter_mut().enumerate() {
        let h = ((e as u64).wrapping_mul(1_000_003).wrapping_add((j as u64).wrapping_mul(7919)) % 1000) as f32 / 1000.0;
        *x *= 0.9 + 0.2 * h;
    }
    o
}
fn cosd(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let (mut d, mut na, mut nb) = (0f32, 0f32, 0f32);
    for i in 0..n { d += a[i]*b[i]; na += a[i]*a[i]; nb += b[i]*b[i]; }
    d / ((na.max(1e-12)).sqrt() * (nb.max(1e-12)).sqrt())
}
fn q158(v: &[f32]) -> Vec<f32> {
    let (pk, sc) = turbovec::TurboVec158Quantizer::quantize_158(v);
    turbovec::TurboVec158Quantizer::dequantize_158(&pk, v.len(), sc)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let txt = std::fs::read_to_string(&a[1]).unwrap_or_default();
    let lines: Vec<&str> = txt.lines().filter(|l| l.contains('\t')).collect();
    let nd = lines.len();
    let mut df: HashMap<String, usize> = HashMap::new();
    let mut toks: Vec<Vec<String>> = Vec::new();
    for l in &lines {
        let (_i, t) = l.split_once('\t').unwrap();
        let row: Vec<String> = t.split_whitespace().map(|s| s.to_string()).collect();
        let mut s = row.clone(); s.sort(); s.dedup();
        for w in &s { *df.entry(w.clone()).or_insert(0) += 1; }
        toks.push(row);
    }
    let cap = 1024usize;
    let mut idx: HashMap<String, usize> = HashMap::new();
    for (w, _) in df.iter() { if idx.len() >= cap { break; } idx.insert(w.clone(), idx.len()); }
    let dim = idx.len();
    let mut docs: Vec<Vec<f32>> = Vec::new();
    for row in &toks {
        let mut v = vec![0f32; dim];
        for w in row { if let Some(&c) = idx.get(w) {
            v[c] += (1.0 + (nd as f32 / (1.0 + *df.get(w).unwrap_or(&1) as f32)).ln()); } }
        let nm = v.iter().map(|x| x*x).sum::<f32>().sqrt().max(1e-12);
        for x in v.iter_mut() { *x /= nm; }
        docs.push(v);
    }
    println!("docs {} | dim {} | chance {:.3}%", nd, dim, 100.0/nd as f64);
    let ne = 14usize;
    let sample: Vec<usize> = (0..nd).step_by(3).collect();
    let mut docq: Vec<Vec<Vec<f32>>> = Vec::with_capacity(nd);
    for d in &docs { let mut per = Vec::new(); for e in 0..ne { per.push(q158(&expert_view(d, e))); } docq.push(per); }

    let (mut okf, mut okb, mut okc, mut okd) = (0usize, 0usize, 0usize, 0usize);
    for &i in &sample {
        let q = docs[i].clone();
        let qs = q158(&q);
        let mut rf: Vec<(f32, usize)> = (0..nd).map(|j| (cosd(&q, &docs[j]), j)).collect();
        rf.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if rf[0].1 == i { okf += 1; }
        let mut rb: Vec<(f32, usize)> = (0..nd).map(|j| (cosd(&qs, &docq[j][0]), j)).collect();
        rb.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if rb[0].1 == i { okb += 1; }
        let mut rc: Vec<(f32, usize)> = (0..nd).map(|j| {
            let mut s = 0f32;
            for e in 0..ne { s += cosd(&q158(&expert_view(&q, e)), &docq[j][e]); }
            (s / ne as f32, j)
        }).collect();
        rc.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if rc[0].1 == i { okc += 1; }
        let mut top: Vec<usize> = rc.iter().take(20).map(|x| x.1).collect();
        top.sort_by(|a, b| cosd(&q, &docs[*b]).partial_cmp(&cosd(&q, &docs[*a])).unwrap());
        if top[0] == i { okd += 1; }
    }
    let n = sample.len();
    let pc = |k: usize| 100.0 * k as f64 / n as f64;
    println!();
    println!("sample {} queries of {}, 400 clusters x 8 near-duplicate variants", n, nd);
    println!("  A fp32 K40 kept            recall@1 {:6.2}%  ({}/{})", pc(okf), okf, n);
    println!("  B single 1.58-bit          recall@1 {:6.2}%  ({}/{})", pc(okb), okb, n);
    println!("  C 14x 1.58-bit ensemble    recall@1 {:6.2}%  ({}/{})", pc(okc), okc, n);
    println!("  D ensemble + fp32 rerank20 recall@1 {:6.2}%  ({}/{})", pc(okd), okd, n);
    println!();
    println!("TARGET 99.9%:  A={}  B={}  C={}  D={}",
        if pc(okf)>=99.9 {"HIT"} else {"miss"}, if pc(okb)>=99.9 {"HIT"} else {"miss"},
        if pc(okc)>=99.9 {"HIT"} else {"miss"}, if pc(okd)>=99.9 {"HIT"} else {"miss"});
}
