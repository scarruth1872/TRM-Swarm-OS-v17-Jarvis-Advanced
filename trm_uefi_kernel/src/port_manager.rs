//! TRM Swarm OS - port_manager (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/port_manager.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const DEFAULT_PORT_RANGE: (i64, i64) = (9000, 9100);
pub const PORT_STATE_FILE: u64 = 0;

//  get_port_manager() 
static SINGLETON_PORT_MANAGER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_port_manager() -> u64 {
    // Singleton handle (lazy-init pattern from port_manager.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_PORT_MANAGER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_PORT_MANAGER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
