//! Verify gate: swarm-k3 vs the real C++ kernel (byte-identical).
//! Runs 1M random floats through both the Rust collapse and the
//! C libswarm_kernel_ops.so collapse; asserts identical trits.

use std::os::raw::{c_float, c_int};

#[repr(C)]
struct KernelCalls {
    handle: *mut std::ffi::c_void,
    quantize: unsafe extern "C" fn(*const c_float, usize, c_float, *mut i8, *mut c_int) -> c_int,
}

fn main() {
    // Load the real kernel
    unsafe {
        let lib = libloading::Library::new(
            swarm_paths::kernel_ops_path_avx2(),
        )
        .expect("load kernel");
        let quantize: libloading::Symbol<
            unsafe extern "C" fn(*const c_float, usize, c_float, *mut i8, *mut c_int) -> c_int,
        > = lib.get(b"c_kleene_quantize_avx2").expect("symbol");

        // 1M deterministic values
        let mut values: Vec<f32> = Vec::with_capacity(1_000_000);
        let mut seed: u64 = 42;
        for _ in 0..1_000_000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let v = ((seed >> 33) as f64 / u32::MAX as f64) * 2.0 - 1.0;
            values.push(v as f32);
        }

        // Rust collapse
        let rust_trits = swarm_k3::collapse(&values, 0.35);

        // C kernel collapse
        let mut c_trits = vec![0i8; 1_000_000];
        let mut counts = [0i32; 3];
        quantize(values.as_ptr(), 1_000_000, 0.35, c_trits.as_mut_ptr(), counts.as_mut_ptr());

        // Compare
        let identical = rust_trits == c_trits;
        let pass = counts[0];
        let analyze = counts[1];
        let block = counts[2];
        println!("Rust vs C kernel over 1,000,000 values:");
        println!("  identical: {identical}");
        println!("  C counts:  pass={pass} analyze={analyze} block={block}");
        println!("  Rust sum:  {}", rust_trits.iter().map(|&t| t as i64).sum::<i64>());
        if identical {
            println!("VERIFY GATE: PASSED — byte-identical to libswarm_kernel_ops.so");
            std::process::exit(0);
        } else {
            // find first mismatch
            for (i, (r, c)) in rust_trits.iter().zip(c_trits.iter()).enumerate() {
                if r != c {
                    println!("first mismatch at {i}: rust={r} c={c}");
                    break;
                }
            }
            println!("VERIFY GATE: FAILED");
            std::process::exit(1);
        }
    }
}
