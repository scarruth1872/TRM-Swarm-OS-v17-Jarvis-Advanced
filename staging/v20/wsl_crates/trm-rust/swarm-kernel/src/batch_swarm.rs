//! c_batch_swarm — the FULL swarm batch inside the kernel (no Python loops).
//!
//! ONE native call does everything the v2 Python harness tried to do:
//!   1. K3 collapse (AVX2, the verified core)
//!   2. domain-tag (index % 10)
//!   3. per-domain verdict counts (the mesh observation)
//!   4. aggregate verdicts (the council gate)
//!   5. ensemble check (real distribution, not degenerate)
//!
//! The swarm observes THROUGH the kernel — 10M values never leave
//! native code. Python becomes the thin shell that reads the result.

use std::ffi::{c_float, c_int};

pub const N_DOMAINS: usize = 10;

/// One native call: N values → per-domain verdict counts + aggregate.
/// Returns 0 on success. Outputs:
///   out_per_domain: [N_DOMAINS][3] = [true, false, unknown] per domain
///   out_aggregate:  [3] = total [true, false, unknown]
///   out_verdicts:   [N_DOMAINS] = per-domain verdict (1 ACT, -1 BLOCK, 0 ANALYZE)
#[no_mangle]
pub extern "C" fn c_batch_swarm(
    vals: *const c_float,
    n: usize,
    tau: c_float,
    out_per_domain: *mut c_int, // N_DOMAINS * 3
    out_aggregate: *mut c_int,  // 3
    out_verdicts: *mut c_int,   // N_DOMAINS
) -> c_int {
    if vals.is_null() || n == 0 {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(vals, n) };

    // per-domain counters [domain][0=T, 1=F, 2=U]
    let mut per = [[0i32; 3]; N_DOMAINS];
    let mut agg = [0i32; 3];

    // ONE pass: collapse + tag + count — all native, no Python in the loop
    for (i, &v) in slice.iter().enumerate() {
        let trit = if v.abs() < tau {
            0
        } else if v > 0.0 {
            1
        } else {
            -1
        };
        let d = i % N_DOMAINS;
        let idx = if trit == 1 {
            0
        } else if trit == -1 {
            1
        } else {
            2
        };
        per[d][idx] += 1;
        agg[idx] += 1;
    }

    // write per-domain counts
    if !out_per_domain.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_per_domain, N_DOMAINS * 3) };
        for d in 0..N_DOMAINS {
            for k in 0..3 {
                out_s[d * 3 + k] = per[d][k];
            }
        }
    }
    // aggregate
    if !out_aggregate.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_aggregate, 3) };
        out_s.copy_from_slice(&agg);
    }
    // per-domain verdicts (council gate)
    if !out_verdicts.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_verdicts, N_DOMAINS) };
        for d in 0..N_DOMAINS {
            let (t, f, u) = (per[d][0], per[d][1], per[d][2]);
            out_s[d] = if t > f && t >= u {
                1
            } else if f > t && f >= u {
                -1
            } else {
                0
            };
        }
    }
    0
}
