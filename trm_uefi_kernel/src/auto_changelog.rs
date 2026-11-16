//! TRM Swarm OS - auto_changelog (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/auto_changelog.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  ChangelogEntry 
#[derive(Debug, Clone)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub changes: Vec<Vec<(String, f64)>>,
}

impl ChangelogEntry {
    pub fn new() -> Self {
        ChangelogEntry {
            version: String::new(),
            date: String::new(),
            changes: vec![],
        }
    }
}

//  ModuleInfo 
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub path: String,
    pub name: String,
    pub category: u64,
    pub hash: String,
    pub last_modified: f64,
    pub status: String,
    pub description: String,
    pub exports: Vec<String>,
}

impl ModuleInfo {
    pub fn new() -> Self {
        ModuleInfo {
            path: String::new(),
            name: String::new(),
            category: 0,
            hash: String::new(),
            last_modified: 0.0,
            status: String::new(),
            description: String::new(),
            exports: vec![],
        }
    }
}

//  get_changelog_engine() 
static SINGLETON_CHANGELOG_ENGINE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_changelog_engine() -> u64 {
    // Singleton handle (lazy-init pattern from auto_changelog.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_CHANGELOG_ENGINE.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_CHANGELOG_ENGINE.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
