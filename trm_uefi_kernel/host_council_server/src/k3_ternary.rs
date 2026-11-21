//! G53 — the agent roster at 1.58 bits, so it fits in L2 instead of L3.
//!
//! Measured state before this module: `k3_heads.bin` is 29.87 MB of dense f32, 15 heads,
//! 1.99 MB each. That fits the 3900X's 64 MB L3 with 2.1x headroom, and a cycling test showed
//! the roster stays there (same head 47.9 us p50, all 15 heads 60.7 us, 1.27x) — but every
//! forward pass streams a whole 2 MB head, and 2 MB is 4x L2, so the pass is L3-bandwidth-bound
//! and never reuses a hot line.
//!
//! Packing the weights to balanced ternary at 2 bits/trit (TurboVec 1.58, BitNet b1.58 doctrine:
//! 4 trits per byte, 00->0, 01->+1, 10->-1, scale = mean|v|, trit threshold 0.5*scale) takes a
//! head to ~127 KB, which is L2-resident, and the entire roster to ~1.9 MB, which fits L3 with
//! ~34x headroom.
//!
//! What is quantised and what is NOT: every weight matrix (emb, A/B/C/D, MoE w1/w2, gate, out,
//! both classifier layers). Biases, LayerNorm weight and bias stay f32 — they are a few hundred
//! floats and quantising them would scale-shift the whole normalised activation.
//!
//! The matmul consumes packed trits directly: w*x with w = t*scale becomes signed adds of x,
//! scaled once at the end. No dequantised f32 copy is ever materialised for the forward pass,
//! because a dequantised copy would cost the same DRAM traffic as the f32 head and make the
//! whole exercise a measurement of nothing.

use crate::turbovec::TurboVec158Quantizer;
use trm_uefi_kernel::k3_head::{char_ngram_ids, featurize, K3Head, CLASSES, HIDDEN, LN_EPS};

/// A ternary tensor: packed 2-bit trits plus one per-tensor scale.
pub struct TriVec {
    packed: Vec<u8>,
    scale: f32,
    len: usize,
}

impl TriVec {
    pub fn from_f32(v: &[f32]) -> Self {
        let (packed, scale) = TurboVec158Quantizer::quantize_158(v);
        Self { packed, scale, len: v.len() }
    }

    /// Pack a table row-by-row with one scale per row.
    ///
    /// The embedding table is 75% of a head's weights and its rows do not share a magnitude:
    /// a single per-tensor scale (mean|v| over 393k values) left the table at 0.51 relative RMS
    /// error, which is why the first parity run agreed on only 34/60 decisions. A scale per row
    /// costs 4 bytes per row (16 KB per head) and removes that mismatch.
    pub fn from_f32_rows(v: &[f32], row: usize) -> (Self, Vec<f32>) {
        if row == 0 {
            let s = Self::from_f32(v);
            return (s, Vec::new());
        }
        let rows = (v.len() + row - 1) / row;
        let mut packed = Vec::with_capacity((v.len() + 3) / 4);
        let mut scales = Vec::with_capacity(rows);
        for r in 0..rows {
            let a = r * row;
            let b = (a + row).min(v.len());
            let (pk, sc) = TurboVec158Quantizer::quantize_158(&v[a..b]);
            scales.push(sc);
            // quantize_158 pads to a byte boundary per call, so re-pack to a continuous bitstream
            for i in a..b {
                let byte_idx = i >> 2;
                if byte_idx >= packed.len() {
                    packed.push(0u8);
                }
                let trit = {
                    let rel = i - a;
                    let c = (pk[rel >> 2] >> ((rel & 3) * 2)) & 3;
                    TurboVec158Quantizer::code_to_trit(c)
                };
                let shift = (i & 3) * 2;
                packed[byte_idx] |= TurboVec158Quantizer::trit_to_code(trit) << shift;
            }
        }
        (Self { packed, scale: 1.0, len: v.len() }, scales)
    }

    /// Bytes this tensor occupies, including its f32 scale.
    pub fn bytes(&self) -> usize {
        self.packed.len() + 4
    }

    /// Full-vector dot, consuming packed trits four at a time.
    #[allow(dead_code)] // G6: no caller today. This is the only ternary dot in the crate; deleting
                        // it would remove the primitive rather than the dead weight.
    pub fn dot(&self, x: &[f32]) -> f32 {
        let mut acc = 0.0f32;
        let n = self.len.min(x.len());
        let mut i = 0usize;
        while i + 4 <= n {
            let t = TurboVec158Quantizer::unpack_4_trits(self.packed[i >> 2]);
            acc += x[i] * t[0] as f32 + x[i + 1] * t[1] as f32
                + x[i + 2] * t[2] as f32 + x[i + 3] * t[3] as f32;
            i += 4;
        }
        while i < n {
            let code = (self.packed[i >> 2] >> ((i & 3) * 2)) & 3;
            match TurboVec158Quantizer::code_to_trit(code) {
                1 => acc += x[i],
                -1 => acc -= x[i],
                _ => {}
            }
            i += 1;
        }
        acc * self.scale
    }

    /// Dot of output row `row` (width `inp`) against `x`, without slicing the packed buffer.
    pub fn row_dot(&self, row: usize, inp: usize, x: &[f32]) -> f32 {
        let start = row * inp;
        let end = (start + inp).min(self.len);
        let mut acc = 0.0f32;
        let mut i = start;
        while i < end {
            let code = (self.packed[i >> 2] >> ((i & 3) * 2)) & 3;
            match TurboVec158Quantizer::code_to_trit(code) {
                1 => acc += x[i - start],
                -1 => acc -= x[i - start],
                _ => {}
            }
            i += 1;
        }
        acc * self.scale
    }

    /// Accumulate row `row` into `acc` in trit space, then scale — used by the embedding bag.
    fn row_accum_trits(&self, row: usize, inp: usize, acc: &mut [f32]) {
        let start = row * inp;
        let end = (start + inp).min(self.len);
        let mut i = start;
        while i < end {
            let code = (self.packed[i >> 2] >> ((i & 3) * 2)) & 3;
            match TurboVec158Quantizer::code_to_trit(code) {
                1 => acc[i - start] += 1.0,
                -1 => acc[i - start] -= 1.0,
                _ => {}
            }
            i += 1;
        }
    }
}

pub struct TriExpert {
    w1: TriVec,
    b1: Vec<f32>,
    w2: TriVec,
    b2: Vec<f32>,
}

pub struct TriHead {
    pub name: String,
    buckets: usize,
    emb: TriVec,
    /// One scale per embedding row (empty = the vector was packed with a single scale).
    emb_scales: Vec<f32>,
    dim: usize,
    state: usize,
    n_experts: usize,
    a: TriVec,
    b: TriVec,
    c: TriVec,
    d: TriVec,
    experts: Vec<TriExpert>,
    gate_w: TriVec,
    gate_b: Vec<f32>,
    cls_w1: TriVec,
    cls_b1: Vec<f32>,
    cls_w2: TriVec,
    cls_b2: Vec<f32>,
    out_w: TriVec,
    out_b: Vec<f32>,
    norm_w: Vec<f32>,
    norm_b: Vec<f32>,
}

fn lin(w: &TriVec, b: &[f32], x: &[f32], out: usize, inp: usize) -> Vec<f32> {
    let mut y = Vec::with_capacity(out);
    for r in 0..out {
        let bias = if r < b.len() { b[r] } else { 0.0 };
        y.push(w.row_dot(r, inp, x) + bias);
    }
    y
}

fn relu_in_place(v: &mut [f32]) {
    for x in v.iter_mut() {
        if *x < 0.0 {
            *x = 0.0;
        }
    }
}

fn exp_f32(x: f32) -> f32 {
    // Range-reduced exp, same shape the f32 path uses.
    let t = 1.0 + x / 256.0;
    let mut r = 1.0f32;
    for _ in 0..8 {
        r *= t;
    }
    r.max(0.0)
}

impl TriHead {
    pub fn from_head(h: &K3Head) -> Self {
        Self {
            name: h.name.clone(),
            buckets: h.buckets,
            // An empty f32 table stays empty rather than becoming a 4-byte scale with no data.
            emb: if h.emb.is_empty() {
                TriVec::from_f32(&h.emb)
            } else {
                TriVec::from_f32_rows(&h.emb, h.dim).0
            },
            emb_scales: if h.emb.is_empty() {
                Vec::new()
            } else {
                TriVec::from_f32_rows(&h.emb, h.dim).1
            },
            dim: h.dim,
            state: h.state,
            n_experts: h.n_experts,
            a: TriVec::from_f32(&h.a),
            b: TriVec::from_f32(&h.b),
            c: TriVec::from_f32(&h.c),
            d: TriVec::from_f32(&h.d),
            experts: h
                .experts
                .iter()
                .map(|e| TriExpert {
                    w1: TriVec::from_f32(&e.w1),
                    b1: e.b1.clone(),
                    w2: TriVec::from_f32(&e.w2),
                    b2: e.b2.clone(),
                })
                .collect(),
            gate_w: TriVec::from_f32(&h.gate_w),
            gate_b: h.gate_b.clone(),
            cls_w1: TriVec::from_f32(&h.cls_w1),
            cls_b1: h.cls_b1.clone(),
            cls_w2: TriVec::from_f32(&h.cls_w2),
            cls_b2: h.cls_b2.clone(),
            out_w: TriVec::from_f32(&h.out_w),
            out_b: h.out_b.clone(),
            norm_w: h.norm_w.clone(),
            norm_b: h.norm_b.clone(),
        }
    }

    pub fn bytes(&self) -> usize {
        self.emb.bytes() + self.emb_scales.len() * 4
            + self.a.bytes()
            + self.b.bytes()
            + self.c.bytes()
            + self.d.bytes()
            + self
                .experts
                .iter()
                .map(|e| e.w1.bytes() + e.w2.bytes() + (e.b1.len() + e.b2.len()) * 4)
                .sum::<usize>()
            + self.gate_w.bytes()
            + (self.gate_b.len() + self.cls_b1.len() + self.cls_b2.len() + self.out_b.len()
                + self.norm_w.len() + self.norm_b.len()) * 4
            + self.cls_w1.bytes()
            + self.cls_w2.bytes()
            + self.out_w.bytes()
    }

    pub fn input(&self, text: &str) -> Vec<f32> {
        if self.buckets == 0 {
            return featurize(text, self.dim);
        }
        let ids = char_ngram_ids(text, self.buckets);
        let mut x = vec![0.0f32; self.dim];
        if self.emb_scales.is_empty() {
            for &id in &ids {
                self.emb.row_accum_trits(id, self.dim, &mut x);
            }
            let n = if ids.is_empty() { 1.0 } else { ids.len() as f32 };
            let s = self.emb.scale / n;
            for v in x.iter_mut() {
                *v *= s;
            }
        } else {
            // accumulate with each row's own scale
            let mut tmp = vec![0.0f32; self.dim];
            for &id in &ids {
                for (j, tv) in tmp.iter_mut().enumerate() {
                    *tv = 0.0;
                }
                self.emb.row_accum_trits(id, self.dim, &mut tmp);
                let sc = self.emb_scales.get(id).copied().unwrap_or(0.0);
                for j in 0..self.dim {
                    x[j] += tmp[j] * sc;
                }
            }
            let n = if ids.is_empty() { 1.0 } else { ids.len() as f32 };
            for v in x.iter_mut() {
                *v /= n;
            }
        }
        x
    }

    pub fn classify(&self, text: &str) -> (i8, [f32; CLASSES]) {
        let x = self.input(text);
        let logits = self.forward_logits(&x);
        let mut best = 0usize;
        for k in 1..CLASSES {
            if logits[k] > logits[best] {
                best = k;
            }
        }
        (best as i8 - 1, logits)
    }

    /// Same evaluation order as `K3Head::forward_logits`, with ternary matmuls.
    pub fn forward_logits(&self, x: &[f32]) -> [f32; CLASSES] {
        let dim = self.dim;
        let state = self.state;
        let zero_s = vec![0.0f32; state];
        let zero_d = vec![0.0f32; dim];

        let dh = lin(&self.b, &zero_s, x, state, dim);
        let mut h = vec![0.0f32; state];
        for j in 0..state {
            h[j] += 0.05 * dh[j];
        }
        let mut bonsai = lin(&self.c, &zero_d, &h, dim, state);
        let dxt = lin(&self.d, &zero_d, x, dim, dim);
        for i in 0..dim {
            bonsai[i] += dxt[i];
        }

        let mut gate = lin(&self.gate_w, &self.gate_b, x, self.n_experts, dim);
        let mut denom = 0.0f32;
        let mut gmax = gate[0];
        for g in &gate {
            if *g > gmax {
                gmax = *g;
            }
        }
        for g in gate.iter_mut() {
            *g = exp_f32(*g - gmax);
            denom += *g;
        }
        for g in gate.iter_mut() {
            *g /= denom;
        }
        let mut i0 = 0usize;
        for i in 1..self.n_experts {
            if gate[i] > gate[i0] {
                i0 = i;
            }
        }
        let mut i1 = usize::MAX;
        for i in 0..self.n_experts {
            if i == i0 {
                continue;
            }
            if i1 == usize::MAX || gate[i] > gate[i1] {
                i1 = i;
            }
        }
        let mut moe = vec![0.0f32; dim];
        for (idx, w) in [(i0, gate[i0]), (i1, gate[i1.min(self.n_experts - 1)])] {
            if idx == usize::MAX {
                continue;
            }
            let e = &self.experts[idx];
            let mut hid = lin(&e.w1, &e.b1, x, HIDDEN, dim);
            relu_in_place(&mut hid);
            let eo = lin(&e.w2, &e.b2, &hid, dim, HIDDEN);
            for i in 0..dim {
                moe[i] += w * eo[i];
            }
        }

        let mut combined = vec![0.0f32; dim];
        for i in 0..dim {
            combined[i] = bonsai[i] + 0.3 * moe[i];
        }
        relu_in_place(&mut combined);
        let proj = lin(&self.out_w, &self.out_b, &combined, dim, dim);
        let mut y = vec![0.0f32; dim];
        for i in 0..dim {
            y[i] = x[i] + proj[i];
        }
        let mut mean = 0.0f32;
        for v in &y {
            mean += *v;
        }
        mean /= dim as f32;
        let mut var = 0.0f32;
        for v in &y {
            let t = *v - mean;
            var += t * t;
        }
        var /= dim as f32;
        let inv = 1.0 / ((var + LN_EPS).sqrt());
        let mut normed = vec![0.0f32; dim];
        for i in 0..dim {
            normed[i] = (y[i] - mean) * inv * self.norm_w[i] + self.norm_b[i];
        }

        let mut c1 = lin(&self.cls_w1, &self.cls_b1, &normed, HIDDEN, dim);
        relu_in_place(&mut c1);
        let logits = lin(&self.cls_w2, &self.cls_b2, &c1, CLASSES, HIDDEN);
        [logits[0], logits[1], logits[2]]
    }
}

/// Size and agreement of pack/unpack against the f32 source, before any forward pass.
pub fn pack_report(heads: &[K3Head]) -> serde_json::Value {
    let f32_bytes: usize = heads.iter().map(f32_head_bytes).sum();
    let tri: Vec<TriHead> = heads.iter().map(TriHead::from_head).collect();
    let tri_bytes: usize = tri.iter().map(TriHead::bytes).sum();
    // Reconstruction error on the biggest tensor of each head: the embedding table.
    let mut rel_err = 0.0f64;
    let mut counted = 0usize;
    for (h, t) in heads.iter().zip(tri.iter()) {
        if h.emb.is_empty() {
            continue;
        }
        // Measure the error the head actually ships: one scale per embedding row.
        let rows = (h.emb.len() + h.dim - 1) / h.dim;
        let mut num = 0.0f64;
        let mut den = 0.0f64;
        for r in 0..rows {
            let a = r * h.dim;
            let b = (a + h.dim).min(h.emb.len());
            let (pk, sc) = TurboVec158Quantizer::quantize_158(&h.emb[a..b]);
            let back = TurboVec158Quantizer::dequantize_158(&pk, b - a, sc);
            for (o, rr) in h.emb[a..b].iter().zip(back.iter()) {
                num += ((o - rr) as f64).powi(2);
                den += (*o as f64).powi(2);
            }
        }
        if den > 0.0 {
            rel_err += (num / den).sqrt();
            counted += 1;
        }
    }
    serde_json::json!({
        "heads": heads.len(),
        "f32_bytes": f32_bytes,
        "ternary_bytes": tri_bytes,
        "compression": if f32_bytes > 0 { f32_bytes as f64 / tri_bytes as f64 } else { 0.0 },
        "f32_mb": f32_bytes as f64 / 1048576.0,
        "ternary_mb": tri_bytes as f64 / 1048576.0,
        "emb_relative_rms_error": if counted > 0 { rel_err / counted as f64 } else { 0.0 },
        "note": "scales are stored f32 (per tensor; per row for the embedding table) and included in ternary_bytes",
    })
}

pub fn f32_head_bytes(h: &K3Head) -> usize {
    let f = |n: usize| n * 4;
    f(h.emb.len())
        + f(h.a.len() + h.b.len() + h.c.len() + h.d.len())
        + h.experts
            .iter()
            .map(|e| f(e.w1.len() + e.w2.len() + e.b1.len() + e.b2.len()))
            .sum::<usize>()
        + f(
            h.gate_w.len()
                + h.gate_b.len()
                + h.cls_w1.len()
                + h.cls_b1.len()
                + h.cls_w2.len()
                + h.cls_b2.len()
                + h.out_w.len()
                + h.out_b.len()
                + h.norm_w.len()
                + h.norm_b.len(),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blob() -> Option<Vec<K3Head>> {
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(dir) = std::env::current_dir() {
            candidates.push(dir.join("assets").join("k3_heads.bin"));
            candidates.push(dir.join("..").join("..").join("scratch").join("wt_feat").join("assets").join("k3_heads.bin"));
            candidates.push(dir.join("..").join("..").join("trm_uefi_kernel").join("assets").join("k3_heads.bin"));
        }
        for p in candidates {
            if let Ok(b) = std::fs::read(&p) {
                if let Some(h) = trm_uefi_kernel::k3_head::parse(&b) {
                    if !h.is_empty() {
                        return Some(h);
                    }
                }
            }
        }
        None
    }

    #[test]
    fn packing_shrinks_every_tensor_and_keeps_the_scale() {
        let v: Vec<f32> = (0..1024).map(|i| ((i as f32) * 0.37).sin()).collect();
        let t = TriVec::from_f32(&v);
        assert_eq!(t.len, 1024);
        assert_eq!(t.packed.len(), 256);
        assert!(t.bytes() < v.len() * 4 / 3, "expected >3x smaller, got {} vs {}", t.bytes(), v.len()*4);
        // dot against the quantised weights must equal the add/sub accumulate exactly
        let x = vec![0.5f32; 1024];
        let deq = TurboVec158Quantizer::dequantize_158(&t.packed, 1024, t.scale);
        let mut reference = 0.0f32;
        for (a, b) in deq.iter().zip(x.iter()) {
            reference += a * b;
        }
        let got = t.dot(&x);
        assert!((got - reference).abs() < 1e-3 * reference.abs().max(1.0), "dot {got} vs {reference}");
    }

    #[test]
    fn zeros_quantise_to_zero() {
        let t = TriVec::from_f32(&vec![0.0f32; 128]);
        assert_eq!(t.scale, 0.0);
        assert_eq!(t.dot(&vec![1.0f32; 128]), 0.0);
    }

    #[test]
    fn head_round_trips_and_reports_compression() {
        let heads = match blob() {
            Some(h) => h,
            None => return, // blob not present in this working dir; route covers the live case
        };
        let rep = pack_report(&heads);
        let comp = rep["compression"].as_f64().unwrap_or(0.0);
        assert!(comp > 8.0, "compression too low: {rep}");
    }
}
