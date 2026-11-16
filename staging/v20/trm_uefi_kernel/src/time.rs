//! no_std time source: TSC (rdtsc) calibrated against EFI Stall.
//! Replaces std::time::Instant in ported intelligence modules.

use core::sync::atomic::{AtomicU64, Ordering};

/// TSC ticks per second, calibrated once at boot.
static TSC_HZ: AtomicU64 = AtomicU64::new(0);

#[inline]
fn rdtsc() -> u64 {
    // SAFETY: rdtsc is safe on x86-64; no memory side effects.
    unsafe {
        let lo: u32;
        let hi: u32;
        core::arch::asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nomem, preserves_flags)
        );
        ((hi as u64) << 32) | (lo as u64)
    }
}

/// Calibrate TSC against EFI Stall (10ms busy-wait) — call once at boot.
pub fn calibrate_tsc(stall_us: impl Fn(u64) -> u64) {
    let start = rdtsc();
    let _ = stall_us(10_000); // 10ms
    let end = rdtsc();
    let ticks = end.saturating_sub(start);
    // ticks per second = ticks / 0.01s (fallback to 3.6 GHz if measurement invalid)
    let hz = if ticks > 10_000 { ticks * 100 } else { 3_600_000_000 };
    TSC_HZ.store(hz, Ordering::Relaxed);
}

/// Monotonic time in nanoseconds since boot (TSC-derived, calibrated).
pub fn now_ns() -> u64 {
    let hz = TSC_HZ.load(Ordering::Relaxed);
    if hz == 0 {
        // Uncalibrated fallback: raw ticks (approximately ns on ~1GHz+ CPUs).
        rdtsc()
    } else {
        rdtsc().saturating_mul(1_000_000_000) / hz
    }
}

/// Calibrated TSC frequency in Hz.
pub fn tsc_hz() -> u64 {
    TSC_HZ.load(Ordering::Relaxed)
}

