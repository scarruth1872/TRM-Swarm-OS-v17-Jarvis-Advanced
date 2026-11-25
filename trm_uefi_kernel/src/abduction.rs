//! TRM Swarm OS — Abductive Reasoning (no_std, alloc). Task 575, enacted by council 13-1 on resubmission.
//!
//! Abduction answers: *what is the smallest set of things that, if true, would explain this observation?*
//! That is the reasoning a failed prediction needs — "the compound was inactive" becomes "it was inactive
//! because of X" only if X is named, bounded, and cheaper than the alternatives.
//!
//! # The three properties that make it abduction rather than guessing
//!
//! 1. **Minimality** — the returned set is the smallest one that derives the observation. The search runs by
//!    increasing size, so the first success at size n proves no smaller set works (given the search was
//!    complete, which [`AbductionResult::complete`] reports).
//! 2. **Bounded search** — combinations are capped ([`MAX_COMBINATIONS`]); a search that hits the cap says
//!    `complete = false` instead of presenting a possibly-non-minimal answer as minimal.
//! 3. **Alternatives are counted, not hidden** — if several hypotheses explain the observation equally well,
//!    the count is published. An abduction that reports one explanation when three exist is a claim about the
//!    world that the engine cannot support.
//!
//! # What it does not do
//!
//! It does not invent hypotheses. The candidate space is supplied by the caller, so the quality of an
//! abduction is bounded by the quality of that list — a limit stated in the proposal to the council and
//! repeated here rather than buried.
//!
//! Entailment is decided by [`crate::reasoning_engine::Engine::forward_chain`], the same Kleene K3 engine used
//! for tasks 571/572, so an assumption makes the observation TRUE only if the rules actually derive it.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::reasoning_engine::{Engine, Trit};

/// Combination cap for the search. 262,144 subsets is 2^18 — enough for 18 candidates at any size, and far
/// past what a caller should be asking (a hypothesis space that large is a framing problem, not a search one).
pub const MAX_COMBINATIONS: usize = 262_144;

/// One candidate explanation: an atom the caller allows the engine to assume, with a cost (default 1.0).
/// Cost breaks ties between explanations of equal size — the cheaper set wins, and the tie is still counted.
#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub atom: String,
    pub cost: f64,
}

impl Hypothesis {
    pub fn new(atom: &str) -> Hypothesis {
        Hypothesis { atom: atom.to_string(), cost: 1.0 }
    }
    pub fn with_cost(atom: &str, cost: f64) -> Hypothesis {
        Hypothesis { atom: atom.to_string(), cost }
    }
}

#[derive(Debug, Clone)]
pub struct AbductionResult {
    /// The minimal assumption set, in the order the caller supplied the hypotheses. Empty when the
    /// observation already follows from the facts.
    pub assumptions: Vec<String>,
    /// Total cost of the returned set.
    pub cost: f64,
    /// Subsets actually evaluated.
    pub searched: usize,
    /// False when [`MAX_COMBINATIONS`] stopped the search: the answer may not be minimal.
    pub complete: bool,
    /// How many distinct sets of the same size and cost also explain the observation.
    pub alternatives: usize,
    /// True when the observation cannot be derived within the hypothesis space at all.
    pub unexplained: bool,
}

/// Source string recorded for an assumed fact, so a derivation can show which atom was *assumed* rather than
/// observed. This is the distinction that matters in a report: assumed facts are not evidence.
pub const ASSUMPTION_SOURCE: &str = "ASSUMED (abduction), not observed";

fn with_assumptions(engine: &Engine, chosen: &[usize], hypotheses: &[Hypothesis]) -> Engine {
    let mut e = engine.clone();
    for &i in chosen {
        e.assert_fact(&hypotheses[i].atom, ASSUMPTION_SOURCE);
    }
    e
}

fn derives(engine: &Engine, observation: &str) -> bool {
    let r = engine.forward_chain();
    matches!(r.truth.get(observation), Some(Trit::True))
}

/// Combination generator: index sets of exactly `k` elements from `n`, in lexicographic order.
/// Standard odometer walk: increment the rightmost index that still has room, reset the ones after it.
fn subsets(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if k == 0 || k > n {
        return out;
    }
    let mut idx: Vec<usize> = (0..k).collect();
    loop {
        out.push(idx.clone());
        let mut advanced = false;
        let mut i = k;
        while i > 0 {
            i -= 1;
            if idx[i] < n - k + i {
                idx[i] += 1;
                for j in (i + 1)..k {
                    idx[j] = idx[j - 1] + 1;
                }
                advanced = true;
                break;
            }
        }
        if !advanced {
            return out;
        }
    }
}

/// Find the minimal assumption set that makes `observation` TRUE.
///
/// The search runs by increasing size and stops at the first size that produces an explanation, so no larger
/// set can be returned while a smaller one exists. Within that size the cheapest set wins; sets of equal size
/// and equal cost are counted in `alternatives` rather than discarded.
pub fn abduce(
    engine: &Engine,
    observation: &str,
    hypotheses: &[Hypothesis],
    max_size: usize,
) -> AbductionResult {
    if derives(engine, observation) {
        return AbductionResult {
            assumptions: Vec::new(),
            cost: 0.0,
            searched: 1,
            complete: true,
            alternatives: 0,
            unexplained: false,
        };
    }

    let n = hypotheses.len();
    let cap = max_size.min(n);
    let mut searched = 0usize;
    let mut complete = true;

    for size in 1..=cap {
        let mut winners: Vec<(Vec<usize>, f64)> = Vec::new();
        for combo in subsets(n, size) {
            if searched >= MAX_COMBINATIONS {
                complete = false;
                break;
            }
            searched += 1;
            let e = with_assumptions(engine, &combo, hypotheses);
            if derives(&e, observation) {
                let cost: f64 = combo.iter().map(|&i| hypotheses[i].cost).sum();
                winners.push((combo, cost));
            }
        }
        if !winners.is_empty() {
            // cheapest set at the minimal size; ties are counted, not silently dropped
            let best_cost = winners
                .iter()
                .map(|(_, c)| *c)
                .fold(f64::INFINITY, f64::min);
            let mut alternatives = 0usize;
            let mut chosen: Option<Vec<usize>> = None;
            for (combo, cost) in &winners {
                if (cost - best_cost).abs() < 1e-12 {
                    if chosen.is_none() {
                        chosen = Some(combo.clone());
                    } else {
                        alternatives += 1;
                    }
                }
            }
            let combo = chosen.expect("at least one winner");
            return AbductionResult {
                assumptions: combo.iter().map(|&i| hypotheses[i].atom.clone()).collect(),
                cost: best_cost,
                searched,
                complete,
                alternatives,
                unexplained: false,
            };
        }
        if !complete {
            break;
        }
    }

    AbductionResult {
        assumptions: Vec::new(),
        cost: 0.0,
        searched,
        complete,
        alternatives: 0,
        unexplained: true,
    }
}

/// Render an abduction for a report: which assumptions, at what cost, and whether the search was exhaustive.
pub fn describe(observation: &str, r: &AbductionResult) -> String {
    if r.unexplained {
        return format!(
            "{} is NOT explained by the hypothesis space within {} subsets ({})",
            observation,
            r.searched,
            if r.complete { "search complete" } else { "SEARCH TRUNCATED" }
        );
    }
    if r.assumptions.is_empty() {
        return format!("{} follows from the facts; no assumption needed", observation);
    }
    format!(
        "{} is explained by assuming {{{}}} at cost {:.2} ({} of {} subsets searched, {} alternative set(s) of equal cost)",
        observation,
        r.assumptions.join(", "),
        r.cost,
        r.searched,
        if r.complete { "all" } else { "NOT all" },
        r.alternatives
    )
}
