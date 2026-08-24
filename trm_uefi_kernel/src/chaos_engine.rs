//! TRM Swarm OS — chaos_engine (no_std port, REAL-verified).
//! 1:1 deterministic state machine of swarm_v2/core/chaos_engine.py:
//!   - enable/disable, set_latency, set_dropout_rate (clamped [0,1])
//!   - should_drop_connection: enabled && rate>0 && uniform_sample < rate
//! The python async sleep (`apply_network_delay`) is I/O-bound — skeleton.

extern crate alloc;

/// Modular chaos engineering state (Phase 18): injects network delays and
/// socket dropouts to verify self-healing edge federation stability.
#[derive(Debug, Clone)]
pub struct ChaosEngine {
    pub network_delay_sec: f64,
    pub dropout_rate: f64, // 0.0 ..= 1.0 probability of drop
    pub enabled: bool,
}

impl ChaosEngine {
    pub fn new() -> Self {
        ChaosEngine { network_delay_sec: 0.0, dropout_rate: 0.0, enabled: false }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// disable() resets latency and dropout rate to 0.0 (1:1 python).
    pub fn disable(&mut self) {
        self.enabled = false;
        self.network_delay_sec = 0.0;
        self.dropout_rate = 0.0;
    }

    pub fn set_latency(&mut self, seconds: f64) {
        self.network_delay_sec = seconds;
    }

    /// python: dropout_rate = max(0.0, min(1.0, rate)).
    pub fn set_dropout_rate(&mut self, rate: f64) {
        self.dropout_rate = if rate < 0.0 { 0.0 } else if rate > 1.0 { 1.0 } else { rate };
    }

    /// Deterministic core of should_drop_connection: python samples
    /// random.random() and drops iff sample < dropout_rate. `sample` is the
    /// injected uniform [0,1) draw, making the decision reproducible.
    pub fn should_drop_connection_with(&self, sample: f64) -> bool {
        if self.enabled && self.dropout_rate > 0.0 {
            return sample < self.dropout_rate;
        }
        false
    }

    /// python get_profile(): (enabled, latency_seconds, dropout_rate).
    pub fn get_profile(&self) -> (bool, f64, f64) {
        (self.enabled, self.network_delay_sec, self.dropout_rate)
    }
}

/// Singleton accessor — python `get_chaos_engine()` returns the module-level
/// `_chaos_engine` instance. no_std handle convention: 1 once initialized.
static SINGLETON_CHAOS_ENGINE: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
pub fn get_chaos_engine() -> u64 {
    let v = SINGLETON_CHAOS_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_CHAOS_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// MANUAL: async sleep I/O — not portable deterministic logic.
pub fn apply_network_delay() -> u64 {
    0
}
