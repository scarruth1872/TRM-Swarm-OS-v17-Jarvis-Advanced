//! TRM Swarm OS - persistent_mmap_allocator (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/persistent_mmap_allocator.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const MMAP_MAGIC: u64 = 1414679809;
pub const MMAP_VERSION: u64 = 1;
pub const ARENA_SIZE: u64 = 0;

//  MMapHeader 
#[derive(Debug, Clone)]
pub struct MMapHeader {
    pub magic: i64,
    pub version: i64,
    pub arena_size: i64,
    pub last_checkpoint: f64,
    pub restart_count: i64,
    pub crc32: i64,
    pub reserved: u64,
}

impl MMapHeader {
    pub fn new() -> Self {
        MMapHeader {
            magic: 0,
            version: 0,
            arena_size: 0,
            last_checkpoint: 0.0,
            restart_count: 0,
            crc32: 0,
            reserved: 0,
        }
    }
}
