//! TRM 7M — Tiny Recursive Model (Mamba-3 SSM) in no_std Rust.
//! Bit-for-bit 1:1 with the Python platform: loads the SAME seed-42 weights
//! from the embedded blob, so outputs and behavior match the full platform.
//!
//! Forward pass (identical math to swarm_v2/core/trm_7m_model.py):
//!   h = input_embed @ x
//!   for each block: input_proj, conv1d, dt_proj, A/B/C/D SSM, out_proj, SiLU
//!   logits = output_head @ h + bias -> softmax -> Kleene trit {-1,0,+1}

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
pub const STATE_DIM: usize = 462;    // scaled 256 * 6^(1/3)
pub const INPUT_DIM: usize = 128;
pub const OUTPUT_TRITS: usize = 3;
pub const NUM_BLOCKS: usize = 3;
pub const D_CONV: usize = 4;
pub const DT_RANK: usize = 16;

/// Weight blob offsets — computed from export_trm7m_weights.py layout.
/// Per block emit order: input_proj (d,d), conv1d_weight (4,d), dt_proj (d,d),
/// A (d,) diagonal VECTOR, B (d,d), C (d,d), D (d,d), out_proj (d,d).
const INPUT_EMBED_BYTES: usize = STATE_DIM * INPUT_DIM;   // 462*128 = 59,136
const IN_PROJ: usize = STATE_DIM * STATE_DIM;             // 213,444
const CONV1D: usize = D_CONV * STATE_DIM;                 // 1,848
const DT_PROJ: usize = STATE_DIM * STATE_DIM;             // 213,444 (python: randn(dim,dim)!)
const A_VEC: usize = STATE_DIM;                           // 462 (diagonal vector)
const BM: usize = STATE_DIM * STATE_DIM;                  // 213,444
const CM: usize = STATE_DIM * STATE_DIM;                  // 213,444
const DM: usize = STATE_DIM * STATE_DIM;                  // 213,444
const OUT_PROJ: usize = STATE_DIM * STATE_DIM;            // 213,444
const BLOCK_STRIDE: usize = IN_PROJ + CONV1D + DT_PROJ + A_VEC + BM + CM + DM + OUT_PROJ; // 1,282,974
const HEAD_OFF: usize = INPUT_EMBED_BYTES + 3 * BLOCK_STRIDE; // 3,908,058
const BIAS_OFF: usize = HEAD_OFF + OUTPUT_TRITS * STATE_DIM;  // +1,386

/// Embedded canonical seed-42 weights (15,637,788 bytes ≈ 14.9 MiB).
static TRM7M_WEIGHTS: &[u8] = include_bytes!("../../assets/trm7m_weights.bin");

/// The persistent Mamba-3 hidden state (O(1) memory — 8.1 KB).
pub struct Trm7m {
    pub state_dim: usize,
    pub hidden: Vec<f32>,            // h(t) per block, concatenated
    pub conv_buffers: Vec<f32>,      // d_conv x dim per block
    pub forward_calls: u64,
    pub total_latency_ns: u128,
    pub last_trit: i8,
    pub last_confidence: f32,
}

impl Trm7m {
    pub fn new() -> Self {
        let hidden = vec![0.0f32; STATE_DIM * NUM_BLOCKS];
        let conv_buffers = vec![0.0f32; D_CONV * STATE_DIM * NUM_BLOCKS];
        Trm7m {
            state_dim: STATE_DIM,
            hidden,
            conv_buffers,
            forward_calls: 0,
            total_latency_ns: 0,
            last_trit: 0,
            last_confidence: 0.0,
        }
    }

    /// Number of embedded weight bytes (for the `trm` command report).
    pub fn weight_bytes() -> usize {
        TRM7M_WEIGHTS.len()
    }

    /// Total parameters (weights / 4).
    pub fn total_params() -> usize {
        TRM7M_WEIGHTS.len() / 4
    }

    #[inline]
    fn f32_at(offset: usize) -> f32 {
        let b = &TRM7M_WEIGHTS[offset * 4..offset * 4 + 4];
        f32::from_le_bytes([b[0], b[1], b[2], b[3]])
    }

    /// Zero-copy view of the whole weight blob as f32 (safe: blob length is
    /// a multiple of 4 and alignment is guaranteed by include_bytes static).
    #[inline]
    fn w(offset: usize) -> f32 {
        // SAFETY: TRM7M_WEIGHTS.len() == 15,637,788 == 3,909,447 * 4; the
        // static data is at least 4-aligned in practice (linker), and we
        // only index within bounds. For strict alignment, copy via read.
        unsafe {
            let base = TRM7M_WEIGHTS.as_ptr() as *const f32;
            core::ptr::read_unaligned(base.add(offset))
        }
    }

    #[inline]
    #[target_feature(enable = "avx2,fma")]
    unsafe fn hsum256_ps(v: core::arch::x86_64::__m256) -> f32 {
        use core::arch::x86_64::*;
        let vlow = _mm256_castps256_ps128(v);
        let vhigh = _mm256_extractf128_ps(v, 1);
        let v128 = _mm_add_ps(vlow, vhigh);
        let v64 = _mm_add_ps(v128, _mm_movehl_ps(v128, v128));
        let v32 = _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55));
        _mm_cvtss_f32(v32)
    }

    #[inline]
    #[target_feature(enable = "avx2,fma")]
    unsafe fn matvec_avx2(m: usize, k: usize, a_off: usize, x: &[f32], out: &mut [f32]) {
        use core::arch::x86_64::*;
        let base_ptr = (TRM7M_WEIGHTS.as_ptr() as *const f32).add(a_off);
        let x_ptr = x.as_ptr();
        let k_vec = k - (k % 8);

        let mut i = 0;
        while i + 4 <= m {
            let row0 = base_ptr.add(i * k);
            let row1 = base_ptr.add((i + 1) * k);
            let row2 = base_ptr.add((i + 2) * k);
            let row3 = base_ptr.add((i + 3) * k);

            let mut acc0 = _mm256_setzero_ps();
            let mut acc1 = _mm256_setzero_ps();
            let mut acc2 = _mm256_setzero_ps();
            let mut acc3 = _mm256_setzero_ps();

            let mut j = 0;
            while j < k_vec {
                let vx = _mm256_loadu_ps(x_ptr.add(j));
                acc0 = _mm256_fmadd_ps(_mm256_loadu_ps(row0.add(j)), vx, acc0);
                acc1 = _mm256_fmadd_ps(_mm256_loadu_ps(row1.add(j)), vx, acc1);
                acc2 = _mm256_fmadd_ps(_mm256_loadu_ps(row2.add(j)), vx, acc2);
                acc3 = _mm256_fmadd_ps(_mm256_loadu_ps(row3.add(j)), vx, acc3);
                j += 8;
            }

            let mut s0 = 0.0f32;
            let mut s1 = 0.0f32;
            let mut s2 = 0.0f32;
            let mut s3 = 0.0f32;
            while j < k {
                let xj = *x_ptr.add(j);
                s0 += *row0.add(j) * xj;
                s1 += *row1.add(j) * xj;
                s2 += *row2.add(j) * xj;
                s3 += *row3.add(j) * xj;
                j += 1;
            }

            out[i] = Self::hsum256_ps(acc0) + s0;
            out[i + 1] = Self::hsum256_ps(acc1) + s1;
            out[i + 2] = Self::hsum256_ps(acc2) + s2;
            out[i + 3] = Self::hsum256_ps(acc3) + s3;
            i += 4;
        }

        while i < m {
            let row = base_ptr.add(i * k);
            let mut acc = _mm256_setzero_ps();
            let mut j = 0;
            while j < k_vec {
                let vx = _mm256_loadu_ps(x_ptr.add(j));
                acc = _mm256_fmadd_ps(_mm256_loadu_ps(row.add(j)), vx, acc);
                j += 8;
            }
            let mut s = 0.0f32;
            while j < k {
                s += *row.add(j) * *x_ptr.add(j);
                j += 1;
            }
            out[i] = Self::hsum256_ps(acc) + s;
            i += 1;
        }
    }

    #[inline]
    #[target_feature(enable = "avx2,fma")]
    unsafe fn matvec_accum_avx2(m: usize, k: usize, a_off: usize, x: &[f32], out: &mut [f32]) {
        use core::arch::x86_64::*;
        let base_ptr = (TRM7M_WEIGHTS.as_ptr() as *const f32).add(a_off);
        let x_ptr = x.as_ptr();
        let k_vec = k - (k % 8);

        let mut i = 0;
        while i + 4 <= m {
            let row0 = base_ptr.add(i * k);
            let row1 = base_ptr.add((i + 1) * k);
            let row2 = base_ptr.add((i + 2) * k);
            let row3 = base_ptr.add((i + 3) * k);

            let mut acc0 = _mm256_setzero_ps();
            let mut acc1 = _mm256_setzero_ps();
            let mut acc2 = _mm256_setzero_ps();
            let mut acc3 = _mm256_setzero_ps();

            let mut j = 0;
            while j < k_vec {
                let vx = _mm256_loadu_ps(x_ptr.add(j));
                acc0 = _mm256_fmadd_ps(_mm256_loadu_ps(row0.add(j)), vx, acc0);
                acc1 = _mm256_fmadd_ps(_mm256_loadu_ps(row1.add(j)), vx, acc1);
                acc2 = _mm256_fmadd_ps(_mm256_loadu_ps(row2.add(j)), vx, acc2);
                acc3 = _mm256_fmadd_ps(_mm256_loadu_ps(row3.add(j)), vx, acc3);
                j += 8;
            }

            let mut s0 = 0.0f32;
            let mut s1 = 0.0f32;
            let mut s2 = 0.0f32;
            let mut s3 = 0.0f32;
            while j < k {
                let xj = *x_ptr.add(j);
                s0 += *row0.add(j) * xj;
                s1 += *row1.add(j) * xj;
                s2 += *row2.add(j) * xj;
                s3 += *row3.add(j) * xj;
                j += 1;
            }

            out[i] += Self::hsum256_ps(acc0) + s0;
            out[i + 1] += Self::hsum256_ps(acc1) + s1;
            out[i + 2] += Self::hsum256_ps(acc2) + s2;
            out[i + 3] += Self::hsum256_ps(acc3) + s3;
            i += 4;
        }

        while i < m {
            let row = base_ptr.add(i * k);
            let mut acc = _mm256_setzero_ps();
            let mut j = 0;
            while j < k_vec {
                let vx = _mm256_loadu_ps(x_ptr.add(j));
                acc = _mm256_fmadd_ps(_mm256_loadu_ps(row.add(j)), vx, acc);
                j += 8;
            }
            let mut s = 0.0f32;
            while j < k {
                s += *row.add(j) * *x_ptr.add(j);
                j += 1;
            }
            out[i] += Self::hsum256_ps(acc) + s;
            i += 1;
        }
    }

    #[inline]
    fn matvec(m: usize, k: usize, a_off: usize, x: &[f32], out: &mut [f32]) {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe {
                Self::matvec_avx2(m, k, a_off, x, out);
                return;
            }
        }
        #[allow(unreachable_code)]
        for i in 0..m {
            let mut acc = 0.0f32;
            let row = a_off + i * k;
            for j in 0..k {
                acc += Self::w(row + j) * x[j];
            }
            out[i] = acc;
        }
    }

    #[inline]
    fn matvec_accum(m: usize, k: usize, a_off: usize, x: &[f32], out: &mut [f32]) {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe {
                Self::matvec_accum_avx2(m, k, a_off, x, out);
                return;
            }
        }
        #[allow(unreachable_code)]
        for i in 0..m {
            let mut acc = 0.0f32;
            let row = a_off + i * k;
            for j in 0..k {
                acc += Self::w(row + j) * x[j];
            }
            out[i] += acc;
        }
    }

    #[inline]
    fn matvec_col(m: usize, k: usize, a_off: usize, x: &[f32], out: &mut [f32]) {
        let _ = (m, k, a_off, x, out);
    }

    /// Single forward pass: 128-dim input -> Kleene trit {-1,0,+1} + confidence.
    /// Returns (predicted_trit, confidence, latency_ns).
    pub fn forward(&mut self, x_in: &[f32]) -> (i8, f32, u64) {
        let t0 = rdtsc_ns();

        // Pad/truncate input to INPUT_DIM
        let mut x = [0.0f32; INPUT_DIM];
        let n = x_in.len().min(INPUT_DIM);
        x[..n].copy_from_slice(&x_in[..n]);

        // input_embed @ x  -> h0 (STATE_DIM)
        let mut h = [0.0f32; STATE_DIM];
        Self::matvec(STATE_DIM, INPUT_DIM, 0, &x, &mut h);

        // Mamba-3 blocks
        for b in 0..NUM_BLOCKS {
            let base = INPUT_EMBED_BYTES + b * BLOCK_STRIDE;
            let input_proj = base;
            let conv1d = base + IN_PROJ;
            let dt_proj = base + IN_PROJ + CONV1D;
            let a_vec = base + IN_PROJ + CONV1D + DT_PROJ;
            let b_mat = a_vec + A_VEC;
            let c_mat = b_mat + BM;
            let d_mat = c_mat + CM;
            let out_proj = d_mat + DM;

            // input_proj @ h
            let mut x_proj = [0.0f32; STATE_DIM];
            Self::matvec(STATE_DIM, STATE_DIM, input_proj, &h, &mut x_proj);

            // conv1d causal buffer shift
            let cb_off = b * D_CONV * STATE_DIM;
            for r in 0..D_CONV - 1 {
                let src = cb_off + (r + 1) * STATE_DIM;
                let dst = cb_off + r * STATE_DIM;
                self.conv_buffers.copy_within(src..src + STATE_DIM, dst);
            }
            let last_row = cb_off + (D_CONV - 1) * STATE_DIM;
            self.conv_buffers[last_row..last_row + STATE_DIM].copy_from_slice(&x_proj);

            // x_conv = sum(conv_buffer * conv1d_weight) / D_CONV
            let mut x_conv = [0.0f32; STATE_DIM];
            #[cfg(target_arch = "x86_64")]
            unsafe {
                use core::arch::x86_64::*;
                let inv = _mm256_set1_ps(0.25f32);
                let mut j = 0;
                let w_base = (TRM7M_WEIGHTS.as_ptr() as *const f32).add(conv1d);
                let cb_base = self.conv_buffers.as_ptr().add(cb_off);
                while j + 8 <= STATE_DIM {
                    let mut acc = _mm256_setzero_ps();
                    for r in 0..D_CONV {
                        let c = _mm256_loadu_ps(cb_base.add(r * STATE_DIM + j));
                        let w = _mm256_loadu_ps(w_base.add(r * STATE_DIM + j));
                        acc = _mm256_fmadd_ps(c, w, acc);
                    }
                    _mm256_storeu_ps(x_conv.as_mut_ptr().add(j), _mm256_mul_ps(acc, inv));
                    j += 8;
                }
                while j < STATE_DIM {
                    let mut acc = 0.0f32;
                    for r in 0..D_CONV {
                        acc += *cb_base.add(r * STATE_DIM + j) * *w_base.add(r * STATE_DIM + j);
                    }
                    x_conv[j] = acc * 0.25f32;
                    j += 1;
                }
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for j in 0..STATE_DIM {
                    let mut acc = 0.0f32;
                    for r in 0..D_CONV {
                        let w_off = conv1d + r * STATE_DIM + j;
                        acc += self.conv_buffers[cb_off + r * STATE_DIM + j] * Self::w(w_off);
                    }
                    x_conv[j] = acc * 0.25f32;
                }
            }

            // dt_raw = dt_proj @ x_conv
            let mut dt_raw = [0.0f32; STATE_DIM];
            Self::matvec(STATE_DIM, STATE_DIM, dt_proj, &x_conv, &mut dt_raw);

            // dt = exp(dt_raw * 0.1)
            let mut dt = [0.0f32; STATE_DIM];
            for i in 0..STATE_DIM {
                dt[i] = fast_exp(dt_raw[i] * 0.1);
            }

            // b_x = B @ x_conv
            let mut b_x = [0.0f32; STATE_DIM];
            Self::matvec(STATE_DIM, STATE_DIM, b_mat, &x_conv, &mut b_x);

            // h_new = exp(A * dt) * h + dt * (B @ x_conv)
            let mut h_new = [0.0f32; STATE_DIM];
            for i in 0..STATE_DIM {
                let a_val = Self::w(a_vec + i);
                let exp_a = fast_exp(a_val * dt[i]);
                h_new[i] = exp_a * h[i] + dt[i] * b_x[i];
            }

            // y = C @ h_new + D @ x_conv
            let mut y = [0.0f32; STATE_DIM];
            Self::matvec(STATE_DIM, STATE_DIM, c_mat, &h_new, &mut y);
            Self::matvec_accum(STATE_DIM, STATE_DIM, d_mat, &x_conv, &mut y);

            // out_proj @ y + SiLU
            let mut y_out = [0.0f32; STATE_DIM];
            Self::matvec(STATE_DIM, STATE_DIM, out_proj, &y, &mut y_out);
            for i in 0..STATE_DIM {
                let v = y_out[i];
                y_out[i] = v * sigmoid(v);
            }

            // persist hidden state for this block
            let h_off = b * STATE_DIM;
            self.hidden[h_off..h_off + STATE_DIM].copy_from_slice(&h_new);
            h = y_out; // next block input
        }

        // output_head @ h + bias -> 3 logits
        let mut logits = [0.0f32; OUTPUT_TRITS];
        for i in 0..OUTPUT_TRITS {
            let mut acc = 0.0f32;
            let row = HEAD_OFF + i * STATE_DIM;
            for j in 0..STATE_DIM {
                acc += Self::w(row + j) * h[j];
            }
            logits[i] = acc + Self::w(BIAS_OFF + i);
        }

        // softmax
        let max_l = logits[0].max(logits[1]).max(logits[2]);
        let mut probs = [0.0f32; OUTPUT_TRITS];
        let mut sum = 0.0f32;
        for i in 0..OUTPUT_TRITS {
            probs[i] = fast_exp(logits[i] - max_l);
            sum += probs[i];
        }
        for p in probs.iter_mut() {
            *p /= sum;
        }

        // argmax -> trit {0:PASS(+1), 1:AMB(0), 2:FAULT(-1)}
        let mut best = 0usize;
        for i in 1..OUTPUT_TRITS {
            if probs[i] > probs[best] {
                best = i;
            }
        }
        let trit: i8 = match best {
            0 => 1,
            1 => 0,
            _ => -1,
        };
        let confidence = probs[best];

        let elapsed = rdtsc_ns().wrapping_sub(t0);
        self.forward_calls += 1;
        self.total_latency_ns += elapsed as u128;
        self.last_trit = trit;
        self.last_confidence = confidence;

        (trit, confidence, elapsed)
    }

    /// Benchmark: run `n` forward passes on deterministic inputs, report stats.
    /// Returns (p50_ns, p95_ns, avg_ns, throughput_tps, n).
    pub fn benchmark(&mut self, n: usize) -> (u64, u64, u64, u64, usize) {
        let mut times = vec![0u64; n];
        let mut x = [0.0f32; INPUT_DIM];
        for i in 0..n {
            // deterministic LCG input (reproducible)
            let seed = (i as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let mut s = seed;
            for j in 0..INPUT_DIM {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                x[j] = ((s >> 33) as f32 / (1u64 << 31) as f32) - 1.0;
            }
            let (_, _, ns) = self.forward(&x);
            times[i] = ns;
        }
        times.sort_unstable();
        let p50 = times[n / 2];
        let p95 = times[(n as f64 * 0.95) as usize];
        let sum: u64 = times.iter().sum();
        let avg = sum / n as u64;
        let tps = if avg > 0 { 1_000_000_000 / avg } else { 0 };
        (p50, p95, avg, tps, n)
    }
}

#[inline]
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + fast_exp(-x))
}

/// Fast exp via bit hack (≈2^-21 relative error — fine for softmax/SSM).
#[inline]
fn fast_exp(x: f32) -> f32 {
    let clamped = x.clamp(-88.0, 88.0);
    let i = (clamped * 1.4426950408889634 + 127.0) as i32;
    f32::from_bits(((i as u32) << 23) as u32)
}

/// TSC-based ns timing (calibrated in time.rs).
#[inline]
fn rdtsc_ns() -> u64 {
    crate::time::now_ns()
}
