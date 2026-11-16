extern crate alloc;
mod float_math;
mod time;
mod turbovec;
use alloc::vec::Vec;
use alloc::vec;
use std::collections::HashMap;

/// One expert's view: each expert carries its own 1.58-bit weights, so its
/// effective per-dimension scaling differs from its siblings'.
fn expert_view(v: &[f32], e: usize) -> Vec<f32> {
    let mut o = v.to_vec();
    for (j, x) in o.iter_mut().enumerate() {
        let h = ((e as u64).wrapping_mul(1_000_003).wrapping_add((j as u64).wrapping_mul(7919)) % 1000) as f32 / 1000.0;
        *x *= 0.9 + 0.2 * h;
    }
    o
}

fn rawdot(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let mut d = 0f32;
    for i in 0..n { d += a[i]*b[i]; }
    d
}

fn cosd(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let (mut d, mut na, mut nb) = (0f32, 0f32, 0f32);
    for i in 0..n { d += a[i]*b[i]; na += a[i]*a[i]; nb += b[i]*b[i]; }
    let den = (na.max(1e-12)).sqrt() * (nb.max(1e-12)).sqrt();
    d / den
}

fn q158(v: &[f32]) -> Vec<f32> {
    let (pk, sc) = turbovec::TurboVec158Quantizer::quantize_158(v);
    turbovec::TurboVec158Quantizer::dequantize_158(&pk, v.len(), sc)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 2 { println!("usage: ensemble14 <corpus.tsv>"); return; }
    let txt = std::fs::read_to_string(&a[1]).unwrap_or_default();
    let lines: Vec<&str> = txt.lines().filter(|l| l.contains('\t')).collect();
    let nd = lines.len();
    let mut df: HashMap<String, usize> = HashMap::new();
    let mut toks: Vec<Vec<String>> = Vec::new();
    for l in &lines {
        let (_id, t) = l.split_once('\t').unwrap();
        let row: Vec<String> = t.split_whitespace().map(|s| s.to_lowercase()).collect();
        let mut seen: Vec<String> = row.clone(); seen.sort(); seen.dedup();
        for w in &seen { *df.entry(w.clone()).or_insert(0) += 1; }
        toks.push(row);
    }
    let dim = 4096usize;
    let mut idx: HashMap<String, usize> = HashMap::new();
    for (w, _) in df.iter() { if idx.len() >= dim { break; } idx.insert(w.clone(), idx.len()); }
    let mut docs: Vec<Vec<f32>> = Vec::new();
    for row in &toks {
        let mut v = vec![0f32; idx.len()];
        for w in row { if let Some(&c) = idx.get(w) {
            v[c] += (1.0 + (nd as f32 / (1.0 + *df.get(w).unwrap_or(&1) as f32)).ln()); } }
        let nm = v.iter().map(|x| x*x).sum::<f32>().sqrt().max(1e-12);
        for x in v.iter_mut() { *x /= nm; }
        docs.push(v);
    }
    println!("corpus: {} docs, dim {}", nd, idx.len());

    let ne = 14usize;
    // per-expert 1.58-bit encoding of every doc, and of every query view
    let mut docq: Vec<Vec<Vec<f32>>> = Vec::new();
    for d in &docs {
        let mut per: Vec<Vec<f32>> = Vec::new();
        for e in 0..ne { per.push(q158(&expert_view(d, e))); }
        docq.push(per);
    }
    let (mut ok_full, mut ok_single, mut ok_ens, mut ok_rer) = (0usize, 0usize, 0usize, 0usize);
    let mut ok_single_raw = 0usize;
    let kk = 20usize;
    for i in 0..nd {
        // query = odd-indexed tokens of doc i
        let mut q = vec![0f32; idx.len()];
        for (k, t) in toks[i].iter().enumerate() {
            if k % 2 == 0 { continue; }
            if let Some(&c) = idx.get(t) {
                q[c] += (1.0 + (nd as f32 / (1.0 + *df.get(t).unwrap_or(&1) as f32)).ln()); }
        }
        let qn = q.iter().map(|x| x*x).sum::<f32>().sqrt().max(1e-12);
        for x in q.iter_mut() { *x /= qn; }
        let mut rf: Vec<(f32, usize)> = (0..nd).map(|j| (cosd(&q, &docs[j]), j)).collect();
        rf.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if rf[0].1 == i { ok_full += 1; }

        let qs = q158(&q);
        let mut rs: Vec<(f32, usize)> = (0..nd).map(|j| (cosd(&qs, &docq[j][0]), j)).collect();
        rs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if rs[0].1 == i { ok_single += 1; }
        let mut rsr: Vec<(f32, usize)> = (0..nd).map(|j| (rawdot(&qs, &docq[j][0]), j)).collect();
        rsr.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if rsr[0].1 == i { ok_single_raw += 1; }

        // ensemble: mean of the 14 experts' quantised scores
        let mut re: Vec<(f32, usize)> = Vec::new();
        for j in 0..nd {
            let mut s = 0f32;
            for e in 0..ne { s += cosd(&q158(&expert_view(&q, e)), &docq[j][e]); }
            re.push((s / ne as f32, j));
        }
        re.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if re[0].1 == i { ok_ens += 1; }

        // ensemble shortlist + full-precision rerank
        let mut top: Vec<usize> = re.iter().take(kk).map(|x| x.1).collect();
        top.sort_by(|a, b| cosd(&q, &docs[*b]).partial_cmp(&cosd(&q, &docs[*a])).unwrap());
        if top[0] == i { ok_rer += 1; }
    }
    let pc = |n: usize| 100.0 * n as f64 / nd as f64;
    println!("A full precision (K40 kept)     recall@1 {:.1}%  ({}/{})", pc(ok_full), ok_full, nd);
    println!("B 1.58-bit, cosine metric       recall@1 {:.1}%  ({}/{})", pc(ok_single), ok_single, nd);
    println!("B' 1.58-bit, RAW DOT metric     recall@1 {:.1}%  ({}/{})", pc(ok_single_raw), ok_single_raw, nd);
    println!("C 14x 1.58-bit ensemble (mean)  recall@1 {:.1}%  ({}/{})", pc(ok_ens), ok_ens, nd);
    println!("D ensemble + fp32 rerank K={}   recall@1 {:.1}%  ({}/{})", kk, pc(ok_rer), ok_rer, nd);
    println!();
    println!("TARGET 99.9%: A={} B={} C={} D={}",
        if pc(ok_full) >= 99.9 {"HIT"} else {"miss"},
        if pc(ok_single) >= 99.9 {"HIT"} else {"miss"},
        if pc(ok_ens) >= 99.9 {"HIT"} else {"miss"},
        if pc(ok_rer) >= 99.9 {"HIT"} else {"miss"});
}
