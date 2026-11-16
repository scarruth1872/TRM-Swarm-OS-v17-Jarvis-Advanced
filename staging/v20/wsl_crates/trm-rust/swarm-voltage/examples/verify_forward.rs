//! Build #1 verify — run the full qwen3moe forward in Rust, compare
//! against the llama.cpp reference hidden state (GPU fast path).
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use swarm_gguf::parse_file;
use swarm_voltage::forward::*;
use swarm_voltage::rms_norm;
use swarm_voltage::VoltageModel;

/// The model path, resolved at CALL time. It cannot be a `const`: resolution is a fact about the
/// host (a WSL absolute path maps to a UNC path on Windows), and a const is evaluated at compile
/// time on whatever machine happens to be building.
fn model_path() -> String {
    swarm_paths::resolve("/home/shawn/models/qwen-moe-2.4b-q4_k_m.gguf")
}

// ggml types: F32=0, F16=1, Q4_K=12 (144B/256), Q6_K=14 (210B/256)
fn type_bytes(t: &swarm_gguf::TensorInfo) -> u64 {
    let n: u64 = t.dims.iter().product();
    match t.ggml_type {
        12 => (n / 256) * 144, // Q4_K
        14 => (n / 256) * 210, // Q6_K
        1 => n * 2,            // F16
        _ => n * 4,            // F32 (0) and default
    }
}

fn load_raw(path: &str, model: &swarm_gguf::GgufModel, name: &str) -> Vec<u8> {
    let t = model
        .tensors
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("tensor {name} not found"));
    let abs = model.data_start + t.offset;
    let bytes = type_bytes(t);
    let mut f = File::open(path).unwrap();
    f.seek(SeekFrom::Start(abs)).unwrap();
    let mut buf = vec![0u8; bytes as usize];
    f.read_exact(&mut buf).unwrap();
    buf
}

/// Read an F32 tensor (norms are ggml_type 0).
fn f32_tensor(vm: &VoltageModel, name: &str) -> Vec<f32> {
    let raw = vm.tensors.get(name).unwrap_or_else(|| panic!("{name} missing"));
    raw.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn main() {
    let m = parse_file(&model_path()).expect("parse gguf");
    let mut vm = VoltageModel {
        tensors: HashMap::new(),
        f16_names: vec![],
    };

    for b in 0..N_LAYERS {
        let blk = format!("blk.{b}.");
        for suffix in [
            "attn_q.weight",
            "attn_k.weight",
            "attn_v.weight",
            "attn_output.weight",
            "ffn_gate_exps.weight",
            "ffn_up_exps.weight",
            "ffn_down_exps.weight",
        ] {
            let n = format!("{blk}{suffix}");
            vm.tensors.insert(n.clone(), load_raw(&model_path(), &m, &n));
        }
        for suffix in [
            "attn_norm.weight",
            "ffn_norm.weight",
            "attn_q_norm.weight",
            "attn_k_norm.weight",
            "ffn_gate_inp.weight",
        ] {
            let n = format!("{blk}{suffix}");
            vm.tensors.insert(n.clone(), load_raw(&model_path(), &m, &n));
        }
    }
    vm.tensors.insert(
        "token_embd.weight".into(),
        load_raw(&model_path(), &m, "token_embd.weight"),
    );
    vm.tensors.insert(
        "output_norm.weight".into(),
        load_raw(&model_path(), &m, "output_norm.weight"),
    );

    println!("loaded {} tensors", vm.tensors.len());

    let tokens: [u32; 4] = [151643, 198, 1513, 1568];

    let te = vm
        .dequant_tensor("token_embd.weight")
        .expect("dequant embd");
    println!("token_embd dequant: {} floats", te.len());

    let mut hs: Vec<f32> = Vec::with_capacity(tokens.len() * DIM);
    for &tok in &tokens {
        for i in 0..DIM {
            hs.push(te[tok as usize * DIM + i]);
        }
    }
    println!(
        "embedded {} tokens, |h[3]|={:.4}",
        tokens.len(),
        hs[3 * DIM..4 * DIM]
            .iter()
            .map(|v| v * v)
            .sum::<f32>()
            .sqrt()
    );

    for b in 0..N_LAYERS {
        let blk = format!("blk.{b}.");
        let gt = |n: &str| -> u32 {
            m.tensors.iter().find(|t| t.name == n).map(|t| t.ggml_type).unwrap()
        };
        let q_w = vm.dequant_typed(&format!("{blk}attn_q.weight"), gt(&format!("{blk}attn_q.weight"))).unwrap();
        let k_w = vm.dequant_typed(&format!("{blk}attn_k.weight"), gt(&format!("{blk}attn_k.weight"))).unwrap();
        let v_w = vm.dequant_typed(&format!("{blk}attn_v.weight"), gt(&format!("{blk}attn_v.weight"))).unwrap();
        let o_w = vm
            .dequant_typed(&format!("{blk}attn_output.weight"), gt(&format!("{blk}attn_output.weight")))
            .unwrap();
        let an_w = f32_tensor(&vm, &format!("{blk}attn_norm.weight"));
        let fn_w = f32_tensor(&vm, &format!("{blk}ffn_norm.weight"));
        let qn_w = f32_tensor(&vm, &format!("{blk}attn_q_norm.weight"));
        let kn_w = f32_tensor(&vm, &format!("{blk}attn_k_norm.weight"));
        let gi_w = f32_tensor(&vm, &format!("{blk}ffn_gate_inp.weight"));
        let ge_w = vm
            .dequant_typed(&format!("{blk}ffn_gate_exps.weight"), gt(&format!("{blk}ffn_gate_exps.weight")))
            .unwrap();
        let up_w = vm
            .dequant_typed(&format!("{blk}ffn_up_exps.weight"), gt(&format!("{blk}ffn_up_exps.weight")))
            .unwrap();
        let dn_w = vm
            .dequant_typed(&format!("{blk}ffn_down_exps.weight"), gt(&format!("{blk}ffn_down_exps.weight")))
            .unwrap();

        hs = block(
            &hs, &vm, &an_w, &q_w, &k_w, &v_w, &o_w, &qn_w, &kn_w, &fn_w, &gi_w, &ge_w, &up_w,
            &dn_w,
        );
        if b % 4 == 0 || b == N_LAYERS - 1 {
            let last = &hs[3 * DIM..4 * DIM];
            println!(
                "  block {b}: |h[3]|={:.4}",
                last.iter().map(|v| v * v).sum::<f32>().sqrt()
            );
        }
        drop((
            q_w, k_w, v_w, o_w, an_w, fn_w, qn_w, kn_w, gi_w, ge_w, up_w, dn_w,
        ));
    }

    let on_w = f32_tensor(&vm, "output_norm.weight");
    let hidden = rms_norm(&hs[3 * DIM..4 * DIM], &on_w, EPS);

    let mut f = File::open(swarm_paths::resolve("/home/shawn/trm-rust/ref_hidden.bin")).unwrap();
    let mut nbuf = [0u8; 4];
    f.read_exact(&mut nbuf).unwrap();
    let n = i32::from_le_bytes(nbuf) as usize;
    let mut refv = vec![0f32; n];
    unsafe {
        let ptr = refv.as_mut_ptr() as *mut u8;
        f.read_exact(std::slice::from_raw_parts_mut(ptr, n * 4))
            .unwrap();
    }
    println!("reference: {} dims, ours: {}", n, hidden.len());

    let mut max_diff = 0f32;
    let mut sum_sq = 0f64;
    for i in 0..n.min(hidden.len()) {
        let d = (hidden[i] - refv[i]).abs();
        max_diff = max_diff.max(d);
        sum_sq += d as f64 * d as f64;
    }
    let rmse = (sum_sq / n as f64).sqrt();
    println!("max diff: {max_diff:.6}");
    println!("rmse: {rmse:.6}");
    println!("ours first 8: {:?}", &hidden[..8]);
    println!("ref  first 8: {:?}", &refv[..8]);
    let pass = max_diff < 0.1;
    println!(
        "\nBUILD #1 VERIFY: {}",
        if pass {
            "PASSED (hidden state matches reference)"
        } else {
            "FAILED — diagnose"
        }
    );
}
