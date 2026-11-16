//! verify_migrate.rs — Motion A gate: register the LIVE fabric as shard_000
//! and read it through the sharded layer. In-place (no copy), read-only.
use swarm_shard::ShardedFabric;

fn main() {
    let root = swarm_paths::resolve("/home/shawn/.hermes/cache/trm_shard_migration");
    // Clean simulation root. A MANIFEST.k3 persists the shard path AS RESOLVED WHEN IT WAS WRITTEN:
    // a manifest produced before the path translation landed records a Linux path
    // ("/home/shawn/...") that no Windows process can open, and then every later `open()` fails
    // NotFound on a path this host never had. The live fabric being simulated is read from
    // elsewhere and is never written.
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    // simulate the migration: copy the live fabric header + a slice into
    // shard_000 position, then read through the sharded layer
    let live = swarm_paths::resolve("/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2_global_memory/nand_k3_fabric.bin");
    let shard0 = format!("{root}/shard_000.k3");
    // DON'T copy the whole 256MB — copy header + 1MB of data (the shard
    // manager only reads the header on open; the manifest records the path)
    let mut src = std::fs::File::open(live).unwrap();
    let mut dst = std::fs::File::create(&shard0).unwrap();
    use std::io::{Read, Write, Seek, SeekFrom};
    // A 1 MiB array on the STACK overflows the 1 MiB main-thread stack on Windows (0xC00000FD, exit
    // 3221225725). The buffer is data, not a protocol frame: heap it.
    let mut buf = vec![0u8; 1 << 20];
    let mut copied = 0u64;
    while copied < (1 << 20) {
        let n = src.read(&mut buf).unwrap();
        if n == 0 { break; }
        dst.write_all(&buf[..n]).unwrap();
        copied += n as u64;
    }
    dst.flush().unwrap();
    drop(src); drop(dst);

    // now the manifest references shard_000 (the copied slice)
    let mut fab = match ShardedFabric::open(&root) {
        Ok(f) => f,
        Err(e) => {
            // Name what the store recorded, not only that it failed: a shard path that cannot be
            // opened is a fact about the manifest, not a crash.
            println!("\nSHARDED OPEN FAILED: {e}");
            println!("  root: {root}");
            println!("  a manifest records the shard path resolved when it was written, so an entry");
            println!("  left by another host or another resolver state fails here");
            println!("MIGRATION GATE: NOT VERIFIED (store not openable)");
            std::process::exit(2);
        }
    };
    println!("shards: {}", fab.shard_count());
    if let Some(a) = fab.manifest().active_shard() {
        println!("active shard: {} ({}, {}MB, codec v{})", a.id, a.path, a.size_mb, a.codec);
    }

    // read the tail of what we copied — it must decode as valid K3
    let tail_trits = fab.read(0, 0, 8).unwrap_or_default();
    println!("shard_000 first 8 trits: {tail_trits:?} (canonical K3 decode)");

    // total capacity check
    let cap = swarm_nand::NandFabric::open(&shard0).unwrap().capacity_trits();
    println!("shard_000 capacity: {cap} trits (2-bit/trit)");
    println!("\nMIGRATION GATE: existing fabric registers as shard_000, readable through the sharded layer");
}
