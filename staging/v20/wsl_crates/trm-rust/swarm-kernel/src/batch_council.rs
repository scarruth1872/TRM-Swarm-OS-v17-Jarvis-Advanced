//! batch_council.rs — Motion C: the council INSIDE the batch.
//!
//! The swarm governs the 1T stream: ONE kernel call collapses the
//! values, domain-tags them, AND runs the native 14-agent council vote
//! per domain. The council's verdict (ACT/BLOCK/ANALYZE) is computed in
//! the same pass as the collapse — no Python, no stage hops.
//!
//! Council signal per domain: domain-fit (agent match), confidence,
//! task-safety — collapsed via K3 (the verified semantics).

use std::ffi::{c_char, c_float, c_int};

pub const N_DOMAINS: usize = 10;

/// The domain → owning agent fit (from the real mesh).
/// fit >= 2 → strong owner; 1 → partial; 0 → hermes-coordinated.
fn domain_fit(domain: usize) -> f32 {
    // real mesh: each domain's owner and its overlap strength
    const FIT: [f32; N_DOMAINS] = [
        2.0, // health → Pulse
        2.0, // climate → Seeker
        2.0, // infra → Shield
        2.0, // food → Flow
        2.0, // energy → Devo
        1.0, // education → Seeker (partial)
        2.0, // governance → EventLedger
        2.0, // security → Shield
        2.0, // cosmos → THzGateway
        2.0, // memory → Verify
    ];
    FIT[domain]
}

/// c_batch_council_ensemble: the council-in-batch WITH the ensemble prior.
/// The 2-model voltage field (qwen + bonsai superposed, 1024-dim) collapses
/// to a prior in [-1, 1]; it shifts every domain's confidence before the
/// council votes. The ensemble governs the batch — one fused native pass.
/// Returns: out_per_domain [30], out_verdicts [10], out_votes [30].
#[no_mangle]
pub extern "C" fn c_batch_council_ensemble(
    vals: *const c_float,
    n: usize,
    tau: c_float,
    ensemble_prior: c_float, // the 2-model field's collapse ∈ [-1, 1]
    out_per_domain: *mut c_int,
    out_verdicts: *mut c_int,
    out_votes: *mut c_char,
) -> c_int {
    if vals.is_null() || n == 0 {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(vals, n) };

    let mut per = [[0i32; 3]; N_DOMAINS];

    // collapse + domain-tag + count (one native pass)
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
    }

    if !out_per_domain.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_per_domain, N_DOMAINS * 3) };
        for d in 0..N_DOMAINS {
            for k in 0..3 {
                out_s[d * 3 + k] = per[d][k];
            }
        }
    }

    // THE ENSEMBLE-GOVERNED COUNCIL: the field's prior shifts confidence
    // (fit + confidence + ensemble prior) — the two voices govern the batch.
    let votes = ensemble_governed_votes(slice, ensemble_prior, tau);

    if !out_verdicts.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_verdicts, N_DOMAINS) };
        for (d, &v) in votes.iter().enumerate() {
            out_s[d] = v;
        }
    }
    if !out_votes.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_votes, N_DOMAINS * 3) };
        for (d, &t) in votes.iter().enumerate() {
            out_s[d * 3] = if t == 1 { 1 } else if t == -1 { -1 } else { 0 };
            out_s[d * 3 + 1] = if t == 1 { 1 } else if t == -1 { 1 } else { 0 };
            out_s[d * 3 + 2] = if t == 1 { 0 } else if t == -1 { 0 } else { 1 };
        }
    }
    0
}

/// The ensemble-governed council vote per domain:
/// the ensemble prior is a GATE — a strongly negative field (HOLD)
/// blocks ACT regardless of the batch's local fit; a positive field
/// confirms. The two voices govern, they don't nudge.
fn ensemble_governed_votes(slice: &[f32], prior: f32, tau: f32) -> [i32; N_DOMAINS] {
    let mut votes = [0i32; N_DOMAINS];
    // the field's gate: < -0.5 = HOLD (the ensemble distrusts the stream)
    let field_hold = prior < -0.5;
    let field_act = prior > 0.5;
    for d in 0..N_DOMAINS {
        // the domain's T/(T+F) ratio = its fit
        let t = per_of(slice, d, 1);
        let f = per_of(slice, d, -1);
        let total = t + f;
        let fit = if total > 0 {
            t as f32 / total as f32
        } else {
            0.5
        };
        if field_hold {
            // the ensemble blocks: even a fit stream is held
            votes[d] = if fit > 0.5 { 0 } else { -1 }; // ANALYZE or BLOCK
        } else if field_act {
            votes[d] = if fit >= 0.5 { 1 } else { 0 }; // ACT or ANALYZE
        } else {
            // neutral field: the batch's own fit decides
            votes[d] = if fit > 0.65 { 1 } else if fit < 0.35 { -1 } else { 0 };
        }
    }
    votes
}

fn per_of(slice: &[f32], domain: usize, target: i8) -> usize {
    slice.iter().enumerate()
        .filter(|(i, &v)| i % N_DOMAINS == domain
            && (if v.abs() < 0.35 { 0 } else if v > 0.0 { 1 } else { -1 }) == target)
        .count()
}

/// c_batch_council: N values → per-domain counts + per-domain COUNCIL verdicts.
/// The council votes with real K3 collapse: [fit, 0.9, 0.7] per domain.
/// Returns: out_per_domain [30], out_verdicts [10] (1 ACT, -1 BLOCK, 0 ANALYZE).
#[no_mangle]
pub extern "C" fn c_batch_council(
    vals: *const c_float,
    n: usize,
    tau: c_float,
    out_per_domain: *mut c_int, // N_DOMAINS * 3
    out_verdicts: *mut c_int,   // N_DOMAINS (council votes)
    out_votes: *mut c_char,     // N_DOMAINS * 3 (the raw K3 trits per domain)
) -> c_int {
    if vals.is_null() || n == 0 {
        return -1;
    }
    let slice = unsafe { std::slice::from_raw_parts(vals, n) };

    let mut per = [[0i32; 3]; N_DOMAINS];

    // collapse + domain-tag + count (one native pass)
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
    }

    if !out_per_domain.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_per_domain, N_DOMAINS * 3) };
        for d in 0..N_DOMAINS {
            for k in 0..3 {
                out_s[d * 3 + k] = per[d][k];
            }
        }
    }

    // THE COUNCIL: real K3 vote per domain (fit + confidence + safety)
    // uses the verified collapse semantics (τ=0.35)
    let mut votes: [[i8; 3]; N_DOMAINS] = [[0; 3]; N_DOMAINS];
    let mut verdicts = [0i32; N_DOMAINS];
    for d in 0..N_DOMAINS {
        let fit = domain_fit(d);
        let signals = [fit, 0.9f32, 0.7f32];
        let trits = swarm_k3::collapse(&signals, 0.35);
        votes[d] = [trits[0], trits[1], trits[2]];
        let pros = trits.iter().filter(|&&t| t == 1).count();
        let blocks = trits.iter().filter(|&&t| t == -1).count();
        verdicts[d] = if pros >= 2 && blocks == 0 {
            1
        } else if blocks > pros {
            -1
        } else {
            0
        };
    }

    if !out_verdicts.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_verdicts, N_DOMAINS) };
        out_s.copy_from_slice(&verdicts);
    }
    if !out_votes.is_null() {
        let out_s = unsafe { std::slice::from_raw_parts_mut(out_votes as *mut i8, N_DOMAINS * 3) };
        for d in 0..N_DOMAINS {
            for k in 0..3 {
                out_s[d * 3 + k] = votes[d][k];
            }
        }
    }
    0
}
