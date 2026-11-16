//! swarm-dequant — GGUF quant dequantization in Rust (Q8_0 + Q4_K).
//!
//! Faithful to llama.cpp's block layouts (ggml-common.h): Q8_0 = 32 int8 + f16 delta (34 B);
//! Q4_K = 256 values, 8 sub-blocks of 32, 2 f16 (d, dmin) + 12 bytes of 6-bit scales/mins +
//! 128 bytes of 4-bit quants (144 B). Dequant: x = q*d*sc - dmin*m.

pub mod mla;

/// IEEE 754 half (binary16) -> f32.
pub fn f16_to_f32(h: u16) -> f32 {
    let sign = ((h >> 15) & 1) as u32;
    let exp = ((h >> 10) & 0x1F) as u32;
    let frac = (h & 0x3FF) as u32;
    if exp == 0 {
        if frac == 0 {
            return if sign == 0 { 0.0 } else { -0.0 };
        }
        let m = frac as f32 / 1024.0 * 2f32.powi(-14);
        return if sign == 1 { -m } else { m };
    }
    if exp == 0x1F {
        return if frac == 0 {
            if sign == 0 { f32::INFINITY } else { f32::NEG_INFINITY }
        } else {
            f32::NAN
        };
    }
    let m = 1.0 + frac as f32 / 1024.0;
    let e = 2f32.powi(exp as i32 - 15);
    if sign == 1 { -m * e } else { m * e }
}

/// The exact llama.cpp get_scale_min_k4 unpack: scales[0..3] hold scales 0-3 (low 6 bits) and mins
/// 0-3 in scales[4..7]; scales 4-7 + mins 4-7 are bit-split across the same 12 bytes.
fn scale_min_k4(scales: &[u8], j: usize) -> (f32, f32) {
    if j < 4 {
        ((scales[j] & 63) as f32, (scales[j + 4] & 63) as f32)
    } else {
        (
            (((scales[j + 4] & 0xF) as u32) | (((scales[j - 4] >> 6) as u32) << 4)) as f32,
            (((scales[j + 4] >> 4) as u32) | (((scales[j] >> 6) as u32) << 4)) as f32,
        )
    }
}

/// Dequantize Q8_0 (block 32: f16 delta + 32 int8). `n_vals` total floats.
pub fn dequant_q8_0(data: &[u8], n_vals: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(n_vals);
    let mut off = 0;
    while out.len() < n_vals {
        let d = f16_to_f32(u16::from_le_bytes([data[off], data[off + 1]]));
        off += 2;
        for _ in 0..32 {
            let q = data[off] as i8 as f32;
            off += 1;
            out.push(q * d);
        }
    }
    out.truncate(n_vals);
    out
}

/// Dequantize Q5_0 (block 32: f16 delta + 4 B of 5th-bits (u32 LE) + 16 B of low nibbles).
/// x = ((low4 | (bit5 << 4)) - 16) * d. Block = 22 bytes.
/// ggml block_q5_0: d(fp16) + qh(4B) + qs(16B) = 22 bytes for 32 values.
/// Layout: ggml's quantize_row_q5_0 writes qs[j] = (low nibble = element j) | (high nibble =
/// element j+16), and stores element j's 5th bit at qh bit j and element j+16's at qh bit j+16.
/// dequantize_row_q5_0 therefore fills y[j] then y[j + 16] — the block is
/// [16 low-nibble values][16 high-nibble values], NOT interleaved. The previous version
/// interleaved low/high nibbles and read the second half's extra bit from qh>>(j+1) instead of
/// qh>>(j+16); that permuted every 32-value block. It silently corrupted ffn_down_exps on the 14
/// Q5_0 layers of amd.Instella-MoE-16B-A3B-Think.Q4_K_M, which is the structural residual that
/// localised at layer 3 (routed-branch rel_l2 0.7100 -> 0.0340 after this fix).
pub fn dequant_q5_0(data: &[u8], n_vals: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; n_vals];
    let mut base = 0usize; // index of this block's first value
    let mut off = 0usize;  // byte offset of this block
    while base < n_vals {
        let d = f16_to_f32(u16::from_le_bytes([data[off], data[off + 1]]));
        let qh = u32::from_le_bytes([data[off + 2], data[off + 3], data[off + 4], data[off + 5]]);
        for j in 0..16usize {
            let qs = data[off + 6 + j];
            let xh0 = (((qh >> j) << 4) & 0x10) as u8;
            let xh1 = ((qh >> (j + 12)) & 0x10) as u8;
            let x0 = (((qs & 0x0F) | xh0) as i32 - 16) as f32;
            let x1 = (((qs >> 4) | xh1) as i32 - 16) as f32;
            let i0 = base + j;
            let i1 = base + j + 16;
            if i0 < n_vals { out[i0] = x0 * d; }
            if i1 < n_vals { out[i1] = x1 * d; }
        }
        base += 32;
        off += 22;
    }
    out
}

/// Dequantize Q6_K (super-block 256: 128 B low-nibbles + 64 B high-bits + 16 int8 scales + 2 f16 d).
/// 6-bit value = low4 | (high2 << 4) - 32. Block = 210 bytes; d is the LAST 2 bytes.
pub fn dequant_q6_k(data: &[u8], n_vals: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(n_vals);
    let mut off = 0;
    while out.len() < n_vals {
        let d = f16_to_f32(u16::from_le_bytes([data[off + 208], data[off + 209]]));
        for n in 0..2 {
            let ql_base = off + n * 64;
            let qh_base = off + 128 + n * 32;
            let sc_base = off + 192 + n * 8;
            // four groups of 32: q1 (l positions 0-31), q2 (32-63), q3 (64-95), q4 (96-127)
            for l in 0..32 {
                let is = l / 16;
                let q1 = (((data[ql_base + l] & 0xF) as i32) | (((data[qh_base + l] >> 0) & 3) as i32) << 4) as i8 - 32;
                out.push(d * data[sc_base + is] as i8 as f32 * q1 as f32);
            }
            for l in 0..32 {
                let is = l / 16;
                let q2 = (((data[ql_base + l + 32] & 0xF) as i32) | (((data[qh_base + l] >> 2) & 3) as i32) << 4) as i8 - 32;
                out.push(d * data[sc_base + is + 2] as i8 as f32 * q2 as f32);
            }
            for l in 0..32 {
                let is = l / 16;
                let q3 = (((data[ql_base + l] >> 4) as i32) | (((data[qh_base + l] >> 4) & 3) as i32) << 4) as i8 - 32;
                out.push(d * data[sc_base + is + 4] as i8 as f32 * q3 as f32);
            }
            for l in 0..32 {
                let is = l / 16;
                let q4 = (((data[ql_base + l + 32] >> 4) as i32) | (((data[qh_base + l] >> 6) & 3) as i32) << 4) as i8 - 32;
                out.push(d * data[sc_base + is + 6] as i8 as f32 * q4 as f32);
            }
        }
        off += 210;
    }
    out.truncate(n_vals);
    out
}

/// Quantize x to Q8_K (llama.cpp's matmul-input quant): 256-value blocks, per-block scale =
/// max_abs/127, q = round(x/scale) clamped to [-127,127]. Returns the DEQUANTIZED values
/// (q * scale) — what ggml_vec_dot_q4_K_q8_K effectively multiplies against.
pub fn quantize_q8_k(x: &[f32]) -> Vec<f32> {
    let mut out = vec![0.0f32; x.len()];
    let mut bi = 0;
    for blk in x.chunks(256) {
        let max_abs = blk.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        let scale = (max_abs / 127.0).max(1e-9);
        for (i, &v) in blk.iter().enumerate() {
            let q = (v / scale).round().clamp(-127.0, 127.0);
            out[bi * 256 + i] = q * scale;
        }
        bi += 1;
    }
    out
}

/// Q8_0 input quant (for Q8_0 weights): 32-value blocks, scale=max_abs/127, no offset.
pub fn quantize_q8_0(x: &[f32]) -> Vec<f32> {
    let mut out = vec![0.0f32; x.len()];
    for (bi, blk) in x.chunks(32).enumerate() {
        let max_abs = blk.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        let scale = (max_abs / 127.0).max(1e-9);
        for (i, &v) in blk.iter().enumerate() {
            let q = (v / scale).round().clamp(-127.0, 127.0);
            out[bi * 32 + i] = q * scale;
        }
    }
    out
}

/// Round an f32 to the nearest f16-representable value: what GGML_FP32_TO_FP16 does (round-to-nearest-even).
/// llama.cpp's flash-attention path casts k and v from F32 to F16 before the fused kernel
/// (build_attn_mha), so a faithful reimplementation must round those operands the same way.
pub fn f16_round(x: f32) -> f32 {
    // Builds the real f16 bit pattern and converts back. An earlier version rebuilt an f32
    // exponent from the f16 exponent, which returned denormals near 1e-35 for ordinary values
    // and silently destroyed every rounded operand. The sandbox copy was correct; this matches it.
    let b = x.to_bits();
    let sign = b & 0x8000_0000;
    let exp = ((b >> 23) & 0xFF) as i32;
    let man = b & 0x007F_FFFF;
    if exp == 0xFF { return x; }
    let e = exp - 127 + 15;
    if e >= 0x1F { return f32::from_bits(sign | 0x7F80_0000); }
    if e <= 0 {
        if e < -10 { return f32::from_bits(sign); }
        let m = man | 0x0080_0000;
        let shift = (14 - e) as u32;
        let mut h = m >> shift;
        let rem = m & ((1u32 << shift) - 1);
        let half = 1u32 << (shift - 1);
        if rem > half || (rem == half && (h & 1) == 1) { h += 1; }
        return f16_to_f32(((sign >> 16) as u16) | h as u16);
    }
    let mut h = ((e as u32) << 10) | (man >> 13);
    let rem = man & 0x1FFF;
    if rem > 0x1000 || (rem == 0x1000 && (h & 1) == 1) { h += 1; }
    f16_to_f32(((sign >> 16) as u16) | h as u16)
}

/// Q8_1 input quant (for Q5_0 weights), matching ggml's quantize_row_q8_1 exactly:
/// SYMMETRIC amax/127 scale, NO mean offset, and the scale kept in f16 like ggml's block.
///   amax = max|x| ; d = f16(amax/127) ; q = round(x/d) ; dequant = q*d
/// The earlier midrange+mean-offset scheme was compensating for missing f16 rounding of the
/// attention operands, not matching llama.cpp. With f16 k/v rounding in place, this exact form is
/// both principled and the best of the configurations measured against the fork's own kq taps.
pub fn quantize_q8_1(x: &[f32]) -> Vec<f32> {
    let mut out = vec![0.0f32; x.len()];
    for (bi, blk) in x.chunks(32).enumerate() {
        let amax = blk.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        if amax == 0.0 {
            for (i, _) in blk.iter().enumerate() { out[bi * 32 + i] = 0.0; }
            continue;
        }
        let d = f16_round(amax / 127.0);
        let inv = if d != 0.0 { 1.0 / d } else { 0.0 };
        for (i, &v) in blk.iter().enumerate() {
            out[bi * 32 + i] = (v * inv).round().clamp(-127.0, 127.0) * d;
        }
    }
    out
}

/// Quantize a matmul INPUT the way llama.cpp does for a WEIGHT of the given ggml_type:
/// 6=Q5_0 -> Q8_1 (mean-offset); 8=Q8_0 -> Q8_0 (32-blk); 12/14=Q4_K/Q6_K -> Q8_K (256-blk);
/// else (F32/F16) -> no quantization (identity).
pub fn q_input(x: &[f32], wtype: u32) -> Vec<f32> {
    match wtype {
        6 => quantize_q8_1(x),
        8 => quantize_q8_0(x),
        12 | 14 => quantize_q8_k(x),
        _ => x.to_vec(),
    }
}

/// 8-lane FMA dot, matching llama.cpp's ggml_vec_dot_f32 accumulation order: one `_mm256` accumulator
/// (8 lanes in parallel), fused multiply-add, trailing horizontal sum. The scalar 1-accumulator loop
/// rounds differently (separate mul+add, sequential partials), and that difference compounds across the
/// 27 layers into the measured 0.837 residual. This is the PARITY fix, not a speed fix.
#[inline]
pub fn dot_fma(x: &[f32], y: &[f32]) -> f32 {
    let n = x.len().min(y.len());
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::x86_64::*;
        unsafe {
            let mut sum = _mm256_setzero_ps();
            let mut i = 0usize;
            while i + 8 <= n {
                let vx = _mm256_loadu_ps(x.as_ptr().add(i));
                let vy = _mm256_loadu_ps(y.as_ptr().add(i));
                sum = _mm256_fmadd_ps(vx, vy, sum);
                i += 8;
            }
            let mut arr = [0.0f32; 8];
            _mm256_storeu_ps(arr.as_mut_ptr(), sum);
            let mut total = 0.0f32;
            for k in 0..8 {
                total += arr[k];
            }
            while i < n {
                total += x[i] * y[i];
                i += 1;
            }
            total
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        (0..n).map(|i| x[i] * y[i]).sum()
    }
}


/// Direct Q8_0 quantized dot: one 32-value block (f16 scale d + 32 int8 qs) against f32 x,
/// WITHOUT dequantizing the weights to f32. x is quantized to Q8_0 on the fly (scale max_abs/127),
/// and the dot runs in int8 (llama.cpp's ggml_vec_dot_q8_0_q8_0 shape): d * d8 * sum(qs[i] * q8[i]).
/// AVX2 uses the sign-trick maddubs path (vpmaddubsw): abs(x) x sign(y) -> 32 8-bit MACs/cycle.
#[inline]
pub fn dot_q8_0_f32(qs: &[u8], d: f32, x: &[f32]) -> f32 {
    let mut amax = 0.0f32;
    for &v in x {
        amax = amax.max(v.abs());
    }
    let d8 = (amax / 127.0).max(1e-9);
    // quantize x to int8 (the Q8_0 input)
    let mut xq = [0i8; 32];
    for i in 0..32 {
        xq[i] = ((x[i] / d8).round() as i32).clamp(-127, 127) as i8;
    }
    // AVX2 sign-trick maddubs path (target-cpu=native enables avx2 globally, so the intrinsics
    // inline here without a #[target_feature] call boundary). abs(qx) x sign(qy) == qx x qy, and
    // abs keeps the maddubs pair sums <= 32258 < 32767, so no i16 saturation.
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::x86_64::*;
        unsafe {
            let qx = _mm256_loadu_si256(qs.as_ptr() as *const __m256i);
            let qy = _mm256_loadu_si256(xq.as_ptr() as *const __m256i);
            let ax = _mm256_sign_epi8(qx, qx);
            let sy = _mm256_sign_epi8(qy, qx);
            let dot = _mm256_maddubs_epi16(ax, sy);
            let summed = _mm256_madd_epi16(_mm256_set1_epi16(1), dot);
            let hi = _mm256_extracti128_si256(summed, 1);
            let lo = _mm256_castsi256_si128(summed);
            let s = _mm_add_epi32(lo, hi);
            let s2 = _mm_add_epi32(s, _mm_srli_si128(s, 8));
            let s3 = _mm_add_epi32(s2, _mm_srli_si128(s2, 4));
            return d * d8 * (_mm_cvtsi128_si32(s3) as f32);
        }
    }
    let mut acc: i32 = 0;
    for i in 0..32 {
        acc += (qs[i] as i8 as i32) * (xq[i] as i32);
    }
    d * d8 * (acc as f32)
}

/// Full Q8_0 GEMV: out[j] = sum over 32-value blocks of the quantized dot, over the raw Q8_0
/// tensor bytes (n_blocks * 34 each: f16 d + 32 int8). in_dim must be a multiple of 32.
pub fn matmul_q8_0(raw: &[u8], in_dim: usize, out_dim: usize, x: &[f32]) -> Vec<f32> {
    let blocks = in_dim / 32;
    let mut out = vec![0.0f32; out_dim];
    for j in 0..out_dim {
        let mut acc = 0.0f32;
        for b in 0..blocks {
            let off = (j * blocks + b) * 34;
            let d = f16_to_f32(u16::from_le_bytes([raw[off], raw[off + 1]]));
            acc += dot_q8_0_f32(&raw[off + 2..off + 34], d, &x[b * 32..b * 32 + 32]);
        }
        out[j] = acc;
    }
    out
}

/// Dequantize Q4_K (super-block 256: 2 f16 + 12 B 6-bit scales/mins + 128 B 4-bit quants).
pub fn dequant_q4_k(data: &[u8], n_vals: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(n_vals);
    let mut off = 0;
    while out.len() < n_vals {
        let d = f16_to_f32(u16::from_le_bytes([data[off], data[off + 1]]));
        let dmin = f16_to_f32(u16::from_le_bytes([data[off + 2], data[off + 3]]));
        off += 4;
        let scales = &data[off..off + 12];
        off += 12;
        // llama.cpp: 4 groups of 64 values; each group = 32 bytes, low nibble -> sub-block 2*it,
        // high nibble -> sub-block 2*it+1.
        for it in 0..4 {
            let base = it * 32;
            let (sc0, m0) = scale_min_k4(scales, 2 * it);
            let (sc1, m1) = scale_min_k4(scales, 2 * it + 1);
            let d1 = d * sc0;
            let min1 = dmin * m0;
            let d2 = d * sc1;
            let min2 = dmin * m1;
            for l in 0..32 {
                out.push(d1 * (data[off + base + l] & 0x0F) as f32 - min1);
            }
            for l in 0..32 {
                out.push(d2 * (data[off + base + l] >> 4) as f32 - min2);
            }
        }
        off += 128;
    }
    out.truncate(n_vals);
    out
}

/// Quantize x to Q8_K int8 (256-blk, scale = max_abs/127) returning the raw int8 and the block scale.
fn quantize_q8_k_int(x: &[f32]) -> (Vec<i8>, f32) {
    let max_abs = x.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
    let scale = (max_abs / 127.0).max(1e-9);
    let qs: Vec<i8> = x.iter().map(|&v| (v / scale).round().clamp(-127.0, 127.0) as i8).collect();
    (qs, scale)
}

/// The int8 Q4_K dot, matching llama.cpp's ggml_vec_dot_q4_K_q8_K: the int32 Σ(q4*q8) is EXACT
/// (vs the dequant-to-f32 which rounds each element), then the f32 scale is applied once per sub-block.
/// q4_data is the raw 144-byte Q4_K blocks; x is the 256*n input quantized to Q8_K on the fly.
pub fn dot_q4_K_q8_K(q4_data: &[u8], x: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    let mut off = 0usize;
    let mut x_off = 0usize;
    while off + 144 <= q4_data.len() && x_off + 256 <= x.len() {
        let d = f16_to_f32(u16::from_le_bytes([q4_data[off], q4_data[off + 1]]));
        let dmin = f16_to_f32(u16::from_le_bytes([q4_data[off + 2], q4_data[off + 3]]));
        let scales = &q4_data[off + 4..off + 16];
        let q4 = &q4_data[off + 16..off + 144];
        let (xq, xd) = quantize_q8_k_int(&x[x_off..x_off + 256]);
        for it in 0..4 {
            let (sc0, m0) = scale_min_k4(scales, 2 * it);
            let (sc1, m1) = scale_min_k4(scales, 2 * it + 1);
            let base = it * 32;
            let mut sumi0 = 0i32;
            let mut sumq0 = 0i32;
            for l in 0..32 {
                let qv = (q4[base + l] & 0x0F) as i32;
                let q8 = xq[it * 64 + l] as i32;
                sumi0 += qv * q8;
                sumq0 += q8;
            }
            sum += xd * (d * sc0 * sumi0 as f32 - dmin * m0 * sumq0 as f32);
            let mut sumi1 = 0i32;
            let mut sumq1 = 0i32;
            for l in 0..32 {
                let qv = (q4[base + l] >> 4) as i32;
                let q8 = xq[it * 64 + 32 + l] as i32;
                sumi1 += qv * q8;
                sumq1 += q8;
            }
            sum += xd * (d * sc1 * sumi1 as f32 - dmin * m1 * sumq1 as f32);
        }
        off += 144;
        x_off += 256;
    }
    sum
}

pub fn silu(x: f32) -> f32 {
    x / (1.0 + (-x).exp())
}

/// softplus(x) = ln(1 + e^x). ggml's op_softplus is `(x > 20) ? x : log(1+exp(x))`; the plain
/// form agrees to float precision, so the threshold only guards exp() overflow above x≈88.
pub fn softplus(x: f32) -> f32 {
    (1.0 + x.exp()).ln()
}

/// sigmoid(x) = 1/(1+e^-x).
pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// The router's gating function. GGUF metadata for amd.Instella-MoE-16B-A3B-Think.Q4_K_M says
/// instella-moe.expert_gating_func = 2, and llama.cpp's llama_expert_gating_func_type maps
/// 2 -> SIGMOID (SOFTMAX=1, SIGMOID=2, SOFTMAX_WEIGHT=3, SQRT_SOFTPLUS=4). build_moe_ffn then
/// does: probs = sigmoid(logits); selection bias added ONLY for top-k choice; top-k; weights =
/// unbiased probs[selected]; normalize by sum (clamped 6.1e-5); scale by 2.5.
/// MOE_GATING=sqrtsoftplus restores the old guess as a control. Measured on the 16-token prompt:
/// sigmoid reproduces the fork's 16 tokens exactly, sqrt-softplus breaks at gen 10.
pub fn moe_gating_is_sigmoid() -> bool {
    !matches!(std::env::var("MOE_GATING").as_deref(), Ok("sqrtsoftplus"))
}

/// RMS norm (LLM_NORM_RMS): x / sqrt(mean(x^2) + eps), element-wise scaled by the weight.
pub fn rms_norm(x: &[f32], weight: &[f32], eps: f32) -> Vec<f32> {
    let n = x.len();
    let mean_sq = x.iter().map(|&v| (v as f64) * (v as f64)).sum::<f64>() / n as f64;
    let inv = 1.0 / ((mean_sq + eps as f64).sqrt() as f32);
    x.iter().zip(weight.iter()).map(|(&v, &w)| v * inv * w).collect()
}

/// Softmax (numerically stable) over a logit vector — the sampler's core.
pub fn softmax(logits: &[f32]) -> Vec<f32> {
    let mx = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|&l| (l - mx).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&e| e / sum).collect()
}

/// Rotary position embedding (standard, no YaRN): rotate the last `n_rot` dims of `x` by
/// pos / freq_base^(2i/n_rot). Matches ggml_rope for positions within the original context.
pub fn rope(x: &[f32], pos: usize, freq_base: f32) -> Vec<f32> {
    rope_scaled(x, pos, freq_base, 0.025)
}

/// Plain RoPE with an explicit freq_scale. `rope` = this at 0.025 (the distill's original guess).
pub fn rope_scaled(x: &[f32], pos: usize, freq_base: f32, freq_scale: f32) -> Vec<f32> {
    let n = x.len();
    let mut out = x.to_vec();
    for i in 0..n / 2 {
        // angle = freq_scale * pos * base^(-2i/n)
        let theta = freq_scale * pos as f32 / freq_base.powf(2.0 * i as f32 / n as f32);
        let (c, s) = (theta.cos(), theta.sin());
        let (x0, x1) = (out[2 * i], out[2 * i + 1]);
        out[2 * i] = x0 * c - x1 * s;
        out[2 * i + 1] = x0 * s + x1 * c;
    }
    out
}

/// llama.cpp's YaRN rope — the same math as ggml's rope_yarn + ggml_rope_yarn_corr_dims:
///   corr_dim(n_rot) = n_dims * ln(n_ctx_orig / (n_rot*2*pi)) / (2*ln(base))
///   theta_extrap = pos * base^(-2i/n_dims); theta_interp = freq_scale * theta_extrap
///   theta = theta_interp*(1-ramp) + theta_extrap*ramp, ramp = 1 - clamp((i-lo)/(hi-lo),0,1)
///   cos/sin multiplied by mscale
/// This is the fix measured on 2026-10-31: with the ramp the Rust forward reproduces the fork's
/// 16 greedy tokens exactly; without it the same run diverges at gen 6. The plain `rope` above
/// omits the ramp, which leaves pairs 0-3 with angles 40x too small (corr_dims = [3,7]).
pub fn yarn_rope(
    x: &[f32],
    pos: usize,
    freq_base: f32,
    freq_scale: f32,
    n_ctx_orig: f32,
    beta_fast: f32,
    beta_slow: f32,
    mscale: f32,
) -> Vec<f32> {
    let n = x.len();
    let n_dims = n as f32;
    let corr_dim = |n_rot: f32| -> f32 {
        n_dims * (n_ctx_orig / (n_rot * 2.0 * std::f32::consts::PI)).ln() / (2.0 * freq_base.ln())
    };
    let lo = corr_dim(beta_fast).floor().max(0.0);
    let hi = corr_dim(beta_slow).ceil().min(n_dims - 1.0);
    let mut out = x.to_vec();
    for i in 0..n / 2 {
        let theta_extrap = pos as f32 / freq_base.powf(2.0 * i as f32 / n_dims);
        let theta_interp = freq_scale * theta_extrap;
        let y = ((2 * i) as f32 / 2.0 - lo) / 0.001f32.max(hi - lo);
        let ramp = 1.0 - y.min(1.0).max(0.0);
        let theta = theta_interp * (1.0 - ramp) + theta_extrap * ramp;
        let (c, s) = (theta.cos() * mscale, theta.sin() * mscale);
        let (x0, x1) = (out[2 * i], out[2 * i + 1]);
        out[2 * i] = x0 * c - x1 * s;
        out[2 * i + 1] = x0 * s + x1 * c;
    }
    out
}

/// The Instella MoE router: probs = sqrt(softplus(h @ gate_inp)); select top-n by
/// probs + bias (the DeepSeek-V3 selection bias, added AFTER the sqrt-softplus); the routed
/// weights use the UNBIASED probs, normalized + scaled. Returns (top experts, routed weights).
pub fn moe_router(
    h: &[f32],
    gate_inp: &[f32],
    bias: &[f32],
    n_experts: usize,
    n_used: usize,
    w_scale: f32,
    q8: bool,
) -> (Vec<usize>, Vec<f32>) {
    let (top, weights, _) = moe_router_probs(h, gate_inp, bias, n_experts, n_used, w_scale, q8);
    (top, weights)
}

/// Same as [`moe_router`] but also returns the raw (un-normalised, un-scaled) gating probs.
/// llama.cpp dumps `ffn_moe_weights` BEFORE `expert_weights_norm` and `expert_weights_scale`
/// (llama-graph.cpp build_moe_ffn), so comparing a crate weight vector against that dump is
/// apples-to-oranges; the raw probs vector is the only like-for-like comparison.
pub fn moe_router_probs(
    h: &[f32],
    gate_inp: &[f32],
    bias: &[f32],
    n_experts: usize,
    n_used: usize,
    w_scale: f32,
    q8: bool,
) -> (Vec<usize>, Vec<f32>, Vec<f32>) {
    let hidden = h.len();
    let hq: Vec<f32> = if q8 { crate::quantize_q8_k(h) } else { h.to_vec() };
    let sig = moe_gating_is_sigmoid();
    let mut probs = vec![0.0f32; n_experts];
    for e in 0..n_experts {
        let lg = crate::dot_fma(&hq, &gate_inp[e * hidden..e * hidden + hidden]);
        // llama.cpp build_moe_ffn: SIGMOID -> ggml_sigmoid(logits); SQRT_SOFTPLUS -> sqrt(softplus).
        probs[e] = if sig { sigmoid(lg) } else { softplus(lg).sqrt() };
    }
    let mut order: Vec<usize> = (0..n_experts).collect();
    order.sort_by(|&a, &b| {
        (probs[b] + bias[b]).partial_cmp(&(probs[a] + bias[a])).unwrap_or(std::cmp::Ordering::Equal)
    });
    let top: Vec<usize> = order[..n_used].to_vec();
    let sum: f32 = top.iter().map(|&e| probs[e]).sum();
    let weights: Vec<f32> = top.iter().map(|&e| probs[e] / sum.max(1e-5) * w_scale).collect();
    (top, weights, probs)
}

/// The Instella gated expert FFN (parallel gate/up + SiLU + down), 2048 -> 1408 -> 2048.
pub fn expert_ffn(
    h: &[f32],
    w_gate: &[f32],
    w_up: &[f32],
    w_down: &[f32],
    hidden: usize,
    expert_dim: usize,
    q8: bool,
    down_type: u32,
) -> Vec<f32> {
    let hq: Vec<f32> = if q8 { crate::quantize_q8_k(h) } else { h.to_vec() };
    let mut act = vec![0.0f32; expert_dim];
    for j in 0..expert_dim {
        let g = crate::dot_fma(&hq, &w_gate[j * hidden..j * hidden + hidden]);
        let u = crate::dot_fma(&hq, &w_up[j * hidden..j * hidden + hidden]);
        act[j] = silu(g) * u;
    }
    let actq: Vec<f32> = if q8 { crate::q_input(&act, down_type) } else { act.clone() };
    let mut out = vec![0.0f32; hidden];
    for j in 0..hidden {
        out[j] = crate::dot_fma(&actq, &w_down[j * expert_dim..j * expert_dim + expert_dim]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rms_norm_unit() {
        let x = [1.0f32, 1.0, 1.0, 1.0];
        let w = [1.0f32, 1.0, 1.0, 1.0];
        let out = rms_norm(&x, &w, 1e-6);
        // mean_sq = 1, inv = 1/sqrt(1+1e-6) ~ 1
        assert!((out[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn rms_norm_scales() {
        let x = [2.0f32, 0.0, 0.0, 0.0];
        let w = [1.0f32, 1.0, 1.0, 1.0];
        let out = rms_norm(&x, &w, 1e-6);
        // mean_sq = 4/4 = 1, so inv ~ 1, out[0] ~ 2
        assert!((out[0] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn softmax_sums_to_one() {
        let p = softmax(&[1.0, 2.0, 3.0]);
        let s: f32 = p.iter().sum();
        assert!((s - 1.0).abs() < 1e-5);
        assert!(p[2] > p[1] && p[1] > p[0]);
    }

    #[test]
        fn silu_zero() {
            assert!((silu(0.0) - 0.0).abs() < 1e-6);
        }
    }

    /// Env-gated activation tap writer, the committed replacement for the lost dumper that produced
    /// `logs_cfg_k1_v1_q1`. That tree was written by a binary nobody committed, which made every tap
    /// comparison unreproducible - the provenance hole recorded in the 2026-11-02 checkpoint.
    ///
    /// `DUMP_TAPS=1` (default off) writes `<name>-<layer>-<pos> n=<len>` then one value per line, the shape
    /// the fork dumper and the existing comparators expect. Layer 0 additionally writes the legacy short
    /// alias `<name>-<pos>` so scratch/stage_compare.py and scratch/layer_decomp.py keep working unchanged.
    /// `DUMP_TAPS_DIR` overrides the output directory (default `logs/taps`).
    pub fn tap_write(name: &str, layer: usize, pos: usize, data: &[f32]) {
        if std::env::var("DUMP_TAPS").map(|v| v == "0").unwrap_or(true) {
            return;
        }
        let dir = std::env::var("DUMP_TAPS_DIR").unwrap_or_else(|_| "logs/taps".to_string());
        let _ = std::fs::create_dir_all(&dir);
        let mut body = String::with_capacity(data.len() * 12 + 32);
        body.push_str(&format!("{name}-{layer}-{pos} n={}\n", data.len()));
        for v in data {
            body.push_str(&format!("{v:.9}\n"));
        }
        let _ = std::fs::write(format!("{dir}/{name}-{layer}-{pos}.txt"), &body);
        if layer == 0 {
            body.replace_range(..body.find(' ').unwrap_or(body.len()), &format!("{name}-{pos}"));
            let _ = std::fs::write(format!("{dir}/{name}-{pos}.txt"), &body);
        }
    }
