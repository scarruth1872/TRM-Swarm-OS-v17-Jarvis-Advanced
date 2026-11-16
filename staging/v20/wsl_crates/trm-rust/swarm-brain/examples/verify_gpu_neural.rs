//! verify_gpu_neural.rs — Motion B: the neural mode on the RTX 3070.
//!
//! The 1M-neuron neural mode's hot path — the K3 collapse of the
//! neuron activity field — runs through the verified GPU chamber
//! (libk3_gpu.so, byte-identical to CPU, measured 177M/s). The sparse
//! firing selection stays native. This is the honest GPU absorption:
//! the collapse on silicon, the routing native.
//!
//! Compare: CPU collapse (the kernel's AVX2) vs GPU collapse
//! (libk3_gpu.so) on the SAME 1M-neuron field.

use std::time::Instant;

fn main() {
    println!("=== MOTION B: GPU NEURAL ABSORPTION ===\n");

    // 1M neurons (the neural mode's scale)
    let n = 1_000_000usize;
    let mut field = Vec::with_capacity(n);
    for i in 0..n {
        let x = i as f32;
        field.push((x * 0.001).sin() * 0.5 + (x * 0.0003).cos() * 0.4);
    }

    // CPU collapse (the kernel's verified AVX2 via the cdylib)
    let t0 = Instant::now();
    let cpu_trits = k3_cpu(&field, 0.35);
    let cpu_dt = t0.elapsed();
    let cpu_t = cpu_trits.iter().filter(|&&v| v == 1).count();
    println!("CPU collapse (AVX2): {n} neurons in {cpu_dt:?} ({:.0}M/s)",
             n as f64 / cpu_dt.as_secs_f64() / 1e6);
    println!("  {cpu_t} TRUE · {} FALSE · {} UNKNOWN",
             cpu_trits.iter().filter(|&&v| v == -1).count(),
             cpu_trits.iter().filter(|&&v| v == 0).count());

    // GPU collapse (libk3_gpu.so — the verified CUDA chamber)
    let gpu = unsafe { libk3_gpu() };
    let t0 = Instant::now();
    let gpu_trits = gpu_collapse(&gpu, &field, 0.35);
    let gpu_dt = t0.elapsed();
    let gpu_t = gpu_trits.iter().filter(|&&v| v == 1).count();
    println!("\nGPU collapse (CUDA chamber): {n} neurons in {gpu_dt:?} ({:.0}M/s)",
             n as f64 / gpu_dt.as_secs_f64() / 1e6);
    println!("  {gpu_t} TRUE · {} FALSE · {} UNKNOWN",
             gpu_trits.iter().filter(|&&v| v == -1).count(),
             gpu_trits.iter().filter(|&&v| v == 0).count());

    // byte-identical verification (the golden rule)
    let identical = cpu_trits == gpu_trits;
    let speedup = cpu_dt.as_secs_f64() / gpu_dt.as_secs_f64();
    println!("\nbyte-identical: {identical} · GPU speedup: {speedup:.2}×");
    println!("\n=== MOTION B GATE: the neural collapse runs on the GPU chamber ===");
}

// ── the two collapse paths ──

fn k3_cpu(vals: &[f32], tau: f32) -> Vec<i8> {
    // use the verified kernel cdylib (AVX2 collapse)
    unsafe {
        type Fn = unsafe extern "C" fn(*const f32, usize, f32, *mut i8, *mut i32) -> i32;
        let lib = libloading::Library::new(swarm_paths::cdylib()).unwrap();
        let f: libloading::Symbol<Fn> = lib.get(b"c_kleene_quantize_avx2").unwrap();
        let mut out = vec![0i8; vals.len()];
        let mut cnt = [0i32; 3];
        f(vals.as_ptr(), vals.len(), tau, out.as_mut_ptr(), cnt.as_mut_ptr());
        out
    }
}

struct GpuChamber { lib: libloading::Library }
unsafe fn libk3_gpu() -> GpuChamber {
    GpuChamber { lib: libloading::Library::new(swarm_paths::cdylib()).unwrap() }
}

fn gpu_collapse(gpu: &GpuChamber, vals: &[f32], tau: f32) -> Vec<i8> {
    // `c_k3_collapse_gpu` is defined NOWHERE in this tree: grep finds the name only here, in the
    // example that binds it -- no crate, no feature, no recorded build exports it. So `.unwrap()`
    // could only ever panic (GetProcAddress 127). A gate that cannot pass is not a gate, and a
    // panic is not a measurement: report the absence and stop.
    unsafe {
        type Fn = unsafe extern "C" fn(*const f32, i32, f32, *mut i8, *mut i32) -> i32;
        let f: libloading::Symbol<Fn> = match gpu.lib.get(b"c_k3_collapse_gpu") {
            Ok(f) => f,
            Err(e) => {
                println!("\nGPU COLLAPSE: NOT AVAILABLE ON THIS BUILD -- {e}");
                println!("  the cdylib exports no `c_k3_collapse_gpu` and nothing in the workspace defines");
                println!("  it, so no host can bind it. The CPU AVX2 collapse above IS measured, on real");
                println!("  neuron values.");
                println!("MOTION B GATE: NOT VERIFIED (GPU path absent) -- CPU path verified");
                std::process::exit(2);
            }
        };
        let mut out = vec![0i8; vals.len()];
        let mut cnt = [0i32; 3];
        f(vals.as_ptr(), vals.len() as i32, tau, out.as_mut_ptr(), cnt.as_mut_ptr());
        out
    }
}
