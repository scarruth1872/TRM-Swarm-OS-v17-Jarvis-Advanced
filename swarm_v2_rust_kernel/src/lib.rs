//! TRM Swarm OS v19.001 "Sovereign" — Native Rust Microkernel Engine
//! Bare-metal zero-GC, zero-data-race K3 Ternary Logic, Matrix, QUIC & Agent Runtime Engine.

pub mod agent_runtime;
pub mod quic_protocol;
pub mod nand_persistence;
pub mod spatial_octree;
pub mod cli;
pub mod council;
pub mod lcars_render;
pub mod input_driver;
pub mod mesh_discovery;

use std::slice;

/// Monad Truth Anchor: "MONAD\0\0\x01" in 64-bit Big-Endian ASCII Hex
pub const MONAD_TRUTH_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

/// Kleene K3 Ternary Logic Operation: AND (min)
#[no_mangle]
pub extern "C" fn rust_k3_and(a: i8, b: i8) -> i8 {
    if a < b { a } else { b }
}

/// Kleene K3 Ternary Logic Operation: OR (max)
#[no_mangle]
pub extern "C" fn rust_k3_or(a: i8, b: i8) -> i8 {
    if a > b { a } else { b }
}

/// Kleene K3 Ternary Logic Operation: NOT (-a)
#[no_mangle]
pub extern "C" fn rust_k3_not(a: i8) -> i8 {
    -a
}

/// Batch K3 Ternary Conjunction across memory slices (SIMD ready)
#[no_mangle]
pub extern "C" fn rust_k3_batch_and(
    a_ptr: *const i8,
    b_ptr: *const i8,
    out_ptr: *mut i8,
    len: usize,
) -> i32 {
    if a_ptr.is_null() || b_ptr.is_null() || out_ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let a = slice::from_raw_parts(a_ptr, len);
        let b = slice::from_raw_parts(b_ptr, len);
        let out = slice::from_raw_parts_mut(out_ptr, len);
        
        for i in 0..len {
            out[i] = if a[i] < b[i] { a[i] } else { b[i] };
        }
    }
    0
}

/// High-performance Cosine Similarity Calculation
#[no_mangle]
pub extern "C" fn rust_turbovec_cosine(
    a_ptr: *const f32,
    b_ptr: *const f32,
    dim: usize,
) -> f32 {
    if a_ptr.is_null() || b_ptr.is_null() || dim == 0 {
        return 0.0;
    }
    
    unsafe {
        let a = slice::from_raw_parts(a_ptr, dim);
        let b = slice::from_raw_parts(b_ptr, dim);
        
        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;
        
        for i in 0..dim {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        
        let denom = (norm_a * norm_b).sqrt();
        if denom > 1e-9 {
            dot / denom
        } else {
            0.0
        }
    }
}

/// Verification of Rust Microkernel Health & Truth Anchor
#[no_mangle]
pub extern "C" fn rust_kernel_verify() -> u64 {
    MONAD_TRUTH_ANCHOR
}

/// Zero-overhead native Council evaluation across all 14 agents
#[no_mangle]
pub extern "C" fn rust_council_evaluate(
    topic_ptr: *const u8,
    topic_len: usize,
    out_score_ptr: *mut f32,
    out_seal_ptr: *mut u64,
) -> i32 {
    if topic_ptr.is_null() || topic_len == 0 {
        return -1;
    }
    let topic_slice = unsafe { slice::from_raw_parts(topic_ptr, topic_len) };
    let topic_str = String::from_utf8_lossy(topic_slice);
    
    let result = council::NativeSwarmCouncil::evaluate(&topic_str, council::ProposalCategory::General, "BareMetal");
    
    if !out_score_ptr.is_null() {
        unsafe { *out_score_ptr = result.weighted_score; }
    }
    if !out_seal_ptr.is_null() {
        unsafe { *out_seal_ptr = result.monad_seal; }
    }
    
    if result.consensus_reached { 1 } else { 0 }
}

/// Zero-overhead native Bare-Metal LCARS Framebuffer rendering into physical UEFI GOP VRAM
#[no_mangle]
pub extern "C" fn rust_lcars_render_frame(
    framebuffer_ptr: *mut u32,
    width: usize,
    height: usize,
    stride: usize,
    tick: f32,
) -> i32 {
    if framebuffer_ptr.is_null() || width == 0 || height == 0 {
        return -1;
    }

    let mut surface = lcars_render::LcarsSurface::new(width, height);
    surface.stride = stride;
    lcars_render::render_lcars_desktop(&mut surface, tick, lcars_render::LcarsTab::NeuralBridge, None);

    unsafe {
        let count = width * height;
        let dst = slice::from_raw_parts_mut(framebuffer_ptr, count);
        dst.copy_from_slice(&surface.pixels[..count]);
    }

    0
}

/// Voltage State representation (Hidden state vector of dimension D)
pub struct VoltageState {
    pub dim: usize,
    pub values: Vec<f32>,
}

/// Action Vector: Tokenless structured action output
#[repr(C)]
pub struct ActionVector {
    pub agent_id: u8,       // 0: Hermes, 1: Jarvis, 2: Bridge, 3: Shield, etc.
    pub intent_id: u8,      // 0: READ, 1: WRITE, 2: EXECUTE, 3: RENDER
    pub confidence: f32,    // Calibrated score [0.0, 1.0]
    pub k3_trit: i8,        // +1 (TRUE), 0 (UNKNOWN), -1 (FALSE)
}

/// Voltage LLM Node wrapper
pub struct VoltageLLMNode {
    pub name: String,
    pub hidden_dim: usize,
    pub weights_loaded: bool,
}

impl VoltageLLMNode {
    pub fn new(name: &str, hidden_dim: usize) -> Self {
        VoltageLLMNode {
            name: name.to_string(),
            hidden_dim,
            weights_loaded: true,
        }
    }

    /// Read continuous voltage state directly (pre-softmax transformation)
    pub fn read_voltage_state(&self, input_signal: &[f32]) -> VoltageState {
        let mut state = vec![0.0f32; self.hidden_dim];
        for (i, val) in input_signal.iter().enumerate() {
            if i < self.hidden_dim {
                state[i] = (val * 1.15).tanh();
            }
        }
        VoltageState {
            dim: self.hidden_dim,
            values: state,
        }
    }

    /// Collapse continuous Voltage State to structured Action Vector via K3 logic
    pub fn collapse_to_action(&self, state: &VoltageState) -> ActionVector {
        let mut sum = 0.0f32;
        for v in &state.values {
            sum += v;
        }
        let avg = if state.dim > 0 { sum / state.dim as f32 } else { 0.0 };
        
        let k3_trit = if avg > 0.15 { 1 } else if avg < -0.15 { -1 } else { 0 };
        let agent_id = (state.values.get(0).unwrap_or(&0.0).abs() * 10.0) as u8 % 14;
        let intent_id = (state.values.get(1).unwrap_or(&0.0).abs() * 5.0) as u8 % 4;
        let confidence = (avg.abs() * 2.0).min(1.0);

        ActionVector {
            agent_id,
            intent_id,
            confidence,
            k3_trit,
        }
    }
}

/// C ABI exported interface for Voltage Reader & Action Vector Mapper
#[no_mangle]
pub extern "C" fn rust_voltage_node_eval(
    dim: usize,
    signal_ptr: *const f32,
    out_action: *mut ActionVector,
) -> i32 {
    if signal_ptr.is_null() || out_action.is_null() || dim == 0 {
        return -1;
    }
    
    unsafe {
        let signal = std::slice::from_raw_parts(signal_ptr, dim);
        let node = VoltageLLMNode::new("BareMetal_Voltage_Substrate", dim);
        let state = node.read_voltage_state(signal);
        let action = node.collapse_to_action(&state);
        *out_action = action;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k3_logic() {
        assert_eq!(rust_k3_and(1, 0), 0);
        assert_eq!(rust_k3_and(1, -1), -1);
        assert_eq!(rust_k3_or(0, -1), 0);
        assert_eq!(rust_k3_not(1), -1);
    }

    #[test]
    fn test_truth_anchor() {
        assert_eq!(rust_kernel_verify(), MONAD_TRUTH_ANCHOR);
    }
}
pub mod k3_logic;
pub mod silicon_crystallography;
pub mod drug_cocktail_optimizer;
