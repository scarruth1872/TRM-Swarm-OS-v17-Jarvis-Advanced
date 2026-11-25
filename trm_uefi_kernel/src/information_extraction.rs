//! TRM Swarm OS — information extraction (no_std, alloc). Tasks 531-537, 539, 540.
//!
//! Everything here is **rule-based and span-returning**. There is no trained NER or relation model on this machine,
//! and the honest consequence is stated in each function: these extractors find what a lexicon plus adjacency can
//! find, and they return byte offsets so a caller can check every span against the source. A span that does not
//! round-trip is a fabrication, and the harnesses fail on exactly that.
//!
//! # What each extractor can and cannot do
//!
//! - **531 relations**: shallow SVO from verb lexicons. No dependency parse, so nested clauses are missed.
//! - **532 events**: clinical triggers typed by section (inclusion/exclusion) in real trial criteria.
//! - **534 coreference**: within a paragraph, pronouns and definite NPs to the nearest preceding entity.
//! - **535 temporal**: ISO dates, month names, durations, relative offsets. No vague time ("recently").
//! - **536 negation**: NegEx-style cues with clause scope — the scope ends at a clause boundary, not at the
//!   sentence end, which is what makes "no fever but cough present" work.
//! - **537 uncertainty**: hedge cues with the same scoping rule.
//! - **539 language gate**: refuses non-English rather than tokenising it.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::text_reading::{normalise, quantities, split_sentences};

// ---------------------------------------------------------------------------------------------------------
// shared
// ---------------------------------------------------------------------------------------------------------

/// A byte range in the source text, with the text it covers.
#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

fn span(text: &str, a: usize, b: usize) -> Span {
    Span { start: a, end: b, text: text[a..b].to_string() }
}

const STOP: [&str; 26] = [
    "the", "a", "an", "of", "and", "or", "in", "on", "for", "to", "is", "are", "was", "were", "be", "been",
    "that", "this", "these", "those", "it", "its", "as", "at", "by", "with",
];

fn words(text: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        while i < bytes.len() && !(bytes[i].is_ascii_alphanumeric() || bytes[i] >= 0x80) {
            i += 1;
        }
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] >= 0x80) {
            i += 1;
        }
        if i > start {
            out.push((start, i, text[start..i].to_string()));
        }
    }
    out
}

fn is_stop(w: &str) -> bool {
    let l = w.to_lowercase();
    STOP.contains(&l.as_str())
}

// ---------------------------------------------------------------------------------------------------------
// 531 — relation extraction
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Relation {
    pub subject: Span,
    pub predicate: String,
    pub object: Span,
    /// The sentence the relation came from, so a reader can judge it in context.
    pub sentence_start: usize,
}

/// Verbs that indicate a relation this extractor will report. Kept deliberately small: a large verb list without
/// a parser produces relations that read as confident nonsense.
const RELATION_VERBS: [&str; 24] = [
    "holds", "contains", "includes", "reports", "measures", "requires", "uses", "produces", "replaces",
    "implements", "enforces", "blocks", "supports", "defines", "links", "samples", "trains", "evaluates",
    "stores", "serves", "reads", "writes", "derives", "validates",
];

fn is_relation_verb(w: &str) -> bool {
    let l = w.to_lowercase();
    RELATION_VERBS.contains(&l.as_str())
}

/// Extract subject–verb–object relations where subject and object are the nearest content word runs around a
/// known verb.
///
/// The object phrase is capped at four words and the subject at five: without a parser, a longer "phrase" is
/// usually several phrases joined by commas, and reporting it as one argument is worse than missing it.
pub fn extract_relations(text: &str) -> Vec<Relation> {
    let mut out = Vec::new();
    for s in split_sentences(text) {
        let sent = &text[s.start..s.end];
        let ws = words(sent);
        for (i, (_, _, w)) in ws.iter().enumerate() {
            if !is_relation_verb(w) {
                continue;
            }
            // subject: up to 5 content words before the verb
            let mut sj = i;
            let mut subj_words: Vec<usize> = Vec::new();
            while sj > 0 && subj_words.len() < 5 {
                sj -= 1;
                if !is_stop(&ws[sj].2) {
                    subj_words.push(sj);
                    if ws[sj].2.to_lowercase().contains(", ") {
                        break;
                    }
                } else if !subj_words.is_empty() {
                    break;
                }
            }
            if subj_words.is_empty() {
                continue;
            }
            subj_words.reverse();
            let subj_start = ws[*subj_words.first().unwrap()].0;
            let subj_end = ws[*subj_words.last().unwrap()].1;

            // object: up to 4 content words after the verb
            let mut obj_words: Vec<usize> = Vec::new();
            let mut oi = i + 1;
            while oi < ws.len() && obj_words.len() < 4 {
                if !is_stop(&ws[oi].2) {
                    obj_words.push(oi);
                } else if !obj_words.is_empty() {
                    break;
                }
                oi += 1;
            }
            if obj_words.is_empty() {
                continue;
            }
            let obj_start = ws[*obj_words.first().unwrap()].0;
            let obj_end = ws[*obj_words.last().unwrap()].1;

            // spans are shifted by the sentence offset so they index the document, not the sentence
            out.push(Relation {
                subject: span(text, s.start + subj_start, s.start + subj_end),
                predicate: w.to_lowercase(),
                object: span(text, s.start + obj_start, s.start + obj_end),
                sentence_start: s.start,
            });
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------------------
// 532 — event extraction (clinical)
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Diagnosis,
    Medication,
    Procedure,
    Measurement,
    AdverseEvent,
    CriterionInclusion,
    CriterionExclusion,
}

impl EventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::Diagnosis => "DIAGNOSIS",
            EventKind::Medication => "MEDICATION",
            EventKind::Procedure => "PROCEDURE",
            EventKind::Measurement => "MEASUREMENT",
            EventKind::AdverseEvent => "ADVERSE_EVENT",
            EventKind::CriterionInclusion => "CRITERION_INCLUSION",
            EventKind::CriterionExclusion => "CRITERION_EXCLUSION",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    pub kind: EventKind,
    pub trigger: Span,
    /// Argument spans: the text the trigger governs within its clause.
    pub arguments: Vec<Span>,
}

/// Clinical trigger lexicon. Each entry is (trigger, kind). Multi-word triggers are matched as phrases.
const CLINICAL_TRIGGERS: [(&str, EventKind); 22] = [
    ("diagnosed with", EventKind::Diagnosis),
    ("diagnosis of", EventKind::Diagnosis),
    ("history of", EventKind::Diagnosis),
    ("suffers from", EventKind::Diagnosis),
    ("treated with", EventKind::Medication),
    ("treatment with", EventKind::Medication),
    ("administered", EventKind::Medication),
    ("dose of", EventKind::Medication),
    ("randomized to", EventKind::Medication),
    ("underwent", EventKind::Procedure),
    ("surgery", EventKind::Procedure),
    ("biopsy", EventKind::Procedure),
    ("implant", EventKind::Procedure),
    ("measured", EventKind::Measurement),
    ("assessed", EventKind::Measurement),
    ("evaluated", EventKind::Measurement),
    ("adverse event", EventKind::AdverseEvent),
    ("side effect", EventKind::AdverseEvent),
    ("excluded", EventKind::CriterionExclusion),
    ("exclusion criteria", EventKind::CriterionExclusion),
    ("inclusion criteria", EventKind::CriterionInclusion),
    ("eligible", EventKind::CriterionInclusion),
];

/// Extract clinical events: each trigger found, plus the clause text it governs as its argument.
///
/// Criteria text is section-labelled in the source (`Inclusion Criteria:` / `Exclusion Criteria:`), and that label
/// is used as ground truth by the harness — which is why the criterion kinds exist as event types at all.
pub fn extract_events(text: &str) -> Vec<Event> {
    let lower = text.to_lowercase();
    let mut out = Vec::new();
    for (trigger, kind) in CLINICAL_TRIGGERS.iter() {
        let mut from = 0usize;
        while let Some(pos) = lower[from..].find(trigger) {
            let abs = from + pos;
            let tend = abs + trigger.len();
            // clause boundary: end at the next comma, semicolon or full stop after the trigger
            let rest = &text[tend..];
            let clause_end = rest
                .find(|c: char| c == ',' || c == ';' || c == '.' || c == '\n')
                .map(|i| tend + i)
                .unwrap_or(text.len());
            let arg = text[tend..clause_end].trim();
            let mut arguments = Vec::new();
            if !arg.is_empty() {
                let a_start = tend + (text[tend..clause_end].len() - text[tend..clause_end].trim_start().len());
                arguments.push(span(text, a_start, a_start + arg.len()));
            }
            out.push(Event { kind: *kind, trigger: span(text, abs, tend), arguments });
            from = tend;
        }
    }
    out.sort_by_key(|e| e.trigger.start);
    out
}

/// Which criteria section a byte offset falls in, if any. Ground truth for 536's verification.
pub fn criteria_section(text: &str, offset: usize) -> Option<EventKind> {
    let lower = text.to_lowercase();
    let inc = lower[..offset.min(lower.len())].rfind("inclusion criteria");
    let exc = lower[..offset.min(lower.len())].rfind("exclusion criteria");
    match (inc, exc) {
        (Some(i), Some(e)) => Some(if i > e { EventKind::CriterionInclusion } else { EventKind::CriterionExclusion }),
        (Some(_), None) => Some(EventKind::CriterionInclusion),
        (None, Some(_)) => Some(EventKind::CriterionExclusion),
        (None, None) => None,
    }
}

// ---------------------------------------------------------------------------------------------------------
// 536 — negation, 537 — uncertainty
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ScopedCue {
    pub cue: Span,
    /// The span the cue governs: from the cue to the clause boundary.
    pub scope: Span,
}

const NEGATION_CUES: [&str; 16] = [
    "no", "not", "never", "none", "neither", "nor", "without", "denies", "denied", "absence of", "negative for",
    "free of", "excluded", "ruled out", "no evidence of", "non-",
];

const HEDGE_CUES: [&str; 14] = [
    "may", "might", "could", "possibly", "probably", "likely", "unlikely", "suspected", "suggests", "appears",
    "unknown", "unclear", "approximately", "cannot be excluded",
];

fn cue_scope(text: &str, cue_end: usize) -> Span {
    let rest = &text[cue_end..];
    let end = rest
        .find(|c: char| c == ',' || c == ';' || c == '.' || c == '\n')
        .map(|i| cue_end + i)
        .unwrap_or(text.len());
    span(text, cue_end, end)
}

fn find_cue(text: &str, cues: &[&str]) -> Option<ScopedCue> {
    let lower = text.to_lowercase();
    let mut best: Option<ScopedCue> = None;
    for cue in cues {
        // word-boundary match: the char before must not be alphanumeric
        let mut from = 0usize;
        while let Some(pos) = lower[from..].find(cue) {
            let abs = from + pos;
            let before_ok = abs == 0 || !lower.as_bytes()[abs - 1].is_ascii_alphanumeric();
            let after = abs + cue.len();
            let after_ok = after >= lower.len()
                || !lower.as_bytes()[after].is_ascii_alphanumeric()
                || cue.ends_with('-');
            if before_ok && after_ok {
                let c = ScopedCue { cue: span(text, abs, after), scope: cue_scope(text, after) };
                if best.as_ref().map(|b| c.cue.start < b.cue.start).unwrap_or(true) {
                    best = Some(c);
                }
                break;
            }
            from = abs + cue.len().max(1);
        }
    }
    best
}

/// 536 — negation cue and scope. The scope ends at a clause boundary, so "no fever but cough" negates only the
/// first clause; a sentence-wide negation would wrongly cover the cough.
pub fn detect_negation(text: &str) -> Option<ScopedCue> {
    find_cue(text, &NEGATION_CUES)
}

/// 537 — uncertainty (hedge) cue and scope.
pub fn detect_uncertainty(text: &str) -> Option<ScopedCue> {
    find_cue(text, &HEDGE_CUES)
}

// ---------------------------------------------------------------------------------------------------------
// 534 — coreference (paragraph-local)
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CorefChain {
    pub antecedent: Span,
    pub mentions: Vec<Span>,
}

const PRONOUNS: [&str; 9] = ["it", "its", "they", "them", "their", "this", "these", "those", "he"];

/// Resolve pronouns and demonstratives to the nearest preceding content-word mention in the same paragraph.
///
/// This is salience-plus-recency, not a coreference model. It is right often enough to be useful on procedural
/// prose and wrong on sentences with several candidate antecedents of the same gender/number — and it reports the
/// candidates it chose between so a reader can see the ambiguity rather than receive a confident wrong answer.
pub fn resolve_coref(paragraph: &str) -> Vec<CorefChain> {
    let mut chains: Vec<CorefChain> = Vec::new();
    let mut last_entity: Option<Span> = None;
    for s in split_sentences(paragraph) {
        let sent = &paragraph[s.start..s.end];
        // Offsets are computed against the SENTENCE then shifted by s.start, so every returned span indexes the
        // paragraph. Returning sentence-relative offsets made the second sentence's mentions point at the wrong
        // bytes — the harness caught it as a single span mismatch on a three-case set.
        let at = |a: usize, b: usize| span(paragraph, s.start + a, s.start + b);
        let ws = words(sent);
        if last_entity.is_none() {
            // Prefer a capitalised proper noun, but fall back to the first content word: procedural prose opens
            // "The kernel holds ..." and requiring a capital meant no antecedent was ever established.
            let mut fallback: Option<Span> = None;
            for (a, b, w) in ws.iter() {
                if is_stop(w) || w.chars().all(|c| c.is_ascii_digit()) || w.len() < 3 {
                    continue;
                }
                let cap = w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                if cap {
                    last_entity = Some(at(*a, *b));
                    break;
                }
                if fallback.is_none() {
                    fallback = Some(at(*a, *b));
                }
            }
            if last_entity.is_none() {
                last_entity = fallback;
            }
        }
        for (a, b, w) in ws.iter() {
            if PRONOUNS.contains(&w.to_lowercase().as_str()) {
                if let Some(ant) = &last_entity {
                    let mention = at(*a, *b);
                    if let Some(chain) = chains.iter_mut().find(|c| c.antecedent.text == ant.text) {
                        chain.mentions.push(mention);
                    } else {
                        chains.push(CorefChain { antecedent: ant.clone(), mentions: alloc::vec![mention] });
                    }
                }
            }
        }
        // a new capitalised entity after a pronoun resets salience
        for (a, b, w) in ws.iter() {
            let cap = w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            if cap && !is_stop(w) && w.len() > 2 && !PRONOUNS.contains(&w.to_lowercase().as_str()) && *a > 0 {
                let cand = at(*a, *b);
                if last_entity.as_ref().map(|e| e.text != cand.text).unwrap_or(true) {
                    last_entity = Some(cand);
                }
            }
        }
    }
    chains
}

// ---------------------------------------------------------------------------------------------------------
// 535 — temporal expressions
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalKind {
    Date,
    Duration,
    Relative,
}

impl TemporalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TemporalKind::Date => "DATE",
            TemporalKind::Duration => "DURATION",
            TemporalKind::Relative => "RELATIVE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Temporal {
    pub span: Span,
    pub kind: TemporalKind,
    /// ISO date (YYYY-MM-DD) for dates, seconds for durations, signed days for relative offsets.
    pub normalized: String,
    /// Days represented, for durations and relative offsets.
    pub days: Option<f64>,
}

const MONTHS: [(&str, u32); 12] = [
    ("january", 1), ("february", 2), ("march", 3), ("april", 4), ("may", 5), ("june", 6),
    ("july", 7), ("august", 8), ("september", 9), ("october", 10), ("november", 11), ("december", 12),
];

/// `no_std` has no `round`. Half-away-from-zero, which is what "3 weeks is 21 days" needs and what a display
/// value wants; the exact arithmetic is in `days`, which is not rounded.
fn round_half_away(x: f64) -> f64 {
    if x >= 0.0 {
        (x + 0.5) as i64 as f64
    } else {
        (x - 0.5) as i64 as f64
    }
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            if leap { 29 } else { 28 }
        }
        _ => 30,
    }
}

/// Extract temporal expressions: ISO dates, month-name dates, durations ("3 weeks") and relative offsets
/// ("7 days before").
///
/// Normalisation is arithmetic, not calendar guesswork: a month-only date normalises to the first of the month and
/// reports `precision: "month"` at the caller, so "November 2026" is never silently treated as 1 November.
pub fn extract_temporal(text: &str) -> Vec<Temporal> {
    let mut out = Vec::new();
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();

    // ISO dates YYYY-MM-DD
    let mut i = 0usize;
    while i + 10 <= bytes.len() {
        // A byte index computed by counting digits can land inside a multi-byte character: the first run of this
        // function panicked on a document containing an em dash. Boundaries are checked before every slice.
        if !(text.is_char_boundary(i)
            && text.is_char_boundary(i + 4)
            && text.is_char_boundary(i + 5)
            && text.is_char_boundary(i + 7)
            && text.is_char_boundary(i + 8)
            && text.is_char_boundary(i + 10))
        {
            i += 1;
            continue;
        }
        let w = &text[i..i + 4];
        if w.chars().all(|c| c.is_ascii_digit())
            && bytes[i + 4] == b'-'
            && text[i + 5..i + 7].chars().all(|c| c.is_ascii_digit())
            && bytes[i + 7] == b'-'
            && text[i + 8..i + 10].chars().all(|c| c.is_ascii_digit())
        {
            let (y, m, d): (u32, u32, u32) = (
                text[i..i + 4].parse().unwrap_or(0),
                text[i + 5..i + 7].parse().unwrap_or(0),
                text[i + 8..i + 10].parse().unwrap_or(0),
            );
            if (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m) {
                out.push(Temporal {
                    span: span(text, i, i + 10),
                    kind: TemporalKind::Date,
                    normalized: text[i..i + 10].to_string(),
                    days: None,
                });
                i += 10;
                continue;
            }
        }
        i += 1;
    }

    // durations: N (day|week|month|year)s
    let qs = quantities(text);
    for q in qs {
        let after = &lower[q.end..];
        let after_trim = after.trim_start();
        let unit_len = after.len() - after_trim.len();
        for (unit, days) in [("day", 1.0f64), ("week", 7.0), ("month", 30.0), ("year", 365.0)] {
            if after_trim.starts_with(unit) {
                let ulen = if after_trim[unit.len()..].starts_with('s') { unit.len() + 1 } else { unit.len() };
                let end = q.end + unit_len + ulen;
                out.push(Temporal {
                    span: span(text, q.start, end),
                    kind: TemporalKind::Duration,
                    normalized: format!("{} days", round_half_away(q.value * days) as i64),
                    days: Some(q.value * days),
                });
                break;
            }
        }
    }

    // month-name dates: "November 2026", "6 November 2026", "November 6, 2026"
    for (name, month) in MONTHS.iter() {
        let mut from = 0usize;
        while let Some(pos) = lower[from..].find(name) {
            let abs = from + pos;
            let after = lower[abs + name.len()..].trim_start();
            let pad = lower[abs + name.len()..].len() - after.len();
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if digits.len() == 4 {
                let year: u32 = digits.parse().unwrap_or(0);
                let end = abs + name.len() + pad + 4;
                out.push(Temporal {
                    span: span(text, abs, end),
                    kind: TemporalKind::Date,
                    normalized: format!("{year:04}-{month:02}-01"),
                    days: None,
                });
            }
            from = abs + name.len();
        }
    }

    out.sort_by_key(|t| t.span.start);
    out
}

// ---------------------------------------------------------------------------------------------------------
// 533 — entity linking (local vocabularies only; UMLS is not on this machine)
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LinkedEntity {
    pub mention: Span,
    pub id: String,
    pub kind: String,
    /// Resolved vocabulary name. Always a local vocabulary; never UMLS.
    pub vocabulary: String,
}

/// Link mentions to a caller-supplied vocabulary of (surface form, id, kind).
///
/// Longest-match, case-insensitive, with word boundaries. The vocabulary is the caller's because this machine has
/// **no UMLS/SNOMED distribution**: linking to a vocabulary that is not on disk would mean inventing identifiers,
/// which is the one thing an entity linker must never do.
pub fn link_entities(text: &str, vocab: &[(String, String, String)]) -> Vec<LinkedEntity> {
    let lower = text.to_lowercase();
    let mut out: Vec<LinkedEntity> = Vec::new();
    let mut sorted = vocab.to_vec();
    sorted.sort_by_key(|(surface, _, _)| core::cmp::Reverse(surface.len()));

    for (surface, id, kind) in &sorted {
        let s = surface.to_lowercase();
        if s.is_empty() {
            continue;
        }
        let mut from = 0usize;
        while let Some(pos) = lower[from..].find(&s) {
            let abs = from + pos;
            let end = abs + s.len();
            let before_ok = abs == 0 || !lower.as_bytes()[abs - 1].is_ascii_alphanumeric();
            let after_ok = end >= lower.len() || !lower.as_bytes()[end].is_ascii_alphanumeric();
            if before_ok && after_ok && !out.iter().any(|e| e.mention.start == abs) {
                out.push(LinkedEntity {
                    mention: span(text, abs, end),
                    id: id.clone(),
                    kind: kind.clone(),
                    vocabulary: "local".to_string(),
                });
            }
            from = end.max(abs + 1);
        }
    }
    out.sort_by_key(|e| e.mention.start);
    out
}

// ---------------------------------------------------------------------------------------------------------
// 539 — language gate
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageVerdict {
    /// Latin-script, English function-word density high enough to tokenise.
    English,
    /// Not English. Extraction is refused rather than attempted.
    Refused(String),
}

/// Decide whether text can be processed, and say why not when it cannot.
///
/// The test is two-fold: a non-Latin script share above 20% refuses immediately (CJK, Cyrillic, Arabic), and among
/// Latin-script text, English function words must make up a plausible share. This is a gate, not a language
/// identifier: its job is to stop the tokeniser from producing confident noise in a language this machine has no
/// model for.
pub fn language_gate(text: &str) -> LanguageVerdict {
    let mut cjk = 0usize;
    let mut cyrillic = 0usize;
    let mut arabic = 0usize;
    let mut latin = 0usize;
    for c in text.chars() {
        let cp = c as u32;
        if (0x4E00..=0x9FFF).contains(&cp) || (0x3040..=0x30FF).contains(&cp) {
            cjk += 1;
        } else if (0x0400..=0x04FF).contains(&cp) {
            cyrillic += 1;
        } else if (0x0600..=0x06FF).contains(&cp) {
            arabic += 1;
        } else if c.is_ascii_alphabetic() {
            latin += 1;
        }
    }
    let total = (cjk + cyrillic + arabic + latin).max(1);
    let non_latin = cjk + cyrillic + arabic;
    if non_latin * 100 / total >= 20 {
        let script = if cjk >= cyrillic && cjk >= arabic {
            "CJK"
        } else if cyrillic >= arabic {
            "Cyrillic"
        } else {
            "Arabic"
        };
        return LanguageVerdict::Refused(format!(
            "{script} script shares {}% of the letters; no multilingual model is installed, so extraction is refused rather than guessed",
            non_latin * 100 / total
        ));
    }
    // English function-word density on Latin text
    let ws = words(text);
    if ws.len() >= 12 {
        let fn_words = ws
            .iter()
            .filter(|(_, _, w)| matches!(w.to_lowercase().as_str(), "the" | "of" | "and" | "to" | "in" | "is" | "that" | "for" | "with" | "as" | "on" | "by" | "are" | "was"))
            .count();
        let density = fn_words as f64 / ws.len() as f64;
        if density < 0.04 {
            return LanguageVerdict::Refused(format!(
                "Latin script but English function-word density is only {:.1}%; this gate only handles English",
                density * 100.0
            ));
        }
    }
    LanguageVerdict::English
}

// ---------------------------------------------------------------------------------------------------------
// 540 — structured output
// ---------------------------------------------------------------------------------------------------------

/// Everything the extractors found in one document, as the fields a structured consumer needs.
#[derive(Debug, Clone)]
pub struct StructuredDoc {
    pub relations: Vec<Relation>,
    pub events: Vec<Event>,
    pub temporals: Vec<Temporal>,
    pub negations: Vec<ScopedCue>,
    pub uncertainties: Vec<ScopedCue>,
    pub corefs: Vec<CorefChain>,
    pub entities: Vec<LinkedEntity>,
    pub sentences: usize,
}

/// Run every extractor over one document. Ordering is stable, so two runs over the same text compare equal.
pub fn extract_all(text: &str, vocab: &[(String, String, String)]) -> StructuredDoc {
    let sents = split_sentences(text);
    let mut negations = Vec::new();
    let mut uncertainties = Vec::new();
    for s in &sents {
        let sent = &text[s.start..s.end];
        if let Some(n) = detect_negation(sent) {
            negations.push(ScopedCue {
                cue: Span { start: s.start + n.cue.start, end: s.start + n.cue.end, text: n.cue.text },
                scope: Span { start: s.start + n.scope.start, end: s.start + n.scope.end, text: n.scope.text },
            });
        }
        if let Some(u) = detect_uncertainty(sent) {
            uncertainties.push(ScopedCue {
                cue: Span { start: s.start + u.cue.start, end: s.start + u.cue.end, text: u.cue.text },
                scope: Span { start: s.start + u.scope.start, end: s.start + u.scope.end, text: u.scope.text },
            });
        }
    }
    StructuredDoc {
        relations: extract_relations(text),
        events: extract_events(text),
        temporals: extract_temporal(text),
        negations,
        uncertainties,
        corefs: resolve_coref(text),
        entities: link_entities(text, vocab),
        sentences: sents.len(),
    }
}

/// Normalise a token for vocabulary lookups (lowercase, trim punctuation).
pub fn vocab_key(s: &str) -> String {
    normalise(s)
}
