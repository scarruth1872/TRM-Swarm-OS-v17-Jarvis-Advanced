//! K3 decision heads - trained per-agent ternary classifiers, loaded by the kernel.
//!
//! WHY THIS EXISTS: the 15 Bonsai-K3 heads were trained in Python and existed only
//! as PyTorch `.pt` pickles, which the Rust kernel cannot read. That is the same
//! "trained but unreachable" gap as the 12 GB of dead `_trits.bin`. They are now
//! exported to one documented blob (tools/export_k3_heads.py) and loaded here, so
//! the microkernel can classify a request without Python and without an LLM.
//!
//! The network is small: 96-dim hashed bag-of-words -> a Bonsai/K3 state block ->
//! 8-expert MoE (top-2) -> residual + LayerNorm -> 3-way classifier. 128,811
//! parameters per agent. Output is a Kleene trit: -1 BLOCK, 0 ANALYZE, +1 PROCEED.
//!
//! Everything here mirrors swarm_v2's BonsaiInstellaModel exactly, including the
//! featuriser, because a classifier that disagrees with its trainer is a second
//! model wearing the first one's name. Parity is asserted by host_test
//! `k3_head_bench`, which compares against Python on identical prompts.
//!
//! NOTE ON A: the Python forward starts h at zero and takes a single step, so
//! `dh = h@A.T + x@B.T` collapses to `x@B.T` and A never affects the output. It is
//! parsed anyway to keep the blob a faithful copy of the checkpoint.

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

// no_std has no f32::sqrt/exp, so the kernel's own implementations are used. The
// host bench provides a std shim under the same names (see host_test/src/float_math.rs).
use crate::float_math::{exp_f32, sqrt_f64};

pub const HIDDEN: usize = 64;
pub const CLASSES: usize = 3;
pub const LN_EPS: f32 = 1e-5;

/// ISO-HDLC CRC-32 (the same polynomial zlib.crc32 uses). Bitwise, no table, so
/// it is const-friendly and needs no crates. Must match Python exactly, or the
/// hashed features land in different buckets and every prediction diverges.
#[inline]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Tokenise exactly like Python's `re.findall(r"[a-z0-9']+", text.lower())`.
fn tokens(text: &str) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    for ch in text.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '\'' {
            cur.push(c as u8);
        } else if !cur.is_empty() {
            out.push(core::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Hashed bag-of-words: unigrams then "a_b" bigrams, sign from bit 16, L2 norm.
pub fn featurize(text: &str, dim: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; dim];
    let toks = tokens(text);
    let mut gram: Vec<u8> = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        gram.clear();
        gram.extend_from_slice(t);
        let h = crc32(&gram);
        v[(h as usize) % dim] += if (h >> 16) & 1 == 1 { 1.0 } else { -1.0 };
        if i + 1 < toks.len() {
            gram.clear();
            gram.extend_from_slice(t);
            gram.push(b'_');
            gram.extend_from_slice(&toks[i + 1]);
            let h2 = crc32(&gram);
            v[(h2 as usize) % dim] += if (h2 >> 16) & 1 == 1 { 1.0 } else { -1.0 };
        }
    }
    let mut n = 0.0f32;
    for x in &v {
        n += x * x;
    }
    n = sqrt_f64(n as f64) as f32;
    if n > 0.0 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
    v
}

/// Number of char n-gram ids per text, fixed. Python pads to the same length with
/// bucket 0; both sides MUST agree or the EmbeddingBag mean is scaled wrong.
pub const EMB_CAP: usize = 192;

/// Hashed character 3-5 gram ids, fixed length, word boundaries marked with < >.
///
/// Mirrors Python char_ngram_ids() exactly: lowercase, non-alnum to space, words
/// joined as <w1><w2>, every gram of length 3..=5 hashed with CRC-32 mod buckets,
/// truncated at EMB_CAP and padded with bucket 0.
pub fn char_ngram_ids(text: &str, buckets: usize) -> Vec<usize> {
    let lower = text.to_lowercase();
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    for ch in lower.chars() {
        if ch.is_alphanumeric() {
            cur.push(ch);
        } else if !cur.is_empty() {
            words.push(core::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    let mut padded = String::from("<");
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            padded.push_str("><");
        }
        padded.push_str(w);
    }
    padded.push('>');
    let bytes: Vec<char> = padded.chars().collect();

    let mut ids: Vec<usize> = Vec::with_capacity(EMB_CAP);
    'outer: for n in 3..=5usize {
        if bytes.len() < n {
            continue;
        }
        for i in 0..=(bytes.len() - n) {
            let gram: String = bytes[i..i + n].iter().collect();
            if gram.chars().all(|c| c == '<' || c == '>') {
                continue;
            }
            ids.push((crc32(gram.as_bytes()) as usize) % buckets);
            if ids.len() >= EMB_CAP {
                break 'outer;
            }
        }
    }
    if ids.is_empty() {
        ids.push(0);
    }
    while ids.len() < EMB_CAP {
        ids.push(0);
    }
    ids
}

pub struct Expert {
    pub w1: Vec<f32>, // [HIDDEN*dim]
    pub b1: Vec<f32>, // [HIDDEN]
    pub w2: Vec<f32>, // [dim*HIDDEN]
    pub b2: Vec<f32>, // [dim]
}

pub struct K3Head {
    pub name: String,
    /// 0 = fixed hashed bag-of-words input; >0 = learned char-n-gram table.
    pub buckets: usize,
    /// Embedding table [buckets, dim], row-major, empty when buckets == 0.
    pub emb: Vec<f32>,
    pub dim: usize,
    pub state: usize,
    pub n_experts: usize,
    pub a: Vec<f32>,
    pub b: Vec<f32>,
    pub c: Vec<f32>,
    pub d: Vec<f32>,
    pub experts: Vec<Expert>,
    pub gate_w: Vec<f32>,
    pub gate_b: Vec<f32>,
    pub cls_w1: Vec<f32>,
    pub cls_b1: Vec<f32>,
    pub cls_w2: Vec<f32>,
    pub cls_b2: Vec<f32>,
    pub out_w: Vec<f32>,
    pub out_b: Vec<f32>,
    pub norm_w: Vec<f32>,
    pub norm_b: Vec<f32>,
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }
    fn u16(&mut self) -> Option<u16> {
        if self.pos + 2 > self.buf.len() {
            return None;
        }
        let v = u16::from_le_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
        Some(v)
    }
    fn u32(&mut self) -> Option<u32> {
        if self.pos + 4 > self.buf.len() {
            return None;
        }
        let v = u32::from_le_bytes([
            self.buf[self.pos],
            self.buf[self.pos + 1],
            self.buf[self.pos + 2],
            self.buf[self.pos + 3],
        ]);
        self.pos += 4;
        Some(v)
    }
    fn name(&mut self, len: usize) -> Option<String> {
        if self.pos + len > self.buf.len() {
            return None;
        }
        let s = core::str::from_utf8(&self.buf[self.pos..self.pos + len]).ok()?.into();
        self.pos += len;
        Some(s)
    }
    fn f32s(&mut self, n: usize) -> Option<Vec<f32>> {
        let bytes = n.checked_mul(4)?;
        if self.pos + bytes > self.buf.len() {
            return None;
        }
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let o = self.pos + i * 4;
            out.push(f32::from_le_bytes([
                self.buf[o],
                self.buf[o + 1],
                self.buf[o + 2],
                self.buf[o + 3],
            ]));
        }
        self.pos += bytes;
        Some(out)
    }
}

/// Parse the K3HD blob. Returns None on any structural problem (bad magic, short
/// buffer, wrong version) - a partially decoded head must never be used, because
/// garbage weights produce a confident trit that looks like a real answer.
pub fn parse(blob: &[u8]) -> Option<Vec<K3Head>> {
    if blob.len() < 12 || &blob[0..4] != b"K3HD" {
        return None;
    }
    let mut r = Reader::new(blob);
    r.pos = 4;
    let version = r.u32()?;
    // v1: no buckets field, fixed BoW only. v2: adds the embedding table. Both are
    // accepted so an older blob still loads with buckets = 0.
    if version != 1 && version != 2 {
        return None;
    }
    let n_agents = r.u32()? as usize;
    let mut heads = Vec::with_capacity(n_agents);
    for _ in 0..n_agents {
        let nl = r.u16()? as usize;
        let name = r.name(nl)?;
        let dim = r.u32()? as usize;
        let state = r.u32()? as usize;
        let n_experts = r.u32()? as usize;
        let buckets = if version >= 2 { r.u32()? as usize } else { 0 };
        if buckets > 65536 {
            return None;
        }
        if dim == 0 || dim > 4096 || state == 0 || state > 4096 || n_experts == 0 || n_experts > 256 {
            return None;
        }
        let emb = if buckets > 0 { r.f32s(buckets * dim)? } else { Vec::new() };
        let a = r.f32s(state * state)?;
        let b = r.f32s(state * dim)?;
        let c = r.f32s(dim * state)?;
        let d = r.f32s(dim * dim)?;
        let mut experts = Vec::with_capacity(n_experts);
        for _ in 0..n_experts {
            experts.push(Expert {
                w1: r.f32s(HIDDEN * dim)?,
                b1: r.f32s(HIDDEN)?,
                w2: r.f32s(dim * HIDDEN)?,
                b2: r.f32s(dim)?,
            });
        }
        let gate_w = r.f32s(n_experts * dim)?;
        let gate_b = r.f32s(n_experts)?;
        let cls_w1 = r.f32s(HIDDEN * dim)?;
        let cls_b1 = r.f32s(HIDDEN)?;
        let cls_w2 = r.f32s(CLASSES * HIDDEN)?;
        let cls_b2 = r.f32s(CLASSES)?;
        let out_w = r.f32s(dim * dim)?;
        let out_b = r.f32s(dim)?;
        let norm_w = r.f32s(dim)?;
        let norm_b = r.f32s(dim)?;
        heads.push(K3Head {
            name, buckets, emb, dim, state, n_experts, a, b, c, d, experts,
            gate_w, gate_b, cls_w1, cls_b1, cls_w2, cls_b2,
            out_w, out_b, norm_w, norm_b,
        });
    }
    Some(heads)
}

/// y[i] = sum_j w[i*in + j] * x[j] + b[i]   (PyTorch Linear row-major [out][in])
#[inline]
fn lin(w: &[f32], b: &[f32], x: &[f32], out: usize, inp: usize) -> Vec<f32> {
    let mut y = vec![0.0f32; out];
    for i in 0..out {
        let row = &w[i * inp..(i + 1) * inp];
        let mut acc = b[i];
        for j in 0..inp {
            acc += row[j] * x[j];
        }
        y[i] = acc;
    }
    y
}

#[inline]
fn relu_in_place(v: &mut [f32]) {
    for x in v.iter_mut() {
        if *x < 0.0 {
            *x = 0.0;
        }
    }
}

impl K3Head {
    /// Total parameters this head actually uses, INCLUDING the embedding table.
    ///
    /// An embedding-equipped head is 522,027 params, not 128,811: omitting a 4096x96
    /// table that is 75% of the weights makes the platform's own inventory wrong in
    /// the flattering direction.
    pub fn params(&self) -> usize {
        self.a.len() + self.b.len() + self.c.len() + self.d.len()
            + self.experts.iter().map(|e| e.w1.len() + e.b1.len() + e.w2.len() + e.b2.len()).sum::<usize>()
            + self.gate_w.len() + self.gate_b.len()
            + self.cls_w1.len() + self.cls_b1.len() + self.cls_w2.len() + self.cls_b2.len()
            + self.out_w.len() + self.out_b.len() + self.norm_w.len() + self.norm_b.len()
            + self.emb.len()
    }

    /// Parameters excluding the embedding table, so the two are separable.
    pub fn params_core(&self) -> usize {
        self.params() - self.emb.len()
    }

    /// Input vector for this head: learned embedding mean, or fixed hashed BoW.
    ///
    /// EmbeddingBag(mode='mean') averages the rows of every index in the bag, so the
    /// divisor is the full padded length (EMB_CAP), not the number of distinct grams.
    pub fn input(&self, text: &str) -> Vec<f32> {
        if self.buckets == 0 {
            return featurize(text, self.dim);
        }
        let ids = char_ngram_ids(text, self.buckets);
        let mut x = vec![0.0f32; self.dim];
        for &id in &ids {
            let row = &self.emb[id * self.dim..(id + 1) * self.dim];
            for j in 0..self.dim {
                x[j] += row[j];
            }
        }
        let n = if ids.is_empty() { 1.0 } else { ids.len() as f32 };
        for v in x.iter_mut() {
            *v /= n;
        }
        x
    }

    /// Classify raw text -> (trit, logits). trit is -1 BLOCK, 0 ANALYZE, +1 PROCEED.
    pub fn classify(&self, text: &str) -> (i8, [f32; CLASSES]) {
        let x = self.input(text);
        let logits = self.forward_logits(&x);
        let mut best = 0usize;
        for k in 1..CLASSES {
            if logits[k] > logits[best] {
                best = k;
            }
        }
        let trit = best as i8 - 1; // 0->-1, 1->0, 2->+1
        (trit, logits)
    }

    pub fn forward_logits(&self, x: &[f32]) -> [f32; CLASSES] {
        let dim = self.dim;
        let state = self.state;

        // 1. K3/Bonsai block. h starts at zero (Python: h = torch.zeros), so the
        //    A term vanishes; kept in the same order the Python code evaluates it.
        let dh = lin(&self.b, &vec![0.0f32; state], x, state, dim); // x @ B.T
        let mut h = vec![0.0f32; state];
        for j in 0..state {
            h[j] = h[j] + 0.05 * dh[j];
        }
        let mut bonsai = lin(&self.c, &vec![0.0f32; dim], &h, dim, state); // h @ C.T
        let dxt = lin(&self.d, &vec![0.0f32; dim], x, dim, dim); // x @ D.T
        for i in 0..dim {
            bonsai[i] += dxt[i];
        }

        // 2. MoE gate (softmax) + top-2 experts.
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
        // top-2 with stable tie-break (earlier index wins), matching torch.topk on CPU.
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
        for (idx, w) in [(i0, gate[i0]), (i1, gate[i1])] {
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

        // 3. combine + residual projection + LayerNorm
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
        var /= dim as f32; // biased, as torch.nn.LayerNorm
        let inv = 1.0 / (sqrt_f64((var + LN_EPS) as f64) as f32);
        let mut normed = vec![0.0f32; dim];
        for i in 0..dim {
            normed[i] = (y[i] - mean) * inv * self.norm_w[i] + self.norm_b[i];
        }

        // 4. classifier -> 3 logits
        let mut c1 = lin(&self.cls_w1, &self.cls_b1, &normed, HIDDEN, dim);
        relu_in_place(&mut c1);
        let logits = lin(&self.cls_w2, &self.cls_b2, &c1, CLASSES, HIDDEN);
        [logits[0], logits[1], logits[2]]
    }
}

/// Softmax over 3 logits, for reporting confidence.
pub fn softmax3(l: &[f32; CLASSES]) -> [f32; CLASSES] {
    let m = l[0].max(l[1]).max(l[2]);
    let e = [exp_f32(l[0] - m), exp_f32(l[1] - m), exp_f32(l[2] - m)];
    let s = e[0] + e[1] + e[2];
    [e[0] / s, e[1] / s, e[2] / s]
}
