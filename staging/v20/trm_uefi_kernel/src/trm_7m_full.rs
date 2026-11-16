//! TRM Swarm OS — trm_7m_full (bare-metal no_std + alloc).
//! 7,082,496 parameter Tiny Recursive Model with Mamba-3 SSM backbone.
//! Quantizable into 2-bit K3 trits for Infinity Cache / L1 residency.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

pub const STATE_DIM: usize = 768;
pub const INPUT_DIM: usize = 128;
pub const NUM_BLOCKS: usize = 3;
pub const BLOCK_PARAMS: usize = STATE_DIM + 4 * STATE_DIM * STATE_DIM; // 2,360,064
pub const OUTPUT_PARAMS: usize = 3 * STATE_DIM; // 2,304
pub const TOTAL_PARAMS: usize = NUM_BLOCKS * BLOCK_PARAMS + OUTPUT_PARAMS; // 7,082,496

static TRM7M_CALLS: AtomicU64 = AtomicU64::new(0);

/// Native 2-bit quantization & trit packing (bare-metal strip twin).
#[inline]
pub fn strip_trm7m_quantize_2bit(arr: &[f32], out_packed: &mut [u8]) {
    let count = arr.len();
    if count == 0 {
        return;
    }

    let mut minv = arr[0];
    let mut maxv = arr[0];
    for &val in arr.iter() {
        if val < minv { minv = val; }
        if val > maxv { maxv = val; }
    }

    let scale = (maxv - minv) / 3.0f32;
    let inv_scale = if scale > 1e-8f32 { 1.0f32 / scale } else { 0.0f32 };

    let out_len = (count + 3) / 4;
    let max_packed = out_packed.len().min(out_len);

    for i in 0..max_packed {
        let base = i * 4;
        let mut b = 0u8;
        for j in 0..4 {
            let idx = base + j;
            if idx < count {
                let norm = (arr[idx] - minv) * inv_scale;
                let q = if norm <= 0.5f32 {
                    0u8
                } else if norm <= 1.5f32 {
                    1u8
                } else if norm <= 2.5f32 {
                    2u8
                } else {
                    3u8
                };
                b |= (q & 0x03) << (j * 2);
            }
        }
        out_packed[i] = b;
    }
}

/// Mamba-3 continuous SSM block: dh/dt = A·h + B·x
#[derive(Debug, Clone)]
pub struct Mamba3Block {
    pub dim: usize,
    pub a: Vec<f32>,
    pub b: Vec<f32>,
    pub c: Vec<f32>,
    pub d: Vec<f32>,
    pub in_proj: Vec<f32>,
    pub h: Vec<f32>,
    pub dt: f32,
}

impl Mamba3Block {
    pub fn new(dim: usize) -> Self {
        let mut a = vec![-0.5f32; dim];
        let mut b = vec![0.02f32; dim * dim];
        let mut c = vec![0.02f32; dim * dim];
        let mut d = vec![0.02f32; dim * dim];
        let mut in_proj = vec![0.02f32; dim * dim];

        // Seed with deterministic pseudo-random values
        let mut lcg = (dim as u64).wrapping_mul(0x9E3779B97F4A7C15);
        for item in a.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = -0.5 - (((lcg >> 32) as i32 % 500) as f32 / 5000.0);
        }
        for item in b.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = (((lcg >> 32) as i32 % 1000) as f32 / 50000.0) - 0.01;
        }
        for item in c.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = (((lcg >> 32) as i32 % 1000) as f32 / 50000.0) - 0.01;
        }
        for item in d.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = (((lcg >> 32) as i32 % 1000) as f32 / 50000.0) - 0.01;
        }
        for item in in_proj.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = (((lcg >> 32) as i32 % 1000) as f32 / 50000.0) - 0.01;
        }

        Self {
            dim,
            a,
            b,
            c,
            d,
            in_proj,
            h: vec![0.0f32; dim],
            dt: 1e-6f32,
        }
    }

    /// Fast forward step: xp = in_proj * x, h' = exp(a * dt) * h + b * xp, y = c * h' + d * xp
    pub fn forward(&mut self, x: &[f32]) -> Vec<f32> {
        let mut xp = vec![0.0f32; self.dim];
        let in_len = self.dim.min(x.len());
        for i in 0..self.dim {
            let mut sum = 0.0f32;
            for j in 0..in_len {
                sum += self.in_proj[i * self.dim + j] * x[j];
            }
            xp[i] = sum;
        }

        let mut h_new = vec![0.0f32; self.dim];
        for i in 0..self.dim {
            // exp(a * dt) ~ 1.0 + a * dt (Taylor approx in bare metal)
            let exp_a = 1.0f32 + self.a[i] * self.dt;
            let mut bx = 0.0f32;
            for j in 0..self.dim {
                bx += self.b[i * self.dim + j] * xp[j];
            }
            h_new[i] = exp_a * self.h[i] + bx;
        }
        self.h = h_new.clone();

        let mut y = vec![0.0f32; self.dim];
        for i in 0..self.dim {
            let mut ch = 0.0f32;
            let mut dx = 0.0f32;
            for j in 0..self.dim {
                ch += self.c[i * self.dim + j] * self.h[j];
                dx += self.d[i * self.dim + j] * xp[j];
            }
            y[i] = ch + dx;
        }

        y
    }
}

/// Full 7 Million Parameter Tiny Recursive Model.
pub struct TRM7M {
    pub embed: Vec<f32>,
    pub blocks: Vec<Mamba3Block>,
    pub out_proj: Vec<f32>,
}

impl TRM7M {
    pub fn new() -> Self {
        let mut embed = vec![0.02f32; STATE_DIM * INPUT_DIM];
        let mut out_proj = vec![0.02f32; 3 * STATE_DIM];

        let mut lcg = 0x7E07E07E0u64;
        for item in embed.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = (((lcg >> 32) as i32 % 1000) as f32 / 50000.0) - 0.01;
        }
        for item in out_proj.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            *item = (((lcg >> 32) as i32 % 1000) as f32 / 50000.0) - 0.01;
        }

        let mut blocks = Vec::with_capacity(NUM_BLOCKS);
        for _ in 0..NUM_BLOCKS {
            blocks.push(Mamba3Block::new(STATE_DIM));
        }

        Self {
            embed,
            blocks,
            out_proj,
        }
    }

    pub fn forward(&mut self, x: &[f32]) -> (Vec<f32>, i8) {
        TRM7M_CALLS.fetch_add(1, Ordering::Relaxed);
        let in_len = INPUT_DIM.min(x.len());
        let mut h = vec![0.0f32; STATE_DIM];

        // Embedding layer: h = embed @ x
        for i in 0..STATE_DIM {
            let mut sum = 0.0f32;
            for j in 0..in_len {
                sum += self.embed[i * INPUT_DIM + j] * x[j];
            }
            h[i] = sum;
        }

        // Mamba-3 SSM sequence
        for block in self.blocks.iter_mut() {
            h = block.forward(&h);
        }

        // Output projection to 3 logits (PASS=+1, AMBIGUOUS=0, FAULT=-1)
        let mut logits = vec![0.0f32; 3];
        for i in 0..3 {
            let mut sum = 0.0f32;
            for j in 0..STATE_DIM {
                sum += self.out_proj[i * STATE_DIM + j] * h[j];
            }
            logits[i] = sum;
        }

        // Softmax argmax
        let mut max_idx = 0;
        let mut max_val = logits[0];
        for i in 1..3 {
            if logits[i] > max_val {
                max_val = logits[i];
                max_idx = i;
            }
        }

        let trit = match max_idx {
            0 => 1i8,
            1 => 0i8,
            _ => -1i8,
        };

        (logits, trit)
    }

    pub fn quantize_2bit(&self) -> Vec<u8> {
        let mut flat = Vec::with_capacity(TOTAL_PARAMS);
        flat.extend_from_slice(&self.embed);
        for b in &self.blocks {
            flat.extend_from_slice(&b.a);
            flat.extend_from_slice(&b.b);
            flat.extend_from_slice(&b.c);
            flat.extend_from_slice(&b.d);
            flat.extend_from_slice(&b.in_proj);
        }
        flat.extend_from_slice(&self.out_proj);

        let out_len = (flat.len() + 3) / 4;
        let mut packed = vec![0u8; out_len];
        strip_trm7m_quantize_2bit(&flat, &mut packed);
        packed
    }
}

pub fn get_trm7m_param_count() -> u64 {
    TOTAL_PARAMS as u64
}

pub fn trm7m_forward(x: &[f32]) -> (Vec<f32>, i8) {
    let mut model = TRM7M::new();
    model.forward(x)
}

pub fn trm7m_quantize(weights: &[f32]) -> Vec<u8> {
    let out_len = (weights.len() + 3) / 4;
    let mut out = vec![0u8; out_len];
    strip_trm7m_quantize_2bit(weights, &mut out);
    out
}

