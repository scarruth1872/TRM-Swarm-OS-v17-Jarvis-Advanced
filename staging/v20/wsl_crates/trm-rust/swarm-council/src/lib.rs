//! swarm-council — K3 council-vote logic (Rust port of the Python council).
//!
//! Absorbs the agent-voting path that previously lived in the Python
//! bridge (agent_intelligence_bridge.py AGENT_MODELS): per-motion domain
//! overlap + confidence -> K3 vote vector -> collapse via
//! c_kleene_quantize_avx2 semantics -> PROCEED / BLOCK / ANALYZE(tie).
//!
//! Vote semantics (council doctrine, Motion A absorption):
//!   for each motion domain-set M:
//!     overlap = |agent.domains intersect M|
//!     overlap == 0 -> ABSTAIN (agent does not enter the vote at all)
//!     else vote vector v = [ov_sig, conf, conf]
//!       ov_sig = 2.0 if overlap >= 2 else 1.0 if overlap == 1
//!     collapse v elementwise with eps = 0.35 (c_kleene_quantize_avx2)
//!     council tally: majority of collapsed trits -> +1 PROCEED,
//!                    -1 BLOCK, 0 tie (ANALYZE)
//!
//! The 15-agent domain table below is generated from the live Python
//! AGENT_MODELS dict (see scratch/regenerate_council_table.py). Confidence is derived as 1.0 - loss (the Python
//! source stores per-agent `loss`, ~0.20-0.30, so confidence lands at
//! the ~0.70-0.79 band the council spec assumes).

/// One council agent: domain table + derived confidence.
pub struct AgentModel {
    pub name: &'static str,
    pub domains: &'static [&'static str],
    /// confidence = 1.0 - loss (loss is the value stored in the Python bridge)
    pub confidence: f32,
}

/// The 15-agent AGENT_MODELS domain table, GENERATED from
/// swarm_v2/core/agent_intelligence_bridge.py (loss -> confidence) by
/// scratch/regenerate_council_table.py. The earlier hand-written table was a
/// partial mirror -- 14 agents and truncated domain lists -- which the oracle
/// comparison in scratch/verify_crate_ops_p2j.py caught.
pub static AGENT_MODELS: [AgentModel; 15] = [
    AgentModel {
        name: "Archi",
        domains: &[
            "architecture",
            "design",
            "synthetic biology",
            "nanobots",
            "metamaterials",
            "swarm robotics",
            "creative",
            "bionics",
            "cybernetics",
            "clockwork",
            "sram",
            "patterns",
            "aesthetics",
            "kalon",
            "narrative structure",
            "metaphor",
            "linguistics",
            "lexicon",
            "etymology",
            "translation",
        ],
        confidence: 1.0 - 0.265,
    },
    AgentModel {
        name: "Shield",
        domains: &[
            "security",
            "vulnerability",
            "cve",
            "exploit",
            "antibody",
            "ddr",
            "protocol",
            "waf",
            "quantum",
            "kem",
            "sovereignty protocol",
            "airgap defense",
            "sandbox",
            "isolation",
            "seccomp",
            "cgroup",
            "namespace",
            "zero-trust",
            "capability",
            "wasm",
            "microvm",
            "crypto",
            "encryption",
            "sha-256",
            "chacha20",
            "ed25519",
            "aes-256",
            "merkle",
            "zero-knowledge",
            "hash",
            "signature",
            "threat",
            "side-channel",
            "secure boot",
            "enclave",
            "tpm",
            "intrusion",
            "penetration",
            "privilege escalation",
            "identity",
            "authentication",
            "access control",
            "credential",
            "biometric",
            "privacy",
            "differential privacy",
            "anonymization",
            "gdpr",
            "malware",
            "rootkit",
            "forensics",
            "attestation",
            "firmware",
            "unclonable",
            "blockchain",
            "smart contract",
            "proof-of-work",
        ],
        confidence: 1.0 - 0.25,
    },
    AgentModel {
        name: "Logic",
        domains: &[
            "theorem",
            "proof",
            "coq",
            "lean4",
            "isabelle",
            "formal",
            "verify",
            "k3",
            "axiom",
            "math",
            "ethics",
            "ethos",
            "moral reasoning",
            "value alignment",
            "ethical framework",
            "normative logic",
            "bayesian",
            "hypothesis",
            "regression",
            "causal inference",
            "gradient descent",
            "statistical",
            "reasoning",
            "inference",
            "knowledge representation",
        ],
        confidence: 1.0 - 0.224,
    },
    AgentModel {
        name: "Devo",
        domains: &[
            "code",
            "engineering",
            "software",
            "hardware",
            "compiler",
            "quantum",
            "devops",
            "kernel",
            "rocket",
            "circuit",
            "simd",
            "avx512",
            "rust",
            "condensed matter",
            "materials",
            "network",
            "topology",
            "load balancing",
            "latency",
            "database",
            "storage",
            "transaction",
            "index",
            "b-tree",
            "lsm",
            "filesystem",
            "scheduler",
            "interrupt",
            "boot",
            "syscall",
            "distributed",
            "consensus",
            "replication",
            "queue",
            "allocator",
            "lock-free",
            "cache",
            "profiling",
            "benchmark",
            "optimization",
            "container",
            "virtualization",
            "mamba",
            "transformer",
            "neural",
            "deep learning",
            "machine learning",
            "quantization",
            "reinforcement learning",
            "simulation",
            "netcdf",
            "scientific data",
            "array data",
            "hdf5",
        ],
        confidence: 1.0 - 0.206,
    },
    AgentModel {
        name: "Seeker",
        domains: &[
            "research",
            "paper",
            "literature",
            "drug",
            "chemistry",
            "biology",
            "knowledge",
            "patent",
            "pubmed",
            "discovery",
            "screening",
            "leads",
            "mcts",
            "hypothesis",
            "ontology",
            "knowledge graph",
            "semantic web",
            "knowledge management",
            "machine reading",
            "knowledge verification",
        ],
        confidence: 1.0 - 0.285,
    },
    AgentModel {
        name: "Jarvis",
        domains: &[
            "user interface",
            "interface",
            "dashboard",
            "animation",
            "visualization",
            "voice",
            "mobile",
            "chart",
            "frontend",
            "react",
            "symbiosis",
            "turbovec",
            "education",
            "paideia",
            "pedagogy",
            "curriculum",
            "explanation synthesis",
            "teaching",
            "tokenizer",
            "named entity",
            "summarization",
            "sentiment",
            "translation",
            "image segmentation",
            "object detection",
            "classification",
            "text mining",
            "information extraction",
            "data integration",
            "document classification",
        ],
        confidence: 1.0 - 0.22,
    },
    AgentModel {
        name: "Pulse",
        domains: &[
            "patient",
            "vital",
            "ecg",
            "sepsis",
            "clinical",
            "heart",
            "monitor",
            "icu",
            "phonon",
            "voltage",
            "energy",
            "dynamo",
            "power grid",
            "voltage rail",
            "renewable",
            "joule",
            "energy optimization",
            "warfarin",
            "clopidogrel",
            "dosing",
            "cpic",
            "thiopurine",
            "statin",
            "prescription",
            "adverse reaction",
            "dicom",
            "medical imaging",
            "radiology",
            "mri",
            "ct scan",
            "fhir",
            "hl7",
            "electronic health record",
            "healthcare",
        ],
        confidence: 1.0 - 0.22,
    },
    AgentModel {
        name: "Flow",
        domains: &[
            "metabolic",
            "pathway",
            "enzyme",
            "drug",
            "glp-1",
            "metabolism",
            "glycolysis",
            "signaling",
            "pipeline",
            "streaming",
            "nand",
            "ecology",
            "gaia",
            "climate",
            "ecosystem",
            "carbon cycle",
            "environmental",
            "biodiversity",
        ],
        confidence: 1.0 - 0.218,
    },
    AgentModel {
        name: "EventLedger",
        domains: &[
            "trial",
            "fda",
            "regulatory",
            "adverse",
            "approval",
            "compliance",
            "clinical",
            "audit",
            "ectd",
            "part11",
            "governance",
            "nomos",
            "legal",
            "jurisdiction",
            "sovereignty",
            "statute",
            "constitutional",
        ],
        confidence: 1.0 - 0.237,
    },
    AgentModel {
        name: "Orchestra",
        domains: &[
            "mortality",
            "population",
            "demographic",
            "epidemic",
            "health",
            "cdc",
            "sram",
            "orchestrate",
            "federation",
        ],
        confidence: 1.0 - 0.269,
    },
    AgentModel {
        name: "THzGateway",
        domains: &[
            "galaxy",
            "star",
            "spectra",
            "telescope",
            "jwst",
            "exoplanet",
            "cosmic",
            "muon",
            "femtosecond",
            "physics",
            "physika",
            "condensed matter",
            "phonon",
            "quantum optics",
            "thermodynamics",
            "raman",
        ],
        confidence: 1.0 - 0.245,
    },
    AgentModel {
        name: "Verify",
        domains: &[
            "test",
            "verify",
            "bug",
            "fuzz",
            "coverage",
            "mutation",
            "qa",
            "static analysis",
            "truth",
            "chaos",
            "circuit breaker",
            "reliability",
            "fault",
            "health check",
            "self-healing",
            "disaster recovery",
            "retry",
            "backoff",
        ],
        confidence: 1.0 - 0.216,
    },
    AgentModel {
        name: "Bridge",
        domains: &[
            "binding",
            "ligand",
            "dock",
            "affinity",
            "ic50",
            "drug",
            "target",
            "molecule",
            "mesh",
            "quic",
            "ntd",
            "genomics",
            "genome",
            "dna sequencing",
            "mutation",
            "crispr",
            "sanger",
            "variant call",
            "trio",
            "de novo",
            "heterozygous",
            "compound heterozygous",
            "x-linked",
            "recessive",
            "dominant",
            "autozygosity",
            "runs of homozygosity",
            "imprinting",
            "methylation",
            "exome",
            "variant prioritization",
            "phenotype",
            "diagnostic yield",
            "carrier",
            "haplotype",
            "mitochondrial dna",
            "heteroplasmy",
            "consanguineous",
            "variant",
            "association",
            "pharmacogenomic",
            "pharmacogenomics",
            "star allele",
            "genotype",
            "allele",
            "cyp2d6",
            "cyp2c19",
            "cyp2c9",
            "vkorc1",
            "slco1b1",
            "tpmt",
            "nudt15",
            "hla",
            "drug response",
            "drug-gene",
            "population genetics",
            "allele frequency",
            "hardy weinberg",
            "carrier frequency",
            "founder mutation",
            "gnomad",
            "ancestry",
            "population stratification",
            "selection signal",
            "pathogenic",
            "benign",
            "driver mutation",
            "mutational signature",
            "neoantigen",
            "clonal evolution",
            "mutational burden",
            "actionable",
            "immunotherapy",
            "cancer subtype",
            "microsatellite",
            "precision oncology",
            "oncogene",
            "tumor suppressor",
            "biomarker",
            "tumor",
            "gene fusion",
            "fusion",
            "breakpoint",
            "translocation",
            "chimeric",
            "fusion transcript",
            "junction",
            "chromosomal rearrangement",
            "chromoplexy",
            "chromothripsis",
            "epigenetic",
            "differentially methylated",
            "chromatin",
            "enhancer",
            "histone",
            "epigenetic clock",
            "histone modification",
            "non-coding",
            "transcriptomic",
            "transcript",
            "differential expression",
            "rna-seq",
            "splicing",
            "co-expression",
            "single-cell",
            "deconvolution",
            "gene expression",
            "rna-seq processing",
            "proteomic",
            "proteogenomic",
            "protein",
            "post-translational",
            "mass spectrometry",
            "phosphoproteomic",
            "protein-protein",
            "protein structure",
            "protein localization",
            "multi-omic",
            "microbiome",
            "metabolomic",
            "systems biology",
            "personalized medicine",
            "cross-omic",
            "cellular pathway",
            "survival analysis",
            "omics integration",
            "mzml",
            "mass spec",
            "mmcif",
            "pdbx",
            "crystallography",
            "abif",
            "sanger trace",
            "chromatogram",
        ],
        confidence: 1.0 - 0.24,
    },
    AgentModel {
        name: "Hermes",
        domains: &[
            "coordinate",
            "swarm",
            "orchestrate",
            "route",
            "communicate",
            "summarize",
            "ssm",
            "trm7m",
            "economics",
            "mercatus",
            "game theory",
            "market",
            "pricing",
            "liquidity",
            "resource allocation",
            "macroeconomics",
        ],
        confidence: 1.0 - 0.235,
    },
    AgentModel {
        name: "LogicMaster",
        domains: &[
            "sat",
            "smt",
            "smt-lib",
            "solver",
            "z3",
            "cvc5",
            "clause",
            "cnf",
            "boolean",
            "satisfiability",
            "automated reasoning",
            "constraint",
            "cdcl",
        ],
        confidence: 1.0 - 0.212,
    },
];

/// Council epsilon for c_kleene_quantize_avx2 semantics (v > eps -> +1,
/// v < -eps -> -1, else 0).
pub const EPS: f32 = 0.35;

/// A single agent's vote on a motion: trits after collapse, or Abstain.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentVote {
    Abstain,
    Voted {
        trits: Vec<i8>,
        vote_vector: Vec<f32>,
    },
}

/// Final council verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Proceed,
    Block,
    Analyze, // tie
}

impl Verdict {
    pub fn label(&self) -> &'static str {
        match self {
            Verdict::Proceed => "PROCEED",
            Verdict::Block => "BLOCK",
            Verdict::Analyze => "ANALYZE",
        }
    }
    pub fn trit(&self) -> i8 {
        match self {
            Verdict::Proceed => 1,
            Verdict::Block => -1,
            Verdict::Analyze => 0,
        }
    }
}

/// Overlap count: |agent.domains intersect motion| (exact token match,
/// same as the Python `set(domains) & set(motion)` semantics).
pub fn overlap(agent: &AgentModel, motion: &[&str]) -> usize {
    motion.iter().filter(|d| agent.domains.contains(d)).count()
}

/// Build the raw vote vector for an agent on a motion (pre-collapse).
/// Returns None when overlap == 0 (ABSTAIN).
pub fn vote_vector(agent: &AgentModel, motion: &[&str]) -> Option<Vec<f32>> {
    let ov = overlap(agent, motion);
    if ov == 0 {
        return None;
    }
    let ov_sig = if ov >= 2 { 2.0 } else { 1.0 };
    Some(vec![ov_sig, agent.confidence, agent.confidence])
}

/// One agent's K3 vote: None (Abstain) or the collapsed trits of
/// [ov_sig, conf, conf] at EPS, via swarm-k3::collapse (byte-identical
/// to c_kleene_quantize_avx2, verified).
pub fn agent_vote(agent: &AgentModel, motion: &[&str]) -> AgentVote {
    match vote_vector(agent, motion) {
        None => AgentVote::Abstain,
        Some(v) => AgentVote::Voted {
            trits: swarm_k3::collapse(&v, EPS),
            vote_vector: v,
        },
    }
}

/// Council tally over ALL voting agents: majority of collapsed trits
/// (sum > 0 -> PROCEED, sum < 0 -> BLOCK, sum == 0 -> ANALYZE tie).
/// Abstaining agents contribute nothing.
pub fn council_verdict(
    agents: &[AgentModel],
    motion: &[&str],
) -> (Verdict, Vec<(usize, AgentVote)>) {
    let mut votes = Vec::new();
    let mut trit_sum: i64 = 0;
    for (i, a) in agents.iter().enumerate() {
        let v = agent_vote(a, motion);
        if let AgentVote::Voted { trits, .. } = &v {
            trit_sum += trits.iter().map(|&t| t as i64).sum::<i64>();
        }
        votes.push((i, v));
    }
    let verdict = if trit_sum > 0 {
        Verdict::Proceed
    } else if trit_sum < 0 {
        Verdict::Block
    } else {
        Verdict::Analyze
    };
    (verdict, votes)
}

/// The two motion domain-sets from the last council meeting.
pub static MOTION_A: [&str; 7] = [
    "code",
    "engineering",
    "software",
    "compiler",
    "test",
    "verify",
    "kernel",
];
pub static MOTION_E: [&str; 7] = [
    "research",
    "literature",
    "drug",
    "chemistry",
    "biology",
    "discovery",
    "clinical",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_counts_match_python_sets() {
        // Devo x Motion A: code, engineering, software, compiler, kernel = 5
        let devo = &AGENT_MODELS[3];
        assert_eq!(overlap(devo, &MOTION_A), 5);
        // Seeker x Motion E: research, literature, drug, chemistry, biology, discovery = 6
        let seeker = &AGENT_MODELS[4];
        assert_eq!(overlap(seeker, &MOTION_E), 6);
        // Archi x Motion A: zero -> abstain
        assert_eq!(overlap(&AGENT_MODELS[0], &MOTION_A), 0);
        assert!(vote_vector(&AGENT_MODELS[0], &MOTION_A).is_none());
    }

    #[test]
    fn vote_vector_semantics() {
        // overlap 5 -> ov_sig 2.0
        let v = vote_vector(&AGENT_MODELS[3], &MOTION_A).unwrap();
        assert_eq!(v[0], 2.0);
        // overlap 1 -> ov_sig 1.0 (Logic x Motion A: only "verify")
        let v = vote_vector(&AGENT_MODELS[2], &MOTION_A).unwrap();
        assert_eq!(v[0], 1.0);
        assert_eq!(v[1], v[2]);
    }

    #[test]
    fn collapse_threshold() {
        // eps 0.35: 2.0/1.0/conf(~0.7) all > eps -> all +1 trits
        let v = vote_vector(&AGENT_MODELS[3], &MOTION_A).unwrap();
        assert_eq!(swarm_k3::collapse(&v, EPS), vec![1, 1, 1]);
    }

    /// The table must carry exactly the platform's council roster, every agent must have domains to vote
    /// on, and every confidence must exceed eps -- below it, a voting agent's own terms collapse to 0 and
    /// it is silently muted. A bare count was what let this table drift one agent and 116 domains behind
    /// the oracle the module claims to mirror.
    #[test]
    fn roster_matches_the_platform() {
        const ROSTER: [&str; 15] = [
            "Shield", "Logic", "Devo", "Seeker", "Verify", "Archi", "Flow", "Bridge", "Orchestra",
            "EventLedger", "LogicMaster", "Pulse", "THzGateway", "Jarvis", "Hermes",
        ];
        let names: Vec<&str> = AGENT_MODELS.iter().map(|a| a.name).collect();
        assert_eq!(
            names.len(),
            ROSTER.len(),
            "the table holds {} agents; the platform roster names {}",
            names.len(),
            ROSTER.len()
        );
        for want in ROSTER {
            assert!(
                names.contains(&want),
                "{want} is on the platform roster but missing from the table"
            );
        }
        for a in AGENT_MODELS.iter() {
            assert!(!a.domains.is_empty(), "{} has no domains and could never vote", a.name);
            assert!(a.confidence > EPS, "{} conf must exceed eps", a.name);
        }
    }
}
