//! TRM Swarm OS — Temporal Reasoning (no_std, alloc). Task 577.
//!
//! Allen's interval algebra: the 13 relations an interval can bear to another, and **composition** — the
//! question "if A is before B and B overlaps C, what can A be to C?".
//!
//! # Semantics, stated because they are a choice
//!
//! Intervals are **half-open**: `[start, end)`, in seconds since the unix epoch. Half-open is what makes
//! `a.end == b.start` mean *meets* rather than *overlaps by a point*, and it matches how ledger and trial
//! records actually behave (a phase that ends at T and another that starts at T do not happen simultaneously).
//! An interval with `end <= start` is refused by [`Interval::try_new`] rather than silently normalised.
//!
//! # What composition is, and is not
//!
//! [`compose`] answers with the set of relations that are *possible* given the two inputs — the qualitative
//! answer, not a single relation. It is computed by constructing **witness intervals** on a bounded integer
//! grid rather than by a hand-typed 13×13 table: a transcribed table is exactly the kind of artefact that
//! drifts, and 169 cells is enough cells to get one wrong. The grid bound [`WITNESS_WINDOW`] exceeds the number
//! of endpoints any two intervals can have between them (at most 12 distinct orderings), so a relation missing
//! from the result is genuinely impossible, not merely unrealisable on the grid.
//!
//! # What this is NOT
//!
//! It is not a temporal *logic*: there is no metric operator such as "within 24h", no projection ("this will
//! happen"), and no linear-time temporal operator (always/eventually/until). It answers questions about
//! orderings of intervals that already exist, and says so.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Integer grid used when constructing witness intervals for [`compose`].
pub const WITNESS_WINDOW: i64 = 16;

/// A half-open interval `[start, end)` in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval {
    pub start: i64,
    pub end: i64,
}

impl Interval {
    /// Refuses a reversed or empty interval. An interval that ends before it starts is a data error, and
    /// silently swapping the ends would invent an ordering the record does not support.
    pub fn try_new(start: i64, end: i64) -> Option<Interval> {
        if end > start {
            Some(Interval { start, end })
        } else {
            None
        }
    }

    pub fn duration(&self) -> i64 {
        self.end - self.start
    }

    pub fn contains_instant(&self, t: i64) -> bool {
        t >= self.start && t < self.end
    }

    pub fn overlaps_instant(&self, other: &Interval) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// Allen's thirteen relations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Allen {
    Before,
    Meets,
    Overlaps,
    Starts,
    During,
    Finishes,
    Equals,
    FinishedBy,
    Contains,
    StartedBy,
    OverlappedBy,
    MetBy,
    After,
}

impl Allen {
    pub fn as_str(self) -> &'static str {
        match self {
            Allen::Before => "BEFORE",
            Allen::Meets => "MEETS",
            Allen::Overlaps => "OVERLAPS",
            Allen::Starts => "STARTS",
            Allen::During => "DURING",
            Allen::Finishes => "FINISHES",
            Allen::Equals => "EQUALS",
            Allen::FinishedBy => "FINISHED_BY",
            Allen::Contains => "CONTAINS",
            Allen::StartedBy => "STARTED_BY",
            Allen::OverlappedBy => "OVERLAPPED_BY",
            Allen::MetBy => "MET_BY",
            Allen::After => "AFTER",
        }
    }

    pub fn from_str(s: &str) -> Option<Allen> {
        let up = s.trim().to_ascii_uppercase();
        const ALL: [Allen; 13] = [
            Allen::Before, Allen::Meets, Allen::Overlaps, Allen::Starts, Allen::During,
            Allen::Finishes, Allen::Equals, Allen::FinishedBy, Allen::Contains, Allen::StartedBy,
            Allen::OverlappedBy, Allen::MetBy, Allen::After,
        ];
        ALL.iter().copied().find(|a| a.as_str() == up)
    }

    /// The converse relation: if A r B, then B r⁻¹ A.
    pub fn inverse(self) -> Allen {
        match self {
            Allen::Before => Allen::After,
            Allen::Meets => Allen::MetBy,
            Allen::Overlaps => Allen::OverlappedBy,
            Allen::Starts => Allen::StartedBy,
            Allen::During => Allen::Contains,
            Allen::Finishes => Allen::FinishedBy,
            Allen::Equals => Allen::Equals,
            Allen::FinishedBy => Allen::Finishes,
            Allen::Contains => Allen::During,
            Allen::StartedBy => Allen::Starts,
            Allen::OverlappedBy => Allen::Overlaps,
            Allen::MetBy => Allen::Meets,
            Allen::After => Allen::Before,
        }
    }
}

/// Classify `a` against `b`. Exactly one relation holds: the thirteen cases are exhaustive and disjoint.
pub fn relation(a: &Interval, b: &Interval) -> Allen {
    if a.end < b.start {
        Allen::Before
    } else if a.end == b.start {
        Allen::Meets
    } else if b.end < a.start {
        Allen::After
    } else if b.end == a.start {
        Allen::MetBy
    } else if a.start == b.start && a.end == b.end {
        Allen::Equals
    } else if a.start == b.start {
        if a.end < b.end { Allen::Starts } else { Allen::StartedBy }
    } else if a.end == b.end {
        if a.start > b.start { Allen::Finishes } else { Allen::FinishedBy }
    } else if a.start > b.start && a.end < b.end {
        Allen::During
    } else if a.start < b.start && a.end > b.end {
        Allen::Contains
    } else if a.start < b.start {
        // a.start < b.start, a.end < b.end, and they overlap (handled above)
        Allen::Overlaps
    } else {
        Allen::OverlappedBy
    }
}

/// All intervals on the witness grid `[s, e)` with `0 <= s < e <= WITNESS_WINDOW`.
fn witnesses() -> Vec<Interval> {
    let mut out = Vec::new();
    let mut s = 0;
    while s <= WITNESS_WINDOW {
        let mut e = s + 1;
        while e <= WITNESS_WINDOW {
            out.push(Interval { start: s, end: e });
            e += 1;
        }
        s += 1;
    }
    out
}

/// Composition: the set of relations `A` can bear to `C` when `A r1 B` and `B r2 C`.
///
/// Computed by searching for a witness triple (A fixed, B and C varied) rather than by transcribing a table.
/// The returned set is sorted by [`Allen`]'s declaration order so results are stable across builds.
pub fn compose(r1: Allen, r2: Allen) -> Vec<Allen> {
    let mid = Interval { start: WITNESS_WINDOW / 2 - 2, end: WITNESS_WINDOW / 2 + 2 };
    let all = witnesses();
    let mut found: Vec<Allen> = Vec::new();
    for b in &all {
        if relation(&mid, b) != r1 {
            continue;
        }
        for c in &all {
            if relation(b, c) != r2 {
                continue;
            }
            let r = relation(&mid, c);
            if !found.contains(&r) {
                found.push(r);
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Compose a chain of relations left-to-right, returning every relation still possible at the end.
/// A chain that reaches the "everything is possible" set has told you nothing, which is a result too.
pub fn compose_chain(chain: &[Allen]) -> Vec<Allen> {
    let mut current: Vec<Allen> = match chain.first() {
        Some(r) => alloc::vec![*r],
        None => return Vec::new(),
    };
    for r in chain.iter().skip(1) {
        let mut next: Vec<Allen> = Vec::new();
        for a in &current {
            for b in compose(*a, *r) {
                if !next.contains(&b) {
                    next.push(b);
                }
            }
        }
        next.sort();
        next.dedup();
        current = next;
    }
    current
}

/// Named interval, so a report can say which record a relation came from.
#[derive(Debug, Clone)]
pub struct Seeded {
    pub name: String,
    pub interval: Interval,
}

/// Pairwise relation matrix for a list of named intervals, in input order.
pub fn relate_all(items: &[Seeded]) -> Vec<(usize, usize, Allen)> {
    let mut out = Vec::new();
    for i in 0..items.len() {
        for j in 0..items.len() {
            if i == j {
                continue;
            }
            out.push((i, j, relation(&items[i].interval, &items[j].interval)));
        }
    }
    out
}

/// A human-readable report line: "A BEFORE B (a=[0,10) b=[12,20))".
pub fn describe(a: &Seeded, b: &Seeded) -> String {
    let r = relation(&a.interval, &b.interval);
    format!(
        "{} {} {} (a=[{}, {}) b=[{}, {}))",
        a.name, r.as_str(), b.name,
        a.interval.start, a.interval.end, b.interval.start, b.interval.end
    )
}
