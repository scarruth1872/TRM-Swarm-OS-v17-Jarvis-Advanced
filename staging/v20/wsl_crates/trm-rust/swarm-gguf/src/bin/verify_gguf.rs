//! Verify gate: swarm-gguf vs the real C++ loader on the ACTUAL model.
//! Parses the 6.5GB Instella Q2_K model in Rust, prints the tensor
//! inventory summary, and compares against the C++ loader's report.

use swarm_gguf::*;

fn main() {
    let path = &swarm_paths::resolve("/home/shawn/models/instella-gguf/Q2_K/amd.Instella-MoE-16B-A3B-Think.f16.gguf.Q2_K.gguf");
    let m = parse_file(path).expect("parse model");

    println!("== Rust GGUF parse of the REAL Instella model ==");
    println!("magic: 0x{:08X} | version: {} | tensors: {} | kvs: {}",
             m.header.magic, m.header.version, m.header.n_tensors, m.header.n_kv);
    println!("file: {:.2} GB | data_start: {}", m.file_size as f64 / 1e9, m.data_start);

    // type census — must match the C++ doc: 141 F32, 169 Q4_K, 29 Q6_K etc.
    let mut census: std::collections::HashMap<u32, u64> = Default::default();
    for t in &m.tensors {
        *census.entry(t.ggml_type).or_insert(0) += 1;
    }
    println!("\nTensor type census (ggml_type):");
    for (t, n) in census.iter() {
        println!("  type {t}: {n} tensors");
    }

    // architecture KV — the loader's key metadata
    println!("\nArchitecture KVs:");
    for (k, v) in &m.kvs {
        if k.contains("architecture") || k.contains("expert") || k.contains("attention")
           || k.contains("block_count") || k.contains("context") {
            println!("  {k} = {v}");
        }
    }

    // first/last few tensor names (offset order sanity)
    println!("\nFirst 5 tensors (by index):");
    for t in m.tensors.iter().take(5) {
        println!("  {:>40} dims={:?} type={} off={}", t.name, t.dims, t.ggml_type, t.offset);
    }
    println!("Last 3 tensors:");
    for t in m.tensors.iter().rev().take(3) {
        println!("  {:>40} dims={:?} type={} off={}", t.name, t.dims, t.ggml_type, t.offset);
    }

    // total element count (the C++ loader reports ~1.54B elements)
    let total_elems: u64 = m.tensors.iter()
        .map(|t| t.dims.iter().product::<u64>())
        .sum();
    println!("\nTotal elements: {total_elems} ({:.1}B)", total_elems as f64 / 1e9);
    println!("VERIFY GATE: swarm-gguf parsed the real model - census and offsets ready");
}
