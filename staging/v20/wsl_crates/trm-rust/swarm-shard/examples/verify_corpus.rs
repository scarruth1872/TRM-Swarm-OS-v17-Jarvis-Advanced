//! verify_corpus.rs — Motion A gate: ingest the REAL code corpus into
//! a sharded fabric, verify by readback.
use swarm_shard::corpus::{corpus_block, ingest_dir};
use swarm_shard::ShardedFabric;
use std::path::Path;

fn main() {
    let root = "/home/shawn/.hermes/cache/trm_corpus_shard";
    let _ = std::fs::remove_dir_all(root);
    let src = Path::new("/home/shawn/trm-rust");

    println!("=== MOTION A: REAL CODE CORPUS INGEST ===");
    let files = ingest_dir(src);
    println!("ingested {} real .rs files from {src:?}", files.len());
    let total_bytes: u64 = files.iter().map(|f| f.size).sum();
    println!("corpus size: {:.2} KB across {total_bytes} bytes", total_bytes as f64 / 1024.0);

    let block = corpus_block(&files);
    println!("K3 block: {} trits ({} bytes packed)", block.len(), (block.len() * 2 + 7) / 8);

    // write through the sharded fabric
    let mut fab = ShardedFabric::open(root).expect("open sharded");
    println!("shards before: {}", fab.shard_count());
    let (sid, off) = fab.append(&block).expect("append");

    // VERIFY by readback — byte-identical
    let back = fab.read(sid, off, block.len()).expect("readback");
    let identical = back == block;
    println!("readback: {} trits, byte-identical: {identical}", back.len());
    let total = fab.total_trits();
    println!("shards after: {} (total {total} trits in fabric)", fab.shard_count());

    // verify the manifest recorded it
    if let Some(a) = fab.manifest().active_shard() {
        println!("active shard: {} (codec v{})", a.id, a.codec);
    }

    assert!(identical, "MOTION A GATE: readback must be byte-identical");
    println!("\nMOTION A GATE: PASSED — real corpus in real shards, verified readback");
}
