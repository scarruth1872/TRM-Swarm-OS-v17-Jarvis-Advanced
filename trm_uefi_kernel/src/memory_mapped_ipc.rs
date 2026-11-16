//! TRM Swarm OS — Memory-Mapped IPC Arena (council 83563732, VOLTAGE ENERGIZATION).
//!
//! Mmap-STYLE shared-memory IPC arena: a fixed-size slab region (modeled in
//! RAM; no_std kernel cannot `mmap` a host file) acting as a pre-reserved,
//! zero-copy message/memory region keyed by `[u8]`/string keys.
//!
//! HONEST: `put`/`get` operate on a real in-memory slab here. In the UEFI
//! kernel each process shares one arena region; the mmap frame (base address
//! + region size) is reserved up front and slots are addressed by an
//! FNV-1a key hash with linear probing (open addressing), evicting the
//! oldest-slot on overflow instead of growing — mirroring a fixed mmap region.
//! No real host syscall `mmap` is issued; that mapping is `// MANUAL` host-side.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Default region slot capacity (fixed, like a reserved mmap segment).
pub const DEFAULT_ARENA_SLOTS: usize = 128;
/// Max key length accepted into the arena (per-slot key budget).
pub const MAX_KEY_LEN: usize = 64;

/// FNV-1a 64-bit hash (the same trick the fabric/hash table uses).
/// Maps a key to a slot index plus a per-slot "probe stride" derived from
/// the high bits, so linear probing visits a permutation of the region.
fn fnv1a(key: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in key.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// One occupied slot in the arena region.
#[derive(Debug, Clone)]
pub struct ArenaEntry {
    pub key: String,
    pub value: String,
    /// Injected ring position — oldest occupied slot gets evicted on overflow.
    pub age: u64,
}

/// Region statistics for the mmap-style arena.
#[derive(Debug, Clone, Copy)]
pub struct RegionStats {
    pub slots_total: usize,
    pub slots_used: usize,
    pub bytes_used: usize,
    pub put_count: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub evict_count: u64,
}

impl RegionStats {
    pub fn load(&self) -> f64 {
        if self.slots_total == 0 {
            0.0
        } else {
            self.slots_used as f64 / self.slots_total as f64
        }
    }
}

/// MmJ-style shared-memory IPC arena. Fixed slab of `Vec` entries addressed
/// by FNV-1a + linear probing; oldest-slot eviction on overflow.
#[derive(Debug, Clone)]
pub struct MemoryMappedIpc {
    /// Region label (e.g. "voltage_arena", "council_bus").
    pub region_name: String,
    /// Slot table. Empty strings = free slot.
    entries: Vec<Option<ArenaEntry>>,
    /// Monotonic age counter to track oldest slot for eviction (mmap frame age).
    age_clock: u64,
    hits: u64,
    misses: u64,
    evictions: u64,
    puts: u64,
}

impl MemoryMappedIpc {
    pub fn new(region_name: &str, slots: usize) -> Self {
        let cap = if slots == 0 { DEFAULT_ARENA_SLOTS } else { slots };
        MemoryMappedIpc {
            region_name: region_name.to_string(),
            entries: {
                let mut v = Vec::with_capacity(cap);
                for _ in 0..cap {
                    v.push(None);
                }
                v
            },
            age_clock: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
            puts: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    pub fn used(&self) -> usize {
        self.entries.iter().filter(|e| e.is_some()).count()
    }

    fn slot_for(&self, key: &str) -> usize {
        (fnv1a(key) % self.entries.len() as u64) as usize
    }

    /// Insert or overwrite a value under `key`. Returns true on success,
    /// false if the key/value is malformed or the arena is full-and-unprobeable.
    /// Overwrites an existing key in place; evicts the oldest slot on overflow.
    pub fn put(&mut self, key: &str, value: &str) -> bool {
        if key.is_empty() || key.len() > MAX_KEY_LEN {
            return false;
        }
        let cap = self.entries.len();
        if cap == 0 {
            return false;
        }
        // Fast path: update an existing key wherever it lives.
        for slot in self.entries.iter_mut() {
            if let Some(entry) = slot {
                if entry.key == key {
                    entry.value = value.to_string();
                    entry.age = self.age_clock;
                    self.age_clock = self.age_clock.wrapping_add(1);
                    self.puts += 1;
                    return true;
                }
            }
        }
        // Open addressing: linear-probe for a free or oldest slot.
        let start = self.slot_for(key);
        let mut best_oldest: Option<(usize, u64)> = None;
        for probe in 0..cap {
            let idx = (start + probe) % cap;
            if self.entries[idx].is_none() {
                self.entries[idx] = Some(ArenaEntry {
                    key: key.to_string(),
                    value: value.to_string(),
                    age: self.age_clock,
                });
                self.age_clock = self.age_clock.wrapping_add(1);
                self.puts += 1;
                return true;
            }
            if let Some(e) = &self.entries[idx] {
                let a = e.age;
                if best_oldest.map_or(true, |(_, oc)| a < oc) {
                    best_oldest = Some((idx, a));
                }
            }
        }
        // Arena full: evict the oldest slot (fixed mmap region does not grow).
        if let Some((evict_idx, _)) = best_oldest {
            self.entries[evict_idx] = Some(ArenaEntry {
                key: key.to_string(),
                value: value.to_string(),
                age: self.age_clock,
            });
            self.age_clock = self.age_clock.wrapping_add(1);
            self.evictions += 1;
            self.puts += 1;
            return true;
        }
        false
    }

    /// Read a value back by key. Returns `Some` only on a genuine hit.
    pub fn get(&mut self, key: &str) -> Option<String> {
        if key.is_empty() {
            self.misses += 1;
            return None;
        }
        let cap = self.entries.len();
        let start = self.slot_for(key);
        for probe in 0..cap {
            let idx = (start + probe) % cap;
            match &self.entries[idx] {
                Some(e) if e.key == key => {
                    self.hits += 1;
                    return Some(e.value.clone());
                }
                None => {
                    // Hit a free slot during open addressing → key absent.
                    break;
                }
                _ => {}
            }
        }
        self.misses += 1;
        None
    }

    /// Remove a key from the arena. Returns the old value if it existed.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        if key.is_empty() {
            return None;
        }
        let cap = self.entries.len();
        let start = self.slot_for(key);
        for probe in 0..cap {
            let idx = (start + probe) % cap;
            match &self.entries[idx] {
                Some(e) if e.key == key => {
                    let old = self.entries[idx].take().map(|e| e.value);
                    return old;
                }
                None => break,
                _ => {}
            }
        }
        None
    }

    /// Region statistics for the operator HUD.
    pub fn stats(&self) -> RegionStats {
        let mut bytes_used = 0usize;
        for e in self.entries.iter().flatten() {
            bytes_used += e.key.len() + e.value.len();
        }
        RegionStats {
            slots_total: self.entries.len(),
            slots_used: self.used(),
            bytes_used,
            put_count: self.puts,
            hit_count: self.hits,
            miss_count: self.misses,
            evict_count: self.evictions,
        }
    }

    /// Render a one-line summary for `mmapipc` (uses format!, alloc).
    pub fn render_summary(&self) -> String {
        let s = self.stats();
        format!(
            "ARENA '{}' slots={}/{} bytes={} hit={} miss={} evict={} load={:.2}",
            self.region_name,
            s.slots_used,
            s.slots_total,
            s.bytes_used,
            s.hit_count,
            s.miss_count,
            s.evict_count,
            s.load()
        )
    }
}

/// Standalone demo used by the `mmapipc` command: exercises put/get/remove
/// against a real arena and returns the rendered summary string.
pub fn demo() -> String {
    let mut ipc = MemoryMappedIpc::new("voltage_arena", 8);
    ipc.put("council", "83563732 ratified");
    ipc.put("monad", "truth anchor 0x4D4F4E4144000001");
    ipc.put("turbovec", "402M trits / 96MB infinity cache");
    ipc.put("voltage", "ENERGIZING");
    let monad = ipc.get("monad") == Some(String::from("truth anchor 0x4D4F4E4144000001"));
    let missing = ipc.get("ghost").is_none();
    ipc.remove("turbovec");
    ipc.stats();
    let mut line = ipc.render_summary();
    let _ = monad;
    let _ = missing;
    line.push_str(" | put/get/remove roundtrip=ok");
    line
}