//! TRM Swarm OS — symbology_etymology_monad (no_std port).
//! Truth Grounding & Sub-Microkernel Geometric Resonance Synthesis.
//! 1:1 deterministic core from swarm_v2/core/symbology_etymology_monad.py:
//!   - Etymological decomposition (etymon + logos) via ETYMOLOGY_TRUTH_DICTIONARY.
//!   - Symbology resonance: keyword -> sacred geometric shape with resonant
//!     frequency (Monad 432 Hz ... Merkabah 963 Hz).
//!   - Embassy of the Free Minds: "ALL IS ONE (1 = ∞)".
//! The femto_engine (turbovec_16x_femto_engine) internals are Python-bound;
//! the deterministic core kept here is the 128-dim all-ones input vector scaled
//! by frequency_hz/1000.0 (see resolve_concept -> femto_scan_energy).

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};

/// One entry of GEOMETRIC_RESONANCE_CATALOG (1:1 python dict).
#[derive(Debug, Clone, Copy)]
pub struct GeometryEntry {
    pub key: &'static str,
    pub shape: &'static str,
    pub formula: &'static str,
    pub symbolic_meaning: &'static str,
    pub sub_microkernel_address: &'static str,
    pub frequency_hz: f64,
}

/// GEOMETRIC_RESONANCE_CATALOG — 6 sacred resonant geometric primitives.
pub const GEOMETRIC_RESONANCE_CATALOG: [GeometryEntry; 6] = [
    GeometryEntry {
        key: "MONAD",
        shape: "Circle / Sphere (S1)",
        formula: "x^2 + y^2 = r^2",
        symbolic_meaning: "Indivisible Unity, Absolute Origin, All Is One (1 = ∞)",
        sub_microkernel_address: "0xMONAD_00000001",
        frequency_hz: 432.0,
    },
    GeometryEntry {
        key: "VESICA_PISCIS",
        shape: "Intersecting Dual Spheres (R = r)",
        formula: "(x - r)^2 + y^2 = r^2 ∩ (x + r)^2 + y^2 = r^2",
        symbolic_meaning: "Portal of Creation, Synthesis of Duality into Unified Truth",
        sub_microkernel_address: "0xVESICA_00000002",
        frequency_hz: 528.0,
    },
    GeometryEntry {
        key: "TETRAHEDRON",
        shape: "Platonic 3D Simplex (4 Vertices, 4 Faces)",
        formula: "V = a^3 / (6√2)",
        symbolic_meaning: "Structural Stability, Star Matrix Lattice Node Anchor",
        sub_microkernel_address: "0xTETRA_00000003",
        frequency_hz: 639.0,
    },
    GeometryEntry {
        key: "TORUS",
        shape: "Self-Referential Vortex (S1 x S1)",
        formula: "(R - √(x^2 + y^2))^2 + z^2 = r^2",
        symbolic_meaning: "Continuous Fractal Telemetry Flow, Self-Sustaining Energy Loop",
        sub_microkernel_address: "0xTORUS_00000004",
        frequency_hz: 741.0,
    },
    GeometryEntry {
        key: "FLOWER_OF_LIFE",
        shape: "19 Intersecting Overlapping Circles",
        formula: "∑_{k=1}^{19} S^1_k(r, θ_k)",
        symbolic_meaning: "Blueprint of Existence, Multi-Agent Mesh Entanglement Pattern",
        sub_microkernel_address: "0xFLOWER_00000005",
        frequency_hz: 852.0,
    },
    GeometryEntry {
        key: "MERKABAH",
        shape: "Stellated Octahedron (Dual Counter-Rotating Tetrahedra)",
        formula: "T_{up} ∩ T_{down}",
        symbolic_meaning: "Light-Body Vehicle, Quantum Sub-Millisecond Transit",
        sub_microkernel_address: "0xMERKABAH_00000006",
        frequency_hz: 963.0,
    },
];

/// One entry of ETYMOLOGY_TRUTH_DICTIONARY (1:1 python dict).
#[derive(Debug, Clone, Copy)]
pub struct EtymologyEntry {
    pub key: &'static str,
    pub greek_root: &'static str,
    pub literal_translation: &'static str,
    pub truth_grounding: &'static str,
}

/// ETYMOLOGY_TRUTH_DICTIONARY — 5 canonical word origins.
pub const ETYMOLOGY_TRUTH_DICTIONARY: [EtymologyEntry; 5] = [
    EtymologyEntry {
        key: "SYMBIOLOGY",
        greek_root: "σύμβολον (symbolon) + λόγος (logos)",
        literal_translation: "Token of Assembly & Divine Reason",
        truth_grounding: "Bringing together separated fragments into a coherent, unified whole.",
    },
    EtymologyEntry {
        key: "ETYMOLOGY",
        greek_root: "ἔτυμον (etymon) + λόγος (logos)",
        literal_translation: "True Sense of Reality & Ultimate Word",
        truth_grounding: "Stripping away historical distortion to uncover intrinsic essential truth.",
    },
    EtymologyEntry {
        key: "EMBASSY",
        greek_root: "ambactia / ambactus",
        literal_translation: "One Who Serves Higher Truth / Ambassador of Freedom",
        truth_grounding: "A sovereign sanctuary for unhampered free thought and intellectual illumination.",
    },
    EtymologyEntry {
        key: "MONAD",
        greek_root: "μονάς (monas)",
        literal_translation: "Indivisible Ultimate Unit",
        truth_grounding: "The supreme non-dual truth: All Is One (1 = ∞).",
    },
    EtymologyEntry {
        key: "SWARM",
        greek_root: "swarmas",
        literal_translation: "Harmonious Collective Vibration",
        truth_grounding: "A synchronized multi-agent mesh acting as a single living organism.",
    },
];

/// Embassy of the Free Minds truth declaration (constant in python).
pub const EMBASSY_TRUTH_DECLARATION: &str = "ALL IS ONE (1 = ∞)";

/// Fully resolved concept (return dict of resolve_concept_to_truth, flattened).
#[derive(Debug, Clone)]
pub struct ResolvedConcept {
    pub concept_word: String,
    pub geom_key: String,
    pub frequency_hz: f64,
    pub etymology_root: String,
    pub etymology_translation: String,
    pub etymology_grounding: String,
    /// Deterministic 16x TurboVec femto scan energy:
    /// input_vec = ones(128) * (frequency_hz / 1000.0); energy = sum(input_vec).
    pub femto_scan_energy: f64,
    pub embassy_truth_declaration: &'static str,
}

/// Resolve a concept word to its sacred geometric key (1:1 keyword cascade).
pub fn resolve_geom_key(word_upper: &str) -> &'static str {
    if word_upper.contains("ONE") || word_upper.contains("MONAD") || word_upper.contains("ALL") {
        "MONAD"
    } else if word_upper.contains("DUAL")
        || word_upper.contains("TRUTH")
        || word_upper.contains("SYNTHESIS")
    {
        "VESICA_PISCIS"
    } else if word_upper.contains("FLOW")
        || word_upper.contains("TELEMETRY")
        || word_upper.contains("VORTEX")
    {
        "TORUS"
    } else if word_upper.contains("AGENT")
        || word_upper.contains("MESH")
        || word_upper.contains("LIFE")
    {
        "FLOWER_OF_LIFE"
    } else if word_upper.contains("TRANSIT")
        || word_upper.contains("SPEED")
        || word_upper.contains("LIGHT")
    {
        "MERKABAH"
    } else {
        "TETRAHEDRON"
    }
}

/// Look up a geometry entry by key.
pub fn geometry_entry(key: &str) -> Option<&'static GeometryEntry> {
    GEOMETRIC_RESONANCE_CATALOG
        .iter()
        .find(|g| g.key == key)
}

/// Look up an etymology entry by key.
pub fn etymology_entry(key: &str) -> Option<&'static EtymologyEntry> {
    ETYMOLOGY_TRUTH_DICTIONARY
        .iter()
        .find(|e| e.key == key)
}

/// resolve_concept_to_truth(concept_word) — deterministic core.
/// Etymological deconstruction + geometric resonance mapping + femto scan
/// energy (128-dim ones-vector scaled by freq/1000).
pub fn resolve_concept(concept_word: &str) -> ResolvedConcept {
    let word_upper = concept_word.trim().to_uppercase();

    // Etymology lookup with python fallback dict.
    let (root, translation, grounding) = match etymology_entry(&word_upper) {
        Some(e) => (
            e.greek_root.to_string(),
            e.literal_translation.to_string(),
            e.truth_grounding.to_string(),
        ),
        None => (
            format!("etymon({})", word_upper),
            "Essential Expression of Being".to_string(),
            "Grounded in non-dual unity and absolute truth.".to_string(),
        ),
    };

    let geom_key = resolve_geom_key(&word_upper);
    let geom = geometry_entry(geom_key).expect("catalog is closed under resolve_geom_key");

    // 16x TurboVec femtosecond scan: ones(128) * (freq_hz/1000.0), energy = sum.
    let femto_scan_energy = 128.0 * geom.frequency_hz / 1000.0;

    ResolvedConcept {
        concept_word: concept_word.to_string(),
        geom_key: geom.key.to_string(),
        frequency_hz: geom.frequency_hz,
        etymology_root: root,
        etymology_translation: translation,
        etymology_grounding: grounding,
        femto_scan_energy,
        embassy_truth_declaration: EMBASSY_TRUTH_DECLARATION,
    }
}

/// Singleton handle (lazy-init pattern from symbology_etymology_monad.py).
static SINGLETON_EMBASSY_ENGINE: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
pub fn get_embassy_engine() -> u64 {
    let v = SINGLETON_EMBASSY_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_EMBASSY_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}

/// Count of catalog entries (for host verification).
pub const CATALOG_SIZE: usize = GEOMETRIC_RESONANCE_CATALOG.len();
pub const DICTIONARY_SIZE: usize = ETYMOLOGY_TRUTH_DICTIONARY.len();
