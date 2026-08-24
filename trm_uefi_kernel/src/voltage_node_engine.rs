//! TRM Swarm OS - voltage_node_engine (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/voltage_node_engine.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const RUST_DLL_PATH: &'static str = r#"F:\Development sites\TRM-Swarm-OS-v2\swarm_v2_rust_kernel\target\release\swarm_rust_kernel.dll"#;

//  get_voltage_engine() 
static SINGLETON_VOLTAGE_ENGINE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_voltage_engine() -> u64 {
    // Singleton handle (lazy-init pattern from voltage_node_engine.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_VOLTAGE_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_VOLTAGE_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
