//! UEFI-backed `SystemProbe` — the live caller for `crate::telemetry`.
//!
//! `telemetry.rs` is a port of `swarm_v2/core/telemetry.py`, whose metrics come
//! from `psutil`. Firmware has no `psutil`, so this module supplies the probe
//! from the sources UEFI genuinely offers:
//!
//!  * **memory** — the firmware memory map, via `GetMemoryMap`. A real
//!    measurement: addressable RAM by type, and how much of it the firmware and
//!    this loader currently occupy.
//!  * **wall clock** — the UEFI Runtime Service `GetTime` (the motherboard RTC),
//!    falling back to TSC uptime when the firmware refuses.
//!  * **cpu** — a DOCUMENTED ZERO, not a measurement. UEFI Boot Services expose
//!    no scheduler-load figure. Returning an invented percentage would make
//!    `check_stability` assert a stability verdict the firmware never produced,
//!    which is exactly the fabrication this port exists to avoid.
//!
//! Nothing here calls `telemetry` unless `boot_report` is invoked; the kernel
//! invokes it once at boot so LTO cannot strip the port as dead code.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::telemetry::{EmergenceReport, SwarmTelemetry, SystemProbe};
use r_efi::efi;

/// UEFI memory types that constitute addressable system RAM. ACPI reclaim/NVS,
/// memory-mapped I/O and reserved ranges are excluded — they are not
/// general-purpose memory the swarm can allocate from.
const RAM_TYPES: &[u32] = &[
    efi::LOADER_CODE,
    efi::LOADER_DATA,
    efi::BOOT_SERVICES_CODE,
    efi::BOOT_SERVICES_DATA,
    efi::RUNTIME_SERVICES_CODE,
    efi::RUNTIME_SERVICES_DATA,
    efi::CONVENTIONAL_MEMORY,
    efi::PERSISTENT_MEMORY,
];

/// Types counted as OCCUPIED: firmware code/data and this loader. These are
/// resident and would not be handed back by `FreePages`.
const OCCUPIED_TYPES: &[u32] = &[
    efi::LOADER_CODE,
    efi::LOADER_DATA,
    efi::BOOT_SERVICES_CODE,
    efi::BOOT_SERVICES_DATA,
    efi::RUNTIME_SERVICES_CODE,
    efi::RUNTIME_SERVICES_DATA,
];

/// Page totals from one memory-map walk. A UEFI page is 4 KiB.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MemoryReading {
    pub total_pages: u64,
    pub occupied_pages: u64,
    pub descriptors: usize,
}

impl MemoryReading {
    /// Bytes of addressable RAM.
    pub fn total_bytes(&self) -> u64 {
        self.total_pages * 4096
    }

    /// Bytes resident in firmware/loader ranges.
    pub fn occupied_bytes(&self) -> u64 {
        self.occupied_pages * 4096
    }

    /// Percentage of addressable RAM currently occupied. An empty or all-zero
    /// map yields 0.0 rather than dividing by zero — `check_stability` then
    /// reads a genuine 0%, not a fabricated figure.
    pub fn percent(&self) -> f64 {
        if self.total_pages == 0 {
            return 0.0;
        }
        (self.occupied_pages as f64 / self.total_pages as f64) * 100.0
    }
}

/// Reduce a descriptor array to page totals.
///
/// Pure pointer arithmetic with no UEFI calls, so the differential harness can
/// drive it against an oracle. `count` is the caller's `size / descriptor_size`;
/// the descriptor stride is taken from the firmware, not assumed.
///
/// # Safety
/// `ptr` must point to at least `count` packed descriptors as laid out by
/// `GetMemoryMap`.
pub unsafe fn tally(ptr: *const efi::MemoryDescriptor, count: usize, stride: usize) -> MemoryReading {
    let mut out = MemoryReading { total_pages: 0, occupied_pages: 0, descriptors: count };
    if ptr.is_null() || stride < core::mem::size_of::<efi::MemoryDescriptor>() {
        out.descriptors = 0;
        return out;
    }
    for i in 0..count {
        let d = &*((ptr as *const u8).add(i * stride) as *const efi::MemoryDescriptor);
        if RAM_TYPES.contains(&d.r#type) {
            out.total_pages = out.total_pages.saturating_add(d.number_of_pages);
        }
        if OCCUPIED_TYPES.contains(&d.r#type) {
            out.occupied_pages = out.occupied_pages.saturating_add(d.number_of_pages);
        }
    }
    out
}

/// Walk the firmware memory map. `None` when the firmware refuses twice —
/// the caller then reports zeros rather than guessing at RAM.
pub fn read_memory_map(bs: *mut efi::BootServices) -> Option<MemoryReading> {
    if bs.is_null() {
        return None;
    }
    let bs = unsafe { &*bs };

    let (mut size, mut key, mut desc_size, mut desc_ver) = (0usize, 0usize, 0usize, 0u32);

    // Sizing call: expected to fail with BUFFER_TOO_SMALL while reporting the
    // map size and the firmware's descriptor stride.
    let s = (bs.get_memory_map)(
        &mut size,
        core::ptr::null_mut(),
        &mut key,
        &mut desc_size,
        &mut desc_ver,
    );
    if s != efi::Status::BUFFER_TOO_SMALL || desc_size == 0 {
        return None;
    }

    // The map can grow between the two calls (a timer or a driver allocating),
    // so take 32 descriptors slack. Over-allocating costs a few KiB once at boot.
    size += desc_size * 32;

    let mut buf: Vec<u8> = alloc::vec![0u8; size];
    let s = (bs.get_memory_map)(
        &mut size,
        buf.as_mut_ptr() as *mut efi::MemoryDescriptor,
        &mut key,
        &mut desc_size,
        &mut desc_ver,
    );
    if s != efi::Status::SUCCESS {
        return None;
    }

    let count = size / desc_size;
    Some(unsafe { tally(buf.as_ptr() as *const efi::MemoryDescriptor, count, desc_size) })
}

/// The firmware's answer to `psutil` — memory map plus RTC.
pub struct UefiSystemProbe {
    pub memory: MemoryReading,
    pub rt: *mut efi::RuntimeServices,
}

impl SystemProbe for UefiSystemProbe {
    /// Always 0.0. See the module header: UEFI exposes no load metric, and a
    /// plausible-looking number here would be a fabrication.
    fn cpu_percent(&self) -> f64 {
        0.0
    }

    fn memory_percent(&self) -> f64 {
        self.memory.percent()
    }

    /// `GetTime` on the firmware RTC, matching python's `datetime.now()`.
    /// Falls back to TSC uptime when the call fails, so the string is never
    /// silently blank.
    fn timestamp_iso(&self) -> String {
        if !self.rt.is_null() {
            let rt = unsafe { &*self.rt };
            let mut t: efi::Time = unsafe { core::mem::zeroed() };
            let s = (rt.get_time)(&mut t, core::ptr::null_mut());
            if s == efi::Status::SUCCESS && t.year >= 2000 && t.year <= 2100 && t.month >= 1 && t.month <= 12 {
                return format!(
                    "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                    t.year, t.month, t.day, t.hour, t.minute, t.second
                );
            }
        }
        format!("uptime {:.6}s", crate::time::now_ns() as f64 / 1e9)
    }
}

/// Build the boot emergence report.
///
/// This is the call that keeps `telemetry` and `coherence_thresholds` resident:
/// without a real invocation LTO drops both and the ports are dead code that
/// merely happens to compile.
pub fn boot_report(st: &efi::SystemTable) -> EmergenceReport {
    let memory = read_memory_map(st.boot_services).unwrap_or_default();
    let probe = UefiSystemProbe { memory, rt: st.runtime_services };
    let tel = SwarmTelemetry::new();
    // No resource arbiter and no expert team exist in firmware; both are
    // reported as genuinely absent rather than filled with stand-ins.
    tel.get_emergence_report(&probe, "unbound (firmware)", &[])
}
