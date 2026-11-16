//! Verify gate: swarm-instella vs c_instella_gate_topk (real kernel).
//! Same 384-dim hidden state + 64×384 expert weights through both;
//! compares selected top-k expert ids and softmax weights.

use std::os::raw::{c_float, c_uint, c_uchar};

const TOTAL_EXPERTS: usize = 64;
const HIDDEN_DIM: usize = 384;

#[repr(C, align(64))]
struct InstellaGateInput {
    hidden_states: [f32; HIDDEN_DIM],
    expert_weights: [[f32; HIDDEN_DIM]; TOTAL_EXPERTS],
}

#[repr(C, align(64))]
struct InstellaGateOutput {
    selected_expert_ids: [c_uchar; 8],
    expert_gate_weights: [c_float; 8],
    active_experts_count: c_uint,
}

fn main() {
    let lib = unsafe {
        libloading::Library::new(
            swarm_paths::kernel_ops_path(),
        )
        .expect("load kernel")
    };
    unsafe {
        let gate_c: libloading::Symbol<
            unsafe extern "C" fn(*const InstellaGateInput, c_uchar, *mut InstellaGateOutput) -> bool,
        > = lib.get(b"c_instella_gate_topk").unwrap();

        // deterministic hidden state + weights
        let mut input = InstellaGateInput {
            hidden_states: [0.0; HIDDEN_DIM],
            expert_weights: [[0.0; HIDDEN_DIM]; TOTAL_EXPERTS],
        };
        let mut seed: u64 = 42;
        for i in 0..HIDDEN_DIM {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            input.hidden_states[i] = ((seed >> 33) % 100) as f32 / 100.0;
        }
        for e in 0..TOTAL_EXPERTS {
            for i in 0..HIDDEN_DIM {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                input.expert_weights[e][i] = ((seed >> 33) % 100) as f32 / 100.0;
            }
        }

        // Rust gate
        let (rust_ids, rust_ws) = swarm_instella::gate_topk(&input.hidden_states, &input.expert_weights, 4);

        // C gate
        let mut out = InstellaGateOutput {
            selected_expert_ids: [0; 8],
            expert_gate_weights: [0.0; 8],
            active_experts_count: 0,
        };
        let ok = gate_c(&input, 4, &mut out);
        assert!(ok);

        let c_ids: Vec<u8> = out.selected_expert_ids[..4].to_vec();
        let c_ws: Vec<f32> = out.expert_gate_weights[..4].to_vec();

        println!("Rust top-4 ids: {rust_ids:?} weights: {rust_ws:?}");
        println!("C    top-4 ids: {c_ids:?} weights: {c_ws:?}");

        let ids_equal = rust_ids == c_ids;
        let mut max_wdiff = 0.0f32;
        for i in 0..4 {
            let d = (rust_ws[i] - c_ws[i]).abs();
            if d > max_wdiff { max_wdiff = d; }
        }
        println!("ids identical: {ids_equal}");
        println!("max weight diff: {max_wdiff:.2e}");
        println!("active count: C={} (expect 4)", out.active_experts_count);

        if ids_equal && max_wdiff < 1e-4 {
            println!("VERIFY GATE: PASSED - swarm-instella matches c_instella_gate_topk");
        } else {
            println!("VERIFY GATE: FAILED");
            std::process::exit(1);
        }
    }
}
