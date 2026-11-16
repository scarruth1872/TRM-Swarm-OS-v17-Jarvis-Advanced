//! swarm-agents — the 64-agent fleet re-wired to the native path (MOTION-052).
//! Each agent is a name + fleet + role + a native op (or its honest status). No Ollama, no Python agent.
//! The registry is data: the census op reports it so the mapping cannot drift from what is dispatched.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status { Native, NewStrip, External }

pub struct Agent {
    pub name: &'static str,
    pub fleet: &'static str,
    pub role: &'static str,
    pub native_op: Option<&'static str>,
    pub status: Status,
    pub gap: Option<&'static str>,
}

const N: Option<&'static str> = None;

pub const AGENTS: [Agent; 64] = [
    // Fleet 1: Security & Defense
    Agent { name: "Shield", fleet: "Security & Defense", role: "eBPF Intrusion Defense", native_op: Some("/ddr/scan"), status: Status::Native, gap: None },
    Agent { name: "VulnScan", fleet: "Security & Defense", role: "Static Code Vulnerability Scanner", native_op: Some("/ddr/scan"), status: Status::Native, gap: None },
    Agent { name: "CryptoAudit", fleet: "Security & Defense", role: "Post-Quantum Cryptographic Audit", native_op: Some("git/verify"), status: Status::Native, gap: None },
    Agent { name: "eBPFProbe", fleet: "Security & Defense", role: "Kernel Space Dynamic Tracer", native_op: Some("strip:ebpf_probe"), status: Status::Native, gap: None },
    Agent { name: "NeuralWall", fleet: "Security & Defense", role: "Adversarial Prompt Barrier", native_op: Some("k3/collapse"), status: Status::Native, gap: None },
    Agent { name: "ZeroTrustCert", fleet: "Security & Defense", role: "Identity Lock & Certificate Validator", native_op: Some("git/verify"), status: Status::Native, gap: None },
    Agent { name: "ASTProof", fleet: "Security & Defense", role: "Abstract Syntax Safety Verifier", native_op: Some("council/vote"), status: Status::Native, gap: None },
    Agent { name: "DDR_Antibody", fleet: "Security & Defense", role: "Digital DNA Fault Antibody Injector", native_op: Some("/ddr/antibodies"), status: Status::Native, gap: None },
    // Fleet 2: Logic & Mathematics
    Agent { name: "Logic", fleet: "Logic & Mathematics", role: "TRM Symbolic Reasoning Lead", native_op: Some("council/vote"), status: Status::Native, gap: None },
    Agent { name: "MathSolver", fleet: "Logic & Mathematics", role: "Differential & Matrix Solver", native_op: Some("strip:math_solver"), status: Status::Native, gap: None },
    Agent { name: "LogicProver", fleet: "Logic & Mathematics", role: "First-Order Logic Theorem Prover", native_op: Some("council/vote"), status: Status::Native, gap: None },
    Agent { name: "AttemptSampler", fleet: "Logic & Mathematics", role: "Quantum Superposition State Sampler", native_op: Some("strip:attempt_sampler"), status: Status::Native, gap: None },
    Agent { name: "ConsensusEngine", fleet: "Logic & Mathematics", role: "Heuristic Consensus Evaluator", native_op: Some("k3/collapse"), status: Status::Native, gap: None },
    Agent { name: "FormalChecker", fleet: "Logic & Mathematics", role: "Formal Protocol Invariant Guard", native_op: Some("council/vote"), status: Status::Native, gap: None },
    Agent { name: "SymbolSolver", fleet: "Logic & Mathematics", role: "Computer Algebra System", native_op: Some("strip:symbol_solver"), status: Status::Native, gap: None },
    Agent { name: "ProofVerifier", fleet: "Logic & Mathematics", role: "Coq/Lean Equivalence Proof Checker", native_op: Some("council/vote"), status: Status::Native, gap: None },
    // Fleet 3: Code & Refactoring
    Agent { name: "Devo", fleet: "Code & Refactoring", role: "Hot-Patch & C-Types Developer", native_op: Some("git/verify"), status: Status::Native, gap: None },
    Agent { name: "RefactorSkill", fleet: "Code & Refactoring", role: "Automated AST Code Restructurer", native_op: Some("strip:refactor_skill"), status: Status::Native, gap: None },
    Agent { name: "TestGenerator", fleet: "Code & Refactoring", role: "Unit & Integration Test Synthesizer", native_op: Some("strip:test_generator"), status: Status::Native, gap: None },
    Agent { name: "FuzzTestSkill", fleet: "Code & Refactoring", role: "Property-Based Fuzz Testing Engine", native_op: Some("strip:fuzz_test_skill"), status: Status::Native, gap: None },
    Agent { name: "ToolForge", fleet: "Code & Refactoring", role: "Dynamic Tool & Plugin Creator", native_op: Some("strip-build"), status: Status::Native, gap: None },
    Agent { name: "HotPatch", fleet: "Code & Refactoring", role: "Zero-Downtime Memory Patch Injector", native_op: Some("strip:hot_patch"), status: Status::Native, gap: None },
    Agent { name: "CTypesBuilder", fleet: "Code & Refactoring", role: "Native C/C++ Binding Compiler", native_op: Some("strip-build"), status: Status::Native, gap: None },
    Agent { name: "CodebaseIndexer", fleet: "Code & Refactoring", role: "AST Dependency Graph Indexer", native_op: Some("module-registry"), status: Status::Native, gap: None },
    // Fleet 4: Telemetry & Memory
    Agent { name: "Pulse", fleet: "Telemetry & Memory", role: "Real-Time System Thermal & Load Monitor", native_op: Some("/health"), status: Status::Native, gap: None },
    Agent { name: "GlobalMemory", fleet: "Telemetry & Memory", role: "High-Dimensional Vector Memory Bank", native_op: Some("/memory/query"), status: Status::Native, gap: None },
    Agent { name: "SystemMetrics", fleet: "Telemetry & Memory", role: "Kernel Resource Arbiter", native_op: Some("/infra/health"), status: Status::Native, gap: None },
    Agent { name: "MemorySynchronizer", fleet: "Telemetry & Memory", role: "Multi-Node Shared Memory Sync", native_op: Some("strip:memory_synchronizer"), status: Status::Native, gap: None },
    Agent { name: "MonitoringSkill", fleet: "Telemetry & Memory", role: "Prometheus/Grafana Metrics Exporter", native_op: Some("strip:monitoring_skill"), status: Status::Native, gap: None },
    Agent { name: "SharedMemory", fleet: "Telemetry & Memory", role: "POSIX Shared Memory Segment Manager", native_op: Some("strip:shared_memory"), status: Status::Native, gap: None },
    Agent { name: "ChromaFallback", fleet: "Telemetry & Memory", role: "Embedding Storage Fallback Engine", native_op: Some("trit/pack"), status: Status::Native, gap: None },
    Agent { name: "LedgerManager", fleet: "Telemetry & Memory", role: "System Event Ledger Recorder", native_op: Some("ledger"), status: Status::Native, gap: None },
    // Fleet 5: Synthesis & Intent
    Agent { name: "Archi", fleet: "Synthesis & Intent", role: "System Architecture Lead", native_op: Some("council/vote"), status: Status::Native, gap: None },
    Agent { name: "CoCreator", fleet: "Synthesis & Intent", role: "Collaborative Co-Design Agent", native_op: Some("humanistic/reason"), status: Status::Native, gap: None },
    Agent { name: "SynthesisPipeline", fleet: "Synthesis & Intent", role: "Multi-Model Response Synthesizer", native_op: Some("strip:synthesis_pipeline"), status: Status::Native, gap: None },
    Agent { name: "CognitiveOrchestrator", fleet: "Synthesis & Intent", role: "High-Level Cognitive Task Dispatcher", native_op: Some("agent/dispatch"), status: Status::Native, gap: None },
    Agent { name: "IntentBuffer", fleet: "Synthesis & Intent", role: "Predictive Intent Pre-Routing Engine", native_op: Some("strip:intent_buffer"), status: Status::Native, gap: None },
    Agent { name: "HeuristicConsensus", fleet: "Synthesis & Intent", role: "Vibe Consensus & Multi-Persona Evaluator", native_op: Some("k3/collapse"), status: Status::Native, gap: None },
    Agent { name: "SemanticCache", fleet: "Synthesis & Intent", role: "Sub-Millisecond Query Cache", native_op: Some("/memory/query"), status: Status::Native, gap: None },
    Agent { name: "PersonaMatrix", fleet: "Synthesis & Intent", role: "4D System Genomics Persona Mutator", native_op: Some("strip:persona_matrix"), status: Status::Native, gap: None },
    // Fleet 6: Autonomous Healing
    Agent { name: "SelfHealing", fleet: "Autonomous Healing", role: "Fault Capture & Remediation Engine", native_op: Some("supervisor"), status: Status::Native, gap: None },
    Agent { name: "ChaosSimulator", fleet: "Autonomous Healing", role: "Resilience & Chaos Monkey Injection", native_op: Some("strip:chaos_simulator"), status: Status::Native, gap: None },
    Agent { name: "Sentinel", fleet: "Autonomous Healing", role: "Watchdog & Health Probe Guard", native_op: Some("/health"), status: Status::Native, gap: None },
    Agent { name: "MicrokernelSpawner", fleet: "Autonomous Healing", role: "TTL Microkernel Sandbox Allocator", native_op: Some("supervisor"), status: Status::Native, gap: None },
    Agent { name: "StarMatrix", fleet: "Autonomous Healing", role: "Gravitational Resonance Lattice Manager", native_op: Some("strip:star_matrix"), status: Status::Native, gap: None },
    Agent { name: "DynamicRouter", fleet: "Autonomous Healing", role: "Adaptive Traffic & Load Balancer", native_op: Some("strip:dynamic_router"), status: Status::Native, gap: None },
    Agent { name: "ResourceArbiter", fleet: "Autonomous Healing", role: "CPU/RAM Emergency Load Shedder", native_op: Some("core-pinning"), status: Status::Native, gap: None },
    Agent { name: "PBFTConsensus", fleet: "Autonomous Healing", role: "3-Phase PBFT Consensus Validator", native_op: Some("council/vote"), status: Status::Native, gap: None },
    // Fleet 7: Scientific Intelligence
    Agent { name: "Seeker", fleet: "Scientific Intelligence", role: "PubMed & Scientific Literature Scraper", native_op: Some("/memory/query"), status: Status::Native, gap: None },
    Agent { name: "PubMedScan", fleet: "Scientific Intelligence", role: "NCBI E-Utilities Literature Parser", native_op: Some("/memory/query"), status: Status::Native, gap: None },
    Agent { name: "OpenAlexSearch", fleet: "Scientific Intelligence", role: "OpenAlex Scholarly Graph Query Engine", native_op: Some("/memory/query"), status: Status::Native, gap: None },
    Agent { name: "AlphaFold3D", fleet: "Scientific Intelligence", role: "AlphaFold 3D Protein Structure Resolver", native_op: Some("swarm-docking"), status: Status::Native, gap: None },
    Agent { name: "AlphaGenome", fleet: "Scientific Intelligence", role: "AlphaGenome Variant Effect Predictor", native_op: Some("strip:alpha_genome"), status: Status::Native, gap: None },
    Agent { name: "FoldseekEngine", fleet: "Scientific Intelligence", role: "3D Structural Homology Alignment", native_op: Some("swarm-docking"), status: Status::Native, gap: None },
    Agent { name: "SequenceAligner", fleet: "Scientific Intelligence", role: "Clustal Omega Protein Sequence Aligner", native_op: Some("strip:sequence_aligner"), status: Status::Native, gap: None },
    Agent { name: "ChemblQuery", fleet: "Scientific Intelligence", role: "ChEMBL Bioactivity & Molecule Search", native_op: Some("chembl-store"), status: Status::Native, gap: None },
    // Fleet 8: Human Interaction & Edge
    Agent { name: "Vision", fleet: "Human Interaction & Edge", role: "Multimodal Image & UI Analytics", native_op: Some("multimodal-ingestion"), status: Status::Native, gap: Some("recognition model (native multimodal ingestion only)") },
    Agent { name: "Flow", fleet: "Human Interaction & Edge", role: "React UI Component Synthesis Lead", native_op: Some("/ui"), status: Status::Native, gap: None },
    Agent { name: "Bridge", fleet: "Human Interaction & Edge", role: "External REST/gRPC API Gateway", native_op: Some("kernel-http"), status: Status::Native, gap: None },
    Agent { name: "Echo", fleet: "Human Interaction & Edge", role: "Voice STT/TTS & Telegram Uplink", native_op: Some("human-swarm-interaction"), status: Status::Native, gap: Some("TTS synthesis (native command translation only)") },
    Agent { name: "TelegramDaemon", fleet: "Human Interaction & Edge", role: "Telegram Event Polling Conduit", native_op: N, status: Status::External, gap: None },
    Agent { name: "MobileSync", fleet: "Human Interaction & Edge", role: "Sovereign Mobile App Companion Sync", native_op: N, status: Status::External, gap: None },
    Agent { name: "VoiceConduit", fleet: "Human Interaction & Edge", role: "Real-Time Audio Stream Processing", native_op: Some("human-swarm-interaction"), status: Status::Native, gap: Some("audio-device capture (native speech parsing only)") },
    Agent { name: "UI_UX_Designer", fleet: "Human Interaction & Edge", role: "Aesthetic & Layout Math Engine", native_op: Some("lcars-render"), status: Status::Native, gap: None },
];

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self { Status::Native => "native", Status::NewStrip => "new-strip", Status::External => "external" }
    }
}

pub fn find(name: &str) -> Option<&'static Agent> {
    AGENTS.iter().find(|a| a.name == name)
}

pub fn tally() -> (usize, usize, usize) {
    let (mut n, mut s, mut e) = (0, 0, 0);
    for a in &AGENTS {
        match a.status { Status::Native => n += 1, Status::NewStrip => s += 1, Status::External => e += 1 }
    }
    (n, s, e)
}

/// Map an agenda facet to the fleet that handles it (the dispatch routing).
pub fn facet_fleet(facet: &str) -> Option<&'static str> {
    Some(match facet {
        "molecule" | "bioassay" | "drug" | "protein" | "docking" | "genome" => "Scientific Intelligence",
        "retrieval" | "trm7m" | "memory" | "embedding" => "Telemetry & Memory",
        "hardware" | "compute" | "rail" | "healing" => "Autonomous Healing",
        "security" | "vuln" | "crypto" | "antibody" => "Security & Defense",
        "logic" | "proof" | "formal" | "math" => "Logic & Mathematics",
        "code" | "refactor" | "test" | "fuzz" | "patch" => "Code & Refactoring",
        "synthesis" | "intent" | "humanistic" => "Synthesis & Intent",
        _ => return None,
    })
}

/// Every agent in a fleet (the dispatchable set for a facet).
pub fn fleet_agents(fleet: &str) -> Vec<&'static Agent> {
    AGENTS.iter().filter(|a| a.fleet == fleet).collect()
}
