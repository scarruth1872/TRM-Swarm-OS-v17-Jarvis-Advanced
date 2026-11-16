//! Verify the Instella MLA attention forward for blk.0: dequant the real weights, run the
//! absorbed-MLA for 2 tokens (so the softmax is non-trivial), check sane output. Parity check
//! against llama.cpp is the next step — this only proves the forward runs without garbage.

use std::io::{Read, Seek, SeekFrom};

use swarm_dequant::mla::{mla_forward, KV_LORA, N_EMBD, N_HEAD, NOPE, V_MLA};
use swarm_dequant::{dequant_q4_k, dequant_q5_0, f16_to_f32};

const PATH: &str = "D:/trm-models/instella/amd.Instella-MoE-16B-A3B-Think.Q4_K_M.gguf";

fn n_bytes(t: &swarm_gguf::TensorInfo) -> usize {
    let n = t.dims.iter().product::<u64>() as usize;
    match t.ggml_type {
        12 => n / 256 * 144, // Q4_K
        8 => n / 32 * 34,    // Q8_0
        6 => n / 32 * 22,    // Q5_0
        1 => n * 2,          // f16
        _ => n * 4,          // f32
    }
}

fn load(f: &mut std::fs::File, m: &swarm_gguf::GgufModel, name: &str) -> Vec<f32> {
    let t = m.tensors.iter().find(|t| t.name == name).expect(name);
    let n = t.dims.iter().product::<u64>() as usize;
    let mut buf = vec![0u8; n_bytes(t)];
    f.seek(SeekFrom::Start(m.data_start + t.offset)).unwrap();
    f.read_exact(&mut buf).unwrap();
    match t.ggml_type {
        12 => dequant_q4_k(&buf, n),
        6 => dequant_q5_0(&buf, n),
        1 => buf
            .chunks_exact(2)
            .map(|c| f16_to_f32(u16::from_le_bytes([c[0], c[1]])))
            .collect(),
        _ => buf
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    }
}

fn stats(v: &[f32], label: &str) {
    let mn = v.iter().cloned().fold(f32::INFINITY, f32::min);
    let mx = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mean = v.iter().map(|&x| x as f64).sum::<f64>() / v.len() as f64;
    let nan = v.iter().filter(|x| x.is_nan()).count();
    println!("{label}: n={} min={mn:.4} max={mx:.4} mean={mean:.4} nan={nan}", v.len());
    if !v.is_empty() {
        println!("  first 8: {:?}", &v[..v.len().min(8)]);
    }
}

fn main() {
    let m = swarm_gguf::parse_file(PATH).expect("parse");
    let mut f = std::fs::File::open(PATH).expect("open");

    let wq = load(&mut f, &m, "blk.0.attn_q.weight");
    let wkv_a = load(&mut f, &m, "blk.0.attn_kv_a_mqa.weight");
    let wk_b = load(&mut f, &m, "blk.0.attn_k_b.weight");
    let wv_b = load(&mut f, &m, "blk.0.attn_v_b.weight");
    let wg = load(&mut f, &m, "blk.0.attn_gate.weight");
    let wo = load(&mut f, &m, "blk.0.attn_output.weight");
    let kv_a_norm = load(&mut f, &m, "blk.0.attn_kv_a_norm.weight");
    println!(
        "loaded: wq {} wkv_a {} wk_b {} wv_b {} wg {} wo {} kv_a_norm {}",
        wq.len(), wkv_a.len(), wk_b.len(), wv_b.len(), wg.len(), wo.len(), kv_a_norm.len()
    );
    for (nm, w) in [("wq", &wq), ("wkv_a", &wkv_a), ("wk_b", &wk_b), ("wv_b", &wv_b), ("wg", &wg), ("wo", &wo)] {
        let nan = w.iter().filter(|x| x.is_nan()).count();
        let inf = w.iter().filter(|x| x.is_infinite()).count();
        let mn = w.iter().cloned().fold(f32::INFINITY, f32::min);
        let mx = w.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        println!("  weight {nm}: nan={nan} inf={inf} min={mn:.4} max={mx:.4}");
    }
    assert_eq!(wk_b.len(), NOPE * KV_LORA * N_HEAD);
    assert_eq!(wv_b.len(), KV_LORA * V_MLA * N_HEAD);

    let mut k_cache: Vec<Vec<f32>> = vec![];
    let mut v_cache: Vec<Vec<f32>> = vec![];

    for pos in 0..2 {
        // sample hidden: deterministic ramp, scaled down
        let h: Vec<f32> = (0..N_EMBD).map(|i| (i as f32 - 1024.0) / 4096.0).collect();
        let t0 = std::time::Instant::now();
        let out = mla_forward(&h, pos, 0, &mut k_cache, &mut v_cache, &wq, &wkv_a, &kv_a_norm, &wk_b, &wv_b, &wg, &wo, false);
        let el = t0.elapsed();
        stats(&out, &format!("mla token {pos}"));
        println!("  time={el:?} cache_len={}", k_cache.len());
    }
}
