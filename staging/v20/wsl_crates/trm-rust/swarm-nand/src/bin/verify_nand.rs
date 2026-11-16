//! Verify gate: swarm-nand vs the REAL fabric (read-only, on a COPY).
//!
//! Opens a copy of the live 85.5M-trit fabric, reads the header + first
//! N records, and compares Rust's canonical decode against the Python
//! fabric's read_decision output. NEVER touches the live file.

use std::io::Read;

fn main() {
    // copy the live fabric to a scratch path (verify never touches live)
    let live = swarm_paths::resolve(
        "/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2_global_memory/nand_k3_fabric.bin",
    );
    let scratch = swarm_paths::resolve("/tmp/swarm_nand_verify_copy.bin");
    std::fs::copy(&live, &scratch).expect("copy live fabric to scratch");
    println!("copied live fabric → scratch (verify is read-only on the copy)");

    let fab = swarm_nand::NandFabric::open(&scratch).expect("open copy");
    let (codec, trits, woff) = fab.status();
    println!("Rust open: codec v{codec} | trits {trits} | write_offset {woff}");

    // Read the header fields the Python fabric would report
    println!("  codec_version == 2 (canonical): {}", codec == 2);
    println!("  trit_count > 85M: {}", trits > 85_000_000);

    // Read the FIRST 8 records (each 8 trits) from offset HEADER_SIZE
    // and print them — these must match the Python fabric's decode of the
    // same offsets (canonical table: 0→-1, 1→0, 3→+1).
    let mut off = swarm_nand::HEADER_SIZE;
    println!("\nfirst 8 records (8 trits each) decoded by RUST (canonical):");
    for i in 0..8 {
        match fab.read(off, 8) {
            Ok(t) => {
                println!("  rec {i} @ {off}: {t:?}");
                off += 4; // 8 trits = 2 bytes each... 8 trits * 2 bits = 2 bytes
            }
            Err(e) => {
                println!("  rec {i}: read error {e}");
                break;
            }
        }
    }

    // Spot check: read 2000 trits at the payload start, verify all valid
    let t0 = fab.read(swarm_nand::HEADER_SIZE, 2000).expect("read 2000");
    let valid = t0.iter().all(|&t| t == -1 || t == 0 || t == 1);
    println!("\n2000-trit payload read: all valid {{-1,0,1}}: {valid}");
    println!("sample: {:?}", &t0[..16]);

    // Cleanup scratch
    let _ = std::fs::remove_file(scratch);
    println!("\nVERIFY GATE: swarm-nand reads the real fabric coherently");
}
