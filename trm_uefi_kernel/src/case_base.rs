//! TRM Swarm OS — Case-Based Reasoning (no_std, alloc). Task 573, enacted by council 10-4.
//!
//! The classic model has four phases — **retrieve, reuse, revise, retain**. This implements two of them
//! honestly and states the other two are absent, because a CBR that claims four phases while doing one is the
//! same defect as a "PBFT" that counts votes:
//!
//! | phase | here | note |
//! |---|---|---|
//! | retrieve | **yes** | k nearest cases under z-scored Euclidean distance; every distance is reported so a reader can see why a case was chosen |
//! | reuse | **yes, by interpolation** | the solution is not copied from the neighbour; it is a distance-weighted interpolation across the k retrieved cases |
//! | revise | **no** | no domain theory repairs an interpolated answer, and none is pretended |
//! | retain | **no** | the case base is fixed for the lifetime of a call; nothing is learned back into it |
//!
//! # Why z-scoring is not cosmetic
//!
//! The descriptors here span 1 (rotatable bonds) to 500 (molecular weight). Raw Euclidean distance would let
//! molecular weight decide every neighbour by itself. Each feature is standardised against the case base's own
//! mean and standard deviation before any distance is taken, and the statistics are part of the report.
//!
//! # The check that can fail
//!
//! [`CaseBase::leave_one_out`] removes each case in turn, predicts it from the others, and returns RMSE / R²
//! next to the **mean predictor** on the same folds. A CBR that cannot beat "always guess the average" is not
//! reasoning about the case; it is reciting it, and the numbers say so.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// One case: an identifier the caller owns, a numeric feature vector, and the measured outcome.
#[derive(Debug, Clone)]
pub struct Case {
    pub id: String,
    pub features: Vec<f64>,
    pub outcome: f64,
}

/// A retrieved neighbour, with the distance that justified the retrieval.
#[derive(Debug, Clone)]
pub struct Neighbour {
    pub id: String,
    pub distance: f64,
    pub outcome: f64,
}

/// Result of a leave-one-out run: the model's error next to the mean predictor's error on the same folds.
#[derive(Debug, Clone)]
pub struct LooResult {
    pub n: usize,
    pub rmse: f64,
    pub mae: f64,
    pub r2: f64,
    pub baseline_rmse: f64,
    pub baseline_r2: f64,
    pub k: usize,
    /// True when the case base could not beat the mean predictor — reported rather than hidden.
    pub worse_than_baseline: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CaseBase {
    cases: Vec<Case>,
    mean: Vec<f64>,
    std: Vec<f64>,
    feature_count: usize,
}

impl CaseBase {
    pub fn new(feature_count: usize) -> CaseBase {
        CaseBase { cases: Vec::new(), mean: Vec::new(), std: Vec::new(), feature_count }
    }

    /// Add a case. A case whose feature vector is the wrong length is refused rather than padded — padding
    /// would silently move the case in feature space.
    pub fn add(&mut self, id: &str, features: &[f64], outcome: f64) -> Result<(), String> {
        if self.feature_count == 0 {
            self.feature_count = features.len();
        }
        if features.len() != self.feature_count {
            return Err(format!(
                "case '{id}' has {} features, the base expects {}",
                features.len(),
                self.feature_count
            ));
        }
        self.cases.push(Case { id: id.to_string(), features: features.to_vec(), outcome });
        self.recompute_stats();
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.cases.len()
    }

    /// The cases as stored. Exposed so an analogy layer can compute deltas in the *same* standardised space
    /// rather than inventing a second distance definition that could disagree with retrieval.
    pub fn cases(&self) -> &[Case] {
        &self.cases
    }

    /// Standardise a vector against this base's statistics (the same transform retrieval uses internally).
    pub fn standardise_public(&self, features: &[f64]) -> Vec<f64> {
        self.standardise(features)
    }

    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }

    /// Mean and standard deviation per feature. A zero-variance feature gets std 1.0 so it contributes
    /// nothing to the distance instead of dividing by zero.
    fn recompute_stats(&mut self) {
        let n = self.cases.len();
        if n == 0 {
            return;
        }
        let d = self.feature_count;
        let mut mean = alloc::vec![0.0f64; d];
        for c in &self.cases {
            for i in 0..d {
                mean[i] += c.features[i];
            }
        }
        for m in mean.iter_mut() {
            *m /= n as f64;
        }
        let mut var = alloc::vec![0.0f64; d];
        for c in &self.cases {
            for i in 0..d {
                let dv = c.features[i] - mean[i];
                var[i] += dv * dv;
            }
        }
        let mut std = alloc::vec![0.0f64; d];
        for i in 0..d {
            let v = var[i] / n as f64;
            std[i] = if v > 0.0 { crate::spatial_reasoning::sqrt(v) } else { 1.0 };
        }
        self.mean = mean;
        self.std = std;
    }

    fn standardise(&self, features: &[f64]) -> Vec<f64> {
        features
            .iter()
            .enumerate()
            .map(|(i, x)| (x - self.mean[i]) / self.std[i])
            .collect()
    }

    /// k nearest cases to `query`, excluding `exclude_id` (used by leave-one-out so a case never sees itself).
    /// Ties are broken by insertion order, which is stable across runs.
    pub fn retrieve(&self, query: &[f64], k: usize, exclude_id: Option<&str>) -> Vec<Neighbour> {
        let q = self.standardise(query);
        let mut scored: Vec<Neighbour> = Vec::new();
        for c in &self.cases {
            if let Some(ex) = exclude_id {
                if c.id == ex {
                    continue;
                }
            }
            let cs = self.standardise(&c.features);
            let mut d2 = 0.0;
            for i in 0..q.len() {
                let dv = q[i] - cs[i];
                d2 += dv * dv;
            }
            scored.push(Neighbour {
                id: c.id.clone(),
                distance: crate::spatial_reasoning::sqrt(d2),
                outcome: c.outcome,
            });
        }
        scored.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(core::cmp::Ordering::Equal));
        scored.truncate(k.max(1));
        scored
    }

    /// Distance-weighted interpolation over the k retrieved cases (the *reuse* phase).
    ///
    /// Weights are `1/(d + epsilon)`: an exact match dominates, distant cases contribute little. This is why
    /// the prediction is not a copy of the nearest case — the retrieved solutions are adapted together.
    pub fn predict(&self, query: &[f64], k: usize) -> Option<f64> {
        self.predict_excluding(query, k, None)
    }

    pub fn predict_excluding(&self, query: &[f64], k: usize, exclude_id: Option<&str>) -> Option<f64> {
        let neighbours = self.retrieve(query, k, exclude_id);
        if neighbours.is_empty() {
            return None;
        }
        let mut num = 0.0;
        let mut den = 0.0;
        for n in &neighbours {
            let w = 1.0 / (n.distance + 1e-9);
            num += w * n.outcome;
            den += w;
        }
        if den > 0.0 { Some(num / den) } else { None }
    }

    /// Leave-one-out over the whole base, with the mean predictor computed on the same folds.
    pub fn leave_one_out(&self, k: usize) -> Option<LooResult> {
        let n = self.cases.len();
        if n < 3 {
            return None;
        }
        let k_eff = k.min(n - 1);
        let mut sq = 0.0;
        let mut abs = 0.0;
        let mut base_sq = 0.0;
        let mut base_abs = 0.0;
        let mut ys = Vec::new();
        let mut preds = Vec::new();

        for c in &self.cases {
            let pred = self.predict_excluding(&c.features, k_eff, Some(&c.id))?;
            // baseline: mean of the OTHER cases, so the comparison uses the same information
            let others: Vec<f64> = self
                .cases
                .iter()
                .filter(|o| o.id != c.id)
                .map(|o| o.outcome)
                .collect();
            let base = others.iter().sum::<f64>() / others.len() as f64;

            let e = pred - c.outcome;
            sq += e * e;
            abs += e.abs();
            let eb = base - c.outcome;
            base_sq += eb * eb;
            base_abs += eb.abs();
            ys.push(c.outcome);
            preds.push(pred);
        }

        let rmse = crate::spatial_reasoning::sqrt(sq / n as f64);
        let baseline_rmse = crate::spatial_reasoning::sqrt(base_sq / n as f64);
        let ybar = ys.iter().sum::<f64>() / n as f64;
        let ss_tot: f64 = ys.iter().map(|y| (y - ybar) * (y - ybar)).sum();
        let ss_res: f64 = ys.iter().zip(preds.iter()).map(|(y, p)| (y - p) * (y - p)).sum();
        Some(LooResult {
            n,
            rmse,
            mae: abs / n as f64,
            r2: if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 },
            baseline_rmse,
            // the mean predictor's R² is 0 by construction - it is the definition of the baseline. Written as
            // a constant because computing 1 - ss_base/ss_base would only ever produce the same 0.0.
            baseline_r2: 0.0,
            k: k_eff,
            worse_than_baseline: rmse >= baseline_rmse,
        })
    }

    /// The statistics used for standardisation, so a report can show what "near" means here.
    pub fn stats(&self) -> (Vec<f64>, Vec<f64>) {
        (self.mean.clone(), self.std.clone())
    }
}
