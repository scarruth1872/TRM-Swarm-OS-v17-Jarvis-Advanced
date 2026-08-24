//! TRM Swarm OS - artifact_pipeline (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/artifact_pipeline.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const ARTIFACTS_DIR: &'static str = "swarm_v2_artifacts";
pub const AGENT_SKILLS_DIR: &'static str = ".agent/skills";
pub const INTEGRATION_DIR: &'static str = "swarm_v2_integrated";
pub const PIPELINE_DB: &'static str = "swarm_v2_memory/artifact_pipeline.json";

//  get_artifact_pipeline() 
static SINGLETON_ARTIFACT_PIPELINE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_artifact_pipeline() -> u64 {
    // Singleton handle (lazy-init pattern from artifact_pipeline.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_ARTIFACT_PIPELINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_ARTIFACT_PIPELINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
