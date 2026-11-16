//! TRM Swarm OS - multimodal_fusion (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/multimodal_fusion.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  MalfunctionEvent 
#[derive(Debug, Clone)]
pub struct MalfunctionEvent {
    pub event_id: String,
    pub event_type: String,
    pub severity: String,
    pub fusion_confidence: f64,
    pub telemetry_score: f64,
    pub visual_score: f64,
    pub root_cause_explanation: String,
    pub recommended_action: String,
    pub timestamp: f64,
}

impl MalfunctionEvent {
    pub fn new() -> Self {
        MalfunctionEvent {
            event_id: String::new(),
            event_type: String::new(),
            severity: String::new(),
            fusion_confidence: 0.0,
            telemetry_score: 0.0,
            visual_score: 0.0,
            root_cause_explanation: String::new(),
            recommended_action: String::new(),
            timestamp: 0.0,
        }
    }
}

//  get_multimodal_fusion() 
static SINGLETON_MULTIMODAL_FUSION: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_multimodal_fusion() -> u64 {
    // Singleton handle (lazy-init pattern from multimodal_fusion.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_MULTIMODAL_FUSION.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_MULTIMODAL_FUSION.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

// =========================================================================
// Fusion rules ported from swarm_v2/core/multimodal_fusion.py :: fuse_and_detect
// Late multimodal fusion (telemetry + visual) and the cross-modal malfunction
// correlation rules.
//
// Host-side in the oracle, therefore NOT ported here (listed, not faked):
//   * logging.info("[MultimodalFusion] Event Detected: ...")  -> no logger in kernel
//   * event_id = f"evt_{int(time.time() * 1000)}"  -> wall clock; caller supplies
//     it (portable format part is event_id_from_epoch_millis below)
//   * timestamp: float = Field(default_factory=time.time)  -> wall clock; the
//     field is left at 0.0 rather than invented
//   * get_multimodal_ingestion() / extract_telemetry_features / extract_visual_features
//     -> swarm_v2.core.multimodal_ingestion (separate module): the
//     FeatureVector.normalized_score and .features values are injected here
//   * telemetry_fabric.publish(TelemetryFrame(...)) (HBTF) -> IO/socket
//   * MultimodalFusionEngine.__init__ singleton wiring (`_fusion_engine`)
// =========================================================================

/// Portable part of the oracle's `event_id = f"evt_{int(time.time() * 1000)}"`:
/// the caller reads the clock, this builds the identifier.
pub fn event_id_from_epoch_millis(epoch_millis: u64) -> String {
    alloc::format!("evt_{}", epoch_millis)
}

/// CPython-compatible `round(x, 3)`.
///
/// CPython rounds the *exact* binary value to 3 decimal places with ties going
/// to even; Rust's `f64::round` rounds half away from zero and
/// `(x * 1000.0).round() / 1000.0` double-rounds, so neither matches.
/// This decomposes the IEEE-754 bits (x == mant * 2^exp, exact), scales by 1000
/// in exact integer arithmetic, and shifts with round-half-to-even.
pub fn round_half_even_3(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let bits = x.to_bits();
    let neg = (bits >> 63) & 1 == 1;
    let exp_bits = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & 0x000f_ffff_ffff_ffff;
    let (mant, exp) = if exp_bits == 0 {
        (frac, -1074i32) // subnormal
    } else {
        (frac | (1u64 << 52), exp_bits - 1075)
    };
    if mant == 0 {
        return x; // preserves -0.0 / 0.0
    }
    // x == mant * 2^exp exactly; want round_half_even(mant * 1000 * 2^exp)
    if exp >= 0 {
        return x; // already an integer
    }
    let k = (-exp) as u32;
    let num: u128 = (mant as u128) * 1000u128; // < 2^63, exact
    let q: u128;
    if k >= 128 {
        // num < 2^63 <= 2^(k-1): strictly below the halfway point -> 0
        q = 0;
    } else {
        let base = num >> k;
        let rem = num & ((1u128 << k) - 1u128);
        let half = 1u128 << (k - 1);
        q = if rem > half {
            base + 1
        } else if rem < half {
            base
        } else if base & 1 == 1 {
            base + 1 // exact tie -> round half to even
        } else {
            base
        };
    }
    let m = q as f64 / 1000.0;
    if neg {
        -m
    } else {
        m
    }
}

/// CPython `repr(float)` for the value ranges this module emits: Rust's `{}` is
/// also shortest-round-trip, but prints `45` where Python prints `45.0`.
/// Exponential notation (|v| < 1e-4 or >= 1e16) is NOT reproduced.
pub fn py_float_repr(v: f64) -> String {
    let mut s = alloc::format!("{}", v);
    if !s.contains('.') && !s.contains('e') && !s.contains('E') && !s.contains("inf") && !s.contains("NaN") {
        s.push_str(".0");
    }
    s
}

/// The fields the oracle reads off a `FeatureVector`
/// (fv_telemetry.normalized_score / fv_telemetry.features).
pub struct FeatureSnapshot<'a> {
    pub features: &'a [(&'a str, f64)],
    pub normalized_score: f64,
}

impl<'a> FeatureSnapshot<'a> {
    pub fn new(features: &'a [(&'a str, f64)], normalized_score: f64) -> Self {
        FeatureSnapshot { features, normalized_score }
    }
}

/// Python `dict.get(key, default)` over a feature map.
fn feature_get(features: &[(&str, f64)], key: &str, default: f64) -> f64 {
    for (k, v) in features.iter() {
        if *k == key {
            return *v;
        }
    }
    default
}

//  MultimodalFusionEngine 
pub struct MultimodalFusionEngine {
    pub event_history: Vec<MalfunctionEvent>,
}

impl MultimodalFusionEngine {
    pub fn new() -> Self {
        MultimodalFusionEngine { event_history: Vec::new() }
    }

    /// Late fusion + cross-modal malfunction rules.
    ///
    /// Python signature is `fuse_and_detect(self, telemetry, visual)`; the raw
    /// data objects and their feature extraction live in multimodal_ingestion,
    /// so the extracted snapshots are taken directly.
    pub fn fuse_and_detect(
        &mut self,
        fv_telemetry: &FeatureSnapshot,
        fv_visual: &FeatureSnapshot,
        event_id: &str,
    ) -> MalfunctionEvent {
        let t_score = fv_telemetry.normalized_score;
        let v_score = fv_visual.normalized_score;

        // Dual-Modality Fusion Formula: Fused_Score = (0.5 * Telemetry) + (0.5 * Visual)
        let fused_score = (0.5 * t_score) + (0.5 * v_score);

        // Cross-Modal Correlation Rules for Equipment Malfunction Detection
        let temp_raw = feature_get(fv_telemetry.features, "raw_temp_c", 45.0);
        let thermal_hotspot = feature_get(fv_visual.features, "thermal_hotspot", 0.0);
        let vibration = feature_get(fv_telemetry.features, "normalized_vibration", 0.0);
        let smoke = feature_get(fv_visual.features, "smoke_density", 0.0);

        let (event_type, severity, confidence, explanation, action): (&str, &str, f64, String, &str) =
            if temp_raw > 85.0 && thermal_hotspot > 0.7 {
                (
                    "THERMAL_RUNAWAY_CRITICAL",
                    "CRITICAL",
                    f64::min(0.98, fused_score + 0.2),
                    alloc::format!(
                        "Critical Overheating Correlated: Sensor temp={}°C matches Visual Thermal Hotspot={}.",
                        py_float_repr(temp_raw),
                        py_float_repr(thermal_hotspot)
                    ),
                    "EMERGENCY SHUTDOWN: Isolate power relay and engage secondary coolant loop.",
                )
            } else if vibration > 0.75 && thermal_hotspot > 0.5 {
                (
                    "BEARING_MECHANICAL_FAILURE",
                    "HIGH",
                    f64::min(0.92, fused_score + 0.15),
                    alloc::format!(
                        "Mechanical Bearing Degradation: High Vibration={} correlated with Visual Friction Heating={}.",
                        py_float_repr(vibration),
                        py_float_repr(thermal_hotspot)
                    ),
                    "SCHEDULE IMMEDIATE MAINTENANCE: Throttle spindle speed by 50%.",
                )
            } else if smoke > 0.6 {
                (
                    "ELECTRICAL_SMOKE_FIRE_HAZARD",
                    "CRITICAL",
                    0.95,
                    alloc::format!(
                        "Visual Smoke Cloud Detected: Density={} with elevated power draw.",
                        py_float_repr(smoke)
                    ),
                    "EVACUATE ZONE: Depower electrical bus and trigger N2 fire suppression.",
                )
            } else if fused_score > 0.5 {
                (
                    "ELEVATED_STRESS_WARNING",
                    "WARNING",
                    0.80,
                    alloc::format!(
                        "Combined Telemetry ({}) & Visual ({}) stress above nominal baseline.",
                        py_float_repr(t_score),
                        py_float_repr(v_score)
                    ),
                    "MONITOR: Increase telemetry sampling frequency to 100Hz.",
                )
            } else {
                (
                    "NOMINAL_SYSTEM_HEALTH",
                    "INFO",
                    0.99,
                    String::from("Both telemetry sensors and visual feeds report nominal baseline operation."),
                    "NO_ACTION_REQUIRED: System operating within optimal tolerances.",
                )
            };

        let event = MalfunctionEvent {
            event_id: String::from(event_id),
            event_type: String::from(event_type),
            severity: String::from(severity),
            fusion_confidence: round_half_even_3(confidence),
            telemetry_score: t_score,
            visual_score: v_score,
            root_cause_explanation: explanation,
            recommended_action: String::from(action),
            // Oracle: timestamp = Field(default_factory=time.time) -- wall clock
            // is host-side; left at 0.0 rather than given a plausible value.
            timestamp: 0.0,
        };

        self.event_history.push(event.clone());
        event
    }
}

impl Default for MultimodalFusionEngine {
    fn default() -> Self {
        Self::new()
    }
}
