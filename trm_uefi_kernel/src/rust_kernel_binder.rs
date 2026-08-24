//! TRM Swarm OS - rust_kernel_binder (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/rust_kernel_binder.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const RUST_DLL_PATH: &'static str = r#"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2_rust_kernel\target\release\swarm_rust_kernel.dll"#;

//  get_rust_kernel() 
static SINGLETON_RUST_KERNEL: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_rust_kernel() -> u64 {
    // Singleton handle (lazy-init pattern from rust_kernel_binder.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_RUST_KERNEL.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_RUST_KERNEL.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
