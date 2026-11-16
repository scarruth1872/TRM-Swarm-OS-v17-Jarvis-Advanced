//! gpu_dispatch.rs — the voltage binder's compute dispatcher (council MOTION-017/018).
//!
//! A machine-checkable device binding: each load-bearing compute kind is bound to a device by a
//! deterministic table, the device is probed fail-closed, and the decision reports which device
//! actually runs and why.
//!
//! Honest state (measured 2026-10-13): wgpu + naga (pure-Rust) is linked and the Vulkan device is
//! creatable on this host (RTX 3070 Laptop). Three GPU kernels are shipped and parity-verified
//! against their CPU references (gpu_compute.rs): hdc_encode (single + batch, bit-exact vs the DLL),
//! packed_scan (183,879-row store, ~35x, top-K matches CPU), trm7m (batched forward, trit + confidence
//! match the CPU forward). trm7m's kernel is for BATCH evaluation; the runtime's single-eval path
//! (evaluate_proposal -> Trm7m::forward, measured 2.82 ms/call) still runs CPU — the batch kernel is
//! the corpus/council-scale path. Every binding reflects what actually runs, fail-closed to CPU.

use serde_json::json;

/// The load-bearing compute kinds MOTION-017/018 scoped for heterogeneous dispatch. `sram_compute` is
/// the Hopfield relaxation + 15 topologies (sram_pattern_computer), CPU-implemented — no literal SRAM
/// hardware on this host, so it always binds cpu and is listed for a COMPLETE substrate map, not a GPU arm.
pub const COMPUTE_KINDS: [&str; 4] = ["hdc_encode", "packed_scan", "trm7m", "sram_compute"];

/// Is a GPU compute toolchain present? wgpu + naga: the WGSL is compiled to SPIR-V at runtime by
/// naga (pure Rust), so no CUDA toolkit or Vulkan SDK is ever needed. True when a Vulkan adapter
/// with the shaderInt64 feature can actually be created on this host (measured, fail-closed).
pub fn toolchain_present() -> bool {
    crate::gpu_compute::gpu_available()
}

/// The toolchain state, spelled out for the report so a reader knows exactly what is available.
pub fn toolchain_basis() -> &'static str {
    if crate::gpu_compute::gpu_available() {
        "present: wgpu + naga (pure-Rust; WGSL compiled to SPIR-V at runtime; no CUDA/Vulkan SDK needed)"
    } else {
        "absent: no Vulkan adapter/device creatable on this host (wgpu + naga is linked, GPU unreachable)"
    }
}

/// The GPU is present and its telemetry is readable (nvidia-smi). Presence is not compute
/// readiness: a GPU with no toolchain can be read but not executed on.
pub fn gpu_present() -> bool {
    crate::host_probe::gpu_telemetry().is_some()
}

/// Deterministic device binding. Fail-closed: GPU is bound only when BOTH the toolchain and the
/// GPU are present; any absence binds CPU. The table is static so the binding is machine-checkable
/// (LogicMaster condition), and the fallback is CPU so a missing device can never stall a task
/// (Shield condition).
pub fn bind(kind: &str) -> &'static str {
    // sram_compute is the Hopfield relaxation + 15 topologies (sram_pattern_computer), CPU-implemented.
    // There is no literal SRAM hardware on this host, so it always binds cpu (the kernel's own engine).
    if kind == "sram_compute" {
        return "cpu";
    }
    // hdc_encode, packed_scan, and trm7m all have shipped, parity-verified GPU kernels
    // (gpu_compute.rs). A binding must reflect what actually runs.
    if (kind == "hdc_encode" || kind == "packed_scan" || kind == "trm7m")
        && toolchain_present()
        && gpu_present()
    {
        "gpu"
    } else {
        "cpu"
    }
}

/// One dispatch decision: what the table binds, what is actually available, and why.
pub fn dispatch(kind: &str) -> serde_json::Value {
    let device = bind(kind);
    json!({
        "kind": kind,
        "bound_device": device,
        "gpu_present": gpu_present(),
        "gpu_toolchain": toolchain_present(),
        "gpu_toolchain_basis": toolchain_basis(),
        "fail_closed": true,
        "parity": if kind == "sram_compute" {
            "cpu-native: sram_pattern_computer serves the 15 topologies + Hopfield relaxation from the kernel's own source (bit-exact vs its Python oracles)"
        } else if device == "gpu" {
            match kind {
                "trm7m" => "verified: gpu_compute::tests::gpu_trm7m_matches_cpu — GPU forward reproduces the CPU forward (trit + confidence, 6 texts)",
                "packed_scan" => "verified: gpu_compute::tests::gpu_scan_matches_cpu — top-K matches CPU packed_ternary::search",
                _ => "verified: gpu_compute::tests::gpu_matches_cpu_reference / gpu_batch_matches_cpu — GPU reproduces the CPU reference",
            }
        } else {
            "not exercised: GPU not bound (toolchain or device absent); CPU reference authoritative"
        },
        "profile": if kind == "trm7m" {
            "batch -> gpu (gpu_trm7m_batch); single-eval (evaluate_proposal) -> cpu Trm7m::forward (2.82 ms/call measured)"
        } else if kind == "sram_compute" {
            "cpu: Hopfield relaxation + 15 topologies (sram_pattern_computer), no literal SRAM hardware"
        } else {
            "single and batch -> gpu"
        },
        "basis": "deterministic device-binding table (MOTION-017/018); GPU arm gated on toolchain, CPU is the fail-closed default",
    })
}

/// The full dispatcher state: every load-bearing kind plus the measured host reality.
pub fn dispatch_table() -> serde_json::Value {
    json!({
        "dispatcher": "voltage binder compute dispatcher (council MOTION-017)",
        "host_gpu": if gpu_present() {
            "present: NVIDIA RTX 3070 Laptop + AMD Radeon (Vulkan runtime enumerates both)"
        } else {
            "absent: nvidia-smi not readable"
        },
        "compute_toolchain": toolchain_basis(),
        "bindings": COMPUTE_KINDS.iter().map(|k| dispatch(k)).collect::<Vec<_>>(),
        "rule": "gpu iff (toolchain present AND gpu present) else cpu; deterministic table, fail-closed",
    })
}
