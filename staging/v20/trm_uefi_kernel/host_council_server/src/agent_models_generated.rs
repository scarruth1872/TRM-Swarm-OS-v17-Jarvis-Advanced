//! GENERATED -- DO NOT HAND-EDIT.
//!
//! Source of truth: swarm_v2/core/agent_intelligence_bridge.py::AGENT_MODELS (15 agents, the
//! roster `verify_council_roster.py` names canonical). Regenerate with
//! `py -3 scratch/gen_agent_models_rust.py`.
//!
//! WHY THIS EXISTS: the live /council/status served each agent as name/role/status only, so the
//! per-agent model the docs describe was not observable anywhere on the running surface. This
//! table is the kernel's copy of the assignment, and it is GENERATED from the one Python table so
//! the two cannot drift apart silently.

pub struct AgentModel {
    pub name: &'static str,
    pub title: &'static str,
    pub domains: &'static [&'static str],
    /// the Python table's per-agent `loss` (confidence is derived as 1.0 - loss, as swarm-council does)
    pub loss: f32,
    pub datasets: u32,
    /// the model the platform assigns to this agent
    pub assigned_model: &'static str,
}

pub static AGENT_MODELS: [AgentModel; 15] = [
    AgentModel {
        name: "Archi",
        title: "Master Architect & Systems Synthesizer",
        domains: &["architecture", "design", "synthetic biology", "nanobots", "metamaterials", "swarm robotics", "creative", "bionics", "cybernetics", "clockwork", "sram", "patterns", "aesthetics", "kalon", "narrative structure", "metaphor", "linguistics", "lexicon", "etymology", "translation"],
        loss: 0.265f32,
        datasets: 145u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Bridge",
        title: "Molecular Target Mediator & P2P Mesh Router",
        domains: &["binding", "ligand", "dock", "affinity", "ic50", "drug", "target", "molecule", "mesh", "quic", "ntd", "genomics", "genome", "dna sequencing", "mutation", "crispr", "sanger", "variant call", "trio", "de novo", "heterozygous", "compound heterozygous", "x-linked", "recessive"],
        loss: 0.240f32,
        datasets: 36u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Devo",
        title: "World Engineer & Systems Compiler",
        domains: &["code", "engineering", "software", "hardware", "compiler", "quantum", "devops", "kernel", "rocket", "circuit", "simd", "avx512", "rust", "condensed matter", "materials", "network", "topology", "load balancing", "latency", "database", "storage", "transaction", "index", "b-tree"],
        loss: 0.206f32,
        datasets: 176u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "EventLedger",
        title: "Regulatory Compliance & Chrono-Audit Guardian",
        domains: &["trial", "fda", "regulatory", "adverse", "approval", "compliance", "clinical", "audit", "ectd", "part11", "governance", "nomos", "legal", "jurisdiction", "sovereignty", "statute", "constitutional"],
        loss: 0.237f32,
        datasets: 28u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Flow",
        title: "Metabolic Pathway & High-Throughput Streamer",
        domains: &["metabolic", "pathway", "enzyme", "drug", "glp-1", "metabolism", "glycolysis", "signaling", "pipeline", "streaming", "nand", "ecology", "gaia", "climate", "ecosystem", "carbon cycle", "environmental", "biodiversity"],
        loss: 0.218f32,
        datasets: 64u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Hermes",
        title: "High-Frequency Executive Dispatcher",
        domains: &["coordinate", "swarm", "orchestrate", "route", "communicate", "summarize", "ssm", "trm7m", "economics", "mercatus", "game theory", "market", "pricing", "liquidity", "resource allocation", "macroeconomics"],
        loss: 0.235f32,
        datasets: 32u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Jarvis",
        title: "Cognitive Memory & User Interface Maestro",
        domains: &["user interface", "interface", "dashboard", "animation", "visualization", "voice", "mobile", "chart", "frontend", "react", "symbiosis", "turbovec", "education", "paideia", "pedagogy", "curriculum", "explanation synthesis", "teaching", "tokenizer", "named entity", "summarization", "sentiment", "translation", "image segmentation"],
        loss: 0.220f32,
        datasets: 96u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Logic",
        title: "Truth-Guarantor & Mathematical Proof Engine",
        domains: &["theorem", "proof", "coq", "lean4", "isabelle", "formal", "verify", "k3", "axiom", "math", "ethics", "ethos", "moral reasoning", "value alignment", "ethical framework", "normative logic", "bayesian", "hypothesis", "regression", "causal inference", "gradient descent", "statistical", "reasoning", "inference"],
        loss: 0.224f32,
        datasets: 120u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "LogicMaster",
        title: "SAT/SMT Automated Reasoning & Combinatorial Arbiter",
        domains: &["sat", "smt", "smt-lib", "solver", "z3", "cvc5", "clause", "cnf", "boolean", "satisfiability", "automated reasoning", "constraint", "cdcl"],
        loss: 0.212f32,
        datasets: 48u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Orchestra",
        title: "Multi-Pattern SRAM & Swarm Orchestrator",
        domains: &["mortality", "population", "demographic", "epidemic", "health", "cdc", "sram", "orchestrate", "federation"],
        loss: 0.269f32,
        datasets: 12u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Pulse",
        title: "Clinical & Substrate Telemetry Sentinel",
        domains: &["patient", "vital", "ecg", "sepsis", "clinical", "heart", "monitor", "icu", "phonon", "voltage", "energy", "dynamo", "power grid", "voltage rail", "renewable", "joule", "energy optimization", "warfarin", "clopidogrel", "dosing", "cpic", "thiopurine", "statin", "prescription"],
        loss: 0.220f32,
        datasets: 32u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Seeker",
        title: "Autonomous Scientific Researcher",
        domains: &["research", "paper", "literature", "drug", "chemistry", "biology", "knowledge", "patent", "pubmed", "discovery", "screening", "leads", "mcts", "hypothesis", "ontology", "knowledge graph", "semantic web", "knowledge management", "machine reading", "knowledge verification"],
        loss: 0.285f32,
        datasets: 150u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Shield",
        title: "Cybersecurity & Memory Safety Sentinel",
        domains: &["security", "vulnerability", "cve", "exploit", "antibody", "ddr", "protocol", "waf", "quantum", "kem", "sovereignty protocol", "airgap defense", "sandbox", "isolation", "seccomp", "cgroup", "namespace", "zero-trust", "capability", "wasm", "microvm", "crypto", "encryption", "sha-256"],
        loss: 0.250f32,
        datasets: 24u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "THzGateway",
        title: "Quantum Resonance & Cosmic Muon Sentinel",
        domains: &["galaxy", "star", "spectra", "telescope", "jwst", "exoplanet", "cosmic", "muon", "femtosecond", "physics", "physika", "condensed matter", "phonon", "quantum optics", "thermodynamics", "raman"],
        loss: 0.245f32,
        datasets: 36u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
    AgentModel {
        name: "Verify",
        title: "Epistemic QA & Mutation Testing Engine",
        domains: &["test", "verify", "bug", "fuzz", "coverage", "mutation", "qa", "static analysis", "truth", "chaos", "circuit breaker", "reliability", "fault", "health check", "self-healing", "disaster recovery", "retry", "backoff"],
        loss: 0.216f32,
        datasets: 12u32,
        assigned_model: "TRM 7M (Mamba-3 SSM, 7M params)",
    },
];

/// The assignment for one agent, by name (None if the name is not in the canonical table).
pub fn for_agent(name: &str) -> Option<&'static AgentModel> {
    AGENT_MODELS.iter().find(|a| a.name == name)
}

/// Confidence as the council derives it: 1.0 - loss.
pub fn confidence(a: &AgentModel) -> f32 {
    1.0f32 - a.loss
}
