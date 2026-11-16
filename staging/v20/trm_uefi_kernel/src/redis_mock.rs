//! TRM Swarm OS — redis_mock (bare-metal no_std + alloc).
//! Phase 38 | Rank 181 | TurboVec 16x BT-Quant | Monad Anchor: 0x4D4F4E4144000001
//! In-memory persistent key-value, hash, and list storage engine for IPC/Nexus bridge.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};

pub const MONAD_ANCHOR: u64 = 0x4D4F4E4144000001;
pub const PHASE: u32 = 38;
pub const RANK: u32 = 181;
pub const SCALE_FACTOR: f32 = 31.8784;

pub static TRIT_WEIGHTS: &[i8] = &[
    -1, 0, 0, -1, 0, -1, 0, 1, 1, -1, 1, 0, 0, 0, 1, -1, 1, -1, 1, -1, 1, -1, 1, -1, -1, 1, 1, 1, -1, 1, 1, -1,
    -1, -1, 1, -1, 0, 0, 1, 1, 1, -1, 1, 1, 0, 0, 1, -1, 1, -1, 1, 0, 1, -1, 0, -1, -1, 1, 1, 0, -1, 0, 1, -1,
    -1, 0, 1, 0, 0, -1, 1, 1, 1, -1, 1, 1, 1, 1, 1, 0, 0, 0, 0, -1, 1, 0, 1, -1, 0, 1, 0, 1, -1, 0, 0, -1,
    -1, -1, 0, -1, 1, -1, 1, 0, 0, -1, 1, 0, 1, 0, 0, 0, 1, -1, 1, 0, 1, -1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0,
    -1, -1, 1, 0, 1, 0, 1, 1, 0, -1, 1, 1, 1, 1, 0, 0, 0, -1, 1, -1, 1, -1, 0, -1, -1, 1, 0, 1, -1, 0, 1, -1,
    0, 0, 1, 0, 0, -1, 1, 1, 1, 0, 1, 1, 1, 1, 0, -1, 0, 0, 1, -1, 1, -1, 1, 0, -1, 1, 1, 1, -1, 1, 1, 0,
    -1, -1, 1, -1, 0, 0, 1, 1, 1, -1, 1, 1, 1, 1, 0, 0, 0, 0, 0, -1, 1, -1, 1, -1, -1, 1, 0, 1, -1, 1, 1, -1,
    -1, -1, 1, -1, 1, -1, 0, 1, 1, -1, 1, 0, 1, 1, 1, 0, 1, -1, 1, -1, 1, -1, 1, 0, -1, 0, 1, 1, -1, 0, 1, -1
];

static INVOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
static SINGLETON_REDIS_MOCK: AtomicU64 = AtomicU64::new(0);

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum K3 {
    False = 0,
    Unknown = 1,
    True = 2,
}

impl K3 {
    pub fn and(self, other: K3) -> K3 {
        match (self as u8).min(other as u8) {
            0 => K3::False,
            1 => K3::Unknown,
            _ => K3::True,
        }
    }
}

#[inline(always)]
pub fn ternary_dot(inputs: &[f32]) -> f32 {
    let n = inputs.len().min(TRIT_WEIGHTS.len());
    let mut acc = 0.0f32;
    for i in 0..n {
        acc += inputs[i] * (TRIT_WEIGHTS[i] as f32) * SCALE_FACTOR / 16.0;
    }
    acc
}

pub fn redis_mock_execute(inputs: &[f32], output: &mut [f32]) -> u32 {
    if inputs.is_empty() || output.is_empty() {
        return 0xDEAD_BEEF;
    }
    INVOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
    let r = ternary_dot(inputs);
    let k = if r.abs() < 1e-3 {
        K3::Unknown
    } else if r > 0.0 {
        K3::True
    } else {
        K3::False
    };
    let _ = k.and(K3::True);
    output[0] = r;
    0x0000_0001
}

#[derive(Debug, Clone)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct HashEntry {
    pub table: String,
    pub field: String,
    pub value: String,
}

pub struct PersistentRedisMock {
    pub strings: Vec<KeyValueEntry>,
    pub hashes: Vec<HashEntry>,
    pub lists: Vec<(String, Vec<String>)>,
}

impl PersistentRedisMock {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            hashes: Vec::new(),
            lists: Vec::new(),
        }
    }

    pub fn ping(&self) -> bool {
        true
    }

    pub fn set(&mut self, key: &str, value: &str) -> bool {
        for entry in self.strings.iter_mut() {
            if entry.key == key {
                entry.value = String::from(value);
                return true;
            }
        }
        self.strings.push(KeyValueEntry {
            key: String::from(key),
            value: String::from(value),
        });
        true
    }

    pub fn get(&self, key: &str) -> Option<String> {
        for entry in self.strings.iter() {
            if entry.key == key {
                return Some(entry.value.clone());
            }
        }
        None
    }

    pub fn hset(&mut self, table: &str, field: &str, value: &str) -> u32 {
        for entry in self.hashes.iter_mut() {
            if entry.table == table && entry.field == field {
                entry.value = String::from(value);
                return 0;
            }
        }
        self.hashes.push(HashEntry {
            table: String::from(table),
            field: String::from(field),
            value: String::from(value),
        });
        1
    }

    pub fn hget(&self, table: &str, field: &str) -> Option<String> {
        for entry in self.hashes.iter() {
            if entry.table == table && entry.field == field {
                return Some(entry.value.clone());
            }
        }
        None
    }

    pub fn rpush(&mut self, name: &str, value: &str) -> usize {
        for (list_name, items) in self.lists.iter_mut() {
            if list_name == name {
                items.push(String::from(value));
                return items.len();
            }
        }
        self.lists.push((String::from(name), vec![String::from(value)]));
        1
    }

    pub fn lrange(&self, name: &str, start: usize, stop: usize) -> Vec<String> {
        for (list_name, items) in self.lists.iter() {
            if list_name == name {
                let end = stop.min(items.len());
                if start < end {
                    return items[start..end].to_vec();
                }
            }
        }
        Vec::new()
    }
}

pub fn get_redis_mock() -> u64 {
    let v = SINGLETON_REDIS_MOCK.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_REDIS_MOCK.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

pub fn redis_mock_set(key: &str, val: &str) -> bool {
    let mut store = PersistentRedisMock::new();
    store.set(key, val)
}

pub fn redis_mock_get(key: &str) -> Option<String> {
    let store = PersistentRedisMock::new();
    store.get(key)
}


