//! Verify gate: swarm-council vs the real C++ kernel (byte-identical).
//!
//! Feeds the ACTUAL 14-agent x 2-motion council vote vectors through both
//! the Rust collapse (swarm-k3) and c_kleene_quantize_avx2 in
//! libswarm_kernel_ops.so (via FFI), and asserts byte-identical trits and
//! counts. Also runs a 1M-value random sweep, per the house verify pattern.

use std::os::raw::{c_float, c_int};

use swarm_council::{
    council_verdict, overlap, vote_vector, AgentVote, AGENT_MODELS, EPS, MOTION_A, MOTION_E,
};

const KERNEL_SO: &str = "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2/core/libswarm_kernel_ops.so";

type QuantizeFn =
    unsafe extern "C" fn(*const c_float, usize, c_float, *mut i8, *mut c_int) -> c_int;

fn main() {
    let mut failed = false;

    let (lib_path, ops_note) = swarm_paths::kernel_ops_avx2()
        .expect("no loadable kernel ops on this host (set TRM_KERNEL_OPS)");
    println!("kernel ops: {} -- {ops_note}", lib_path.display());
    let lib = unsafe { libloading::Library::new(&lib_path) }.expect("load kernel ops");
    let quantize: libloading::Symbol<QuantizeFn> =
        unsafe { lib.get(b"c_kleene_quantize_avx2") }.expect("bind c_kleene_quantize_avx2");

    println!("=== VERIFY GATE: swarm-council vs libswarm_kernel_ops.so ===");
    println!("recorded gate target: {KERNEL_SO}");
    println!(
        "eps: {EPS} | agents: {} | motions: A({}) E({})",
        AGENT_MODELS.len(),
        MOTION_A.len(),
        MOTION_E.len()
    );
    println!();

    // ── Gate 1: per-agent vote vectors, Rust collapse vs C kernel ──
    println!("[1/3] Per-agent vote vectors (14 agents x 2 motions):");
    println!(
        "  {:12} {:5} {:5}  vec                rust_trits  c_trits   identical",
        "agent", "ovA", "ovE"
    );
    let mut vecs_checked = 0usize;
    let mut vecs_mismatch = 0usize;
    for a in AGENT_MODELS.iter() {
        let ov_a = overlap(a, &MOTION_A);
        let ov_e = overlap(a, &MOTION_E);
        let mut row = format!("  {:12} {:5} {:5}", a.name, ov_a, ov_e);
        for motion in [&MOTION_A[..], &MOTION_E[..]] {
            if let Some(v) = vote_vector(a, motion) {
                let rust_trits = swarm_k3::collapse(&v, EPS);
                let mut c_trits = vec![0i8; v.len()];
                let mut counts = [0i32; 3];
                let rc = unsafe {
                    quantize(
                        v.as_ptr(),
                        v.len(),
                        EPS,
                        c_trits.as_mut_ptr(),
                        counts.as_mut_ptr(),
                    )
                };
                vecs_checked += 1;
                let identical = rc == 0
                    && rust_trits == c_trits
                    && counts[0] as usize + counts[1] as usize + counts[2] as usize == v.len();
                if !identical {
                    vecs_mismatch += 1;
                    failed = true;
                    row.push_str(&format!(
                        "  [{:.1},{:.3},{:.3}]  {:?}  {:?}  MISMATCH(rc={},counts={:?})",
                        v[0], v[1], v[2], rust_trits, c_trits, rc, counts
                    ));
                } else {
                    row.push_str(&format!(
                        "  [{:.1},{:.3},{:.3}]  {:?}  {:?}  identical",
                        v[0], v[1], v[2], rust_trits, c_trits
                    ));
                }
            }
        }
        println!("{row}");
    }
    println!("  -> {vecs_checked} vote vectors compared, {vecs_mismatch} mismatched");
    println!();

    // ── Gate 2: council verdicts (Rust tally) vs C-kernel tally ──
    println!("[2/3] Council verdicts, Rust tally vs C-kernel tally:");
    for (motion, label) in [(&MOTION_A[..], "A"), (&MOTION_E[..], "E")] {
        let (rust_verdict, votes) = council_verdict(&AGENT_MODELS, motion);
        // recompute tally from C kernel trits
        let mut c_sum: i64 = 0;
        for (_, v) in votes.iter() {
            if let AgentVote::Voted { vote_vector, .. } = v {
                let mut c_trits = vec![0i8; vote_vector.len()];
                let mut counts = [0i32; 3];
                unsafe {
                    quantize(
                        vote_vector.as_ptr(),
                        vote_vector.len(),
                        EPS,
                        c_trits.as_mut_ptr(),
                        counts.as_mut_ptr(),
                    )
                };
                c_sum += c_trits.iter().map(|&t| t as i64).sum::<i64>();
            }
        }
        let c_verdict = if c_sum > 0 {
            "PROCEED"
        } else if c_sum < 0 {
            "BLOCK"
        } else {
            "ANALYZE"
        };
        let agree = rust_verdict.label() == c_verdict;
        if !agree {
            failed = true;
        }
        println!(
            "  Motion {label}: rust={} c_kernel={} -> {}",
            rust_verdict.label(),
            c_verdict,
            if agree { "AGREE" } else { "DISAGREE" }
        );
    }
    println!();

    // ── Gate 3: 1M-value random sweep (house verify pattern, LCG seed 42) ──
    println!("[3/3] 1M-value LCG sweep (seed 42):");
    let mut values: Vec<f32> = Vec::with_capacity(1_000_000);
    let mut seed: u64 = 42;
    for _ in 0..1_000_000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let v = ((seed >> 33) as f64 / u32::MAX as f64) * 2.0 - 1.0;
        values.push(v as f32);
    }
    let rust_trits = swarm_k3::collapse(&values, EPS);
    let mut c_trits = vec![0i8; values.len()];
    let mut counts = [0i32; 3];
    unsafe {
        quantize(
            values.as_ptr(),
            values.len(),
            EPS,
            c_trits.as_mut_ptr(),
            counts.as_mut_ptr(),
        );
    }
    let sweep_identical = rust_trits == c_trits;
    if !sweep_identical {
        failed = true;
    }
    println!("  values=1,000,000 identical={sweep_identical}");
    println!(
        "  C counts: pass={} analyze={} block={}",
        counts[0], counts[1], counts[2]
    );
    println!("  Rust sum: {}", rust_trits.iter().map(|&t| t as i64).sum::<i64>());
    if !sweep_identical {
        for (i, (r, c)) in rust_trits.iter().zip(c_trits.iter()).enumerate() {
            if r != c {
                println!("  first mismatch at {i}: rust={r} c={c}");
                break;
            }
        }
    }
    println!();

    if !failed {
        println!("VERIFY GATE: PASSED - byte-identical to libswarm_kernel_ops.so on all council vote vectors and the 1M sweep");
        std::process::exit(0);
    } else {
        println!("VERIFY GATE: FAILED");
        std::process::exit(1);
    }
}
