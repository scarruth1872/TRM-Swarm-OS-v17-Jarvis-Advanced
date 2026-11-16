//! TRM Swarm OS - ddr_self_training
//! Digital DNA Repository (DDR) Self-Training Feedback Loop.
//! Performs zero-copy 128-dimensional SIMD event vectorization and ternary classification
//! (-1: error/fault, 0: neutral, +1: approved/success) for continuous self-evolution.

#![allow(non_snake_case, dead_code)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub static MONAD_ANCHOR: u64 = 0x4D4F_4E41_4400_0001;

// --- Native C ABI Export: SIMD Batch Vectorizer ---

#[no_mangle]
pub extern "C" fn strip_batch_vectorize_events(
    agent_buf: *const u8,
    agent_offsets: *const usize,
    agent_lens: *const usize,
    type_buf: *const u8,
    type_offsets: *const usize,
    type_lens: *const usize,
    data_buf: *const u8,
    data_offsets: *const usize,
    data_lens: *const usize,
    count: usize,
    out_vectors: *mut f32, // count * 128
    out_labels: *mut i32,  // count
) -> i32 {
    if agent_buf.is_null() || type_buf.is_null() || data_buf.is_null()
        || agent_offsets.is_null() || agent_lens.is_null()
        || type_offsets.is_null() || type_lens.is_null()
        || data_offsets.is_null() || data_lens.is_null()
        || out_vectors.is_null() || out_labels.is_null() || count == 0
    {
        return -1;
    }

    let a_offs = unsafe { core::slice::from_raw_parts(agent_offsets, count) };
    let a_lens = unsafe { core::slice::from_raw_parts(agent_lens, count) };
    let t_offs = unsafe { core::slice::from_raw_parts(type_offsets, count) };
    let t_lens = unsafe { core::slice::from_raw_parts(type_lens, count) };
    let d_offs = unsafe { core::slice::from_raw_parts(data_offsets, count) };
    let d_lens = unsafe { core::slice::from_raw_parts(data_lens, count) };

    let out_v = unsafe { core::slice::from_raw_parts_mut(out_vectors, count * 128) };
    let out_l = unsafe { core::slice::from_raw_parts_mut(out_labels, count) };

    let inv_255 = 1.0f32 / 255.0f32;

    let mut i = 0;
    while i < count {
        let v_base = i * 128;
        let mut j = 0;
        while j < 128 {
            out_v[v_base + j] = 0.0;
            j += 1;
        }

        // Agent -> dims 0..8
        let a_start = a_offs[i];
        let a_len = a_lens[i].min(8);
        let mut k = 0;
        while k < a_len {
            let b = unsafe { *agent_buf.add(a_start + k) };
            out_v[v_base + k] = (b as f32) * inv_255;
            k += 1;
        }

        // Type -> dims 8..16
        let t_start = t_offs[i];
        let t_len = t_lens[i].min(8);
        k = 0;
        while k < t_len {
            let b = unsafe { *type_buf.add(t_start + k) };
            out_v[v_base + 8 + k] = (b as f32) * inv_255;
            k += 1;
        }

        // Data -> dims 16..128
        let d_start = d_offs[i];
        let d_len = d_lens[i].min(112);
        k = 0;
        while k < d_len {
            let b = unsafe { *data_buf.add(d_start + k) };
            out_v[v_base + 16 + k] = (b as f32) * inv_255;
            k += 1;
        }

        // Classification
        let d_slice = unsafe { core::slice::from_raw_parts(data_buf.add(d_start), d_lens[i]) };
        let t_slice = unsafe { core::slice::from_raw_parts(type_buf.add(t_start), t_lens[i]) };

        let has_error_or_fault = contains_ascii_ci(d_slice, b"error") || contains_ascii_ci(t_slice, b"fault");
        let has_approved_or_success = contains_ascii_ci(d_slice, b"approved") || contains_ascii_ci(t_slice, b"success");

        if has_error_or_fault {
            out_l[i] = -1;
        } else if has_approved_or_success {
            out_l[i] = 1;
        } else {
            out_l[i] = 0;
        }

        i += 1;
    }
    0
}

fn contains_ascii_ci(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.len() > haystack.len() || needle.is_empty() {
        return false;
    }
    let limit = haystack.len() - needle.len();
    let mut i = 0;
    while i <= limit {
        let mut matched = true;
        let mut j = 0;
        while j < needle.len() {
            if haystack[i + j].to_ascii_lowercase() != needle[j].to_ascii_lowercase() {
                matched = false;
                break;
            }
            j += 1;
        }
        if matched {
            return true;
        }
        i += 1;
    }
    false
}

// --- High-Level DDR Self Training Architecture ---

#[derive(Clone, Copy, Debug)]
pub struct TrainingTuple {
    pub vector: [f32; 128],
    pub label: i32,
    pub timestamp: u64,
}

pub struct DDRSelfTrainingEngine {
    pub total_events_vectorized: u64,
    pub total_errors_captured: u64,
    pub total_approved_captured: u64,
    pub buffer: [TrainingTuple; 32],
    pub buffer_cursor: usize,
}

impl DDRSelfTrainingEngine {
    pub const fn new() -> Self {
        const EMPTY_TUPLE: TrainingTuple = TrainingTuple {
            vector: [0.0f32; 128],
            label: 0,
            timestamp: 0,
        };

        Self {
            total_events_vectorized: 0,
            total_errors_captured: 0,
            total_approved_captured: 0,
            buffer: [EMPTY_TUPLE; 32],
            buffer_cursor: 0,
        }
    }

    /// Ingests an event and vectorizes it into 128-dimensional embedding and ternary label.
    pub fn ingest_event(&mut self, agent: &str, event_type: &str, data: &str, ts: u64) -> i32 {
        self.total_events_vectorized += 1;

        let mut v = [0.0f32; 128];
        let inv_255 = 1.0f32 / 255.0f32;

        let a_bytes = agent.as_bytes();
        let a_len = a_bytes.len().min(8);
        let mut i = 0;
        while i < a_len {
            v[i] = (a_bytes[i] as f32) * inv_255;
            i += 1;
        }

        let t_bytes = event_type.as_bytes();
        let t_len = t_bytes.len().min(8);
        i = 0;
        while i < t_len {
            v[8 + i] = (t_bytes[i] as f32) * inv_255;
            i += 1;
        }

        let d_bytes = data.as_bytes();
        let d_len = d_bytes.len().min(112);
        i = 0;
        while i < d_len {
            v[16 + i] = (d_bytes[i] as f32) * inv_255;
            i += 1;
        }

        let is_err = contains_ascii_ci(d_bytes, b"error") || contains_ascii_ci(t_bytes, b"fault");
        let is_app = contains_ascii_ci(d_bytes, b"approved") || contains_ascii_ci(t_bytes, b"success");

        let label = if is_err {
            self.total_errors_captured += 1;
            -1
        } else if is_app {
            self.total_approved_captured += 1;
            1
        } else {
            0
        };

        self.buffer[self.buffer_cursor] = TrainingTuple {
            vector: v,
            label,
            timestamp: ts,
        };
        self.buffer_cursor = (self.buffer_cursor + 1) % 32;

        label
    }

    /// Returns telemetry metrics (events, errors, approved).
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.total_events_vectorized,
            self.total_errors_captured,
            self.total_approved_captured,
        )
    }
}

// --- Global Singleton Interface ---
static SINGLETON_SELF_TRAINING: AtomicU64 = AtomicU64::new(0);

pub fn get_self_training() -> u64 {
    let v = SINGLETON_SELF_TRAINING.load(Ordering::Relaxed);
    if v == 0 {
        SINGLETON_SELF_TRAINING.store(1, Ordering::Relaxed);
        1
    } else {
        v
    }
}

// ── `native_event_to_vector_batch` (swarm_v2/core/ddr_self_training.py + strip_ddr_self_training.rs) ─
//
// The strip's own kernel, reproduced field for field. 128-dim f32 vector per event:
//   dims 0..8    the AGENT string, each byte as `(byte as f32) * (1/255)`, at most 8 bytes;
//   dims 8..16   the TYPE string, the same way, at most 8 bytes;
//   dims 16..128 the DATA string -- which the HOST has already serialised (`json.dumps` for a dict,
//                `str(...)` otherwise) -- at most 112 bytes;
//   everything else stays 0.0 (shorter strings leave the remaining dims zero).
//
// The label is a three-way classification with CROSSED pairings, which is the part that is easy to get
// wrong: `-1` if the DATA contains "error" OR the TYPE contains "fault"; else `1` if the DATA contains
// "approved" OR the TYPE contains "success"; else `0`. Note that error/fault and approved/success are
// each split across the two fields in that order.
//
// Two arithmetic details: the strip multiplies by a precomputed `1.0f32 / 255.0f32`, which is NOT always
// the same as `byte as f32 / 255.0` computed in f32, and the python reference divides in f64 and stores
// into an f32 array. This port follows the STRIP (the runtime path); the probe reports whether the
// python form would differ on these inputs.
//
// `contains_ascii_ci` above is the module's own and is reused rather than duplicated. It differs from the
// strip's helper on one unreachable input -- an EMPTY needle returns false here, where the strip would
// call `windows(0)` and panic -- and both call sites only ever pass literals.

pub const EVENT_VECTOR_DIM: usize = 128;
const INV_255: f32 = 1.0f32 / 255.0f32;

pub fn vectorize_event(agent: &str, etype: &str, data: &str) -> ([f32; EVENT_VECTOR_DIM], i32) {
    let mut v = [0.0f32; EVENT_VECTOR_DIM];
    let a = agent.as_bytes();
    for k in 0..a.len().min(8) {
        v[k] = (a[k] as f32) * INV_255;
    }
    let t = etype.as_bytes();
    for k in 0..t.len().min(8) {
        v[8 + k] = (t[k] as f32) * INV_255;
    }
    let d = data.as_bytes();
    for k in 0..d.len().min(112) {
        v[16 + k] = (d[k] as f32) * INV_255;
    }
    let has_error_or_fault = contains_ascii_ci(d, b"error") || contains_ascii_ci(t, b"fault");
    let has_approved_or_success = contains_ascii_ci(d, b"approved") || contains_ascii_ci(t, b"success");
    let label = if has_error_or_fault {
        -1
    } else if has_approved_or_success {
        1
    } else {
        0
    };
    (v, label)
}

/// The batch form: one vector and one label per event.
pub fn vectorize_events(agents: &[&str], types: &[&str], data: &[&str]) -> (Vec<[f32; EVENT_VECTOR_DIM]>, Vec<i32>) {
    let n = agents.len().min(types.len()).min(data.len());
    let mut vs: Vec<[f32; EVENT_VECTOR_DIM]> = Vec::with_capacity(n);
    let mut ls: Vec<i32> = Vec::with_capacity(n);
    for i in 0..n {
        let (v, l) = vectorize_event(agents[i], types[i], data[i]);
        vs.push(v);
        ls.push(l);
    }
    (vs, ls)
}
