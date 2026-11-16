//! Timing shim for the kernel modules reused by the council server.
//!
//! The UEFI kernel supplies `crate::time::now_ns()` from firmware timers. Under
//! std there is no such module, so turbovec's femto_step (which reports its own
//! per-step latency in nanoseconds) fails to resolve. This backs it with the
//! monotonic clock so the identical code path measures real elapsed nanoseconds.

use std::sync::OnceLock;
use std::time::Instant;

static START: OnceLock<Instant> = OnceLock::new();

pub fn now_ns() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_nanos() as u64
}
