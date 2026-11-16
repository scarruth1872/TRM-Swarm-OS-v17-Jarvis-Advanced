//! TRM Swarm OS — Predictive Watchdog (no_std port).
//! 1:1 core semantics from swarm_v2/core/predictive_watchdog.py — Catalyst.
//! Predicts restart risk from telemetry: the strip's four-predictor model (memory pressure, trend
//! acceleration, disk headroom, D_H chaos), argmax by probability.

extern crate alloc;

/// The strip's thresholds and regression weights.
// ── The strip's model (native_voltage_strips/strip_predictive_watchdog.rs) ──────────────────────
//
// This module carried a failure-STREAK model (`consecutive_failures >= 2`) that appears nowhere in
// the oracle: `predictive_watchdog.py` predicts from TELEMETRY, and its strip computes a risk
// probability and a time estimate from four independent predictors, keeping the highest probability.
// The streak constants are retired, not kept alongside.
//
// The four predictors, exactly as the strip has them:
//   1. memory pressure: recent > 85% -> p = min((recent-80)/20, 1), t = max((100-recent)*2, 5)
//   2. trend: slope over the 10-sample window with the precomputed weights (below); slope > 1.0 and
//      a time-to-critical below 60s -> p = 0.7, t = max(ttc, 0)
//   3. disk: free < 5GB -> p = max(1 - free/5, 0.5), t = min(free*60, 300)
//   4. D_H chaos: any of the five values > 1.8 -> p = 0.85, t = 10
// then the argmax (strict >, so ties keep the earlier predictor). Nothing fires -> (0.0, 999.0).
//
// The strip takes a temperature window and never reads it, so neither does this port; the python
// passes one because its fallback path uses it.

use alloc::vec::Vec;

pub const MEM_CRITICAL_PCT: f32 = 85.0;
pub const TREND_SLOPE_THRESHOLD: f32 = 1.0;
pub const DISK_FREE_THRESHOLD_GB: f32 = 5.0;
pub const DH_CHAOS_THRESHOLD: f32 = 1.8;
pub const TREND_TIME_TO_CRITICAL_S: f32 = 60.0;
pub const NO_RISK_TIME_S: f32 = 999.0;

/// The strip's precomputed regression weights `(x - 4.5)/82.5` for N=10 (mean 4.5, SS 82.5), so the
/// slope is a weighted sum rather than a fit.
pub const X_WEIGHTS: [f32; 10] = [
    -4.5 / 82.5, -3.5 / 82.5, -2.5 / 82.5, -1.5 / 82.5, -0.5 / 82.5,
    0.5 / 82.5, 1.5 / 82.5, 2.5 / 82.5, 3.5 / 82.5, 4.5 / 82.5,
];

/// The oracle names this `native_assess_telemetry_risk`; the name is kept so the parity audit can
/// pair it. One assessment: (probability, time estimate in seconds).
pub fn native_assess_telemetry_risk(mem: &[f32; 10], dh: &[f32; 5], free_gb: f32) -> (f32, f32) {
    let recent_mem = mem[9];

    let mut p_mem = 0.0f32;
    let mut t_mem = NO_RISK_TIME_S;
    if recent_mem > MEM_CRITICAL_PCT {
        p_mem = ((recent_mem - 80.0) / 20.0).min(1.0);
        t_mem = ((100.0 - recent_mem) * 2.0).max(5.0);
    }

    let mut slope = 0.0f32;
    for k in 0..10 {
        slope += mem[k] * X_WEIGHTS[k];
    }
    let mut p_trend = 0.0f32;
    let mut t_trend = NO_RISK_TIME_S;
    if slope > TREND_SLOPE_THRESHOLD {
        let denom = if slope < 0.01 { 0.01 } else { slope };
        let time_to_crit = (MEM_CRITICAL_PCT - recent_mem) / denom * 5.0;
        if time_to_crit < TREND_TIME_TO_CRITICAL_S {
            p_trend = 0.7;
            t_trend = if time_to_crit < 0.0 { 0.0 } else { time_to_crit };
        }
    }

    let mut p_disk = 0.0f32;
    let mut t_disk = NO_RISK_TIME_S;
    if free_gb < DISK_FREE_THRESHOLD_GB {
        p_disk = (1.0 - free_gb / DISK_FREE_THRESHOLD_GB).max(0.5);
        t_disk = (free_gb * 60.0).min(300.0);
    }

    let mut p_dh = 0.0f32;
    let mut t_dh = NO_RISK_TIME_S;
    for d in dh.iter() {
        if *d > DH_CHAOS_THRESHOLD {
            p_dh = 0.85;
            t_dh = 10.0;
            break;
        }
    }

    let mut best_p = p_mem;
    let mut best_t = t_mem;
    if p_trend > best_p {
        best_p = p_trend;
        best_t = t_trend;
    }
    if p_disk > best_p {
        best_p = p_disk;
        best_t = t_disk;
    }
    if p_dh > best_p {
        best_p = p_dh;
        best_t = t_dh;
    }
    (best_p, best_t)
}

/// Batch entry point, mirroring `strip_batch_assess_risk`: one mem window (10 each), dh window (5
/// each) and free-disk figure per row.
pub fn assess_risk_batch(mems: &[f32], dhs: &[f32], disks: &[f32]) -> Vec<(f32, f32)> {
    let count = disks.len();
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let mb = i * 10;
        let db = i * 5;
        if mb + 10 > mems.len() || db + 5 > dhs.len() {
            break;
        }
        let mut m = [0f32; 10];
        m.copy_from_slice(&mems[mb..mb + 10]);
        let mut d = [0f32; 5];
        d.copy_from_slice(&dhs[db..db + 5]);
        out.push(native_assess_telemetry_risk(&m, &d, disks[i]));
    }
    out
}

// ── the python's consumption rule (swarm_v2/core/predictive_watchdog.py) ─────────────────────────

/// The python only acts on a strip result above this probability, and otherwise falls back to its
/// own per-predictor path.
pub const PREDICT_THRESHOLD: f32 = 0.5;

/// `_assess_risk`: the python's decision on top of a strip assessment. It returns a prediction only
/// above `PREDICT_THRESHOLD`, with the cause following the strip's exact probability; below it the
/// result is a warning, not a prediction. Returns (cause, probability, time estimate).
pub fn assess_risk(probability: f32, time_estimate_s: f32) -> Option<(&'static str, f32, f32)> {
    if probability > PREDICT_THRESHOLD {
        Some((cause_for(probability), probability, time_estimate_s))
    } else {
        None
    }
}

/// The python's cause mapping, keyed on the strip's exact probability values.
pub fn cause_for(p: f32) -> &'static str {
    if p == 0.85 {
        "Fractal chaos D_H threshold exceeded"
    } else if p == 0.7 {
        "Memory trend acceleration detected"
    } else if p > 0.0 {
        "Memory pressure/Disk boundary warning"
    } else {
        "no risk factor fired"
    }
}

/// Predictive watchdog state: the most recent telemetry assessment, not a failure streak.
#[derive(Debug, Clone)]
pub struct Watchdog {
    pub last_probability: f32,
    pub last_time_s: f32,
}

impl Watchdog {
    pub fn new() -> Self {
        Watchdog {
            last_probability: 0.0,
            last_time_s: NO_RISK_TIME_S,
        }
    }

    /// Assess one telemetry window and report whether it crosses the python's action threshold.
    pub fn assess(&mut self, mem: &[f32; 10], dh: &[f32; 5], free_gb: f32) -> bool {
        let (p, t) = native_assess_telemetry_risk(mem, dh, free_gb);
        self.last_probability = p;
        self.last_time_s = t;
        p > PREDICT_THRESHOLD
    }

    /// Predicted right now? The python acts only above `PREDICT_THRESHOLD`, so a fired predictor
    /// below it is a warning rather than a prediction.
    pub fn is_restart_predicted(&self) -> bool {
        self.last_probability > PREDICT_THRESHOLD
    }

    /// Status: (probability, time estimate, predicted).
    pub fn get_status(&self) -> (f32, f32, bool) {
        (self.last_probability, self.last_time_s, self.is_restart_predicted())
    }
}

/// Singleton handle.
pub fn get_watchdog() -> u64 {
    0x5741_5443_4844_4F47
}
