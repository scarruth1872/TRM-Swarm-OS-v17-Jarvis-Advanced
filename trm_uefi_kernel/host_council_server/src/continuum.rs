//! Monad continuum — the index realised in the kernel, on the faceted 4D-torus substrate.
//!
//! WHAT THIS IS
//! ------------
//! The 4D torus (`trm_uefi_kernel::faceted_torus_ternary`) is the continuum on which the platform's
//! states are placed: one `FacetedTrit` is `round(clamp(v,-1,1)*108)` and sits on a 217-state axis
//! (`axis_index`), which embeds into S^3 as (r cos θ, r sin θ, r cos φ, r sin φ). That is the substrate
//! for realisation — placement and phase — and this module realises the council's evidence on it.
//!
//! WHAT IT IS NOT
//! --------------
//! Measured 2026-11-24 (docs/TRM_Torus_Architecture_2026-11-24.md), a torus point carries two numbers and
//! 2–4 numbers cannot rank 768-dim neighbours: torus-distance retrieval scored recall@10 0.038–0.044
//! against cosine, and unquantised 4-PC controls did no better (0.081), so the loss is dimensionality and
//! not the Clifford map. Therefore:
//!   * **placement** uses the continuum — each chunk gets a cell (0..216) from its dominant facet axis;
//!   * **similarity** is the integer dot of facet rows, never a torus distance.
//!
//! Facet storage keeps 0.944 recall@10 of the f32 ranking at 4.0x less space (7.48 MB -> 1.87 MB), and
//! the dot accumulates in i32, which is exactly what a kernel can do without floating point.
//!
//! FORMAT (written by tools/export_continuum.py)
//!   "K3CX" | u32 version | u32 n | u32 dim | n*dim int8 facets   (row-major, no per-row header)
//!   data/continuum_<tag>_chunks.jsonl : {"i","source","chunk","text"} per line

use serde_json::{json, Value};
use std::sync::OnceLock;

pub const MAGIC: &[u8; 4] = b"K3CX";
pub const VERSION: u32 = 1;
pub const MAX_FACET: i16 = 108;
pub const TOTAL_STATES: usize = 217;

struct ChunkMeta {
    source: String,
    text: String,
}

pub struct Continuum {
    n: usize,
    dim: usize,
    /// n*dim int8 facets, row-major.
    facets: Vec<i8>,
    /// Per-row sqrt(sum of squares of the i8 facets). Present so the evidence path can keep using the
    /// SAME 0.30 similarity floor as the f32 retriever: ranking stays integer, the floor stays comparable.
    row_norm: Vec<f32>,
    meta: Vec<ChunkMeta>,
    path: String,
}

/// The exact rule of `FacetedTrit::from_f32`, applied to a query vector.
pub fn to_facet(v: f32) -> i8 {
    let c = v.clamp(-1.0, 1.0);
    (c * MAX_FACET as f32).round() as i8
}

/// Axis index of the largest-magnitude facet: the chunk's cell on the continuum.
/// Placement rule only — it is deliberately not used for ranking.
pub fn cell_of(row: &[i8]) -> usize {
    let mut best = 0usize;
    let mut bestm = -1i16;
    for (i, &t) in row.iter().enumerate() {
        let m = (t as i16).abs();
        if m > bestm {
            bestm = m;
            best = i;
        }
    }
    let facet = row[best] as i16;
    ((facet + MAX_FACET) as usize).min(TOTAL_STATES - 1)
}

fn data_dir() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("TRM_CONTINUUM_DIR") {
        let pb = std::path::PathBuf::from(p);
        if pb.is_dir() {
            return Some(pb);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf())?;
        for _ in 0..5 {
            let cand = dir.join("data").join("continuum_v3.bin");
            if cand.exists() {
                return Some(dir.join("data"));
            }
            if !dir.pop() {
                break;
            }
        }
    }
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..5 {
        let cand = dir.join("data").join("continuum_v3.bin");
        if cand.exists() {
            return Some(dir.join("data"));
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

impl Continuum {
    /// Load the K3CX artefact and its chunk metadata. Returns None on ANY structural problem —
    /// a partially loaded continuum would answer confidently with the wrong architecture, so it is
    /// refused rather than degraded.
    pub fn load() -> Option<Self> {
        let dir = data_dir()?;
        let bin = dir.join("continuum_v3.bin");
        let raw = std::fs::read(&bin).ok()?;
        if raw.len() < 16 || &raw[0..4] != MAGIC {
            eprintln!("[continuum] bad magic in {}", bin.display());
            return None;
        }
        let ver = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let n = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
        let dim = u32::from_le_bytes([raw[12], raw[13], raw[14], raw[15]]) as usize;
        if ver != VERSION || n == 0 || dim == 0 || n > 200_000 || dim > 4096 {
            eprintln!("[continuum] rejected header v{ver} n={n} dim={dim}");
            return None;
        }
        if raw.len() != 16 + n * dim {
            eprintln!("[continuum] size mismatch: {} != 16+{}*{}", raw.len(), n, dim);
            return None;
        }
        let facets: Vec<i8> = raw[16..].iter().map(|&b| b as i8).collect();

        let mut meta: Vec<ChunkMeta> = Vec::with_capacity(n);
        let jsonl = dir.join("continuum_v3_chunks.jsonl");
        if let Ok(txt) = std::fs::read_to_string(&jsonl) {
            for line in txt.lines() {
                if let Ok(v) = serde_json::from_str::<Value>(line) {
                    meta.push(ChunkMeta {
                        source: v.get("source").and_then(|x| x.as_str()).unwrap_or("?").to_string(),
                        text: v.get("text").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                    });
                }
            }
        }
        if meta.len() != n {
            eprintln!("[continuum] metadata rows {} != index rows {}", meta.len(), n);
            return None;
        }
        let mut row_norm = Vec::with_capacity(n);
        for i in 0..n {
            let mut s = 0f32;
            for j in 0..dim {
                let v = facets[i * dim + j] as f32;
                s += v * v;
            }
            row_norm.push(s.sqrt());
        }
        eprintln!("[continuum] loaded {} x {} facets ({:.2} MB) from {}",
                  n, dim, raw.len() as f64 / 1e6, bin.display());
        Some(Continuum { n, dim, facets, row_norm, meta, path: bin.display().to_string() })
    }

    pub fn len(&self) -> usize { self.n }
    pub fn is_empty(&self) -> bool { self.n == 0 }

    /// Integer dot of the query's facet row against chunk `i`. i8 x i8 accumulates in i32 — no floats,
    /// no multiply-add rounding, which is what realises this on the continuum substrate.
    fn dot(&self, q: &[i8], i: usize) -> i32 {
        let row = &self.facets[i * self.dim..(i + 1) * self.dim];
        let mut acc = 0i32;
        for j in 0..self.dim {
            acc += (q[j] as i32) * (row[j] as i32);
        }
        acc
    }

    /// Top-k by integer dot. Returns (score, index).
    pub fn search_facets(&self, q: &[i8], k: usize) -> Vec<(i32, usize)> {
        if q.len() != self.dim {
            return Vec::new();
        }
        let mut scored: Vec<(i32, usize)> = (0..self.n).map(|i| (self.dot(q, i), i)).collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.truncate(k.max(1));
        scored
    }

    pub fn status_json(&self) -> Value {
        let mut cells = vec![0u32; TOTAL_STATES];
        for i in 0..self.n {
            let row = &self.facets[i * self.dim..(i + 1) * self.dim];
            cells[cell_of(row)] += 1;
        }
        let used = cells.iter().filter(|&&c| c > 0).count();
        let fullest = cells.iter().copied().max().unwrap_or(0);
        json!({
            "loaded": true, "path": self.path, "n_chunks": self.n, "dim": self.dim,
            "format": "K3CX v1 (i8 FacetedTrit per dim)",
            "bytes": 16 + self.n * self.dim,
            "monad_anchor": format!("0x{:016X}", trm_uefi_kernel::faceted_torus_ternary::MONAD_TRUTH_ANCHOR),
            "continuum": {
                "states": TOTAL_STATES, "cells_occupied": used, "fullest_cell": fullest,
                "placement_rule": "axis_index of the chunk's largest-|facet| dimension",
                "role": "placement and phase only - similarity is the integer facet dot, never torus distance",
            },
            "measured": {"recall_at_10_vs_f32": 0.944, "top1_vs_f32": 0.938,
                         "note": "docs/TRM_Torus_Architecture_2026-11-24.md"},
        })
    }
}

static CONT: OnceLock<Option<Continuum>> = OnceLock::new();

fn get() -> Option<&'static Continuum> {
    CONT.get_or_init(Continuum::load).as_ref()
}

/// The loaded continuum, if any. Used by the evidence path to rank without touching the f32 JSON.
pub fn loaded() -> Option<&'static Continuum> {
    get()
}

impl Continuum {
    /// Evidence block, same "[source] snippet" shape the council already consumes, but ranked by the
    /// integer facet dot and filtered by the caller's similarity floor.
    ///
    /// The floor is applied to `dot / (|q| * |row|)`, i.e. the cosine of the QUANTISED rows. That keeps
    /// `MIN_SCORE = 0.30` meaning what it meant on the f32 index instead of inventing a new number for
    /// integer scores (which have no fixed scale). Ranking is still decided by the integer dot alone.
    pub fn evidence_for(&self, query: &str, k: usize, floor: f32) -> Option<String> {
        let qv = crate::evidence_index::embed_query_pub(query)?;
        if qv.len() != self.dim {
            return None;
        }
        let qf: Vec<i8> = qv.iter().map(|&v| to_facet(v)).collect();
        let mut qn = 0f32;
        for &v in &qf {
            let f = v as f32;
            qn += f * f;
        }
        let qn = qn.sqrt();
        if qn == 0.0 {
            return None;
        }
        let mut scored: Vec<(f32, usize)> = (0..self.n)
            .map(|i| {
                let d = self.dot(&qf, i) as f32;
                let denom = qn * self.row_norm[i];
                (if denom > 0.0 { d / denom } else { 0.0 }, i)
            })
            .filter(|(s, _)| *s >= floor)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut out = String::new();
        for (_, i) in scored.into_iter().take(k.max(1)) {
            let m = &self.meta[i];
            let snip: String = m.text.chars().take(240).collect();
            out.push_str(&format!("[{}] {}\n", m.source, snip.replace('\n', " ")));
        }
        Some(out)
    }
}

/// Status of the continuum as loaded in this process.
pub fn status() -> Value {
    match get() {
        Some(c) => c.status_json(),
        None => json!({
            "loaded": false,
            "how": "run tools/export_continuum.py --tag v3 to write data/continuum_v3.bin",
            "refused": "no fallback: an empty continuum must not read as 'nothing relevant'",
        }),
    }
}

/// Search: embed the query on :8593, facet-quantise it, integer-dot the rows, return top-k.
pub fn search_json(query: &str, k: usize) -> Value {
    let c = match get() {
        Some(c) => c,
        None => return json!({"error": "continuum not loaded", "hint": "see GET /continuum/status"}),
    };
    if query.trim().is_empty() {
        return json!({"error": "empty query"});
    }
    let qv = match crate::evidence_index::embed_query_pub(query) {
        Some(v) => v,
        None => return json!({"error": "embedder unreachable on :8593"}),
    };
    if qv.len() != c.dim {
        return json!({"error": "dimension mismatch", "query": qv.len(), "index": c.dim});
    }
    let qf: Vec<i8> = qv.iter().map(|&v| to_facet(v)).collect();
    let hits = c.search_facets(&qf, k);
    let results: Vec<Value> = hits.iter().map(|(score, i)| {
        let m = &c.meta[*i];
        let row = &c.facets[*i * c.dim..(*i + 1) * c.dim];
        json!({
            "i": i, "score": score, "cell": cell_of(row),
            "source": m.source,
            "text": m.text.chars().take(240).collect::<String>().replace('\n', " "),
        })
    }).collect();
    json!({
        "query": query, "k": k, "n_scanned": c.n, "dim": c.dim,
        "scoring": "i32 integer dot over i8 facets (217-level)",
        "results": results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth() -> Continuum {
        // three 4-dim rows; row 1 is the exact query, row 2 is its negative, row 0 is orthogonal-ish
        let facets: Vec<i8> = vec![
            10, 0, 0, 0,
            20, -30, 40, 0,
            -20, 30, -40, 0,
        ];
        let meta = (0..3).map(|i| ChunkMeta { source: format!("s{i}"), text: format!("t{i}") }).collect();
        let mut row_norm = Vec::new();
        for i in 0..3 {
            let mut s = 0f32;
            for j in 0..4 {
                let v = facets[i * 4 + j] as f32;
                s += v * v;
            }
            row_norm.push(s.sqrt());
        }
        Continuum { n: 3, dim: 4, facets, row_norm, meta, path: "mem".into() }
    }

    #[test]
    fn exact_row_ranks_first() {
        let c = synth();
        let q = vec![20, -30, 40, 0];
        let hits = c.search_facets(&q, 3);
        assert_eq!(hits[0].1, 1, "the identical row must rank first: {:?}", hits);
        assert!(hits[2].0 < hits[0].0, "the negated row must score lower");
    }

    #[test]
    fn facet_rule_matches_the_kernel() {
        assert_eq!(to_facet(1.0), 108);
        assert_eq!(to_facet(-1.0), -108);
        assert_eq!(to_facet(0.0), 0);
        assert_eq!(to_facet(2.5), 108, "clamped before rounding");
        assert_eq!(to_facet(-0.0046), 0);
    }

    #[test]
    fn cell_is_an_axis_index_in_range() {
        let c = synth();
        for i in 0..c.n {
            let row = &c.facets[i * c.dim..(i + 1) * c.dim];
            assert!(cell_of(row) < TOTAL_STATES);
        }
        // row 1's largest magnitude is +40 at dim 2 -> facet 40 -> index 148
        let row = &c.facets[4..8];
        assert_eq!(cell_of(row), (40 + MAX_FACET) as usize);
    }

    #[test]
    fn wrong_dimension_query_returns_nothing() {
        let c = synth();
        assert!(c.search_facets(&[1, 2, 3], 5).is_empty());
    }
}
