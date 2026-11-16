//! TRM Swarm OS - gpu_infinity_computer (no_std port).
//!
//! Portable half of `swarm_v2/core/gpu_infinity_computer.py`: the four Infinity Cache pattern
//! generators, which are pure integer arithmetic, and the strip's cache-line hash from
//! `native_voltage_strips/strip_gpu_infinity.rs`.
//!
//! Host-side and not ported: `load_architecture`/`benchmark_all` (they drive a real GPU) and the
//! `summary` reporting.

extern crate alloc;
use alloc::vec::Vec;

/// `gpu_pattern_fpga`: one 7-input LUT per 128-byte Infinity Cache line.
pub fn sram_lut_fpga(size_mb: u64) -> (u64, u64) {
    let lines = (size_mb * 1024 * 1024) / 128;
    (lines, 7)
}

/// `gpu_pattern_neural`: 8 bytes per neuron (16-bit weights x 4) and 4-way connectivity.
pub fn pattern_neural(size_mb: u64) -> (u64, u64) {
    let neurons = (size_mb * 1024 * 1024) / 8;
    (neurons, neurons * 4)
}

/// `gpu_pattern_cellular`: cells at 8 bits each, and the python's update rate
/// `2.1e12 / (100e6 * 8)` over a 100M-cell reference grid.
pub fn pattern_cellular(size_mb: u64) -> (u64, f64) {
    let cells = (size_mb * 1024 * 1024 * 8) / 8;
    let grid_cells = 100_000_000u64;
    let updates = 2.1e12 / (grid_cells as f64 * 8.0);
    (cells, updates)
}

/// `gpu_pattern_k3_ternary`: 2 cells per trit over a full 96MB cache, and the python's throughput
/// `trits * 2.1e12 / 1e12`.
pub fn pattern_k3_ternary() -> (u64, u64, f64) {
    let cells = 96u64 * 1024 * 1024 * 8;
    let trits = cells / 2;
    (cells, trits, trits as f64 * 2.1e12 / 1e12)
}

/// `strip_batch_gpu_infinity_reconfig`: an FNV-1a 32 hash of a 128-byte cache line filled with a byte
/// chosen by the index -- 0xAA for 0, 0x55 for 1, 0x01 for 2, 0xFF otherwise. The hash is folded over
/// the 128 bytes, which is why the same index always yields the same value.
pub fn infinity_cache_line_hash(index: i32) -> u32 {
    let byte_val: u8 = match index {
        0 => 0xAA,
        1 => 0x55,
        2 => 0x01,
        _ => 0xFF,
    };
    let mut h: u32 = 2166136261;
    for _ in 0..128 {
        h = (h ^ (byte_val as u32)).wrapping_mul(16777619);
    }
    h
}

/// Batch entry point, mirroring `strip_batch_gpu_infinity_reconfig`.
pub fn infinity_cache_hashes(indices: &[i32]) -> Vec<u32> {
    indices.iter().map(|i| infinity_cache_line_hash(*i)).collect()
}
