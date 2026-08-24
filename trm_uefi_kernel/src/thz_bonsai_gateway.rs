//! TRM Swarm OS — thz_bonsai_gateway (no_std port).
//! THz + Bonsai ternary gateway routing logic.

extern crate alloc;

static SINGLETON_THZ_BONSAI_GATEWAY: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_thz_bonsai_gateway() -> u64 {
    // Singleton handle (lazy-init pattern from thz_bonsai_gateway.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_THZ_BONSAI_GATEWAY.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_THZ_BONSAI_GATEWAY.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

pub fn route_gateway(e: u64, agent: u64) -> u64 {

        // THz (<60) fast path vs Bonsai deep synthesis.
        let _ = agent;
        if e < 60 { 1 } else { 2 }
}
