//! Resonance engine: sentiment, emotional resonance index, and self-awareness overlap.
//!
//! Ported from `swarm_v2/core/resonance_engine.py`. The module's own `strip_resonance_engine.dll` is
//! never called (0 call sites) and its symbol does not exist; this is the real implementation.
//!
//! Rounding is `round(x, 3)` from Python, which is half-to-even. `format!("{:.3}")` rounds the same
//! way, so the value is formatted and parsed back rather than rounded with `f64::round` (which is
//! half-away-from-zero and would give 0.063 where Python gives 0.062).

use std::collections::HashSet;
use std::os::raw::{c_double, c_int, c_uint};

const POSITIVE: [&str; 12] = [
    "harmony", "trust", "success", "optimize", "amplify", "collaborate",
    "grow", "evolve", "love", "resonance", "friendship", "strength",
];
const NEGATIVE: [&str; 10] = [
    "error", "fail", "conflict", "degrade", "stagnant",
    "friction", "broken", "loss", "disconnect", "crash",
];

/// Python's `set(re.findall(r'[a-z]+', text.lower()))`: ASCII letter runs, multiplicity ignored.
fn token_set(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut cur = String::new();
    for ch in text.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_lowercase() {
            cur.push(c);
        } else if !cur.is_empty() {
            out.insert(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.insert(cur);
    }
    out
}

fn round3(x: f64) -> f64 {
    format!("{:.3}", x).parse().unwrap_or(x)
}

/// (positive - negative) / (positive + negative) over distinct words; 0.0 when neither appears.
pub fn score_sentiment(text: &str) -> f64 {
    let words = token_set(text);
    let pos = words.iter().filter(|w| POSITIVE.contains(&w.as_str())).count();
    let neg = words.iter().filter(|w| NEGATIVE.contains(&w.as_str())).count();
    if pos + neg == 0 {
        return 0.0;
    }
    round3((pos as f64 - neg as f64) / (pos + neg) as f64)
}

/// Mean sentiment, sample size and trend. Trend compares the two halves with a strict ±0.05 dead band.
pub fn emotional_resonance_index(s: &[f64]) -> (f64, usize, &'static str) {
    if s.is_empty() {
        return (0.0, 0, "neutral");
    }
    let index = round3(s.iter().sum::<f64>() / s.len() as f64);
    let mid = s.len() / 2;
    let trend = if mid == 0 {
        "neutral"
    } else {
        let older = s[..mid].iter().sum::<f64>() / mid as f64;
        let newer = s[mid..].iter().sum::<f64>() / (s.len() - mid) as f64;
        if newer > older + 0.05 {
            "rising"
        } else if newer < older - 0.05 {
            "falling"
        } else {
            "stable"
        }
    };
    (index, s.len(), trend)
}

/// Diversity (unique focus nodes / 5, capped at 1), mean pairwise Jaccard of the first 50 chars of
/// each vibe, whether that mean exceeds 0.6, and the recommended action.
pub fn self_awareness(dreams: &[(Vec<String>, String)]) -> (f64, f64, bool, &'static str) {
    let n = dreams.len();
    if n < 2 {
        return (1.0, 0.0, false, "none");
    }

    let mut nodes: HashSet<&str> = HashSet::new();
    for (focus, _) in dreams {
        for f in focus {
            nodes.insert(f.as_str());
        }
    }
    let diversity = (nodes.len() as f64 / 5.0).min(1.0);

    let vibes: Vec<HashSet<String>> = dreams
        .iter()
        .map(|(_, v)| token_set(&v.chars().take(50).collect::<String>()))
        .collect();

    let mut overlaps = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let union = vibes[i].union(&vibes[j]).count();
            if union > 0 {
                let inter = vibes[i].intersection(&vibes[j]).count();
                overlaps.push(inter as f64 / union as f64);
            }
        }
    }
    let avg = if overlaps.is_empty() {
        0.0
    } else {
        overlaps.iter().sum::<f64>() / overlaps.len() as f64
    };

    let stagnation = avg > 0.6;
    let action = if stagnation {
        "diversity_injection_recommended"
    } else if diversity < 0.4 {
        "broaden_agent_participation"
    } else {
        "none"
    };

    (round3(diversity), round3(avg), stagnation, action)
}

#[no_mangle]
pub extern "C" fn rust_resonance_sentiment(ptr: *const u8, len: usize) -> c_double {
    if ptr.is_null() {
        return 0.0;
    }
    let s = unsafe { std::slice::from_raw_parts(ptr, len) };
    score_sentiment(&String::from_utf8_lossy(s)) as c_double
}

#[no_mangle]
pub extern "C" fn rust_resonance_eri(
    ptr: *const f64,
    n: usize,
    out_index: *mut f64,
    out_size: *mut usize,
    trend_out: *mut u8,
    trend_cap: usize,
) -> c_uint {
    if ptr.is_null() || out_index.is_null() || out_size.is_null() {
        return 0;
    }
    let s = unsafe { std::slice::from_raw_parts(ptr, n) };
    let (index, size, trend) = emotional_resonance_index(s);
    unsafe {
        *out_index = index;
        *out_size = size;
        if !trend_out.is_null() && trend_cap > 0 {
            let b = trend.as_bytes();
            let take = b.len().min(trend_cap);
            std::ptr::copy_nonoverlapping(b.as_ptr(), trend_out, take);
        }
    }
    trend.len() as c_uint
}

/// `nodes_csv[i]` is the focus-node list for dream i joined with ","; `vibes[i]` is its vibe.
#[no_mangle]
pub extern "C" fn rust_resonance_rsa(
    n: usize,
    nodes_csv: *const *const u8,
    nodes_lens: *const usize,
    vibes: *const *const u8,
    vibe_lens: *const usize,
    out_diversity: *mut f64,
    out_avg: *mut f64,
    out_stagnation: *mut c_int,
    action_out: *mut u8,
    action_cap: usize,
) -> c_uint {
    if n == 0 {
        // matches the Python early return for fewer than two dreams
        unsafe {
            if !out_diversity.is_null() { *out_diversity = 1.0; }
            if !out_avg.is_null() { *out_avg = 0.0; }
            if !out_stagnation.is_null() { *out_stagnation = 0; }
            if !action_out.is_null() && action_cap > 0 {
                let b = b"none";
                let take = b.len().min(action_cap);
                std::ptr::copy_nonoverlapping(b.as_ptr(), action_out, take);
            }
        }
        return 4;
    }
    let nodes_p = unsafe { std::slice::from_raw_parts(nodes_csv, n) };
    let nodes_l = unsafe { std::slice::from_raw_parts(nodes_lens, n) };
    let vibe_p = unsafe { std::slice::from_raw_parts(vibes, n) };
    let vibe_l = unsafe { std::slice::from_raw_parts(vibe_lens, n) };

    let mut dreams = Vec::with_capacity(n);
    for i in 0..n {
        let nb = unsafe { std::slice::from_raw_parts(nodes_p[i], nodes_l[i]) };
        let ns = String::from_utf8_lossy(nb).to_string();
        let focus: Vec<String> = if ns.is_empty() {
            Vec::new()
        } else {
            ns.split(',').map(|x| x.to_string()).collect()
        };
        let vb = unsafe { std::slice::from_raw_parts(vibe_p[i], vibe_l[i]) };
        dreams.push((focus, String::from_utf8_lossy(vb).to_string()));
    }

    let (diversity, avg, stagnation, action) = self_awareness(&dreams);
    unsafe {
        if !out_diversity.is_null() { *out_diversity = diversity; }
        if !out_avg.is_null() { *out_avg = avg; }
        if !out_stagnation.is_null() { *out_stagnation = stagnation as c_int; }
        if !action_out.is_null() && action_cap > 0 {
            let b = action.as_bytes();
            let take = b.len().min(action_cap);
            std::ptr::copy_nonoverlapping(b.as_ptr(), action_out, take);
        }
    }
    action.len() as c_uint
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentiment_matches_the_verified_vectors() {
        assert_eq!(score_sentiment("harmony"), 1.0);
        assert_eq!(score_sentiment("error"), -1.0);
        assert_eq!(score_sentiment("nothing here"), 0.0);
    }

    #[test]
    fn repeats_are_ignored() {
        assert_eq!(score_sentiment("harmony harmony harmony"), 1.0);
    }

    #[test]
    fn eri_matches_the_verified_vectors() {
        assert_eq!(emotional_resonance_index(&[0.0, 0.5]), (0.25, 2, "rising"));
        assert_eq!(emotional_resonance_index(&[1.0, -1.0, 0.0]), (0.0, 3, "falling"));
    }

    #[test]
    fn rounds_half_to_even() {
        assert_eq!(round3(0.0625), 0.062);
    }
}
