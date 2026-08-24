//! TRM Swarm OS - kernel_lora_adapter (mechanical no_std port, TRM platform engine).
//! Constants and structure extracted 1:1 from swarm_v2/core/kernel_lora_adapter.py.
//! Functions marked // MANUAL need human verification of math semantics.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;


//  CONSTANTS 
pub const LORA_MAGIC: u64 = 1280266817;
pub const FLOAT32: u64 = 4;
pub const HEADER_SIZE: u64 = 72;

//  LoRAHeader 
#[derive(Debug, Clone)]
pub struct LoRAHeader {
    pub magic: i64,
    pub version: i64,
    pub adapter_id: i64,
    pub agent_name: u64,
    pub rank_r: i64,
    pub input_dim_d: i64,
    pub output_dim_k: i64,
    pub alpha: f64,
    pub matrix_a_offset: i64,
    pub matrix_b_offset: i64,
    pub total_size: i64,
    pub checksum: i64,
    pub reserved: u64,
}

impl LoRAHeader {
    pub fn new() -> Self {
        LoRAHeader {
            magic: 0,
            version: 0,
            adapter_id: 0,
            agent_name: 0,
            rank_r: 0,
            input_dim_d: 0,
            output_dim_k: 0,
            alpha: 0.0,
            matrix_a_offset: 0,
            matrix_b_offset: 0,
            total_size: 0,
            checksum: 0,
            reserved: 0,
        }
    }
}

//  get_lora_registry() 
static SINGLETON_LORA_REGISTRY: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub fn get_lora_registry() -> u64 {
    // Singleton handle (lazy-init pattern from kernel_lora_adapter.py).
    // Returns 1 once configured; 0 until first call initializes.
    let v = SINGLETON_LORA_REGISTRY.load(core::sync::atomic::Ordering::Relaxed);
    if v == 0 {
        SINGLETON_LORA_REGISTRY.store(1, core::sync::atomic::Ordering::Relaxed);
        1
    } else {
        v
    }
}
