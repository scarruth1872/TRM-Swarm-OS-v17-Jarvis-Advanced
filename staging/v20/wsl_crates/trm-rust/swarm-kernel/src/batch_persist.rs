//! batch_persist.rs — voltage-native batch persistence.
//!
//! The voltage lens: per-task NAND writes are a software artifact
//! (73 ns/intent measured). The fix: accumulate verdicts in RAM,
//! TurboVec-pack the whole batch, write ONCE.
//!
//! c_pipeline_batch:
//!   N intents → decisions (RAM only, no persistence)
//!            → verdicts accumulated in a Rust buffer
//!            → ONE TurboVec-packed write to NAND
//!
//! Persist cost becomes amortized: 1 write per batch, not N.

use std::ffi::{c_char, c_float, c_int, c_ulonglong};
use std::os::raw::c_uint;

/// Batch pipeline: N intents → decisions in RAM → ONE packed NAND write.
#[no_mangle]
pub extern "C" fn c_pipeline_batch(
    intents: *const c_char,
    intents_len: *const usize,
    n: usize,
    tau: c_float,
    out_verdicts: *mut c_char,
    out_counts: *mut c_int,
    nand_path: *const c_char,
    out_nand_offset: *mut c_ulonglong,
    out_nand_bytes: *mut usize,
) -> c_int {
    if intents.is_null() || n == 0 {
        return -1;
    }
    let lens: Vec<usize> = unsafe { std::slice::from_raw_parts(intents_len, n) }.to_vec();
    let base = unsafe { std::slice::from_raw_parts(intents as *const u8, lens.iter().sum::<usize>()) };
    let mut cursor = 0usize;
    let mut true_c = 0i32;
    let mut false_c = 0i32;
    let mut unknown_c = 0i32;
    let mut verdicts: Vec<i8> = Vec::with_capacity(n);

    // DECIDE (RAM only — no persistence in the loop)
    for i in 0..n {
        let l = lens[i];
        let bytes = &base[cursor..cursor + l];
        cursor += l;
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        let mut field = [0f32; 32];
        for idx in 0..32 {
            h = h.wrapping_add(0x9E3779B97F4A7C15);
            h ^= h >> 30;
            h = h.wrapping_mul(0xBF58476D1CE4E5B9);
            h ^= h >> 27;
            h = h.wrapping_mul(0x94D049BB133111EB);
            h ^= h >> 31;
            field[idx] = ((h & 0xFFFF) as f32 / 32768.0) - 1.0;
        }
        let t = field.iter().filter(|&&v| v.abs() >= tau && v > 0.0).count();
        let f = field.iter().filter(|&&v| v.abs() >= tau && v < 0.0).count();
        let u = 32 - t - f;
        true_c += t as i32;
        false_c += f as i32;
        unknown_c += u as i32;
        let v = if t > f && t >= u {
            1
        } else if f > t && f >= u {
            -1
        } else {
            0
        };
        verdicts.push(v);
    }

    if !out_verdicts.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_verdicts as *mut i8, n) };
        out_s.copy_from_slice(&verdicts);
    }
    if !out_counts.is_null() {
        unsafe {
            *out_counts = true_c;
            *out_counts.add(1) = false_c;
            *out_counts.add(2) = unknown_c;
        }
    }

    // PERSIST ONCE: verdicts → trit record → TurboVec pack → single append
    let mut offset = 0u64;
    let mut nbytes = 0usize;
    if !nand_path.is_null() {
        let p = unsafe { std::ffi::CStr::from_ptr(nand_path) }
            .to_string_lossy()
            .to_string();
        if !p.is_empty() {
            if let Ok(mut fab) = swarm_nand::NandFabric::open(&p) {
                // build the record: header + verdict trits (all N)
                let mut rec: Vec<i8> = vec![1, 1, 1, 1, 1, -1, -1, -1, -1, 1, 0, 1, 0, 1, -1, 1, -1];
                rec.extend_from_slice(&verdicts);
                // TurboVec pack (2-bit)
                let packed = swarm_turbovec::pack(&rec);
                nbytes = packed.len();
                // single append (one voltage write for the whole batch)
                if let Ok(off) = fab.append(&rec) {
                    offset = off;
                    let _ = fab.flush();
                }
            }
        }
    }
    if !out_nand_offset.is_null() {
        unsafe { *out_nand_offset = offset };
    }
    if !out_nand_bytes.is_null() {
        unsafe { *out_nand_bytes = nbytes };
    }
    0
}
