// probe: verify dot_q4_K_q8_K (int8) matches the CURRENT path (dequant_q4_k + quantize_q8_k + f32 dot)
fn splitmix(seed: &mut u64) -> u64 { *seed = seed.wrapping_add(0x9E3779B97F4A7C15); let mut z = *seed; z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9); z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB); z ^ (z >> 31) }
fn scalar(x: &[f32], y: &[f32]) -> f32 { let mut a = 0.0f32; for i in 0..x.len() { a += x[i] * y[i]; } a }
fn main() {
    let mut seed = 7u64;
    let mut max = 0.0f32; let mut sum = 0.0f64;
    for _t in 0..32 {
        let mut blk = vec![0u8; 144];
        blk[0] = (0.01f32.to_bits() >> 16) as u8; blk[1] = (0.01f32.to_bits() >> 24) as u8;
        blk[2] = (0.005f32.to_bits() >> 16) as u8; blk[3] = (0.005f32.to_bits() >> 24) as u8;
        for i in 4..16 { blk[i] = (splitmix(&mut seed) & 0xFF) as u8; }
        for i in 16..144 { blk[i] = (splitmix(&mut seed) & 0xFF) as u8; }
        let x: Vec<f32> = (0..256).map(|_| ((splitmix(&mut seed) as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32).collect();
        // reference = CURRENT path: dequant_q4_k + quantize_q8_k (dequantized) + f32 dot
        let wref = swarm_dequant::dequant_q4_k(&blk, 256);
        let xq = swarm_dequant::quantize_q8_k(&x);
        let s = scalar(&wref, &xq);
        // int8 dot
        let f = swarm_dequant::dot_q4_K_q8_K(&blk, &x);
        let dabs = (s - f).abs();
        max = max.max(dabs); sum += dabs as f64;
    }
    println!("dequant+q8k+f32 vs dot_q4_K_q8_K: max_abs={max:.6e} mean_abs={:.6e} (relative ~{:.6e})", sum / 32.0, (sum / 32.0) / 10.0);
}
