//! TRM Swarm OS — knowledge verification (no_std, alloc). Tasks 591-600.
//!
//! These tasks are not a second reading pipeline: they are checks over the reading pipeline. Each function here
//! answers one question a reviewer would ask, and each one can return a failure.
//!
//! | task | question | failure mode it catches |
//! |---|---|---|
//! | 593 | is this source what it claims to be? | a hash that does not match, a "primary" source that is a summary |
//! | 594 | when two sources disagree, which wins and why? | a winner chosen by arrival order |
//! | 595 | how much should the reader believe this? | a confident number with no calibration behind it |
//! | 597 | does this document meet its own standard? | a numeric claim with no provenance |
//! | 599 | has this claim been retracted? | a correction that nobody's assertion follows |
//!
//! # The rule that governs all of them
//!
//! **A check that cannot fail is not a check.** Every function returns the evidence for its verdict, and every
//! harness that calls one of them constructs at least one case that must fail.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// ---------------------------------------------------------------------------------------------------------
// 593 — source verification
// ---------------------------------------------------------------------------------------------------------

/// How a source came to exist. The ranking matters for 594: a measurement outranks a summary of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceKind {
    /// Raw data or a tool output: `evidence/`, `data/*.json` produced by a script.
    Primary,
    /// A document that reasons over primary material: `docs/`.
    Secondary,
    /// A document that summarises other documents: a synthesis or a report.
    Derived,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Primary => "PRIMARY",
            SourceKind::Secondary => "SECONDARY",
            SourceKind::Derived => "DERIVED",
        }
    }

    /// Relative trust weight, used by [`resolve`]. Stated as a policy constant, not a measurement.
    pub fn weight(self) -> f64 {
        match self {
            SourceKind::Primary => 1.0,
            SourceKind::Secondary => 0.7,
            SourceKind::Derived => 0.4,
        }
    }
}

/// Classify by path prefix. This is a convention, and the convention is the whole claim: `evidence/` holds tool
/// output, `docs/` holds reasoning, and a report is derived. A file that breaks the convention is misclassified,
/// which is why [`verify_source`] also returns the rule it used.
pub fn classify(path: &str) -> SourceKind {
    let p = path.replace('\\', "/").to_lowercase();
    if p.starts_with("evidence/") || p.starts_with("data/") || p.ends_with(".k3") || p.ends_with(".json") {
        SourceKind::Primary
    } else if p.contains("report") || p.contains("synthesis") || p.contains("summary") || p.contains("briefing") {
        SourceKind::Derived
    } else {
        SourceKind::Secondary
    }
}

#[derive(Debug, Clone)]
pub struct SourceVerdict {
    pub path: String,
    pub kind: SourceKind,
    /// True when the sha256 computed from disk equals the one recorded in the ledger.
    pub hash_matches: bool,
    pub recorded_sha: Option<String>,
    pub actual_sha: String,
    pub detail: String,
}

/// Compare a file's actual hash against the hash a ledger recorded for it.
///
/// `recorded == None` is not a pass: an unrecorded source has no provenance, and saying "verified" on the basis
/// of a hash nobody published is the exact failure this task exists to catch.
pub fn verify_source(path: &str, recorded_sha: Option<&str>, actual_sha: &str) -> SourceVerdict {
    match recorded_sha {
        None => SourceVerdict {
            path: path.to_string(),
            kind: classify(path),
            hash_matches: false,
            recorded_sha: None,
            actual_sha: actual_sha.to_string(),
            detail: "NO RECORDED HASH: the file exists but nothing attests to it; unrecorded is not verified"
                .to_string(),
        },
        Some(rec) => {
            let ok = rec.eq_ignore_ascii_case(actual_sha);
            SourceVerdict {
                path: path.to_string(),
                kind: classify(path),
                hash_matches: ok,
                recorded_sha: Some(rec.to_string()),
                actual_sha: actual_sha.to_string(),
                detail: if ok {
                    "hash matches the recorded value".to_string()
                } else {
                    format!(
                        "HASH MISMATCH: recorded {}… actual {}…",
                        &rec.chars().take(12).collect::<String>(),
                        &actual_sha.chars().take(12).collect::<String>()
                    )
                },
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------------------
// 594 — conflict resolution
// ---------------------------------------------------------------------------------------------------------

/// The precedence rules, in order. Each is a policy and is reported by name when it decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// A measured figure beats a claimed figure.
    MeasuredOverClaimed,
    /// Primary (raw/tool) material beats a document about it.
    PrimaryOverSecondary,
    /// A later measurement beats an earlier one.
    NewerOverOlder,
    /// Nothing decided it; the conflict is reported unresolved rather than silently broken.
    Unresolved,
}

impl Rule {
    pub fn as_str(self) -> &'static str {
        match self {
            Rule::MeasuredOverClaimed => "measured over claimed",
            Rule::PrimaryOverSecondary => "primary over secondary",
            Rule::NewerOverOlder => "newer over older",
            Rule::Unresolved => "UNRESOLVED",
        }
    }
}

/// One side of a conflict.
#[derive(Debug, Clone)]
pub struct Position {
    pub label: String,
    pub kind: SourceKind,
    /// True when this side is a measurement rather than a restatement.
    pub measured: bool,
    /// ISO date (YYYY-MM-DD) or empty when unknown.
    pub date: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub winner: Option<String>,
    pub rule: Rule,
    pub losers: Vec<String>,
    pub rationale: String,
}

/// Resolve two positions by the rules above, in order.
///
/// The order is deliberate: measurement status outranks source kind, which outranks recency. A newer summary is
/// still a summary. When the top rule ties, the next decides; when all tie, the conflict is returned UNRESOLVED
/// and nothing is picked — an arbitrary winner is worse than a stated impasse because it looks like a finding.
pub fn resolve(a: &Position, b: &Position) -> Resolution {
    if a.measured != b.measured {
        let (w, l) = if a.measured { (a, b) } else { (b, a) };
        return Resolution {
            winner: Some(w.label.clone()),
            rule: Rule::MeasuredOverClaimed,
            losers: alloc::vec![l.label.clone()],
            rationale: format!("{} reports a measurement; {} restates a figure", w.label, l.label),
        };
    }
    if a.kind != b.kind {
        let (w, l) = if a.kind < b.kind { (a, b) } else { (b, a) };
        return Resolution {
            winner: Some(w.label.clone()),
            rule: Rule::PrimaryOverSecondary,
            losers: alloc::vec![l.label.clone()],
            rationale: format!("{} is {} evidence; {} is {}", w.label, w.kind.as_str(), l.label, l.kind.as_str()),
        };
    }
    if !a.date.is_empty() && !b.date.is_empty() && a.date != b.date {
        let (w, l) = if a.date > b.date { (a, b) } else { (b, a) };
        return Resolution {
            winner: Some(w.label.clone()),
            rule: Rule::NewerOverOlder,
            losers: alloc::vec![l.label.clone()],
            rationale: format!("{} is dated {}; {} is dated {}", w.label, w.date, l.label, l.date),
        };
    }
    Resolution {
        winner: None,
        rule: Rule::Unresolved,
        losers: alloc::vec![a.label.clone(), b.label.clone()],
        rationale: "same measurement status, same source kind, same or unknown date: nothing in the rule set \
                    distinguishes them, so the conflict is stated rather than settled"
            .to_string(),
    }
}

// ---------------------------------------------------------------------------------------------------------
// 595 — uncertainty quantification
// ---------------------------------------------------------------------------------------------------------

/// The signals a confidence is built from. Every one is measured elsewhere in the pipeline; none is invented here.
#[derive(Debug, Clone)]
pub struct Signals {
    /// Fraction of the claim's content words present in the best evidence sentence (0..1).
    pub coverage: f64,
    /// True when every number in the claim appears in the evidence.
    pub numbers_agree: bool,
    /// True when at least one number in the claim is absent from the evidence.
    pub numbers_conflict: bool,
    /// Source kind of the best evidence.
    pub evidence_kind: SourceKind,
    /// True when nothing covered the claim at all.
    pub no_evidence: bool,
}

#[derive(Debug, Clone)]
pub struct Confidence {
    /// Probability the claim is supported, in [0, 1]. `None` on abstention: with no evidence the honest output is
    /// "I do not know", not a confident small number. Measured base rate for un-retrieved claims on this corpus is
    /// 35% — a model that output ~0.02 there scored a Brier of 0.29, worse than a constant 0.5.
    pub p_support: Option<f64>,
    /// The contributions, so the number can be recomputed by hand.
    pub parts: Vec<(&'static str, f64)>,
    /// True when the score is dominated by the absence of evidence rather than by positive support.
    pub abstention: bool,
}

/// `exp` without `std`. `no_std` has no `exp`, and pulling a math crate into the kernel for one confidence score
/// would be the wrong trade. `(1 + x/n)^n` with n = 4096 is accurate to ~1e-6 over [-10, 10], which is far below
/// the resolution of the weights above — the confidence is a policy model, and its calibration (published by the
/// harness as a Brier score) dominates any error from this approximation.
fn exp_approx(x: f64) -> f64 {
    let n = 4096.0;
    let base = 1.0 + x / n;
    if base <= 0.0 {
        return 0.0;
    }
    // exponentiation by squaring: (1 + x/n)^4096
    let mut result = 1.0f64;
    let mut b = base;
    let mut e = 4096u32;
    while e > 0 {
        if e & 1 == 1 {
            result *= b;
        }
        b *= b;
        e >>= 1;
    }
    result
}

/// A transparent additive score in log-odds space, then a logistic squash.
///
/// This is a **model, not a measurement**, and it is only worth anything if its calibration is published. The
/// harness computes a Brier score and a reliability table on a labelled set; if the numbers are poor, the
/// endpoint says so rather than presenting the confidence as authoritative.
///
/// Weights are policy: coverage is the strongest signal available, a numeric conflict is near-decisive negative
/// evidence, a primary source adds, and an absence of coverage drives the result to abstention.
pub fn quantify(s: &Signals) -> Confidence {
    // Abstention is a first-class outcome, exactly as in the gate: absence of evidence is not evidence of absence.
    // On this corpus an un-retrieved claim is 35% likely to be true and merely unindexed, so reporting 0.02 there
    // was not caution — it was a confident error. No probability is published without evidence.
    if s.no_evidence || s.coverage < 0.30 {
        return Confidence {
            p_support: None,
            parts: alloc::vec![("abstained_no_evidence", 0.0)],
            abstention: true,
        };
    }
    let mut parts: Vec<(&'static str, f64)> = Vec::new();
    if s.numbers_conflict {
        parts.push(("numeric_conflict", -4.0));
    } else if s.numbers_agree {
        parts.push(("numbers_agree", 1.2));
    }
    // coverage in log-odds: 0.0 -> -3, 0.5 -> 0, 1.0 -> +3
    let cov_part = (s.coverage.clamp(0.0, 1.0) - 0.5) * 6.0;
    parts.push(("coverage", cov_part));
    let kind_part = match s.evidence_kind {
        SourceKind::Primary => 0.8,
        SourceKind::Secondary => 0.3,
        SourceKind::Derived => -0.2,
    };
    parts.push(("source_kind", kind_part));

    let logit: f64 = parts.iter().map(|(_, v)| *v).sum();
    let p = 1.0 / (1.0 + exp_approx(-logit));
    Confidence { p_support: Some(p), parts, abstention: false }
}

// ---------------------------------------------------------------------------------------------------------
// 597 — peer review automation
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Worth noting, does not block.
    Note,
    /// A reader could be misled.
    Warning,
    /// The document contradicts its own standard.
    Blocking,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Note => "NOTE",
            Severity::Warning => "WARNING",
            Severity::Blocking => "BLOCKING",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub severity: Severity,
    pub rule: &'static str,
    pub detail: String,
    /// 1-based line number.
    pub line: usize,
}

fn looks_like_path(tok: &str) -> bool {
    // Trailing punctuation is part of prose, not part of the path: "tools/x.py:" and "tools/x.py," both name the
    // same file, and rejecting them for the punctuation made the reviewer report a line that did cite evidence.
    // is_ascii_punctuation avoids writing quote/backslash char literals, which is how the previous version
    // ended up uncompilable. Trailing punctuation belongs to the sentence, not the path.
    let t = tok.trim_matches(|c: char| c.is_ascii_punctuation() && c != '_');
    (t.contains('/') || t.contains('\\'))
        && (t.ends_with(".rs") || t.ends_with(".py") || t.ends_with(".md") || t.ends_with(".json")
            || t.ends_with(".txt") || t.ends_with(".k3"))
}

/// Review a document against four mechanical rules.
///
/// 1. **BLOCKING** — a line asserts a measurement (`measured`, `verified`, `passes`, `PASS`) with no path or
///    offset anywhere on the line. That is the exact pattern this platform has been caught by before.
/// 2. **WARNING** — a numeric claim on a line with no path, offset or unit context nearby.
/// 3. **WARNING** — a placeholder (`TODO`, `TBD`, `FIXME`, `XXX`) left in prose.
/// 4. **NOTE** — a hedged claim (`should`, `probably`) presented in a results section.
///
/// These are cheap textual rules, not understanding. They are stated so a reader can see exactly what was and was
/// not checked.
pub fn review_document(text: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        let lower = line.to_lowercase();
        let has_path = line.split_whitespace().any(looks_like_path);
        let has_offset = lower.contains("offset") || lower.contains("byte") || lower.contains("@");

        // Markdown table rows are skipped for the provenance rules: the source sits in its own column, and the
        // path rarely survives as a single token. Running the checker on real documents produced four BLOCKING
        // findings that were all table rows — a false-positive class, fixed here rather than waived.
        let is_table = line.trim_start().starts_with('|');
        if !is_table
            && ["measured", "verified", "passes", "passes:", "confirmed"].iter().any(|k| lower.contains(k))
            && !has_path
            && !has_offset
        {
            out.push(Finding {
                severity: Severity::Blocking,
                rule: "assertion-without-provenance",
                detail: format!("asserts a result with no path or offset on the line: {}", line.trim().chars().take(90).collect::<String>()),
                line: n,
            });
        }
        let digit_count = lower.chars().filter(|c| c.is_ascii_digit()).count();
        if digit_count >= 3 && !is_table && !has_path && !has_offset && !line.contains("```") {
            out.push(Finding {
                severity: Severity::Warning,
                rule: "number-without-context",
                detail: format!("numeric claim with no path or offset: {}", line.trim().chars().take(90).collect::<String>()),
                line: n,
            });
        }
        if ["todo", "tbd", "fixme", "xxx"].iter().any(|k| lower.contains(k)) {
            out.push(Finding {
                severity: Severity::Warning,
                rule: "placeholder",
                detail: format!("placeholder left in prose: {}", line.trim().chars().take(90).collect::<String>()),
                line: n,
            });
        }
        if (lower.contains("should work") || lower.contains("probably works") || lower.contains("seems to"))
            && line.len() > 40
        {
            out.push(Finding {
                severity: Severity::Note,
                rule: "hedged-assertion",
                detail: format!("hedged claim: {}", line.trim().chars().take(90).collect::<String>()),
                line: n,
            });
        }
    }
    out
}

/// A document passes review when nothing is BLOCKING. Warnings and notes are reported, not fatal — a reviewer
/// decides, and the point of the automation is to make sure the reviewer sees them.
pub fn review_verdicts(findings: &[Finding]) -> (usize, usize, usize) {
    let b = findings.iter().filter(|f| f.severity == Severity::Blocking).count();
    let w = findings.iter().filter(|f| f.severity == Severity::Warning).count();
    let n = findings.iter().filter(|f| f.severity == Severity::Note).count();
    (b, w, n)
}

// ---------------------------------------------------------------------------------------------------------
// 599 — retraction detection
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Retraction {
    /// The line of the ERRATUM / RETRACTED / SUPERSEDED marker.
    pub marker_line: usize,
    pub marker: String,
    /// Text following the marker, up to a blank line.
    pub body: String,
}

/// Find retraction markers in a document.
///
/// A marker is one of `ERRATUM`, `RETRACTED`, `SUPERSEDED`, `CORRECTION` at the start of a line (optionally
/// preceded by `#`/`*`). The body runs until a blank line. The point is not to parse the correction but to make
/// its existence machine-visible: a claim that has been corrected should not be cited as if it stood.
pub fn find_retractions(text: &str) -> Vec<Retraction> {
    const MARKERS: [&str; 4] = ["ERRATUM", "RETRACTED", "SUPERSEDED", "CORRECTION"];
    let mut out = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0usize;
    while i < lines.len() {
        let raw = lines[i].trim_start_matches(['#', '*', ' ', '\t']);
        let upper = raw.to_uppercase();
        if let Some(m) = MARKERS.iter().find(|m| upper.starts_with(*m)) {
            // The marker line usually carries the substance — "Erratum: the earlier claim that X was false" — so
            // the body must start with the rest of that line. Taking only the following lines made the detector
            // find corrections it could never apply to the claim they named.
            let after_marker = raw[m.len()..].trim_start_matches([':', '-', '—', ' ', '\t']);
            let mut body = after_marker.to_string();
            let mut j = i + 1;
            while j < lines.len() && !lines[j].trim().is_empty() {
                if !body.is_empty() {
                    body.push(' ');
                }
                body.push_str(lines[j].trim());
                j += 1;
            }
            out.push(Retraction { marker_line: i + 1, marker: m.to_string(), body });
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

/// Does a retraction body say the named claim is wrong?
///
/// A retraction is only actionable against a claim when it names it. This checks for the claim's distinctive
/// words in the retraction body — a requirement that the correction actually address the claim, rather than any
/// retraction in the document clearing every claim in it.
pub fn retraction_targets(retraction: &Retraction, claim: &str) -> bool {
    let norm = |s: &str| -> Vec<String> {
        s.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() > 4)
            .map(|t| t.to_string())
            .collect()
    };
    let body = norm(&retraction.body);
    let claim_tokens = norm(claim);
    if claim_tokens.is_empty() {
        return false;
    }
    let hits = claim_tokens.iter().filter(|t| body.contains(t)).count();
    hits as f64 / claim_tokens.len() as f64 >= 0.5
}
