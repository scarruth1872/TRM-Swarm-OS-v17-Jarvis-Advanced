//! swarm-deepdta — native Rust DeepDTA forward pass (fine-tuned Mpro model).
//! Walks the exported CNN: embedding -> conv1 -> conv2 -> conv3 -> ReLU -> global maxpool -> MLP.
//! Weights are sha256-gated before load (Shield's binding condition): a mismatched artifact is rejected.
use sha2::{Digest, Sha256};

pub const REGISTERED_SHA256: &str = "77300fe6829ddd308657feeedfefdaabd0b9c1d9472ecad7bc369cf91230c7f2";
pub const RECORDED_MODEL: &str = "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v19-Sovereign-Private-main/TRM-Swarm-OS-v19-Sovereign-Private-main/swarm_v2_artifacts/deepdta/deepdta_mpro_ft.bin";

pub const LIG_VOCAB: usize = 37;
pub const PRO_VOCAB: usize = 24;
const CHANNEL: usize = 32;
const K: usize = 8;
pub const SEQLEN: usize = 2000;
pub const SMILEN: usize = 200;

#[inline]
fn relu(x: f32) -> f32 { if x > 0.0 { x } else { 0.0 } }

fn conv_layer(input: &[f32], seq: usize, weight: &[f32], bias: &[f32], out_c: usize, in_c: usize) -> Vec<f32> {
    let out_len = seq - K + 1;
    let mut out = vec![0.0f32; out_len * out_c];
    for o in 0..out_len {
        for c in 0..out_c {
            let mut acc = bias[c];
            for k in 0..K {
                for d in 0..in_c {
                    acc += input[(o + k) * in_c + d] * weight[(c * in_c + d) * K + k];
                }
            }
            out[o * out_c + c] = acc;
        }
    }
    out
}

fn relu_maxpool(x: &[f32], seq: usize, out_c: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; out_c];
    for c in 0..out_c {
        let mut m = relu(x[c]);
        for o in 1..seq {
            let v = relu(x[o * out_c + c]);
            if v > m { m = v; }
        }
        out[c] = m;
    }
    out
}

fn mlp(x: &[f32], wt: &[f32], b: &[f32], out_c: usize) -> Vec<f32> {
    let in_c = x.len();
    let mut out = vec![0.0f32; out_c];
    for c in 0..out_c {
        let mut acc = b[c];
        for d in 0..in_c { acc += x[d] * wt[c * in_c + d]; }
        out[c] = relu(acc);
    }
    out
}

struct Weights {
    lig_emb: Vec<f32>, pro_emb: Vec<f32>,
    lig_c1w: Vec<f32>, lig_c1b: Vec<f32>, lig_c2w: Vec<f32>, lig_c2b: Vec<f32>, lig_c3w: Vec<f32>, lig_c3b: Vec<f32>,
    pro_c1w: Vec<f32>, pro_c1b: Vec<f32>, pro_c2w: Vec<f32>, pro_c2b: Vec<f32>, pro_c3w: Vec<f32>, pro_c3b: Vec<f32>,
    fc1w: Vec<f32>, fc1b: Vec<f32>, fc2w: Vec<f32>, fc2b: Vec<f32>,
    fc3w: Vec<f32>, fc3b: Vec<f32>, fc4w: Vec<f32>, fc4b: Vec<f32>,
}

fn load_weights(bin: &[u8]) -> Weights {
    let f: Vec<f32> = bin.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
    let mut o = 0;
    let mut take = |n: usize| { let s = o; o += n; f[s..o].to_vec() };
    let fc1b = take(1024); let fc1w = take(1024 * 192);
    let fc2b = take(1024); let fc2w = take(1024 * 1024);
    let fc3b = take(512);  let fc3w = take(512 * 1024);
    let fc4b = take(1);    let fc4w = take(1 * 512);
    let lig_c1b = take(CHANNEL);       let lig_c1w = take(CHANNEL * 128 * K);
    let lig_c2b = take(CHANNEL * 2);   let lig_c2w = take(CHANNEL * 2 * CHANNEL * K);
    let lig_c3b = take(CHANNEL * 3);   let lig_c3w = take(CHANNEL * 3 * CHANNEL * 2 * K);
    let lig_emb = take(LIG_VOCAB * 128);
    let pro_c1b = take(CHANNEL);       let pro_c1w = take(CHANNEL * 128 * K);
    let pro_c2b = take(CHANNEL * 2);   let pro_c2w = take(CHANNEL * 2 * CHANNEL * K);
    let pro_c3b = take(CHANNEL * 3);   let pro_c3w = take(CHANNEL * 3 * CHANNEL * 2 * K);
    let pro_emb = take(PRO_VOCAB * 128);
    Weights { lig_emb, pro_emb, lig_c1w, lig_c1b, lig_c2w, lig_c2b, lig_c3w, lig_c3b, pro_c1w, pro_c1b, pro_c2w, pro_c2b, pro_c3w, pro_c3b, fc1w, fc1b, fc2w, fc2b, fc3w, fc3b, fc4w, fc4b }
}

pub struct DeepDtaModel { w: Weights }

impl DeepDtaModel {
    pub fn load(path: &str) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("read model: {e}"))?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        if digest != REGISTERED_SHA256 {
            return Err(format!("sha256 gate: {digest} != registered {REGISTERED_SHA256}"));
        }
        Ok(Self { w: load_weights(&bytes) })
    }
    /// The protein branch depends only on the sequence, so a library screen over one target
    /// computes it once. It is ~194M MACs of the ~200M per molecule: recomputing it per ligand
    /// is what made the in-kernel screen run at 5.6 molecules/s.
    pub fn protein_branch(&self, pro_idx: &[usize]) -> Vec<f32> {
        let mut pro_in = vec![0.0f32; SEQLEN * 128];
        for i in 0..SEQLEN {
            let e = pro_idx[i].min(PRO_VOCAB - 1) * 128;
            for d in 0..128 { pro_in[i * 128 + d] = self.w.pro_emb[e + d]; }
        }
        let p1 = conv_layer(&pro_in, SEQLEN, &self.w.pro_c1w, &self.w.pro_c1b, CHANNEL, 128);
        let p2 = conv_layer(&p1, SEQLEN - K + 1, &self.w.pro_c2w, &self.w.pro_c2b, CHANNEL * 2, CHANNEL);
        let p3 = conv_layer(&p2, SEQLEN - 2 * K + 2, &self.w.pro_c3w, &self.w.pro_c3b, CHANNEL * 3, CHANNEL * 2);
        relu_maxpool(&p3, SEQLEN - 3 * K + 3, CHANNEL * 3)
    }

    /// Score a ligand against a precomputed protein branch.
    pub fn predict_with_protein(&self, protein_pooled: &[f32], lig_idx: &[usize]) -> f32 {
        let mut lig_in = vec![0.0f32; SMILEN * 128];
        for i in 0..SMILEN {
            let e = lig_idx[i].min(LIG_VOCAB - 1) * 128;
            for d in 0..128 { lig_in[i * 128 + d] = self.w.lig_emb[e + d]; }
        }
        let l1 = conv_layer(&lig_in, SMILEN, &self.w.lig_c1w, &self.w.lig_c1b, CHANNEL, 128);
        let l2 = conv_layer(&l1, SMILEN - K + 1, &self.w.lig_c2w, &self.w.lig_c2b, CHANNEL * 2, CHANNEL);
        let l3 = conv_layer(&l2, SMILEN - 2 * K + 2, &self.w.lig_c3w, &self.w.lig_c3b, CHANNEL * 3, CHANNEL * 2);
        let lp = relu_maxpool(&l3, SMILEN - 3 * K + 3, CHANNEL * 3);
        let mut x = vec![0.0f32; CHANNEL * 6];
        x[..96].copy_from_slice(&lp);
        x[96..].copy_from_slice(&protein_pooled[..CHANNEL * 3]);
        let x = mlp(&x, &self.w.fc1w, &self.w.fc1b, 1024);
        let x = mlp(&x, &self.w.fc2w, &self.w.fc2b, 1024);
        let x = mlp(&x, &self.w.fc3w, &self.w.fc3b, 512);
        let mut out = self.w.fc4b[0];
        for d in 0..512 { out += x[d] * self.w.fc4w[d]; }
        out
    }

    /// Single-call convenience: compute the protein branch then score the ligand.
    pub fn predict(&self, pro_idx: &[usize], lig_idx: &[usize]) -> f32 {
        let branch = self.protein_branch(pro_idx);
        self.predict_with_protein(&branch, lig_idx)
    }
}
