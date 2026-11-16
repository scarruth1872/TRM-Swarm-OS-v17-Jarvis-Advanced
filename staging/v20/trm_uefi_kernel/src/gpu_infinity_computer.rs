//! TRM Swarm OS - gpu_infinity_computer (no_std port).
//!
//! Portable half of `swarm_v2/core/gpu_infinity_computer.py`: the four Infinity Cache pattern
//! generators (`gpu_pattern_fpga` / `gpu_pattern_neural` / `gpu_pattern_cellular` /
//! `gpu_pattern_k3_ternary`), the CPU-vs-GPU `compare_fabrics` tally, and the cache-line
//! identity hash from `native_voltage_strips/strip_gpu_infinity.rs` (FNV-1a over a 128-byte line
//! filled cyclically from the architecture's own name -- the rule that replaced the old
//! 0xAA/0x55/0x01/0xFF index-switch, which collapsed every architecture past the third).
//!
//! The full-pattern structs and `compare_fabrics` are the differential-vector surface
//! (`vectors/gpu_infinity_computer.json`); the kernel's live callers consume only the numeric
//! tuple ABIs, so those richer forms carry `#[allow(dead_code)]` until the harness wires them.
//!
//! Host-side and not ported: `load_architecture`/`benchmark_all`/`benchmark_all_native` (they
//! drive a real GPU and a real clock), `_probe_gpu` (a subprocess `wmic` probe), `_write_pattern_
//! to_device` (a Vulkan device buffer), and `summary` (host probe + timing strings).

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

/// The oracle's `ARCHITECTURES_GPU`, in its own order. Owned by the generated strip
/// (`strip_gpu_infinity.rs` NAMES) and reproduced here so the cache-line hash and the derived
/// architecture count share one source rather than drifting apart.
pub const GPU_ARCH_NAMES: [&str; 4] = [
    "GPU FPGA-Lite",
    "GPU SRAM Neural Network",
    "GPU Cellular Automaton",
    "GPU K3 Ternary Engine",
];

/// `strip_gpu_infinity.rs`'s `line_hash`: FNV-1a over one 128-byte Infinity Cache line, filled
/// cyclically from `name` (zero-filled when the name is empty). Hashing the name is what makes a
/// reconfiguration carry its architecture -- the old index-switch gave one value for every index
/// past the third.
pub fn line_hash(name: &str) -> u32 {
    let b = name.as_bytes();
    let mut h: u32 = 2166136261;
    for i in 0..128usize {
        let v = if b.is_empty() { 0u8 } else { b[i % b.len()] };
        h = (h ^ (v as u32)).wrapping_mul(16777619);
    }
    h
}

/// `strip_gpu_infinity.rs`'s `strip_gpu_arch_hash`: the line hash of architecture `idx` mod the
/// architecture count, so every index hashes its own architecture.
pub fn gpu_arch_hash(idx: u32) -> u32 {
    let i = (idx as usize) % GPU_ARCH_NAMES.len();
    line_hash(GPU_ARCH_NAMES[i])
}

/// `strip_batch_gpu_infinity_reconfig`'s per-index hash. The name-based rule, not the old
/// 0xAA/0x55/0x01/0xFF byte-switch: `infinity_cache_hashes(&[0, 1, 2, 7, 0])` now yields the four
/// distinct architecture hashes instead of three.
pub fn infinity_cache_line_hash(index: i32) -> u32 {
    gpu_arch_hash(index as u32)
}

/// Batch entry point, mirroring `strip_batch_gpu_infinity_reconfig`.
pub fn infinity_cache_hashes(indices: &[i32]) -> Vec<u32> {
    indices.iter().map(|i| infinity_cache_line_hash(*i)).collect()
}

/// `gpu_pattern_fpga`: one 7-input LUT per 128-byte Infinity Cache line. Full dict, in the
/// oracle's field order.
#[allow(dead_code)]
pub struct FpgaPattern {
    pub name: &'static str,
    pub cache_used: String,
    pub logic_cells: u64,
    pub lut_size: u64,
    pub total_luts: u64,
    pub bandwidth: &'static str,
}

#[allow(dead_code)]
pub fn gpu_pattern_fpga(size_mb: u64) -> FpgaPattern {
    let lines = (size_mb * 1024 * 1024) / 128;
    FpgaPattern {
        name: "GPU FPGA-Lite",
        cache_used: alloc::format!("{}MB Infinity Cache", size_mb),
        logic_cells: lines,
        lut_size: 7,
        total_luts: lines,
        bandwidth: "2.1 TB/s",
    }
}

/// The strip's numeric ABI for fpga: `(logic_cells, lut_size)`.
pub fn sram_lut_fpga(size_mb: u64) -> (u64, u64) {
    let p = gpu_pattern_fpga(size_mb);
    (p.logic_cells, p.lut_size)
}

/// `gpu_pattern_neural`: 8 bytes per neuron (16-bit weights x 4) and 4-way connectivity.
#[allow(dead_code)]
pub struct NeuralPattern {
    pub name: &'static str,
    pub cache_used: String,
    pub neurons: u64,
    pub synapses: u64,
    pub precision: &'static str,
}

#[allow(dead_code)]
pub fn gpu_pattern_neural(size_mb: u64) -> NeuralPattern {
    let neurons = (size_mb * 1024 * 1024) / 8;
    NeuralPattern {
        name: "GPU SRAM Neural Network",
        cache_used: alloc::format!("{}MB Infinity Cache", size_mb),
        neurons,
        synapses: neurons * 4,
        precision: "16-bit floating point",
    }
}

/// The strip's numeric ABI for neural: `(neurons, synapses)`.
pub fn pattern_neural(size_mb: u64) -> (u64, u64) {
    let p = gpu_pattern_neural(size_mb);
    (p.neurons, p.synapses)
}

/// `gpu_pattern_cellular`: cells at 8 bits each, and the python's update rate
/// `2.1e12 / (100e6 * 8)` over a 100M-cell reference grid.
#[allow(dead_code)]
pub struct CellularPattern {
    pub name: &'static str,
    pub cache_used: String,
    pub grid: &'static str,
    pub cells: u64,
    pub updates_per_second: f64,
}

#[allow(dead_code)]
pub fn gpu_pattern_cellular(size_mb: u64) -> CellularPattern {
    let cells = (size_mb * 1024 * 1024 * 8) / 8;
    let grid_cells = 100_000_000u64;
    let updates = 2.1e12 / (grid_cells as f64 * 8.0);
    CellularPattern {
        name: "GPU Cellular Automaton",
        cache_used: alloc::format!("{}MB (full Infinity Cache)", size_mb),
        grid: "8192 \u{00d7} 8192 cells (approximately)",
        cells,
        updates_per_second: updates,
    }
}

/// The strip's numeric ABI for cellular: `(cells, updates_per_second)`.
pub fn pattern_cellular(size_mb: u64) -> (u64, f64) {
    let p = gpu_pattern_cellular(size_mb);
    (p.cells, p.updates_per_second)
}

/// `gpu_pattern_k3_ternary`: 2 cells per trit over a full 96MB cache, and the python's
/// throughput `trits * 2.1e12 / 1e12`.
#[allow(dead_code)]
pub struct K3Pattern {
    pub name: &'static str,
    pub cache_used: &'static str,
    pub trits: u64,
    pub cells_used: u64,
    pub encoding: &'static str,
    pub throughput_tps: f64,
    pub instruction: &'static str,
}

#[allow(dead_code)]
pub fn gpu_pattern_k3_ternary() -> K3Pattern {
    let cells = 96u64 * 1024 * 1024 * 8;
    let trits = cells / 2;
    K3Pattern {
        name: "GPU K3 Ternary Engine",
        cache_used: "96MB Infinity Cache",
        trits,
        cells_used: cells,
        encoding: "2 cells per trit",
        throughput_tps: trits as f64 * 2.1e12 / 1e12,
        instruction: "S_BITCMP_B32 (RDNA2 native)",
    }
}

/// The strip's numeric ABI for k3: `(cells_used, trits, throughput_tps)`.
pub fn pattern_k3_ternary() -> (u64, u64, f64) {
    let p = gpu_pattern_k3_ternary();
    (p.cells_used, p.trits, p.throughput_tps)
}

/// One fabric block of `compare_fabrics` (cpu / gpu share this shape).
#[allow(dead_code)]
pub struct FabricBlock {
    pub cache: &'static str,
    pub cells: &'static str,
    pub bandwidth: &'static str,
    pub transistors: &'static str,
    pub architectures: u64,
    pub architectures_basis: &'static str,
}

/// The `combined` block of `compare_fabrics`, whose keys differ from the per-fabric blocks.
#[allow(dead_code)]
pub struct CombinedBlock {
    pub total_sram: &'static str,
    pub total_cells: &'static str,
    pub total_transistors: &'static str,
    pub peak_bandwidth: &'static str,
    pub architectures: String,
}

#[allow(dead_code)]
pub struct FabricComparison {
    pub cpu: FabricBlock,
    pub gpu: FabricBlock,
    pub combined: CombinedBlock,
}

/// `compare_fabrics`: CPU vs GPU fabric. The GPU count is DERIVED from `ARCHITECTURES_GPU`
/// (`GPU_ARCH_NAMES.len()`); the CPU count is declared (owned by `sram_pattern_computer`).
#[allow(dead_code)]
pub fn compare_fabrics() -> FabricComparison {
    let gpu_n = GPU_ARCH_NAMES.len() as u64;
    let cpu_n = 5u64; // declared: owned by sram_pattern_computer, not recounted here
    FabricComparison {
        cpu: FabricBlock {
            cache: "32MB L3",
            cells: "268M SRAM",
            bandwidth: "~200 GB/s (read)",
            transistors: "1.61B accessible (SRAM)",
            architectures: cpu_n,
            architectures_basis: "declared: owned by sram_pattern_computer, not recounted here",
        },
        gpu: FabricBlock {
            cache: "96MB Infinity Cache (design target)",
            cells: "805M SRAM (design target)",
            bandwidth: "2.1 TB/s (design target)",
            transistors: "4.83B accessible (IC SRAM)",
            architectures: gpu_n,
            architectures_basis: "derived: len(ARCHITECTURES_GPU)",
        },
        combined: CombinedBlock {
            total_sram: "128MB (32+96, design target)",
            total_cells: "1.07B SRAM cells (design target)",
            total_transistors: "6.44B programmable",
            peak_bandwidth: "2.3 TB/s (design target)",
            architectures: alloc::format!("{} total ({} CPU + {} GPU)", cpu_n + gpu_n, cpu_n, gpu_n),
        },
    }
}
