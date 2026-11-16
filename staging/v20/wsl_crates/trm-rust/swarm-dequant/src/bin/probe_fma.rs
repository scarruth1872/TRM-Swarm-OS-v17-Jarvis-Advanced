// probe: scalar 1-accumulator vs dot_fma 8-lane FMA (deterministic SplitMix64)
fn splitmix(seed: &mut u64) -> u64 { *seed = seed.wrapping_add(0x9E3779B97F4A7C15); let mut z = *seed; z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9); z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB); z ^ (z >> 31) }
fn scalar(x: &[f32], y: &[f32]) -> f32 { let mut a = 0.0f32; for i in 0..x.len() { a += x[i] * y[i]; } a }
fn main() {
    let mut seed = 42u64;
    let n = 2048usize;
    let mut max = 0.0f32; let mut sum = 0.0f64;
    for _t in 0..64 {
        let x: Vec<f32> = (0..n).map(|_| ((splitmix(&mut seed) as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32).collect();
        let y: Vec<f32> = (0..n).map(|_| ((splitmix(&mut seed) as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32).collect();
        let s = scalar(&x, &y);
        let f = swarm_dequant::dot_fma(&x, &y);
        let d = (s - f).abs();
        max = max.max(d); sum += d as f64;
    }
    println!("scalar vs dot_fma (2048-dim, 64 trials): max_abs={max:.6e} mean_abs={:.6e}", sum / 64.0);
}
