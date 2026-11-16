//! FUNCTIONAL benchmarks for the real kernel primitives.
extern crate alloc;
mod k3_logic; mod turbovec; mod float_math; mod time;
use k3_logic::KleeneTernaryEngine as K3;
use turbovec::TurboVec158Quantizer as TV;
use std::time::Instant;

fn lcg(s: &mut u64) -> u64 { *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); *s >> 33 }

fn main() {
    let mut s = 0x12345678u64;

    println!("A. K3 CLEAN CHANNEL (safety property on real trit streams)");
    for n in [4usize, 16, 64, 256] {
        let mut vetoed = 0; let trials = 20000;
        for _ in 0..trials {
            let mut xs: Vec<i8> = (0..n).map(|_| if lcg(&mut s)%4==0 {0} else {1}).collect();
            let idx = (lcg(&mut s) as usize) % n;
            xs[idx] = -1;
            let mut acc = xs[0];
            for x in &xs[1..] { acc = K3::kleene_and(acc, *x); }
            if acc == -1 { vetoed += 1; }
        }
        println!("   K={:>3}  veto rate {}/{} = {:.2}%   (must be 100%)", n, vetoed, trials,
                 100.0 * vetoed as f64 / trials as f64);
    }

    println!("B. K3 THROUGHPUT (1e6 kleene_and over a real trit stream)");
    let stream: Vec<i8> = (0..1_000_000).map(|_| (lcg(&mut s)%3) as i8 - 1).collect();
    let t0 = Instant::now();
    let mut acc = stream[0];
    for x in &stream[1..] { acc = K3::kleene_and(acc, *x); }
    let dt = t0.elapsed();
    println!("   {:?} -> {:.1} M ANDs/sec   (acc={})", dt, 1.0/dt.as_secs_f64(), acc);

    println!("C. TURBOVEC 1.58-bit ROUND-TRIP FIDELITY");
    let dim = 256usize; let reps = 2000;
    let mut worst = 1.0f64; let mut sum = 0.0f64; let mut cnt = 0u32;
    for _ in 0..reps {
        let v: Vec<f32> = (0..dim).map(|_| (lcg(&mut s) as f64 / u32::MAX as f64 * 2.0 - 1.0) as f32).collect();
        let n = (v.iter().map(|x| (*x as f64)*(*x as f64)).sum::<f64>()).sqrt();
        if n == 0.0 { continue; }
        let vn: Vec<f32> = v.iter().map(|x| (*x as f64 / n) as f32).collect();
        let (packed, scale) = TV::quantize_158(&vn);
        let deq = TV::dequantize_158(&packed, dim, scale);
        let dn = (deq.iter().map(|x| (*x as f64)*(*x as f64)).sum::<f64>()).sqrt();
        if dn == 0.0 { continue; }
        let dot: f64 = vn.iter().zip(deq.iter()).map(|(a,b)| (*a as f64)*(*b as f64)).sum();
        let cos = dot / dn;
        sum += cos; cnt += 1; if cos < worst { worst = cos; }
    }
    println!("   dim={} n={}  mean cos {:.6}  worst {:.6}   (1.0 = lossless)", dim, cnt, sum/cnt as f64, worst);

    println!("D. TURBOVEC ENCODE THROUGHPUT");
    let v: Vec<f32> = (0..dim).map(|_| (lcg(&mut s) as f64 / u32::MAX as f64) as f32).collect();
    let iters = 200_000u32;
    let t0 = Instant::now();
    let mut bytes = 0usize;
    for _ in 0..iters { let (p, _sc) = TV::quantize_158(&v); bytes += p.len(); }
    let dt = t0.elapsed();
    println!("   {:?} for {} encodes -> {:.0} encodes/sec | {:.1} MB/s packed",
             dt, iters, iters as f64/dt.as_secs_f64(),
             (bytes/iters as usize) as f64 * iters as f64 / dt.as_secs_f64() / 1e6);

    println!("E. TURBOVEC 16x COMPRESSION CHECK");
    let packed_bytes = bytes / iters as usize;
    let fp32_bytes = dim * 4;
    println!("   fp32 {} B -> packed {} B  =  {:.1}x", fp32_bytes, packed_bytes,
             fp32_bytes as f64 / packed_bytes as f64);
}
