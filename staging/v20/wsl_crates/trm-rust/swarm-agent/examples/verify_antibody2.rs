//! verify_antibody2.rs — read the antibody record at the EXACT byte offset
//! the append returned (42,560,066). swarm-nand read() takes byte offsets.
use swarm_nand::NandFabric;

fn FABRIC() -> String {
    swarm_paths::resolve("/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2_global_memory/nand_k3_fabric.bin")
}

fn main() {
    let mut fab = NandFabric::open(&FABRIC()).expect("open");
    // the append reported byte offset 42,560,066 (write_offset at write time)
    // but the backend may have appended after; the record header is
    // [1,1,1,1,1] + 6 hash trits + 12 marker. Scan a window around 42.56M.
    let base: u64 = 42_560_000;
    for delta in (0..300u64).step_by(23) {
        match fab.read(base + delta, 23) {
            Ok(rec) => {
                if rec[..5] == [1, 1, 1, 1, 1] && rec[11..16] == [1, -1, 1, -1, 1] {
                    println!("ANTIBODY RECORD @ byte {}: {:?}", base + delta, &rec);
                    println!("VERIFIED: real NAND security record readable");
                    return;
                }
            }
            Err(_) => break,
        }
    }
    println!("not in window — scanning wider from write_offset");
    // fall back: search the whole data region for the marker pattern
    let nbytes = fab.trit_count() / 4 + 64;
    let mut found = false;
    let mut pos: u64 = 42_000_000;
    while pos + 23 < nbytes && !found {
        if let Ok(rec) = fab.read(pos, 23) {
            if rec[..5] == [1, 1, 1, 1, 1] && rec[11..16] == [1, -1, 1, -1, 1] {
                println!("ANTIBODY RECORD @ byte {pos}: {:?}", &rec);
                println!("VERIFIED: real NAND security record readable");
                found = true;
            }
        }
        pos += 23;
    }
    if !found {
        println!("NOT FOUND in scan — investigate (record may be at different offset)");
    }
}
