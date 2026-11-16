//! TRM Swarm OS — TurboVec & Ternary Knowledge Fusion (Paper 148)
//!
//! Reports what the platform's own stores hold. Every number is read from a file or measured here:
//! the "shards" below are the packed store's real key table grouped by provenance, not a hand-written
//! domain list, and the latency figures are timed. Nothing in this module ingests, quantizes or seals
//! anything -- the stores are built by swarm_v2/core/build_nand_fabric_index.py from the corpus, and
//! the fuse here is a report, not an act.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use crate::MONAD_TRUTH_ANCHOR;
use crate::unified_resonance_stream::{k3_consensus_of, K3Trit};

#[derive(Debug, Clone)]
pub struct TurboVecQuantizedShard {
    pub shard_id: usize,
    pub domain_name: String,
    pub vector_count: usize,
    pub original_dim: usize,
    pub ternary_trit_count: usize,
    pub recall_latency_us: f64,
    pub cosine_fidelity_pct: f64,
    pub k3_logic_filter: K3Trit,
}

#[derive(Debug, Clone)]
pub struct TurboVecTernaryFusionReport {
    pub total_models_quantized: usize,
    pub total_embeddings_ingested: usize,
    pub total_ternary_trits_generated: usize,
    pub fabric_store_trits: Option<u64>,
    pub shards: Vec<TurboVecQuantizedShard>,
    pub mean_recall_us: f64,
    pub memory_compression_ratio: f64,
    pub mean_cosine_fidelity_pct: f64,
    pub unanimous_k3_truth: K3Trit,
    pub total_pipeline_latency_us: f64,
    pub monad_truth_anchor: u64,
    pub seal_basis: &'static str,
}

fn repo_rel(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(rel)
}

/// (keys, dequantized vectors, dim) straight from the packed store's header -- no side data.
fn load_packed_store() -> Option<(Vec<String>, Vec<Vec<f32>>, usize)> {
    let b = fs::read(repo_rel("swarm_v2_nand_fabric.turbovec")).ok()?;
    if b.len() < 128 || &b[0..8] != b"TURBOVEC" {
        return None;
    }
    let dim = u32::from_le_bytes([b[20], b[21], b[22], b[23]]) as usize;
    let vectors = u32::from_le_bytes([b[24], b[25], b[26], b[27]]) as usize;
    let bits = u32::from_le_bytes([b[28], b[29], b[30], b[31]]) as usize;
    let keys_off = u64::from_le_bytes([b[32], b[33], b[34], b[35], b[36], b[37], b[38], b[39]]) as usize;
    let keys_len = u32::from_le_bytes([b[40], b[41], b[42], b[43]]) as usize;
    if bits != 2 || keys_off + keys_len > b.len() {
        return None;
    }
    let keys: Vec<String> = serde_json::from_slice::<serde_json::Value>(&b[keys_off..keys_off + keys_len])
        .ok()?
        .get("keys")?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let per = 8 + (dim * bits + 7) / 8;
    let mut vecs = Vec::with_capacity(vectors);
    for i in 0..vectors {
        let off = 128 + i * per;
        if off + per > b.len() {
            return None;
        }
        let scale = f32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]]);
        let vmin = f32::from_le_bytes([b[off + 4], b[off + 5], b[off + 6], b[off + 7]]);
        let mut v = Vec::with_capacity(dim);
        for j in 0..dim {
            let byte = b[off + 8 + (j * bits) / 8];
            let code = ((byte >> ((j * bits) % 8)) & 3) as f32;
            v.push(vmin + code * scale);
        }
        vecs.push(v);
    }
    Some((keys, vecs, dim))
}

/// The source embeddings, so the round-trip loss can be measured rather than assumed.
fn load_source_embeddings() -> Option<std::collections::HashMap<String, Vec<f32>>> {
    let b = fs::read(repo_rel("swarm_v2_artifacts/kernel_knowledge_index.json")).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&b).ok()?;
    let idx = v.get("index")?.as_object()?;
    let mut out = std::collections::HashMap::new();
    for (k, rec) in idx {
        if let Some(arr) = rec.get("embedding").and_then(|e| e.as_array()) {
            out.insert(k.clone(), arr.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect());
        }
    }
    Some(out)
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let (mut d, mut na, mut nb) = (0f32, 0f32, 0f32);
    for (x, y) in a.iter().zip(b.iter()) {
        d += x * y;
        na += x * x;
        nb += y * y;
    }
    if na > 0.0 && nb > 0.0 {
        d / (na.sqrt() * nb.sqrt())
    } else {
        0.0
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v[v.len() / 2]
}

pub struct TurboVecTernaryFusionEngine;

impl TurboVecTernaryFusionEngine {
    pub fn execute_turbovec_ternary_fusion() -> TurboVecTernaryFusionReport {
        let t0 = Instant::now();
        let loaded = load_packed_store();
        let sources = load_source_embeddings();
        let models_present: usize = crate::ollama_knowledge_harvester::model_stores().iter().map(|(_, n, _)| *n).sum();
        let fabric_trits = fs::metadata(repo_rel("swarm_v2_nand_fabric.k3")).ok().map(|m| m.len() * 2);

        // Group the store's own keys by provenance -- the real partition, not a declared one.
        let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
        if let Some((keys, _v, _d)) = &loaded {
            for (i, k) in keys.iter().enumerate() {
                let g = if k.starts_with("papers/") {
                    "Corpus: Monad papers (papers/)".to_string()
                } else if k.starts_with("docs/") {
                    "Corpus: platform documentation (docs/)".to_string()
                } else {
                    "Corpus: platform root documents".to_string()
                };
                match groups.iter_mut().find(|(name, _)| *name == g) {
                    Some((_, v)) => v.push(i),
                    None => groups.push((g, vec![i])),
                }
            }
        }

        let mut shards = Vec::new();
        let (mut total_vectors, mut total_trits, mut fidelity_sum, mut fidelity_n) = (0usize, 0usize, 0.0f64, 0usize);
        let mut per_shard_recall: Vec<(usize, f64)> = Vec::new();

        if let Some((keys, vecs, dim)) = &loaded {
            for (sid, (name, idxs)) in groups.iter().enumerate() {
                // fidelity: measured round-trip of every row in this group against its source vector
                let mut fids = Vec::new();
                for &i in idxs {
                    if let (Some(src), Some(rec)) = (sources.as_ref().and_then(|s| s.get(&keys[i])), vecs.get(i)) {
                        fids.push(cosine(src, rec) as f64);
                    }
                }
                let fid_mean = if fids.is_empty() { 0.0 } else { fids.iter().sum::<f64>() / fids.len() as f64 };
                // latency: time a real scan of this group against its own first vector
                let mut times = Vec::new();
                if let Some(q) = idxs.first().and_then(|&i| vecs.get(i)) {
                    for _ in 0..25 {
                        let t = Instant::now();
                        for &i in idxs {
                            let _ = cosine(q, &vecs[i]);
                        }
                        times.push(t.elapsed().as_secs_f64() * 1e6);
                    }
                }
                let recall_us = median(times);
                per_shard_recall.push((idxs.len(), recall_us));
                total_vectors += idxs.len();
                total_trits += idxs.len() * dim;
                fidelity_sum += fid_mean * idxs.len() as f64;
                fidelity_n += idxs.len();
                shards.push(TurboVecQuantizedShard {
                    shard_id: sid + 1,
                    domain_name: name.clone(),
                    vector_count: idxs.len(),
                    original_dim: *dim,
                    ternary_trit_count: idxs.len() * dim,
                    recall_latency_us: recall_us,
                    cosine_fidelity_pct: fid_mean * 100.0,
                    // Nothing evaluates a K3 invariant per group, so the honest per-item verdict is Unknown.
                    k3_logic_filter: k3_consensus_of(&[]),
                });
            }
        }

        let mean_recall = if per_shard_recall.is_empty() {
            0.0
        } else {
            per_shard_recall.iter().map(|(n, t)| *n as f64 * *t).sum::<f64>() / total_vectors.max(1) as f64
        };
        let elapsed = t0.elapsed();

        TurboVecTernaryFusionReport {
            total_models_quantized: models_present,
            total_embeddings_ingested: total_vectors,
            total_ternary_trits_generated: total_trits,
            fabric_store_trits: fabric_trits,
            shards,
            mean_recall_us: mean_recall,
            memory_compression_ratio: 16.0, // property of the format: 32-bit float -> 2-bit code
            mean_cosine_fidelity_pct: if fidelity_n == 0 { 0.0 } else { fidelity_sum / fidelity_n as f64 * 100.0 },
            unanimous_k3_truth: k3_consensus_of(&[]),
            total_pipeline_latency_us: elapsed.as_secs_f64() * 1e6,
            monad_truth_anchor: MONAD_TRUTH_ANCHOR,
            seal_basis: "no seal is computed here; the store header carries the SHA-256 over payload+keys (check: swarm_v2/core/verify_nand_fabric_index.py)",
        }
    }
}
