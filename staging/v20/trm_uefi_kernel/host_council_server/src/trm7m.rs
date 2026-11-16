//! TRM 7M — Tiny Recursive Model (Mamba-3 SSM), ported 1:1 from trm_uefi_kernel/src/trm7m.rs.
//!
//! WHY THIS FILE EXISTS: the TRM 7M bundle was REGISTERED by this server (see model_registry.rs, which
//! serves it at /models) but nothing in this crate ever invoked it -- `grep -i trm7m` over
//! host_council_server/src returned 0 hits while the kernel reported the model as present. A registered
//! model that no route can run is a declaration, not a capability. This module makes it runnable.
//!
//! FIDELITY: the math below is the UEFI kernel's forward pass, unchanged -- same STATE_DIM (462), same
//! NUM_BLOCKS (3), same weight offsets, same order of operations, same seed-42 blob (the SAME file, via
//! include_bytes!, not a re-export). Two adaptations only, both stated where they occur:
//!   1. `b_dt` lives on the heap instead of the stack: it is 462*462 f32 = 854 KB, which is fine inside the
//!      UEFI kernel's own stack but not inside an HTTP handler thread.
//!   2. timing uses std::time::Instant instead of the kernel's TSC reader (no TSC shim in this crate).
//!
//! Output contract: one Kleene trit in {-1, 0, +1} + softmax confidence -- the same (trit, confidence)
//! the kernel's `Trm7m::forward` returns, so the served value is the model's, not a re-derivation of it.

use std::time::Instant;

pub const STATE_DIM: usize = 462;    // scaled 256 * 6^(1/3)
pub const INPUT_DIM: usize = 128;
pub const OUTPUT_TRITS: usize = 3;
pub const NUM_BLOCKS: usize = 3;
pub const D_CONV: usize = 4;

const INPUT_EMBED_BYTES: usize = STATE_DIM * INPUT_DIM;
const IN_PROJ: usize = STATE_DIM * STATE_DIM;
const CONV1D: usize = D_CONV * STATE_DIM;
const DT_PROJ: usize = STATE_DIM * STATE_DIM;
const A_VEC: usize = STATE_DIM;
const BM: usize = STATE_DIM * STATE_DIM;
const CM: usize = STATE_DIM * STATE_DIM;
const DM: usize = STATE_DIM * STATE_DIM;
const OUT_PROJ: usize = STATE_DIM * STATE_DIM;
const BLOCK_STRIDE: usize = IN_PROJ + CONV1D + DT_PROJ + A_VEC + BM + CM + DM + OUT_PROJ;
const HEAD_OFF: usize = INPUT_EMBED_BYTES + 3 * BLOCK_STRIDE;
const BIAS_OFF: usize = HEAD_OFF + OUTPUT_TRITS * STATE_DIM;

/// The SAME canonical seed-42 blob the UEFI kernel embeds (relative to this file: host_council_server/src
/// -> host_council_server -> trm_uefi_kernel/assets/trm7m_weights.bin).
static TRM7M_WEIGHTS: &[u8] = include_bytes!("../../assets/trm7m_weights.bin");

pub struct Trm7m {
    pub hidden: Vec<f32>,
    pub conv_buffers: Vec<f32>,
    pub forward_calls: u64,
    pub last_trit: i8,
    pub last_confidence: f32,
    pub carry_hidden: bool,
    pub last_logits: [f32; OUTPUT_TRITS],
}

impl Default for Trm7m {
    fn default() -> Self {
        Self::new()
    }
}

impl Trm7m {
    pub fn new() -> Self {
        Trm7m {
            hidden: vec![0.0f32; STATE_DIM * NUM_BLOCKS],
            conv_buffers: vec![0.0f32; D_CONV * STATE_DIM * NUM_BLOCKS],
            forward_calls: 0,
            last_trit: 0,
            last_confidence: 0.0,
            carry_hidden: false,
            last_logits: [0.0; OUTPUT_TRITS],
        }
    }

    pub fn weight_bytes() -> usize {
        TRM7M_WEIGHTS.len()
    }

    /// The weights as a properly-aligned f32 slice (copied — `include_bytes!` is not 4-byte aligned on
    /// every target). Used by the GPU kernel to upload the blob to the device once.
    pub fn weights_f32() -> Vec<f32> {
        TRM7M_WEIGHTS
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }

    /// Total parameters (weights / 4) -- reported, never inferred from a filename.
    pub fn total_params() -> usize {
        TRM7M_WEIGHTS.len() / 4
    }

    #[inline]
    fn w(offset: usize) -> f32 {
        let base = TRM7M_WEIGHTS.as_ptr() as *const f32;
        // SAFETY: offset < len/4 for every call site (offsets are derived from the same layout that
        // produced the blob, and the blob length is a multiple of 4). read_unaligned tolerates the
        // linker's alignment, exactly as the UEFI kernel does. (An aligned slice load would let LLVM
        // vectorize the 18 matvec loops at once, which OOMs the optimizer on this host — so the port
        // keeps the kernel's own read_unaligned access and stays at ~2.8 ms/call, the GPU batch path
        // being the scale fix.)
        unsafe { std::ptr::read_unaligned(base.add(offset)) }
    }

    #[inline]
    fn matvec(m: usize, k: usize, a_off: usize, x: &[f32], out: &mut [f32]) {
        for i in 0..m {
            let mut acc = 0.0f32;
            let row = a_off + i * k;
            for j in 0..k {
                acc += Self::w(row + j) * x[j];
            }
            out[i] = acc;
        }
    }

    /// The kernel's prompt->input transform, unchanged: FNV-1a over the bytes, each 32-bit draw mapped to
    /// [-1, 1) into x[i % 128]. Deterministic, so the same text always yields the same trit.
    pub fn encode_text(text: &str) -> [f32; INPUT_DIM] {
        let mut x = [0.0f32; INPUT_DIM];
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for (i, &b) in text.as_bytes().iter().enumerate() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
            x[i % INPUT_DIM] = ((h >> 32) as f32 / (1u64 << 31) as f32) - 1.0;
        }
        x
    }

    /// One forward pass: 128-dim input -> (trit, confidence, latency_ns). The kernel's math, verbatim.
    pub fn forward(&mut self, x_in: &[f32]) -> (i8, f32, u64) {
        let t0 = Instant::now();

        let mut x = [0.0f32; INPUT_DIM];
        let n = x_in.len().min(INPUT_DIM);
        x[..n].copy_from_slice(&x_in[..n]);

        let mut h = [0.0f32; STATE_DIM];
        if self.carry_hidden {
            // A "thinking turn" continuation: seed the block-0 input from the previous turn's carried
            // SSM state (self.hidden) instead of restarting from the input projection, so the recurrence
            // evolves across turns.
            h.copy_from_slice(&self.hidden[..STATE_DIM]);
        } else {
            Self::matvec(STATE_DIM, INPUT_DIM, 0, &x, &mut h);
        }

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

            let mut x_proj = [0.0f32; STATE_DIM];
            Self::matvec(STATE_DIM, STATE_DIM, input_proj, &h, &mut x_proj);
            // (canonical model's norm is all-ones: identity, skipped as in the kernel)

            let cb_off = b * D_CONV * STATE_DIM;
            for r in 0..D_CONV - 1 {
                let src = cb_off + (r + 1) * STATE_DIM;
                let dst = cb_off + r * STATE_DIM;
                for j in 0..STATE_DIM {
                    self.conv_buffers[dst + j] = self.conv_buffers[src + j];
                }
            }
            let last_row = cb_off + (D_CONV - 1) * STATE_DIM;
            self.conv_buffers[last_row..last_row + STATE_DIM].copy_from_slice(&x_proj);

            let mut x_conv = [0.0f32; STATE_DIM];
            for j in 0..STATE_DIM {
                let mut acc = 0.0f32;
                for r in 0..D_CONV {
                    let w_off = conv1d + r * STATE_DIM + j;
                    acc += self.conv_buffers[cb_off + r * STATE_DIM + j] * Self::w(w_off);
                }
                x_conv[j] = acc / D_CONV as f32;
            }

            let mut dt_raw = [0.0f32; STATE_DIM];
            for i in 0..STATE_DIM {
                let mut acc = 0.0f32;
                let row = dt_proj + i * STATE_DIM;
                for j in 0..STATE_DIM {
                    acc += x_conv[j] * Self::w(row + j);
                }
                dt_raw[i] = acc;
            }

            let mut dt = [0.0f32; STATE_DIM];
            // ADAPTATION 1: heap, not stack (854 KB). Same values, same order.
            let mut b_dt = vec![0.0f32; STATE_DIM * STATE_DIM];
            for i in 0..STATE_DIM {
                dt[i] = fast_exp(dt_raw[i] * 0.1);
            }
            for i in 0..STATE_DIM {
                let row = b_mat + i * STATE_DIM;
                for j in 0..STATE_DIM {
                    b_dt[i * STATE_DIM + j] = Self::w(row + j) * dt[i];
                }
            }

            let mut exp_a = [0.0f32; STATE_DIM];
            for i in 0..STATE_DIM {
                exp_a[i] = fast_exp(Self::w(a_vec + i) * dt[i]);
            }

            let mut h_new = [0.0f32; STATE_DIM];
            for i in 0..STATE_DIM {
                h_new[i] = exp_a[i] * h[i];
            }
            for i in 0..STATE_DIM {
                let mut acc = 0.0f32;
                let row = i * STATE_DIM;
                for j in 0..STATE_DIM {
                    acc += b_dt[row + j] * x_conv[j];
                }
                h_new[i] += acc;
            }

            let mut y = [0.0f32; STATE_DIM];
            for i in 0..STATE_DIM {
                let mut acc = 0.0f32;
                let row = c_mat + i * STATE_DIM;
                for j in 0..STATE_DIM {
                    acc += Self::w(row + j) * h_new[j];
                }
                y[i] = acc;
            }
            for i in 0..STATE_DIM {
                let row = d_mat + i * STATE_DIM;
                let mut acc = 0.0f32;
                for j in 0..STATE_DIM {
                    acc += Self::w(row + j) * x_conv[j];
                }
                y[i] += acc;
            }

            let mut y_out = [0.0f32; STATE_DIM];
            Self::matvec(STATE_DIM, STATE_DIM, out_proj, &y, &mut y_out);
            for i in 0..STATE_DIM {
                let v = y_out[i];
                y_out[i] = v * sigmoid(v);
            }

            let h_off = b * STATE_DIM;
            self.hidden[h_off..h_off + STATE_DIM].copy_from_slice(&h_new);
            h = y_out;
        }

        let mut logits = [0.0f32; OUTPUT_TRITS];
        for i in 0..OUTPUT_TRITS {
            let mut acc = 0.0f32;
            let row = HEAD_OFF + i * STATE_DIM;
            for j in 0..STATE_DIM {
                acc += Self::w(row + j) * h[j];
            }
            logits[i] = acc + Self::w(BIAS_OFF + i);
        }

        let max_l = logits[0].max(logits[1]).max(logits[2]);
        self.last_logits = logits;
        let mut probs = [0.0f32; OUTPUT_TRITS];
        let mut sum = 0.0f32;
        for i in 0..OUTPUT_TRITS {
            probs[i] = fast_exp(logits[i] - max_l);
            sum += probs[i];
        }
        for p in probs.iter_mut() {
            *p /= sum;
        }

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

        // ADAPTATION 2: Instant instead of the kernel's TSC reader.
        let elapsed = t0.elapsed().as_nanos() as u64;
        self.forward_calls += 1;
        self.last_trit = trit;
        self.last_confidence = confidence;
        (trit, confidence, elapsed)
    }

    /// Evaluate a text through the model and report it as the platform's own trit vocabulary.
    pub fn evaluate_text(&mut self, text: &str) -> (i8, f32, u64) {
        let x = Self::encode_text(text);
        self.forward(&x)
    }

    /// N-turn "thinking" with per-turn logic. Each turn runs the forward (carrying the hidden
    /// state across turns) and applies the K3 collapse: the raw argmax trit is only a decision when
    /// the turn is CONFIDENT (a peaked softmax). A near-uniform softmax (confidence ~ 1/3) is the
    /// model saying "unknown" — the turn's decision is UNKNOWN (0), never the argmax's PASS on a
    /// tie. Carrying the hidden state across turns lets a decision solidify over deliberation.
    pub fn think(&mut self, text: &str, turns: usize) -> (i8, f32, u64) {
        let x = Self::encode_text(text);
        let t0 = Instant::now();
        let turns = turns.max(1);
        let mut decision = 0i8;
        let mut conf = 0.0f32;
        for turn in 0..turns {
            self.carry_hidden = turn > 0;
            let (trit, c, _ns) = self.forward(&x);
            conf = c;
            // LOGIC (per turn): only a confident read is a decision; otherwise the turn withholds
            // and the loop re-deliberates with the hidden state carried.
            decision = if c >= 0.6 { trit } else { 0 };
        }
        (decision, conf, t0.elapsed().as_nanos() as u64)
    }
}

#[inline]
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + fast_exp(-x))
}

#[inline]
fn fast_exp(x: f32) -> f32 {
    let clamped = x.clamp(-88.0, 88.0);
    let i = (clamped * 1.442_695_040_888_963_4 + 127.0) as i32;
    f32::from_bits((i as u32) << 23)
}

/// What the trit means, in the platform's own words (kept beside the model so no handler invents it).
pub fn trit_label(t: i8) -> &'static str {
    match t {
        1 => "PASS (+1)",
        0 => "AMBIGUOUS (0)",
        _ => "FAULT (-1)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The runtime model must stay a single-digit-millisecond evaluator or it cannot be the in-kernel
    /// tier. Measures the forward latency on this host (a receipt, not an estimate) and pins a loose
    /// ceiling so a pathological regression is caught — not a change-detector, a contract.
    #[test]
    fn forward_is_microsecond_scale() {
        let mut m = Trm7m::new();
        let x = Trm7m::encode_text("LpxC zinc-dependent deacetylase inhibitor TRM-ABX-082");
        for _ in 0..2000 {
            m.forward(&x); // warm the caches
        }
        let n = 50_000u64;
        let t = Instant::now();
        for _ in 0..n {
            m.forward(&x);
        }
        let us = t.elapsed().as_secs_f64() * 1e6 / n as f64;
        eprintln!("[trm7m] forward latency {:.2} us/call over {n} calls", us);
        let (_, _, reported_ns) = m.forward(&x);
        eprintln!("[trm7m] single reported latency_ns {reported_ns}");
        assert!(us < 10_000.0, "trm7m forward regressed to {us:.1} us/call");
    }
}
