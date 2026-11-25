//! TRM Swarm OS — claim extraction, verification and contradiction detection (no_std, alloc).
//! Tasks 585, 587, 589, enacted together (12/2, 12/2, 13/1).
//!
//! # What this is, precisely
//!
//! A **bag-of-content-words model with numeric and polarity comparison**. It is not syntactic parsing: there is
//! no dependency structure, no coreference, and no semantics beyond "the same words in the same order of
//! importance, with equal or different numbers". That limit is stated here because a claim checker that
//! overstates its reading is worse than none: it would mark a claim SUPPORTED on lexical overlap alone and the
//! reader would never know.
//!
//! # The three verdicts, and why UNKNOWN is first-class
//!
//! `SUPPORTED`, `REFUTED`, `UNKNOWN`. A two-way checker must guess when the corpus is silent; that guess is the
//! most damaging thing it can do, because "no evidence" and "counter-evidence" look identical to the reader.
//! UNKNOWN is returned when nothing in the evidence set is about the same thing, and the harness scores
//! abstention rather than penalising it.
//!
//! # Numeric disagreement is the strongest signal available
//!
//! When a claim and a sentence talk about the same thing but carry different numbers, that is a REFUTATION with
//! a concrete reason (`2,403` against `558`), not a matter of degree. Everything else — polarity, hedging — is
//! weaker and is reported as such.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::text_reading::{normalise, quantities};

const STOP: [&str; 30] = [
    "the", "a", "an", "of", "and", "or", "in", "on", "for", "to", "is", "are", "was", "were", "be", "been",
    "that", "this", "these", "those", "it", "its", "as", "at", "by", "with", "from", "than", "then", "but",
];

fn content_tokens(text: &str) -> Vec<String> {
    normalise(text)
        .split(|c: char| !c.is_alphanumeric() && c != '.' && c != '%' && c != '-')
        .filter(|t| t.len() > 1 && !STOP.contains(t))
        .map(|t| t.to_string())
        .collect()
}

/// Overlap between two token sets: |intersection| / |claim tokens|. Asymmetric on purpose — a long evidence
/// sentence covering a short claim should still count as being about the claim.
/// Tokens with pure numbers removed. Numbers are compared separately: leaving them in the overlap inflates
/// coverage and makes two sentences about different quantities look like the same statement.
fn content_tokens_without_numbers(text: &str) -> Vec<String> {
    content_tokens(text)
        .into_iter()
        .filter(|t| !t.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ','))
        .collect()
}

fn coverage(claim: &[String], evidence: &[String]) -> f64 {
    if claim.is_empty() {
        return 0.0;
    }
    let hit = claim.iter().filter(|t| evidence.contains(t)).count();
    hit as f64 / claim.len() as f64
}

fn is_negated(text: &str) -> bool {
    let n = normalise(text);
    ["not ", "no ", "never ", "none", "cannot", "without", "n't"]
        .iter()
        .any(|m| n.contains(m))
}

/// Rough hedging detection: a claim that says "may" or "approximately" is not refuted by a different number.
fn is_hedged(text: &str) -> bool {
    let n = normalise(text);
    ["may", "might", "could", "approximately", "about", "roughly", "estimated", "up to", "at most", "at least"]
        .iter()
        .any(|m| n.contains(m))
}

/// A structured claim extracted from one sentence.
#[derive(Debug, Clone)]
pub struct Claim {
    pub text: String,
    pub tokens: Vec<String>,
    /// (value, unit) pairs, in order of appearance.
    pub numbers: Vec<(f64, String)>,
    pub negated: bool,
    pub hedged: bool,
    /// The quantity text exactly as written, so a report can quote it rather than a re-formatted float.
    pub numbers_literal: Vec<String>,
}

/// Task 587 — extract a claim from a sentence. Returns `None` when the sentence carries no content words.
pub fn extract(sentence: &str) -> Option<Claim> {
    let tokens = content_tokens(sentence);
    if tokens.is_empty() {
        return None;
    }
    let qs = quantities(sentence);
    Some(Claim {
        text: sentence.trim().to_string(),
        tokens,
        numbers: qs.iter().map(|q| (q.value, q.unit.clone())).collect(),
        negated: is_negated(sentence),
        hedged: is_hedged(sentence),
        numbers_literal: qs.iter().map(|q| q.text.clone()).collect(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Supported,
    Refuted,
    Unknown,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Supported => "SUPPORTED",
            Verdict::Refuted => "REFUTED",
            Verdict::Unknown => "UNKNOWN",
        }
    }
}

/// Outcome of checking one claim against one evidence sentence.
#[derive(Debug, Clone)]
pub struct Check {
    pub verdict: Verdict,
    pub evidence_index: Option<usize>,
    pub reason: String,
    /// How much of the claim the evidence covered, so the reader can see the decision's margin.
    pub coverage: f64,
}

/// Minimum coverage for evidence to count as "about the claim".
pub const MIN_COVERAGE: f64 = 0.60;

/// Task 585 — verify a claim against a set of evidence sentences.
///
/// Order of decision, and each branch says why:
///   1. no evidence covers the claim  -> UNKNOWN  (silence is not refutation)
///   2. numeric disagreement          -> REFUTED  (unless the claim is hedged)
///   3. polarity disagreement         -> REFUTED
///   4. otherwise the best-covering sentence -> SUPPORTED
pub fn verify(claim_text: &str, evidence: &[String]) -> Check {
    let Some(claim) = extract(claim_text) else {
        return Check {
            verdict: Verdict::Unknown,
            evidence_index: None,
            reason: "the claim has no content words to check".to_string(),
            coverage: 0.0,
        };
    };

    let mut best: Option<(usize, f64)> = None;
    let mut numeric_conflict: Option<(usize, f64, String, String)> = None;
    let mut polarity_conflict: Option<(usize, f64)> = None;

    for (i, ev) in evidence.iter().enumerate() {
        let ev_tokens = content_tokens_without_numbers(ev);
        let claim_tokens = content_tokens_without_numbers(&claim.text);
        let cov = coverage(&claim_tokens, &ev_tokens);
        if cov < MIN_COVERAGE {
            continue;
        }
        if best.map(|(_, b)| cov > b).unwrap_or(true) {
            best = Some((i, cov));
        }
        let ev_q = quantities(ev);
        // numeric disagreement, when the claim asserts a number and the evidence asserts a different one
        // EVERY number the claim asserts must appear in the evidence. Accepting any one shared number was the
        // first rule tried, and it let "558 chunks from 191 files" pass as SUPPORTED against evidence saying
        // "2,403 chunks from 191 files" because 191 matched.
        if !claim.numbers.is_empty() && !ev_q.is_empty() && !claim.hedged {
            let ev_vals: Vec<f64> = ev_q.iter().map(|q| q.value).collect();
            let missing: Option<(&String, f64)> = claim
                .numbers_literal
                .iter()
                .zip(claim.numbers.iter())
                .map(|(lit, (v, _))| (lit, *v))
                .find(|(_, v)| !ev_vals.iter().any(|e| (v - e).abs() < 1e-9));
            if let Some((lit, _)) = missing {
                if numeric_conflict.is_none() {
                    let ev_lit = ev_q
                        .iter()
                        .map(|q| q.text.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    numeric_conflict = Some((i, cov, lit.clone(), ev_lit));
                }
            }
        }
        // polarity disagreement on the same subject
        if claim.negated != is_negated(ev) && polarity_conflict.is_none() {
            polarity_conflict = Some((i, cov));
        }
    }

    if best.is_none() {
        return Check {
            verdict: Verdict::Unknown,
            evidence_index: None,
            reason: format!(
                "no evidence sentence covered the claim above {:.0}% — silence is not refutation",
                MIN_COVERAGE * 100.0
            ),
            coverage: 0.0,
        };
    }

    if let Some((i, cov, claim_lit, ev_lit)) = numeric_conflict {
        return Check {
            verdict: Verdict::Refuted,
            evidence_index: Some(i),
            reason: format!("numeric disagreement: claim says {claim_lit}, evidence says {ev_lit}"),
            coverage: cov,
        };
    }
    if let Some((i, cov)) = polarity_conflict {
        return Check {
            verdict: Verdict::Refuted,
            evidence_index: Some(i),
            reason: "polarity disagreement: one is negated and the other is not".to_string(),
            coverage: cov,
        };
    }
    let (i, cov) = best.expect("checked non-empty");
    Check {
        verdict: Verdict::Supported,
        evidence_index: Some(i),
        reason: format!("evidence covers {:.0}% of the claim's content words with no numeric or polarity conflict", cov * 100.0),
        coverage: cov,
    }
}

/// Kind of contradiction found between two sentences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    Numeric,
    Polarity,
}

impl ConflictKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ConflictKind::Numeric => "NUMERIC",
            ConflictKind::Polarity => "POLARITY",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Contradiction {
    pub kind: ConflictKind,
    pub coverage: f64,
    pub detail: String,
}

/// Task 589 — do two sentences contradict each other?
///
/// Requires that they be **about the same thing** (coverage both ways above [`MIN_COVERAGE`]) before any conflict
/// is claimed. Two sentences that merely differ are not a contradiction, and reporting them as one is how a
/// contradiction detector produces noise that a reviewer learns to ignore.
pub fn contradicts(a: &str, b: &str) -> Option<Contradiction> {
    let ta = content_tokens_without_numbers(a);
    let tb = content_tokens_without_numbers(b);
    if ta.is_empty() || tb.is_empty() {
        return None;
    }
    let cov_ab = coverage(&ta, &tb);
    let cov_ba = coverage(&tb, &ta);
    if cov_ab.min(cov_ba) < MIN_COVERAGE {
        return None;
    }

    let qa = quantities(a);
    let qb = quantities(b);
    if !qa.is_empty() && !qb.is_empty() {
        // compare the PRIMARY quantity: sentences carry several numbers, and demanding that all of them match
        // would make any two sentences about the same table a contradiction.
        let primary_a = qa[0].value;
        let primary_b = qb[0].value;
        let same = (primary_a - primary_b).abs() < 1e-9;
        if !same && !is_hedged(a) && !is_hedged(b) {
            return Some(Contradiction {
                kind: ConflictKind::Numeric,
                coverage: cov_ab.min(cov_ba),
                detail: format!(
                    "same subject, different numbers: {} vs {}",
                    qa.first().map(|q| q.text.clone()).unwrap_or_default(),
                    qb.first().map(|q| q.text.clone()).unwrap_or_default()
                ),
            });
        }
    }

    if is_negated(a) != is_negated(b) {
        return Some(Contradiction {
            kind: ConflictKind::Polarity,
            coverage: cov_ab.min(cov_ba),
            detail: "same subject, opposite polarity".to_string(),
        });
    }
    None
}
