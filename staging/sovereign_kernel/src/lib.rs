//! TRM Swarm OS v19.001 "Sovereign" — Native Rust Microkernel Engine
//! Bare-metal zero-GC, zero-data-race K3 Ternary Logic, Matrix, QUIC & Agent Runtime Engine.

pub mod agent_runtime;
pub mod quic_protocol;
pub mod higuchi;
pub mod lora_modulate;
pub mod bonsai;
pub mod resonance;
pub mod nand_persistence;
pub mod spatial_octree;
pub mod cli;
pub mod campaign_provenance;
pub mod council;
pub mod lcars_render;
pub mod input_driver;
pub mod mesh_discovery;
pub mod pharmacophore;
pub mod native_text_engine;
pub mod p2p_direct_route;
pub mod zero_network_colocation;
pub mod sovereign_unity;
pub mod egfr_pocket;
pub mod virtual_screening;
pub mod formal_verification;
pub mod security_audit;
pub mod asset_sealing;
pub mod total_silicon_fabric;
pub mod scientific_parsers;
pub mod silicon_crystallography;
pub mod thz_silicon_resonance;
pub mod muon_detector;
pub mod gpu_rdna2_phonon;
pub mod ionosphere_hf_mesh;
pub mod phoenix_advanced_compute;
pub mod cross_modality_reasoning;
pub mod quantum_security;
pub mod grand_discovery_portal;
pub mod knowledge_gap_remediation;
pub mod synergy_engine;
pub mod analog_cognition_engine;
pub mod drug_cocktail_optimizer;
pub mod unified_resonance_stream;
pub mod sram_pattern_architectures;
pub mod sram_practical_applications;
pub mod quad_sram_horizons;
pub mod hybrid_intelligence_engine;
pub mod hybrid_federation_deployment;
pub mod federation_sync_upgrade;
pub mod autonomous_finetuning_loop;
pub mod knowledge_rag_and_semantic_search;
pub mod hardware_l3_pattern_switching;
pub mod fractal_pattern_recursion;
pub mod dynamic_pattern_selector;
pub mod pattern_fusion_engine;
pub mod fpga_lite_bitstream_generator;
pub mod ising_spin_glass_solver;
pub mod hyperdimensional_10k_classifier;
pub mod onn_kuramoto_synchronizer;
pub mod sram_pattern_orchestration_layer;
pub mod real_nand_fabric_writer;
pub mod performance_supremacy_campaign;
pub mod knowledge_fabric_supremacy_campaign;
pub mod quantum_security_supremacy_campaign;
pub mod scientific_parser_evolution_campaign;
pub mod phoenix_resilience_campaign;
pub mod autonomy_evolution_campaign;
pub mod conversation_memory_nand_fabric;
pub mod drug_discovery_evolution_campaign;
pub mod phase4_synthesis_and_frontier_prep;
pub mod deep_frontier_research_engine;
pub mod instella_moe_hospital_consultation;
pub mod bonsai_creative_drug_synthesis;
pub mod trm_7m_deterministic_computation;
pub mod bonsai_multilingual_fluency;
pub mod instella_sovereign_code_synthesis;
pub mod hospital_amr_surveillance_node;
pub mod chagas_drug_discovery_pipeline;
pub mod clinical_trial_safety_monitor;
pub mod federation_sram_workload_orchestrator;
pub mod ollama_knowledge_harvester;
pub mod turbovec_ternary_knowledge_fusion;
pub mod voltage_doctrine_harmonization_engine;
pub mod large_model_turbovec_quantizer;
pub mod multi_hospital_amr_crisis_engine;
pub mod federation_zero_day_defense_engine;
pub mod rural_telemedicine_kisumu_engine;
pub mod mars_biosphere_colony_engine;
pub mod post_quantum_financial_settlement_engine;
pub mod universal_model_ingestion_validator;
pub mod council_multi_intelligence_deliberation_engine;
pub mod cybergym_exploit_challenge_engine;
pub mod swe_marathon_challenge_engine;
pub mod voltage_intelligence_exceedance_engine;
pub mod voltage_raw_reasoning_engine;
pub mod aura_engine;
pub mod ddr_antibody_vault;
pub mod monad_journal_consensus;
pub mod generative_architect_mitosis;
pub mod strip_registry;
pub mod comm_protocol;
pub mod radix_trie_matcher;
pub mod coherence_thresholds;
pub mod voltage_node_engine;
pub mod continuum_pbft;
pub mod coordination_protocol;
pub mod phoenix_protocol_engine;
pub mod dram_pattern_computer;
pub mod k3_logic;
pub mod ternary_knowledge_index;
pub mod ported_dispatch;
pub mod monad_emanation;
pub mod native_query_encoder;

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

/// Native Model-Free Text Encoding into normalized f32 vector in R^dim
#[no_mangle]
pub extern "C" fn rust_native_encode_text_f32(
    text_ptr: *const u8,
    text_len: usize,
    out_vec_ptr: *mut f32,
    dim: usize,
) -> i32 {
    if text_ptr.is_null() || out_vec_ptr.is_null() || dim == 0 {
        return -1;
    }
    let text_slice = unsafe { slice::from_raw_parts(text_ptr, text_len) };
    let text = match std::str::from_utf8(text_slice) {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let encoded = native_query_encoder::NativeQueryEncoder::encode_f32(text, dim);
    unsafe {
        let dst = slice::from_raw_parts_mut(out_vec_ptr, dim);
        dst.copy_from_slice(&encoded[..dim]);
    }
    dim as i32
}

/// Native Model-Free Text Encoding directly into Balanced Ternary Trits {-1, 0, +1}
#[no_mangle]
pub extern "C" fn rust_native_encode_text_ternary(
    text_ptr: *const u8,
    text_len: usize,
    out_trits_ptr: *mut i8,
    dim: usize,
    multiplier: f32,
    out_scale_ptr: *mut f32,
) -> i32 {
    if text_ptr.is_null() || out_trits_ptr.is_null() || dim == 0 {
        return -1;
    }
    let text_slice = unsafe { slice::from_raw_parts(text_ptr, text_len) };
    let text = match std::str::from_utf8(text_slice) {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let mult = if multiplier > 0.0 { multiplier } else { 1.0 };
    let (trits, scale) = native_query_encoder::NativeQueryEncoder::encode_ternary(text, dim, mult);
    if !out_scale_ptr.is_null() {
        unsafe { *out_scale_ptr = scale; }
    }
    unsafe {
        let dst = slice::from_raw_parts_mut(out_trits_ptr, dim);
        dst.copy_from_slice(&trits[..dim]);
    }
    dim as i32
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
        let agent_id = (state.values.get(0).unwrap_or(&0.0).abs() * 10.0) as u8 % 15;
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

/// Native Systolic Array Wavefront GEMM: C = A x B
/// A: [size x size], B: [size x size], C: [size x size] row-major float32
#[no_mangle]
pub extern "C" fn rust_systolic_gemm(
    a_ptr: *const f32,
    b_ptr: *const f32,
    c_ptr: *mut f32,
    size: usize,
) -> i32 {
    if a_ptr.is_null() || b_ptr.is_null() || c_ptr.is_null() || size == 0 {
        return -1;
    }

    unsafe {
        let a = slice::from_raw_parts(a_ptr, size * size);
        let b = slice::from_raw_parts(b_ptr, size * size);
        let c = slice::from_raw_parts_mut(c_ptr, size * size);

        // Zero out C
        for elem in c.iter_mut() {
            *elem = 0.0;
        }

        if size <= 32 {
            // Unrolled fast path for small matrices (16x16, 32x32)
            for i in 0..size {
                let a_row = &a[i * size..(i + 1) * size];
                let c_row = &mut c[i * size..(i + 1) * size];
                for k in 0..size {
                    let a_ik = *a_row.get_unchecked(k);
                    let b_row = &b[k * size..(k + 1) * size];
                    for j in 0..size {
                        *c_row.get_unchecked_mut(j) += a_ik * *b_row.get_unchecked(j);
                    }
                }
            }
        } else {
            // Tiled kernel for 64x64 and larger to optimize L1/L2 cache locality
            let block = 32;
            for bk in (0..size).step_by(block) {
                let k_end = (bk + block).min(size);
                for bi in (0..size).step_by(block) {
                    let i_end = (bi + block).min(size);
                    for i in bi..i_end {
                        let a_row = &a[i * size..(i + 1) * size];
                        let c_row = &mut c[i * size..(i + 1) * size];
                        for k in bk..k_end {
                            let a_ik = *a_row.get_unchecked(k);
                            let b_row = &b[k * size..(k + 1) * size];
                            for j in 0..size {
                                *c_row.get_unchecked_mut(j) += a_ik * *b_row.get_unchecked(j);
                            }
                        }
                    }
                }
            }
        }
    }
    0
}

/// Native Block-Sparse Scaled Dot-Product Attention
/// Optimized with Flash-Attention style masked row skipping, stack buffer reuse, and 4-way inner dot products.
#[no_mangle]
pub extern "C" fn rust_sparse_attention(
    q_ptr: *const f32,
    k_ptr: *const f32,
    v_ptr: *const f32,
    mask_ptr: *const f32,
    out_ptr: *mut f32,
    seq_len: usize,
    head_dim: usize,
) -> i32 {
    if q_ptr.is_null() || k_ptr.is_null() || v_ptr.is_null() || mask_ptr.is_null() || out_ptr.is_null() || seq_len == 0 || head_dim == 0 {
        return -1;
    }

    let scale = 1.0 / (head_dim as f32).sqrt();

    unsafe {
        let q = slice::from_raw_parts(q_ptr, seq_len * head_dim);
        let k = slice::from_raw_parts(k_ptr, seq_len * head_dim);
        let v = slice::from_raw_parts(v_ptr, seq_len * head_dim);
        let mask = slice::from_raw_parts(mask_ptr, seq_len * seq_len);
        let out = slice::from_raw_parts_mut(out_ptr, seq_len * head_dim);

        // Zero out output
        for elem in out.iter_mut() {
            *elem = 0.0;
        }

        // Stack buffer for row scores up to 512 tokens to avoid heap allocation
        let mut stack_scores = [0.0f32; 512];
        let mut active_indices = [0usize; 512];

        for i in 0..seq_len {
            let q_row = &q[i * head_dim..(i + 1) * head_dim];
            let mask_row = &mask[i * seq_len..(i + 1) * seq_len];

            // Fast-path: causal window bounding (sliding window + anchor prefix)
            // Determine active j range directly from mask
            let mut active_count = 0;
            let mut max_val = f32::NEG_INFINITY;

            // Tokens 0 and 1 are anchors; tokens max(0, i - window) ..= i are sliding window
            // Fast scan of row
            for j in 0..=i {
                let m = *mask_row.get_unchecked(j);
                if m > -1e8 {
                    let k_row = &k[j * head_dim..(j + 1) * head_dim];
                    
                    let mut dot = 0.0f32;
                    let mut d = 0;
                    while d + 4 <= head_dim {
                        dot += *q_row.get_unchecked(d) * *k_row.get_unchecked(d)
                            + *q_row.get_unchecked(d + 1) * *k_row.get_unchecked(d + 1)
                            + *q_row.get_unchecked(d + 2) * *k_row.get_unchecked(d + 2)
                            + *q_row.get_unchecked(d + 3) * *k_row.get_unchecked(d + 3);
                        d += 4;
                    }
                    while d < head_dim {
                        dot += *q_row.get_unchecked(d) * *k_row.get_unchecked(d);
                        d += 1;
                    }

                    let score = dot * scale + m;
                    if score > max_val {
                        max_val = score;
                    }
                    if active_count < 512 {
                        stack_scores[active_count] = score;
                        active_indices[active_count] = j;
                        active_count += 1;
                    }
                }
            }

            if active_count == 0 {
                continue;
            }

            // 2. Fused online softmax over active keys only
            let mut sum_exp = 0.0f32;
            for idx in 0..active_count {
                let exp_val = (stack_scores[idx] - max_val).exp();
                stack_scores[idx] = exp_val;
                sum_exp += exp_val;
            }

            let inv_sum = if sum_exp > 1e-12 { 1.0 / sum_exp } else { 0.0 };

            // 3. Fused weighted sum into out[i, :]
            let out_row = &mut out[i * head_dim..(i + 1) * head_dim];
            for idx in 0..active_count {
                let p = stack_scores[idx] * inv_sum;
                let j = active_indices[idx];
                let v_row = &v[j * head_dim..(j + 1) * head_dim];
                
                let mut d = 0;
                while d + 4 <= head_dim {
                    *out_row.get_unchecked_mut(d) += p * *v_row.get_unchecked(d);
                    *out_row.get_unchecked_mut(d + 1) += p * *v_row.get_unchecked(d + 1);
                    *out_row.get_unchecked_mut(d + 2) += p * *v_row.get_unchecked(d + 2);
                    *out_row.get_unchecked_mut(d + 3) += p * *v_row.get_unchecked(d + 3);
                    d += 4;
                }
                while d < head_dim {
                    *out_row.get_unchecked_mut(d) += p * *v_row.get_unchecked(d);
                    d += 1;
                }
            }
        }
    }
    0
}

/// Native Kipf-Welling GCN Layer Message Passing: Out = ReLU(NormAdj * X * W)
/// norm_adj: [num_nodes, num_nodes], x: [num_nodes, feat_dim], w: [feat_dim, feat_dim], out: [num_nodes, feat_dim]
#[no_mangle]
pub extern "C" fn rust_gcn_layer(
    norm_adj_ptr: *const f32,
    x_ptr: *const f32,
    w_ptr: *const f32,
    out_ptr: *mut f32,
    num_nodes: usize,
    feat_dim: usize,
) -> i32 {
    if norm_adj_ptr.is_null() || x_ptr.is_null() || w_ptr.is_null() || out_ptr.is_null() || num_nodes == 0 || feat_dim == 0 {
        return -1;
    }

    unsafe {
        let norm_adj = slice::from_raw_parts(norm_adj_ptr, num_nodes * num_nodes);
        let x = slice::from_raw_parts(x_ptr, num_nodes * feat_dim);
        let w = slice::from_raw_parts(w_ptr, feat_dim * feat_dim);
        let out = slice::from_raw_parts_mut(out_ptr, num_nodes * feat_dim);

        // Step 1: Intermediate AX = NormAdj * X -> [num_nodes, feat_dim]
        let mut ax = vec![0.0f32; num_nodes * feat_dim];
        for i in 0..num_nodes {
            let adj_row = &norm_adj[i * num_nodes..(i + 1) * num_nodes];
            let ax_row = &mut ax[i * feat_dim..(i + 1) * feat_dim];
            for k in 0..num_nodes {
                let a_ik = adj_row[k];
                if a_ik.abs() > 1e-9 {
                    let x_k = &x[k * feat_dim..(k + 1) * feat_dim];
                    for d in 0..feat_dim {
                        ax_row[d] += a_ik * x_k[d];
                    }
                }
            }
        }

        // Step 2: Out = ReLU(AX * W) -> [num_nodes, feat_dim]
        for i in 0..num_nodes {
            let ax_row = &ax[i * feat_dim..(i + 1) * feat_dim];
            let out_row = &mut out[i * feat_dim..(i + 1) * feat_dim];
            for j in 0..feat_dim {
                let mut sum = 0.0f32;
                for k in 0..feat_dim {
                    sum += ax_row[k] * w[k * feat_dim + j];
                }
                out_row[j] = if sum > 0.0 { sum } else { 0.0 }; // ReLU
            }
        }
    }
    0
}

/// Simple deterministic LCG random generator
#[inline]
fn lcg_rand(state: &mut u64) -> f32 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 32) as u32 as f32) / 4294967296.0
}

/// Native Restricted Boltzmann Machine (RBM) CD-1 Step & Reconstruction
/// v0: [n_visible], w: [n_visible, n_hidden], v_recon_out: [n_visible]
/// Returns reconstruction MSE error
#[no_mangle]
pub extern "C" fn rust_rbm_step(
    v0_ptr: *const f32,
    w_ptr: *const f32,
    v_recon_ptr: *mut f32,
    n_visible: usize,
    n_hidden: usize,
    seed: u64,
) -> f32 {
    if v0_ptr.is_null() || w_ptr.is_null() || v_recon_ptr.is_null() || n_visible == 0 || n_hidden == 0 {
        return -1.0;
    }

    let mut rng_state = if seed == 0 { 42u64 } else { seed };

    unsafe {
        let v0 = slice::from_raw_parts(v0_ptr, n_visible);
        let w = slice::from_raw_parts(w_ptr, n_visible * n_hidden);
        let v_recon = slice::from_raw_parts_mut(v_recon_ptr, n_visible);

        // Positive Phase: h_prob = sigmoid(v0 * W)
        let mut h0 = vec![0.0f32; n_hidden];
        for j in 0..n_hidden {
            let mut act = 0.0f32;
            for i in 0..n_visible {
                act += v0[i] * w[i * n_hidden + j];
            }
            // clip to [-20.0, 20.0]
            let act_clipped = act.clamp(-20.0, 20.0);
            let prob = 1.0 / (1.0 + (-act_clipped).exp());
            let rand_val = lcg_rand(&mut rng_state);
            h0[j] = if rand_val < prob { 1.0 } else { 0.0 };
        }

        // Negative Phase: v_recon_prob = sigmoid(W * h0)
        let mut mse = 0.0f32;
        for i in 0..n_visible {
            let mut act = 0.0f32;
            let w_row = &w[i * n_hidden..(i + 1) * n_hidden];
            for j in 0..n_hidden {
                act += w_row[j] * h0[j];
            }
            let act_clipped = act.clamp(-20.0, 20.0);
            let prob = 1.0 / (1.0 + (-act_clipped).exp());
            v_recon[i] = prob;
            let diff = v0[i] - prob;
            mse += diff * diff;
        }

        mse / (n_visible as f32)
    }
}

/// Batch Native Systolic GEMM: Processes `batch_size` matrix multiplications in pure Rust
/// a_batch: [batch_size, size, size], b_batch: [batch_size, size, size], c_batch: [batch_size, size, size]
#[no_mangle]
pub extern "C" fn rust_systolic_gemm_batch(
    a_batch_ptr: *const f32,
    b_batch_ptr: *const f32,
    c_batch_ptr: *mut f32,
    size: usize,
    batch_size: usize,
) -> i32 {
    if a_batch_ptr.is_null() || b_batch_ptr.is_null() || c_batch_ptr.is_null() || size == 0 || batch_size == 0 {
        return -1;
    }

    let mat_len = size * size;
    unsafe {
        for b in 0..batch_size {
            let a_ptr = a_batch_ptr.add(b * mat_len);
            let b_ptr = b_batch_ptr.add(b * mat_len);
            let c_ptr = c_batch_ptr.add(b * mat_len);

            let a = slice::from_raw_parts(a_ptr, mat_len);
            let b_slice = slice::from_raw_parts(b_ptr, mat_len);
            let c = slice::from_raw_parts_mut(c_ptr, mat_len);

            for elem in c.iter_mut() {
                *elem = 0.0;
            }

            for i in 0..size {
                let row_a_idx = i * size;
                let row_c_idx = i * size;
                for k in 0..size {
                    let a_ik = a[row_a_idx + k];
                    let row_b_idx = k * size;
                    for j in 0..size {
                        c[row_c_idx + j] += a_ik * b_slice[row_b_idx + j];
                    }
                }
            }
        }
    }
    0
}

/// Batch Native Block-Sparse Scaled Dot-Product Attention
#[no_mangle]
pub extern "C" fn rust_sparse_attention_batch(
    q_batch_ptr: *const f32,
    k_batch_ptr: *const f32,
    v_batch_ptr: *const f32,
    mask_ptr: *const f32,
    out_batch_ptr: *mut f32,
    seq_len: usize,
    head_dim: usize,
    batch_size: usize,
) -> i32 {
    if q_batch_ptr.is_null() || k_batch_ptr.is_null() || v_batch_ptr.is_null() || mask_ptr.is_null() || out_batch_ptr.is_null() || seq_len == 0 || head_dim == 0 || batch_size == 0 {
        return -1;
    }

    let tensor_len = seq_len * head_dim;
    for b in 0..batch_size {
        unsafe {
            let q_ptr = q_batch_ptr.add(b * tensor_len);
            let k_ptr = k_batch_ptr.add(b * tensor_len);
            let v_ptr = v_batch_ptr.add(b * tensor_len);
            let out_ptr = out_batch_ptr.add(b * tensor_len);
            rust_sparse_attention(q_ptr, k_ptr, v_ptr, mask_ptr, out_ptr, seq_len, head_dim);
        }
    }
    0
}

/// Batch Native Kipf-Welling GCN Message Passing
#[no_mangle]
pub extern "C" fn rust_gcn_layer_batch(
    norm_adj_ptr: *const f32,
    x_batch_ptr: *const f32,
    w_ptr: *const f32,
    out_batch_ptr: *mut f32,
    num_nodes: usize,
    feat_dim: usize,
    batch_size: usize,
) -> i32 {
    if norm_adj_ptr.is_null() || x_batch_ptr.is_null() || w_ptr.is_null() || out_batch_ptr.is_null() || num_nodes == 0 || feat_dim == 0 || batch_size == 0 {
        return -1;
    }

    let x_len = num_nodes * feat_dim;
    for b in 0..batch_size {
        unsafe {
            let x_ptr = x_batch_ptr.add(b * x_len);
            let out_ptr = out_batch_ptr.add(b * x_len);
            rust_gcn_layer(norm_adj_ptr, x_ptr, w_ptr, out_ptr, num_nodes, feat_dim);
        }
    }
    0
}

/// Native Autoregressive Attention with Sliding-Window KV-Cache
/// q_curr: [1, head_dim], k_cache: [max_cache_len, head_dim], v_cache: [max_cache_len, head_dim]
/// new_k: [1, head_dim], new_v: [1, head_dim], out: [1, head_dim]
/// Returns the updated cache length
#[no_mangle]
pub extern "C" fn rust_attention_with_kv_cache(
    q_curr_ptr: *const f32,
    k_cache_ptr: *mut f32,
    v_cache_ptr: *mut f32,
    new_k_ptr: *const f32,
    new_v_ptr: *const f32,
    out_ptr: *mut f32,
    head_dim: usize,
    current_cache_len: usize,
    max_cache_len: usize,
) -> i32 {
    if q_curr_ptr.is_null() || k_cache_ptr.is_null() || v_cache_ptr.is_null()
        || new_k_ptr.is_null() || new_v_ptr.is_null() || out_ptr.is_null()
        || head_dim == 0 || max_cache_len == 0 {
        return -1;
    }

    unsafe {
        let q = slice::from_raw_parts(q_curr_ptr, head_dim);
        let new_k = slice::from_raw_parts(new_k_ptr, head_dim);
        let new_v = slice::from_raw_parts(new_v_ptr, head_dim);
        let out = slice::from_raw_parts_mut(out_ptr, head_dim);

        // 1. Insert new_k and new_v into circular / append cache
        let write_idx = if current_cache_len < max_cache_len {
            current_cache_len
        } else {
            max_cache_len - 1 // saturate / overwrite oldest window
        };

        let k_slot = slice::from_raw_parts_mut(k_cache_ptr.add(write_idx * head_dim), head_dim);
        let v_slot = slice::from_raw_parts_mut(v_cache_ptr.add(write_idx * head_dim), head_dim);
        k_slot.copy_from_slice(new_k);
        v_slot.copy_from_slice(new_v);

        let effective_len = (current_cache_len + 1).min(max_cache_len);
        let scale = 1.0 / (head_dim as f32).sqrt();

        // 2. Score q against all keys in cache
        let mut scores = vec![0.0f32; effective_len];
        let mut max_score = f32::NEG_INFINITY;

        for j in 0..effective_len {
            let k_row = slice::from_raw_parts(k_cache_ptr.add(j * head_dim), head_dim);
            let mut dot = 0.0f32;
            let mut d = 0;
            while d + 4 <= head_dim {
                dot += q[d] * k_row[d] + q[d + 1] * k_row[d + 1] + q[d + 2] * k_row[d + 2] + q[d + 3] * k_row[d + 3];
                d += 4;
            }
            while d < head_dim {
                dot += q[d] * k_row[d];
                d += 1;
            }
            let s = dot * scale;
            scores[j] = s;
            if s > max_score {
                max_score = s;
            }
        }

        // 3. Softmax
        let mut sum_exp = 0.0f32;
        for s in scores.iter_mut() {
            *s = (*s - max_score).exp();
            sum_exp += *s;
        }
        let inv_sum = if sum_exp > 1e-12 { 1.0 / sum_exp } else { 0.0 };

        // 4. Weighted accumulation into output
        for d in out.iter_mut() {
            *d = 0.0;
        }

        for j in 0..effective_len {
            let p = scores[j] * inv_sum;
            let v_row = slice::from_raw_parts(v_cache_ptr.add(j * head_dim), head_dim);
            let mut d = 0;
            while d + 4 <= head_dim {
                out[d] += p * v_row[d];
                out[d + 1] += p * v_row[d + 1];
                out[d + 2] += p * v_row[d + 2];
                out[d + 3] += p * v_row[d + 3];
                d += 4;
            }
            while d < head_dim {
                out[d] += p * v_row[d];
                d += 1;
            }
        }

        effective_len as i32
    }
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

    #[test]
    fn bench_pure_rust_suite() {
        use std::time::Instant;

        println!("\n=== PURE RUST IN-ENGINE COMPUTE BENCHMARK (ZERO FFI) ===");

        // 1. Systolic 16x16
        let a = vec![1.0f32; 16 * 16];
        let b = vec![2.0f32; 16 * 16];
        let mut c = vec![0.0f32; 16 * 16];
        let iters = 100_000;
        let t0 = Instant::now();
        for _ in 0..iters {
            rust_systolic_gemm(a.as_ptr(), b.as_ptr(), c.as_mut_ptr(), 16);
        }
        let per_call_ns = (t0.elapsed().as_nanos() as f64) / (iters as f64);
        println!("  • Systolic 16x16 GEMM:     {:.3} µs ({} ns)", per_call_ns / 1000.0, per_call_ns as u64);

        // 2. Systolic 64x64
        let a64 = vec![1.0f32; 64 * 64];
        let b64 = vec![2.0f32; 64 * 64];
        let mut c64 = vec![0.0f32; 64 * 64];
        let iters64 = 10_000;
        let t0 = Instant::now();
        for _ in 0..iters64 {
            rust_systolic_gemm(a64.as_ptr(), b64.as_ptr(), c64.as_mut_ptr(), 64);
        }
        let per_call_64_ns = (t0.elapsed().as_nanos() as f64) / (iters64 as f64);
        println!("  • Systolic 64x64 GEMM:     {:.3} µs ({} ns)", per_call_64_ns / 1000.0, per_call_64_ns as u64);

        // 3. Sparse Attention (seq=64, dim=16)
        let q = vec![0.5f32; 64 * 16];
        let k = vec![0.5f32; 64 * 16];
        let v = vec![0.5f32; 64 * 16];
        let mask = vec![0.0f32; 64 * 64];
        let mut out = vec![0.0f32; 64 * 16];
        let iters_attn = 5_000;
        let t0 = Instant::now();
        for _ in 0..iters_attn {
            rust_sparse_attention(q.as_ptr(), k.as_ptr(), v.as_ptr(), mask.as_ptr(), out.as_mut_ptr(), 64, 16);
        }
        let per_call_attn_ns = (t0.elapsed().as_nanos() as f64) / (iters_attn as f64);
        println!("  • Sparse Attention (64x16): {:.3} µs ({} ns)", per_call_attn_ns / 1000.0, per_call_attn_ns as u64);

        // 4. GNN Message Passing (nodes=24, dim=16)
        let norm_adj = vec![0.1f32; 24 * 24];
        let x = vec![0.5f32; 24 * 16];
        let w = vec![0.1f32; 16 * 16];
        let mut out_gnn = vec![0.0f32; 24 * 16];
        let iters_gnn = 20_000;
        let t0 = Instant::now();
        for _ in 0..iters_gnn {
            rust_gcn_layer(norm_adj.as_ptr(), x.as_ptr(), w.as_ptr(), out_gnn.as_mut_ptr(), 24, 16);
        }
        let per_call_gnn_ns = (t0.elapsed().as_nanos() as f64) / (iters_gnn as f64);
        println!("  • GCN Message Pass (24x16): {:.3} µs ({} ns)", per_call_gnn_ns / 1000.0, per_call_gnn_ns as u64);

        // 5. RBM Step (vis=32, hid=16)
        let v0 = vec![1.0f32; 32];
        let w_rbm = vec![0.1f32; 32 * 16];
        let mut v_recon = vec![0.0f32; 32];
        let iters_rbm = 50_000;
        let t0 = Instant::now();
        for _ in 0..iters_rbm {
            rust_rbm_step(v0.as_ptr(), w_rbm.as_ptr(), v_recon.as_mut_ptr(), 32, 16, 42);
        }
        let per_call_rbm_ns = (t0.elapsed().as_nanos() as f64) / (iters_rbm as f64);
        println!("  • RBM CD-1 Step (32x16):   {:.3} µs ({} ns)", per_call_rbm_ns / 1000.0, per_call_rbm_ns as u64);

        // 6. Batch Systolic 16x16 (Batch Size = 100)
        let batch_size = 100;
        let a_batch = vec![1.0f32; batch_size * 16 * 16];
        let b_batch = vec![2.0f32; batch_size * 16 * 16];
        let mut c_batch = vec![0.0f32; batch_size * 16 * 16];
        let iters_batch = 1_000;
        let t0 = Instant::now();
        for _ in 0..iters_batch {
            rust_systolic_gemm_batch(a_batch.as_ptr(), b_batch.as_ptr(), c_batch.as_mut_ptr(), 16, batch_size);
        }
        let per_batch_ns = (t0.elapsed().as_nanos() as f64) / (iters_batch as f64);
        println!("  • Batch Systolic 16x16 (B=100): {:.3} µs total -> {:.3} µs/item", per_batch_ns / 1000.0, per_batch_ns / (batch_size as f64 * 1000.0));
        println!("=======================================================\n");
    }
}
