//! hardware_binder.rs — bind a task to the measured host's silicon (the voltage doctrine).
//!
//! Port of `swarm_v2/core/transistor_task_scheduler.py`::`classify_task` + `bind_task_to_silicon`,
//! with the desktop identities already rectified to the measured host:
//!   AMD Ryzen 9 5900HX (8C/16T Zen 3 Cezanne) + NVIDIA RTX 3070 Laptop (Ampere GA104).
//!   Measured rails: CPU V_core 1.2V, DRAM 1.2V ConfiguredVoltage. GPU/SoC/NVMe rails: not measured.
//!   phi_v = 1.0 (machine-derived at nominal). energy_uj = 0.0 (0.084 was a fabricated target).

use serde_json::json;

/// The 15 SRAM pattern names (transistor_task_scheduler._init_hardware_topology, 1-indexed).
pub const SRAM_PATTERNS: [&str; 15] = [
    "FPGA-Lite (Reconfigurable Logic & K3 Truth Tables)",
    "Cellular Automaton Grid (Self-Healing & Defense Dynamics)",
    "SRAM Neural Network (Direct In-Cache Weight Inference)",
    "Content-Addressable Memory (O(1) Exact Key-Value Recall)",
    "Quantum Circuit Simulator (Superposition State Vectors)",
    "Ising Spin Glass (Native Spin Hamiltonian Optimization)",
    "Hopfield Attractor Network (Auto-Associative Denoising)",
    "Boltzmann Stochastic Resonance (Thermal Noise Inference)",
    "Graph Neural Network Accelerator (Relational Message Passing)",
    "Ternary Content-Addressable Memory (K3-Native Wildcards)",
    "Systolic Matrix Array (Rhythmic 2D Dataflow Matrix Multiply)",
    "Sparse Transformer Attention (93.75% Sparse Block Routing)",
    "Hyperdimensional Computing (10,000-dim HDC VSA Binding)",
    "Holographic Associative Memory (Distributed Optical Fourier Plates)",
    "Oscillatory Neural Network (Coupled Kuramoto Phase Synchrony)",
];

fn contains_any(text: &str, kws: &[&str]) -> bool {
    kws.iter().any(|k| text.contains(k))
}

/// The 15 agents' vibratory signatures: a DECLARED visible-band chromatic frequency per agent
/// (a designed spectral spread across 430-750 THz, not a measurement), its DERIVED wavelength
/// (lambda = c/f), and its color. The phonon unification is a RATIO to the 15.53 THz silicon
/// optical phonon — a quotient, not a "lock". The silicon optical phonon itself (15.53 THz) is
/// far-infrared (19.3 um) and carries no visible color, so color is a separate declared signature.
/// Council motion "agent vibratory signature" (PROCEED).
pub const AGENT_VIBRATORY: &[(&str, f64, &str)] = &[
    ("Archi",       750.0, "Violet"),
    ("LogicMaster", 730.0, "Violet-blue"),
    ("Shield",      710.0, "Violet-blue"),
    ("Logic",       690.0, "Blue"),
    ("Devo",        660.0, "Blue"),
    ("Seeker",      630.0, "Cyan"),
    ("Jarvis",      600.0, "Cyan-green"),
    ("Pulse",       570.0, "Green"),
    ("Flow",        540.0, "Green"),
    ("EventLedger", 510.0, "Yellow-green"),
    ("Orchestra",   500.0, "Yellow"),
    ("THzGateway",  480.0, "Orange"),
    ("Verify",      460.0, "Orange-red"),
    ("Bridge",      440.0, "Red"),
    ("Hermes",      430.0, "Red"),
];

pub const SPEED_OF_LIGHT_M_S: f64 = 299_792_458.0;

/// One agent's vibratory signature: declared frequency, derived wavelength, declared color, and
/// the phonon unification ratio to the 15.53 THz silicon optical phonon (Fd-3m diamond cubic).
pub fn vibratory_signature(agent: &str) -> serde_json::Value {
    let (freq_thz, color) = AGENT_VIBRATORY
        .iter()
        .find(|(n, _, _)| *n == agent)
        .map(|(_, f, c)| (*f, *c))
        .unwrap_or((0.0, ""));
    let wavelength_nm = if freq_thz > 0.0 {
        SPEED_OF_LIGHT_M_S / (freq_thz * 1e12) * 1e9
    } else {
        0.0
    };
    let phonon_ratio = if freq_thz > 0.0 { 15.53 / freq_thz } else { 0.0 };
    serde_json::json!({
        "agent": agent,
        "vibratory_frequency_thz": freq_thz,
        "frequency_basis": "declared_design (visible-band chromatic signature; not a measured resonance)",
        "wavelength_nm": wavelength_nm,
        "wavelength_basis": "derived: lambda = c/f",
        "color": color,
        "color_basis": "declared_design (visible-spectrum name; the 15.53 THz phonon is far-IR, 19.3 um, and carries no visible color)",
        "phonon_unification_ratio": phonon_ratio,
        "phonon_ratio_basis": "derived quotient 15.53 THz optical phonon / agent frequency — a RATIO, not a resonance lock",
        "silicon_geometric_resonance": {
            "space_group": "Fd-3m (#227)",
            "lattice_constant_a": 5.4307,
            "optical_phonon_thz": 15.53,
            "acoustic_phonon_thz": 4.52,
            "bond_angle_deg": 109.47,
        },
    })
}

/// The transistor scheduler's `classify_task` (task + role -> silicon execution class).
/// Returns LOGIC | GENERATIVE | STREAMING | DEFENSIVE | ARCHITECTURAL (default LOGIC).
pub fn classify_silicon_task(task: &str, role: &str) -> &'static str {
    let t = format!("{} {}", task, role).to_lowercase();
    if contains_any(&t, &["security", "shield", "vulnerability", "cve", "ddr", "antibody", "memory safe"]) {
        "DEFENSIVE"
    } else if contains_any(&t, &["logic", "proof", "theorem", "lean4", "k3", "verify", "qa", "test"]) {
        "LOGIC"
    } else if contains_any(&t, &["stream", "ingest", "fetch", "radar", "signal", "wifi", "telemetry", "pulse"]) {
        "STREAMING"
    } else if contains_any(
        &t,
        &[
            "generate", "code", "dev", "devo", "compile", "simd", "rust", "creative",
            // GPU work was falling through to LOGIC and binding to the CPU rail: the classifier
            // had no keyword for any of the workloads the kernel actually runs on the GPU.
            "gpu", "wgpu", "vulkan", "cuda", "docking", "vina", "screen", "deepdta",
            "molecular", "ligand", "inference",
        ],
    ) {
        "GENERATIVE"
    } else if contains_any(&t, &["arch", "blueprint", "plan", "topology", "dag", "mesh", "seeker"]) {
        "ARCHITECTURAL"
    } else {
        "LOGIC"
    }
}

/// The silicon binding for a task, mirroring `bind_task_to_silicon` with honest values.
/// The mapping (class -> rail/SRAM/die region) is a DETERMINISTIC MODEL, not a live probe:
/// rails are the measured values where they exist, "not measured" elsewhere; die coordinates
/// name the die (Zen 3 Cezanne / GA104 Ampere) without fabricating an X/Y.
pub fn bind_task_to_silicon(task: &str, role: &str) -> serde_json::Value {
    let class = classify_silicon_task(task, role);
    let (rail, volts, sram_id, region, die, transistors, t_basis) = match class {
        "GENERATIVE" => (
            "GPU V_core (not measured on host)",
            0.0,
            13,
            "Compute Unit SIMD Vector Lanes & L2 Cache Slice",
            "GA104 Ampere (RTX 3070 Laptop; exact coords not measured)",
            Some(435_000_000u64),
            "derived: GA104 17.4B transistors / 40 SMs (laptop) (~435M/SM)",
        ),
        "STREAMING" => (
            "DRAM VDDQ (1.2V ConfiguredVoltage measured)",
            1.2,
            4,
            "Unified Memory Controller (cUMC) & PHY Bank 0/1",
            "Cezanne I/O (exact coords not measured)",
            None,
            "not published: cUMC/PHY is a sub-block of the Cezanne I/O die (AMD publishes only the whole-die 10.7B)",
        ),
        "DEFENSIVE" => (
            "NVMe VCC (not measured on host)",
            0.0,
            1,
            "PCIe Root Complex & Hardware Cryptographic Engine",
            "Cezanne I/O Secure Enclave (exact coords not measured)",
            None,
            "not published: PCIe root complex / crypto engine is a sub-block of the Cezanne I/O die",
        ),
        "ARCHITECTURAL" => (
            "SoC V_core (not measured on host)",
            0.0,
            14,
            "Infinity Fabric Crossbar & L3 Slice",
            "Cezanne Central Fabric Interconnect (exact coords not measured)",
            None,
            "not published: Infinity Fabric crossbar / L3 slice is a sub-block of the Cezanne die",
        ),
        _ => (
            "CPU V_core (1.2V measured)",
            1.2,
            10,
            "CCX0 Core Complex — ALU/Branch Prediction Unit",
            "Zen 3 Cezanne (exact coords not measured)",
            Some(520_000_000u64),
            "derived: Vermeer Zen 3 CCD 4.15B transistors / 8 cores (~520M/core incl. L3 share); Cezanne is the monolithic 10.7B APU die",
        ),
    };
    // The unified fabric: the GPU is a measured functional co-processor on EVERY binding, not a
    // separate silicon silo. Its power/clock/temp/util are real nvidia-smi readings; the core
    // voltage is not exposed by nvidia-smi and stays honestly "not measured".
    let gpu = crate::host_probe::gpu_telemetry();
    let gpu_coprocessor = match gpu {
        Some((power, clock, mem_clock, temp, util)) => json!({
            "present": true,
            "model": "NVIDIA RTX 3070 Laptop (GA104 Ampere)",
            "power_w": power,
            "clock_mhz": clock,
            "mem_clock_mhz": mem_clock,
            "temp_c": temp,
            "util_pct": util,
            "voltage": "not exposed by nvidia-smi (power/clock/temp/util are measured)",
        }),
        None => json!({
            "present": false,
            "basis": "nvidia-smi not present on this host (iGPU answers to wmic, not nvidia-smi)",
        }),
    };
    json!({
        "task_class": class,
        "voltage_rail": rail,
        "nominal_voltage": volts,
        "sram_pattern_id": sram_id,
        "sram_pattern_name": SRAM_PATTERNS[(sram_id - 1) as usize],
        "transistor_region": region,
        "die_coordinates": die,
        "transistor_count": transistors,
        "transistor_count_basis": t_basis,
        "phi_v": 1.0,                 // machine-derived at nominal
        "energy_uj": 0.0,             // not measured (0.084 was a fabricated target)
        "phonon_thz": 15.53,          // real silicon Fd-3m optical-phonon constant
        "gpu_coprocessor": gpu_coprocessor,
        "vibratory_signature": vibratory_signature(role),
        "binding_type": "DETERMINISTIC_MODEL (rail measured, die coords not measured; GPU telemetry measured)",
    })
}
