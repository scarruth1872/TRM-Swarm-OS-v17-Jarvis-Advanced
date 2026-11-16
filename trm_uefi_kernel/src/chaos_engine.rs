//! TRM Swarm OS — chaos_engine (no_std port, REAL-verified).
//! 1:1 deterministic state machine of swarm_v2/core/chaos_engine.py:
//!   - enable/disable, set_latency, set_dropout_rate (clamped [0,1])
//!   - should_drop_connection: enabled && rate>0 && uniform_sample < rate
//!   - get_profile: (enabled, latency_seconds, dropout_rate)
//! The python async sleep (`apply_network_delay`) is I/O-bound — MANUAL
//! skeleton. `should_drop_connection()` draws random.random() in python; the
//! deterministic core takes the uniform [0,1) draw as an injected `sample`.

extern crate alloc;

use core::cell::Cell;

/// Modular chaos engineering state (Phase 18): injects network delays and
/// socket dropouts to verify self-healing edge federation stability.
#[derive(Debug, Clone, PartialEq)]
pub struct ChaosEngine {
    pub network_delay_sec: f64,
    pub dropout_rate: f64, // 0.0 ..= 1.0 probability of drop
    pub enabled: bool,
    /// xorshift64 stream state for the dropout draw. `Cell` because
    /// `should_drop_connection` takes `&self` (Python's method is also
    /// non-mutating; the entropy lives in the module-level `random`).
    rng_state: Cell<u64>,
}

impl ChaosEngine {
    /// python `ChaosEngine()` defaults: latency 0.0, dropout 0.0, disabled.
    pub fn new() -> Self {
        ChaosEngine {
            network_delay_sec: 0.0,
            dropout_rate: 0.0,
            enabled: false,
            // Fixed seed (golden-ratio constant) keeps runs reproducible.
            rng_state: Cell::new(0x9E37_79B9_7F4A_7C15),
        }
    }

    /// python `enable()`: enabled = True (no state reset).
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// python `disable()`: enabled=False AND latency/dropout reset to 0.0.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.network_delay_sec = 0.0;
        self.dropout_rate = 0.0;
    }

    /// python `set_latency(seconds)`: network_delay_sec = seconds (no clamp).
    pub fn set_latency(&mut self, seconds: f64) {
        self.network_delay_sec = seconds;
    }

    /// python: dropout_rate = max(0.0, min(1.0, rate)) — clamped to [0,1].
    pub fn set_dropout_rate(&mut self, rate: f64) {
        self.dropout_rate = if rate < 0.0 { 0.0 } else if rate > 1.0 { 1.0 } else { rate };
    }

    /// Deterministic core of python `should_drop_connection()`: python
    /// samples `random.random()` and drops iff `sample < dropout_rate`, only
    /// while enabled and rate > 0. `sample` is the injected uniform [0,1)
    /// draw, making the decision reproducible in no_std.
    pub fn should_drop_connection_with(&self, sample: f64) -> bool {
        if self.enabled && self.dropout_rate > 0.0 {
            return sample < self.dropout_rate;
        }
        false
    }

    /// python `should_drop_connection()`: drops iff enabled && rate > 0 &&
    /// sample < rate. xorshift64 stands in for Python's Mersenne Twister -
    /// the sequence differs, the semantics are 1:1.
    pub fn should_drop_connection(&self) -> bool {
        if self.enabled && self.dropout_rate > 0.0 {
            return self.next_uniform() < self.dropout_rate;
        }
        false
    }

    /// xorshift64 -> uniform [0,1); 53-bit mantissa, matching Python float draws.
    fn next_uniform(&self) -> f64 {
        let mut x = self.rng_state.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state.set(x);
        ((x >> 11) as f64) / ((1u64 << 53) as f64)
    }

    /// python `get_profile()` -> dict {enabled, latency_seconds, dropout_rate}.
    pub fn get_profile(&self) -> (bool, f64, f64) {
        (self.enabled, self.network_delay_sec, self.dropout_rate)
    }
}

impl Default for ChaosEngine {
    fn default() -> Self {
        ChaosEngine::new()
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

/// MANUAL: python `apply_network_delay()` awaits asyncio.sleep(latency) —
/// I/O bound, not portable deterministic logic.
pub fn apply_network_delay() -> u64 {
    0
}
