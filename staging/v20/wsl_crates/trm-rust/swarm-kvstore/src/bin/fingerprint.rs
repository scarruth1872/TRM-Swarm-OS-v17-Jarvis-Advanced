//! Print the Rust kvstore key fingerprints for cross-check.
use swarm_kvstore::KvStore;
fn main() {
    for key in ["jarvis.monad", "swarm.boot", "trm.protocol"] {
        println!("{} {:?}", key, KvStore::key_trits(key));
    }
}
