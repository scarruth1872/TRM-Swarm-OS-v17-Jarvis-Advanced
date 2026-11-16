//! Total Silicon Fabric — Sovereign Hardware Unification
//! Unifies CPU Threads (48) + GPU Cores (7,680) = 7,728 Compute Units
//! Unifies DRAM (64GB) + SRAM (120MB) = ~1.1 Trillion Trits
//! Unifies ~69.5 Billion Transistors as one sovereign gate array

use crate::MONAD_TRUTH_ANCHOR;
use crate::silicon_crystallography::SiliconCrystallography;
use crate::voltage_doctrine_harmonization_engine::VoltageDoctrineHarmonizer;

#[derive(Debug, Clone)]
pub struct TotalSiliconFabric {
    // CPU Substrates (Laptop + Desktop + Edge)
    pub cpu_threads: usize,
    pub cpu_clock_mhz: f64,
    
    // GPU Substrates (Laptop RTX 3070 + Desktop RX 6700 XT)
    pub gpu_cores: usize,
    pub gpu_clock_mhz: f64,
    
    // Memory Substrates (DDR4 RAM + Infinity Cache SRAM)
    pub dram_bytes: u64,
    pub dram_trits: u64,
    pub sram_bytes: u64,
    pub sram_cells: u64,
    
    // Physical Silicon Substrates
    pub transistors: u64,
    
    // Voltage Harmonics
    pub unified_rail_v: f64,
    pub harmonized_phi_v: f64,
    /// Where `harmonized_phi_v` came from: a host derivation or a stated reference.
    pub phi_v_basis: &'static str,
}

#[derive(Debug, Clone)]
pub struct UnifiedSilicon {
    pub total_compute_units: u64,
    pub total_trit_capacity: u64,
    pub total_transistor_gates: u64,
    pub unified_clock_ghz: f64,
    pub unified_rail_v: f64,
    pub harmonized_phi_v: f64,
    pub phi_v_basis: &'static str,
    pub monad_anchor: u64,
    pub identity_signature: String,
}

/// Nominal figures for the reference desktop + laptop + edge substrate this fabric
/// is designed around. These are SPECIFICATIONS, not measurements taken here.
pub struct DesignReference;

impl DesignReference {
    pub const CPU_CLOCK_MHZ: f64 = 3650.0; // nominal
    pub const GPU_CORES: usize = 7680; // RX 6700 XT, vendor spec
    pub const GPU_CLOCK_MHZ: f64 = 2070.0; // RX 6700 XT, vendor spec
    pub const DRAM_BYTES: u64 = 64 * 1024 * 1024 * 1024; // 64 GB
    pub const SRAM_BYTES: u64 = 120 * 1024 * 1024; // 120 MB
    pub const TRANSISTORS: u64 = 69_500_000_000; // ~69.5 Billion
    pub const UNIFIED_RAIL_V: f64 = 170.732;
    // `HARMONIZED_PHI_V = 0.8681` was a published constant here. Φ_V is derived from
    // the host instead (see VoltageDoctrineHarmonizer::derived_phi_v); the constant
    // disagreed with the derivation by ~1.7x and 0.8681 x 1.05 is where the 0.9115
    // quoted elsewhere came from.
}

impl Default for TotalSiliconFabric {
    /// `cpu_threads` is read from this host at runtime. Every other field is a
    /// DESIGN REFERENCE for the intended substrate, not a reading from the machine
    /// running this binary.
    ///
    /// This previously asserted `cpu_threads: 48` on every host regardless of the
    /// machine, and presented `unified_rail_v` / `harmonized_phi_v` as though they
    /// were sensed values.
    fn default() -> Self {
        // Measured: how many threads this process can actually run.
        let cpu_threads: usize = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0);

        // Φ_V is derived from this host, never asserted (paper_03 / paper_149).
        let (phi_v, phi_v_basis, host_clock_mhz, _mem_pct) =
            VoltageDoctrineHarmonizer::derived_phi_v();
        // Adopt the clock only when the derivation actually read one. Its fallback is
        // the 4000 MHz doctrine reference, which is not this machine's clock and must
        // not be presented as a reading.
        let cpu_clock_mhz = if phi_v_basis.starts_with("measured") {
            host_clock_mhz
        } else {
            DesignReference::CPU_CLOCK_MHZ
        };

        let dram_bytes: u64 = DesignReference::DRAM_BYTES;
        let sram_bytes: u64 = DesignReference::SRAM_BYTES;

        Self {
            cpu_threads, // measured on this host
            cpu_clock_mhz, // measured when the host query resolved, else nominal
            gpu_cores: DesignReference::GPU_CORES,
            gpu_clock_mhz: DesignReference::GPU_CLOCK_MHZ,
            dram_bytes,
            dram_trits: dram_bytes * 8 * 2, // design reference at 64 GB
            sram_bytes,
            sram_cells: sram_bytes * 8, // design reference at 120 MB
            transistors: DesignReference::TRANSISTORS,
            unified_rail_v: DesignReference::UNIFIED_RAIL_V,
            harmonized_phi_v: phi_v,
            phi_v_basis,
        }
    }
}

impl TotalSiliconFabric {
    pub fn new() -> Self {
        Self::default()
    }

    /// Unify all hardware resources into one sovereign compute fabric
    pub fn unify(&self) -> UnifiedSilicon {
        let compute_units = (self.cpu_threads + self.gpu_cores) as u64;
        let trit_fabric = self.dram_trits + self.sram_cells;
        let gates = self.transistors;
        // Highest nominal node clock in the reference substrate. (This was the
        // literal `125.561` — a 125.561 GHz clock, which no silicon runs at. It was
        // an aggregate figure that had been relabelled as a clock.)
        let clock_ghz = self.cpu_clock_mhz / 1000.0;
        let phonon_phi_v = SiliconCrystallography::compute_phonon_harmonized_voltage(self.harmonized_phi_v, 3.30);

        let sig = format!(
            "COMPUTE-{}::TRITS-{}B::GATES-{}B::PHI_V-{:.4}::RAIL-{:.2}V::AN-{:016X}",
            compute_units,
            trit_fabric / 1_000_000_000,
            gates / 1_000_000_000,
            phonon_phi_v,
            self.unified_rail_v,
            MONAD_TRUTH_ANCHOR
        );

        UnifiedSilicon {
            total_compute_units: compute_units,
            total_trit_capacity: trit_fabric,
            total_transistor_gates: gates,
            unified_clock_ghz: clock_ghz,
            unified_rail_v: self.unified_rail_v,
            harmonized_phi_v: phonon_phi_v,
            phi_v_basis: self.phi_v_basis,
            monad_anchor: MONAD_TRUTH_ANCHOR,
            identity_signature: sig,
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_total_silicon_compute_units() -> u64 {
    let fabric = TotalSiliconFabric::default();
    (fabric.cpu_threads + fabric.gpu_cores) as u64
}

#[no_mangle]
pub extern "C" fn rust_total_silicon_trits() -> u64 {
    let fabric = TotalSiliconFabric::default();
    fabric.dram_trits + fabric.sram_cells
}

#[no_mangle]
pub extern "C" fn rust_total_silicon_transistors() -> u64 {
    let fabric = TotalSiliconFabric::default();
    fabric.transistors
}
