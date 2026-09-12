//! Host-side 1:1 verification for K3 logic + Instella router + TurboVec.
mod k3_logic;
mod instella_router;
mod turbovec;
mod trm7m;

/// Minimal time shim (std-based).
mod time {
    use std::time::Instant;
    static T0: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    pub fn now_ns() -> u64 {
        let t0 = *T0.get_or_init(Instant::now);
        t0.elapsed().as_nanos() as u64
    }
}

fn main() {
    use k3_logic::*;

    // K3 truth tables
    let mut and_ok = true;
    for x in [TRUE, UNKNOWN, FALSE] {
        for y in [TRUE, UNKNOWN, FALSE] {
            if KleeneTernaryEngine::kleene_and(x, y) != x.min(y) { and_ok = false; }
        }
    }
    println!("K3 AND==min: {}", if and_ok { "OK" } else { "FAIL" });

    let mut engine = KleeneTernaryEngine::new();
    let test_vec = [0.5f32, -0.5, 0.1, -0.8, 0.9, -0.2, 0.3, -0.6];
    let psi = engine.quick_reduction(&test_vec);
    println!("K3 PSI: {:?} (py: [1,-1,0,-1,1,0,0,-1])", psi);
    let s = engine.bypass_decision(&psi);
    println!(
        "K3 DECISION: trit={} conf={:.3} amb={:.3} (py: trit=-1 conf=0.375 amb=0.375)",
        s.trit, s.confidence, s.ambiguity_ratio
    );
    let p = KleeneTernaryEngine::pack_5_trits([TRUE, FALSE, UNKNOWN, TRUE, FALSE]);
    let u = KleeneTernaryEngine::unpack_5_trits(p);
    println!("K3 PACK: {} -> {:?} ok={} (py: 65 -> [1,-1,0,1,-1])", p, u, u == [TRUE, FALSE, UNKNOWN, TRUE, FALSE]);
    let off = engine.evaluate_offload(&[TRUE, UNKNOWN, TRUE, FALSE]);
    println!("K3 OFFLOAD mixed: {} (py: INSTELLA_MOE)", off);

    // sha256 sanity: hash of "abc" should be known vector
    let h = k3_logic::sha256(b"abc");
    let expect = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let got: String = h.iter().map(|b| format!("{:02x}", b)).collect();
    println!("SHA256(abc): {} {}", &got[..16], if got == expect { "OK" } else { "FAIL" });

    // Instella router
    let router = instella_router::InstellaSingleExpertRouter::new();
    let r1 = router.route("verify security audit");
    println!("MOE security: team={} lead={} conf={:.2} (py: Security/Shield)", r1.team, r1.lead_agent, r1.confidence);
    let r2 = router.route("arxiv paper search");
    println!("MOE research: team={} lead={} conf={:.2} (py: Research/Seeker)", r2.team, r2.lead_agent, r2.confidence);

    // TurboVec
    let mut tv = [0.0f32; 128];
    let mut s7: u64 = 7;
    for j in 0..128 {
        s7 = s7.wrapping_mul(6364136223846793005).wrapping_add(1);
        tv[j] = ((s7 >> 33) as f32 / (1u64 << 31) as f32) - 1.0;
    }
    let (codes, scale, vmin) = turbovec::TurboVec16xQuantizer::quantize_16x(&tv);
    let rec = turbovec::TurboVec16xQuantizer::dequantize_16x(&codes, scale, vmin);
    let mut sim = 0.0f32;
    for (a, b) in tv.iter().zip(rec.iter()) {
        sim += a * b;
    }
    println!("TURBOVEC 2-BIT: 128-dim -> {}B, recon sim={:.3} (16x compression)", codes.len() / 4, sim);

    // TurboVec 1.58-bit Vector Quantization Check
    let (tv158_codes, tv158_scale) = turbovec::TurboVec158Quantizer::quantize_158(&tv);
    let tv158_rec = turbovec::TurboVec158Quantizer::dequantize_158(&tv158_codes, 128, tv158_scale);
    let mut sim158 = 0.0f32;
    for (a, b) in tv.iter().zip(tv158_rec.iter()) {
        sim158 += a * b;
    }
    println!("TURBOVEC 1.58-BIT: 128-dim -> {}B trits, scale={:.4}, recon sim={:.3} (16x compression)", tv158_codes.len(), tv158_scale, sim158);
    
    // TRM 7M Empirical Benchmark: FP32 AVX2 vs TurboVec 1.58-bit AVX2
    println!("Initializing TRM 7M model (Mamba-3 SSM FP32 AVX2)...");
    let mut model_fp32 = trm7m::Trm7m::new();
    let n = 1000;
    let (p50_f, p95_f, avg_f, tps_f, _) = model_fp32.benchmark(n);
    println!("TRM 7M FP32 BENCH     n={} p50={}ns p95={}ns avg={}ns tps={} (weight={:.2}MB)", 
             n, p50_f, p95_f, avg_f, tps_f, trm7m::Trm7m::weight_bytes() as f64 / 1024.0 / 1024.0);

    println!("Initializing TRM 7M TurboVec 1.58-bit model (Mamba-3 SSM 1.58b Ternary)...");
    let mut model_tv158 = trm7m::Trm7mTurboVec158::new();
    let (p50_tv, p95_tv, avg_tv, tps_tv, _) = model_tv158.benchmark(n);
    let speedup = avg_f as f64 / avg_tv.max(1) as f64;
    let compression = trm7m::Trm7m::weight_bytes() as f64 / trm7m::Trm7mTurboVec158::weight_bytes() as f64;
    println!("TRM 7M TURBOVEC 1.58b n={} p50={}ns p95={}ns avg={}ns tps={} (weight={:.2}MB)", 
             n, p50_tv, p95_tv, avg_tv, tps_tv, trm7m::Trm7mTurboVec158::weight_bytes() as f64 / 1024.0 / 1024.0);
    println!("TURBOVEC SPEEDUP: {:.2}x faster | COMPRESSION: {:.2}x smaller ({:.1}% reduction)", 
             speedup, compression, (1.0 - 1.0 / compression) * 100.0);

    // Parity Verification: 10,000 random forward passes comparing decisions
    println!("\n========================================================");
    println!("RUNNING 10,000-SAMPLE EMPIRICAL PARITY VERIFICATION");
    println!("========================================================");
    let mut agreements = 0;
    let mut max_conf_diff = 0.0f32;
    let mut sum_conf_diff = 0.0f64;
    let mut sum_sq_conf_diff = 0.0f64;
    let mut trit_counts_fp32 = [0usize; 3]; // [PASS (+1), AMB (0), FAULT (-1)]
    let mut trit_counts_tv = [0usize; 3];
    let mut parity_model_fp32 = trm7m::Trm7m::new();
    let mut parity_model_tv = trm7m::Trm7mTurboVec158::new();
    let parity_n = 10000;
    let t_parity_start = std::time::Instant::now();

    for i in 0..parity_n {
        let mut x = [0.0f32; 128];
        let mut s: u64 = (i as u64 + 42).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        for j in 0..128 {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            x[j] = ((s >> 33) as f32 / (1u64 << 31) as f32) - 1.0;
        }
        parity_model_fp32.hidden.fill(0.0);
        parity_model_fp32.conv_buffers.fill(0.0);
        parity_model_tv.hidden.fill(0.0);
        parity_model_tv.conv_buffers.fill(0.0);

        let (trit_f, conf_f, _) = parity_model_fp32.forward(&x);
        let (trit_tv, conf_tv, _) = parity_model_tv.forward(&x);

        match trit_f {
            1 => trit_counts_fp32[0] += 1,
            0 => trit_counts_fp32[1] += 1,
            -1 => trit_counts_fp32[2] += 1,
            _ => {}
        }
        match trit_tv {
            1 => trit_counts_tv[0] += 1,
            0 => trit_counts_tv[1] += 1,
            -1 => trit_counts_tv[2] += 1,
            _ => {}
        }

        if trit_f == trit_tv {
            agreements += 1;
        }
        let diff = (conf_f - conf_tv).abs();
        sum_conf_diff += diff as f64;
        sum_sq_conf_diff += (diff as f64) * (diff as f64);
        if diff > max_conf_diff {
            max_conf_diff = diff;
        }

        if (i + 1) % 2500 == 0 {
            println!("  [Sample {:5}/{}] Current agreement: {}/{} ({:.2}%) | max delta: {:.6}", 
                     i + 1, parity_n, agreements, i + 1, (agreements as f64 / (i + 1) as f64) * 100.0, max_conf_diff);
        }
    }
    let parity_elapsed = t_parity_start.elapsed();
    let mean_conf_diff = sum_conf_diff / parity_n as f64;
    let rmse_conf_diff = (sum_sq_conf_diff / parity_n as f64).sqrt();

    println!("--------------------------------------------------------");
    println!("10,000-SAMPLE DECISION PARITY RESULTS:");
    println!("  Total Samples Evaluated: {}", parity_n);
    println!("  Exact Decision Match:    {}/{} ({:.4}%)", agreements, parity_n, (agreements as f64 / parity_n as f64) * 100.0);
    println!("  Max Confidence Delta:    {:.6}", max_conf_diff);
    println!("  Mean Confidence Delta:   {:.6}", mean_conf_diff);
    println!("  RMSE Confidence Delta:   {:.6}", rmse_conf_diff);
    println!("  Trit Distribution (FP32): PASS(+1)={}, AMB(0)={}, FAULT(-1)={}", 
             trit_counts_fp32[0], trit_counts_fp32[1], trit_counts_fp32[2]);
    println!("  Trit Distribution (TV158): PASS(+1)={}, AMB(0)={}, FAULT(-1)={}", 
             trit_counts_tv[0], trit_counts_tv[1], trit_counts_tv[2]);
    println!("  Total Evaluation Time:   {:.2?} ({:.1} µs/sample combined)", 
             parity_elapsed, parity_elapsed.as_micros() as f64 / parity_n as f64);
    println!("========================================================\n");
}
