//! TRM Swarm OS - skill_loader (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/skill_loader.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const SKILL_SEARCH_PATHS: &'static [&'static str] = &[".agent/skills", ".agents/skills", "_agent/skills", "swarm_v2/skills"];

//  SkillDefinition 
#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub version: String,
    pub instructions: String,
    pub prerequisites: Vec<String>,
    pub tags: Vec<String>,
    pub source_path: String,
    pub source_type: String,
    pub loaded_at: String,
}

impl SkillDefinition {
    pub fn new() -> Self {
        SkillDefinition {
            name: String::new(),
            description: String::new(),
            version: String::new(),
            instructions: String::new(),
            prerequisites: vec![],
            tags: vec![],
            source_path: String::new(),
            source_type: String::new(),
            loaded_at: String::new(),
        }
    }
}

//  get_skill_loader() 
static SINGLETON_SKILL_LOADER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_skill_loader() -> u64 {
    // Singleton handle (lazy-init pattern from skill_loader.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_SKILL_LOADER.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SKILL_LOADER.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
