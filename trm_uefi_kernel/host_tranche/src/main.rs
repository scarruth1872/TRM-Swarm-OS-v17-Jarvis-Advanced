mod radix_trie_matcher;
pub mod time { use std::time::Instant; static T0: std::sync::OnceLock<Instant> = std::sync::OnceLock::new(); pub fn now_ns() -> u64 { let t0 = *T0.get_or_init(Instant::now); t0.elapsed().as_nanos() as u64 } }

fn main() {
    println!("TRANCHE_HARNESS radix_trie_matcher ready");
}
