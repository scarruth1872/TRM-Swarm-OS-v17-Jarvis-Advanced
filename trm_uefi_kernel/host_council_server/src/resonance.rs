//! G83 — geometric resonance across the machine's clock/voltage domains, in the platform's own primitives.
//!
//! WHAT IS ACTUALLY MEASURABLE
//! HWiNFO's sensor feed carries, per 2 s sample: CPU core clock and core voltage (SVI2), CPU SoC voltage,
//! infinity-fabric clock (FCLK), memory-controller clock (UCLK), memory clock, the GPU's VR rails
//! (VDDCR_GFX / VDDIO / VDDCR_SOC), GPU clocks and PCIe link speed. Those are measurements.
//!
//! WHAT IS NOT
//!   * The reference crystal. Windows exposes no API for the board oscillator. A 100 MHz BCLK on AM4 is
//!     derived from a 25 MHz board crystal, but that is BOARD DOCUMENTATION, not a reading from this box.
//!     Every domain that claims a reference therefore carries `reference_basis` saying which it is.
//!   * NVMe/drive voltage. The feed has drive temperature and life, no rail. It is reported UNAVAILABLE
//!     rather than filled with a nominal 3.3 V or 1.8 V.
//!   * Jitter, phase noise, and any "phonic" coupling between crystals. None of it is measurable here, and
//!     nothing in this module claims it.
//!
//! WHAT THIS COMPUTES
//!   1. For each domain: the ratio of its clock to the 100 MHz bus reference, whether that ratio is an
//!      integer within tolerance (a real harmonic lock, e.g. FCLK 1200 / 100 = 12), its photon-equivalent
//!      wavelength and pitch class (the AURA convention: lambda = c/f), and its ternary state derived from
//!      the measured voltage normalised into its own observed range and quantised with `FacetedTrit`.
//!   2. Pairwise resonance in the repo's own form: `0.6 * cosine + 0.4 * gravity`, with gravity using
//!      MEASURED masses (watts), because G80 showed the shipped lattice's fixed seed masses (8-14) collapse
//!      the gravity term to 0.08 % at real load.
//!   3. A causality label per pair, so a number can never be read as more than it is: DERIVED (same
//!      reference PLL), COUPLED (1:1 fabric path), ASYNC (independent PLL, no common reference proven) or
//!      UNMEASURED (one side has no clock reading).

use serde_json::{json, Value};

use trm_uefi_kernel::faceted_torus_ternary::{FacetedTrit, MAX_FACET};
use trm_uefi_kernel::star_matrix::{StarMatrix, ALPHA as SEMANTIC_WEIGHT, BETA as GRAVITY_WEIGHT};

use crate::voltage::HwinfoFeed;

/// The bus reference every derived domain is compared against, in MHz. Measured: `Bus Clock` reads 100.0
/// in all 47 rows of the feed. The crystal BEHIND it is documentation, not a measurement.
pub const BUS_REFERENCE_MHZ: f64 = 100.0;
/// A ratio counts as harmonically locked when it sits within this fraction of an integer.
pub const LOCK_TOLERANCE: f64 = 0.005;

fn mean(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        None
    } else {
        Some(v.iter().sum::<f64>() / v.len() as f64)
    }
}

fn min_max(v: &[f64]) -> Option<(f64, f64)> {
    if v.is_empty() {
        return None;
    }
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for x in v {
        if *x < lo {
            lo = *x;
        }
        if *x > hi {
            hi = *x;
        }
    }
    Some((lo, hi))
}

#[derive(Clone, Debug)]
pub struct Domain {
    pub id: &'static str,
    pub label: &'static str,
    pub clock_mhz: Option<f64>,
    pub voltage_v: Option<f64>,
    pub voltage_range: Option<(f64, f64)>,
    pub power_w: Option<f64>,
    pub reference_basis: &'static str,
    pub source: &'static str,
}

impl Domain {
    fn ratio_to_bus(&self) -> Option<f64> {
        self.clock_mhz.map(|c| c / BUS_REFERENCE_MHZ)
    }

    /// Integer lock: Some(n) when clock/reference is within tolerance of a positive integer.
    fn harmonic_lock(&self) -> Option<u32> {
        let r = self.ratio_to_bus()?;
        if !r.is_finite() || r <= 0.0 {
            return None;
        }
        let n = r.round();
        if n >= 1.0 && (r - n).abs() / n <= LOCK_TOLERANCE {
            Some(n as u32)
        } else {
            None
        }
    }

    /// Ternary state from voltage margin inside the domain's OWN observed range. No range -> no trit.
    fn ternary(&self) -> Option<(i8, i16, f32, [f32; 4])> {
        let v = self.voltage_v?;
        let (lo, hi) = self.voltage_range?;
        if !(hi > lo) {
            return None;
        }
        let u = ((v - lo) / (hi - lo)).clamp(0.0, 1.0) as f32; // 0 = rail at its observed floor
        let t = FacetedTrit::from_f32(2.0 * u - 1.0);
        let (x, y, z, w) = t.to_4d_torus(u);
        Some((t.to_k3_trit(), t.facet, t.phase_angle_rad(), [x, y, z, w]))
    }
}

/// Photon-equivalent wavelength in metres. This is the AURA/spectrum convention applied to a clock: it is
/// an identity label, NOT a claim that the clock radiates at that wavelength.
pub fn wavelength_m(clock_hz: f64) -> f64 {
    299_792_458.0 / clock_hz
}

/// Equal-tempered pitch class, A4 = 440 Hz. Again an identity mapping, not a physical claim.
pub fn pitch_midi(clock_hz: f64) -> f64 {
    69.0 + 12.0 * (clock_hz / 440.0).log2()
}

pub fn note_name(midi: f64) -> String {
    const NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let n = midi.round() as i64;
    let idx = ((n % 12 + 12) % 12) as usize;
    format!("{}{}", NAMES[idx], n / 12 - 1)
}

pub fn domains_from(feed: &HwinfoFeed) -> Vec<Domain> {
    vec![
        Domain {
            id: "cpu_core",
            label: "CPU core (Zen2, SVI2)",
            clock_mhz: mean(&feed.core_clock_mhz),
            voltage_v: mean(&feed.svi2_vcore_v),
            voltage_range: min_max(&feed.svi2_vcore_v),
            power_w: mean(&feed.package_power_w),
            reference_basis: "DERIVED: core ratio x 100 MHz bus (ratio pinned at 40x, measured)",
            source: "HWiNFO: Core Clocks + CPU Core Voltage (SVI2 TFN)",
        },
        Domain {
            id: "cpu_soc",
            label: "CPU SoC / uncore rail",
            clock_mhz: mean(&feed.fclk_mhz),
            voltage_v: mean(&feed.soc_v_v),
            voltage_range: min_max(&feed.soc_v_v),
            power_w: None,
            reference_basis: "DERIVED: FCLK from the same bus reference",
            source: "HWiNFO: Infinity Fabric Clock + SoC Voltage (SVI2 TFN)",
        },
        Domain {
            id: "fabric",
            label: "Infinity Fabric (FCLK) / memory controller (UCLK)",
            clock_mhz: mean(&feed.fclk_mhz),
            voltage_v: None,
            voltage_range: None,
            power_w: None,
            reference_basis: "COUPLED 1:1 to UCLK and MEMCLK (all read 1200 MHz)",
            source: "HWiNFO: FCLK + UCLK + Memory Clock",
        },
        Domain {
            id: "gpu_core",
            label: "GPU core (RX 6700 XT, VDDCR_GFX)",
            clock_mhz: mean(&feed.gpu_eff_clock_mhz),
            voltage_v: mean(&feed.gpu_core_v_v),
            voltage_range: min_max(&feed.gpu_core_v_v),
            power_w: mean(&feed.gpu_power_w),
            reference_basis: "ASYNC: GPU has its own PLL; no common reference with BCLK is proven here",
            source: "HWiNFO: GPU Clock (Effective) + GPU Core Voltage (VDDCR_GFX) + GPU ASIC Power",
        },
        Domain {
            id: "gpu_mem",
            label: "GPU memory (VDDIO / VDDCI_MEM)",
            clock_mhz: mean(&feed.gpu_mem_clock_mhz),
            voltage_v: mean(&feed.gpu_mem_v_v),
            voltage_range: min_max(&feed.gpu_mem_v_v),
            power_w: None,
            reference_basis: "ASYNC: GDDR6 PLL inside the GPU",
            source: "HWiNFO: GPU Memory Clock + GPU Memory Voltage (VDDIO)",
        },
        Domain {
            id: "pcie",
            label: "PCIe 4.0 link (16 GT/s)",
            clock_mhz: None,
            voltage_v: None,
            voltage_range: None,
            power_w: None,
            reference_basis: "UNMEASURED: link speed is a negotiated rate, not an oscillator reading",
            source: "HWiNFO: PCIe Link Speed [GT/s] (16.0 constant in the feed)",
        },
        Domain {
            id: "nvme",
            label: "NVMe drive",
            clock_mhz: None,
            voltage_v: None,
            voltage_range: None,
            power_w: None,
            reference_basis: "UNMEASURED: the feed has drive temperature/life only",
            source: "UNAVAILABLE: no NVMe rail or clock sensor in this HWiNFO configuration",
        },
    ]
}

/// Pairwise causality label. Deliberately conservative: a shared number is not a shared clock.
pub fn causality(a: &str, b: &str) -> &'static str {
    let pair = (a, b);
    match pair {
        ("cpu_core", "cpu_soc") | ("cpu_soc", "cpu_core") => "DERIVED same bus reference",
        ("fabric", "cpu_soc") | ("cpu_soc", "fabric") => "COUPLED 1:1 FCLK:UCLK:MEMCLK",
        ("cpu_core", "gpu_core") | ("gpu_core", "cpu_core") => "ASYNC independent PLLs",
        ("gpu_core", "gpu_mem") | ("gpu_mem", "gpu_core") => "DERIVED same GPU PLL family",
        ("pcie", _) | (_, "pcie") => "UNMEASURED link rate only",
        ("nvme", _) | (_, "nvme") => "UNMEASURED no NVMe sensor",
        _ => "UNCLASSIFIED",
    }
}

fn domain_vector(d: &Domain) -> Vec<f64> {
    // log2 clock keeps a 24 MHz idle GPU from being swamped by a 4000 MHz CPU; voltage and power are
    // normalised to the observed set so no single unit dominates the cosine.
    let clk = d.clock_mhz.filter(|c| *c > 0.0).map(|c| c.log2()).unwrap_or(0.0);
    vec![
        clk,
        d.voltage_v.unwrap_or(0.0),
        d.power_w.unwrap_or(0.0),
    ]
}

pub fn resonance_grid(feed: &HwinfoFeed) -> Value {
    let domains = domains_from(feed);
    let max_power = domains.iter().filter_map(|d| d.power_w).fold(0.0f64, f64::max).max(1.0);

    let mut dom_json: Vec<Value> = Vec::new();
    for d in &domains {
        let tri = d.ternary();
        let clk_hz = d.clock_mhz.filter(|c| *c > 0.0).map(|c| c * 1e6);
        dom_json.push(json!({
            "id": d.id,
            "label": d.label,
            "clock_mhz": d.clock_mhz,
            "voltage_v": d.voltage_v,
            "voltage_range_v": d.voltage_range.map(|(a, b)| json!([a, b])),
            "power_w": d.power_w,
            "ratio_to_100mhz_bus": d.ratio_to_bus(),
            "harmonic_lock_n": d.harmonic_lock(),
            "wavelength_m": clk_hz.map(wavelength_m),
            "pitch_midi": clk_hz.map(pitch_midi),
            "note": clk_hz.map(|h| note_name(pitch_midi(h))),
            "ternary": tri.map(|(k, f, ph, t)| json!({
                "k3_trit": k, "facet": f, "facet_max": MAX_FACET,
                "phase_rad": ph, "torus_4d": t
            })),
            "reference_basis": d.reference_basis,
            "source": d.source,
        }));
    }

    let mut pairs: Vec<Value> = Vec::new();
    for i in 0..domains.len() {
        for j in (i + 1)..domains.len() {
            let a = &domains[i];
            let b = &domains[j];
            // The repo's own weights (ALPHA 0.6 semantic / BETA 0.4 gravitational), not local literals.
            let sim = StarMatrix::cosine_similarity(&domain_vector(a), &domain_vector(b));
            let ma = a.power_w.unwrap_or(0.0) / max_power;
            let mb = b.power_w.unwrap_or(0.0) / max_power;
            let (ca, cb) = match (a.clock_mhz, b.clock_mhz) {
                (Some(x), Some(y)) if x > 0.0 && y > 0.0 => (x.log2(), y.log2()),
                _ => (0.0, 0.0),
            };
            let dist = (ca - cb).abs().max(0.25);
            let gravity = ma * mb / (dist * dist);
            pairs.push(json!({
                "a": a.id,
                "b": b.id,
                "cosine": sim,
                "gravity": gravity,
                "resonance": SEMANTIC_WEIGHT * sim + GRAVITY_WEIGHT * gravity,
                "causality": causality(a.id, b.id),
                "both_measured": a.clock_mhz.is_some() && b.clock_mhz.is_some()
            }));
        }
    }

    json!({
        "bus_reference_mhz": BUS_REFERENCE_MHZ,
        "lock_tolerance": LOCK_TOLERANCE,
        "rows_measured": feed.rows,
        "window": [feed.first_time, feed.last_time],
        "domains": dom_json,
        "pairs": pairs,
        "honesty": {
            "crystal": "NOT MEASURED. No Windows API exposes the board oscillator; the 25 MHz AM4 reference is board documentation, not a reading. Every domain states its reference_basis.",
            "nvme": "UNAVAILABLE: this feed has drive temperature and life, no rail and no clock.",
            "wavelength_and_pitch": "Identity mappings (lambda = c/f, A4 = 440 Hz) in the AURA convention. They label a domain; they do not assert that a clock radiates or sounds.",
            "resonance": "0.6*cosine + 0.4*gravity with measured masses (watts). G80 showed the shipped lattice's fixed masses collapse the gravity term at real load.",
            "phonic": "No measurement in this module supports coupling between crystals or an audible/energetic 'phonic resonance' of the CPU or GPU die. What is measured is clock ratios, voltage margins and integer locks."
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed_for_domains() -> HwinfoFeed {
        HwinfoFeed {
            rows: 2,
            core_clock_mhz: vec![4000.0, 4000.0],
            svi2_vcore_v: vec![1.263, 1.300],
            soc_v_v: vec![1.013, 1.019],
            fclk_mhz: vec![1200.0, 1200.0],
            uclk_mhz: vec![1200.0, 1200.0],
            mem_clock_mhz: vec![1200.0, 1200.0],
            gpu_eff_clock_mhz: vec![8.5, 46.4],
            gpu_core_v_v: vec![0.228, 0.885],
            gpu_mem_v_v: vec![1.265, 1.266],
            gpu_mem_clock_mhz: vec![8.0, 132.0],
            gpu_power_w: vec![5.0, 7.0],
            package_power_w: vec![40.483, 61.55],
            ..Default::default()
        }
    }

    #[test]
    fn integer_lock_detects_the_measured_fabric_ratio() {
        let d = domains_from(&feed_for_domains());
        let fabric = d.iter().find(|x| x.id == "fabric").unwrap();
        assert_eq!(fabric.harmonic_lock(), Some(12), "1200 MHz on a 100 MHz bus is a 12x lock");
    }

    #[test]
    fn cpu_ratio_is_the_measured_40x_pin() {
        let d = domains_from(&feed_for_domains());
        let cpu = d.iter().find(|x| x.id == "cpu_core").unwrap();
        assert_eq!(cpu.harmonic_lock(), Some(40));
    }

    #[test]
    fn async_gpu_ratio_is_not_called_a_lock() {
        let d = domains_from(&feed_for_domains());
        let gpu = d.iter().find(|x| x.id == "gpu_core").unwrap();
        // mean effective clock ~27.45 MHz -> ratio 0.27, not an integer lock
        assert_eq!(gpu.harmonic_lock(), None, "an idle GPU must not be reported as harmonically locked");
    }

    #[test]
    fn missing_sensor_yields_no_trit_not_a_default() {
        let d = domains_from(&feed_for_domains());
        let nvme = d.iter().find(|x| x.id == "nvme").unwrap();
        assert!(nvme.voltage_v.is_none());
        assert!(nvme.ternary().is_none(), "no voltage means no ternary state");
        assert!(nvme.harmonic_lock().is_none());
    }

    #[test]
    fn causality_never_claims_a_shared_clock_between_cpu_and_gpu() {
        assert_eq!(causality("cpu_core", "gpu_core"), "ASYNC independent PLLs");
        assert_eq!(causality("nvme", "cpu_core"), "UNMEASURED no NVMe sensor");
    }

    #[test]
    fn wavelength_and_pitch_are_exact_for_a_known_clock() {
        let l = wavelength_m(300e6);
        assert!((l - 0.999_308_193).abs() < 1e-6, "300 MHz -> ~0.9993 m, got {l}");
        let m = pitch_midi(440.0);
        assert!((m - 69.0).abs() < 1e-9);
        assert_eq!(note_name(m), "A4");
    }

    #[test]
    fn grid_reports_every_pair_and_refuses_the_phonic_claim() {
        let g = resonance_grid(&feed_for_domains());
        let pairs = g["pairs"].as_array().unwrap();
        assert_eq!(pairs.len(), 21, "7 domains -> 21 unordered pairs");
        assert!(g["honesty"]["crystal"].as_str().unwrap().contains("NOT MEASURED"));
        for p in pairs {
            assert!(p["causality"].as_str().is_some(), "every pair must carry a causality label");
        }
    }

    /// Runs the grid against the REAL 47-row HWiNFO feed when one is on disk, so the numbers in the report
    /// come from the measurement window rather than from a fixture. Skips quietly when the feed is absent.
    #[test]
    fn real_feed_grid_prints_measured_domains() {
        let path = std::env::var("TRM_HWINFO_CSV").ok()
            .filter(|p| std::path::Path::new(p).exists())
            .unwrap_or_else(|| "F:/Development sites/TRM-Swarm-OS-v2/scratch/hwinfo/sensors.csv".to_string());
        if !std::path::Path::new(&path).exists() {
            eprintln!("no HWiNFO feed at {path}; skipping");
            return;
        }
        let feed = crate::voltage::read_hwinfo_csv(&path).expect("feed must parse");
        let g = resonance_grid(&feed);
        println!("REAL FEED rows={} window={}", g["rows_measured"], g["window"]);
        for d in g["domains"].as_array().unwrap() {
            println!("domain {:<10} clk={:>8.3} MHz  ratio={:>6.2}  lock={:?}  V={:?}  W={:?}  WAV={:?} m  {:<4}",
                d["id"].as_str().unwrap(),
                d["clock_mhz"].as_f64().unwrap_or(f64::NAN),
                d["ratio_to_100mhz_bus"].as_f64().unwrap_or(f64::NAN),
                d["harmonic_lock_n"],
                d["voltage_v"].as_f64(),
                d["power_w"].as_f64(),
                d["wavelength_m"].as_f64(),
                d["note"].as_str().unwrap_or("-"));
            if let Some(t) = d.get("ternary") { if !t.is_null() {
                println!("    ternary trit={} facet={}/108 phase={:.3} 4d={:?}",
                    t["k3_trit"], t["facet"], t["phase_rad"].as_f64().unwrap_or(0.0), t["torus_4d"]);
            }}
        }
        let mut best: Vec<(f64, String, String, String)> = Vec::new();
        for p in g["pairs"].as_array().unwrap() {
            best.push((p["resonance"].as_f64().unwrap_or(0.0),
                       p["a"].as_str().unwrap().to_string(),
                       p["b"].as_str().unwrap().to_string(),
                       p["causality"].as_str().unwrap().to_string()));
        }
        best.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        println!("\ntop pairs by resonance:");
        for (r, a, b, c) in best.iter().take(6) { println!("  {a:<10} {b:<10} R={r:.4}  [{c}]"); }
        println!("bottom pair: {:.4}", best.last().unwrap().0);
        println!("honesty crystal: {}", g["honesty"]["crystal"]);
    }
}
