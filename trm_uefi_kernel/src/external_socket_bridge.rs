//! TRM Swarm OS - external_socket_bridge (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/external_socket_bridge.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const DEFAULT_SOCKET_PORT: u64 = 8090;
pub const DEFAULT_HOST: &'static str = "0.0.0.0";

//  get_external_socket_server() 
static SINGLETON_EXTERNAL_SOCKET_SERVER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_external_socket_server() -> u64 {
    // Singleton handle (lazy-init pattern from external_socket_bridge.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_EXTERNAL_SOCKET_SERVER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_EXTERNAL_SOCKET_SERVER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
