//! TRM Swarm OS — cognitive_ingestion_engine (no_std port).
//! Constants/structs extracted from swarm_v2/core/cognitive_ingestion_engine.py.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

static SINGLETON_EVENT_LEDGER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_event_ledger() -> u64 {
    // Singleton handle (lazy-init pattern from cognitive_ingestion_engine.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_EVENT_LEDGER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_EVENT_LEDGER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

static SINGLETON_INGESTION_ENGINE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_ingestion_engine() -> u64 {
    // Singleton handle (lazy-init pattern from cognitive_ingestion_engine.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_INGESTION_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_INGESTION_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
