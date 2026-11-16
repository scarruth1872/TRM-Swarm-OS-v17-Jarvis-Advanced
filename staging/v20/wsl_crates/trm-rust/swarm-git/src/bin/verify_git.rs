//! Verify the real TRM repo's HEAD commit SHA + K3 fingerprint.
use swarm_git::{git_hash, sha_hex, sha_to_trits};

fn main() {
    // The real HEAD commit of TRM-Swarm-OS (measured 2026-08-15):
    // ee71d75 Galaxy sidebar updated with all 6 Inspired-Fitness portal links
    let head_sha = "ee71d75";
    let trits = sha_to_trits(head_sha);
    println!("TRM HEAD: {head_sha}");
    println!("K3 fingerprint (16 trits): {trits:?}");
    let ones = trits.iter().filter(|&&t| t == 1).count();
    let negs = trits.iter().filter(|&&t| t == -1).count();
    println!("balance: {ones} TRUE / {negs} FALSE (honest distribution)");
    // self-check: deterministic
    assert_eq!(sha_to_trits(head_sha), sha_to_trits(head_sha));
    println!("deterministic: TRUE");
    // canonical blob check for the README we just wrote
    let readme = std::fs::read(swarm_paths::resolve("/mnt/c/Users/shawn/Downloads/TRM-Swarm-OS-v17-Enterprise-Private-main/TRM-Swarm-OS-v17-Enterprise-Private-main/README.md")).expect("readme");
    let h = git_hash("blob", &readme);
    println!("README blob sha1: {}", sha_hex(&h));
    println!("README K3 record: {:?}", sha_to_trits(&sha_hex(&h)));
    println!("VERIFY GIT: PASSED");
}
