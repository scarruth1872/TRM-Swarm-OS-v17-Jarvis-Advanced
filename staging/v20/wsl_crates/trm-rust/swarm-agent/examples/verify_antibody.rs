//! verify_antibody.rs — read back the antibody record via swarm-nand
//! (same crate that wrote it — authoritative byte-offset semantics).
use swarm_nand::NandFabric;

fn FABRIC() -> String {
    swarm_paths::resolve("/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/swarm_v2_global_memory/nand_k3_fabric.bin")
}

fn main() {
    let mut fab = NandFabric::open(&FABRIC()).expect("open");
    let total = fab.trit_count();
    println!("fabric: {total} trits");
    // the antibody record was 23 trits; read the LAST 40 trits (it's the
    // most recent append before any backend writes) — search for the marker
    let tail = fab.read(total - 64, 64).expect("read tail");
    // find the antibody header [1,1,1,1,1] in the tail
    for i in 0..tail.len().saturating_sub(23) {
        if tail[i..i + 5] == [1, 1, 1, 1, 1] {
            println!("antibody header found at tail+{i}: {:?}", &tail[i..i + 23]);
            println!("VERIFIED: real antibody record readable via swarm-nand");
            return;
        }
    }
    println!("tail: {tail:?}");
    println!("NOT FOUND in last 64 trits — checking further back");
    let tail2 = fab.read(total - 256, 256).expect("read tail2");
    for i in 0..tail2.len().saturating_sub(23) {
        if tail2[i..i + 5] == [1, 1, 1, 1, 1] {
            println!("antibody header found: {:?}", &tail2[i..i + 23]);
            println!("VERIFIED: real antibody record readable via swarm-nand");
            return;
        }
    }
    println!("antibody record NOT located in last 256 trits — investigate");
}
