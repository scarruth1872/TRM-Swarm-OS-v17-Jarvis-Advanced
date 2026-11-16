//! Temporary measurement probe: are two challenges issued in the same clock tick distinct?
//! Run: cargo test -p swarm-thsm --release --test probe_uniqueness -- --nocapture

use std::collections::HashSet;
use swarm_thsm::Thsm;

#[test]
fn measure_challenge_uniqueness() {
    const N: usize = 100_000;
    let mut set: HashSet<[u8; 32]> = HashSet::with_capacity(N);
    let mut pairs_equal = 0usize;
    let mut prev: Option<[u8; 32]> = None;
    let mut time_bytes_equal = 0usize;
    let mut prev_time: Option<[u8; 8]> = None;

    for _ in 0..N {
        let c = Thsm::challenge();
        if let Some(p) = prev {
            if p == c {
                pairs_equal += 1;
            }
        }
        let tb: [u8; 8] = c[..8].try_into().unwrap();
        if let Some(pt) = prev_time {
            if pt == tb {
                time_bytes_equal += 1;
            }
        }
        prev_time = Some(tb);
        prev = Some(c);
        set.insert(c);
    }

    println!("issued          : {N}");
    println!("distinct        : {}", set.len());
    println!("consecutive dup : {pairs_equal}");
    println!("same time bytes : {time_bytes_equal}");
    assert_eq!(set.len(), N, "challenges repeated");
}
