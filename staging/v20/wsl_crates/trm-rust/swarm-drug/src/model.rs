//! swarm-drug QSAR model — the antibacterial-activity predictor ported from
//! `swarm_v2/core/qsar_predictor.py` (MOTION-011).
//!
//! The RandomForest decision is walked natively in Rust. The Morgan fingerprint
//! (RDKit, radius 2, 1024-bit) is computed OFFLINE and passed in as `[u8; 128]`
//! (1024 packed bits, LSB-first) — Python/RDKit owns the fingerprint, Rust owns
//! the runtime decision. The model binary is sha256-gated before load (Shield's
//! binding condition); a mismatched artifact is rejected, never loaded.

use sha2::{Digest, Sha256};

/// The registered model artifact's sha256. The loader rejects anything else.
pub const REGISTERED_SHA256: &str =
    "db15689db21fac26103b17301aca4287faae498593dc6e4a760646ba931ebdfb";

/// Recorded (WSL-style) path to the exported model binary; resolved by swarm-paths.
pub const RECORDED_MODEL: &str = "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main/swarm_v2_artifacts/qsar/qsar_bindingdb_rf_prod.bin";

#[derive(Debug, Clone)]
pub struct Node {
    pub feature: i32,
    pub threshold: f32,
    pub left: i32,
    pub right: i32,
    pub v_inactive: f64,
    pub v_active: f64,
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub struct QsarModel {
    pub trees: Vec<Tree>,
    pub n_bits: u32,
    pub threshold_nm: f64,
    pub scaffold_auc: f64,
}

/// Bit i (0..1024) of a packed 1024-bit fingerprint.
#[inline]
pub fn bit(fp: &[u8], i: usize) -> u8 {
    (fp[i / 8] >> (i % 8)) & 1
}

/// Decode a 256-char hex string into a 128-byte fingerprint.
pub fn parse_fp_hex(s: &str) -> Option<[u8; 128]> {
    if s.len() != 256 {
        return None;
    }
    let mut out = [0u8; 128];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

impl QsarModel {
    /// Walk one tree to a leaf and return its normalized active probability.
    fn predict_tree(&self, tree: &Tree, fp: &[u8; 128]) -> f64 {
        let mut node = 0i32;
        loop {
            let n = &tree.nodes[node as usize];
            if n.feature < 0 {
                return n.v_active / (n.v_inactive + n.v_active);
            }
            let x = bit(fp, n.feature as usize) as f32;
            node = if x <= n.threshold { n.left } else { n.right };
        }
    }

    /// Active probability P(antibacterial activity < threshold_nm), averaged over trees.
    pub fn predict(&self, fp: &[u8; 128]) -> f64 {
        let mut sum = 0.0f64;
        for t in &self.trees {
            sum += self.predict_tree(t, fp);
        }
        sum / self.trees.len() as f64
    }

    /// Load, sha256-gate, and parse the model from `path`.
    pub fn load(path: &str) -> Result<QsarModel, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
        let digest = {
            let mut h = Sha256::new();
            h.update(&bytes);
            format!("{:x}", h.finalize())
        };
        if digest != REGISTERED_SHA256 {
            return Err(format!(
                "sha256 gate FAILED: {digest} != registered {REGISTERED_SHA256}"
            ));
        }
        parse(&bytes)
    }
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn i32_at(b: &[u8], o: usize) -> i32 {
    i32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn f32_at(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn f64_at(b: &[u8], o: usize) -> f64 {
    f64::from_le_bytes(b[o..o + 8].try_into().unwrap())
}

fn parse(bytes: &[u8]) -> Result<QsarModel, String> {
    if bytes.len() < 32 || &bytes[0..4] != b"QSRF" {
        return Err("bad magic (not a QSRF model)".to_string());
    }
    let mut o = 4usize;
    let _version = u32_at(bytes, o); o += 4;
    let n_trees = u32_at(bytes, o); o += 4;
    let n_bits = u32_at(bytes, o); o += 4;
    let threshold_nm = f64_at(bytes, o); o += 8;
    let scaffold_auc = f64_at(bytes, o); o += 8;

    let mut trees = Vec::with_capacity(n_trees as usize);
    for _ in 0..n_trees {
        let node_count = u32_at(bytes, o); o += 4;
        let mut nodes = Vec::with_capacity(node_count as usize);
        for _ in 0..node_count {
            let feature = i32_at(bytes, o); o += 4;
            let threshold = f32_at(bytes, o); o += 4;
            let left = i32_at(bytes, o); o += 4;
            let right = i32_at(bytes, o); o += 4;
            let v_inactive = f64_at(bytes, o); o += 8;
            let v_active = f64_at(bytes, o); o += 8;
            nodes.push(Node { feature, threshold, left, right, v_inactive, v_active });
        }
        trees.push(Tree { nodes });
    }
    Ok(QsarModel { trees, n_bits, threshold_nm, scaffold_auc })
}

#[cfg(test)]
mod tests {
    use super::*;

    // amoxicillin — control #1 (golden: 0.794991942502)
    const AMOX_FP: &str = "220004001200000000000000000000600500080000002000000000000002008c000004000200000400002000100400002800800000000000000000000000000000000000000000200402008008000000000400000a00000220004000000206000000040080000020000002000000002000008200800000000000000000000008";

    fn model() -> QsarModel {
        let p = swarm_paths::path(RECORDED_MODEL);
        let p = p.to_str().expect("model path");
        QsarModel::load(p).expect("load model")
    }

    #[test]
    fn sha256_gate_rejects_a_tampered_artifact() {
        let src = swarm_paths::path(RECORDED_MODEL);
        let bytes = std::fs::read(&src).expect("read model");
        let mut tampered = bytes.clone();
        tampered[100] ^= 0xff; // flip a byte in the tree body
        let tmp = std::env::temp_dir().join("qsar_tampered.bin");
        std::fs::write(&tmp, &tampered).expect("write tampered");
        let err = QsarModel::load(tmp.to_str().unwrap()).expect_err("must reject");
        assert!(err.contains("sha256 gate FAILED"), "{err}");
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn amoxicillin_prediction_matches_python() {
        let m = model();
        assert_eq!(m.n_bits, 1024);
        assert_eq!(m.trees.len(), 80);
        let fp = parse_fp_hex(AMOX_FP).expect("fp hex");
        let p = m.predict(&fp);
        let expected = 0.794991942502f64;
        assert!((p - expected).abs() < 1e-6, "rust={p} python={expected}");
    }
}
