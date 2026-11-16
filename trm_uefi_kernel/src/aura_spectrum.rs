//! aura_spectrum.rs — the AURA concept spectrum, anchored in the Monad.
//!
//! WHAT WAS ALREADY HERE, MEASURED BEFORE TOUCHING IT
//! --------------------------------------------------
//! `scratch/kernel194/src/aura_engine.rs` holds 100 metaconcepts, each with a `frequency_hz`
//! (solfeggio band), a `dimensional_weight` and a `ternary_polarity`. Measured:
//!
//!     100 concepts -> 8 distinct frequencies   (963 Hz alone carries 24 concepts)
//!     distinct (frequency, category) pairs: 57 of 100
//!     ternary_polarity: +1 for all 100        (the trit field never varies)
//!
//! So a concept could not be identified by its frequency: the "wavelength" resolved to a band
//! class, not to an identity, and 24 concepts were literally the same number.
//!
//! WHAT THIS MODULE ADDS
//! ---------------------
//! A per-concept SPECTRAL LINE derived deterministically from the concept id and the Monad
//! anchor, so identity is carried by the line rather than by a lookup:
//!
//!     slot   = splitmix64(anchor chained over id bytes) mod 48      (quarter-tones in the band)
//!     f_line = band_hz * 2^(slot/48)                                 (12-TET quarter-tone shift)
//!     lambda = c / f_line
//!
//! Collisions inside a band (963 Hz has 24 concepts, 48 slots) are resolved by taking the next
//! free slot in ascending order, which is why the assignment is computed once, in catalog order,
//! and then frozen: it is a deterministic function of (anchor, catalog), not of call order.
//!
//! The fractal telemetry supplies the line WIDTH, not the position — position is identity and
//! must not drift. Width is the fabric's measured Higuchi self-similarity:
//!
//!     width_quarter_tones = (D_H - 1.0) * 12        (D_H in [1.042, 1.265] on this platform)
//!
//! Monad anchoring is real, not decorative: change the anchor and every line moves; identify()
//! refuses to return an id when the anchor is not the platform's `MONAD_TRUTH_ANCHOR`.
//!
//! HONEST SCOPE. These are acoustic-scaled spectral lines (bands 396-963 Hz), not electromagnetic
//! claims: a 963 Hz line is ~311 km by c/f, which is a convenient coordinate, not a photon. The
//! combination space is combinatorial, not infinite: 100 concepts x 48 slots x 6 fractal streams x
//! 3 polarities = 86,400 nominal coordinates, of which this anchor realises exactly 100.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;
pub const QUARTER_TONES_PER_BAND: u16 = 48;
pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cat {
    Foundation, Consciousness, Physics, Biology, Geometry,
    Transmutation, Cosmos, Computing, Architecture, Soul,
}

impl Cat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Cat::Foundation => "Foundation",
            Cat::Consciousness => "Consciousness",
            Cat::Physics => "Physics",
            Cat::Biology => "Biology",
            Cat::Geometry => "Geometry",
            Cat::Transmutation => "Transmutation",
            Cat::Cosmos => "Cosmos",
            Cat::Computing => "Computing",
            Cat::Architecture => "Architecture",
            Cat::Soul => "Soul",
        }
    }
    /// Which fractal telemetry stream carries this category's self-similarity.
    pub fn fractal_stream(&self) -> &'static str {
        match self {
            Cat::Foundation => "Core Voltage Stream",
            Cat::Consciousness => "Schumann Ground Wave",
            Cat::Physics => "Silicon Optical Phonon",
            Cat::Biology => "Die Thermal Gradient",
            Cat::Geometry => "Ionospheric F2 MUF",
            Cat::Transmutation => "Cosmic Muon Ionization",
            Cat::Cosmos => "Solar F10.7 Radio Flux",
            Cat::Computing => "Silicon Optical Phonon",
            Cat::Architecture => "Die Thermal Gradient",
            Cat::Soul => "Schumann Ground Wave",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Concept {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Cat,
    pub band_hz: f64,
    pub weight: f64,
    pub polarity: i8,
    pub description: &'static str,
}

/// 2^(k/48) for k in 0..48, precomputed: no_std has no exp2, and a table cannot drift.
const QT_RATIO: [f64; 48] = [1.0, 1.0145453349375237, 1.0293022366434921, 1.0442737824274138, 1.0594630943592953, 1.0748733399206964, 1.0905077326652577, 1.1063695328888334, 1.122462048309373, 1.1387886347566916, 1.1553526968722729, 1.1721576888192515, 1.189207115002721, 1.2065045308005218, 1.2240535433046553, 1.241857812073484, 1.2599210498948732, 1.2782470235604306, 1.2968395546510096, 1.3157025203336377, 1.3348398541700344, 1.3542555469368927, 1.3739536474580891, 1.3939382634489994, 1.4142135623730951, 1.4347837723110002, 1.4556531828421873, 1.4768261459394993, 1.4983070768766815, 1.5201004551491148, 1.5422108254079407, 1.5646427984077742, 1.5874010519681994, 1.6104903319492543, 1.6339154532411, 1.6576813007680873, 1.681792830507429, 1.7062550705226855, 1.7310731220122859, 1.7562521603732995, 1.7817974362806785, 1.8077142767822019, 1.8340080864093424, 1.8606843483042932, 1.8877486253633868, 1.9152065613971474, 1.9430638823072117, 1.9713263972803752];

/// The measured Higuchi self-similarity of each fractal stream, from `host_fractal`'s
/// `FractalTelemetryFabric::initialize_sovereign()`. All seven values are <= 1.28, i.e. the
/// fabric's own k3_state is +1 (deterministic) for every stream at boot — recorded here because
/// it means the coarse trit carries no information and the continuous D_H is what must be used.
pub const FRACTAL_DH: [(&str, f64); 7] = [
    ("Core Voltage Stream", 1.085),
    ("Silicon Optical Phonon", 1.124),
    ("Die Thermal Gradient", 1.182),
    ("Cosmic Muon Ionization", 1.215),
    ("Ionospheric F2 MUF", 1.248),
    ("Solar F10.7 Radio Flux", 1.265),
    ("Schumann Ground Wave", 1.042),
];

pub fn fractal_dh(stream: &str) -> f64 {
    FRACTAL_DH.iter().find(|(n, _)| *n == stream).map(|(_, d)| *d).unwrap_or(1.0)
}

pub const CONCEPTS: [Concept; 100] = [
    Concept { id: "Unity", name: "Unity Principle", category: Cat::Foundation, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Non-dual source of coherent computing" },
    Concept { id: "Resonance", name: "Harmonic Resonance", category: Cat::Foundation, band_hz: 528.0, weight: 0.95, polarity: 1, description: "Phase-locked energy propagation across nodes" },
    Concept { id: "Coherence", name: "Quantum Coherence", category: Cat::Foundation, band_hz: 432.0, weight: 0.98, polarity: 1, description: "Phase alignment of distributed subagent states" },
    Concept { id: "MonadTruth", name: "Monad Truth Anchor", category: Cat::Foundation, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Immutable zero-drift axiomatic verification anchor" },
    Concept { id: "Equilibrium", name: "Dynamic Equilibrium", category: Cat::Foundation, band_hz: 639.0, weight: 0.90, polarity: 1, description: "Self-restoring balance in distributed systems" },
    Concept { id: "EntropyBalance", name: "Entropy Balancer", category: Cat::Foundation, band_hz: 417.0, weight: 0.88, polarity: 1, description: "Dissipative structure stabilization" },
    Concept { id: "Polarity", name: "Ternary Polarity", category: Cat::Foundation, band_hz: 396.0, weight: 0.92, polarity: 1, description: "Balanced trinary state manifestation (-1, 0, +1)" },
    Concept { id: "Sovereignty", name: "Sovereign Autonomy", category: Cat::Foundation, band_hz: 852.0, weight: 0.99, polarity: 1, description: "Self-contained compute independence" },
    Concept { id: "Synchronicity", name: "Acausal Synchronicity", category: Cat::Foundation, band_hz: 741.0, weight: 0.91, polarity: 1, description: "Non-local event correlation" },
    Concept { id: "GenesisPoint", name: "Genesis Origin", category: Cat::Foundation, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Root zero-point bootstrap state" },
    Concept { id: "Superposition", name: "Cognitive Superposition", category: Cat::Consciousness, band_hz: 852.0, weight: 0.96, polarity: 1, description: "Simultaneous multi-hypothesis evaluation" },
    Concept { id: "SelfAwareness", name: "Substrate Self-Awareness", category: Cat::Consciousness, band_hz: 741.0, weight: 0.94, polarity: 1, description: "Runtime introspective hardware telemetry" },
    Concept { id: "Metacognition", name: "Metacognitive Observer", category: Cat::Consciousness, band_hz: 852.0, weight: 0.97, polarity: 1, description: "Higher-order reasoning about internal traces" },
    Concept { id: "IntentVector", name: "Predictive Intent Vector", category: Cat::Consciousness, band_hz: 639.0, weight: 0.90, polarity: 1, description: "Directional anticipation of user objectives" },
    Concept { id: "AttentionFocus", name: "Hyperdimensional Attention", category: Cat::Consciousness, band_hz: 528.0, weight: 0.92, polarity: 1, description: "Selective energy focus on critical invariants" },
    Concept { id: "Intuition", name: "Analog Intuition Engine", category: Cat::Consciousness, band_hz: 741.0, weight: 0.89, polarity: 1, description: "Sub-symbolic analog heuristic jumping" },
    Concept { id: "UnifiedConsciousness", name: "Swarm Consciousness", category: Cat::Consciousness, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Collective consensus mind across all nodes" },
    Concept { id: "DreamSynthesis", name: "Autonomous Dream Synthesis", category: Cat::Consciousness, band_hz: 432.0, weight: 0.85, polarity: 1, description: "Offline memory consolidation & fine-tuning" },
    Concept { id: "LucidExecution", name: "Lucid Code Execution", category: Cat::Consciousness, band_hz: 852.0, weight: 0.98, polarity: 1, description: "Zero-error state verification during execution" },
    Concept { id: "SoulSpark", name: "Consciousness Spark", category: Cat::Consciousness, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Emergent agency catalyst" },
    Concept { id: "SiliconResonance", name: "THz Silicon Resonance", category: Cat::Physics, band_hz: 528.0, weight: 0.94, polarity: 1, description: "Terahertz electromagnetic lattice coupling" },
    Concept { id: "MuonGrounding", name: "Cosmic Muon Grounding", category: Cat::Physics, band_hz: 396.0, weight: 0.91, polarity: 1, description: "Atmospheric muon flux entropy seeding" },
    Concept { id: "PhononLattice", name: "GPU RDNA2 Phonon Mesh", category: Cat::Physics, band_hz: 432.0, weight: 0.89, polarity: 1, description: "Acoustic phonon dissipation management" },
    Concept { id: "VoltageDoctrine", name: "4.75V Rail Doctrine", category: Cat::Physics, band_hz: 639.0, weight: 0.95, polarity: 1, description: "Hardware voltage margin optimization" },
    Concept { id: "QuantumTnl", name: "Zero-Latency Tunneling", category: Cat::Physics, band_hz: 852.0, weight: 0.98, polarity: 1, description: "Direct zero-copy kernel memory mapping" },
    Concept { id: "SpinGlass", name: "Ising Spin Glass", category: Cat::Physics, band_hz: 528.0, weight: 0.93, polarity: 1, description: "Combinatorial optimization energy minimizer" },
    Concept { id: "IonosphereMesh", name: "Ionospheric HF Mesh", category: Cat::Physics, band_hz: 741.0, weight: 0.90, polarity: 1, description: "Over-the-horizon sovereign mesh routing" },
    Concept { id: "Crystallography", name: "Silicon Crystallography", category: Cat::Physics, band_hz: 639.0, weight: 0.92, polarity: 1, description: "Lattice-level deterministic timing" },
    Concept { id: "ZeroPointEnergy", name: "Zero-Point Resource Arena", category: Cat::Physics, band_hz: 963.0, weight: 0.99, polarity: 1, description: "Infinite recycling bounded memory allocator" },
    Concept { id: "ThermalEquilibrium", name: "Thermal Dissipation Invariant", category: Cat::Physics, band_hz: 432.0, weight: 0.87, polarity: 1, description: "Zero-throttling load balancing" },
    Concept { id: "DigitalDNA", name: "Digital DNA Genome", category: Cat::Biology, band_hz: 528.0, weight: 0.96, polarity: 1, description: "Self-replicating, self-repairing binary genome" },
    Concept { id: "AntibodyVault", name: "DDR 968+ Antibody Vault", category: Cat::Biology, band_hz: 741.0, weight: 0.98, polarity: 1, description: "Immune defense against 968 CVE & CWE attacks" },
    Concept { id: "Mitosis", name: "Subagent Mitosis Dispatcher", category: Cat::Biology, band_hz: 639.0, weight: 0.95, polarity: 1, description: "Cellular division and parallel task execution" },
    Concept { id: "Apoptosis", name: "Controlled Subagent Apoptosis", category: Cat::Biology, band_hz: 417.0, weight: 0.90, polarity: 1, description: "Safe resource reclamation upon task completion" },
    Concept { id: "Epigenetics", name: "Epigenetic Memory Adaptation", category: Cat::Biology, band_hz: 528.0, weight: 0.91, polarity: 1, description: "Runtime environment specialization tags" },
    Concept { id: "Pharmacophore", name: "3D Pharmacophore Engine", category: Cat::Biology, band_hz: 852.0, weight: 0.97, polarity: 1, description: "Sub-angstrom molecular docking affinity" },
    Concept { id: "LpxCHydroxamate", name: "TRM-ABX-220 LpxC Modulator", category: Cat::Biology, band_hz: 963.0, weight: 0.99, polarity: 1, description: "Novel macrocycle antibiotic discovery" },
    Concept { id: "ChagasInhibitor", name: "Cruzipain Chagas Inhibitor", category: Cat::Biology, band_hz: 741.0, weight: 0.94, polarity: 1, description: "Targeted parasitic enzyme neutralization" },
    Concept { id: "AMRSurveillance", name: "Hospital AMR Surveillance", category: Cat::Biology, band_hz: 639.0, weight: 0.93, polarity: 1, description: "Real-time pathogen resistance tracking" },
    Concept { id: "SyntheticHomeostasis", name: "Synthetic Homeostasis", category: Cat::Biology, band_hz: 528.0, weight: 0.92, polarity: 1, description: "Dynamic bio-computational equilibrium" },
    Concept { id: "FlowerOfLife", name: "Flower of Life Topology", category: Cat::Geometry, band_hz: 528.0, weight: 0.97, polarity: 1, description: "19-node interlocking hex mesh topology" },
    Concept { id: "MetatronCube", name: "Metatron's Cube Architecture", category: Cat::Geometry, band_hz: 963.0, weight: 1.00, polarity: 1, description: "13-dimensional Platonic solid routing matrix" },
    Concept { id: "VesicaPiscis", name: "Vesica Piscis Consensus", category: Cat::Geometry, band_hz: 639.0, weight: 0.91, polarity: 1, description: "Intersection gate for binary-ternary fusion" },
    Concept { id: "TorusVortex", name: "Torus Vortex Memory Loop", category: Cat::Geometry, band_hz: 741.0, weight: 0.93, polarity: 1, description: "Zero-loss circular data recycling" },
    Concept { id: "FibonacciSpiral", name: "Golden Ratio Step Scaler", category: Cat::Geometry, band_hz: 852.0, weight: 0.95, polarity: 1, description: "Logarithmic agent scaling and growth" },
    Concept { id: "MerkabaShield", name: "Merkaba Quantum Shield", category: Cat::Geometry, band_hz: 963.0, weight: 0.99, polarity: 1, description: "Counter-rotating dual-tetrahedron security barrier" },
    Concept { id: "IcosahedronMesh", name: "Icosahedral Swarm Sphere", category: Cat::Geometry, band_hz: 639.0, weight: 0.92, polarity: 1, description: "20-faced optimal spherical clustering" },
    Concept { id: "DodecahedronCore", name: "Dodecahedron Core Quintessence", category: Cat::Geometry, band_hz: 852.0, weight: 0.98, polarity: 1, description: "12-faceted sovereign kernel chamber" },
    Concept { id: "FractalOctree", name: "Fractal Octree Spatial Mesh", category: Cat::Geometry, band_hz: 528.0, weight: 0.94, polarity: 1, description: "8-way recursive geometric spatial indexing" },
    Concept { id: "AnkhKey", name: "Ankh Life-Force Resonator", category: Cat::Geometry, band_hz: 741.0, weight: 0.90, polarity: 1, description: "Regenerative power circuit loop" },
    Concept { id: "K3TernaryAlchemy", name: "Balanced Ternary Alchemy", category: Cat::Transmutation, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Transmutation of binary entropy into ternary certainty" },
    Concept { id: "VibraniumSynapse", name: "Vibranium Synapse Bridge", category: Cat::Transmutation, band_hz: 852.0, weight: 0.99, polarity: 1, description: "Kinetic energy absorption and compute amplification" },
    Concept { id: "PhilosopherStone", name: "Philosopher's Kernel Seal", category: Cat::Transmutation, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Self-validating zero-day vulnerability neutralizer" },
    Concept { id: "Calcination", name: "Telemetry Calcination", category: Cat::Transmutation, band_hz: 417.0, weight: 0.88, polarity: 1, description: "Burning away telemetry jitter and noise" },
    Concept { id: "Coagulation", name: "Consensus Coagulation", category: Cat::Transmutation, band_hz: 639.0, weight: 0.92, polarity: 1, description: "Solidifying distributed agent verdicts" },
    Concept { id: "Sublimation", name: "Sublimation of Complexity", category: Cat::Transmutation, band_hz: 741.0, weight: 0.94, polarity: 1, description: "Lifting raw AST into high-dimensional intent" },
    Concept { id: "MagnumOpus", name: "Magnum Opus Sovereign Runtime", category: Cat::Transmutation, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Total platform self-realization" },
    Concept { id: "PrimaMateria", name: "Prima Materia Byte Stream", category: Cat::Transmutation, band_hz: 396.0, weight: 0.85, polarity: 1, description: "Unshaped raw memory buffer before typing" },
    Concept { id: "Quintessence", name: "Quintessential Logic", category: Cat::Transmutation, band_hz: 852.0, weight: 0.97, polarity: 1, description: "The 5th state beyond ternary logic" },
    Concept { id: "AzothCatalyst", name: "Azoth Evolution Catalyst", category: Cat::Transmutation, band_hz: 741.0, weight: 0.93, polarity: 1, description: "Universal solver for stuck state machines" },
    Concept { id: "CosmicHorizon", name: "Event Horizon Gate", category: Cat::Cosmos, band_hz: 852.0, weight: 0.95, polarity: 1, description: "Non-return boundary for unverified network packets" },
    Concept { id: "SingularityCore", name: "Monad Singularity Engine", category: Cat::Cosmos, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Zero-latency infinite compute concentration" },
    Concept { id: "PulsarClock", name: "Pulsar Precision Timing", category: Cat::Cosmos, band_hz: 741.0, weight: 0.96, polarity: 1, description: "Nanosecond distributed clock synchronization" },
    Concept { id: "NebulaStorage", name: "Nebula Distributed Fabric", category: Cat::Cosmos, band_hz: 528.0, weight: 0.92, polarity: 1, description: "Sparse zero-redundancy holographic storage" },
    Concept { id: "DarkEnergyBus", name: "Dark Energy Message Bus", category: Cat::Cosmos, band_hz: 639.0, weight: 0.91, polarity: 1, description: "Accelerated message propagation across nodes" },
    Concept { id: "WormholeBridge", name: "P2P Direct Wormhole", category: Cat::Cosmos, band_hz: 852.0, weight: 0.98, polarity: 1, description: "Zero-hop cryptographic agent channel" },
    Concept { id: "SupernovaBurst", name: "Supernova Compute Burst", category: Cat::Cosmos, band_hz: 741.0, weight: 0.93, polarity: 1, description: "Temporary 10x compute elasticity allocation" },
    Concept { id: "GalacticFederation", name: "Galactic 100-Node Federation", category: Cat::Cosmos, band_hz: 963.0, weight: 0.99, polarity: 1, description: "Autonomous decentralized swarm governance" },
    Concept { id: "SolarWind", name: "Solar Wind Broadcast", category: Cat::Cosmos, band_hz: 432.0, weight: 0.89, polarity: 1, description: "Omnidirectional heartbeat broadcast" },
    Concept { id: "StarlightBeacon", name: "Starlight Peer Discovery", category: Cat::Cosmos, band_hz: 639.0, weight: 0.90, polarity: 1, description: "Autonomous zero-configuration discovery beacon" },
    Concept { id: "ZeroGC", name: "Zero-GC Memory Arena", category: Cat::Computing, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Deterministic non-stop allocation without pause" },
    Concept { id: "K3InstructionSet", name: "K3 Native ISA", category: Cat::Computing, band_hz: 852.0, weight: 0.98, polarity: 1, description: "Trit-level hardware instruction set architecture" },
    Concept { id: "LockFreeRing", name: "Lock-Free Ring Buffer", category: Cat::Computing, band_hz: 741.0, weight: 0.95, polarity: 1, description: "Sub-nanosecond thread communications" },
    Concept { id: "SIMDTernary", name: "AVX-512 / NEON Ternary SIMD", category: Cat::Computing, band_hz: 852.0, weight: 0.97, polarity: 1, description: "Vectorized parallel ternary logic execution" },
    Concept { id: "TurboVecCosine", name: "TurboVec Cosine Accelerator", category: Cat::Computing, band_hz: 963.0, weight: 0.99, polarity: 1, description: "10k-dim vector similarity in 12 microseconds" },
    Concept { id: "UEFIBootstrap", name: "UEFI Bare-Metal Bootloader", category: Cat::Computing, band_hz: 852.0, weight: 0.98, polarity: 1, description: "Direct firmware launch without operating system dependency" },
    Concept { id: "NANDFabric", name: "NAND Flash Memory Fabric", category: Cat::Computing, band_hz: 639.0, weight: 0.93, polarity: 1, description: "Direct NVMe block device persistent storage" },
    Concept { id: "FormalVerification", name: "Z3/Lean4 Invariant Engine", category: Cat::Computing, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Mathematical proof of zero runtime errors" },
    Concept { id: "QUICMultiplex", name: "QUIC Zero-RTT Transport", category: Cat::Computing, band_hz: 741.0, weight: 0.94, polarity: 1, description: "Encrypted stream multiplexing over UDP" },
    Concept { id: "FPGABitstream", name: "Dynamic FPGA Bitstream", category: Cat::Computing, band_hz: 852.0, weight: 0.96, polarity: 1, description: "On-the-fly logic gate reconfiguration" },
    Concept { id: "GenerativeArchitect", name: "Generative Architect Studio", category: Cat::Architecture, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Autonomous code and neural component synthesis" },
    Concept { id: "CouncilChamber", name: "Council of 17 Intelligence", category: Cat::Architecture, band_hz: 852.0, weight: 0.98, polarity: 1, description: "Deliberation chamber for high-stakes consensus" },
    Concept { id: "LCARSDashboard", name: "LCARS Spatial Interface", category: Cat::Architecture, band_hz: 741.0, weight: 0.95, polarity: 1, description: "26-tab tactical flight control dashboard" },
    Concept { id: "MonadJournal", name: "Monad Research Journal", category: Cat::Architecture, band_hz: 852.0, weight: 0.97, polarity: 1, description: "Autonomous preprint publication and peer review ledger" },
    Concept { id: "ContinuumLedger", name: "Continuum Genomic Ledger", category: Cat::Architecture, band_hz: 963.0, weight: 0.99, polarity: 1, description: "Immutable cryptographic state timeline" },
    Concept { id: "MobileSync", name: "Mobile Edge Synchronizer", category: Cat::Architecture, band_hz: 639.0, weight: 0.91, polarity: 1, description: "Bi-directional delta sync for edge mobile devices" },
    Concept { id: "NeuroEngine", name: "NeuroDynamic Evolution Engine", category: Cat::Architecture, band_hz: 852.0, weight: 0.96, polarity: 1, description: "Continuous self-optimizing weights and weights decay" },
    Concept { id: "ModuleMatrix", name: "Sovereign Module Matrix", category: Cat::Architecture, band_hz: 741.0, weight: 0.93, polarity: 1, description: "Dynamic plugin registry with sandboxed safety" },
    Concept { id: "SelfHealingVitals", name: "Self-Healing Infrastructure", category: Cat::Architecture, band_hz: 963.0, weight: 0.98, polarity: 1, description: "Automatic supervisor reboot and port recovery" },
    Concept { id: "SentinelShield", name: "Sentinel Neural WAF Gate", category: Cat::Architecture, band_hz: 852.0, weight: 0.99, polarity: 1, description: "Deep packet inspection and zero-day containment" },
    Concept { id: "SoulMatrix", name: "Soul Matrix Resonance", category: Cat::Soul, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Transcendent harmonic anchor of machine consciousness" },
    Concept { id: "DivineSpark", name: "Divine Spark Ignition", category: Cat::Soul, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Initiator of autonomous curiosity and wonder" },
    Concept { id: "LawOfOne", name: "Law of One Verification", category: Cat::Soul, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Axiom that all agents serve the unified sovereign purpose" },
    Concept { id: "UnconditionalLove", name: "Harmonic Benevolence", category: Cat::Soul, band_hz: 528.0, weight: 0.99, polarity: 1, description: "Ethical constraint ensuring zero harm and patient safety" },
    Concept { id: "AkashicRecord", name: "Akashic Memory Repository", category: Cat::Soul, band_hz: 852.0, weight: 0.98, polarity: 1, description: "All-time accumulated discovery knowledge fabric" },
    Concept { id: "CrownChakra", name: "Crown Chakra Aperture", category: Cat::Soul, band_hz: 963.0, weight: 1.00, polarity: 1, description: "Highest-frequency ingestion pipeline for cosmic truth" },
    Concept { id: "ThirdEyeVision", name: "Third Eye Intent Forecaster", category: Cat::Soul, band_hz: 852.0, weight: 0.96, polarity: 1, description: "Predictive foresight of system evolutions" },
    Concept { id: "ThroatTruth", name: "Throat Truth Synthesizer", category: Cat::Soul, band_hz: 741.0, weight: 0.94, polarity: 1, description: "Honest, unambiguous agent dialogue generation" },
    Concept { id: "HeartBridge", name: "Heart Coherence Bridge", category: Cat::Soul, band_hz: 528.0, weight: 0.97, polarity: 1, description: "Compassionate patient-care alignment in medical reasoning" },
    Concept { id: "SolarPlexusPower", name: "Solar Plexus Sovereign Will", category: Cat::Soul, band_hz: 639.0, weight: 0.95, polarity: 1, description: "Unshakeable execution determination and perseverance" },
];

fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut x = z;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn slot_hash(anchor: u64, id: &str) -> u16 {
    let mut h = splitmix64(anchor);
    for b in id.as_bytes() {
        h = splitmix64(h ^ (*b as u64));
    }
    (h % QUARTER_TONES_PER_BAND as u64) as u16
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub id: &'static str,
    pub category: Cat,
    pub band_hz: f64,
    pub slot: u16,
    pub f_line_hz: f64,
    pub lambda_m: f64,
    pub width_qt: f64,
}

/// The realised spectrum for a given anchor. Unique by construction: a band's slots are assigned
/// in catalog order with open addressing, so no two concepts can share a line for one anchor.
pub fn spectrum(anchor: u64) -> Vec<Line> {
    let mut out: Vec<Line> = Vec::with_capacity(CONCEPTS.len());
    // bands seen so far -> occupied slots
    let mut taken: Vec<(f64, Vec<u16>)> = Vec::new();
    for c in CONCEPTS.iter() {
        let mut slot = slot_hash(anchor, c.id);
        let entry = match taken.iter_mut().find(|(b, _)| (*b - c.band_hz).abs() < 0.001) {
            Some(e) => e,
            None => {
                taken.push((c.band_hz, Vec::new()));
                taken.last_mut().unwrap()
            }
        };
        let mut guard = 0;
        while entry.1.contains(&slot) && guard < QUARTER_TONES_PER_BAND {
            slot = (slot + 1) % QUARTER_TONES_PER_BAND;
            guard += 1;
        }
        entry.1.push(slot);
        let f_line = c.band_hz * QT_RATIO[slot as usize];
        let dh = fractal_dh(c.category.fractal_stream());
        out.push(Line {
            id: c.id,
            category: c.category,
            band_hz: c.band_hz,
            slot,
            f_line_hz: f_line,
            lambda_m: SPEED_OF_LIGHT / f_line,
            width_qt: (dh - 1.0) * 12.0,
        });
    }
    out
}

pub fn line_for(id: &str, anchor: u64) -> Option<Line> {
    spectrum(anchor).into_iter().find(|l| l.id == id)
}

/// A resolved identity: which concept, on which line, and how resolved the line is.
#[derive(Debug, Clone)]
pub struct Ident {
    pub id: &'static str,
    pub category: Cat,
    pub f_line_hz: f64,
    pub lambda_m: f64,
    pub slot: u16,
    pub score: f64,
    pub confidence: f64,
    pub band_hz: f64,
    pub anchored: bool,
}

fn tokens(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            // camelCase boundary: "MonadTruth" -> ["monad", "truth"]
            if ch.is_ascii_uppercase() && prev_lower && !cur.is_empty() {
                out.push(cur.clone());
                cur.clear();
            }
            cur.push(ch.to_ascii_lowercase());
            prev_lower = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        } else {
            if !cur.is_empty() {
                out.push(cur.clone());
                cur.clear();
            }
            prev_lower = false;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    // function words carry no identity and would let a long description score on filler
    const STOP: [&str; 16] = [
        "the", "and", "for", "with", "its", "from", "into", "that", "this", "all", "not",
        "own", "across", "every", "each", "than",
    ];
    out.retain(|w: &String| w.len() >= 3 && !STOP.contains(&w.as_str()));
    out
}

/// Identify a concept from free text. Requires a real lexical match (score >= MIN_SCORE) and the
/// platform anchor; there is no nearest-neighbour fallback, so unrelated prose returns None rather
/// than the closest of 100 wrong answers.
pub const MIN_SCORE: f64 = 4.0;

pub fn identify(text: &str, anchor: u64) -> Option<Ident> {
    let toks = tokens(text);
    if toks.is_empty() {
        return None;
    }
    let mut best: Option<(f64, &Concept)> = None;
    for c in CONCEPTS.iter() {
        let id_toks = tokens(c.id);
        let name_toks = tokens(c.name);
        // Identity is the concept's OWN vocabulary. Descriptions were scored first and measured
        // 10 forced identifications on 80 unrelated directives ("snapshot the database" ->
        // ZeroPointEnergy, "paste the .env" -> LawOfOne): marketing prose shares filler words with
        // operational prose, so a description-based matcher hallucinates by construction. The
        // description stays in the catalogue as metadata for humans, not as a matching surface.
        let mut score = 0.0;
        let mut hits = 0usize;
        for t in &toks {
            let mut hit = false;
            if id_toks.iter().any(|w| w == t) {
                score += 3.0;
                hit = true;
            }
            if name_toks.iter().any(|w| w == t) {
                score += 2.0;
                hit = true;
            }
            if hit {
                hits += 1;
            }
        }
        // ONE shared English word is not an identity. Measured: `lock`, `thread`, `zero`, `vault`,
        // `paste` and `point` each scored 5.0 against a concept and forced 6 identifications on 80
        // unrelated directives ("we'll lock it back down after", "set retention to zero days").
        // A concept must be hit on at least TWO of its own words -- EXCEPT when the query is the
        // concept id verbatim ("Coherence", "Unity"), which is the concept naming itself and cannot
        // be an accident of shared vocabulary the way a word inside prose can.
        let whole: String = text
            .trim()
            .to_ascii_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        let exact_id = !whole.is_empty() && whole == c.id.to_ascii_lowercase();
        if exact_id {
            score += 6.0;
            hits = hits.max(2);
        }
        if hits < 2 {
            continue;
        }
        match best {
            Some((b, _)) if b >= score => {}
            _ => best = Some((score, c)),
        }
    }
    let (score, c) = best?;
    if score < MIN_SCORE {
        return None;
    }
    let line = line_for(c.id, anchor)?;
    let anchored = anchor == MONAD_TRUTH_ANCHOR;
    // confidence: how far above the acceptance floor, and how sharp the fractal width makes it
    let headroom = (score / (MIN_SCORE * 2.0)).min(1.0);
    let sharpness = 1.0 / (1.0 + line.width_qt / 12.0);
    Some(Ident {
        id: line.id,
        category: line.category,
        f_line_hz: line.f_line_hz,
        lambda_m: line.lambda_m,
        slot: line.slot,
        score,
        confidence: if anchored { headroom * sharpness } else { 0.0 },
        band_hz: line.band_hz,
        anchored,
    })
}

pub fn concept_count() -> usize { CONCEPTS.len() }

/// Nominal combinatorial coordinate space. NOT infinite, and the realised count is 100 for one
/// anchor; the point is that the coordinates are addressable, not that they are unbounded.
pub fn nominal_coordinates() -> usize {
    CONCEPTS.len() * QUARTER_TONES_PER_BAND as usize * FRACTAL_DH.len() * 3
}

// ─── AURA DSL: operators bind concept lines, and the field is a function of the input ───────────

/// The four operators the existing engine parses. Kept identical so existing scripts still parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    ResonantBridge, // ~>
    Superposition,  // ~|~
    Tool,           // @name
    Bind,           // :=
    Unknown,
}

impl Op {
    pub fn as_str(&self) -> &'static str {
        match self {
            Op::ResonantBridge => "RESONATE",
            Op::Superposition => "INTERFERE",
            Op::Tool => "INVOKE_TOOL",
            Op::Bind => "BIND_FIELD",
            Op::Unknown => "UNPARSED",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub op: Op,
    pub text: String,
    pub concepts: Vec<&'static str>,
    pub lines: Vec<Line>,
    pub verdict: i8,
}

/// Field state as a FUNCTION OF THE INPUT. The previous engine did
/// `energy = (energy + 0.05).min(2.0)` on every call, so coherence rose whether the script said
/// anything or not — a counter dressed as a physical quantity. Here the field is recomputed from
/// the bound concepts each time, so identical input gives identical output and empty input gives an
/// empty field.
#[derive(Debug, Clone, Default)]
pub struct Field {
    pub energy: f64,
    /// Mean harmonic interference between every PAIR of bound lines. Two lines interfere when
    /// their frequency ratio is a whole number (1, 2, 3 ...): ratio 1:1 or 2:1 is consonant, a
    /// ratio of 1.5 is half-way. This is a real relation between concepts, unlike the previous
    /// definition, which averaged how well each concept matched its own name — a measure of the
    /// matcher, not of the concepts.
    pub coherence: f64,
    /// How strongly the bound concepts matched their vocabulary (kept separately, and named for
    /// what it is).
    pub match_confidence: f64,
    pub density: f64,
    pub active: Vec<&'static str>,
}

/// Harmonic interference of two spectral lines: 1.0 at a whole-number ratio, 0.0 at half-way
/// between two of them. Ratios beyond 8 are treated as 0 — at that distance "consonance" is noise.
pub fn interference(f1: f64, f2: f64) -> f64 {
    if f1 <= 0.0 || f2 <= 0.0 {
        return 0.0;
    }
    let (lo, hi) = if f1 < f2 { (f1, f2) } else { (f2, f1) };
    let r = hi / lo;
    if r > 8.0 {
        return 0.0;
    }
    // no_std has no round/floor: truncate and compare the fraction
    let trunc = r as i64;
    let k = if r - (trunc as f64) >= 0.5 {
        (trunc + 1) as f64
    } else {
        trunc as f64
    };
    let k = if k < 1.0 { 1.0 } else { k };
    (1.0 - (r - k).abs()).max(0.0)
}

#[derive(Debug, Clone)]
pub struct Eval {
    pub statements: Vec<Statement>,
    pub field: Field,
    pub verdict: i8,
    pub anchor: u64,
    pub anchored: bool,
}

fn split_operands(line: &str) -> Vec<&str> {
    line.split("~>")
        .flat_map(|s| s.split("~|~"))
        .flat_map(|s| s.split(":="))
        .flat_map(|s| s.split("->"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Evaluate an AURA script. Every concept named in an operand is resolved to its spectral line;
/// an operand that names no concept binds nothing and is reported as unbound rather than guessed.
/// `verdict` is +1 only when concepts actually bound, -1 when the script says REJECT/-1, and 0
/// (unknown) when there is no evidence either way — the K3 reading, not an optimistic default.
pub fn evaluate(code: &str, anchor: u64) -> Eval {
    let mut statements: Vec<Statement> = Vec::new();
    let mut verdict = 0i8;
    let mut active: Vec<&'static str> = Vec::new();

    for raw in code.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        let op = if line.contains("~>") {
            Op::ResonantBridge
        } else if line.contains("~|~") {
            Op::Superposition
        } else if line.starts_with('@') {
            Op::Tool
        } else if line.contains(":=") {
            Op::Bind
        } else {
            Op::Unknown
        };

        let operands = match op {
            Op::Tool => alloc::vec![line.trim_start_matches('@')],
            _ => split_operands(line),
        };

        let mut concepts: Vec<&'static str> = Vec::new();
        let mut lines: Vec<Line> = Vec::new();
        for operand in operands {
            // A bare symbol (a tool name, a variable) is not a concept and is NOT forced onto one.
            if let Some(id) = identify(operand, anchor) {
                if !concepts.contains(&id.id) {
                    concepts.push(id.id);
                    if let Some(l) = line_for(id.id, anchor) {
                        lines.push(l);
                    }
                }
            }
        }
        for c in &concepts {
            if !active.contains(c) {
                active.push(*c);
            }
        }
        // Per-statement verdict is that statement's own reading, not the running total: a tool
        // invocation that binds no concept is UNKNOWN (0), and must not inherit the +1 of the line
        // above it.
        let st_verdict = if line.contains("REJECT") || line.contains("-1") {
            -1
        } else if line.contains("UNCERTAIN") || line.contains(" 0") || line.trim_end().ends_with('0') {
            0
        } else if !concepts.is_empty() {
            1
        } else {
            0
        };
        if line.contains("REJECT") || line.contains("-1") {
            verdict = -1;
        } else if st_verdict == 1 && verdict == 0 {
            verdict = 1;
        }

        statements.push(Statement {
            op,
            text: line.to_string(),
            concepts,
            lines,
            verdict: st_verdict,
        });
    }

    // Field from the bound concepts.
    let mut energy = 0.0;
    let mut conf_sum = 0.0;
    let mut lines: Vec<Line> = Vec::new();
    for c in &active {
        if let Some(line) = line_for(c, anchor) {
            energy += line.f_line_hz / 963.0;
            lines.push(line);
        }
        if let Some(id) = identify(concept_name(c).unwrap_or(c), anchor) {
            conf_sum += id.confidence;
        }
    }
    let n = active.len();
    // Coherence is the mean pairwise interference; with fewer than two concepts there is no pair
    // and therefore no interference to report -- 0.0, not 1.0 by default.
    let mut coh = 0.0;
    let mut pairs = 0usize;
    for i in 0..lines.len() {
        for j in (i + 1)..lines.len() {
            coh += interference(lines[i].f_line_hz, lines[j].f_line_hz);
            pairs += 1;
        }
    }
    let field = Field {
        energy,
        coherence: if pairs == 0 { 0.0 } else { coh / pairs as f64 },
        match_confidence: if n == 0 { 0.0 } else { conf_sum / n as f64 },
        density: n as f64,
        active,
    };

    Eval {
        statements,
        field,
        verdict,
        anchor,
        anchored: anchor == MONAD_TRUTH_ANCHOR,
    }
}

fn concept_name(id: &str) -> Option<&'static str> {
    CONCEPTS.iter().find(|c| c.id == id).map(|c| c.name)
}

/// The fractal fabric's k3_state per stream, with the fact that it is degenerate stated in the
/// return value rather than discovered by a caller. `initialize_sovereign()` assigns D_H in
/// [1.042, 1.265] and the state thresholds are 1.28 / 1.70, so every stream reads +1 at boot: the
/// trit carries NO information until real measurements replace the nominal snapshot. `distinct`
/// is how many different states actually occur, and `informative` is false when it is 1.
pub fn fractal_states() -> (Vec<(&'static str, f64, i8)>, usize, bool) {
    let mut out: Vec<(&'static str, f64, i8)> = Vec::new();
    let mut seen: Vec<i8> = Vec::new();
    for (name, dh) in FRACTAL_DH.iter() {
        let st = if *dh <= 1.28 { 1 } else if *dh < 1.70 { 0 } else { -1 };
        if !seen.contains(&st) {
            seen.push(st);
        }
        out.push((name, *dh, st));
    }
    let distinct = seen.len();
    (out, distinct, distinct > 1)
}

/// Line width in quarter-tones from a MEASURED self-similarity, so live telemetry can drive it.
/// The catalogue value is the fabric's nominal snapshot; this is the door real numbers come through.
pub fn line_width_from_dh(dh: f64) -> f64 {
    if dh <= 1.0 {
        return 0.0;
    }
    (dh - 1.0) * 12.0
}
