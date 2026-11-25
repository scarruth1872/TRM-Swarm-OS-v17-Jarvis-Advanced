//! TRM Swarm OS — Analogical Reasoning (no_std, alloc). Task 574, enacted by council 12-2 on resubmission.
//!
//! The mechanism is a **proportional analogy with a matched transformation**:
//!
//! ```text
//!   A : B  ::  C : D          A is the case nearest the query C
//!                              A→B is the known transformation most aligned with A→C
//!   therefore  y_D ≈ y_A + (y_B − y_A) · |A→C| / |A→B|
//! ```
//!
//! Retrieval alone copies a neighbour's answer. Analogy instead asks *which known change points in the same
//! direction as the query lies*, and transfers that change in proportion to distance travelled.
//!
//! # Why the scale is clamped
//!
//! `|A→C| / |A→B| > 1` means C lies beyond the transformation the case base actually observed. Extrapolating
//! past the evidence is how an analogy becomes a fabrication, so the scale is clamped to `[0, 1]` and the
//! clamp is recorded in the result ([`Analogy::extrapolated`]). The predication is then a transfer along a
//! measured direction, not an invention beyond it.
//!
//! # The check that can fail
//!
//! [`leave_one_out`] compares three predictors on the same folds: this analogy, **zero-delta** (just take the
//! neighbour's outcome — the "retrieval only" baseline), and the **mean**. An analogy that cannot beat copying
//! the neighbour is not reasoning about the transformation at all, and the numbers say so. [`random_outcome_control`]
//! shuffles the outcomes and re-runs: with the signal destroyed, the analogy must *not* beat zero-delta.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::case_base::CaseBase;
use crate::spatial_reasoning::sqrt;

/// One analogy, with every ingredient visible so a report can reconstruct it.
#[derive(Debug, Clone)]
pub struct Analogy {
    /// The case nearest the query: the source of the analogy.
    pub source_id: String,
    /// The case that supplies the transformation (A→B).
    pub transformation_id: String,
    /// Cosine similarity between A→B and A→C. Negative alignment is never used.
    pub cosine: f64,
    /// |A→C| / |A→B|, clamped to [0, 1].
    pub scale: f64,
    /// True when the raw scale exceeded 1 and was clamped — the query lies beyond the observed transformation.
    pub extrapolated: bool,
    /// The predicted outcome.
    pub prediction: f64,
    /// The source's measured outcome, kept so the delta can be checked by hand.
    pub source_outcome: f64,
    /// Why the analogy took the shape it did — an exact match, a fallback with no aligned transformation, or
    /// empty when a real transfer happened. A fallback must never be indistinguishable from a transfer.
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct AnalogyLoo {
    pub n: usize,
    pub analogy_rmse: f64,
    pub zero_delta_rmse: f64,
    pub mean_rmse: f64,
    pub r2: f64,
    /// True when the analogy failed to beat copying the neighbour — reported, never hidden.
    pub worse_than_zero_delta: bool,
    pub clamped_fraction: f64,
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn norm(a: &[f64]) -> f64 {
    sqrt(dot(a, a))
}

/// Predict a query by analogy against the case base. `exclude` removes a case (leave-one-out).
pub fn predict(base: &CaseBase, query: &[f64], exclude: Option<&str>) -> Option<Analogy> {
    if base.len() < 3 {
        return None;
    }
    let q = base.standardise_public(query);

    // 1. the analogue source: nearest case that is not the query itself
    let ordered = base.retrieve(query, base.len(), exclude);
    let source = ordered.first()?;
    let source_case = base.cases().iter().find(|c| c.id == source.id)?;
    let a = base.standardise_public(&source_case.features);

    // Exact match: the query is the source. There is no transformation to transfer, and inventing one would be
    // a fabrication; the honest answer is the source's own measured outcome.
    let d_ac: Vec<f64> = q.iter().zip(a.iter()).map(|(x, y)| x - y).collect();
    let nac = norm(&d_ac);
    if nac < 1e-12 {
        return Some(Analogy {
            source_id: source_case.id.clone(),
            transformation_id: String::new(),
            cosine: 0.0,
            scale: 0.0,
            extrapolated: false,
            prediction: source_case.outcome,
            source_outcome: source_case.outcome,
            note: "exact match: no transformation to transfer".to_string(),
        });
    }

    // 2. The observed transformation is the pair (B, A): the case base moved from B to A, and the query lies
    //    BEYOND A on the same side. Orientation matters — with A nearest to the query, a delta measured FROM A
    //    points away from the query by construction, which made the first version of this module fall back on
    //    almost every input. The aligned quantity is therefore (A - B) against (C - A).
    let mut best: Option<(f64, &crate::case_base::Case, f64)> = None; // (cosine, B, |A-B|)
    for case in base.cases() {
        if case.id == source.id {
            continue;
        }
        if let Some(ex) = exclude {
            if case.id == ex {
                continue;
            }
        }
        let b = base.standardise_public(&case.features);
        let mut d_ba = Vec::with_capacity(a.len());
        for i in 0..a.len() {
            d_ba.push(a[i] - b[i]);
        }
        let nba = norm(&d_ba);
        if nba < 1e-12 {
            continue; // identical features: no observed change to extend
        }
        let cosine = dot(&d_ba, &d_ac) / (nba * nac);
        if cosine <= 0.0 {
            continue; // this pair points away from the query, not toward it
        }
        // Prefer the most aligned transformation; among equally aligned ones prefer the NEAREST pair, because
        // a local change is a safer thing to extend than a distant one. Without this tie-break every collinear
        // pair scored cosine 1.0 and the first case in insertion order won, which produced a scale of 0.1 where
        // the local pair gives 0.5.
        let better = match best.as_ref() {
            None => true,
            Some((best_cos, _, best_nba)) => {
                if cosine > best_cos + 1e-12 {
                    true
                } else {
                    (cosine - *best_cos).abs() <= 1e-12 && nba < *best_nba
                }
            }
        };
        if better {
            best = Some((cosine, case, nba));
        }
    }

    let Some((cosine, reference_case, nba)) = best else {
        return Some(Analogy {
            source_id: source_case.id.clone(),
            transformation_id: String::new(),
            cosine: 0.0,
            scale: 0.0,
            extrapolated: false,
            prediction: source_case.outcome,
            source_outcome: source_case.outcome,
            note: "no transformation points toward the query; fell back to the neighbour's outcome".to_string(),
        });
    };

    let raw_scale = if nba > 1e-12 { nac / nba } else { 0.0 };
    let extrapolated = raw_scale > 1.0;
    let scale = if extrapolated { 1.0 } else { raw_scale };

    // the observed change was B -> A; continuing that direction past A is what the query asks for
    let delta_outcome = source_case.outcome - reference_case.outcome;
    Some(Analogy {
        source_id: source_case.id.clone(),
        transformation_id: reference_case.id.clone(),
        cosine,
        scale,
        extrapolated,
        prediction: source_case.outcome + delta_outcome * scale,
        source_outcome: source_case.outcome,
        note: if extrapolated {
            "query lies beyond the observed transformation; scale clamped to 1".to_string()
        } else {
            String::new()
        },
    })
}

/// Leave-one-out comparison: analogy vs zero-delta (copy the neighbour) vs mean.
pub fn leave_one_out(base: &CaseBase) -> Option<AnalogyLoo> {
    let n = base.len();
    if n < 4 {
        return None;
    }
    let mut sq_an = 0.0;
    let mut sq_zero = 0.0;
    let mut sq_mean = 0.0;
    let mut clamped = 0usize;
    let mut ys = Vec::new();
    let mut preds = Vec::new();

    for case in base.cases() {
        let an = predict(base, &case.features, Some(&case.id))?;
        if an.extrapolated {
            clamped += 1;
        }
        let others: Vec<f64> = base
            .cases()
            .iter()
            .filter(|o| o.id != case.id)
            .map(|o| o.outcome)
            .collect();
        let mean = others.iter().sum::<f64>() / others.len() as f64;

        let e_an = an.prediction - case.outcome;
        let e_zero = an.source_outcome - case.outcome;
        let e_mean = mean - case.outcome;
        sq_an += e_an * e_an;
        sq_zero += e_zero * e_zero;
        sq_mean += e_mean * e_mean;
        ys.push(case.outcome);
        preds.push(an.prediction);
    }

    let ybar = ys.iter().sum::<f64>() / n as f64;
    let ss_tot: f64 = ys.iter().map(|y| (y - ybar) * (y - ybar)).sum();
    let ss_res: f64 = ys.iter().zip(preds.iter()).map(|(y, p)| (y - p) * (y - p)).sum();
    let analogy_rmse = sqrt(sq_an / n as f64);
    let zero_delta_rmse = sqrt(sq_zero / n as f64);

    Some(AnalogyLoo {
        n,
        analogy_rmse,
        zero_delta_rmse,
        mean_rmse: sqrt(sq_mean / n as f64),
        r2: if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 },
        worse_than_zero_delta: analogy_rmse >= zero_delta_rmse,
        clamped_fraction: clamped as f64 / n as f64,
    })
}

/// Control: rebuild the base with the outcomes shuffled (deterministic LCG), then re-run leave-one-out.
///
/// With the signal destroyed, an analogy that still "beats" zero-delta is not transferring anything - it is
/// fitting the shuffle. The control is expected to show `worse_than_zero_delta = true`; if it does not, the
/// earlier result means nothing.
pub fn random_outcome_control(base: &CaseBase, seed: u64) -> Option<AnalogyLoo> {
    let mut shuffled: Vec<f64> = base.cases().iter().map(|c| c.outcome).collect();
    let mut state = seed | 1;
    let n = shuffled.len();
    for i in (1..n).rev() {
        // xorshift64
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = (state % (i as u64 + 1)) as usize;
        shuffled.swap(i, j);
    }
    let mut control = CaseBase::new(base.cases().first()?.features.len());
    for (case, y) in base.cases().iter().zip(shuffled.iter()) {
        control.add(&case.id, &case.features, *y).ok()?;
    }
    leave_one_out(&control)
}
