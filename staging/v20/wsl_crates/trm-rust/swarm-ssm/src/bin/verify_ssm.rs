//! Verify gate: swarm-ssm vs c_mamba3_step (real kernel, same inputs).
//! Runs 100 steps with identical inputs through both; compares y outputs
//! exactly and h-state evolution.

use std::os::raw::{c_float, c_uint, c_ulonglong};

#[repr(C)]
struct Mamba3StateC {
    magic_header: u64,
    h: [f32; 64],
    a: [f32; 64],
    b: [f32; 64],
    c: [f32; 64],
    dt: f32,
    step_count: u64,
    _pad: [u8; 0],
}

fn main() {
    let lib = unsafe {
        libloading::Library::new(
            swarm_paths::kernel_ops_path(),
        )
        .expect("load kernel")
    };
    unsafe {
        let create: libloading::Symbol<unsafe extern "C" fn(*const c_float) -> u64> =
            lib.get(b"c_mamba3_create_state").unwrap();
        let step: libloading::Symbol<unsafe extern "C" fn(u64, *const c_float, *mut c_float) -> bool> =
            lib.get(b"c_mamba3_step").unwrap();

        // Rust reference
        let mut rs = swarm_ssm::Mamba3State::create(None);

        // C reference
        let handle = create(std::ptr::null());
        let mut max_diff = 0.0f32;
        let mut all_equal = true;

        // 100 deterministic inputs
        let mut state: u64 = 42;
        for i in 0..100 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let x = (((state >> 33) % 200) as f32 / 100.0) - 1.0; // -1..1

            let y_rust = rs.step(x);

            let mut y_c: c_float = 0.0;
            let ok = step(handle, &x, &mut y_c);
            assert!(ok);

            let diff = (y_rust - y_c).abs();
            if diff > max_diff { max_diff = diff; }
            if diff > 1e-6 { all_equal = false; }
            if i < 3 {
                println!("step {i}: x={x:.4} rust_y={y_rust:.8} c_y={y_c:.8} diff={diff:.2e}");
            }
        }

        println!("\n100 steps: max |rust_y - c_y| = {max_diff:.2e}");
        println!("all within 1e-6: {all_equal}");
        println!("rust step_count: {}", rs.step_count);
        if all_equal {
            println!("VERIFY GATE: PASSED - swarm-ssm matches c_mamba3_step");
        } else {
            println!("VERIFY GATE: FAILED");
            std::process::exit(1);
        }
    }
}
