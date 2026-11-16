//! CPU core pinning for the runtime worker threads + the GPU device pin.
//!
//! The 5900HX topology is MEASURED on this host, not asserted:
//!   8 physical cores / 16 logical threads; physical core k = logical (2k, 2k+1)
//!   (verified via GetLogicalProcessorInformationEx, RelationshipProcessorCore).
//! The GPU is NVIDIA GeForce RTX 3070 Laptop GPU, device 0, 8192 MiB.
//!
//! This is the OS-level counterpart to `hardware_binder::bind_task_to_silicon` (the model
//! binding). `bind_task_to_silicon` maps a task to a silicon region + voltage rail; this pins
//! the runtime thread that executes it to a real physical core so the platform's compute
//! threads are not preempted — the "fully utilized" half the model binding alone cannot give.
//! The 8 HTTP workers are NOT pinned: they are the event loop and must be free to migrate.

/// Pin the current thread to a physical core (both SMT siblings) — core k -> logical (2k, 2k+1).
#[cfg(target_os = "windows")]
pub fn pin_current_thread_to_core(core: usize) {
    let lo = core * 2;
    let hi = core * 2 + 1;
    let mask = (1usize << lo) | (1usize << hi);
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
    unsafe {
        extern "C" {
            fn sched_setaffinity(pid: i32, cpusetsize: usize, mask: *const u8) -> i32;
        }
        let lo = core * 2;
        let hi = core * 2 + 1;
        // cpu_set_t (CPU_SETSIZE 1024 -> 128 bytes on x86_64). Set the two SMT-sibling bits.
        let mut set = [0u8; 128];
        set[lo / 8] |= 1 << (lo % 8);
        set[hi / 8] |= 1 << (hi % 8);
        let _ = sched_setaffinity(0, set.len(), set.as_ptr());
    }
}

/// The measured core map for the 5900HX (8C/16T) + RTX 3070 runtime threads.
pub const CORE_EVENT_LOOP: usize = 0; // OS + the 8 HTTP workers — NOT pinned (the event loop must migrate)
pub const CORE_QUIC: usize = 1;       // trm_datagram::run — network I/O (TRM0 UDP)
pub const CORE_HEARTBEAT: usize = 2;  // heartbeat_auto_loop — periodic dispatch
pub const CORE_AGENT_WORK: usize = 3; // agent_work_loop — task execution / recall (the compute thread)
pub const CORE_PROBE: usize = 4;      // host_probe samplers — background measurement

/// The GPU device the GENERATIVE bindings target (measured: nvidia-smi device 0).
pub const GPU_DEVICE_INDEX: usize = 0;
pub const GPU_DEVICE_NAME: &str = "NVIDIA GeForce RTX 3070 Laptop GPU";
pub const GPU_DEVICE_VRAM_MIB: usize = 8192;

/// The measured parallelism ceiling: 8 physical cores / 16 logical threads (SMT). This is the
/// real "how many things can run concurrently" — the honest substitute for the 584-agent
/// digital-twin constant. 3 cores are reserved spare for parallel agents / Instella / GPU.
/// Council motion "Real capacity, not 584" (PROCEED).
pub const PHYSICAL_CORES: usize = 8;
pub const LOGICAL_THREADS: usize = 16;
pub const SPARE_CORES: usize = 3; // cores 5-7, reserved for parallel agents

/// The parallel capacity the runtime actually has: agent_slots = the 16 logical threads (the
/// real concurrency ceiling), not the 584 digital-twin constant the reconciliation retired.
pub fn parallel_capacity() -> serde_json::Value {
    serde_json::json!({
        "agent_slots": LOGICAL_THREADS,
        "physical_cores": PHYSICAL_CORES,
        "logical_threads": LOGICAL_THREADS,
        "spare_cores": SPARE_CORES,
        "spare_logical_threads": SPARE_CORES * 2,
        "basis": "measured topology (GetLogicalProcessorInformationEx): 8C/16T; agent_slots = logical_threads, the real concurrency ceiling — the 584 'digital-twin singleton' is retired from any live claim",
    })
}

/// The pinning report the dashboard/hardware route reads: which core each runtime thread is
/// pinned to, derived from the measured topology above (not a per-thread live read-back).
pub fn pinning_report() -> serde_json::Value {
    serde_json::json!({
        "topology": {
            "physical_cores": 8,
            "logical_threads": 16,
            "smt_pairing": "physical core k = logical (2k, 2k+1) — measured via GetLogicalProcessorInformationEx",
            "cpu": "AMD Ryzen 9 5900HX (Zen 3 Cezanne)",
        },
        "gpu": {
            "device_index": GPU_DEVICE_INDEX,
            "name": GPU_DEVICE_NAME,
            "vram_mib": GPU_DEVICE_VRAM_MIB,
        },
        "core_map": {
            "core_0_event_loop": "OS + 8 HTTP workers — NOT pinned (must migrate)",
            "core_1_quic": "trm_datagram::run",
            "core_2_heartbeat": "heartbeat_auto_loop",
            "core_3_agent_work": "agent_work_loop (task execution / recall)",
            "core_4_probe": "host_probe samplers",
            "cores_5_7_spare": "reserved for parallel agents / Instella / GPU offload",
        },
        "basis": "pin_current_thread_to_core() is called at the top of each worker-thread closure",
    })
}
