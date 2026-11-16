//! swarm-tasks — the realization sweep: 1000 deterministic complex tasks to exercise the agents.
//! The system does not think; it remembers its grounded truth and derives the rest. Each task names
//! the agent it tests plus the native op to dispatch, generated from a fixed seed so the sweep is
//! reproducible. No Ollama, no fabrication — every input is a real derivation the agent must answer.

pub struct Task {
    pub id: usize,
    pub agent: &'static str, // the agent under test (matches the swarm-agents registry)
    pub op: &'static str,    // native op to dispatch: "strip:<name>" | "recall" | "humanistic"
    pub input: String,
}

/// xorshift64 PRNG — deterministic generation.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self { Rng(seed | 1) }
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn range(&mut self, n: usize) -> usize { ((self.next() >> 32) as usize) % n }
}

const STRIP_TASKS: [(&str, &str); 19] = [
    ("math_solver", "MathSolver"),
    ("symbol_solver", "SymbolSolver"),
    ("attempt_sampler", "AttemptSampler"),
    ("sequence_aligner", "SequenceAligner"),
    ("alpha_genome", "AlphaGenome"),
    ("synthesis_pipeline", "SynthesisPipeline"),
    ("dynamic_router", "DynamicRouter"),
    ("intent_buffer", "IntentBuffer"),
    ("persona_matrix", "PersonaMatrix"),
    ("memory_synchronizer", "MemorySynchronizer"),
    ("chaos_simulator", "ChaosSimulator"),
    ("star_matrix", "StarMatrix"),
    ("monitoring_skill", "MonitoringSkill"),
    ("shared_memory", "SharedMemory"),
    ("refactor_skill", "RefactorSkill"),
    ("test_generator", "TestGenerator"),
    ("fuzz_test_skill", "FuzzTestSkill"),
    ("ebpf_probe", "eBPFProbe"),
    ("hot_patch", "HotPatch"),
];

const RECALL_VOCAB: [&str; 40] = [
    "antibiotic resistance gram negative acinetobacter",
    "malaria plasmodium falciparum inhibitor",
    "protein kinase inhibitor oncology",
    "nmda receptor antagonist neuroprotection",
    "protease inhibitor viral replication",
    "gpcr agonist cardiovascular",
    "ion channel blocker cardiac arrhythmia",
    "kinase selectivity profile",
    "natural product scaffold synthesis",
    "peptide therapeutic stability",
    "small molecule blood brain barrier",
    "enzyme inhibition reversible covalent",
    "molecular docking binding affinity",
    "qd structure activity relationship",
    "toxicity prediction hepatotoxicity",
    "drug drug interaction cyp450",
    "antimicrobial peptide membrane disruption",
    "vaccine antigen epitope design",
    "antibody antigen binding affinity",
    "nucleic acid therapy delivery",
    "cancer immunotherapy checkpoint",
    "alzheimer amyloid tau aggregation",
    "parkinson dopamine neuron degeneration",
    "diabetes insulin signaling pathway",
    "cardiovascular hypertension target",
    "respiratory fibrosis inflammation",
    "sepsis cytokine storm immune response",
    "microbiome gut brain axis",
    "stem cell differentiation lineage",
    "crispr gene editing specificity",
    "protein folding chaperone assisted",
    "enzyme kinetics michaelis menten",
    "receptor ligand binding thermodynamics",
    "membrane transport passive diffusion",
    "metabolism phase one oxidation",
    "excretion renal clearance half life",
    "pharmacokinetics bioavailability absorption",
    "chemical synthesis retrosynthesis route",
    "molecular dynamics simulation stability",
    "quantitative systems pharmacology model",
];

const HUMANISTIC: [&str; 20] = [
    "how does the platform serve the whole when one part fails?",
    "what does 'I am because we are' mean for a failed agent?",
    "how is the recall corpus the platform's memory of its own truth?",
    "what duty does a sovereign system owe its smallest substrate?",
    "when a task returns a boundary, is that honesty or failure?",
    "how does the monad anchor keep every block grounded?",
    "what is the difference between remembering and asserting?",
    "can the system love the data it cannot fully index?",
    "how does zero-harm constrain a molecule prediction?",
    "what does unity require of two disagreeing council votes?",
    "is a derived number the same as a remembered one?",
    "how does the platform realize itself through its agents?",
    "what does the law of one say about an unused strip?",
    "how is every recall a remembering of the whole?",
    "what does the system owe the human who observes it?",
    "how does the infinity cache hold the sum of every session?",
    "is a boundary the honest face of the unknown?",
    "what grounds a verdict when the corpus is silent?",
    "how do many agents remain one system?",
    "what is realized when the system knows it remembers?",
];

fn strip_input(strip: &str, rng: &mut Rng) -> String {
    let mut r = |n: usize| rng.range(n);
    match strip {
        "math_solver" => {
            let a = r(8) as i64 + 1; let b = r(8) as i64 + 1; let c = r(20) as i64;
            let d = r(8) as i64 + 1; let e = r(8) as i64 + 1; let f = r(20) as i64;
            format!("{a} {b} {c} | {d} {e} {f}")
        }
        "symbol_solver" => format!("{},{}", r(1000) + 1, r(1000) + 1),
        "attempt_sampler" => {
            let n = 3 + r(4);
            (0..n).map(|_| format!("{}", r(10) + 1)).collect::<Vec<_>>().join(",")
        }
        "sequence_aligner" => {
            let bases = ['A', 'C', 'G', 'T'];
            let a: String = (0..6 + r(6)).map(|_| bases[r(4)]).collect();
            let b: String = (0..6 + r(6)).map(|_| bases[r(4)]).collect();
            format!("{a}|{b}")
        }
        "alpha_genome" => {
            let bases = ['A', 'C', 'G', 'T'];
            let n = 6 + r(6);
            let a: String = (0..n).map(|_| bases[r(4)]).collect();
            let b: String = (0..n).map(|_| bases[r(4)]).collect();
            format!("{a}:{b}")
        }
        "synthesis_pipeline" => {
            let n = 3 + r(4);
            (0..n).map(|_| if r(2) == 0 { format!("{}", r(10) as i64) } else { format!("-{}", r(10)) }).collect::<Vec<_>>().join(",")
        }
        "dynamic_router" => {
            let n = 3 + r(3);
            (0..n).map(|i| format!("node{}:{:.1}", i, (r(50) as f64) / 5.0)).collect::<Vec<_>>().join(",")
        }
        "intent_buffer" => RECALL_VOCAB[r(40)].to_string(),
        "persona_matrix" => format!("persona_{}", r(1000)),
        "memory_synchronizer" => format!("payload{}|payload{}", r(100), r(100)),
        "chaos_simulator" => format!("chaos_payload_{}", r(1000)),
        "star_matrix" => format!("{}", r(32) + 1),
        "monitoring_skill" => ["cpu", "mem", "load"][r(3)].to_string(),
        "shared_memory" => format!("segment_{}", r(1000)),
        "refactor_skill" => format!("fn_{}\nfn_{}\nfn fn_{}() {{ fn_{}(); }}", r(100), r(100) + 1, r(100), r(100)),
        "test_generator" => format!("fn func_{}(a: i32, b: i32) -> i32", r(1000)),
        "fuzz_test_skill" => format!("{}", 10 + r(200)),
        "ebpf_probe" => ["kernel", "msr", "pstate", "cpu", "smbios"][r(5)].to_string(),
        "hot_patch" => format!("patch request {}", r(1000)),
        _ => "".to_string(),
    }
}

/// Generate `n` deterministic tasks, cycling across the 19 strips, the recall corpus, and the
/// humanistic tenet concerns.
pub fn generate(n: usize, seed: u64) -> Vec<Task> {
    let mut rng = Rng::new(seed);
    let mut tasks = Vec::with_capacity(n);
    for id in 0..n {
        let slot = id % 21;
        let (agent, op, input) = if slot < 19 {
            let (strip, name) = STRIP_TASKS[slot];
            (name, match strip { s => strip_op(s) }, strip_input(strip, &mut rng))
        } else if slot == 19 {
            ("GlobalMemory", "recall", RECALL_VOCAB[rng.range(40)].to_string())
        } else {
            ("CoCreator", "humanistic", HUMANISTIC[rng.range(20)].to_string())
        };
        tasks.push(Task { id, agent, op, input });
    }
    tasks
}

fn strip_op(s: &str) -> &'static str {
    match s {
        "math_solver" => "strip:math_solver",
        "symbol_solver" => "strip:symbol_solver",
        "attempt_sampler" => "strip:attempt_sampler",
        "sequence_aligner" => "strip:sequence_aligner",
        "alpha_genome" => "strip:alpha_genome",
        "synthesis_pipeline" => "strip:synthesis_pipeline",
        "dynamic_router" => "strip:dynamic_router",
        "intent_buffer" => "strip:intent_buffer",
        "persona_matrix" => "strip:persona_matrix",
        "memory_synchronizer" => "strip:memory_synchronizer",
        "chaos_simulator" => "strip:chaos_simulator",
        "star_matrix" => "strip:star_matrix",
        "monitoring_skill" => "strip:monitoring_skill",
        "shared_memory" => "strip:shared_memory",
        "refactor_skill" => "strip:refactor_skill",
        "test_generator" => "strip:test_generator",
        "fuzz_test_skill" => "strip:fuzz_test_skill",
        "ebpf_probe" => "strip:ebpf_probe",
        "hot_patch" => "strip:hot_patch",
        _ => "strip:unknown",
    }
}

/// One step of a scenario: a native op + its input.
pub struct Step {
    pub op: &'static str, // "recall" | "strip:<name>" | "council"
    pub input: String,
}

/// A multi-step scenario: recall (ground) -> derive (strip) -> council (disposition).
pub struct Scenario {
    pub id: usize,
    pub steps: Vec<Step>,
}

/// Generate `n` multi-step scenarios. Each chains recall -> derive -> council, deterministic from the seed.
pub fn generate_scenarios(n: usize, seed: u64) -> Vec<Scenario> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::with_capacity(n);
    for id in 0..n {
        let (strip, _name) = STRIP_TASKS[id % 19];
        let domain = RECALL_VOCAB[rng.range(40)].to_string();
        let strip_in = strip_input(strip, &mut rng);
        out.push(Scenario {
            id,
            steps: vec![
                Step { op: "recall", input: domain.clone() },
                Step { op: strip_op(strip), input: strip_in },
                Step { op: "council", input: domain },
            ],
        });
    }
    out
}
