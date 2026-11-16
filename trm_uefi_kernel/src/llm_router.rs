//! TRM Swarm OS - llm_router (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/llm_router.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const _HTTP_SESSION: u64 = 0;
pub const PROVIDER_MAP: u64 = 0;
pub const FALLBACK_CHAIN: &'static [&'static str] = &["instella", "local", "bonsai"];

//  get_session() 
static SINGLETON_SESSION: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_session() -> u64 {
    // Singleton handle (lazy-init pattern from llm_router.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_SESSION.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SESSION.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
