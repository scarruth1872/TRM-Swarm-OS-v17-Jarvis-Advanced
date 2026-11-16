//! TRM Swarm OS — Jarvis VibeThinker Evaluation Metrics (no_std port).
//! 1:1 with Jarvis-Advanced-main/VibeThinker/eval/math/main_evaluation.py:
//! pass@1, pass@n, cons@n (consensus pass rate), mode-finding over
//! extracted answers, max-score selection.

extern crate alloc;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

/// find_mode: most frequent element(s) in a list (matches python Counter).
pub fn find_mode<T: PartialEq + Clone>(lst: &[T]) -> Vec<T> {
    if lst.is_empty() {
        return Vec::new();
    }
    let mut best: Vec<T> = Vec::new();
    let mut best_count = 0usize;
    let mut seen: Vec<(T, usize)> = Vec::new();
    for item in lst {
        if let Some(entry) = seen.iter_mut().find(|(v, _)| v == item) {
            entry.1 += 1;
        } else {
            seen.push((item.clone(), 1));
        }
    }
    for (v, c) in &seen {
        if *c > best_count {
            best_count = *c;
            best = vec![v.clone()];
        } else if *c == best_count {
            best.push(v.clone());
        }
    }
    best
}

/// pass@1: mean score over all responses (0/1 scores).
pub fn pass_at_1(scores_per_item: &[Vec<f64>]) -> f64 {
    let mut total = 0.0f64;
    let mut n = 0u64;
    for scores in scores_per_item {
        for s in scores {
            total += s;
            n += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        total / n as f64
    }
}

/// pass@n: fraction of items where ANY response scored 1.0 (max_score == 1).
pub fn pass_at_n(scores_per_item: &[Vec<f64>]) -> f64 {
    if scores_per_item.is_empty() {
        return 0.0;
    }
    let mut passes = 0u64;
    for scores in scores_per_item {
        if scores.iter().any(|&s| s == 1.0) {
            passes += 1;
        }
    }
    passes as f64 / scores_per_item.len() as f64
}

/// cons@n: consensus pass rate — find mode of extracted answers, score the
/// responses matching the mode, average. Matches the eval loop.
pub fn cons_at_n(scores_per_item: &[Vec<f64>], extracted: &[Vec<Option<u64>>]) -> f64 {
    if scores_per_item.is_empty() {
        return 0.0;
    }
    let mut conses = 0.0f64;
    let mut n_items = 0u64;
    for (scores, exts) in scores_per_item.iter().zip(extracted.iter()) {
        n_items += 1;
        let present: Vec<u64> = exts.iter().filter_map(|e| *e).collect();
        let mode = find_mode(&present);
        let mut cons_correct: Vec<f64> = Vec::new();
        for (i, e) in exts.iter().enumerate() {
            if let Some(v) = e {
                if mode.contains(v) && i < scores.len() {
                    cons_correct.push(scores[i]);
                }
            }
        }
        if !cons_correct.is_empty() {
            conses += cons_correct.iter().sum::<f64>() / cons_correct.len() as f64;
        }
    }
    if n_items == 0 {
        0.0
    } else {
        conses / n_items as f64
    }
}
