//! TRM Swarm OS — DRAM Pattern Computer (real no_std port).
//! 1:1 pure math from swarm_v2/core/dram_pattern_computer.py.
//! 32GB DDR4 as programmable fabric: 274.9B cells, 128 banks, 51.2 GB/s.
//! Three integrated pathways (swarm voted MULTI):
//!   1. Row Buffer Pattern  — 8KB per bank, deterministic, 45ns row cycle
//!   2. Bulk K3 Fabric      — 116.8B trits across full DIMMs
//!   3. RowHammer           — probabilistic bit-flip (emergence)
//!
//! Only the deterministic pattern-computation math is ported (numeric params
//! in, numeric patterns out). The Python mmap/ctypes CDYLIB loader, the
//! benchmark timing loops, and self-test prints are not ported. Float
//! expressions reproduce Python's exact left-to-right operation order so
//! results match the pure-Python fallback bit-for-bit.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// DRAM_SPECS (1:1 dict from dram_pattern_computer.py).
// ---------------------------------------------------------------------------
pub const DRAM_CAPACITY_GB: u64 = 32;
/// python: 32 * 1024 * 1024 * 1024 * 8 == 274,877,906,944
pub const DRAM_CELLS: u64 = DRAM_CAPACITY_GB * 1024 * 1024 * 1024 * 8;
pub const DRAM_TYPE: &str = "DDR4-3200";
pub const DRAM_CHANNELS: u64 = 2;
pub const DRAM_BANDWIDTH_GBPS: f64 = 51.2;
/// 2 DIMM × 8 chips × 8 banks
pub const DRAM_BANKS: u64 = 128;
/// 8KB per bank
pub const DRAM_ROW_BUFFER_BYTES: u64 = 8192;
pub const DRAM_ROW_CYCLE_NS: u64 = 45;
pub const DRAM_REFRESH_INTERVAL_US: f64 = 7.8;
pub const DRAM_CAS_LATENCY_NS: u64 = 14;
/// Activations needed to flip an adjacent bit.
pub const DRAM_ROWHAMMER_THRESHOLD: u64 = 100_000;

/// Struct mirror of the python DRAM_SPECS dict (numeric fields).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DramSpecs {
    pub capacity_gb: u64,
    pub cells: u64,
    pub channels: u64,
    pub bandwidth_gbps: f64,
    pub banks: u64,
    pub row_buffer_bytes: u64,
    pub row_cycle_ns: u64,
    pub refresh_interval_us: f64,
    pub cas_latency_ns: u64,
    pub rowhammer_threshold: u64,
}

pub fn dram_specs() -> DramSpecs {
    DramSpecs {
        capacity_gb: DRAM_CAPACITY_GB,
        cells: DRAM_CELLS,
        channels: DRAM_CHANNELS,
        bandwidth_gbps: DRAM_BANDWIDTH_GBPS,
        banks: DRAM_BANKS,
        row_buffer_bytes: DRAM_ROW_BUFFER_BYTES,
        row_cycle_ns: DRAM_ROW_CYCLE_NS,
        refresh_interval_us: DRAM_REFRESH_INTERVAL_US,
        cas_latency_ns: DRAM_CAS_LATENCY_NS,
        rowhammer_threshold: DRAM_ROWHAMMER_THRESHOLD,
    }
}

/// Cells per row buffer: row_buffer_bytes * 8 = 65,536.
pub fn cells_per_row_buffer_bank() -> u64 {
    DRAM_ROW_BUFFER_BYTES * 8
}

// ---------------------------------------------------------------------------
// Pathway 1: Row Buffer Pattern (deterministic).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct RowBufferPattern {
    pub name: &'static str,
    pub banks: u64,
    pub cells_per_bank: u64,
    pub total_cells: u64,
    pub access_time_ns: u64,
    /// python f-string value: banks*cells_per_bank/8/row_cycle_ns*1e9/1e9
    pub bandwidth_gbps: f64,
    pub use_case: &'static str,
}

impl RowBufferPattern {
    /// python: f'{bandwidth_gbps:.1f} GB/s per row cycle'
    pub fn bandwidth_text(&self) -> String {
        format!("{:.1} GB/s per row cycle", self.bandwidth_gbps)
    }
}

/// python pathway_row_buffer(banks_to_use=128). Expression order preserved:
/// total/8/45*1e9/1e9 (each step a Python float op).
pub fn pathway_row_buffer(banks_to_use: u64) -> RowBufferPattern {
    let cells_per_bank = cells_per_row_buffer_bank();
    let total_cells = banks_to_use * cells_per_bank;
    let bandwidth = (total_cells as f64) / 8.0 / 45.0 * 1e9 / 1e9;
    RowBufferPattern {
        name: "Row Buffer Pattern (Deterministic)",
        banks: banks_to_use,
        cells_per_bank,
        total_cells,
        access_time_ns: DRAM_ROW_CYCLE_NS,
        bandwidth_gbps: bandwidth,
        use_case: "FPGA-lite, small LUTs, agent state vectors",
    }
}

// ---------------------------------------------------------------------------
// Pathway 2: Bulk K3 Fabric (full DIMM).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct BulkK3Fabric {
    pub name: &'static str,
    pub total_cells: u64,
    /// cells * 0.85 (15% refresh overhead) — python float.
    pub usable_cells: f64,
    /// usable / 2 — python float (116,823,110,451.2 for the full DIMM set).
    pub trits: f64,
    pub encoding: &'static str,
    /// trits * 51.2e9 / 1e12 — python float.
    pub throughput_tps: f64,
    pub refresh_overhead: &'static str,
    pub use_case: &'static str,
}

/// python pathway_bulk_k3(). Expression order preserved: cells*0.85/2 and
/// trits*51.2e9/1e12.
pub fn pathway_bulk_k3() -> BulkK3Fabric {
    let total_cells = DRAM_CELLS;
    let usable_cells = (total_cells as f64) * 0.85;
    let trits = usable_cells / 2.0;
    let throughput_tps = trits * 51.2e9 / 1e12;
    BulkK3Fabric {
        name: "Bulk K3 Ternary Fabric",
        total_cells,
        usable_cells,
        trits,
        encoding: "2 cells per trit (with refresh)",
        throughput_tps,
        refresh_overhead: "15% of bandwidth",
        use_case: "Massive K3 state space, neural weight matrices, CA grids",
    }
}

// ---------------------------------------------------------------------------
// Pathway 3: RowHammer Probabilistic (emergent).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct RowhammerPattern {
    pub name: &'static str,
    pub mechanism: &'static str,
    pub activations_per_flip: u64,
    /// 1 / rowhammer_threshold (python literal 1e-5).
    pub flip_probability_per_activation: f64,
    pub stochastic_model: &'static str,
    pub use_case: &'static str,
    pub warning: &'static str,
}

impl RowhammerPattern {
    /// python: f'{1/DRAM_SPECS["rowhammer_threshold"]:.5%} per activation'
    /// -> "0.00100% per activation" for the 100k threshold.
    pub fn flip_probability_text(&self) -> String {
        format!(
            "{:.5}% per activation",
            self.flip_probability_per_activation * 100.0
        )
    }
}

/// python pathway_rowhammer().
pub fn pathway_rowhammer() -> RowhammerPattern {
    RowhammerPattern {
        name: "RowHammer Probabilistic Computing",
        mechanism: "Repeated row activation - adjacent bit flips",
        activations_per_flip: DRAM_ROWHAMMER_THRESHOLD,
        flip_probability_per_activation: 1.0 / DRAM_ROWHAMMER_THRESHOLD as f64,
        stochastic_model: "Bit flips as Ising model / Boltzmann machine",
        use_case: "Stochastic resonance, Bayesian inference, annealing",
        warning: "Can corrupt data - requires isolated rows",
    }
}

// ---------------------------------------------------------------------------
// Combined RAM architectures (RAM_ARCHITECTURES dict, insertion order).
// ---------------------------------------------------------------------------
/// Registry keys in the same order as the python RAM_ARCHITECTURES dict.
pub const RAM_ARCHITECTURE_KEYS: [&str; 6] = [
    "fpga",
    "neural",
    "stochastic",
    "row_buffer",
    "bulk_k3",
    "rowhammer",
];

#[derive(Debug, Clone, PartialEq)]
pub struct FpgaLite {
    pub name: &'static str,
    pub kind: &'static str,
    /// cfg['total_cells'] // 64  (128 banks -> 131,072 LUTs)
    pub luts: u64,
    pub lut_size: u64,
    pub banks_used: u64,
    pub reconfiguration_time_ns: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NeuralRam {
    pub name: &'static str,
    pub kind: &'static str,
    /// int(cfg['trits'] / 1024) — python int() truncation == f64 as-cast.
    pub neurons: u64,
    /// int(cfg['trits'])
    pub synapses: u64,
    pub precision: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StochasticRam {
    pub name: &'static str,
    pub kind: &'static str,
    /// banks * row_buffer_bytes * 8 // 2 == 4,194,304
    pub nodes: u64,
    pub coupling: &'static str,
    pub temperature: &'static str,
}

/// Union of the six architecture configs (python dict values have different
/// shapes per key — kept as variants of one enum for dispatch).
#[derive(Debug, Clone, PartialEq)]
pub enum RamArchitectureConfig {
    RowBuffer(RowBufferPattern),
    BulkK3(BulkK3Fabric),
    Rowhammer(RowhammerPattern),
    Fpga(FpgaLite),
    Neural(NeuralRam),
    Stochastic(StochasticRam),
}

impl RamArchitectureConfig {
    /// python config['name'] for every entry.
    pub fn name(&self) -> &'static str {
        match self {
            RamArchitectureConfig::RowBuffer(c) => c.name,
            RamArchitectureConfig::BulkK3(c) => c.name,
            RamArchitectureConfig::Rowhammer(c) => c.name,
            RamArchitectureConfig::Fpga(c) => c.name,
            RamArchitectureConfig::Neural(c) => c.name,
            RamArchitectureConfig::Stochastic(c) => c.name,
        }
    }
}

/// python ram_architecture_fpga(): row_buffer(128) -> LUT fabric.
pub fn ram_architecture_fpga() -> FpgaLite {
    let cfg = pathway_row_buffer(128);
    FpgaLite {
        name: "RAM FPGA-Lite",
        kind: "Row Buffer",
        luts: cfg.total_cells / 64,
        lut_size: 6,
        banks_used: 128,
        reconfiguration_time_ns: DRAM_ROW_CYCLE_NS,
    }
}

/// python ram_architecture_neural(): bulk_k3() scaled to neurons/synapses.
pub fn ram_architecture_neural() -> NeuralRam {
    let cfg = pathway_bulk_k3();
    NeuralRam {
        name: "RAM Neural Network",
        kind: "Bulk K3",
        neurons: (cfg.trits / 1024.0) as u64,
        synapses: cfg.trits as u64,
        precision: "Ternary (-1, 0, +1)",
    }
}

/// python ram_architecture_stochastic().
pub fn ram_architecture_stochastic() -> StochasticRam {
    StochasticRam {
        name: "RAM Boltzmann Machine",
        kind: "RowHammer",
        nodes: DRAM_BANKS * DRAM_ROW_BUFFER_BYTES * 8 / 2,
        coupling: "Adjacent row interference",
        temperature: "Controlled via activation count",
    }
}

/// Dispatch equivalent of RAM_ARCHITECTURES[name]() — python maps each key to
/// a zero-arg builder call; row_buffer uses the 128-bank default.
pub fn ram_architecture(name: &str) -> Option<RamArchitectureConfig> {
    match name {
        "fpga" => Some(RamArchitectureConfig::Fpga(ram_architecture_fpga())),
        "neural" => Some(RamArchitectureConfig::Neural(ram_architecture_neural())),
        "stochastic" => Some(RamArchitectureConfig::Stochastic(ram_architecture_stochastic())),
        "row_buffer" => Some(RamArchitectureConfig::RowBuffer(pathway_row_buffer(128))),
        "bulk_k3" => Some(RamArchitectureConfig::BulkK3(pathway_bulk_k3())),
        "rowhammer" => Some(RamArchitectureConfig::Rowhammer(pathway_rowhammer())),
        _ => None,
    }
}

/// python: {'error': f'Unknown: {name}', 'available': list(keys)}
pub fn unknown_architecture_error(name: &str) -> String {
    format!(
        "Unknown: {}; available: {}",
        name,
        RAM_ARCHITECTURE_KEYS.join(", ")
    )
}

/// python DRAMPatternComputer.load_architecture(name) envelope. The python
/// 'bandwidth' and 'cells_available' fields are f-strings of spec constants;
/// both raw values and formatted helpers are exposed here.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedArchitecture {
    pub architecture: &'static str,
    pub config: RamArchitectureConfig,
    pub refresh_required: &'static str, // "Yes — 7.8μs interval"
    pub bandwidth_gbps: f64,            // specs["bandwidth_gbps"]
    /// specs["cells"] / 1e9 (python f'{...:.1f}B' -> 274.9)
    pub cells_available_b: f64,
}

impl LoadedArchitecture {
    /// python: f'{self.specs["bandwidth_gbps"]} GB/s' -> "51.2 GB/s"
    pub fn bandwidth_text(&self) -> String {
        format!("{} GB/s", self.bandwidth_gbps)
    }
    /// python: f'{cells/1e9:.1f}B' -> "274.9B"
    pub fn cells_available_text(&self) -> String {
        format!("{:.1}B", self.cells_available_b)
    }
}

/// Deterministic load_architecture: Err(message) for unknown names (the python
/// error dict also carried 'available', exposed via RAM_ARCHITECTURE_KEYS).
pub fn load_architecture(name: &str) -> Result<LoadedArchitecture, String> {
    match ram_architecture(name) {
        Some(config) => {
            let architecture = config.name();
            Ok(LoadedArchitecture {
                architecture,
                config,
                refresh_required: "Yes — 7.8μs interval",
                bandwidth_gbps: DRAM_BANDWIDTH_GBPS,
                cells_available_b: DRAM_CELLS as f64 / 1e9,
            })
        }
        None => Err(unknown_architecture_error(name)),
    }
}

// ---------------------------------------------------------------------------
// DRAMPatternComputer.summary() (deterministic text-free numeric form).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct DramSummary {
    /// python: f'{capacity_gb}GB DDR4-3200' -> "32GB DDR4-3200"
    pub ram_label: String,
    /// python: f'{cells/1e9:.1f}B DRAM cells' -> "274.9B DRAM cells"
    pub cells_label: String,
    pub banks: u64,
    /// python: f'{bandwidth_gbps} GB/s' -> "51.2 GB/s"
    pub bandwidth_label: String,
    /// python: f'{refresh_interval_us}μs' -> "7.8μs"
    pub refresh_label: String,
    pub pathways: u64,
    pub architectures: Vec<&'static str>,
}

pub fn dram_summary() -> DramSummary {
    DramSummary {
        ram_label: format!("{}GB {}", DRAM_CAPACITY_GB, DRAM_TYPE),
        cells_label: format!("{:.1}B DRAM cells", DRAM_CELLS as f64 / 1e9),
        banks: DRAM_BANKS,
        bandwidth_label: format!("{} GB/s", DRAM_BANDWIDTH_GBPS),
        refresh_label: format!("{}μs", DRAM_REFRESH_INTERVAL_US),
        pathways: 3,
        architectures: RAM_ARCHITECTURE_KEYS.to_vec(),
    }
}

// ---------------------------------------------------------------------------
// Full system comparison (1:1 full_system_summary dict — all string constants).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub struct FabricRegion {
    pub size: &'static str,
    pub cells: &'static str,
    pub kind: &'static str,
    pub architectures: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FullSystemSummary {
    pub cpu_l3: FabricRegion,
    pub gpu_ic: FabricRegion,
    pub ram: FabricRegion,
    pub total_cells_label: &'static str,
    pub total_sram_label: &'static str,
    pub total_dram_label: &'static str,
    pub total_architectures: u64,
}

pub fn full_system_summary() -> FullSystemSummary {
    FullSystemSummary {
        cpu_l3: FabricRegion {
            size: "32MB",
            cells: "268M",
            kind: "SRAM",
            architectures: 5,
        },
        gpu_ic: FabricRegion {
            size: "96MB",
            cells: "805M",
            kind: "SRAM",
            architectures: 4,
        },
        ram: FabricRegion {
            size: "32GB",
            cells: "274.9B",
            kind: "DRAM",
            architectures: 6,
        },
        total_cells_label: "276.0B cells total",
        total_sram_label: "128MB (1.07B cells, fast, no refresh)",
        total_dram_label: "32GB (274.9B cells, 256× more, needs refresh)",
        total_architectures: 15,
    }
}

/// Convenience aliases retained from the stub skeleton names (now with real
/// typed values): DRAM_SPECS-style access via dram_specs(), and the
/// RAM_ARCHITECTURES key list via RAM_ARCHITECTURE_KEYS.
pub fn dram_total_cells() -> u64 {
    DRAM_CELLS
}
