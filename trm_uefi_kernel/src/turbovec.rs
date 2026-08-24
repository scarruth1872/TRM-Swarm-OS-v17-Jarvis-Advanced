//! TRM Swarm OS - TurboVec 16x quantizer + femtosecond SSM scanner (no_std Rust port).
//! 1:1 with swarm_v2/core/turbovec_16x_femto_engine.py:
//!   - 2-bit scalar quantization (4 levels), 16.0x compression
//!   - normalize (L2), quantize_16x, dequantize_16x
//!   - femto_step: h_t = 0.95*h_{t-1} + 0.05*input (state scan)

extern crate alloc;
use alloc::vec::Vec;

pub const DIM: usize = 128;
pub const MAX_VAL: f32 = 3.0; // (1 << 2) - 1

pub struct TurboVec16xQuantizer;

impl TurboVec16xQuantizer {
    pub fn new() -> Self {
        TurboVec16xQuantizer
    }

    /// L2 normalize (matches np.linalg.norm; zero-norm -> 1.0 guard).
    pub fn normalize(v: &[f32]) -> Vec<f32> {
        let mut norm = 0.0f32;
        for &x in v {
            norm += x * x;
        }
        norm = norm.sqrt_approx();
        if norm == 0.0 {
            norm = 1.0;
        }
        v.iter().map(|&x| x / norm).collect()
    }

    /// Quantize to 2-bit codes + (scale, v_min). Returns (codes, scale, v_min).
    pub fn quantize_16x(v: &[f32]) -> (Vec<u8>, f32, f32) {
        let v_norm = Self::normalize(v);
        let v_min = v_norm.iter().copied().fold(f32::INFINITY, f32::min);
        let v_max = v_norm.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let scale = if v_max - v_min > 1e-8 { v_max - v_min } else { 1.0 };

        let codes: Vec<u8> = v_norm
            .iter()
            .map(|&x| round_f32((x - v_min) / scale * MAX_VAL) as u8)
            .collect();
        (codes, scale, v_min)
    }

    pub fn dequantize_16x(q_codes: &[u8], scale: f32, v_min: f32) -> Vec<f32> {
        q_codes
            .iter()
            .map(|&c| (c as f32 / MAX_VAL) * scale + v_min)
            .collect()
    }

    /// Fast cosine similarity (L2-normalized inner product).
    pub fn fast_femto_cosine_similarity(q: &[f32], db_row: &[f32]) -> f32 {
        let q_norm = Self::normalize(q);
        let db_norm = Self::normalize(db_row);
        let mut dot = 0.0f32;
        for (a, b) in q_norm.iter().zip(db_norm.iter()) {
            dot += a * b;
        }
        dot
    }
}

pub struct FemtosecondGenomicSSMScanner {
    pub h_state: [f32; DIM],
    pub alpha: f32,
}

impl FemtosecondGenomicSSMScanner {
    pub fn new() -> Self {
        FemtosecondGenomicSSMScanner {
            h_state: [0.0f32; DIM],
            alpha: 0.95,
        }
    }

    /// h_t = alpha * h_{t-1} + (1-alpha) * dequant(input)
    /// Returns (state_norm, step_latency_ns, quantized_bytes).
    pub fn femto_step(&mut self, input_vector: &[f32]) -> (f32, u64, usize) {
        let t0 = crate::time::now_ns();

        let (codes, scale, v_min) = TurboVec16xQuantizer::quantize_16x(input_vector);
        let v_rec = TurboVec16xQuantizer::dequantize_16x(&codes, scale, v_min);

        let alpha = self.alpha;
        let mut norm_sq = 0.0f32;
        for i in 0..DIM {
            let next = alpha * self.h_state[i] + (1.0 - alpha) * v_rec[i];
            self.h_state[i] = next;
            norm_sq += next * next;
        }

        let lat_ns = crate::time::now_ns().wrapping_sub(t0);
        (norm_sq.sqrt_approx(), lat_ns, codes.len() / 4)
    }
}

/// no_std sqrt for f32 - Newton-Raphson on bit representation (no asm, no SSE).
trait SqrtApprox {
    fn sqrt_approx(self) -> f32;
}
impl SqrtApprox for f32 {
    fn sqrt_approx(self) -> f32 {
        if self <= 0.0 {
            return 0.0;
        }
        // Fast inverse-ish seed via bit hack, then 2 Newton iterations.
        let bits = self.to_bits();
        let seed_bits = (bits.wrapping_add(127u32 << 23) >> 1) as u32;
        let mut x = f32::from_bits(seed_bits);
        for _ in 0..3 {
            x = 0.5 * (x + self / x);
        }
        x
    }
}

/// Round half-away-from-zero (matches np.round / rust f32::round semantics
/// for the quantizer: values are in [0,3], so tie direction rarely matters).
fn round_f32(x: f32) -> f32 {
    if x >= 0.0 {
        (x + 0.5).floor_hack()
    } else {
        (x - 0.5).ceil_hack()
    }
}

trait FloorCeilHack {
    fn floor_hack(self) -> f32;
    fn ceil_hack(self) -> f32;
}
impl FloorCeilHack for f32 {
    fn floor_hack(self) -> f32 {
        let i = self as i32;
        if self >= 0.0 || self == i as f32 { i as f32 } else { (i - 1) as f32 }
    }
    fn ceil_hack(self) -> f32 {
        let i = self as i32;
        if self <= 0.0 || self == i as f32 { i as f32 } else { (i + 1) as f32 }
    }
}
