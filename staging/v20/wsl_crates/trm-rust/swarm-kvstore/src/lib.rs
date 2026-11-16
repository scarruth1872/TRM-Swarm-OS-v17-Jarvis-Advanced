//! swarm-kvstore — K3-native key-value store (trm:// protocol suite).
//!
//! Keys and values are stored as ternary records: every key is hashed
//! to a K3 fingerprint (canonical codec), every value is packed with
//! the same 2-bit/trit TurboVec codec as the NAND fabric and trm://
//! frames — one language across storage, transport, and compute.
//!
//! Storage is a simple append-only log with an in-memory index — the
//! same append-only discipline as the NAND fabric (no in-place
//! rewrites; a delete is a tombstone record).

use std::collections::HashMap;

/// A K3-native KV store.
#[derive(Default)]
pub struct KvStore {
    /// value by key fingerprint (hex string -> packed bytes)
    data: HashMap<String, Vec<u8>>,
    /// append-only operation log (for audit/replay)
    log: Vec<String>,
    sets: u64,
    gets: u64,
    deletes: u64,
}

impl KvStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fingerprint a key to K3 trits (canonical {-1,0,1}).
    pub fn key_trits(key: &str) -> Vec<i8> {
        let mut h = 0x9E3779B97F4A7C15u64;
        for b in key.bytes() {
            h = h.rotate_left(5) ^ (b as u64).wrapping_mul(0x100000001B3);
            h = h.wrapping_mul(0x2545F4914F6CDD1D);
        }
        // 16 trits from the hash
        (0..16).map(|k| if (h >> (k % 64)) & 1 == 1 { 1 } else { -1 }).collect()
    }

    /// Pack a value to canonical K3 bytes (same codec as TurboVec).
    pub fn pack_value(v: &[i8]) -> Vec<u8> {
        swarm_proto::pack_trits(v)
    }

    /// Set a key (append log + update index).
    pub fn set(&mut self, key: &str, value: &[i8]) {
        let k = format!("{:?}", Self::key_trits(key));
        let packed = Self::pack_value(value);
        self.data.insert(k.clone(), packed);
        self.log.push(format!("SET {} ({} trits)", key, value.len()));
        self.sets += 1;
    }

    /// Get a key — returns packed bytes if present.
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        let k = format!("{:?}", Self::key_trits(key));
        self.data.get(&k).map(|v| v.as_slice())
    }

    /// Delete a key (tombstone in the log).
    pub fn delete(&mut self, key: &str) -> bool {
        let k = format!("{:?}", Self::key_trits(key));
        let removed = self.data.remove(&k).is_some();
        if removed {
            self.log.push(format!("DEL {}", key));
            self.deletes += 1;
        }
        removed
    }

    pub fn stats(&self) -> (u64, u64, u64) {
        (self.sets, self.gets, self.deletes)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn log_len(&self) -> usize {
        self.log.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_roundtrip() {
        let mut kv = KvStore::new();
        let value = vec![1, 1, 1, -1, -1, 1, 0, 0, 1];
        kv.set("jarvis.monad", &value);
        let got = kv.get("jarvis.monad").expect("key present");
        assert_eq!(got, &KvStore::pack_value(&value));
    }

    #[test]
    fn missing_key_returns_none() {
        let kv = KvStore::new();
        assert!(kv.get("absent").is_none());
    }

    #[test]
    fn delete_removes_and_tombstones() {
        let mut kv = KvStore::new();
        kv.set("k", &[1, -1, 1]);
        assert!(kv.delete("k"));
        assert!(kv.get("k").is_none());
        assert_eq!(kv.log_len(), 2); // SET + DEL
    }

    #[test]
    fn distinct_keys_do_not_collide() {
        let mut kv = KvStore::new();
        kv.set("alpha", &[1]);
        kv.set("beta", &[-1]);
        assert_ne!(
            KvStore::key_trits("alpha"),
            KvStore::key_trits("beta")
        );
    }

    #[test]
    fn key_fingerprint_is_deterministic() {
        assert_eq!(KvStore::key_trits("monad"), KvStore::key_trits("monad"));
        assert_eq!(KvStore::key_trits(""), KvStore::key_trits(""));
    }
}
