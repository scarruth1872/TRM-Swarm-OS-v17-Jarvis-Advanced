//! TRM Swarm OS - expert_team_harnesses (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/expert_team_harnesses.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const HARNESS_DOMAINS: &'static [(&'static str, i64, &'static [&'static str])] = &[("Security & Cryptography", 0, &["Shield", "VulnScanSkill", "CryptoAuditSkill", "NeuralWall", "DDR_Antibody"]), ("Logic & Mathematical Proof", 0, &["Logic", "MathSolver", "LogicProver", "AttemptSampler", "ConsensusEngine"]), ("Code Engineering & Refactoring", 0, &["Devo", "RefactorSkill", "TestGeneratorSkill", "FuzzTestSkill", "ToolForge"]), ("Telemetry & Memory Synchronization", 0, &["Pulse", "GlobalMemory", "SystemMetricsSkill", "MemorySynchronizer", "MonitoringSkill"]), ("Synthesis & Chain-of-Thought", 0, &["Archi", "CoCreatorSkill", "SynthesisPipeline", "CognitiveOrchestrator", "IntentBuffer"]), ("Autonomous Healing & Resilience", 0, &["SelfHealing", "ChaosSimulator", "Sentinel", "MicrokernelSpawner", "StarMatrix"])];

//  get_harness_manager() 
static SINGLETON_HARNESS_MANAGER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_harness_manager() -> u64 {
    // Singleton handle (lazy-init pattern from expert_team_harnesses.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_HARNESS_MANAGER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_HARNESS_MANAGER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
