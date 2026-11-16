//! TurboVec packed-domain ternary retrieval — the kernel's own scoring engine, not a report.
//!
//! `swarm_v2_nand_fabric.turbovec` keeps each document as 2-bit codes against that row's own
//! min..max (builder: `swarm_v2/core/build_nand_fabric_index.py`; BITS=2, LEVELS=3, so
//! `value_j = vmin + code_j * scale`). The previous reader dequantized every row to DIM f32s at
//! load and then scored in f32 — the exact path with a decode tax on top and 16x the memory.
//!
//! A dot product expands over that affine form:
//!
//! ```text
//! q . v = D*qmin*vmin + qscale*vmin*Sq + qmin*scale*Sc + qscale*scale*P
//! ```
//!
//! with `Sq = SUM q_code` (a query constant), `Sc = SUM c` (a row constant precomputed at load) and
//! `P = SUM q_code*c_code` — the only term that needs a scan. The scan therefore walks packed bytes
//! with integer arithmetic and the cosine still comes out exact rather than approximated.
//!
//! Ternary logic enters as the verdict: the ranking this produces is turned into a tri-state whose
//! band is derived from the score spread, never asserted (`Verdict::basis` carries the numbers).

use serde_json::Value;

const BITS: usize = 2;
const LEVELS: i32 = 3;
const STORE_RELS: [&str; 2] = [
    "swarm_v2_nand_fabric.turbovec",
    "swarm_v2_rust_kernel/swarm_v2_nand_fabric.turbovec",
];
const CHEMBL_STORE_RELS: [&str; 1] = ["swarm_v2_chembl_fabric.turbovec"];
const STRUCTURED_STORE_RELS: [&str; 1] = ["swarm_v2_structured_text.turbovec"];
const BIOASSAY_STORE_RELS: [&str; 1] = ["swarm_v2_bioassay.turbovec"];

struct Row {
    scale: f64,
    vmin: f64,
    sum_c: f64,
    sum_c2: f64,
    norm: f64,
}

pub struct Store {
    pub path: String,
    pub bytes: u64,
    dim: usize,
    keys: Vec<String>,
    codes: Vec<u8>,
    rows: Vec<Row>,
    per_vec: usize,
    /// Bytes of code per row: the record is 8 bytes of scale+vmin followed by the packed codes, and
    /// the scan walks only the codes. Keeping the two apart is what this stride is for.
    stride: usize,
}

impl Store {
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
    pub fn dim(&self) -> usize {
        self.dim
    }
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
    /// The packed 2-bit codes (stride bytes per row), for a scorer that reads the same bytes the
    /// scan walks.
    pub fn codes(&self) -> &[u8] {
        &self.codes
    }
    /// Bytes of code per row.
    pub fn stride(&self) -> usize {
        self.stride
    }
    /// Per-row scalar params as flat f32 [scale, vmin, sum_c, norm], for the GPU scan's affine
    /// reconstruction (the row's own precomputed moments).
    pub fn row_params(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.rows.len() * 4);
        for r in &self.rows {
            out.push(r.scale as f32);
            out.push(r.vmin as f32);
            out.push(r.sum_c as f32);
            out.push(r.norm as f32);
        }
        out
    }
    /// The store's own header facts, for anyone reporting what was actually mapped -- and the file's
    /// signature read at the same instant, so the report can be checked against the disk instead of
    /// describing a load that has already been replaced.
    pub fn header(&self) -> Value {
        let disk = crate::file_cache::Sig::of(std::path::Path::new(&self.path));
        serde_json::json!({
            "path": self.path, "bytes": self.bytes, "vectors": self.keys.len(),
            "dim": self.dim, "bits": BITS, "levels": LEVELS,
            "bytes_per_vector": self.per_vec,
            "file_bytes": disk.map(|s| s.len), "file_mtime": disk.map(|s| s.mtime_s()),
            "serves_the_file_on_disk": disk.map(|s| s.len == self.bytes as u64).unwrap_or(false),
        })
    }
}

/// The packed store's file, in the order the repo carries it.
fn store_path() -> Option<std::path::PathBuf> {
    STORE_RELS.iter().find_map(|r| crate::k3_retrieval::repo_relative(r))
}

/// Read the packed store, keeping the codes packed.
fn load(path: &std::path::Path) -> Option<Store> {
    let md = std::fs::metadata(path).ok()?;
    let b = std::fs::read(path).ok()?;
    if b.len() < 128 || &b[0..8] != b"TURBOVEC" {
        return None;
    }
    let dim = u32::from_le_bytes([b[20], b[21], b[22], b[23]]) as usize;
    let vectors = u32::from_le_bytes([b[24], b[25], b[26], b[27]]) as usize;
    let bits = u32::from_le_bytes([b[28], b[29], b[30], b[31]]) as usize;
    let keys_off =
        u64::from_le_bytes([b[32], b[33], b[34], b[35], b[36], b[37], b[38], b[39]]) as usize;
    let keys_len = u32::from_le_bytes([b[40], b[41], b[42], b[43]]) as usize;
    if bits != BITS || keys_off + keys_len > b.len() {
        return None;
    }
    let keys: Vec<String> = serde_json::from_slice::<Value>(&b[keys_off..keys_off + keys_len])
        .ok()?
        .get("keys")?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let per_vec = 8 + (dim * BITS + 7) / 8;
    if b.len() < 128 + vectors * per_vec {
        return None;
    }
    let stride = per_vec - 8;
    let mut codes = Vec::with_capacity(vectors * stride);
    let mut rows = Vec::with_capacity(vectors);
    for i in 0..vectors {
        let off = 128 + i * per_vec;
        let scale = f32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]]) as f64;
        let vmin = f32::from_le_bytes([b[off + 4], b[off + 5], b[off + 6], b[off + 7]]) as f64;
        let blob = &b[off + 8..off + per_vec];
        codes.extend_from_slice(blob);
        let (mut sc, mut sc2) = (0f64, 0f64);
        for j in 0..dim {
            let c = ((blob[(j * BITS) / 8] >> ((j * BITS) % 8)) & (LEVELS as u8)) as f64;
            sc += c;
            sc2 += c * c;
        }
        // |row| from the moments: no float vector is ever materialized.
        let n = dim as f64;
        let norm = (n * vmin * vmin + 2.0 * vmin * scale * sc + scale * scale * sc2).sqrt();
        rows.push(Row { scale, vmin, sum_c: sc, sum_c2: sc2, norm });
    }
    Some(Store {
        path: path.display().to_string(),
        bytes: md.len(),
        dim,
        keys,
        codes,
        rows,
        per_vec,
        stride,
    })
}

/// The packed store, reloaded whenever its file has changed since it was last read.
pub fn store() -> Option<std::sync::Arc<Store>> {
    static S: crate::file_cache::Slot<Store> = std::sync::Mutex::new(None);
    let p = store_path()?;
    crate::file_cache::get(&S, &p, |p| {
        load(p).ok_or_else(|| format!("packed store rejected: {}", p.display()))
    })
    .ok()
}

/// Load a specific packed store by repo-relative path (e.g. "swarm_v2_nand_fabric.realmodel.turbovec"),
/// so the same on-silicon engine can serve any distilled store, not just the default zero-LLM fabric.
pub fn store_at(rel: &str) -> Option<Store> {
    let p = crate::k3_retrieval::repo_relative(rel)?;
    load(&p)
}

/// The ChEMBL co-processor's packed store (2,897,805 compounds), reloaded when its file changes.
/// Same TURBOVEC layout + native-turbovec-hdc encoder as the fabric store (measured on this host).
pub fn chembl_store() -> Option<std::sync::Arc<Store>> {
    static S: crate::file_cache::Slot<Store> = std::sync::Mutex::new(None);
    let p = CHEMBL_STORE_RELS
        .iter()
        .find_map(|r| crate::k3_retrieval::repo_relative(r))?;
    crate::file_cache::get(&S, &p, |p| {
        load(p).ok_or_else(|| format!("chembl store rejected: {}", p.display()))
    })
    .ok()
}

/// Vector-search the ChEMBL store — the same packed-domain engine as the fabric scan (GPU fast path
/// + CPU fallback), over the 2.9M-compound store, same encoder.
pub fn chembl_search(qv: &[f32], k: usize) -> Option<Out> {
    let s = chembl_store()?;
    search_in(&s, qv, k)
}

/// The structured tier's packed text store (ethics + prose, ~97,480 docs), reloaded when its file changes.
pub fn structured_store() -> Option<std::sync::Arc<Store>> {
    static S: crate::file_cache::Slot<Store> = std::sync::Mutex::new(None);
    let p = STRUCTURED_STORE_RELS
        .iter()
        .find_map(|r| crate::k3_retrieval::repo_relative(r))?;
    crate::file_cache::get(&S, &p, |p| {
        load(p).ok_or_else(|| format!("structured store rejected: {}", p.display()))
    })
    .ok()
}

/// Vector-search the structured text store (ethics + prose), same engine + encoder.
pub fn structured_search(qv: &[f32], k: usize) -> Option<Out> {
    let s = structured_store()?;
    search_in(&s, qv, k)
}

/// The bioassay packed store (tox21 + binding, ~597,430 SMILES/family records), reloaded when its file changes.
pub fn bioassay_store() -> Option<std::sync::Arc<Store>> {
    static S: crate::file_cache::Slot<Store> = std::sync::Mutex::new(None);
    let p = BIOASSAY_STORE_RELS
        .iter()
        .find_map(|r| crate::k3_retrieval::repo_relative(r))?;
    crate::file_cache::get(&S, &p, |p| {
        load(p).ok_or_else(|| format!("bioassay store rejected: {}", p.display()))
    })
    .ok()
}

/// Vector-search the bioassay store (tox21 + binding), same engine + encoder.
pub fn bioassay_search(qv: &[f32], k: usize) -> Option<Out> {
    let s = bioassay_store()?;
    search_in(&s, qv, k)
}

/// A query reduced to the store's own alphabet, with the moments its cosine needs.
pub struct Query {
    dim: usize,
    codes: Vec<u8>,
    qmin: f64,
    qscale: f64,
    sum_q: f64,
    norm: f64,
}

impl Query {
    pub fn dim(&self) -> usize {
        self.dim
    }
    pub fn codes(&self) -> &[u8] {
        &self.codes
    }
    pub fn qmin(&self) -> f64 {
        self.qmin
    }
    pub fn qscale(&self) -> f64 {
        self.qscale
    }
    pub fn sum_q(&self) -> f64 {
        self.sum_q
    }
    pub fn norm(&self) -> f64 {
        self.norm
    }
}

/// Same per-vector min..max rule the builder used for the rows, so the query is scored in the
/// alphabet it is being compared against rather than in some other scale.
pub fn quantize(qv: &[f32], dim: usize) -> Option<Query> {
    if qv.len() != dim {
        return None;
    }
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for &x in qv {
        if !x.is_finite() {
            return None;
        }
        if x < lo {
            lo = x;
        }
        if x > hi {
            hi = x;
        }
    }
    let span = f64::from(hi - lo);
    let qscale = if span > 0.0 { span / LEVELS as f64 } else { 0.0 };
    let mut codes = vec![0u8; (dim * BITS + 7) / 8];
    let (mut sum_q, mut sum_q2) = (0f64, 0f64);
    for (j, &x) in qv.iter().enumerate() {
        let c = if qscale > 0.0 {
            ((f64::from(x) - f64::from(lo)) / qscale).round()
        } else {
            0.0
        };
        let c = (c as i32).clamp(0, LEVELS) as u8;
        codes[(j * BITS) / 8] |= (c & LEVELS as u8) << ((j * BITS) % 8);
        let cf = f64::from(c);
        sum_q += cf;
        sum_q2 += cf * cf;
    }
    let n = dim as f64;
    let qmin = f64::from(lo);
    let norm = (n * qmin * qmin + 2.0 * qmin * qscale * sum_q + qscale * qscale * sum_q2).sqrt();
    Some(Query { dim, codes, qmin, qscale, sum_q, norm })
}

pub struct Hit {
    pub key: String,
    pub score: f64,
    pub rank: usize,
}

pub struct Out {
    pub hits: Vec<Hit>,
    pub verdict: i8,
    pub basis: String,
    pub scanned: usize,
    pub us: f64,
    pub spread: f64,
    pub store: Value,
}

/// Score the whole store in the packed domain and rank it.
pub fn search(qv: &[f32], k: usize) -> Option<Out> {
    let s = store()?;
    search_in(&*s, qv, k)
}

/// The CPU packed scan: the byte-for-byte reference the GPU kernel is parity-tested against.
/// Scores the whole store and returns (cosine, row_index) in row order.
pub fn cpu_scan_in(s: &Store, q: &Query) -> Vec<(f64, usize)> {
    let mut sc: Vec<(f64, usize)> = Vec::with_capacity(s.keys.len());
    for (i, row) in s.rows.iter().enumerate() {
        let bytes = &s.codes[i * s.stride..(i + 1) * s.stride];
        let mut p = 0f64;
        for (bi, &cb) in bytes.iter().enumerate() {
            let qb = q.codes[bi];
            if qb == 0 {
                continue; // no query mass in these four dimensions
            }
            let (mut a, mut b) = (qb, cb);
            for _ in 0..4 {
                p += f64::from(a & 3) * f64::from(b & 3);
                a >>= 2;
                b >>= 2;
            }
        }
        let dot = (s.dim as f64) * q.qmin * row.vmin
            + q.qscale * row.vmin * q.sum_q
            + q.qmin * row.scale * row.sum_c
            + q.qscale * row.scale * p;
        let cos = if row.norm > 0.0 { dot / (q.norm * row.norm) } else { 0.0 };
        sc.push((cos, i));
    }
    sc
}

/// The packed-domain scan over an already-loaded store, so the same engine serves any distilled
/// store (any encoder, any dim) — not just the default zero-LLM fabric.
pub fn search_in(s: &Store, qv: &[f32], k: usize) -> Option<Out> {
    let q = quantize(qv, s.dim)?;
    if s.is_empty() || q.norm <= 0.0 {
        return None;
    }
    let t0 = std::time::Instant::now();
    // GPU fast path (parity-verified): score the whole store on the GPU, reusing the persistent
    // device buffers. On any failure (no device, store changed, shader error) fall back to the CPU
    // loop below, which is byte-for-byte the reference the GPU kernel is parity-tested against.
    let mut scored: Vec<(f64, usize)> = match crate::gpu_compute::gpu_scan_scores_cached(&q, s) {
        Some(scores) => scores.into_iter().enumerate().map(|(i, sc)| (sc as f64, i)).collect(),
        None => cpu_scan_in(s, &q),
    };
    let us = t0.elapsed().as_secs_f64() * 1e6;
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let kk = k.max(1).min(scored.len());
    let hits: Vec<Hit> = scored
        .iter()
        .take(kk)
        .enumerate()
        .map(|(r, (sc, i))| Hit { key: s.keys[*i].clone(), score: *sc, rank: r + 1 })
        .collect();
    let (verdict, basis, spread) = judge(&scored);
    Some(Out { hits, verdict, basis, scanned: scored.len(), us, spread, store: s.header() })
}

/// The tri-state, derived from the score spread rather than asserted: True only when the leader
/// clears the runner-up by more than the band the corpus itself exhibits.
fn judge(scored: &[(f64, usize)]) -> (i8, String, f64) {
    let n = scored.len();
    if n == 0 {
        return (0, "empty store: nothing to judge".to_string(), 0.0);
    }
    let mean = scored.iter().map(|x| x.0).sum::<f64>() / n as f64;
    let sd = (scored.iter().map(|x| (x.0 - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
    let best = scored[0].0;
    let second = if n > 1 { scored[1].0 } else { f64::NEG_INFINITY };
    let margin = best - second;
    let band = 0.02f64.max(sd);
    if best <= 0.0 {
        (-1, format!("False: best packed-domain cosine {best:.4} <= 0 over {n} documents (mean {mean:.4}, sd {sd:.4})"), sd)
    } else if margin > band {
        (1, format!("True: best {best:.4} leads {second:.4} by {margin:.4} > band {band:.4} (= max(0.02, sd {sd:.4}) over {n} documents)"), sd)
    } else {
        (0, format!("Unknown: best {best:.4} leads runner-up {second:.4} by {margin:.4}, inside band {band:.4} (= max(0.02, sd {sd:.4}) over {n} documents), so the ranking does not separate the top two"), sd)
    }
}

impl Store {
    /// Row index for a key, so a caller never assumes the packed order matches another store's.
    pub fn index_of(&self, key: &str) -> Option<usize> {
        self.keys.iter().position(|k| k == key)
    }

    /// The slow path, deliberately: dequantize one row to f32 and score it. Sharing no code with the
    /// integer scan is the point -- it can only agree with the fast path if both are right.
    pub fn f32_cosine(&self, qv: &[f32], i: usize) -> Option<f64> {
        if i >= self.rows.len() || qv.len() != self.dim {
            return None;
        }
        let row = &self.rows[i];
        let bytes = &self.codes[i * self.stride..(i + 1) * self.stride];
        let (mut dot, mut nq, mut nr) = (0f64, 0f64, 0f64);
        for j in 0..self.dim {
            let c = ((bytes[(j * BITS) / 8] >> ((j * BITS) % 8)) & (LEVELS as u8)) as f64;
            let v = row.vmin + c * row.scale;
            let x = f64::from(qv[j]);
            dot += x * v;
            nq += x * x;
            nr += v * v;
        }
        if nq > 0.0 && nr > 0.0 {
            Some(dot / (nq.sqrt() * nr.sqrt()))
        } else {
            None
        }
    }
}

/// The quantized query put back into f32, so the slow path can be given exactly what the packed
/// scan scored (otherwise the comparison would differ by the quantization itself, not by the method).
pub fn dequant_query(q: &Query) -> Vec<f32> {
    let mut v = Vec::with_capacity(q.dim);
    for j in 0..q.dim {
        let c = ((q.codes[(j * BITS) / 8] >> ((j * BITS) % 8)) & (LEVELS as u8)) as f64;
        v.push((q.qmin + c * q.qscale) as f32);
    }
    v
}

/// Score the store against a query already in the platform's own ternary alphabet
/// (`k3_retrieval`'s engine quantizer emits trits), instead of re-quantizing it with the store's
/// 4-level min..max rule.
///
/// MEASURED, and it does NOT close the gap: a 4-level code cannot carry the engine's zero trit, so
/// the sign expansion is a different (sign-only) representation rather than the same one packed
/// smaller. `?verify=1` measures the gap on every request -- it rebuilds the engine's own integer sum
/// from the engine's trit corpus beside this path's score, at tolerance 0. Against the 180-row store
/// those figures were 6/10 to 7/10 top-1 agreement and 29-56 integer units, recorded in
/// backups/checkpoint_20260920_122636/CHECKPOINT.json and deliberately not repeated here: the store
/// has since been rebuilt from the full 2180-document corpus, so a count frozen in a comment would
/// be a number where a measurement belongs.
///
/// A store that ranks identically has to be written in the engine's ternary alphabet in the first
/// place. That store now exists -- `TerStore` (`.tercode`), written from the engine's own trits at 2
/// bits per trit -- and `search_tercode` is the path that reproduces the engine's sums exactly. This
/// path must not be described as a drop-in for the engine's ranking.
pub fn search_trits(qtrits: &[i8], k: usize) -> Option<Out> {
    let s = store()?;
    if s.is_empty() || qtrits.len() != s.dim {
        return None;
    }
    let t0 = std::time::Instant::now();
    let mut scored: Vec<(f64, usize)> = Vec::with_capacity(s.keys.len());
    for (i, _row) in s.rows.iter().enumerate() {
        let bytes = &s.codes[i * s.stride..(i + 1) * s.stride];
        let mut acc = 0i64;
        for (j, &qt) in qtrits.iter().enumerate() {
            if qt == 0 {
                continue;
            }
            let c = (bytes[(j * BITS) / 8] >> ((j * BITS) % 8)) & (LEVELS as u8);
            let rt = if c >= 2 { 1i64 } else { -1i64 };
            acc += i64::from(qt) * rt;
        }
        scored.push((acc as f64, i));
    }
    let us = t0.elapsed().as_secs_f64() * 1e6;
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let kk = k.max(1).min(scored.len());
    let hits: Vec<Hit> = scored
        .iter()
        .take(kk)
        .enumerate()
        .map(|(r, (sc, i))| Hit { key: s.keys[*i].clone(), score: *sc, rank: r + 1 })
        .collect();
    let (verdict, basis, spread) = judge(&scored);
    Some(Out {
        hits,
        verdict,
        basis: format!("[trit alphabet] {basis}"),
        scanned: scored.len(),
        us,
        spread,
        store: s.header(),
    })
}

// ── The tercode store: the engine's own trits, packed 4x smaller ─────────────────────────────
//
// `.turbovec` is min..max quantized, so its 4-level codes cannot carry the engine's zero trit --
// measured, scoring it in the engine's alphabet still only reached 7/10 top-1 agreement with a
// 29-56 unit divergence against the engine's corpus. This store removes the representation
// difference instead of trying to score around it: it is written *from the engine's own trit
// corpus*, so an identical ranking is a property of the construction, not a hope, while each trit
// costs 2 bits instead of the 8 the engine's i8 corpus uses (4x smaller; 16x smaller than f32).
//
// Layout: 128-byte header, then `vectors * stride` code bytes, then the keys JSON.
//   0..8 magic "TRITCODE" | 8..12 version | 12..16 dim | 16..20 vectors | 20..24 bits
//   24..28 stride | 32..40 keys_off | 40..44 keys_len | 48..56 payload FNV-1a-64
// The header carries FNV-1a-64 rather than SHA-256 and says so: this crate has no SHA-256, and a
// checksum named honestly is worth more than a hash name borrowed from somewhere else.
const TERCODE_REL: &str = "swarm_v2_nand_fabric.tercode";
const TER_MAGIC: &[u8; 8] = b"TRITCODE";

fn fnv1a64(b: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &x in b {
        h ^= u64::from(x);
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

/// Code for a trit: -1 -> 0, 0 -> 1, +1 -> 2. Code 3 is never written.
fn trit_code(t: i8) -> u8 {
    if t > 0 {
        2
    } else if t < 0 {
        0
    } else {
        1
    }
}

fn code_trit(c: u8) -> i8 {
    match c & 3 {
        2 => 1,
        0 => -1,
        _ => 0,
    }
}

/// Write the store from the engine's corpus. Returns (bytes written, rows, payload FNV-1a-64).
pub fn write_tercode(
    path: &std::path::Path,
    keys: &[String],
    corpus: &[i8],
    dim: usize,
) -> Result<(u64, usize, u64), String> {
    if dim == 0 || corpus.len() != keys.len() * dim {
        return Err(format!(
            "corpus is {} trits but {} keys x {dim} dims needs {}",
            corpus.len(),
            keys.len(),
            keys.len() * dim
        ));
    }
    let stride = (dim * BITS + 7) / 8;
    let mut codes = vec![0u8; keys.len() * stride];
    for (i, row) in corpus.chunks(dim).enumerate() {
        for (j, &t) in row.iter().enumerate() {
            codes[i * stride + (j * BITS) / 8] |= trit_code(t) << ((j * BITS) % 8);
        }
    }
    let keys_json = serde_json::to_vec(&serde_json::json!({ "keys": keys })).map_err(|e| e.to_string())?;
    let mut out = vec![0u8; 128];
    out[0..8].copy_from_slice(TER_MAGIC);
    out[8..12].copy_from_slice(&1u32.to_le_bytes());
    out[12..16].copy_from_slice(&(dim as u32).to_le_bytes());
    out[16..20].copy_from_slice(&(keys.len() as u32).to_le_bytes());
    out[20..24].copy_from_slice(&(BITS as u32).to_le_bytes());
    out[24..28].copy_from_slice(&(stride as u32).to_le_bytes());
    out[32..40].copy_from_slice(&((128 + codes.len()) as u64).to_le_bytes());
    out[40..44].copy_from_slice(&(keys_json.len() as u32).to_le_bytes());
    let sum = fnv1a64(&codes);
    out[48..56].copy_from_slice(&sum.to_le_bytes());
    out.extend_from_slice(&codes);
    out.extend_from_slice(&keys_json);
    std::fs::write(path, &out).map_err(|e| format!("writing {}: {e}", path.display()))?;
    Ok((out.len() as u64, keys.len(), sum))
}

pub struct TerStore {
    pub path: String,
    pub bytes: u64,
    pub keys: Vec<String>,
    pub dim: usize,
    pub fnv: u64,
    codes: Vec<u8>,
    stride: usize,
}

fn terstore_path() -> Option<std::path::PathBuf> {
    crate::k3_retrieval::repo_relative(TERCODE_REL)
}

fn load_tercode(path: &std::path::Path) -> Option<TerStore> {
    let md = std::fs::metadata(path).ok()?;
    let b = std::fs::read(path).ok()?;
    if b.len() < 128 || &b[0..8] != TER_MAGIC {
        return None;
    }
    let dim = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
    let vectors = u32::from_le_bytes([b[16], b[17], b[18], b[19]]) as usize;
    let bits = u32::from_le_bytes([b[20], b[21], b[22], b[23]]) as usize;
    let stride = u32::from_le_bytes([b[24], b[25], b[26], b[27]]) as usize;
    let keys_off = u64::from_le_bytes([b[32], b[33], b[34], b[35], b[36], b[37], b[38], b[39]]) as usize;
    let keys_len = u32::from_le_bytes([b[40], b[41], b[42], b[43]]) as usize;
    let fnv = u64::from_le_bytes([b[48], b[49], b[50], b[51], b[52], b[53], b[54], b[55]]);
    if bits != BITS || stride != (dim * BITS + 7) / 8 || keys_off + keys_len > b.len() {
        return None;
    }
    let end = 128 + vectors * stride;
    if end > b.len() {
        return None;
    }
    let codes = b[128..end].to_vec();
    // An unverifiable store is not a store: the checksum is checked before anything is scored.
    if fnv1a64(&codes) != fnv {
        return None;
    }
    let keys: Vec<String> = serde_json::from_slice::<Value>(&b[keys_off..keys_off + keys_len])
        .ok()?
        .get("keys")?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    if keys.len() != vectors {
        return None;
    }
    Some(TerStore { path: path.display().to_string(), bytes: md.len(), keys, dim, fnv, codes, stride })
}

/// The tercode store, reloaded whenever its file has changed since it was last read.
pub fn terstore() -> Option<std::sync::Arc<TerStore>> {
    static T: crate::file_cache::Slot<TerStore> = std::sync::Mutex::new(None);
    let p = terstore_path()?;
    crate::file_cache::get(&T, &p, |p| {
        load_tercode(p).ok_or_else(|| format!("tercode store rejected: {}", p.display()))
    })
    .ok()
}

/// Seconds since the Unix epoch for a file's mtime, or 0 when it cannot be read.
fn mtime_unix(p: &std::path::Path) -> u64 {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Is this store older than the corpus index it was built from?
///
/// The store is a build artifact: the index can be re-encoded and the store would keep serving the
/// previous ranking, silently, because nothing about a stale store looks broken. Reported rather than
/// auto-rebuilt -- rebuilding under a reader would be a different bug.
pub fn terstore_staleness() -> Option<(u64, u64, bool)> {
    let store = crate::k3_retrieval::repo_relative(TERCODE_REL)?;
    let index = crate::k3_retrieval::repo_relative("swarm_v2_artifacts/kernel_knowledge_index.json")?;
    let (s, i) = (mtime_unix(&store), mtime_unix(&index));
    Some((s, i, i > s))
}

/// Score with the engine's own basis: the number of positions where the query and the row share a
/// **non-zero** trit, ties broken by ascending candidate index -- the rule
/// `rust_ternary_index_search` documents. Because this store holds those same trits, the ranking is
/// identical by construction; `?verify=1` on the route checks that against the engine itself.
pub fn search_tercode(qtrits: &[i8], k: usize) -> Option<Out> {
    let s = terstore()?;
    if qtrits.len() != s.dim || s.keys.is_empty() {
        return None;
    }
    let t0 = std::time::Instant::now();
    let mut scored: Vec<(f64, usize)> = Vec::with_capacity(s.keys.len());
    for i in 0..s.keys.len() {
        let bytes = &s.codes[i * s.stride..(i + 1) * s.stride];
        let mut agree = 0i64;
        for (j, &qt) in qtrits.iter().enumerate() {
            if qt == 0 {
                continue;
            }
            if code_trit(bytes[(j * BITS) / 8] >> ((j * BITS) % 8)) == qt {
                agree += 1;
            }
        }
        scored.push((agree as f64, i));
    }
    let us = t0.elapsed().as_secs_f64() * 1e6;
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1))
    });
    let kk = k.max(1).min(scored.len());
    let hits: Vec<Hit> = scored
        .iter()
        .take(kk)
        .enumerate()
        .map(|(r, (sc, i))| Hit { key: s.keys[*i].clone(), score: *sc, rank: r + 1 })
        .collect();
    let (verdict, basis, spread) = judge(&scored);
    Some(Out {
        hits,
        verdict,
        basis: format!("[tercode: engine trits, 2 bits each] {basis}"),
        scanned: scored.len(),
        us,
        spread,
        store: {
            // The file's signature read now, beside the loaded store's own bytes: a rebuilt store that
            // the process has not picked up shows here as the two disagreeing.
            let file_sig = crate::file_cache::Sig::of(std::path::Path::new(&s.path));
            let loaded = s.bytes as u64;
            let (stale, why) = match terstore_staleness() {
                Some((sm, im, true)) => (
                    true,
                    format!("STALE: the corpus index is newer (index mtime {im} > store mtime {sm}) -- rebuild with /k3/tercode/build"),
                ),
                Some((sm, im, false)) => (
                    false,
                    format!("built from the corpus index as it stands (index mtime {im} <= store mtime {sm})"),
                ),
                None => (false, "staleness unknown: store or index mtime unreadable".to_string()),
            };
            serde_json::json!({
                "path": s.path, "bytes": s.bytes, "vectors": s.keys.len(), "dim": s.dim,
                "bits_per_trit": BITS, "payload_fnv1a64": format!("{:016x}", s.fnv),
                "vs_engine_i8_corpus": format!("{} bytes packed vs {} bytes as i8", s.codes.len(), s.keys.len() * s.dim),
                "stale": stale, "freshness": why,
                "file_bytes": file_sig.map(|g| g.len), "file_mtime": file_sig.map(|g| g.mtime_s()),
                "serves_the_file_on_disk": file_sig.map(|g| g.len == loaded).unwrap_or(false),
            })
        },
    })
}

/// `trit` names, so a caller never re-invents the mapping.
pub fn trit_name(t: i8) -> &'static str {
    match t {
        1 => "True",
        -1 => "False",
        _ => "Unknown",
    }
}
