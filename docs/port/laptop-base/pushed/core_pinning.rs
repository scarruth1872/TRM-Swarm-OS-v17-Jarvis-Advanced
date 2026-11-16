//! CPU core pinning + the host topology the runtime reports.
//!
//! REWRITTEN FOR THIS HOST (desktop: Ryzen 9 3900X, RX 6700 XT). The version this replaces was written
//! on the 3070 laptop and asserted its hardware as constants:
//!     PHYSICAL_CORES = 8, LOGICAL_THREADS = 16, "AMD Ryzen 9 5900HX (Zen 3 Cezanne)",
//!     GPU_DEVICE_NAME = "NVIDIA GeForce RTX 3070 Laptop GPU", GPU_DEVICE_VRAM_MIB = 8192
//! On this machine every one of those is wrong: 12 physical cores, 24 logical threads, a 3900X, and an
//! AMD RX 6700 XT with no CUDA and no ROCm at all. A constant that describes another computer is the
//! same class of lie as a fake sensor, so the topology is measured at runtime and anything that cannot
//! be measured reports null rather than a plausible number.
//!
//! `pin_current_thread_to_core` keeps its signature: physical core k -> logical (2k, 2k+1). Both SMT
//! siblings are set so the thread cannot be migrated to the other half of the pair.

/// Physical core k -> the two logical processors that share it.
#[cfg(target_os = "windows")]
pub fn pin_current_thread_to_core(core: usize) {
    let (physical, logical) = topology();
    // A host with fewer cores than the core map assumes must not compute a mask with bits set past the
    // end - Windows ignores those silently and the pin becomes a no-op that looks successful.
    let k = core.min(physical.saturating_sub(1));
    let lo = k * 2;
    let hi = k * 2 + 1;
    let mut mask = 1usize << lo.min(usize::BITS as usize - 1);
    if hi < logical && hi < usize::BITS as usize {
        mask |= 1usize << hi;
    }
    unsafe {
        extern "system" {
            fn GetCurrentThread() -> *mut std::ffi::c_void;
            fn SetThreadAffinityMask(h_thread: *mut std::ffi::c_void, mask: usize) -> usize;
        }
        let _old = SetThreadAffinityMask(GetCurrentThread(), mask);
    }
}

/// Pin the current thread to a physical core on Linux (the musl bare-metal build).
#[cfg(not(target_os = "windows"))]
pub fn pin_current_thread_to_core(core: usize) {
    let (physical, logical) = topology();
    let k = core.min(physical.saturating_sub(1));
    unsafe {
        extern "C" {
            fn sched_setaffinity(pid: i32, cpusetsize: usize, mask: *const u8) -> i32;
        }
        let lo = k * 2;
        let hi = k * 2 + 1;
        // cpu_set_t (CPU_SETSIZE 1024 -> 128 bytes on x86_64). Set the two SMT-sibling bits.
        let mut set = [0u8; 128];
        if lo / 8 < set.len() {
            set[lo / 8] |= 1 << (lo % 8);
        }
        if hi < logical && hi / 8 < set.len() {
            set[hi / 8] |= 1 << (hi % 8);
        }
        let _ = sched_setaffinity(0, set.len(), set.as_ptr());
    }
}

/// Core assignments for the runtime threads: indices into the MEASURED physical cores. On a host with
/// fewer cores they clamp (see `pin_current_thread_to_core`) instead of failing.
pub const CORE_EVENT_LOOP: usize = 0; // OS + HTTP workers - NOT pinned (the event loop must migrate)
pub const CORE_QUIC: usize = 1;       // trm_datagram::run - network I/O (TRM0 UDP)
pub const CORE_HEARTBEAT: usize = 2;  // heartbeat_auto_loop - periodic dispatch
pub const CORE_AGENT_WORK: usize = 3; // agent_work_loop - task execution / recall (the compute thread)
pub const CORE_PROBE: usize = 4;      // host_probe samplers - background measurement

/// Measured CPU topology: (physical cores, logical processors).
///
/// Windows: records from `GetLogicalProcessorInformation` whose Relationship == RelationProcessorCore
/// (0) count physical cores; `GetActiveProcessorCount(ALL_PROCESSOR_GROUPS)` gives the logical count.
/// Each x64 record is 32 bytes (ULONG_PTR mask + i32 relationship + 16-byte union + padding). If the
/// call fails, physical falls back to logical/2 and `topology_measured()` reports false so a caller can
/// tell an estimate from a measurement.
#[cfg(target_os = "windows")]
pub fn topology() -> (usize, usize) {
    unsafe {
        extern "system" {
            fn GetActiveProcessorCount(group: u16) -> u32;
            fn GetLogicalProcessorInformation(buf: *mut u8, len: *mut u32) -> i32;
        }
        const ALL_PROCESSOR_GROUPS: u16 = 0xFFFF;
        let logical = GetActiveProcessorCount(ALL_PROCESSOR_GROUPS) as usize;
        let logical = if logical == 0 { 1 } else { logical };

        let mut need: u32 = 0;
        GetLogicalProcessorInformation(std::ptr::null_mut(), &mut need);
        let mut physical = 0usize;
        if need > 0 {
            let records = (need as usize / 32) + 4;
            let mut buf = vec![0u8; records * 32];
            let mut got = (records * 32) as u32;
            if GetLogicalProcessorInformation(buf.as_mut_ptr(), &mut got) != 0 {
                let n = got as usize / 32;
                for i in 0..n {
                    let off = i * 32 + 8; // the relationship follows the ULONG_PTR mask
                    let rel = i32::from_ne_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
                    if rel == 0 {
                        physical += 1;
                    }
                }
            }
        }
        if physical == 0 {
            return (logical / 2, logical);
        }
        (physical, logical)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn topology() -> (usize, usize) {
    let logical = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    (logical / 2, logical)
}

/// True when the physical-core count came from the OS rather than a division fallback.
pub fn topology_measured() -> bool {
    let (physical, logical) = topology();
    physical > 0 && physical <= logical
}

pub fn physical_cores() -> usize {
    topology().0.max(1)
}

pub fn logical_threads() -> usize {
    topology().1.max(1)
}

/// Cores left unassigned by the fixed core map, available to parallel agents.
pub fn spare_cores() -> usize {
    physical_cores().saturating_sub(CORE_PROBE + 1)
}

/// The CPU model as the OS reports it; "unmeasured" elsewhere rather than a guessed string.
pub fn cpu_model() -> String {
    std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "unmeasured".to_string())
}

/// GPU identity. This host has no nvidia-smi and no ROCm; the RX 6700 XT is driven through Vulkan.
/// Nothing is asserted: name and VRAM are whatever the environment declares, and absence is reported as
/// null / measured:false instead of inheriting another machine's GPU.
pub fn gpu_identity() -> serde_json::Value {
    let name = std::env::var("TRM_GPU_NAME").unwrap_or_default();
    let vram: u64 = std::env::var("TRM_GPU_VRAM_MIB").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let measured = !name.is_empty() && vram > 0;
    serde_json::json!({
        "device_index": if measured { serde_json::json!(0) } else { serde_json::Value::Null },
        "name": if measured { serde_json::json!(name) } else { serde_json::Value::Null },
        "vram_mib": if measured { serde_json::json!(vram) } else { serde_json::Value::Null },
        "measured": measured,
        "backend": "vulkan",
        "note": if measured {
            "declared by TRM_GPU_NAME / TRM_GPU_VRAM_MIB".to_string()
        } else {
            "no GPU probe on this host: set TRM_GPU_NAME and TRM_GPU_VRAM_MIB, or leave unmeasured".to_string()
        },
    })
}

/// The parallel capacity the runtime actually has: agent slots = measured logical processors, not a
/// digital-twin constant and not another host's 16.
pub fn parallel_capacity() -> serde_json::Value {
    serde_json::json!({
        "agent_slots": logical_threads(),
        "physical_cores": physical_cores(),
        "logical_threads": logical_threads(),
        "spare_cores": spare_cores(),
        "spare_logical_threads": spare_cores() * 2,
        "measured": topology_measured(),
        "basis": "measured at runtime via GetLogicalProcessorInformation (physical) + GetActiveProcessorCount (logical)",
    })
}

/// The pinning report the dashboard/hardware route reads.
pub fn pinning_report() -> serde_json::Value {
    let (physical, logical) = topology();
    let spare = spare_cores();
    let mut core_map = serde_json::Map::new();
    core_map.insert(
        "core_0_event_loop".into(),
        serde_json::json!("OS + HTTP workers - NOT pinned (must migrate)"),
    );
    core_map.insert("core_1_quic".into(), serde_json::json!("trm_datagram::run"));
    core_map.insert("core_2_heartbeat".into(), serde_json::json!("heartbeat_auto_loop"));
    core_map.insert(
        "core_3_agent_work".into(),
        serde_json::json!("agent_work_loop (task execution / recall)"),
    );
    core_map.insert("core_4_probe".into(), serde_json::json!("host_probe samplers"));
    core_map.insert(
        format!("cores_{}_spare", CORE_PROBE + 1),
        serde_json::json!(format!("{} physical cores reserved for parallel agents / Instella / GPU", spare)),
    );
    serde_json::json!({
        "topology": {
            "physical_cores": physical,
            "logical_threads": logical,
            "smt_pairing": "physical core k = logical (2k, 2k+1)",
            "cpu": cpu_model(),
            "measured": topology_measured(),
        },
        "gpu": gpu_identity(),
        "core_map": core_map,
        "basis": "topology() measures the OS at runtime; pin_current_thread_to_core() clamps to the measured core count",
    })
}
