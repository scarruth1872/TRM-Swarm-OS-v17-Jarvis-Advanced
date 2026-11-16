//! Probe: what a second handle sees of a record appended through the first.
//!
//! Run with: cargo run -p swarm-nand --example probe_reopen
//! Prints the file size and the on-disk header after each step, so the reopen contract is observed
//! rather than reasoned about.
use swarm_nand::NandFabric;
use std::fs;

fn size(p: &str) -> u64 {
    fs::metadata(p).map(|m| m.len()).unwrap_or(999)
}

fn main() {
    let p = std::env::temp_dir().join("probe_reopen.k3");
    let p = p.to_str().unwrap().to_string();
    let _ = fs::remove_file(&p);

    let mut fab = NandFabric::open(&p).expect("open");
    println!("after open (new): size={} trit_count={}", size(&p), fab.trit_count());

    let off = fab.append(&[1, -1, 1, 1, 0, -1, 1]).expect("append");
    println!("after append: off={off} size={} trit_count={}", size(&p), fab.trit_count());

    let mut fab2 = NandFabric::open(&p).expect("reopen");    println!("reopened: size={} trit_count={}", size(&p), fab2.trit_count());
    match fab2.read(off, 7) {
        Ok(v) => println!("READ OK: {v:?}"),
        Err(e) => println!("READ ERR: {e}"),
    }

    fab.flush().expect("flush");
    let mut fab3 = NandFabric::open(&p).expect("third open");
    println!("after flush: size={} trit_count={}", size(&p), fab3.trit_count());
    match fab3.read(off, 7) {
        Ok(v) => println!("READ BACK OK: {v:?}"),
        Err(e) => println!("READ BACK ERR: {e}"),
    }
    let _ = fs::remove_file(&p);
}
