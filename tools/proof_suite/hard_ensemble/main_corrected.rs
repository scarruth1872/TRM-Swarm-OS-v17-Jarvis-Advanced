//! CORRECTED ensemble benchmark.
//! Fixes two validity defects in the first attempt:
//!   1. dim was CAPPED at 1024 of a 6600-token vocabulary -> 84.5% of tokens
//!      dropped, so documents lost their distinguishing tokens and fp32 itself
//!      could only reach 87.7%.
//!   2. the kept tokens came from HashMap iteration order -> arbitrary and not
//!      reproducible. Now BTreeMap: deterministic, and no cap.
extern crate alloc;
mod float_math;
mod turbovec;
mod time;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use std::collections::BTreeMap;

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
    let mut toks: Vec<Vec<String>> = Vec::new();
    let mut dfc: BTreeMap<String, usize> = BTreeMap::new();
    for l in &lines {
        let (_i, t) = l.split_once('\t').unwrap();
        let row: Vec<String> = t.split_whitespace().map(|s| s.to_string()).collect();
        let mut s = row.clone(); s.sort(); s.dedup();
        for w in &s { *dfc.entry(w.clone()).or_insert(0) += 1; }
        toks.push(row);
    }
    // DETERMINISTIC index: sorted vocabulary, NO cap
    let mut idx: BTreeMap<String, usize> = BTreeMap::new();
    for (w, _) in dfc.iter() { idx.insert(w.clone(), idx.len()); }
    let dim = idx.len();
    let mut docs: Vec<Vec<f32>> = Vec::new();
    for row in &toks {
        let mut v = vec![0f32; dim];
        for w in row { if let Some(&c) = idx.get(w) {
            v[c] += (1.0 + (nd as f32 / (1.0 + *dfc.get(w).unwrap_or(&1) as f32)).ln()); } }
        let nm = v.iter().map(|x| x*x).sum::<f32>().sqrt().max(1e-12);
        for x in v.iter_mut() { *x /= nm; }
        docs.push(v);
    }
    println!("docs {} | dim {} (FULL vocab, deterministic) | chance {:.3}%", nd, dim, 100.0/nd as f64);
    let ne = 14usize;
    let mut docq: Vec<Vec<Vec<f32>>> = Vec::with_capacity(nd);
    for d in &docs { let mut per = Vec::new(); for e in 0..ne { per.push(q158(&expert_view(d, e))); } docq.push(per); }
    let (mut okf, mut okb, mut okc, mut okd, mut oke) = (0usize,usize::MAX,0usize,0usize,0usize);
    let mut oke2 = (0usize, usize::MAX, 0usize);
    let mut oke3 = (0usize, usize::MAX, 0usize);
    let mut okf2 = 0usize; let mut okf3 = 0usize;
    let mut okb2 = 0usize; let mut okb3 = 0usize;
    let mut okc2 = 0usize; let mut okc3 = 0usize;
    for i in 0..nd {
        let q = docs[i].clone();
        let qs = q158(&q);
        let mut rf: Vec<(f32, usize)> = (0..nd).map(|j| (cosd(&q, &docs[j]), j)).collect();
        rf.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if rf[0].1 == i { okf += 1; }
        if rf.iter().take(3).any(|x| x.1 == i) { okf2 += 1; }
        if rf.iter().take(40).any(|x| x.1 == i) { okf3 += 1; }
        let mut rb: Vec<(f32, usize)> = (0..nd).map(|j| (cosd(&qs, &docq[j][0]), j)).collect();
        rb.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if rb[0].1 == i { okb += 1; }
        if rb.iter().take(3).any(|x| x.1 == i) { okb2 += 1; }
        if rb.iter().take(40).any(|x| x.1 == i) { okb3 += 1; }
        let mut rc: Vec<(f32, usize)> = (0..nd).map(|j| {
            let mut s = 0f32;
            for e in 0..ne { s += cosd(&q158(&expert_view(&q, e)), &docq[j][e]); }
            (s / ne as f32, j)
        }).collect();
        rc.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if rc[0].1 == i { okc += 1; }
        if rc.iter().take(3).any(|x| x.1 == i) { okc2 += 1; }
        if rc.iter().take(40).any(|x| x.1 == i) { okc3 += 1; }
        let mut t20: Vec<usize> = rc.iter().take(20).map(|x| x.1).collect();
        t20.sort_by(|a, b| cosd(&q, &docs[*b]).partial_cmp(&cosd(&q, &docs[*a])).unwrap());
        if t20[0] == i { okd += 1; }
        let mut t40: Vec<usize> = rc.iter().take(40).map(|x| x.1).collect();
        t40.sort_by(|a, b| cosd(&q, &docs[*b]).partial_cmp(&cosd(&q, &docs[*a])).unwrap());
        if t40[0] == i { oke += 1; }
    }
    let n = nd; let pc = |k: usize| 100.0 * k as f64 / n as f64;
    let _ = (&mut oke2, &mut oke3);
    println!();
    println!("  A fp32 (K40 kept)          recall@1 {:7.2}%  recall@3 {:7.2}%  recall@40 {:7.2}%", pc(okf), pc(okf2), pc(okf3));
    println!("  B single 1.58-bit          recall@1 {:7.2}%  recall@3 {:7.2}%  recall@40 {:7.2}%", pc(okb), pc(okb2), pc(okb3));
    println!("  C 14x 1.58-bit ensemble    recall@1 {:7.2}%  recall@3 {:7.2}%  recall@40 {:7.2}%", pc(okc), pc(okc2), pc(okc3));
    println!("  D ensemble + fp32 rerank20 recall@1 {:7.2}%", pc(okd));
    println!("  E ensemble + fp32 rerank40 recall@1 {:7.2}%", pc(oke));
    println!();
    println!("TARGET 99.9%: A={} B={} C={} D={} E={}",
        if pc(okf)>=99.9 {"HIT"} else {"miss"}, if pc(okb)>=99.9 {"HIT"} else {"miss"},
        if pc(okc)>=99.9 {"HIT"} else {"miss"}, if pc(okd)>=99.9 {"HIT"} else {"miss"},
        if pc(oke)>=99.9 {"HIT"} else {"miss"});
}
