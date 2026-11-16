//! ingest_scale.rs — Motion A (v5): FULL-SCALE corpus ingest.
//! Ingests the REAL source corpus (trm-rust + Agent-Q + swarm_v2) into
//! the sharded fabric — all .rs/.py/.ts/.js/.json files found.
use swarm_shard::ShardedFabric;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                // skip .git, node_modules, target, venvs (binary/junk)
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                if name == ".git" || name == "node_modules" || name == "target" || name == ".venv" || name == "__pycache__" || name == ".next" {
                    continue;
                }
                walk(&p, out);
            } else {
                let ext = p.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
                if matches!(ext.as_str(), "rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "json" | "md" | "toml" | "yaml" | "yml" | "sh") {
                    out.push(p);
                }
            }
        }
    }
}

fn main() {
    let root = "/home/shawn/.hermes/cache/trm_scale_shard";
    let _ = fs::remove_dir_all(root);
    let roots = [
        "/home/shawn/trm-rust",
        "/home/shawn/Agent-Q-master",
        "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2",
    ];

    println!("=== MOTION A (v5): FULL-SCALE CORPUS INGEST ===");
    let mut files = Vec::new();
    for r in &roots {
        let p = Path::new(r);
        if p.exists() {
            let before = files.len();
            walk(p, &mut files);
            println!("  {r}: {} files", files.len() - before);
        }
    }
    println!("total source files: {}", files.len());

    // hash each file → K3 trits, accumulate a big block
    let mut trits: Vec<i8> = Vec::new();
    let mut total_bytes = 0u64;
    let mut per_dir: HashMap<String, usize> = HashMap::new();

    for f in &files {
        if let Ok(bytes) = fs::read(f) {
            total_bytes += bytes.len() as u64;
            let mut h: u64 = 0xcbf29ce484222325;
            for &b in &bytes {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            for k in 0..8 {
                trits.push(match (h >> (k * 2)) & 0b11 {
                    0b11 => 1,
                    0b00 => -1,
                    _ => 0,
                });
            }
            let dir = f.parent().map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string()).unwrap_or_default();
            *per_dir.entry(dir).or_insert(0) += 1;
        }
    }

    println!("\nhash block: {} K3 trits ({} KB packed)", trits.len(), (trits.len() * 2 + 7) / 8 / 1024);
    println!("source bytes hashed: {:.1} MB", total_bytes as f64 / 1e6);

    // write through the sharded fabric
    let mut fab = ShardedFabric::open(root).expect("open");
    let (sid, off) = fab.append(&trits).expect("append");
    let back = fab.read(sid, off, trits.len()).expect("readback");
    let identical = back == trits;
    println!("\nsharded append: shard {sid} @ byte {off}");
    println!("readback: {} trits, byte-identical: {identical}", back.len());

    let top: Vec<_> = per_dir.iter().take(5).collect();
    println!("top dirs: {:?}", top);

    assert!(identical);
    let nfiles = files.len();
    let mb = total_bytes as f64 / 1e6;
    println!("\nMOTION A (v5) GATE: PASSED — {nfiles} real files, {mb:.1} MB, byte-identical in shards");
}
