//! swarm-kernel — the Rust kernel library (cdylib).
//!
//! The absorption: exports the SAME C ABI symbols the Python platform
//! binds (c_mamba3_*, c_kleene_*, c_instella_*, c_nand_*), backed by
//! the VERIFIED Rust crates. Swap the .so path → the Rust kernel becomes
//! the live execution path without touching the 185 Python modules.
//!
//! Symbols implemented (Rust-backed):
//!   - c_mamba3_create_state / c_mamba3_step / c_mamba3_destroy_state
//!   - c_kleene_quantize_avx2  (K3 collapse — verified)
//!   - c_instella_gate_topk    (sparse gate — verified)
//!   - c_nand_append / c_nand_read / c_nand_trit_count (fabric)
#![crate_type = "cdylib"]

pub mod pipeline;
pub mod batch_persist;
pub mod batch_swarm;
pub mod batch_council;

use std::ffi::c_void;
use std::os::raw::{c_char, c_float, c_int, c_uint, c_ulonglong};

// ─── Mamba-3 SSM, backed by swarm-ssm (the crate whose gate is byte-for-byte the C reference) ───
//
// The C ABI is `c_mamba3_step(handle, const float* input_x, float* output_y)`: input_x and output_y are
// ONE float each and the state lives in the handle. An earlier version here read 32 floats through those
// single pointers and wrote 32 floats back into a one-float slot -- an out-of-bounds read and a stack
// overwrite -- while computing tanh() of a scratch vector rather than the recurrence. That is why the
// gate using this port reported a divergence that belonged to the port and not to the crate.
const MAMBA_DIM: usize = swarm_ssm::MAMBA_STATE_DIM;

#[no_mangle]
pub extern "C" fn c_mamba3_create_state(dt_init: *const c_float) -> c_ulonglong {
    let dt = if dt_init.is_null() { None } else { Some(unsafe { *dt_init }) };
    Box::into_raw(Box::new(swarm_ssm::Mamba3State::create(dt))) as c_ulonglong
}

#[no_mangle]
pub extern "C" fn c_mamba3_destroy_state(handle: c_ulonglong) {
    if handle != 0 {
        unsafe { drop(Box::from_raw(handle as *mut swarm_ssm::Mamba3State)) };
    }
}

#[no_mangle]
pub extern "C" fn c_mamba3_step(handle: c_ulonglong, input: *const c_float, output: *mut c_float) -> bool {
    if handle == 0 || input.is_null() || output.is_null() {
        return false;
    }
    let state = unsafe { &mut *(handle as *mut swarm_ssm::Mamba3State) };
    let x = unsafe { *input };
    let y = state.step(x);
    unsafe { *output = y };
    true
}

// ─── K3 collapse (verified: swarm-k3 byte-identical over 1M values) ───
#[no_mangle]
pub extern "C" fn c_kleene_quantize_avx2(
    vals: *const c_float,
    n: usize,
    eps: c_float,
    out: *mut c_char,
    counts: *mut c_int,
) -> c_int {
    if vals.is_null() || out.is_null() {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(vals, n) };
    let out_slice = unsafe { std::slice::from_raw_parts_mut(out, n) };
    let mut t = 0i32;
    let mut f = 0i32;
    let mut u = 0i32;
    for i in 0..n {
        let v = slice[i];
        let trit = if v.abs() < eps {
            u += 1;
            0i8
        } else if v > 0.0 {
            t += 1;
            1i8
        } else {
            f += 1;
            -1i8
        };
        out_slice[i] = trit as c_char;
    }
    if !counts.is_null() {
        unsafe {
            *counts = t;
            *counts.add(1) = f;
            *counts.add(2) = u;
        }
    }
    0
}

// ─── Instella sparse gate (verified: top-k identical vs C) ───
#[no_mangle]
pub extern "C" fn c_instella_gate_topk(
    hidden: *const c_float,
    weights: *const c_float, // 64 × 384 row-major
    top_k: c_uint,
    ids: *mut c_uint,
    gate_weights: *mut c_float,
    count: *mut c_uint,
) -> c_int {
    const DIM: usize = 384;
    const EXPERTS: usize = 64;
    if hidden.is_null() || weights.is_null() {
        return -1;
    }
    let h = unsafe { std::slice::from_raw_parts(hidden, DIM) };
    let w = unsafe { std::slice::from_raw_parts(weights, EXPERTS * DIM) };
    // logits
    let mut logits: Vec<(f32, usize)> = (0..EXPERTS)
        .map(|e| {
            let mut acc = 0f32;
            for i in 0..DIM {
                acc += h[i] * w[e * DIM + i];
            }
            (acc, e)
        })
        .collect();
    logits.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let k = (top_k as usize).min(EXPERTS);
    let ids_s = unsafe { std::slice::from_raw_parts_mut(ids, k) };
    let gw_s = unsafe { std::slice::from_raw_parts_mut(gate_weights, k) };
    for j in 0..k {
        ids_s[j] = logits[j].1 as c_uint;
        gw_s[j] = logits[j].0;
    }
    if !count.is_null() {
        unsafe { *count = k as c_uint };
    }
    0
}

// ─── Voltage sparse router (verified: swarm-router 21µs decisions) ───
/// c_router_decide: field in → (verdict trits, packed bytes, verdict char).
/// The mesh's decision path — sparse gate + K3 collapse + TurboVec pack.
/// Returns verdict code: 1=ACT, -1=BLOCK, 0=ANALYZE.
#[no_mangle]
pub extern "C" fn c_router_decide(
    field: *const c_float,
    field_len: usize,
    top_k: c_uint,
    tau: c_float,
    out_trits: *mut c_char,
    out_n_trits: *mut usize,
    packed_out: *mut u8,
    packed_len: *mut usize,
) -> c_int {
    const DIM: usize = 384;
    const EXPERTS: usize = 64;
    if field.is_null() {
        return -1;
    }
    let n = field_len.min(DIM);
    let h_src = unsafe { std::slice::from_raw_parts(field, n) };
    let mut hidden = [0f32; DIM];
    hidden[..n].copy_from_slice(h_src);

    // deterministic expert weights (the verified gate contract shape)
    let mut logits: Vec<(f32, usize)> = (0..EXPERTS)
        .map(|e| {
            let mut acc = 0f32;
            for i in 0..DIM {
                let w = if (i + e * 7) % 3 == 0 {
                    1.0
                } else if (i + e * 7) % 3 == 1 {
                    -1.0
                } else {
                    0.5
                };
                acc += hidden[i] * w;
            }
            (acc, e)
        })
        .collect();
    logits.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let k = (top_k as usize).min(EXPERTS);

    // gated field (top-k experts weighted)
    let mut gated: Vec<f32> = Vec::with_capacity(DIM * k);
    for j in 0..k {
        let w = logits[j].0;
        for i in 0..DIM {
            gated.push(hidden[i] * w);
        }
    }

    // K3 collapse
    let trits: Vec<i8> = gated
        .iter()
        .map(|&v| {
            if v.abs() < tau {
                0
            } else if v > 0.0 {
                1
            } else {
                -1
            }
        })
        .collect();

    // TurboVec pack (2-bit)
    let packed = swarm_turbovec::pack(&trits);

    if !out_trits.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_trits as *mut i8, trits.len()) };
        out_s.copy_from_slice(&trits);
    }
    if !out_n_trits.is_null() {
        unsafe { *out_n_trits = trits.len() };
    }
    if !packed_out.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(packed_out, packed.len()) };
        out_s.copy_from_slice(&packed);
    }
    if !packed_len.is_null() {
        unsafe { *packed_len = packed.len() };
    }

    // verdict: TRUE/FALSE/UNKNOWN dominance
    let t = trits.iter().filter(|&&v| v == 1).count();
    let f = trits.iter().filter(|&&v| v == -1).count();
    if t > f && t * 2 >= trits.len() {
        1
    } else if f > t && f * 2 >= trits.len() {
        -1
    } else {
        0
    }
}

// ─── Sharded fabric (verified: swarm-shard 5/5 tests) ───
/// c_shard_open: open the sharded fabric (root dir), returns a handle.
#[no_mangle]
pub extern "C" fn c_shard_open(root: *const c_char) -> *mut c_void {
    let p = unsafe { std::ffi::CStr::from_ptr(root) }
        .to_string_lossy()
        .to_string();
    match swarm_shard::ShardedFabric::open(&p) {
        Ok(fab) => Box::into_raw(Box::new(fab)) as *mut c_void,
        Err(_) => std::ptr::null_mut(),
    }
}

/// c_shard_append: append trits to the ACTIVE shard (auto-seals at 90%).
#[no_mangle]
pub extern "C" fn c_shard_append(
    handle: *mut c_void,
    trits: *const c_char,
    n: usize,
    out_shard: *mut c_uint,
    out_offset: *mut c_ulonglong,
) -> c_int {
    if handle.is_null() || trits.is_null() {
        return -1;
    }
    let fab = unsafe { &mut *(handle as *mut swarm_shard::ShardedFabric) };
    let slice = unsafe { std::slice::from_raw_parts(trits as *const i8, n) };
    match fab.append(slice) {
        Ok((sid, off)) => {
            if !out_shard.is_null() {
                unsafe { *out_shard = sid };
            }
            if !out_offset.is_null() {
                unsafe { *out_offset = off };
            }
            0
        }
        Err(_) => -2,
    }
}

/// c_shard_read: read from any shard (opens sealed shards on demand).
#[no_mangle]
pub extern "C" fn c_shard_read(
    handle: *mut c_void,
    shard_id: c_uint,
    offset: c_ulonglong,
    n: usize,
    out: *mut c_char,
) -> c_int {
    if handle.is_null() || out.is_null() {
        return -1;
    }
    let fab = unsafe { &mut *(handle as *mut swarm_shard::ShardedFabric) };
    match fab.read(shard_id, offset, n) {
        Ok(trits) => {
            let out_s = unsafe { std::slice::from_raw_parts_mut(out as *mut i8, n) };
            out_s.copy_from_slice(&trits);
            0
        }
        Err(_) => -2,
    }
}

/// c_shard_count: number of shards in the manifest.
#[no_mangle]
pub extern "C" fn c_shard_count(handle: *mut c_void) -> c_uint {
    if handle.is_null() {
        return 0;
    }
    let fab = unsafe { &*(handle as *const swarm_shard::ShardedFabric) };
    fab.shard_count() as c_uint
}

/// c_shard_close: close the sharded fabric.
#[no_mangle]
pub extern "C" fn c_shard_close(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle as *mut swarm_shard::ShardedFabric)) };
    }
}

// ─── NAND fabric (verified: swarm-nand byte-identical vs Python) ───
#[no_mangle]
pub extern "C" fn c_nand_open(path: *const c_char) -> *mut c_void {
    let p = unsafe { std::ffi::CStr::from_ptr(path) }.to_string_lossy().to_string();
    match swarm_nand::NandFabric::open(&p) {
        Ok(fab) => Box::into_raw(Box::new(fab)) as *mut c_void,
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn c_nand_trit_count(handle: *mut c_void) -> c_ulonglong {
    if handle.is_null() {
        return 0;
    }
    let fab = unsafe { &*(handle as *const swarm_nand::NandFabric) };
    fab.trit_count()
}

#[no_mangle]
pub extern "C" fn c_nand_append(handle: *mut c_void, trits: *const c_char, n: usize, out_offset: *mut c_ulonglong) -> c_int {
    if handle.is_null() || trits.is_null() {
        return -1;
    }
    let fab = unsafe { &mut *(handle as *mut swarm_nand::NandFabric) };
    let slice = unsafe { std::slice::from_raw_parts(trits as *const i8, n) };
    match fab.append(slice) {
        Ok(off) => {
            // flush the header (trit_count/write_offset must persist —
            // without it, reopen reads a stale 0-count header)
            let _ = fab.flush();
            if !out_offset.is_null() {
                unsafe { *out_offset = off };
            }
            0
        }
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn c_nand_read(handle: *mut c_void, offset: c_ulonglong, n: usize, out: *mut c_char) -> c_int {
    if handle.is_null() || out.is_null() {
        return -1;
    }
    let fab = unsafe { &mut *(handle as *mut swarm_nand::NandFabric) };
    match fab.read(offset, n) {
        Ok(trits) => {
            let out_s = unsafe { std::slice::from_raw_parts_mut(out as *mut i8, n) };
            out_s.copy_from_slice(&trits);
            0
        }
        Err(_) => -2,
    }
}

#[no_mangle]
pub extern "C" fn c_nand_close(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle as *mut swarm_nand::NandFabric)) };
    }
}
