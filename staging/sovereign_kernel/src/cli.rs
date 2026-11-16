//! TRM Swarm OS v19.001 — CLI Argument Parser & Subcommand Dispatcher
//! Zero-dependency, high-performance command-line interface parser for trm_swarm_boot.exe.
//! Supports production bare-metal boot, 14-agent Council debate/voting, LCARS GUI rendering, Mesh Discovery, and verification.

use std::env;

#[derive(Debug, PartialEq)]
pub enum Command {
    Boot,
    BareMetal { fast: bool },
    Council { topic: String, category: String, proposer: String },
    CouncilBench { iterations: usize },
    Gui { frames: usize, headless: bool, tab: String },
    Serve,
    Mesh { broadcast: bool },
    Bench { tasks: usize },
    Spatial { points: usize },
    Pharmacophore { target: String, ligand: String, dg: f32 },
    Report { report_type: String },
    P2PRoute { peer: String, port: u16, count: usize },
    Colocation { tasks: usize },
    Unity { target: String, ligand: String, dg: f32 },
    Screen { compounds: usize },
    ScreenLibrary { path: String, poses: usize },
    Ingest { modality: String, file: String },
    MultiModalFusion,
    Federation { mode: String },
    SiliconCrystallography { export_cif: bool },
    ThzGateway,
    Muon { die_area_cm2: f64 },
    GpuSubstrate,
    PhononRealtime { temp_k: f64 },
    Ionosphere { f10_7_sfu: f64, kp: f64 },
    Phoenix { ops: u64, strikes: u64 },
    Reason { gene: String, mutation: String, inhibitor: String },
    QuantumMesh,
    GrandDiscovery,
    PortalServer { port: u16, max_requests: Option<usize> },
    GapAnalysis,
    Synergy { current_antibodies: Option<usize> },
    SynergyDeploy { current_antibodies: Option<usize> },
    SynergyExtended { current_antibodies: Option<usize> },
    AnalogCognition,
    QuarterlySecurity { current_antibodies: Option<usize> },
    DeepConsciousness,
    MultiDieScaling,
    OptimizeCocktail { nodes: Option<usize> },
    UnifiedResonance,
    SramPatterns,
    SramApplications,
    QuadHorizons,
    HybridIntelligence,
    HybridDeployment,
    PlatformSync,
    AutonomousFinetuning,
    RagAndSemanticSearch,
    HardwareL3Switch,
    FractalRecursion,
    PatternSelector,
    PatternFusion,
    FpgaBitstream,
    SolveIsing,
    HdcClassification,
    OnnSync,
    Orchestrate15,
    RealNandWrites,
    SupremacyCampaign,
    KnowledgeFabricCampaign,
    QuantumSecurityCampaign,
    ScientificParserCampaign,
    PhoenixCampaign,
    AutonomyCampaign,
    ConversationMemory,
    DiscoveryCampaign,
    Phase4Execution,
    DeepFrontierResearch,
    InstellaHospitalConsultation,
    BonsaiCreativeDrugSynthesis,
    Trm7mDeterministicComputation,
    BonsaiMultilingualFluency,
    InstellaSovereignCodeSynthesis,
    HospitalAmrSurveillanceDeployment,
    ChagasDrugDiscoveryPipeline,
    ClinicalTrialSafetyMonitoring,
    FederationSramOrchestration,
    OllamaKnowledgeHarvest,
    TurboVecTernaryFusion,
    VoltageDoctrineHarmonize,
    LargeModelTurboVecQuantize,
    AmrCrisisResponse,
    ZeroDayExploitDefense,
    RuralTelemedicineDeployment,
    MarsBiosphereOptimization,
    PostQuantumFinancialSettlement,
    UniversalModelIngestionValidate,
    CouncilMultiIntelligenceDeliberate,
    CyberGymExploitChallenge,
    SweMarathonChallenge,
    VoltageIntelligenceExceedance,
    VoltageRawReasoning,
    AuraEngine { code: Option<String> },
    DdrScan { payload: Option<String> },
    MonadJournal { title: Option<String> },
    MitosisSpawn { role: Option<String> },
    Verify,
    VerifyProofs,
    SecurityAudit,
    PortedModules,
    Emanate,
    Help,
}

pub struct CLIConfig {
    pub command: Command,
}

impl CLIConfig {
    pub fn parse_args() -> Self {
        let args: Vec<String> = env::args().collect();
        if args.len() <= 1 {
            // Boot handoff default: PERSISTENT serve mode (no verify-and-exit).
            return CLIConfig { command: Command::Serve };
        }

        let cmd_str = args[1].to_lowercase();
        let command = match cmd_str.as_str() {
            "boot" => Command::Boot,
            "serve" | "resident" | "daemon" | "platform" => Command::Serve,
            "baremetal" | "bare-metal" | "production" => {
                let fast = args.iter().any(|a| {
                    let lower = a.to_lowercase();
                    lower == "--fast" || lower == "--production" || lower == "-f"
                });
                Command::BareMetal { fast }
            }
            "gui" | "desktop" | "lcars" | "compositor" | "jarvis" => {
                let mut frames = 60;
                let mut tab = "jarvis".to_string();
                let headless = args.iter().any(|a| a == "--headless" || a == "-h");
                for i in 2..args.len() {
                    if (args[i] == "--frames" || args[i] == "-n") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            frames = parsed;
                        }
                    } else if (args[i] == "--tab" || args[i] == "-t") && i + 1 < args.len() {
                        tab = args[i + 1].to_lowercase();
                    }
                }
                Command::Gui { frames, headless, tab }
            }
            "mesh" | "peers" => {
                let broadcast = !args.iter().any(|a| a == "--listen-only");
                Command::Mesh { broadcast }
            }
            "council" | "debate" | "vote" => {
                let mut topic = "Sovereign System State Mutation Protocol".to_string();
                let mut category = "general".to_string();
                let mut proposer = "Shawn (Operator)".to_string();

                let mut i = 2;
                while i < args.len() {
                    let arg = &args[i];
                    if (arg == "--topic" || arg == "-t") && i + 1 < args.len() {
                        topic = args[i + 1].clone();
                        i += 2;
                    } else if (arg == "--category" || arg == "-c") && i + 1 < args.len() {
                        category = args[i + 1].clone();
                        i += 2;
                    } else if (arg == "--proposer" || arg == "-p") && i + 1 < args.len() {
                        proposer = args[i + 1].clone();
                        i += 2;
                    } else if arg == "--bench" || arg == "bench" {
                        let iterations = if i + 1 < args.len() {
                            args[i + 1].parse::<usize>().unwrap_or(10_000)
                        } else {
                            10_000
                        };
                        return CLIConfig { command: Command::CouncilBench { iterations } };
                    } else {
                        if !arg.starts_with('-') && topic == "Sovereign System State Mutation Protocol" {
                            topic = arg.clone();
                        }
                        i += 1;
                    }
                }
                Command::Council { topic, category, proposer }
            }
            "bench" | "benchmark" => {
                let mut tasks = 1_000_000;
                for i in 2..args.len() {
                    if (args[i] == "--tasks" || args[i] == "-n") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            tasks = parsed;
                        }
                    }
                }
                Command::Bench { tasks }
            }
            "spatial" | "octree" => {
                let mut points = 100_000;
                for i in 2..args.len() {
                    if (args[i] == "--points" || args[i] == "-p") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            points = parsed;
                        }
                    }
                }
                Command::Spatial { points }
            }
            "pharmacophore" | "pharma" | "dock" => {
                let mut target = "3P3G_LpxC".to_string();
                let mut ligand = "TRM-ABX-082".to_string();
                let mut dg = -10.42f32;
                let mut i = 2;
                while i < args.len() {
                    let arg = &args[i];
                    if (arg == "--target" || arg == "-t") && i + 1 < args.len() {
                        target = args[i + 1].clone();
                        i += 2;
                    } else if (arg == "--ligand" || arg == "-l") && i + 1 < args.len() {
                        ligand = args[i + 1].clone();
                        i += 2;
                    } else if (arg == "--dg" || arg == "-d") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<f32>() {
                            dg = parsed;
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                Command::Pharmacophore { target, ligand, dg }
            }
            "report" | "brief" | "summary" => {
                let report_type = if args.len() > 2 {
                    args[2].to_lowercase()
                } else {
                    "pharma".to_string()
                };
                Command::Report { report_type }
            }
            "p2p" | "direct" | "route" => {
                let mut peer = "127.0.0.1".to_string();
                let mut port = 8091u16;
                let mut count = 5usize;
                let mut i = 2;
                while i < args.len() {
                    let arg = &args[i];
                    if (arg == "--peer" || arg == "-p") && i + 1 < args.len() {
                        peer = args[i + 1].clone();
                        i += 2;
                    } else if (arg == "--port" || arg == "-P") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<u16>() {
                            port = parsed;
                        }
                        i += 2;
                    } else if (arg == "--count" || arg == "-n") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            count = parsed;
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                Command::P2PRoute { peer, port, count }
            }
            "colocation" | "shm" | "ipc" => {
                let mut tasks = 1_000usize;
                for i in 2..args.len() {
                    if (args[i] == "--tasks" || args[i] == "-n") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            tasks = parsed;
                        }
                    }
                }
                Command::Colocation { tasks }
            }
            "unity" | "pipeline" | "sovereign" => {
                let mut target = "3P3G_LpxC".to_string();
                let mut ligand = "TRM-ABX-082".to_string();
                let mut dg = -10.42f32;
                let mut i = 2;
                while i < args.len() {
                    let arg = &args[i];
                    if (arg == "--target" || arg == "-t") && i + 1 < args.len() {
                        target = args[i + 1].clone();
                        i += 2;
                    } else if (arg == "--ligand" || arg == "-l") && i + 1 < args.len() {
                        ligand = args[i + 1].clone();
                        i += 2;
                    } else if (arg == "--dg" || arg == "-d") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<f32>() {
                            dg = parsed;
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                Command::Unity { target, ligand, dg }
            }
            "screen-library" | "library" => {
                let mut path = String::new();
                let mut poses = 8usize;
                for i in 2..args.len() {
                    if (args[i] == "--file" || args[i] == "-f") && i + 1 < args.len() {
                        path = args[i + 1].clone();
                    } else if (args[i] == "--poses" || args[i] == "-p") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            poses = parsed;
                        }
                    }
                }
                Command::ScreenLibrary { path, poses }
            }
            "screen" | "campaign" | "docking" => {
                let mut compounds = 50_000usize;
                for i in 2..args.len() {
                    if (args[i] == "--compounds" || args[i] == "-n") && i + 1 < args.len() {
                        if let Ok(parsed) = args[i + 1].parse::<usize>() {
                            compounds = parsed;
                        }
                    }
                }
                Command::Screen { compounds }
            }
            "ingest" | "parse" | "wetlab" => {
                let modality = if args.len() > 2 { args[2].to_lowercase() } else { "help".to_string() };
                let file = if args.len() > 3 { args[3].clone() } else { String::new() };
                Command::Ingest { modality, file }
            }
            "pipeline" | "cross-resonance" | "fusion" | "multimodal" => Command::MultiModalFusion,
            "federation" | "cluster" | "nodes" | "broadcast-pipeline" => {
                let mode = if args.len() > 2 { args[2].to_lowercase() } else { "broadcast".to_string() };
                Command::Federation { mode }
            }
            "silicon" | "crystallography" | "phonon" | "lattice" => {
                let export_cif = args.iter().any(|a| a == "--cif" || a == "-c" || a == "--export");
                Command::SiliconCrystallography { export_cif }
            }
            "thz" | "thz-gateway" | "gateway" | "polariton" => Command::ThzGateway,
            "muon" | "muons" | "cosmic" | "cosmic-ray" => {
                let die_area = if args.len() > 2 { args[2].parse::<f64>().unwrap_or(1.80) } else { 1.80 };
                Command::Muon { die_area_cm2: die_area }
            }
            "gpu" | "rdna2" | "gpu-substrate" => Command::GpuSubstrate,
            "phonon-map" | "phonon-realtime" | "phonons" => {
                let temp = if args.len() > 2 { args[2].parse::<f64>().unwrap_or(335.0) } else { 335.0 };
                Command::PhononRealtime { temp_k: temp }
            }
            "ionosphere" | "hf-mesh" | "hf" | "skywave" => {
                let sfu = if args.len() > 2 { args[2].parse::<f64>().unwrap_or(135.0) } else { 135.0 };
                let kp = if args.len() > 3 { args[3].parse::<f64>().unwrap_or(2.3) } else { 2.3 };
                Command::Ionosphere { f10_7_sfu: sfu, kp }
            }
            "phoenix" | "resurrection" | "infinite" | "zero-loss" => {
                let ops = if args.len() > 2 { args[2].parse::<u64>().unwrap_or(1_000_000) } else { 1_000_000 };
                let strikes = if args.len() > 3 { args[3].parse::<u64>().unwrap_or(10_000) } else { 10_000 };
                Command::Phoenix { ops, strikes }
            }
            "reason" | "cross-modality" | "infer" | "mutation" => {
                let gene = if args.len() > 2 { args[2].clone() } else { "lpxC".to_string() };
                let mutation = if args.len() > 3 { args[3].clone() } else { "A215V".to_string() };
                let inhibitor = if args.len() > 4 { args[4].clone() } else { "TRM-ABX-220".to_string() };
                Command::Reason { gene, mutation, inhibitor }
            }
            "quantum" | "quantum-mesh" | "kyber" | "dilithium" | "pq" => Command::QuantumMesh,
            "discovery" | "grand-discovery" | "10b" => Command::GrandDiscovery,
            "portal" | "server" | "api" | "public-portal" | "rest" => {
                let port = if args.len() > 2 {
                    args[2].parse::<u16>().unwrap_or(8030)
                } else {
                    8030
                };
                let max_requests = if args.len() > 3 {
                    args[3].parse::<usize>().ok()
                } else {
                    None
                };
                Command::PortalServer { port, max_requests }
            }
            "gap-analysis" | "gaps" | "knowledge-gaps" | "remediation" => Command::GapAnalysis,
            "synergy" | "synergy-engine" | "threat-response" | "immune" => {
                let current_antibodies = if args.len() > 2 {
                    args[2].parse::<usize>().ok()
                } else {
                    None
                };
                Command::Synergy { current_antibodies }
            }
            "synergy-deploy" | "synergy-100" | "attack-sim" | "deploy-synergy" => {
                let current_antibodies = if args.len() > 2 {
                    args[2].parse::<usize>().ok()
                } else {
                    None
                };
                Command::SynergyDeploy { current_antibodies }
            }
            "synergy-extended" | "synergy-p103" | "extended-tests" | "stress-suite" => {
                let current_antibodies = if args.len() > 2 {
                    args[2].parse::<usize>().ok()
                } else {
                    None
                };
                Command::SynergyExtended { current_antibodies }
            }
            "analog" | "analog-cognition" | "mit-theory" | "mit-analog" | "field-cognition" => Command::AnalogCognition,
            "quarterly-security" | "q3-security" | "paper-104" | "threat-landscape" => {
                let current_antibodies = if args.len() > 2 {
                    args[2].parse::<usize>().ok()
                } else {
                    None
                };
                Command::QuarterlySecurity { current_antibodies }
            }
            "consciousness" | "deep-consciousness" | "paper-105" | "analog-demo" => Command::DeepConsciousness,
            "multi-die" | "multi-die-scaling" | "paper-107" | "planetary-cognition" => Command::MultiDieScaling,
            "optimize-cocktail" | "cocktail-optimizer" | "drug-cocktail" | "scenario-10" => {
                let nodes = if args.len() > 2 {
                    args[2].parse::<usize>().ok()
                } else {
                    None
                };
                Command::OptimizeCocktail { nodes }
            }
            "unified-resonance" | "resonance" | "voltage-quantum-k3" | "paper-109" => Command::UnifiedResonance,
            "sram-patterns" | "sram" | "15-sram" | "paper-110" => Command::SramPatterns,
            "sram-apps" | "sram-applications" | "paper-111" | "production-sram" => Command::SramApplications,
            "quad-horizons" | "explore-all" | "horizons" | "paper-112" => Command::QuadHorizons,
            "hybrid-intelligence" | "on-device-moe" | "bonsai-instella" | "paper-113" => Command::HybridIntelligence,
            "hybrid-deploy" | "federation-hybrid" | "paper-114" | "hybrid-100" => Command::HybridDeployment,
            "platform-sync" | "sync" | "upgrade-all" | "full-sync" | "paper-115" => Command::PlatformSync,
            "autonomous-finetuning" | "finetune" | "finetuning-loop" | "ddr-finetuning" | "paper-116" => Command::AutonomousFinetuning,
            "rag-search" | "rag" | "semantic-search" | "paper-117" => Command::RagAndSemanticSearch,
            "hardware-l3-switch" | "l3-switch" | "real-sram" | "hardware-sram" | "paper-118" => Command::HardwareL3Switch,
            "fractal-recursion" | "fractal-patterns" | "3375-patterns" | "paper-119" => Command::FractalRecursion,
            "pattern-selector" | "auto-select" | "workload-benchmark" | "paper-120" => Command::PatternSelector,
            "pattern-fusion" | "hybrid-fusion" | "225-fusions" | "paper-121" => Command::PatternFusion,
            "fpga-bitstream" | "fpga-lite" | "bitstream" | "paper-122" => Command::FpgaBitstream,
            "ising-spin" | "ising" | "spin-glass" | "paper-123" => Command::SolveIsing,
            "hdc-classification" | "hdc" | "10k-hdc" | "paper-124" => Command::HdcClassification,
            "onn-sync" | "kuramoto" | "phase-lock" | "paper-125" => Command::OnnSync,
            "orchestrate-15" | "orchestrator" | "15-pipeline" | "multi-pattern" | "paper-126" => Command::Orchestrate15,
            "real-nand" | "nand-writes" | "nand-fabric" | "170k-writes" | "paper-127" => Command::RealNandWrites,
            "supremacy-campaign" | "tasks-81-90" | "hardware-supremacy" | "benchmarks-81-90" | "paper-128" => Command::SupremacyCampaign,
            "fabric-campaign" | "tasks-42-50" | "knowledge-fabric" | "turbovec-campaign" | "paper-129" => Command::KnowledgeFabricCampaign,
            "quantum-campaign" | "tasks-51-60" | "quantum-security" | "kyber-campaign" | "paper-130" => Command::QuantumSecurityCampaign,
            "parser-campaign" | "tasks-61-70" | "scientific-parsers" | "parsers-evolution" | "paper-131" => Command::ScientificParserCampaign,
            "phoenix-campaign" | "tasks-71-80" | "resilience-evolution" | "phoenix-resilience" | "paper-132" => Command::PhoenixCampaign,
            "autonomy-campaign" | "tasks-91-100" | "self-evolution" | "recursive-improvement" | "paper-133" => Command::AutonomyCampaign,
            "conversation-memory" | "dialogue-memory" | "nand-dialogue" | "k3-conversation" | "paper-134" => Command::ConversationMemory,
            "discovery-campaign" | "tasks-11-20" | "drug-discovery" | "eskape-screen" | "paper-135" => Command::DiscoveryCampaign,
            "phase-4" | "phase4" | "memory-synthesis" | "frontier-prep" | "council-deliberation" | "paper-136" => Command::Phase4Execution,
            "phase-1" | "phase1" | "frontier-research" | "deep-frontier" | "virtual-screening" | "paper-137" => Command::DeepFrontierResearch,
            "scenario-1" | "instella-consult" | "hospital-consult" | "ed-consult" | "paper-138" => Command::InstellaHospitalConsultation,
            "scenario-2" | "bonsai-synthesis" | "drug-design" | "creative-synthesis" | "paper-139" => Command::BonsaiCreativeDrugSynthesis,
            "scenario-6" | "trm-7m" | "deterministic" | "deterministic-computation" | "paper-140" => Command::Trm7mDeterministicComputation,
            "scenario-5" | "multilingual" | "translation" | "polyglot" | "paper-141" => Command::BonsaiMultilingualFluency,
            "scenario-4" | "scenario4" | "code-synthesis" | "sovereign-code" | "paper-142" => Command::InstellaSovereignCodeSynthesis,
            "scenario-a" | "scenarioa" | "amr-node" | "amr-surveillance" | "paper-143" => Command::HospitalAmrSurveillanceDeployment,
            "scenario-b" | "scenariob" | "chagas-discovery" | "ntd-pipeline" | "paper-144" => Command::ChagasDrugDiscoveryPipeline,
            "scenario-c" | "scenarioc" | "clinical-trial" | "safety-monitor" | "paper-145" => Command::ClinicalTrialSafetyMonitoring,
            "federation-orchestrate" | "federation-sram" | "orchestra-mesh" | "100-sram" | "paper-146" => Command::FederationSramOrchestration,
            "harvest-ollama" | "ollama-harvest" | "mine-ollama" | "absorb-models" | "paper-147" => Command::OllamaKnowledgeHarvest,
            "turbovec-k3-fusion" | "turbovec-fusion" | "ternary-vector" | "turbovec-k3" | "paper-148" => Command::TurboVecTernaryFusion,
            "voltage-doctrine" | "voltage" | "phi-v" | "voltage-harmonize" | "paper-149" => Command::VoltageDoctrineHarmonize,
            "quantize-weights" | "turbovec-weights" | "large-models" | "quantize-models" | "paper-150" => Command::LargeModelTurboVecQuantize,
            "amr-crisis" | "crisis-1" | "scenario-1-amr" | "sepsis-outbreak" | "paper-151" => Command::AmrCrisisResponse,
            "zero-day" | "zero-day-defense" | "quic-exploit" | "scenario-2-defense" | "paper-152" => Command::ZeroDayExploitDefense,
            "rural-kisumu" | "kisumu" | "rural-telemed" | "scenario-3-kisumu" | "paper-153" => Command::RuralTelemedicineDeployment,
            "mars-biosphere" | "mars" | "mars-colony" | "scenario-4-mars" | "paper-154" => Command::MarsBiosphereOptimization,
            "pqc-settlement" | "settlement" | "financial-settlement" | "scenario-5-settlement" | "paper-155" => Command::PostQuantumFinancialSettlement,
            "validate-universal" | "universal-models" | "model-ingestion" | "benchmark-paradigms" | "paper-156" => Command::UniversalModelIngestionValidate,
            "council-deliberate" | "council-test" | "stress-test-models" | "multi-intelligence" | "paper-157" => Command::CouncilMultiIntelligenceDeliberate,
            "cybergym" | "cybergym-challenge" | "gpt-5.6" | "exploit-challenge" | "paper-158" => Command::CyberGymExploitChallenge,
            "swe-marathon" | "swe" | "deepseek-v4" | "swe-bench" | "paper-159" => Command::SweMarathonChallenge,
            "voltage-exceed" | "voltage-intelligence" | "exceed-llm" | "analog-intelligence" | "paper-160" => Command::VoltageIntelligenceExceedance,
            "raw-reasoning" | "voltage-reasoning" | "terminal-bench" | "reasoning-exceed" | "paper-161" => Command::VoltageRawReasoning,
            "aura" | "aura-engine" | "wavefunction" => {
                let code = if args.len() > 2 { Some(args[2..].join(" ")) } else { None };
                Command::AuraEngine { code }
            }
            "ddr-scan" | "antibody-vault" | "ddr-vault" => {
                let payload = if args.len() > 2 { Some(args[2..].join(" ")) } else { None };
                Command::DdrScan { payload }
            }
            "monad-journal" | "journal" | "publish-journal" => {
                let title = if args.len() > 2 { Some(args[2..].join(" ")) } else { None };
                Command::MonadJournal { title }
            }
            "mitosis" | "mitosis-spawn" | "subagent-spawn" => {
                let role = if args.len() > 2 { Some(args[2].clone()) } else { None };
                Command::MitosisSpawn { role }
            }
            "verify" | "anchor" => Command::Verify,
            "ported-modules" | "ported" | "dispatch-ports" => Command::PortedModules,
            "emanate" | "monad-emanation" | "facets" | "emanation" => Command::Emanate,
            "verify-proofs" | "proofs" | "formal" => Command::VerifyProofs,
            "audit-security" | "security" | "ddr" => Command::SecurityAudit,
            "help" | "--help" | "-h" => Command::Help,
            // A typo used to fall through to Command::Boot: `trm_swarm_boot solve-ising` (the real
            // alias is `ising`) ran a full cold boot, printed a clean diagnostic sweep and exited 0,
            // so a wrong command was indistinguishable from a working one. Exit non-zero and say so.
            other => {
                eprintln!(
                    "error: unknown subcommand '{}'\n       run `trm_swarm_boot help` for the list",
                    other
                );
                std::process::exit(2);
            }
        };

        CLIConfig { command }
    }

    pub fn print_help() {
        println!("===========================================================================");
        println!("  TRM SWARM OS v19.001 \"Sovereign\" — CLI TOOL & KERNEL BOOT");
        println!("===========================================================================");
        println!("Usage: trm_swarm_boot [SUBCOMMAND] [OPTIONS]\n");
        println!("SUBCOMMANDS:");
        println!("  boot                           Default cold-boot sequence & 1M diagnostic sweep");
        println!("  baremetal [--fast]             Production bare-metal boot (fast 14-agent health check)");
        println!("  gui [--frames N] [--headless]  Execute native Rust LCARS Framebuffer Compositor (120+ FPS)");
        println!("  mesh [--listen-only]           Broadcast & probe sovereign UDP peer mesh discovery");
        println!("  council [--topic T] [-c CAT]   Execute pure Rust 14-agent Council deliberation & K3 vote");
        println!("  council --bench [N]            Run high-throughput Council consensus benchmark");
        println!("  bench [--tasks N]              Execute Rayon multi-threaded task benchmark (default N=1,000,000)");
        println!("  spatial [--points N]           Run 3D Octree spatial indexing benchmark (default N=100,000)");
        println!("  verify                         Verify Monad Truth Anchor (0x4D4F4E4144000001)");
        println!("  help                           Print this operational CLI guide\n");
        println!("OPTIONS:");
        println!("  --topic, -t <STRING>           Proposal topic for Council debate");
        println!("  --category, -c <CAT>           Category: security, architecture, code, data, comms, health, infra, protocol");
        println!("  --proposer, -p <STRING>        Entity initiating the proposal");
        println!("  --frames, -n <NUM>             Number of frames to render in GUI test (default 60)");
        println!("  --headless, -h                 Run GUI compositor in headless test mode");
        println!("  --fast, -f                     Skip diagnostic sweep (production mode, ~1ms boot)");
        println!("  --production                   Alias for --fast\n");
        println!("EXAMPLES:");
        println!("  trm_swarm_boot gui --frames 120");
        println!("  trm_swarm_boot mesh");
        println!("  trm_swarm_boot council --topic \"Enable Zero-RTT WiFi Mesh\" -c infra");
        println!("  trm_swarm_boot baremetal --fast");
        println!("===========================================================================");
    }
}
