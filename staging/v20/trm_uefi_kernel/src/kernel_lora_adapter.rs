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

// ── LoRA low-rank delta (kernel_lora_adapter.py:44-58, `native_lora_apply` fallback) ────────────
// `(B @ (A @ x)) * scaling` with A ∈ R^(r×d), B ∈ R^(k×r), x ∈ R^d, all f32 row-major. The oracle
// reaches this same expression as the DLL-free fallback (`(mat_b @ (mat_a @ vec_x)) * scaling`).

/// `(B @ (A @ x)) * scaling`: the LoRA delta projector, dimensioned by the caller.
pub fn lora_apply(
    a: &[f32],
    b: &[f32],
    x: &[f32],
    d: usize,
    k: usize,
    r: usize,
    scaling: f32,
) -> Vec<f32> {
    let mut tmp: Vec<f32> = vec![0.0; r];
    for i in 0..r {
        let mut s = 0.0f32;
        for j in 0..d {
            s += a[i * d + j] * x[j];
        }
        tmp[i] = s;
    }
    let mut out: Vec<f32> = vec![0.0; k];
    for i in 0..k {
        let mut s = 0.0f32;
        for j in 0..r {
            s += b[i * r + j] * tmp[j];
        }
        out[i] = s * scaling;
    }
    out
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
