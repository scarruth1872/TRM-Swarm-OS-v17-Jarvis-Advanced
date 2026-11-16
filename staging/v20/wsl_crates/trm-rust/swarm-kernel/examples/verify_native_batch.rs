//! verify_native_batch.rs — the numpy-free batch (all abstract layers gone).
//!
//! The S2 slow path (99.9ms) was numpy allocation + ctypes marshalling
//! + subprocess spawn. This does the SAME 1M-value ensemble-governed
//! batch entirely in native Rust: field generated in-process, collapse
//! via the kernel's own exported symbol (its cdylib), verdicts read
//! natively. No Python, no numpy, no ctypes, no spawn.

use std::time::Instant;

type BatchFn =
    unsafe extern "C" fn(*const f32, usize, f32, f32, *mut i32, *mut i32, *mut i8) -> i32;

fn batch_fn() -> BatchFn {
    unsafe {
        // the Library MUST outlive the fn pointer (dropping it unloads
        // the .so → dangling pointer → segfault). Leak it deliberately.
        let lib: &'static libloading::Library = Box::leak(Box::new(
            libloading::Library::new(swarm_paths::cdylib())
                .unwrap()));
        let f: libloading::Symbol<BatchFn> = lib.get(b"c_batch_council_ensemble").unwrap();
        *f
    }
}

fn main() {
    let n = 1_000_000usize;
    println!("=== NATIVE BATCH — NO NUMPY, NO CTYPES, NO SPAWN ===");

    // 1. the field generated IN RUST (the numpy allocation is gone)
    let t0 = Instant::now();
    let field: Vec<f32> = (0..n)
        .map(|i| {
            let x = i as f32;
            (x * 0.001).sin() * 0.5 + (x * 0.0003).cos() * 0.4
        })
        .collect();
    let gen_dt = t0.elapsed();
    println!("field generation (native): {n} values in {gen_dt:?}");

    // 2. the ensemble-governed batch — the kernel's own symbol, in-process
    let batch = batch_fn();
    let mut per = [0i32; 30];
    let mut ver = [0i32; 10];
    let mut votes = [0i8; 30];
    let t0 = Instant::now();
    let rc = unsafe {
        batch(
            field.as_ptr(),
            n,
            0.35,
            0.9,
            per.as_mut_ptr(),
            ver.as_mut_ptr(),
            votes.as_mut_ptr(),
        )
    };
    let batch_dt = t0.elapsed();
    println!(
        "ensemble batch (native): rc={rc} in {batch_dt:?} ({:.0}M/s)",
        n as f64 / batch_dt.as_secs_f64() / 1e6
    );

    // 3. the three priors (the full gate semantics, all native)
    for prior in [0.9f32, -0.9, 0.0] {
        let t0 = Instant::now();
        unsafe {
            batch(
                field.as_ptr(),
                n,
                0.35,
                prior,
                per.as_mut_ptr(),
                ver.as_mut_ptr(),
                votes.as_mut_ptr(),
            )
        };
        let dt = t0.elapsed();
        println!("prior {prior:+}: {dt:?} · verdicts {:?}", &ver[..]);
    }

    // 4. the honest comparison vs the numpy path (measured 99.9ms)
    println!("\n=== THE LATENCY ELIMINATED ===");
    println!("numpy+ctypes path (S2): ~99.9 ms/task (measured)");
    println!(
        "native path: {:?} + {:?} = {:.2} ms total",
        gen_dt,
        batch_dt,
        (gen_dt + batch_dt).as_secs_f64() * 1000.0
    );
    let speedup = 0.0999 / (gen_dt + batch_dt).as_secs_f64();
    println!("speedup: {speedup:.0}× — the abstract layers are gone");
    println!("=== THE 1T IMPLICATION ===");
    println!(
        "1T batch native: {:.2}h (vs the marshalled path)",
        1e12 / (n as f64 / batch_dt.as_secs_f64()) / 3600.0
    );
}
